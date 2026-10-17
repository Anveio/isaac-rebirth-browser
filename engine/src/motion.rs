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
use std::sync::OnceLock;

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
pub(crate) const WORLD_ORIGIN_X: f64 = 32.0;
pub(crate) const WORLD_ORIGIN_Y: f64 = -46.0;
const FORWARD_RETENTION: f32 = 0.880_340_8;
// The source damps the perpendicular component first, then the recombined
// velocity. Precombining these factors loses a native f32 rounding step.
const TRANSVERSE_RETENTION: f32 = 0.894_427_2;

/// Player control runs in both full and intermediate phases. Grid/contact
/// solvers operate on the returned native f32 state before projection.
pub fn advance_world(
    position: WorldVector,
    velocity: WorldVector,
    dx: i32,
    dy: i32,
    move_speed: f32,
) -> Result<(WorldVector, WorldVector), String> {
    if !(-1..=1).contains(&dx)
        || !(-1..=1).contains(&dy)
        || !position.is_finite()
        || !velocity.is_finite()
        || !move_speed.is_finite()
        || !(0.1..=2.0).contains(&move_speed)
    {
        return Err("player motion requires finite world vectors and digital input signs".into());
    }
    let position = position + velocity;
    let mut vx = velocity.x;
    let mut vy = velocity.y;
    if dx != 0 || dy != 0 {
        let length = ((dx * dx + dy * dy) as f32).sqrt();
        // Source529801..529835 scales input before its repeated dots.
        let input_scale = 0.25_f32 * move_speed + 0.75_f32;
        let ux = (dx as f32 / length) * input_scale;
        let uy = (dy as f32 / length) * input_scale;
        let forward = vx * ux + vy * uy;
        let sideways_x = vx - forward * ux;
        let sideways_y = vy - forward * uy;
        let adjusted_x = forward * ux + sideways_x * TRANSVERSE_RETENTION;
        let adjusted_y = forward * uy + sideways_y * TRANSVERSE_RETENTION;
        // Source 0x529926..0x529990 repeats the dot after transverse damping,
        // then rounds the -0.2, +3, *0.4 and input *0.5 operations separately.
        // Collapsing this to 0.6 + max(-forward*0.04, 0) loses reverse control
        // precision immediately after independently observed spike knockback.
        let adjusted_forward = adjusted_x * ux + adjusted_y * uy;
        let acceleration = ((adjusted_forward * -0.2).max(0.0) / move_speed + 3.0) * 0.4;
        vx = (adjusted_x + (ux * 0.5) * acceleration) * FORWARD_RETENTION;
        vy = (adjusted_y + (uy * 0.5) * acceleration) * FORWARD_RETENTION;
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

/// Ordinary Full manager callbacks defer their grid response until after
/// entity contacts. This keeps the same movement arithmetic and projection.
pub(crate) fn advance_without_grid(
    player: &mut Player,
    dx: i32,
    dy: i32,
    move_speed: f32,
) -> Result<(), String> {
    let position = WorldVector {
        x: ((player.x - WORLD_ORIGIN_X) / WORLD_SCALE) as f32,
        y: ((player.y - WORLD_ORIGIN_Y) / WORLD_SCALE) as f32,
    };
    let velocity = WorldVector {
        x: (player.vx / WORLD_SCALE) as f32,
        y: (player.vy / WORLD_SCALE) as f32,
    };
    let (position, velocity) = advance_world(position, velocity, dx, dy, move_speed)?;
    player.x = f64::from(position.x) * WORLD_SCALE + WORLD_ORIGIN_X;
    player.y = f64::from(position.y) * WORLD_SCALE + WORLD_ORIGIN_Y;
    player.vx = f64::from(velocity.x) * WORLD_SCALE;
    player.vy = f64::from(velocity.y) * WORLD_SCALE;
    Ok(())
}

pub fn advance(player: &mut Player, room: &Room, dx: i32, dy: i32, move_speed: f32) {
    let grid = player_grid(room);
    advance_in_grid(player, room, &grid, player_collider(), dx, dy, move_speed)
        .expect("gameplay input and world vectors are bounded");
}

fn player_grid(room: &Room) -> crate::grid::Grid {
    room.grid
        .clone()
        .map(crate::grid::Grid::new)
        .transpose()
        .expect("admitted room grid retains bounded dimensions")
        .unwrap_or_else(|| crate::traversal::doorway_grid(room))
}

fn player_collider() -> &'static crate::grid::Collider {
    static COLLIDER: OnceLock<crate::grid::Collider> = OnceLock::new();
    COLLIDER.get_or_init(|| {
        crate::grid::Collider::new(crate::grid::CollisionGeometry {
            radius: 10.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: 40,
        })
        .expect("authored Isaac collision geometry is valid")
    })
}

/// Player443d00(false), without Base motion or ordinary controls. The
/// manager queries the supported Isaac class5 and sample list after contacts,
/// including a dead Player with zero or contact-written velocity.
pub(crate) fn resolve_player_grid(player: &mut Player, room: &Room) {
    if player.grid_collision_class == Some(0) {
        return;
    }
    let grid = player_grid(room);
    let mut position = WorldVector {
        x: ((player.x - WORLD_ORIGIN_X) / WORLD_SCALE) as f32,
        y: ((player.y - WORLD_ORIGIN_Y) / WORLD_SCALE) as f32,
    };
    let mut velocity = WorldVector {
        x: (player.vx / WORLD_SCALE) as f32,
        y: (player.vy / WORLD_SCALE) as f32,
    };
    player_collider()
        .resolve_player(&grid, &mut position, &mut velocity, false)
        .expect("admitted player grid vectors are bounded");
    player.x = f64::from(position.x) * WORLD_SCALE + WORLD_ORIGIN_X;
    player.y = f64::from(position.y) * WORLD_SCALE + WORLD_ORIGIN_Y;
    player.vx = f64::from(velocity.x) * WORLD_SCALE;
    player.vy = f64::from(velocity.y) * WORLD_SCALE;
}

/// Full and intermediate player motion share the source ground-contact
/// solver. Room admission owns the grid and cached collision geometry.
pub fn advance_in_grid(
    player: &mut Player,
    _room: &Room,
    grid: &crate::grid::Grid,
    collider: &crate::grid::Collider,
    dx: i32,
    dy: i32,
    move_speed: f32,
) -> Result<(), String> {
    let position = WorldVector {
        x: ((player.x - WORLD_ORIGIN_X) / WORLD_SCALE) as f32,
        y: ((player.y - WORLD_ORIGIN_Y) / WORLD_SCALE) as f32,
    };
    let velocity = WorldVector {
        x: (player.vx / WORLD_SCALE) as f32,
        y: (player.vy / WORLD_SCALE) as f32,
    };
    let (mut position, mut velocity) = advance_world(position, velocity, dx, dy, move_speed)?;
    collider.resolve_player(grid, &mut position, &mut velocity, false)?;
    player.x = f64::from(position.x) * WORLD_SCALE + WORLD_ORIGIN_X;
    player.y = f64::from(position.y) * WORLD_SCALE + WORLD_ORIGIN_Y;

    player.vx = f64::from(velocity.x) * WORLD_SCALE;
    player.vy = f64::from(velocity.y) * WORLD_SCALE;
    Ok(())
}
