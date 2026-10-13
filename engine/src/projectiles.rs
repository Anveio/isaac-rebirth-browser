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
    fn sign(self) -> f32 {
        match self {
            Self::Negative => -1.0,
            Self::Positive => 1.0,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt::uint_to_uniform;

    #[test]
    fn native_damage_scaled_collision_radius_preserves_rounding() {
        assert_eq!(
            collision_radius(3.5, 8.0).unwrap().to_bits(),
            8.164_994_f32.to_bits()
        );
    }

    #[test]
    fn independent_off_wall_range_keeps_two_dying_presentations() {
        let velocity = WorldVector { x: 10.0, y: 0.0 };
        let origin = eye_origin(
            WorldVector {
                x: 188.266_98,
                y: 400.0,
            },
            velocity,
            Eye::Negative,
            uint_to_uniform(1109439887),
        )
        .unwrap();
        let mut tear = TearMotion::spawn(
            origin,
            velocity,
            Parameters::FRESH_ISAAC,
            uint_to_uniform(3200849611),
        )
        .unwrap();
        tear.present_birth(Phase::Update).unwrap();
        for age in 1..=50 {
            tear.advance(
                if age % 2 == 1 {
                    Phase::Interpolate
                } else {
                    Phase::Update
                },
                false,
            )
            .unwrap();
        }
        assert_eq!(tear.height.to_bits(), (-4.913_714_4_f32).to_bits());
        assert_eq!(
            tear.position,
            WorldVector {
                x: 448.266_97,
                y: 396.483_37
            }
        );
        tear.advance(Phase::Interpolate, false).unwrap();
        assert_eq!(tear.advance(Phase::Update, false).unwrap(), Lifetime::Dying);
        assert_eq!(tear.presented_position.x, 458.266_97);
        assert_eq!(
            tear.advance(Phase::Interpolate, false).unwrap(),
            Lifetime::Dying
        );
        assert_eq!(tear.presented_position.x, 463.266_97);
        assert_eq!(
            tear.advance(Phase::Update, false).unwrap(),
            Lifetime::Removed
        );
        assert_eq!(tear.presented_position.x, 458.266_97);
        assert_eq!(tear.render_offset_y.to_bits(), (-0.746_782_3_f32).to_bits());
    }

    #[test]
    fn overflowed_step_does_not_mutate_motion() {
        let mut tear = TearMotion::spawn(
            WorldVector { x: 0.0, y: 0.0 },
            WorldVector { x: 1.0, y: 0.0 },
            Parameters::FRESH_ISAAC,
            0.5,
        )
        .unwrap();
        tear.velocity.x = f32::MAX;
        tear.position.x = f32::MAX;
        let before = tear;
        assert!(tear.advance(Phase::Update, false).is_err());
        assert_eq!(tear, before);
    }

    #[test]
    fn native_five_directional_and_range_captures_match_215_exact_phases() {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Input {
            player_world: WorldVector,
            stored_velocity: WorldVector,
            eye: Eye,
            eye_uniform: f32,
            final_falling_uniform: f32,
            parameters: Parameters,
            birth_phase: Phase,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Sample {
            phase: Phase,
            position: WorldVector,
            presented_position: WorldVector,
            height: f32,
            falling_speed: f32,
            falling_acceleration: f32,
            render_offset_y: f32,
        }
        #[derive(Deserialize)]
        struct Case {
            name: String,
            input: Input,
            samples: Vec<Sample>,
        }
        #[derive(Deserialize)]
        struct Fixture {
            schema: u32,
            cases: Vec<Case>,
        }
        let fixture: Fixture = serde_json::from_str(include_str!(
            "../../references/isaac/foundation/physics/native-projectiles.json"
        ))
        .unwrap();
        assert_eq!(fixture.schema, 1);
        let mut compared = 0;
        for case in fixture.cases {
            let input = case.input;
            let origin = eye_origin(
                input.player_world,
                input.stored_velocity,
                input.eye,
                input.eye_uniform,
            )
            .unwrap();
            let mut tear = TearMotion::spawn(
                origin,
                input.stored_velocity,
                input.parameters,
                input.final_falling_uniform,
            )
            .unwrap();
            tear.present_birth(input.birth_phase).unwrap();
            for (index, sample) in case.samples.iter().enumerate() {
                if index != 0 {
                    tear.advance(sample.phase, false).unwrap();
                }
                assert_eq!(
                    tear.position, sample.position,
                    "{} phase{} ground",
                    case.name, index
                );
                assert_eq!(
                    tear.presented_position, sample.presented_position,
                    "{} phase{} presentation",
                    case.name, index
                );
                assert_eq!(
                    [
                        tear.height.to_bits(),
                        tear.falling_speed.to_bits(),
                        tear.falling_acceleration.to_bits(),
                        tear.render_offset_y.to_bits()
                    ],
                    [
                        sample.height.to_bits(),
                        sample.falling_speed.to_bits(),
                        sample.falling_acceleration.to_bits(),
                        sample.render_offset_y.to_bits()
                    ],
                    "{} phase{} height",
                    case.name,
                    index
                );
                compared += 1;
            }
        }
        assert_eq!(compared, 215);
    }
}
