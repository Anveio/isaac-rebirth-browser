//! Immutable values consumed at actual regular-tear birth.
//! Random samples, actor allocation, input/charge, and lifecycle stay backend-owned.
use crate::{player_stats::PlayerStats, projectiles};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RegularTearPlan {
    pub(crate) shot_speed: f32,
    pub(crate) damage: f32,
    pub(crate) parameters: projectiles::Parameters,
    pub(crate) scale: f32,
    pub(crate) animation_tier: u8,
    pub(crate) collision_radius: f32,
    pub(crate) fire_delay: i32,
    pub(crate) head_timer: i32,
}

impl RegularTearPlan {
    pub(crate) fn resolve(stats: PlayerStats, max_fire_delay: i32) -> Result<Self, String> {
        stats.validate()?;
        let scale = projectiles::scale(stats.damage)?;
        Ok(Self {
            shot_speed: stats.shot_speed,
            damage: stats.damage,
            parameters: stats.parameters(),
            scale,
            animation_tier: projectiles::regular_animation_tier(scale)?,
            collision_radius: projectiles::collision_radius(scale, 8.0)?,
            fire_delay: max_fire_delay,
            head_timer: ((max_fire_delay as f32 * 0.45) as i32 + 7).max(11),
        })
    }
}
