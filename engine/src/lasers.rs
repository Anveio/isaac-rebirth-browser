//! Entity_Laser (type 7) storage shared with its NPC runtime callbacks.
//! The public model and existing entry points remain in this module.
use crate::motion::WorldVector;
use serde::{Deserialize, Serialize};

pub(crate) use crate::laser_npc_runtime::{admit, LaserBirth};
pub use crate::laser_npc_runtime::{advance_actor, advance_impact, retire};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Laser {
    pub id: u32,
    pub variant: u32,
    pub spawn_seed: u32,
    /// Parent NPC (+498) by room NPC id, and its type for the damage filter.
    pub parent: u32,
    pub parent_type: u32,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    /// +7d4 fixed follow offset from the parent.
    pub follow_offset: WorldVector,
    /// +458 PositionOffset.
    pub position_offset: WorldVector,
    /// +7dc angle in degrees.
    pub angle: f32,
    /// +7e4 render depth offset.
    pub depth: f32,
    /// +7f0 remaining timeout.
    pub timeout: i32,
    /// +7f6 shrinking after the timeout.
    pub shrinking: bool,
    /// +7f8 beam length from pos+offset to the endpoint.
    pub length: f32,
    /// +8a0 clipped endpoint.
    pub endpoint: WorldVector,
    /// Sprite +2c4 scale and +2cc rotation.
    pub scale: WorldVector,
    pub rotation: f32,
    /// +42c Game+ce9f4 at Init.
    pub birth_game_update: u32,
    pub collision_radius: f32,
    pub collision_damage: f32,
    pub actor: String,
    pub animation: String,
    pub animation_clock: f32,
    pub dead: bool,
    pub removed: bool,
    pub impact: Option<Impact>,
}

/// Brimstone Impact 1000.50 attached to the beam end.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Impact {
    pub id: u32,
    pub spawn_seed: u32,
    pub position: WorldVector,
    pub position_offset: WorldVector,
    pub scale: WorldVector,
    pub rotation: f32,
    pub depth: f32,
    pub actor: String,
    pub animation: String,
    pub animation_clock: f32,
    pub animation_playing: bool,
    pub dead: bool,
    pub removed: bool,
}
