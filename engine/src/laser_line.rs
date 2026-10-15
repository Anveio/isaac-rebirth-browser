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
fn scale(a: WorldVector, value: f32) -> WorldVector {
    WorldVector {
        x: a.x * value,
        y: a.y * value,
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
}
