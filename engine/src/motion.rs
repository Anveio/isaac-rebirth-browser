//! Fresh Isaac motion observed independently in native axial/diagonal tapes.
//!
//! The public clock supplies presentation intervals, not native simulation
//! ticks. At its nominal 60 Hz phases, position advances with the previous
//! velocity. Held input accelerates along the requested direction and damps
//! sideways drift more strongly than forward motion; release preserves inertia.
//! Native f32 world-coordinate observations are projected at 0.65 logical
//! pixels per world unit. These constants remain presentation-calibrated until
//! a simulation-clock adapter establishes the original update contract.
use crate::{Player, Room};
use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorldVector {
    pub x: f32,
    pub y: f32,
}

impl WorldVector {
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add for WorldVector {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Mul<f32> for WorldVector {
    type Output = Self;
    fn mul(self, factor: f32) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

pub(crate) const WORLD_SCALE: f64 = 0.65;
const WORLD_ORIGIN_X: f64 = 32.0;
const WORLD_ORIGIN_Y: f64 = -46.0;
const FORWARD_RETENTION: f32 = 0.880_340_8;
// The source damps the perpendicular component first, then the recombined
// velocity. Precombining these factors loses a native f32 rounding step.
const TRANSVERSE_RETENTION: f32 = 0.894_427_2;
const ACCELERATION: f32 = 0.6;

/// Player control runs in both full and intermediate phases. Grid/contact
/// solvers operate on the returned native f32 state before projection.
pub fn advance_world(
    position: WorldVector,
    velocity: WorldVector,
    dx: i32,
    dy: i32,
) -> Result<(WorldVector, WorldVector), String> {
    if !(-1..=1).contains(&dx)
        || !(-1..=1).contains(&dy)
        || !position.is_finite()
        || !velocity.is_finite()
    {
        return Err("player motion requires finite world vectors and digital input signs".into());
    }
    let position = position + velocity;
    let mut vx = velocity.x;
    let mut vy = velocity.y;
    if dx != 0 || dy != 0 {
        let length = ((dx * dx + dy * dy) as f32).sqrt();
        let ux = dx as f32 / length;
        let uy = dy as f32 / length;
        let forward = vx * ux + vy * uy;
        let sideways_x = vx - forward * ux;
        let sideways_y = vy - forward * uy;
        // The original counteracts velocity opposite the requested heading
        // with extra acceleration, independently verified by immediate turns.
        let acceleration = ACCELERATION + (-forward * 0.04).max(0.0);
        let adjusted_x = forward * ux + sideways_x * TRANSVERSE_RETENTION;
        let adjusted_y = forward * uy + sideways_y * TRANSVERSE_RETENTION;
        vx = (adjusted_x + acceleration * ux) * FORWARD_RETENTION;
        vy = (adjusted_y + acceleration * uy) * FORWARD_RETENTION;
    } else {
        vx *= FORWARD_RETENTION;
        vy *= FORWARD_RETENTION;
    }
    let velocity = WorldVector { x: vx, y: vy };
    if !position.is_finite() || !velocity.is_finite() {
        return Err("player motion overflowed world vectors".into());
    }
    Ok((position, velocity))
}

pub fn advance(player: &mut Player, room: &Room, dx: i32, dy: i32) {
    let position = WorldVector {
        x: ((player.x - WORLD_ORIGIN_X) / WORLD_SCALE) as f32,
        y: ((player.y - WORLD_ORIGIN_Y) / WORLD_SCALE) as f32,
    };
    let velocity = WorldVector {
        x: (player.vx / WORLD_SCALE) as f32,
        y: (player.vy / WORLD_SCALE) as f32,
    };
    let (position, velocity) = advance_world(position, velocity, dx, dy)
        .expect("gameplay input and world vectors are bounded");
    player.x = (f64::from(position.x) * WORLD_SCALE + WORLD_ORIGIN_X).clamp(room.left, room.right);
    player.y = (f64::from(position.y) * WORLD_SCALE + WORLD_ORIGIN_Y).clamp(room.top, room.bottom);
    player.vx = f64::from(velocity.x) * WORLD_SCALE;
    player.vy = f64::from(velocity.y) * WORLD_SCALE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_grid_holdout_preserves_two_stage_sideways_rounding() {
        // Native rock-held steps23/24 differ from a precombined retention,
        // despite both phases executing the same source player-control law.
        let position = WorldVector {
            x: 80.0,
            y: 329.999_82,
        };
        let velocity = WorldVector {
            x: 1.437_272_3e-8,
            y: 0.0,
        };
        let (_, next) = advance_world(position, velocity, 0, 1).unwrap();
        assert_eq!(next.x.to_bits(), 1.131_709_3e-8_f32.to_bits());
        let (_, next) = advance_world(position, next, 0, 1).unwrap();
        assert_eq!(next.x.to_bits(), 8.911_087e-9_f32.to_bits());
    }
}
