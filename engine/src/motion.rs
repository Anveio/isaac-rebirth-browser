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

pub(crate) const WORLD_SCALE: f64 = 0.65;
const WORLD_ORIGIN_X: f64 = 32.0;
const WORLD_ORIGIN_Y: f64 = -46.0;
const FORWARD_RETENTION: f32 = 0.880_340_8;
const SIDEWAYS_RETENTION: f32 = 0.787_400_8;
const ACCELERATION: f32 = 0.6;

pub fn advance(player: &mut Player, room: &Room, dx: i32, dy: i32) {
    let mut vx = (player.vx / WORLD_SCALE) as f32;
    let mut vy = (player.vy / WORLD_SCALE) as f32;
    let world_x = (player.x - WORLD_ORIGIN_X) / WORLD_SCALE;
    let world_y = (player.y - WORLD_ORIGIN_Y) / WORLD_SCALE;
    player.x = (f64::from(world_x as f32 + vx) * WORLD_SCALE + WORLD_ORIGIN_X)
        .clamp(room.left, room.right);
    player.y = (f64::from(world_y as f32 + vy) * WORLD_SCALE + WORLD_ORIGIN_Y)
        .clamp(room.top, room.bottom);
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
        let next_forward = (forward + acceleration) * FORWARD_RETENTION;
        vx = next_forward * ux + sideways_x * SIDEWAYS_RETENTION;
        vy = next_forward * uy + sideways_y * SIDEWAYS_RETENTION;
    } else {
        vx *= FORWARD_RETENTION;
        vy *= FORWARD_RETENTION;
    }
    player.vx = f64::from(vx) * WORLD_SCALE;
    player.vy = f64::from(vy) * WORLD_SCALE;
}
