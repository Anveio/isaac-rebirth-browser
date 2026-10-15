//! Actual ordinary/controlled subtype 0 and Has118 subtype 1 Knife.
//! Its caller owns composition; this module owns Init, movement and lifetime.
use crate::{
    actors::ActorRef,
    familiars::{Definition, Sprite},
    items::Rng,
    motion::WorldVector,
    mt::Mt19937,
    phase::Phase,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

include!(concat!(env!("OUT_DIR"), "/attack_catalog.rs"));
const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
pub(crate) const CONTROLLED: u64 = 0x2_0000_0000;
pub use crate::attack_flags::AttackFlags;

///Actual Base+4A0 empty-chain proof. Missing/nonempty histories remain unknown;
///447FC0's Parent+498 registration does not produce this independent owner.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ChildChain {
    #[default]
    Unknown,
    Empty,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Knife {
    pub id: u32,
    pub entity_type: u32,
    pub variant: u32,
    pub subtype: u32,
    pub spawn_seed: u32,
    pub birth_game_update: u32,
    pub actor_rng: Rng,
    pub spawner_type: u32,
    pub spawner_variant: u32,
    pub parent: Option<ActorRef>,
    #[serde(default)]
    pub child_chain_4a0: ChildChain,

    pub active: bool,
    pub dead: bool,
    pub visible: bool,
    pub position: WorldVector,
    ///Actual Base+440, distinct from the saved fast-presentation position.
    pub home_position: WorldVector,
    ///Registered target+4a8, distinct from actual linked parent+498.
    pub homing_target: Option<ActorRef>,
    pub saved_position: WorldVector,
    pub render_offset: WorldVector,
    pub velocity: WorldVector,
    pub configured_friction: f32,
    pub current_friction: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub collision_damage: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub raw_494: bool,
    pub entity_flags: u32,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub attack_flags: AttackFlags,
    ///Permanent source3a0..3c8 input, distinct from rendered current2d0..2f8.
    pub source_color: crate::attack_runtime::AttackColor,
    pub current_color: crate::attack_runtime::AttackColor,
    pub fast: bool,
    pub sprite: Sprite,
    pub raw_7e0: f32,
    pub raw_7e4: f32,
    pub raw_7e8: f32,
    pub raw_7ec: f32,
    pub raw_7f0: f32,
    pub raw_7f4: f32,
    pub raw_7f8: f32,
    pub raw_7fc: WorldVector,
    pub raw_804: WorldVector,
    pub raw_80c: f32,
    pub raw_810: f32,
    pub raw_814: bool,
    ///Actual818 Line constructor and copied curve/history vectors.
    pub line: crate::laser_line::LaserLine,
    ///Line+18/+1c are copied by473e7e, but576f50 has no initializer
    ///and the admitted Knife path has no producer for these opaque words.
    pub line_raw_18: Option<[u32; 2]>,
    pub raw_8a0: u32,
    pub raw_8a4: f32,
    pub raw_8a8: f32,
    pub hit_sequences: BTreeSet<u32>,
}

impl Knife {
    ///473020 runs while Base.Init still has physical zeroP. GameSpawn writes
    /// requested position afterward; Init's804 copy remains zero.
    pub(crate) fn initialize(id: u32, seed: u32, age: u32, shared: &mut Mt19937) -> Self {
        let definition = &KNIFE_DEFINITION;
        let initial_uniform = shared.uniform();
        Self {
            id,
            entity_type: 8,
            variant: 0,
            subtype: 0,
            spawn_seed: seed,
            birth_game_update: age,
            actor_rng: Rng::new(seed, [4, 3, 17]),
            spawner_type: 0,
            spawner_variant: 0,
            parent: None,
            child_chain_4a0: ChildChain::Empty, //441F2A before the Parent498 link.

            active: true,
            dead: false,
            visible: true,
            position: ZERO,
            home_position: ZERO,
            homing_target: None,
            saved_position: ZERO,
            render_offset: ZERO,
            velocity: ZERO,
            configured_friction: definition.friction,
            current_friction: definition.friction,
            collision_radius: definition.collision_radius,
            collision_mass: definition.collision_mass,
            collision_damage: 22.0,
            hp: definition.base_hp,
            max_hp: definition.base_hp,
            raw_494: false,
            entity_flags: 0,
            grid_collision_class: 0,
            entity_collision_class: 4,
            attack_flags: AttackFlags(0),
            source_color: crate::attack_runtime::AttackColor::WHITE,
            current_color: crate::attack_runtime::AttackColor::WHITE,
            fast: false,
            sprite: Sprite::new(definition),
            raw_7e0: 1.0,
            raw_7e4: 0.0,
            raw_7e8: 0.0,
            raw_7ec: 0.0,
            raw_7f0: 0.0,
            raw_7f4: 0.0,
            raw_7f8: 0.0,
            raw_7fc: ZERO,
            raw_804: ZERO,
            raw_80c: 100.0,
            raw_810: 0.0,
            raw_814: false,
            line: crate::laser_line::LaserLine::default(), //4729dc→576f50.
            line_raw_18: None,
            raw_8a0: 0,
            raw_8a4: -100.0,
            raw_8a8: initial_uniform * 50.0 - 25.0,
            hit_sequences: BTreeSet::new(),
        }
    }

    ///441bb0 marks inactive/dead. Registered pointers clear only in Uninit.
    pub(crate) fn remove(&mut self) {
        self.active = false;
        self.dead = true;
    }

    /// Controlled474f40 admits once per exact target sequence in its registry,
    /// gated by444890(period5, actual own Entity+8), not by browser handles.
    pub(crate) fn admit_hit_sequence(
        &mut self,
        game_age: u32,
        own_sequence: u32,
        target_sequence: u32,
    ) -> bool {
        if !self.can_admit_hit_sequence(game_age, own_sequence, target_sequence) {
            return false;
        }
        self.hit_sequences.insert(target_sequence);
        true
    }

    pub(crate) fn can_admit_hit_sequence(
        &self,
        game_age: u32,
        own_sequence: u32,
        target_sequence: u32,
    ) -> bool {
        !self.hit_sequences.contains(&target_sequence)
            && (self.attack_flags.0 & CONTROLLED == 0
                || ((game_age.wrapping_add(own_sequence) as i32) % 5 == 0
                    && self.entity_flags & 0x4000 == 0))
    }

    /// ParentPlayer contact resolves50d090(kind4) at contact time, which may
    /// differ from the actor's earlier copied collisionDamage. Mutable source
    /// charge fields remain source actor state, independent of Player C0C.
    pub(crate) fn hit_damage(&self, resolved_kind4_damage: f32) -> Result<f32, String> {
        let charge = ((self.raw_7f4 * 4.0) / self.raw_80c) * self.raw_810;
        // Native ucomiss/jbe also takes this branch for an unordered value.
        let multiplier = if charge.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            2.0
        } else if charge < 4.0 {
            charge + 2.0
        } else {
            6.0
        };
        let damage = resolved_kind4_damage * multiplier;
        if !damage.is_finite() {
            return Err("knife contact damage overflowed source fields".into());
        }
        Ok(damage)
    }

    /// System5954dd→550860→445370 precedes normal Game dispatch, including
    /// full callbacks that later freeze. The render v30 is unrelated.
    pub(crate) fn restore_fast_position(&mut self) {
        if self.fast {
            self.position = self.saved_position;
            self.fast = false;
        }
    }

    ///472a40 neutral bit16-clear launch. The caller owns actual C0C/C78
    ///clamping and the source range input; no per-item distance is guessed here.
    pub(crate) fn launch_ordinary(&mut self, ratio: f32, range: f32) -> Result<(), String> {
        if !ratio.is_finite() || !range.is_finite() || ratio <= 0.0 || range <= 0.0 {
            return Err("ordinary Knife launch requires finite actual charge/range".into());
        }
        if self.attack_flags.0 & (CONTROLLED | 0x1_0000) != 0 {
            return Err("ordinary launch requires its admitted bit16-clear source path".into());
        }
        self.raw_80c = range * ratio;
        self.raw_7f4 = 0.0;
        self.raw_810 = ratio;
        self.raw_814 = true;
        self.raw_7f8 = self.raw_80c.abs().sqrt() * 2.0;
        Ok(())
    }

    ///4737ee/4739ad neutral body up to the optional 473b92 child producer.
    ///The real adapter must dispatch that nested factory before the rotation,
    ///registry/grid/Base tail below. First birth also invokes this body.
    pub(crate) fn begin_ordinary_update(
        &mut self,
        phase: Phase,
        parent: ParentView,
        game_age: u32,
        parent_has_118: bool,
    ) -> Result<bool, String> {
        if phase == Phase::Skipped {
            return Ok(false);
        }
        if self.subtype != 0
            || self.attack_flags.0 & (CONTROLLED | 4 | 0x400 | 0x1_0000 | 0x800000) != 0
        {
            return Err("ordinary Knife body requires its admitted neutral subtype0 path".into());
        }
        if phase == Phase::Interpolate {
            self.fast = true;
        }
        if !self.fast {
            self.current_friction *= 0.7;
        }
        if parent.primary_c08 != 4 {
            self.remove();
        }
        if !self.raw_814 {
            self.raw_7f0 = if (self.raw_7f0 - self.raw_7e8).abs() > 0.1 {
                crate::source_angles::approach(self.raw_7f0, self.raw_7e8, 0.3)
            } else {
                self.raw_7e8
            };
            self.position = parent.position + degree_direction(self.raw_7f0) * 30.0;
            self.raw_7fc = parent.position;
            return Ok(false);
        }
        self.raw_7f0 =
            crate::source_angles::approach(self.raw_7f0, self.raw_7e8 + self.raw_7ec, 0.8);
        self.raw_7f4 += self.raw_7f8 * 0.5;
        if !self.fast {
            self.raw_7f8 += if self.raw_7f8 > 0.0 { -2.0 } else { -5.0 };
        }
        if self.raw_7f4 < 0.0 {
            //474339 actual440 reset and447110(NULL), not parent unlink.
            self.home_position = ZERO;
            self.homing_target = None;
            self.raw_7f4 = 0.0;
            self.raw_814 = false;
        }
        let follow = 1.0 - ((self.raw_7f4 / self.raw_80c) * self.raw_810) * 0.8;
        //612560: old + (parent-old)*follow, keeping each f32 operation.
        self.raw_7fc = self.raw_7fc
            + WorldVector {
                x: parent.position.x - self.raw_7fc.x,
                y: parent.position.y - self.raw_7fc.y,
            } * follow;
        self.position = self.raw_7fc + degree_direction(self.raw_7f0) * (self.raw_7f4 + 30.0);
        Ok(parent_has_118
            && self.raw_7f8 < 0.0
            && self.entity_flags & 0x4000 == 0
            && (game_age as i32) % 2 == 0)
    }

    ///473929 runs after an actual nested Brim child callback when requested.
    ///Caller then performs the actual grid receiver and finish_full when true.
    pub(crate) fn finish_ordinary_update(
        &mut self,
        parent: ParentView,
        game_age: u32,
    ) -> Result<bool, String> {
        let from_parent = WorldVector {
            x: self.position.x - parent.position.x,
            y: self.position.y - parent.position.y,
        };
        self.sprite.rotation = crate::source_angles::wrap_180(
            radians_to_degrees(crate::fmath::atan2_f32(from_parent.y, from_parent.x)) - 90.0,
        );
        self.sprite
            .layer_flips
            .insert(0, [self.sprite.rotation > 0.0, false]);
        if self.entity_flags & 0x4000 == 0 && (game_age as i32) % 3 == 0 {
            self.hit_sequences.clear();
        }
        if self.fast {
            self.saved_position = self.position;
            //47268f ordinary subtype0 fast calls18 then saves450 only.
            //The controlled path and subtype1 have distinct prediction laws.
        }
        if !self.position.is_finite()
            || !self.velocity.is_finite()
            || !self.raw_7fc.is_finite()
            || !self.raw_7f4.is_finite()
            || !self.raw_7f8.is_finite()
            || !self.sprite.rotation.is_finite()
        {
            return Err("ordinary Knife body overflowed actual source fields".into());
        }
        Ok(!self.fast)
    }

    ///4726b8 subtype1 fast never dispatches18. It saves450 first and only
    ///predicts physical448 from the existing velocity/current friction.
    pub(crate) fn brim_child_fast(&mut self) -> Result<(), String> {
        self.fast = true;
        self.saved_position = self.position;
        self.position = self.position + (self.velocity * self.current_friction) * 0.5;
        if !self.position.is_finite() {
            return Err("Brim Knife fast prediction overflowed actual fields".into());
        }
        Ok(())
    }

    ///473570→47424E actual subtype1 cubic following. Full has no read of
    ///current Player weapon/charge, and the copied line does not follow changes
    ///to its parent's later818 owner.
    pub(crate) fn brim_child_full(
        &mut self,
        room: &crate::grid::Grid,
        game_age: u32,
    ) -> Result<(), String> {
        if self.subtype != 1 || self.fast || self.attack_flags.0 != 0 {
            return Err("Brim Knife full requires its actual subtype1 neutral owner".into());
        }
        if !self.position.is_finite()
            || !self.velocity.is_finite()
            || !self.current_friction.is_finite()
            || !self.raw_8a4.is_finite()
            || !self.raw_8a8.is_finite()
            || self.line.controls.iter().any(|point| {
                !point.position.is_finite()
                    || !point.incoming.is_finite()
                    || !point.outgoing.is_finite()
            })
        {
            return Err("Brim Knife body requires finite actual curve/motion owners".into());
        }
        self.current_friction *= 0.7;
        if self
            .parent
            .is_some_and(|parent| parent.kind == crate::actors::ActorKind::Knife)
        {
            let (index, parameter) = crate::knife_path::nearest_forward(
                self.position,
                &self.line.controls,
                self.raw_8a0,
                self.raw_8a4,
            );
            if parameter > -99.0 {
                let a = self.line.controls[index as usize];
                let b = self.line.controls[index as usize + 1];
                let on_curve = crate::laser_line::cubic(a, b, parameter);
                let next = crate::laser_line::cubic(a, b, parameter + 0.1);
                let tangent = normalized(WorldVector {
                    x: next.x - on_curve.x,
                    y: next.y - on_curve.y,
                });
                let target = next
                    + WorldVector {
                        x: tangent.y,
                        y: -tangent.x,
                    } * self.raw_8a8;
                let delta = WorldVector {
                    x: target.x - self.position.x,
                    y: target.y - self.position.y,
                };
                let length = squared_length(delta).sqrt();
                //6128e0 returns the original only for its zero threshold;
                //otherwise it sets magnitude8, even when the target is nearer.
                let push = if length > 0.0 {
                    delta * (8.0 / length)
                } else {
                    delta
                };
                if self.current_friction != 0.0 {
                    self.velocity = self.velocity
                        + WorldVector {
                            x: push.x / self.current_friction,
                            y: push.y / self.current_friction,
                        };
                }
                self.sprite.rotation =
                    crate::source_angles::wrap_180(angle_degrees(self.velocity) - 90.0);
                self.sprite
                    .layer_flips
                    .insert(0, [self.sprite.rotation > 0.0, false]);
                self.raw_8a0 = index;
                self.raw_8a4 = parameter;
            } else {
                self.remove();
            }
            let bounded = room.clamp_room(self.position, -200.0);
            let difference = WorldVector {
                x: bounded.x - self.position.x,
                y: bounded.y - self.position.y,
            };
            if squared_length(difference).sqrt() > 1.0
                || (game_age.wrapping_sub(self.birth_game_update) as i32) > 300
            {
                self.remove();
            }
        } else {
            self.remove();
        }
        if self.entity_flags & 0x4000 == 0 && (game_age as i32) % 3 == 0 {
            self.hit_sequences.clear();
        }
        if !self.position.is_finite()
            || !self.velocity.is_finite()
            || !self.sprite.rotation.is_finite()
        {
            return Err("Brim Knife full overflowed actual fields".into());
        }
        Ok(())
    }

    pub(crate) fn finish_full(&mut self) -> Result<(), String> {
        self.sprite.advance(&KNIFE_DEFINITION)?;
        self.velocity = self.velocity * self.current_friction;
        self.position = self.position + self.velocity;
        self.current_friction = self.configured_friction;
        if !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("knife Base motion overflowed source f32 fields".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ParentView {
    pub position: WorldVector,
    pub primary_c08: u32,
}

pub(crate) fn squared_length(vector: WorldVector) -> f32 {
    vector.x * vector.x + vector.y * vector.y
}

pub(crate) fn normalized(vector: WorldVector) -> WorldVector {
    let length = squared_length(vector).sqrt();
    if length > 0.0 {
        vector * (1.0 / length)
    } else {
        vector
    }
}

fn radians_to_degrees(angle: f32) -> f32 {
    angle * 57.295_776
}
///612700 delegates to atan2f then source6124b0 degree conversion.
pub(crate) fn angle_degrees(direction: WorldVector) -> f32 {
    radians_to_degrees(crate::fmath::atan2_f32(direction.y, direction.x))
}
///6126c0 uses source degree->radian multiplication before native sincosf.
pub(crate) fn degree_direction(degrees: f32) -> WorldVector {
    let radians = degrees * 0.017_453_292;
    let (sin, cos) = (crate::fmath::sin(radians), crate::fmath::cos(radians));
    WorldVector { x: cos, y: sin }
}
