//! Source-derived regular tear kinematics in native f32 world units.
//!
//! This kernel receives the global phase and already-routed random values.
//! It does not invent an independent tear RNG or own room collision/effects.
use crate::motion::WorldVector;
use crate::phase::Phase;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Eye {
    Negative,
    Positive,
}

impl Eye {
    pub fn flipped(self) -> Self {
        match self {
            Self::Negative => Self::Positive,
            Self::Positive => Self::Negative,
        }
    }
    fn sign(self) -> f32 {
        match self {
            Self::Negative => -1.0,
            Self::Positive => 1.0,
        }
    }
}

/// Normal Isaac excludes the component moving against the aim, then inherits
/// the remaining native entering velocity with f32 factor1.2. Source51bce6
/// adds that vector to the ten-unit aim before51bd71 scales the whole sum
/// by cached shot speed, including inherited transverse motion.
pub fn stored_velocity(
    aim: WorldVector,
    inherited: WorldVector,
    shot_speed: f32,
) -> Result<WorldVector, String> {
    if !aim.is_finite() || !inherited.is_finite() || !shot_speed.is_finite() || shot_speed < 0.6 {
        return Err("tear velocity requires finite vectors".into());
    }
    let backward = (inherited.x * aim.x + inherited.y * aim.y).min(0.0);
    let velocity = WorldVector {
        x: (aim.x * 10.0 + (inherited.x - aim.x * backward) * 1.2) * shot_speed,
        y: (aim.y * 10.0 + (inherited.y - aim.y * backward) * 1.2) * shot_speed,
    };
    if !velocity.is_finite() {
        return Err("tear velocity overflowed".into());
    }
    Ok(velocity)
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Lifetime {
    Alive,
    Dying,
    Removed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Parameters {
    pub height: f32,
    pub falling_speed: f32,
    pub falling_acceleration: f32,
}

impl Parameters {
    pub const FRESH_ISAAC: Self = Self {
        height: -23.75,
        falling_speed: 0.0,
        falling_acceleration: 0.0,
    };
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TearMotion {
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub velocity: WorldVector,
    pub height: f32,
    pub falling_speed: f32,
    pub falling_acceleration: f32,
    pub render_offset_y: f32,
    pub lifetime: Lifetime,
}

fn validate_uniform(value: f32) -> Result<(), String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err("tear random inputs require a source uniform in 0..=1".into());
    }
    Ok(())
}

/// Eye offset uses effective stored velocity, including movement inheritance.
pub fn eye_origin(
    player: WorldVector,
    velocity: WorldVector,
    eye: Eye,
    uniform: f32,
) -> Result<WorldVector, String> {
    validate_uniform(uniform)?;
    if !player.is_finite() || !velocity.is_finite() {
        return Err("tear origin requires finite vectors".into());
    }
    let coefficient = uniform * 0.2 + 0.3;
    let origin = WorldVector {
        x: player.x - (velocity.y * eye.sign()) * coefficient,
        y: player.y + (velocity.x * eye.sign()) * coefficient,
    };
    if !origin.is_finite() {
        return Err("tear eye offset overflowed".into());
    }
    Ok(origin)
}

pub fn height_offset(height: f32, acceleration: f32) -> f32 {
    if acceleration > 0.001 {
        return height;
    }
    let bend = (height + 8.5).max(0.0);
    height * 0.5 - 15.0 + bend * bend
}

/// Native hit parameters round the linear term to f32 before the double sqrt.
pub fn collision_radius(damage: f32, authored_radius: f32) -> Result<f32, String> {
    if !damage.is_finite() || damage < 0.0 || !authored_radius.is_finite() || authored_radius < 0.0
    {
        return Err("tear collision size requires finite nonnegative parameters".into());
    }
    let linear = damage * 0.04 + 0.6;
    let scale = (f64::from(linear) + f64::from(damage).sqrt() * f64::from(0.15_f32)) as f32;
    let radius = authored_radius * scale;
    if !radius.is_finite() {
        return Err("tear collision radius overflowed".into());
    }
    Ok(radius)
}

impl TearMotion {
    /// FireTear overrides initialized stats, calls Update immediately and
    /// subtracts stored velocity. Preserve the add/add/sub birth rounding.
    /// The caller routes eye, entity seed, discarded initializer randomness,
    /// admitted audio sample and final falling randomness through shared MT.
    pub fn spawn(
        origin: WorldVector,
        velocity: WorldVector,
        parameters: Parameters,
        final_uniform: f32,
    ) -> Result<Self, String> {
        validate_uniform(final_uniform)?;
        let position = origin + velocity;
        let mut motion = Self {
            position,
            presented_position: position,
            velocity,
            height: parameters.height,
            falling_speed: final_uniform * 0.2 - parameters.falling_speed,
            falling_acceleration: parameters.falling_acceleration,
            render_offset_y: 0.0,
            lifetime: Lifetime::Alive,
        };
        if !motion.is_finite() {
            return Err("tear birth requires finite parameters".into());
        }
        motion.position = motion.position + velocity;
        motion.update_height();
        motion.position = motion.position + velocity * -1.0;
        motion.presented_position = motion.position;
        if !motion.is_finite() {
            return Err("tear birth overflowed".into());
        }
        Ok(motion)
    }

    fn is_finite(&self) -> bool {
        self.position.is_finite()
            && self.presented_position.is_finite()
            && self.velocity.is_finite()
            && self.height.is_finite()
            && self.falling_speed.is_finite()
            && self.falling_acceleration.is_finite()
            && self.render_offset_y.is_finite()
    }

    fn update_height(&mut self) {
        self.falling_speed = self.falling_speed * 0.9 + 0.1;
        self.falling_speed += self.falling_acceleration;
        self.height += self.falling_speed;
        self.render_offset_y = height_offset(self.height, self.falling_acceleration);
    }

    fn interpolate(&mut self) {
        self.presented_position = self.position + self.velocity * 0.5;
        if self.lifetime == Lifetime::Alive {
            self.render_offset_y = height_offset(
                self.height + self.falling_speed * 0.5,
                self.falling_acceleration,
            );
        }
    }

    /// Birth already performed its immediate Update. Even-phase birth still
    /// receives the current global intermediate presentation afterward.
    pub fn present_birth(&mut self, phase: Phase) -> Result<(), String> {
        if phase == Phase::Skipped {
            return Err("a tear cannot be born in a skipped phase".into());
        }
        let mut next = *self;
        if phase == Phase::Interpolate {
            next.interpolate();
        }
        if !next.is_finite() {
            return Err("tear birth presentation overflowed".into());
        }
        *self = next;
        Ok(())
    }

    /// Restore transient ground prediction before a full phase or pause entry.
    /// The last render height is retained until the next admitted update.
    pub fn revert_prediction(&mut self) {
        self.presented_position = self.position;
    }

    /// Returns the lifetime transition. Grid collision is an explicit caller
    /// result; this kernel never substitutes projected sprite bounds for it.
    pub fn advance(&mut self, phase: Phase, collides: bool) -> Result<Lifetime, String> {
        let mut next = *self;
        if next.lifetime == Lifetime::Removed || phase == Phase::Skipped {
            return Ok(next.lifetime);
        }
        if phase == Phase::Interpolate {
            next.interpolate();
        } else if next.lifetime == Lifetime::Dying {
            next.lifetime = Lifetime::Removed;
            next.revert_prediction();
        } else {
            next.position = next.position + next.velocity;
            next.presented_position = next.position;
            if next.height > -5.0 || collides {
                next.lifetime = Lifetime::Dying;
            } else {
                next.update_height();
            }
        }
        if !next.is_finite() {
            return Err("tear step overflowed".into());
        }
        *self = next;
        Ok(self.lifetime)
    }
}
