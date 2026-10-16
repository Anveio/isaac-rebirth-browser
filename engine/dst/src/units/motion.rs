//! Player motion (`motion::advance_world`, source 529801..529990) over
//! arbitrary positions, velocities, inputs and speed stats, and over long
//! held-input and release runs.
//!
//! The oracles split each velocity along and across the unit input direction
//! in f64; the source projects onto an input vector scaled by the speed stat
//! instead. Position advances by the previous velocity. Release keeps the
//! forward retention of the velocity. Held input damps drift across it by
//! both retentions and accelerates along it, braking harder against it.
//!
//! Each step moves the velocity geometrically closer to a terminal velocity
//! along the input, so speed never rises past the terminal speed, whichever
//! of the eight directions is held. Every step also commutes exactly with
//! the eight symmetries of the square.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use rebirth_cleanroom::motion::{advance_world, WorldVector};
use serde::Serialize;
use serde_json::json;

/// The source's binary32 retentions: every step's forward retention, and the
/// extra damping of drift across a held input.
const FORWARD: f32 = 0.880_340_8;
const TRANSVERSE: f32 = 0.894_427_2;
/// Steps a settling run holds its input, then releases it.
const HOLD: usize = 600;
const RELEASE: usize = 200;

const DIRECTIONS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

#[derive(Clone, Copy, Serialize)]
struct Step {
    position: WorldVector,
    velocity: WorldVector,
    dx: i32,
    dy: i32,
    move_speed: f32,
}

impl Step {
    fn advance(self) -> Result<(WorldVector, WorldVector), String> {
        advance_world(
            self.position,
            self.velocity,
            self.dx,
            self.dy,
            self.move_speed,
        )
    }

    /// Digital input signs, finite vectors and a speed stat in 0.1..=2.0.
    fn admitted(self) -> bool {
        (-1..=1).contains(&self.dx)
            && (-1..=1).contains(&self.dy)
            && self.position.is_finite()
            && self.velocity.is_finite()
            && (0.1..=2.0).contains(&self.move_speed)
    }
}

/// One of the square's eight symmetries: an optional transpose, then
/// optional reflections of each axis.
#[derive(Clone, Copy)]
struct Symmetry {
    transpose: bool,
    flip_x: bool,
    flip_y: bool,
}

/// The seven symmetries besides the identity.
fn symmetries() -> impl Iterator<Item = Symmetry> {
    (1..8_u8).map(|bits| Symmetry {
        transpose: bits & 1 != 0,
        flip_x: bits & 2 != 0,
        flip_y: bits & 4 != 0,
    })
}

impl Symmetry {
    fn vector(self, v: WorldVector) -> WorldVector {
        let (x, y) = if self.transpose {
            (v.y, v.x)
        } else {
            (v.x, v.y)
        };
        WorldVector {
            x: if self.flip_x { -x } else { x },
            y: if self.flip_y { -y } else { y },
        }
    }

    fn input(self, dx: i32, dy: i32) -> (i32, i32) {
        let (x, y) = if self.transpose { (dy, dx) } else { (dx, dy) };
        (
            if self.flip_x { x.wrapping_neg() } else { x },
            if self.flip_y { y.wrapping_neg() } else { y },
        )
    }

    fn step(self, step: Step) -> Step {
        let (dx, dy) = self.input(step.dx, step.dy);
        Step {
            position: self.vector(step.position),
            velocity: self.vector(step.velocity),
            dx,
            dy,
            ..step
        }
    }
}

/// The f64 model of a held input: its unit direction and the source's input
/// scale s = 0.25·speed + 0.75.
#[derive(Clone, Copy)]
struct Held {
    ux: f64,
    uy: f64,
    move_speed: f64,
    scale: f64,
}

impl Held {
    fn new(dx: i32, dy: i32, move_speed: f32) -> Self {
        let length = f64::from(dx * dx + dy * dy).sqrt();
        let move_speed = f64::from(move_speed);
        Self {
            ux: f64::from(dx) / length,
            uy: f64::from(dy) / length,
            move_speed,
            scale: 0.25 * move_speed + 0.75,
        }
    }

    /// The velocity's component along the input and its part across it.
    fn split(self, v: WorldVector) -> (f64, (f64, f64)) {
        let (vx, vy) = (f64::from(v.x), f64::from(v.y));
        let along = vx * self.ux + vy * self.uy;
        (along, (vx - along * self.ux, vy - along * self.uy))
    }

    /// The retention of the part along the input before forward damping:
    /// projecting onto the scaled input keeps T + (1 - T)·s² of it.
    fn retention(self) -> f64 {
        let transverse = f64::from(TRANSVERSE);
        transverse + (1.0 - transverse) * self.scale * self.scale
    }

    /// The next velocity, split as `split` splits it.
    fn next(self, v: WorldVector) -> (f64, (f64, f64)) {
        let forward = f64::from(FORWARD);
        let across = forward * f64::from(TRANSVERSE);
        let (along, (ax, ay)) = self.split(v);
        let kept = self.retention() * along;
        // Moving against the input raises the acceleration (529926..529990).
        let acceleration = ((-0.2 * kept * self.scale).max(0.0) / self.move_speed + 3.0) * 0.4;
        (
            forward * (kept + 0.5 * self.scale * acceleration),
            (across * ax, across * ay),
        )
    }

    /// Every step is a contraction by R·λ (braking only shortens the part
    /// along the input), toward the fixed point w = R·(λ·w + 0.6·s).
    fn contraction(self) -> f64 {
        f64::from(FORWARD) * self.retention()
    }

    /// The most a step adds: R·0.5·s times the acceleration 1.2 when not
    /// braking.
    fn gain(self) -> f64 {
        f64::from(FORWARD) * 0.6 * self.scale
    }

    fn terminal(self) -> f64 {
        self.gain() / (1.0 - self.contraction())
    }

    /// f32 rounding slack for one step from `speed`: the source's
    /// intermediates stay within a few times the speed, and braking divides
    /// by the speed stat.
    fn slack(self, speed: f64) -> f64 {
        1e-5 * (speed * (3.0 + 0.2 / self.move_speed) + 1.0)
    }
}

/// A step's resulting position and velocity components, if it ran.
fn components<E>(result: &Result<(WorldVector, WorldVector), E>) -> Option<[f32; 4]> {
    result
        .as_ref()
        .ok()
        .map(|(position, velocity)| [position.x, position.y, velocity.x, velocity.y])
}

fn norm(v: WorldVector) -> f64 {
    let (x, y) = (f64::from(v.x), f64::from(v.y));
    (x * x + y * y).sqrt()
}

pub fn run(case: &mut Case) {
    let e = &mut *case.entropy;
    for _ in 0..case.steps.max(256) {
        let step = step(e);
        check(step);
    }
    for _ in 0..(case.steps / 64).max(8) {
        settle(e);
    }
}

fn unit(e: &mut Entropy) -> f32 {
    e.u32() as f32 / u32::MAX as f32
}

fn signed(e: &mut Entropy, magnitude: f32) -> f32 {
    (unit(e) * 2.0 - 1.0) * magnitude
}

/// Mostly a speed stat the engine admits, its edges included, sometimes one
/// just past an edge or invalid.
fn speed_stat(e: &mut Entropy) -> f32 {
    match e.below(16) {
        0 => *e.pick(&[0.1, 2.0, 1.0, 0.85, 1.3]),
        1 => *e.pick(&[
            f32::from_bits(0.1_f32.to_bits() - 1),
            f32::from_bits(2.0_f32.to_bits() + 1),
            0.0,
            -1.0,
            f32::NAN,
            f32::INFINITY,
        ]),
        _ => admitted_speed(e),
    }
}

fn admitted_speed(e: &mut Entropy) -> f32 {
    (0.1 + unit(e) * 1.9).min(2.0)
}

fn input(e: &mut Entropy) -> (i32, i32) {
    let sign = |e: &mut Entropy| e.range(0, 2) as i32 - 1;
    if e.chance(1, 32) {
        let wrong = *e.pick(&[2, -2, 7, i32::MIN, i32::MAX]);
        let other = sign(e);
        if e.chance(1, 2) {
            (wrong, other)
        } else {
            (other, wrong)
        }
    } else {
        (sign(e), sign(e))
    }
}

/// A component at gameplay scale or beyond it, sometimes invalid.
fn component(e: &mut Entropy, gameplay: f32) -> f32 {
    match e.below(32) {
        0 => 0.0,
        1 => -0.0,
        2 => 1e-40,
        3 => f32::NAN,
        4 => f32::INFINITY,
        5 => signed(e, 1e6),
        6 => signed(e, 1e30),
        _ => signed(e, gameplay),
    }
}

fn step(e: &mut Entropy) -> Step {
    let (dx, dy) = input(e);
    if e.chance(1, 32) {
        // Near the top of f32, where a step overflows.
        let sign = if e.chance(1, 2) { 1.0 } else { -1.0 };
        let mut huge = || sign * f32::MAX * (0.5 + unit(e) * 0.5);
        return Step {
            position: WorldVector {
                x: huge(),
                y: huge(),
            },
            velocity: WorldVector {
                x: huge(),
                y: huge(),
            },
            dx,
            dy,
            move_speed: 1.0,
        };
    }
    let zero = e.chance(1, 16);
    Step {
        position: WorldVector {
            x: component(e, 1000.0),
            y: component(e, 1000.0),
        },
        velocity: if zero {
            WorldVector { x: 0.0, y: 0.0 }
        } else {
            WorldVector {
                x: component(e, 15.0),
                y: component(e, 15.0),
            }
        },
        dx,
        dy,
        move_speed: speed_stat(e),
    }
}

fn check(step: Step) {
    let details = || json!({ "step": step });
    let result = step.advance();
    props::always!(
        components(&step.advance()).map(|values| values.map(f32::to_bits))
            == components(&result).map(|values| values.map(f32::to_bits)),
        "motion: a step is a pure function of its inputs",
        details()
    );
    let admitted = step.admitted();
    // Near the top of f32 an intermediate may overflow where the result
    // would not; below this bound neither can.
    let bounded = norm(step.position) + 16.0 * norm(step.velocity) < f64::from(f32::MAX);
    // The admitted domain must run; outside it the engine may refuse or
    // serve the step (a widened domain is not a failure), but never with a
    // non-finite vector.
    props::always!(
        match &result {
            Ok((position, velocity)) => position.is_finite() && velocity.is_finite(),
            Err(_) => !admitted || !bounded,
        },
        "motion: a step runs on digital input, finite vectors and an admitted speed stat, and never returns a non-finite vector",
        details()
    );
    props::sometimes!(
        !admitted,
        "motion: steps outside the admitted domain are tried"
    );
    props::sometimes!(
        admitted && !bounded,
        "motion: steps near the top of f32 are tried"
    );
    for symmetry in symmetries() {
        let image = symmetry.step(step);
        let expected = result
            .as_ref()
            .map(|&(position, velocity)| (symmetry.vector(position), symmetry.vector(velocity)));
        // Equal values: a reflected zero may keep its sign.
        let same = match (components(&image.advance()), components(&expected)) {
            (Some(left), Some(right)) => left
                .iter()
                .zip(right)
                .all(|(&left, right)| left == right || (left.is_nan() && right.is_nan())),
            (left, right) => left.is_none() && right.is_none(),
        };
        props::always!(
            same,
            "motion: a step commutes with the square's symmetries",
            json!({ "step": step, "image": image })
        );
    }
    // The source model covers the admitted domain only.
    let (Ok((position, velocity)), true) = (result, admitted) else {
        return;
    };
    props::always!(
        position.x == step.position.x + step.velocity.x
            && position.y == step.position.y + step.velocity.y,
        "motion: position advances by the previous velocity",
        details()
    );
    if step.dx == 0 && step.dy == 0 {
        props::always!(
            velocity.x == step.velocity.x * FORWARD && velocity.y == step.velocity.y * FORWARD,
            "motion: a released player keeps the forward retention of its velocity",
            details()
        );
        return;
    }
    let held = Held::new(step.dx, step.dy, step.move_speed);
    let speed = norm(step.velocity);
    let slack = held.slack(speed);
    let (along, (ax, ay)) = held.next(step.velocity);
    let (next_along, (nx, ny)) = held.split(velocity);
    props::always!(
        (nx - ax).abs() <= slack && (ny - ay).abs() <= slack,
        "motion: held input damps drift across it by both retentions",
        details()
    );
    props::always!(
        (next_along - along).abs() <= slack,
        "motion: held input accelerates along it, braking harder against it",
        details()
    );
    props::always!(
        norm(velocity) <= held.contraction() * speed + held.gain() + slack,
        "motion: held input contracts speed toward its terminal speed",
        details()
    );
    props::sometimes!(
        held.split(step.velocity).0 < -1.0,
        "motion: held input brakes a player moving against it"
    );
    props::sometimes!(
        speed > held.terminal(),
        "motion: a speed beyond the terminal speed decays toward it"
    );
}

/// Holds one direction long enough to settle, then releases it.
fn settle(e: &mut Entropy) {
    let move_speed = if e.chance(1, 8) {
        *e.pick(&[0.1, 2.0])
    } else {
        admitted_speed(e)
    };
    let (dx, dy) = *e.pick(&DIRECTIONS);
    let held = Held::new(dx, dy, move_speed);
    let reach = match e.below(4) {
        0 => 0.0,
        1 => 1e4,
        _ => 20.0,
    };
    let start = Step {
        position: WorldVector {
            x: signed(e, 1000.0),
            y: signed(e, 1000.0),
        },
        velocity: WorldVector {
            x: signed(e, reach),
            y: signed(e, reach),
        },
        dx,
        dy,
        move_speed,
    };
    let details = || json!({ "start": start });
    let terminal = held.terminal();
    let target = (terminal * held.ux, terminal * held.uy);
    let distance = |v: WorldVector| {
        let (x, y) = (f64::from(v.x) - target.0, f64::from(v.y) - target.1);
        (x * x + y * y).sqrt()
    };
    let mut step = start;
    for _ in 0..HOLD {
        let result = step.advance();
        props::always!(
            result.is_ok(),
            "motion: every step of a settling run is admitted",
            details()
        );
        let Ok((position, velocity)) = result else {
            return;
        };
        // Both split components may round by the single-step slack.
        let bound =
            held.contraction() * distance(step.velocity) + 2.0 * held.slack(norm(step.velocity));
        props::always!(
            distance(velocity) <= bound,
            "motion: held input brings the velocity geometrically closer to its terminal velocity",
            json!({ "start": start, "step": step })
        );
        step.position = position;
        step.velocity = velocity;
    }
    // A settled velocity lies within the rounding the contraction absorbs.
    let settled = 2.0 * held.slack(terminal) / (1.0 - held.contraction());
    props::always!(
        (norm(step.velocity) - terminal).abs() <= settled,
        "motion: a settled player moves at its speed stat's terminal speed, along axes and diagonals alike",
        json!({ "start": start, "settled": step })
    );
    let released = step;
    let mut farthest = norm(step.position);
    for _ in 0..RELEASE {
        step.dx = 0;
        step.dy = 0;
        let result = step.advance();
        props::always!(
            result.is_ok(),
            "motion: every step of a settling run is admitted",
            details()
        );
        let Ok((position, velocity)) = result else {
            return;
        };
        step.position = position;
        step.velocity = velocity;
        farthest = farthest.max(norm(position));
    }
    // Each release step multiplies by R, so the glide is v·(1 - R^n)/(1 - R),
    // within the rounding of n multiplications and n position additions.
    let forward = f64::from(FORWARD);
    let geometric = (1.0 - forward.powi(RELEASE as i32)) / (1.0 - forward);
    let glide = |start: f32, end: f32, velocity: f32| {
        (f64::from(end) - f64::from(start) - f64::from(velocity) * geometric).abs()
    };
    let tolerance =
        1e-5 * norm(released.velocity) + RELEASE as f64 * farthest * f64::from(f32::EPSILON) + 1e-6;
    props::always!(
        glide(released.position.x, step.position.x, released.velocity.x) <= tolerance
            && glide(released.position.y, step.position.y, released.velocity.y) <= tolerance,
        "motion: a released player glides the geometric sum of its velocity",
        json!({ "released": released, "stopped": step })
    );
}
