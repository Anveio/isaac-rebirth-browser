//! Source Line577120/578760/579fa0 state used by the real Laser7 callbacks.
use crate::motion::WorldVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CurvePoint {
    pub position: WorldVector,
    pub incoming: WorldVector,
    pub outgoing: WorldVector,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaserLine {
    pub origin: WorldVector,
    pub offset: WorldVector,
    ///Actual source+14 and+68, not independently inferred visual settings.
    pub subdivisions: Option<u32>,
    pub mode: Option<u32>,
    pub width: Option<f32>,
    ///Actual+70..78 cubic control vector; full rebuild clears this vector.
    pub controls: Vec<CurvePoint>,
    ///Actual+50..58 retains the unsimplified resampling/smoothing history.
    pub history: Vec<WorldVector>,
    ///Actual+20..28, including source578420 point simplification.
    pub points: Vec<WorldVector>,
    ///Actual+38..40 captures points before source578420 changes them.
    pub prior_points: Vec<WorldVector>,
    ///576F50 leaves these scalar words untouched. They become known only
    ///through577120/the actual Laser first callback/579FA0, respectively.
    pub length: Option<f32>,
}

impl Default for LaserLine {
    fn default() -> Self {
        Self {
            origin: WorldVector { x: 0.0, y: 0.0 },
            offset: WorldVector { x: 0.0, y: 0.0 },
            subdivisions: None,
            mode: None,
            width: None,
            controls: Vec::new(),
            history: Vec::new(),
            points: Vec::new(),
            prior_points: Vec::new(),
            length: None,
        }
    }
}

fn add(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}
fn sub(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
fn scale(a: WorldVector, value: f32) -> WorldVector {
    WorldVector {
        x: a.x * value,
        y: a.y * value,
    }
}
fn magnitude(a: WorldVector) -> f32 {
    (a.x * a.x + a.y * a.y).sqrt()
}
fn unit(a: WorldVector) -> WorldVector {
    let length = magnitude(a);
    if length == 0.0 {
        a
    } else {
        scale(a, 1.0 / length)
    }
}

///577020 preserves the scalar coefficient arithmetic and addition order.
pub(crate) fn cubic(a: CurvePoint, b: CurvePoint, t: f32) -> WorldVector {
    let t2 = t * t;
    let opposite = 1.0 - t;
    let opposite2 = opposite * opposite;
    let c0 = opposite2 * opposite;
    let c1 = (opposite2 * 3.0) * t;
    let c2 = (opposite * 3.0) * t2;
    let c3 = t * t2;
    add(
        add(
            add(scale(a.position, c0), scale(a.outgoing, c1)),
            scale(b.incoming, c2),
        ),
        scale(b.position, c3),
    )
}

impl LaserLine {
    ///577120 clears all four logical vectors without fitting prior samples.
    pub(crate) fn initialize(&mut self, subdivisions: u32, mode: u32) {
        self.subdivisions = Some(subdivisions);
        self.mode = Some(mode);
        self.controls.clear();
        self.history.clear();
        self.points.clear();
        self.prior_points.clear();
    }

    ///578270 clears only controls; sampled/history state survives each rebuild.
    pub(crate) fn begin_curve(&mut self) {
        self.controls.clear();
    }

    ///5791F0 creates these two control triples before the mode0 query. Both
    ///middle handles are independently formed from (origin+endpoint)*.5.
    pub(crate) fn straight_controls(&mut self) {
        let middle = scale(add(self.origin, self.offset), 0.5);
        self.controls.push(CurvePoint {
            incoming: self.origin,
            position: self.origin,
            outgoing: middle,
        });
        self.controls.push(CurvePoint {
            incoming: middle,
            position: self.offset,
            outgoing: self.offset,
        });
    }

    /// 578760 forms seven closed-circle controls. Literal handle angles and
    /// radius scaling retain their separate f32/f64 rounding boundaries.
    pub(crate) fn circle(&mut self, center: WorldVector, radius: f32) {
        let handle_radius = 1.05_f32 * radius;
        for index in 0..7_u32 {
            let fraction = index as f32 / 6.0;
            let radians = ((f64::from(fraction) * std::f64::consts::PI) * 2.0) as f32;
            let angle = 0.0 - radians;
            let polar = |at: f32, distance: f32| WorldVector {
                x: crate::fmath::cos(at) * distance,
                y: crate::fmath::sin(at) * distance,
            };
            self.controls.push(CurvePoint {
                incoming: add(center, polar(0.25 + angle, handle_radius)),
                position: add(center, polar(angle, radius)),
                outgoing: add(center, polar(angle - 0.25, handle_radius)),
            });
        }
    }

    ///577D60 uses equal parameter ranges per control pair;579FA0 supplies
    ///an independently computed arc-length remap before calling it.
    fn evaluate(&self, parameter: f32) -> WorldVector {
        let segments = self.controls.len() - 1;
        let scaled = segments as f32 * parameter;
        let index = scaled as i32;
        if index < 0 {
            return self.controls[0].position;
        }
        if index as usize >= segments {
            return self.controls[segments].position;
        }
        cubic(
            self.controls[index as usize],
            self.controls[index as usize + 1],
            scaled - index as f32,
        )
    }

    ///579FA0 estimates each cubic length at ten real parameter steps, then
    ///samples+smooths all subdivisions+1 points. No random demands occur.
    pub(crate) fn resample(&mut self, smoothing: f32) -> Result<(), String> {
        let subdivisions = self
            .subdivisions
            .ok_or("Laser line subdivision owner is not initialized")?;
        let mode = self
            .mode
            .ok_or("Laser line mode owner is not initialized")?;
        if self.controls.len() < 2
            || subdivisions == 0
            || self.controls.iter().any(|point| {
                !point.position.is_finite()
                    || !point.incoming.is_finite()
                    || !point.outgoing.is_finite()
            })
            || !smoothing.is_finite()
        {
            return Err("Laser line requires real finite source control points".into());
        }
        let mut boundaries = vec![0.0_f32];
        self.length = Some(0.0);
        let mut total_length = 0.0;
        for pair in self.controls.windows(2) {
            let mut length = 0.0;
            let mut previous = 0.0;
            for index in 1..=10_u32 {
                let next = index as f32 / 10.0;
                length += magnitude(sub(
                    cubic(pair[0], pair[1], next),
                    cubic(pair[0], pair[1], previous),
                ));
                previous = next;
            }
            boundaries.push(length);
            total_length += length;
        }
        self.length = Some(total_length);
        if total_length != 0.0 {
            boundaries[0] /= total_length;
            for index in 1..boundaries.len() {
                boundaries[index] = (boundaries[index] / total_length) + boundaries[index - 1];
            }
        }
        let mut segment = 0_usize;
        let mut sampled = Vec::with_capacity(subdivisions as usize + 1);
        let segments = (boundaries.len() - 1) as f32;
        for index in 0..=subdivisions {
            let fraction = index as f32 / subdivisions as f32;
            let mut parameter = fraction;
            while segment < boundaries.len() - 1 && fraction > boundaries[segment] {
                let following = segment + 1;
                parameter = segment as f32 / segments
                    + ((fraction - boundaries[segment])
                        / (boundaries[following] - boundaries[segment]))
                        / segments;
                if following == boundaries.len() - 1 || fraction <= boundaries[following] {
                    break;
                }
                segment = following;
            }
            sampled.push(self.evaluate(parameter));
        }
        if sampled.len() != self.history.len() {
            self.history = sampled;
        } else if mode == 1 {
            for (old, new) in self.history.iter_mut().zip(sampled) {
                *old = add(*old, scale(sub(new, *old), smoothing));
            }
        } else {
            //Native42EFC0→42EF50 before the non-loop smoothing reduction.
            let factor = smoothing * (total_length / 200.0).clamp(0.0, 1.0);
            let last = (self.history.len() - 1) as f32;
            for (index, (old, new)) in self.history.iter_mut().zip(sampled).enumerate() {
                let amount = 1.0 - ((index as f32 / last) * factor);
                *old = add(*old, scale(sub(new, *old), amount));
            }
        }
        self.points.clone_from(&self.history);
        Ok(())
    }

    ///578420 removes nearly collinear interior points after preserving+38.
    pub(crate) fn simplify(&mut self, tolerance: f32) {
        self.prior_points.clone_from(&self.points);
        let mut index = 1;
        while index + 1 < self.points.len() {
            let after = unit(sub(self.points[index + 1], self.points[index]));
            let before = unit(sub(self.points[index], self.points[index - 1]));
            if tolerance > (1.0 - (before.x * after.x + before.y * after.y)).abs() {
                self.points.remove(index);
            } else {
                index += 1;
            }
        }
    }
}
