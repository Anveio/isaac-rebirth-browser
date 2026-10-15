//! One Entity_Laser (type7) record and allocator shared by its runtime adapters.
//! NPC callback arithmetic stays in laser_npc_runtime; Player producers are separate.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    familiars::{Clip, Definition, Sprite},
    grid::Grid,
    items::Rng,
    laser_line::LaserLine,
    motion::WorldVector,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, sync::OnceLock};
const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
const CLIPS: &[Clip] = &[Clip {
    name: "LargeRedLaser",
    frames: 4,
    looping: true,
}];

fn definition() -> &'static Definition {
    static DEFINITION: OnceLock<Definition> = OnceLock::new();
    DEFINITION.get_or_init(|| {
        //449590 resolves the unique7/1 configuration; subtype1 uses its
        //actual subtype0 fallback. Authored subtype does not select a newANM.
        let config = crate::content::entity(7, 1);
        Definition {
            entity_type: 7,
            variant: 1,
            actor: "gfx/007.001_thick red laser.anm2",
            base_hp: config.base_hp,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: config.collision_damage,
            friction: config.friction,
            shadow_size: config.shadow_size,
            grid_points: config.grid_points,
            default_clip: "LargeRedLaser",
            clips: CLIPS,
        }
    })
}

// Actual7/2 authored config has one layer:478416 skips its root displacement.
// It owns RedLaser, not the thick Laser's LargeRedLaser clip or extents.
fn technology_definition() -> &'static Definition {
    static DEFINITION: OnceLock<Definition> = OnceLock::new();
    DEFINITION.get_or_init(|| {
        let config = crate::content::entity(7, 2);
        Definition {
            entity_type: 7,
            variant: 2,
            actor: "gfx/007.002_thin red laser.anm2",
            base_hp: config.base_hp,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: config.collision_damage,
            friction: config.friction,
            shadow_size: config.shadow_size,
            grid_points: config.grid_points,
            default_clip: "RedLaser",
            clips: &[
                Clip {
                    name: "RedLaser",
                    frames: 3,
                    looping: true,
                },
                Clip {
                    name: "RedLaserWavy",
                    frames: 3,
                    looping: true,
                },
            ],
        }
    })
}

pub use crate::laser_npc_runtime::retire;
pub(crate) use crate::laser_npc_runtime::{admit, LaserBirth};
pub use crate::laser_runtime::{advance_actor, advance_impact};

/// Port dispatch provenance; not an inferred native entity word.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LaserOrigin {
    NpcShootAngle,
    PlayerCharged,
    PlayerTechnology,
}
impl LaserOrigin {
    pub(crate) fn is_player(self) -> bool {
        matches!(self, Self::PlayerCharged | Self::PlayerTechnology)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Laser {
    pub id: u32,
    pub origin: LaserOrigin,
    ///Known only for the actual Player full-Uninit owner; NPC legacy adapter stays unknown.
    pub uninitialized: Option<bool>,
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
            uninitialized: None,
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
    #[serde(default)]
    pub source_subtype: Option<u32>,
    #[serde(default)]
    pub sprite_flip_x: Option<bool>,
    ///Actual Effect+814 word, independent of Sprite.rotation. Missing old
    ///history stays unknown; selected Effect50 Full does not copy it to2CC.
    #[serde(default)]
    pub raw_814: Option<f32>,
    ///Actual registered Parent value; false knownness is retained NPC history, not NULL proof.
    pub parent: Option<ActorRef>,
    pub parent_known: bool,
    pub source_color: Option<AttackColor>,
    pub current_color: Option<AttackColor>,
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

///Literal Room1638 query fields; its owning adapter preserves query order.
///444810 reads these fields, independently of health/visibility/active31E.
#[derive(Clone, Copy)]
pub(crate) struct LaserTarget {
    pub(crate) actor: ActorRef,
    pub(crate) entity_type: u32,
    pub(crate) variant: u32,
    pub(crate) entity_collision_class: u32,
    pub(crate) raw_494: bool,
    pub(crate) dead: bool,
    pub(crate) entity_flags: u32,
    pub(crate) position: WorldVector,
    pub(crate) collision_radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LaserContact {
    ///47735B writes31F only, retaining actual Remove/Uninit ownership.
    MarkEffectDead { target: ActorRef },
    ///476760 passes ParentPlayer DamageInfo, flags8 and countdown30.
    Damage {
        target: ActorRef,
        source: ActorRef,
        amount: f32,
    },
}

fn difference(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn squared_length(vector: WorldVector) -> f32 {
    vector.x * vector.x + vector.y * vector.y
}

///577150 returns the source f32 squared distance to the closed segment.
fn segment_distance_squared(point: WorldVector, from: WorldVector, to: WorldVector) -> f32 {
    let edge = difference(to, from);
    let length = squared_length(edge);
    if length == 0.0 {
        return squared_length(difference(point, from));
    }
    let delta = difference(point, from);
    let parameter = (delta.x * edge.x + delta.y * edge.y) / length;
    if parameter < 0.0 {
        return squared_length(delta);
    }
    if parameter > 1.0 {
        return squared_length(difference(point, to));
    }
    squared_length(difference(point, from + edge * parameter))
}

impl Laser {
    pub(crate) fn initialize(id: u32, subtype: u32, seed: u32, age: u32) -> Result<Self, String> {
        if subtype != 0 {
            return Err("ordinary Player Laser requires actual subtype0".into());
        }
        let mut laser = Self::initialize_npc(id, 1, seed, age, crate::content::entity(7, 1));
        laser.origin = LaserOrigin::PlayerCharged;
        laser.uninitialized = Some(false);
        laser.spawner_variant = Some(0);
        laser.entity_flags = Some(0); //441EDE, actual fresh Base.Init318.
        laser.previous_angle = Some(0.0); //475EF8 actual Init7E0.
        Ok(laser)
    }
    ///475EA0/441C80 Init through the actual7/2 config; factory50E1C0 owns
    ///the later lifetime3, grid5 and copied properties. No C18 registration.
    pub(crate) fn initialize_technology(id: u32, seed: u32, age: u32) -> Self {
        let mut laser = Self::initialize_npc(id, 2, seed, age, crate::content::entity(7, 2));
        laser.origin = LaserOrigin::PlayerTechnology;
        laser.sprite = Sprite::new(technology_definition());
        laser.uninitialized = Some(false);
        laser.spawner_variant = Some(0);
        laser.entity_flags = Some(0);
        laser.previous_angle = Some(0.0);
        laser
    }

    pub(crate) fn restore_fast_position(&mut self) {
        if self.fast {
            self.position = self.saved_position;
            self.fast = false;
        }
    }

    pub(crate) fn begin_fast(&mut self) -> bool {
        if self.first_update {
            return false;
        }
        self.fast = true;
        self.saved_position = self.position;
        if self.subtype == 1 {
            let damped = self.velocity * self.current_friction;
            self.position = self.position
                + WorldVector {
                    x: damped.x / 2.0,
                    y: damped.y / 2.0,
                };
        }
        true
    }

    fn has_first_player_parent(&self) -> bool {
        self.parent
            == Some(ActorRef {
                kind: ActorKind::Player,
                id: 0,
            })
    }

    pub(crate) fn grid_contact_indices(&self, grid: &Grid) -> Result<Vec<usize>, String> {
        if !self.line_enabled || self.line.points.len() < 2 {
            return Err("Laser grid contact requires actual initialized polyline".into());
        }
        let offset = self.render_offset * 0.8;
        let mut indices = BTreeSet::new();
        for point in self.line.points.iter().skip(2) {
            let probe = *point + offset;
            if grid.collision_at(probe) > 1 {
                if let Some(index) = grid.index(probe) {
                    indices.insert(index);
                }
            }
        }
        let count = self.line.points.len();
        let edge = difference(self.line.points[count - 1], self.line.points[count - 2]);
        let length = squared_length(edge).sqrt();
        let direction = if length > 0.0 {
            edge * (1.0 / length)
        } else {
            edge
        };
        let probe = self.line.points[count - 2] + direction * 35.0 + offset;
        if let Some(index) = grid.index(probe) {
            indices.insert(index);
        }
        Ok(indices.into_iter().collect())
    }

    pub(crate) fn finish_grid_contacts(&mut self) {
        self.line.simplify(0.001);
    }

    pub(crate) fn validate_contact_geometry(&self) -> Result<(), String> {
        if !self.collision_radius.is_finite()
            || self.collision_radius < 0.0
            || self.line.points.iter().any(|point| !point.is_finite())
        {
            return Err("Laser contact geometry contains invalid actual numeric owners".into());
        }
        Ok(())
    }

    pub(crate) fn contacts(&self, targets: &[LaserTarget]) -> Result<Vec<LaserContact>, String> {
        self.validate_contact_geometry()?;
        if self.attack_flags.0 != 0
            || !self.has_first_player_parent()
            || !self.line_enabled
            || self.line.points.len() < 2
        {
            return Err(
                "Laser neutral contact requires actual flags/Parent/polyline owners".into(),
            );
        }
        let parent = self.parent.ok_or("Laser actual parent missing")?;
        let mut contacts = Vec::new();
        for target in targets {
            if !target.position.is_finite()
                || !target.collision_radius.is_finite()
                || target.collision_radius < 0.0
            {
                return Err("Laser target query contains nonfinite actual geometry".into());
            }
            let effect = target.entity_type == 1000 && matches!(target.variant, 21 | 33 | 63 | 64);
            let eligible = (10..1000).contains(&target.entity_type)
                && !matches!(target.entity_collision_class, 0 | 1)
                && !target.raw_494
                && !target.dead
                && target.entity_flags & 0x10 == 0;
            let collect = effect
                || ((eligible || matches!(target.entity_type, 1 | 33))
                    && target.entity_type != 1
                    && target.entity_type != self.spawner_type);
            if !collect {
                continue;
            }
            let radius = target.collision_radius + self.collision_radius;
            let threshold = radius * radius;
            for segment in self.line.points.windows(2) {
                if threshold > segment_distance_squared(target.position, segment[0], segment[1]) {
                    contacts.push(if effect {
                        LaserContact::MarkEffectDead {
                            target: target.actor,
                        }
                    } else {
                        LaserContact::Damage {
                            target: target.actor,
                            source: parent,
                            amount: self.collision_damage,
                        }
                    });
                }
            }
        }
        Ok(contacts)
    }

    pub(crate) fn finish_full(&mut self) -> Result<(), String> {
        self.sprite
            .advance(if self.origin == LaserOrigin::PlayerTechnology {
                technology_definition()
            } else {
                definition()
            })?;
        self.velocity = self.velocity * self.current_friction;
        self.position = self.position + self.velocity;
        self.current_friction = self.configured_friction;
        if !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("Laser Base motion overflowed actual f32 owners".into());
        }
        Ok(())
    }
}
