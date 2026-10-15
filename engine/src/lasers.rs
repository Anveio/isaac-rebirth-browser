//! One Entity_Laser (type7) record and allocator shared by its runtime adapters.
//! NPC callback arithmetic stays in laser_npc_runtime; Player producers are separate.
use crate::{
    actors::ActorRef,
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    familiars::{Clip, Definition, Sprite},
    items::Rng,
    laser_line::LaserLine,
    motion::WorldVector,
};
use serde::{Deserialize, Serialize};
const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
const CLIPS: &[Clip] = &[Clip {
    name: "LargeRedLaser",
    frames: 4,
    looping: true,
}];

pub(crate) use crate::laser_npc_runtime::{admit, LaserBirth};
pub use crate::laser_npc_runtime::{advance_actor, advance_impact, retire};

/// Port dispatch provenance; not an inferred native entity word.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LaserOrigin {
    NpcShootAngle,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Laser {
    pub id: u32,
    pub origin: LaserOrigin,
    pub actor: String,
    ///The imported real1000/50 retains its own Effect identity and callback.
    pub impact: Option<Impact>,
    pub entity_type: u32,
    pub variant: u32,
    pub subtype: u32,
    pub spawn_seed: u32,
    pub birth_game_update: u32,
    pub actor_rng: Rng,
    pub spawner_type: u32,
    /// The existing NPC birth adapter supplied only the parent type.
    pub spawner_variant: Option<u32>,
    pub parent: Option<ActorRef>,
    pub child: Option<ActorRef>,
    pub active: bool,
    pub dead: bool,
    pub visible: bool,
    pub position: WorldVector,
    pub saved_position: WorldVector,
    pub render_offset: WorldVector,
    pub velocity: WorldVector,
    pub configured_friction: f32,
    pub current_friction: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub collision_damage: f32,
    /// Native config/HP scaling and later base status are not imported here.
    pub hp: Option<f32>,
    pub max_hp: Option<f32>,
    pub raw_494: bool,
    /// Initial318 is zero; the inherited NPC adapter does not own later status.
    pub entity_flags: Option<u32>,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub attack_flags: AttackFlags,
    ///Permanent ColorMod3A0..3C8 written by virtual58=446F00.
    pub source_color: AttackColor,
    ///Current ColorMod2D0..2F8 copied by virtual88=441C10. The admitted
    ///negative-duration caller has no timed override; these remain distinct
    ///owners even when their actual values match.
    pub current_color: AttackColor,
    pub fast: bool,
    pub sprite: Sprite,
    ///Actual7D4 follow offset, independent of renderOffset458.
    pub parent_offset: WorldVector,
    pub angle: f32,
    /// Init writes zero, but the inherited body omits478C61; keep it unknown.
    pub previous_angle: Option<f32>,
    ///Actual7E4 feeds the renderer sorting key; it is not7F0 lifetime.
    pub render_sort_offset: f32,
    pub remaining_updates: i32,
    pub first_update: bool,
    pub line_enabled: bool,
    pub contact_disabled: bool,
    pub beam_length: f32,
    pub line: LaserLine,
    ///Caller writes818 separately from7E8;476060/475EA0 do not initialize it.
    pub line_flags: Option<AttackFlags>,
    pub rotation_enabled: bool,
    pub max_distance: f32,
    pub line_controller: i32,
    pub endpoint: WorldVector,
}

impl Laser {
    /// Shared476060/475EA0 Init values, with the existing NPC request applied
    /// afterward. Unimported callback/history words stay unknown; no random draw.
    pub(crate) fn initialize_npc(
        id: u32,
        variant: u32,
        seed: u32,
        age: u32,
        config: &crate::content::EntityConfig,
    ) -> Self {
        let sprite_config = Definition {
            entity_type: 7,
            variant,
            actor: config.actor,
            base_hp: config.base_hp,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: config.collision_damage,
            friction: config.friction,
            shadow_size: config.shadow_size,
            grid_points: config.grid_points,
            default_clip: "LargeRedLaser",
            clips: CLIPS,
        };
        Self {
            id,
            origin: LaserOrigin::NpcShootAngle,
            actor: config.actor.into(),
            impact: None,
            entity_type: 7,
            variant,
            subtype: 0,
            spawn_seed: seed,
            birth_game_update: age,
            actor_rng: Rng::new(seed, [4, 3, 17]),
            spawner_type: 0,
            spawner_variant: None,
            parent: None,
            child: None,
            active: true,
            dead: false,
            visible: true,
            position: ZERO,
            saved_position: ZERO,
            render_offset: ZERO,
            velocity: ZERO,
            configured_friction: config.friction,
            current_friction: config.friction,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: config.collision_damage,
            hp: None,
            max_hp: None,
            raw_494: false,
            entity_flags: None,
            grid_collision_class: 3,
            entity_collision_class: 4,
            attack_flags: AttackFlags(0),
            source_color: AttackColor::WHITE,
            current_color: AttackColor::WHITE,
            fast: false,
            sprite: Sprite::new(&sprite_config),
            parent_offset: ZERO,
            angle: 0.0,
            previous_angle: None,
            render_sort_offset: 0.0,
            remaining_updates: 0,
            first_update: true,
            line_enabled: false,
            contact_disabled: false,
            beam_length: 0.0,
            line: LaserLine::default(),
            line_flags: None,
            rotation_enabled: false,
            max_distance: 0.0,
            line_controller: 0,
            endpoint: ZERO,
        }
    }

    /// The old NPC controller owns distinct normal/presented positions. Its
    /// adapter stores them in saved_position/position without a second DTO.
    /// This does not qualify its inherited age or missing-Parent source law.
    pub(crate) fn npc_positions(&self) -> (WorldVector, WorldVector) {
        (self.saved_position, self.position)
    }
    pub(crate) fn set_npc_positions(
        &mut self,
        normal: WorldVector,
        presented: WorldVector,
        interpolating: bool,
    ) {
        self.saved_position = normal;
        self.position = presented;
        self.fast = interpolating;
    }
    pub(crate) fn remove(&mut self) {
        self.active = false;
        self.dead = true;
    }
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
