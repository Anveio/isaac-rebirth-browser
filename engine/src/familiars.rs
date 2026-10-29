//! The Familiar entity (type 3): the 513E80 roster's CheckFamiliar 508D10
//! reconcile, Init 46EE90, the Update shell 4704A0 (appearance, the variant
//! switch, the common tail) and Remove 466AD0. Variant data lives in
//! familiar_variants.rs and the behaviour primitives in familiar_ai.rs.
use crate::{
    actors::{ActorKind, ActorRef},
    familiar_ai, familiar_roster,
    familiar_variants::{self, InterpolateCase, UpdateCase},
    grid::{Collider, CollisionGeometry, Grid},
    items,
    motion::WorldVector,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };
fn blood_variant() -> u32 {
    7
}
fn white() -> [f32; 4] {
    [1.0; 4]
}

#[derive(Debug)]
pub struct Clip {
    pub name: &'static str,
    pub frames: u32,
    pub looping: bool,
}

#[derive(Debug)]
pub struct Definition {
    pub entity_type: u32,
    pub variant: u32,
    pub actor: &'static str,
    pub base_hp: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub collision_damage: f32,
    pub friction: f32,
    pub shadow_size: f32,
    pub grid_points: u16,
    pub default_clip: &'static str,
    pub clips: &'static [Clip],
}
include!(concat!(env!("OUT_DIR"), "/familiar_catalog.rs"));

pub fn definition(entity_type: u32, variant: u32) -> Option<&'static Definition> {
    DEFINITIONS
        .iter()
        .find(|config| config.entity_type == entity_type && config.variant == variant)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Sprite {
    pub animation: String,
    pub frame: f32,
    pub playing: bool,
    pub speed: f32,
    pub scale: WorldVector,
    pub offset: WorldVector,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default)]
    pub layer_flips: std::collections::BTreeMap<u32, [bool; 2]>,
    /// Per-layer LayerState offset (+0x30), in anm2 pixels.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub layer_offsets: std::collections::BTreeMap<u32, [f32; 2]>,
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default = "white")]
    pub tint: [f32; 4],
    #[serde(default)]
    pub offsets: [f32; 3],
    /// ColorMod colorize RGBA (+0x10); the sprite shader mixes in the texel
    /// luminance times its RGB by its alpha. Zero for every colour but a
    /// rainbow poop's particles (56a820..56a9b3).
    #[serde(default, skip_serializing_if = "is_zero4")]
    pub colorize: [f32; 4],
}

fn is_zero4(value: &[f32; 4]) -> bool {
    *value == [0.0; 4]
}

impl Sprite {
    pub(crate) fn new(config: &Definition) -> Self {
        Self {
            animation: config.default_clip.into(),
            frame: 0.0,
            playing: true,
            speed: 1.0,
            scale: WorldVector { x: 1.0, y: 1.0 },
            offset: ZERO,
            rotation: 0.0,
            layer_flips: std::collections::BTreeMap::new(),
            layer_offsets: std::collections::BTreeMap::new(),
            flip_x: false,
            flip_y: false,
            tint: white(),
            offsets: [0.0; 3],
            colorize: [0.0; 4],
        }
    }
    pub(crate) fn advance(&mut self, config: &Definition) -> Result<(), String> {
        if !self.playing {
            return Ok(());
        }
        let clip = config
            .clips
            .iter()
            .find(|clip| clip.name == self.animation)
            .ok_or("Familiar/effect clip absent from authored catalogue")?;
        let next = self.frame + self.speed;
        if next.floor() < clip.frames as f32 {
            self.frame = next;
        } else if clip.looping {
            self.frame = 0.0;
        } else {
            self.frame = clip.frames as f32 - 1.0;
            self.playing = false;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Familiar {
    /// Browser handle and source Entity+8 have independent owners.
    pub id: u32,
    pub variant: u32,
    pub subtype: u32,
    pub spawn_seed: u32,
    pub owner: ActorRef,
    pub parent: ActorRef,
    pub parent_type: u32,
    pub parent_variant: u32,
    pub birth_game_update: u32,
    pub actor_rng: items::Rng,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub velocity: WorldVector,
    pub configured_friction: f32,
    pub current_friction: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub collision_damage: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub shadow_size: f32,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub flags: u32,
    pub visible: bool,
    pub active: bool,
    pub removed: bool,
    pub follower_index: i32,
    pub appearance_counter: i32,
    pub sprite: Sprite,
    pub full_updates: u32,
    /// Calls requiring the still-unimplemented Leech path/target callbacks.
    pub unresolved_ai_updates: u32,
    /// Familiar+F58: the shooter's fire cooldown, zeroed by Init 46EF04.
    #[serde(default)]
    pub fire_cooldown: i32,
    /// Familiar+F5C: the shooter's pose hold, zeroed by Init 46EF0E.
    #[serde(default)]
    pub shoot_hold: i32,
    /// Entity+498 (the follower ahead) and +4A0 (the follower behind),
    /// linked by 46ED90 at Init; None follows the Player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_parent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_child: Option<u32>,
    /// Familiar+F6C/F70/F78/F7C: orbit phase, radius vector, speed and
    /// layer (-1 when not an orbital), set by Init and 466780.
    #[serde(default)]
    pub orbit_phase: f32,
    #[serde(default = "zero_vector")]
    pub orbit_radius: WorldVector,
    #[serde(default)]
    pub orbit_speed: f32,
    #[serde(default = "no_orbit_layer")]
    pub orbit_layer: i32,
    /// Familiar+F98: the Entity+8 of every enemy contact damage reached
    /// since the last 444890 clear.
    #[serde(default)]
    pub hits: Vec<u32>,
    /// Entity+4A8: the chased enemy (an NPC id) from 4688D0.
    #[serde(default)]
    pub target: Option<u32>,
    /// Familiar+7E0: the NPCAI_Pathfinder a chaser walks with (5B41E0).
    #[serde(default = "crate::pathfinder::Pathfinder::new")]
    pub pathfinder: crate::pathfinder::Pathfinder,
}

fn zero_vector() -> WorldVector {
    ZERO
}

fn no_orbit_layer() -> i32 {
    -1
}

impl Familiar {
    fn new(variant: u32, id: u32, seed: u32, position: WorldVector, game_count: u32) -> Self {
        let config = definition(3, variant).expect("authored Familiar config");
        let player = ActorRef {
            kind: ActorKind::Player,
            id: 0,
        };
        Self {
            id,
            variant,
            subtype: 0,
            spawn_seed: seed,
            owner: player,
            parent: player,
            parent_type: 1,
            parent_variant: 0,
            birth_game_update: game_count,
            actor_rng: items::Rng::new(seed, [4, 3, 17]),
            position,
            presented_position: position,
            velocity: ZERO,
            configured_friction: config.friction,
            current_friction: config.friction,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: config.collision_damage,
            hp: config.base_hp,
            max_hp: config.base_hp,
            shadow_size: config.shadow_size,
            grid_collision_class: familiar_variants::grid_class(variant),
            entity_collision_class: 4,
            flags: 0x20_0004,
            visible: true,
            active: true,
            removed: false,
            follower_index: -1,
            appearance_counter: 5,
            sprite: Sprite::new(config),
            full_updates: 0,
            unresolved_ai_updates: 0,
            fire_cooldown: 0,
            shoot_hold: 0,
            follow_parent: None,
            follow_child: None,
            orbit_phase: 0.0,
            orbit_radius: ZERO,
            orbit_speed: 0.0,
            orbit_layer: -1,
            hits: Vec::new(),
            target: None,
            pathfinder: crate::pathfinder::Pathfinder::new(),
        }
    }
}

/// Parent identity and copied type/variant are distinct source fields.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Parent {
    pub actor: ActorRef,
    pub entity_type: u32,
    pub variant: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpriteEffect {
    pub id: u32,
    pub variant: u32,
    pub subtype: u32,
    pub spawn_seed: u32,
    /// GameSpawn's cached caller identity, distinct from linkedEntity.parent.
    #[serde(default)]
    pub spawner_type: u32,
    #[serde(default)]
    pub spawner_variant: u32,
    pub parent: Option<Parent>,
    pub birth_game_update: u32,
    pub actor_rng: items::Rng,
    pub position: WorldVector,
    pub presented_position: WorldVector,
    pub render_offset: WorldVector,
    pub velocity: WorldVector,
    pub configured_friction: f32,
    pub current_friction: f32,
    pub collision_radius: f32,
    pub collision_mass: f32,
    pub collision_damage: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub grid_collision_class: u32,
    pub entity_collision_class: u32,
    pub flags: u32,
    pub visible: bool,
    pub active: bool,
    pub removed: bool,
    pub render_order_offset: i32,
    pub lifetime_divisor: i32,
    pub time_to_live: i32,
    pub raw_7f4: i32,
    pub raw_7f8: f32,
    /// Effect+814: Init 458b9c draws Random(4) * 90 into it for 12/13/0x4f/0x50
    /// (zero otherwise). The 45abe3 leaf copies it into sprite rotation every
    /// Update, negated unless FlipX.
    #[serde(default)]
    pub rotation_814: f32,
    pub raw_7d8: bool,
    pub sprite: Sprite,
    pub full_updates: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poop_effect: Option<crate::poop_effects::Controller>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blood_effect: Option<crate::blood_effects::Controller>,
    /// Player footprint Blood7 whose floor-sampled color offsets are unknown.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub footprint: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub critter: Option<crate::critters::Critter>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creep_effect: Option<crate::creep_effects::Controller>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fly_effect: Option<crate::fly_effects::Controller>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epic_fetus: Option<crate::epic_fetus::Controller>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explosion_effect: Option<crate::explosion_effects::Controller>,
}

impl SpriteEffect {
    pub(crate) fn advance(&mut self, phase: Phase) -> Result<(), String> {
        if self.removed || phase == Phase::Skipped {
            return Ok(());
        }
        if phase == Phase::Interpolate {
            self.presented_position = self.position + (self.velocity * self.current_friction) * 0.5;
            return Ok(());
        }
        self.full_updates = self
            .full_updates
            .checked_add(1)
            .ok_or("effect full callback count exhausted")?;
        self.sprite
            .advance(definition(1000, self.variant).ok_or("effect config absent")?)?;
        // 45abe3 (variants 3, 11-16, 20, 34, 38, 40, 48, 49, 0x4f, 0x50):
        // rotation is +814, negated unless FlipX.
        if matches!(
            self.variant,
            3 | 11..=16 | 20 | 34 | 38 | 40 | 48 | 49 | 79 | 80
        ) {
            self.sprite.rotation = if self.sprite.flip_x {
                self.rotation_814
            } else {
                -self.rotation_814
            };
        }
        self.velocity = self.velocity * self.current_friction;
        self.position = self.position + self.velocity;
        self.presented_position = self.position;
        self.current_friction = self.configured_friction;
        if !self.sprite.playing {
            // Virtual Remove441bb0 clears active and marks dead. Visibility
            // remains unchanged; the following TTL branch marks dead only.
            self.active = false;
            self.removed = true;
        }
        if self.time_to_live > 0 {
            self.time_to_live -= 1;
        }
        if self.time_to_live == 0 {
            self.removed = true;
        }
        if !self.position.is_finite() || !self.velocity.is_finite() {
            return Err("effect motion overflowed".into());
        }
        Ok(())
    }
}

/// Real Game.Spawn factory; its caller owns the explicit seed and whether an
/// immediate Update is required. Init itself consumes no shared draws.
pub fn admit_poof(
    state: &mut State,
    subtype: u32,
    seed: u32,
    position: WorldVector,
    velocity: WorldVector,
    spawner: Option<Parent>,
) -> Result<u32, String> {
    NpcEffectAdmission {
        game_update_count: state.game_update_count,
        next_effect: &mut state.next_effect,
        effects: &mut state.sprite_effects,
        scheduler: &mut state.actor_scheduler,
    }
    .admit(subtype, seed, position, velocity, spawner)
}

/// Battery overcharge 509F14/50A443/52967B: one shared seed, then
/// Spawn550a40 of 1000.48 at the Player's +448 with zero velocity, subtype0
/// and a NULL spawner (Init draws nothing for 48). The caller then sets the
/// sprite offset +2BC to (0,-24) and Effect+7F8 to 1. Its update is the
/// one-shot 45ABE3 leaf: play the default clip, then Remove.
pub(crate) fn admit_battery(state: &mut State) -> Result<u32, String> {
    let seed = state.boot_lifecycle.shared.next_u32();
    let position = crate::bombs::player_world(state);
    let id = state.next_effect;
    state.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
    let mut effect = crate::fly_effects::base(id, 48, seed, position, state.game_update_count);
    effect.sprite.offset = WorldVector { x: 0.0, y: -24.0 };
    effect.raw_7f8 = 1.0;
    state.sprite_effects.push(effect);
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Effect,
        id,
    })?;
    Ok(id)
}

/// The source NPC callback admits appearance effects before BaseUpdate moves
/// its body or drains damage. Borrow only the actual factory owners so that
/// admission occurs at that call site, while the NPC is still being updated.
pub(crate) struct NpcEffectAdmission<'a> {
    pub game_update_count: u32,
    pub next_effect: &'a mut u32,
    pub effects: &'a mut Vec<SpriteEffect>,
    pub scheduler: &'a mut crate::actors::Scheduler,
}

impl NpcEffectAdmission<'_> {
    pub(crate) fn admit_npc(
        &mut self,
        npc: &crate::npcs::Npc,
        subtype: u32,
        seed: u32,
    ) -> Result<(), String> {
        self.admit(
            subtype,
            seed,
            npc.position,
            ZERO,
            Some(Parent {
                actor: ActorRef {
                    kind: ActorKind::Npc,
                    id: npc.id,
                },
                entity_type: npc.kind,
                variant: npc.variant,
            }),
        )?;
        // Generic4fbee6 copies the NPC render offset and4fbef9 writes height+1.
        // The supported fresh214/Monstro appearance has zero height/offset.
        self.effects.last_mut().expect("new NPC Poof").raw_7f8 = 1.0;
        Ok(())
    }

    fn admit(
        &mut self,
        subtype: u32,
        seed: u32,
        position: WorldVector,
        velocity: WorldVector,
        spawner: Option<Parent>,
    ) -> Result<u32, String> {
        if subtype > 3 || !position.is_finite() || !velocity.is_finite() {
            return Err(
                "Poof admission requires authored subtype0..3 and finite world vectors".into(),
            );
        }
        let config = definition(1000, 15).expect("authored Poof config");
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        let mut sprite = Sprite::new(config);
        sprite.animation = match subtype {
            1 => "Poof_Small",
            3 => "Poof_Large",
            _ => "Poof",
        }
        .into();
        self.effects.push(SpriteEffect {
            id,
            variant: 15,
            subtype,
            spawn_seed: seed,
            spawner_type: spawner.map_or(0, |caller| caller.entity_type),
            spawner_variant: spawner.map_or(0, |caller| caller.variant),
            // Game.Spawn550acd copies only cached1c/20. This appearance
            // caller never invokes447fc0, so actual Entity.parent498 is NULL.
            parent: None,
            birth_game_update: self.game_update_count,
            actor_rng: items::Rng::new(seed, [4, 3, 17]),
            position,
            presented_position: position,
            render_offset: ZERO,
            velocity,
            configured_friction: config.friction,
            current_friction: config.friction,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            // Common Effect.Init458b31 overrides authored0 to22 before subclass dispatch.
            collision_damage: 22.0,
            hp: config.base_hp,
            max_hp: config.base_hp,
            grid_collision_class: 0,
            entity_collision_class: 0,
            flags: 0,
            visible: true,
            active: true,
            removed: false,
            render_order_offset: if subtype == 0 { 8000 } else { 0 },
            lifetime_divisor: 0,
            time_to_live: -1,
            raw_7f4: 1,
            raw_7f8: 0.0,
            rotation_814: 0.0,
            raw_7d8: false,
            sprite,
            full_updates: 0,
            poop_effect: None,
            blood_effect: None,
            footprint: false,
            critter: None,
            creep_effect: None,
            fly_effect: None,
            epic_fetus: None,
            explosion_effect: None,
        });
        self.scheduler.admit(ActorRef {
            kind: ActorKind::Effect,
            id,
        })?;
        Ok(id)
    }
}

pub fn advance_effect(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if state
        .sprite_effects
        .iter()
        .any(|effect| effect.id == id && effect.critter.is_some())
    {
        return crate::critters::advance(state, phase, id);
    }
    if state
        .sprite_effects
        .iter()
        .any(|effect| effect.id == id && effect.explosion_effect.is_some())
    {
        return crate::explosion_effects::advance(state, phase, id);
    }
    if state
        .sprite_effects
        .iter()
        .any(|effect| effect.id == id && effect.epic_fetus.is_some())
    {
        return crate::epic_fetus::advance(state, phase, id);
    }
    if state
        .sprite_effects
        .iter()
        .any(|effect| effect.id == id && effect.fly_effect.is_some())
    {
        return crate::fly_effects::advance(state, phase, id);
    }
    if state
        .sprite_effects
        .iter()
        .any(|effect| effect.id == id && effect.creep_effect.is_some())
    {
        return crate::creep_effects::advance(state, phase, id);
    }
    if state
        .sprite_effects
        .iter()
        .any(|effect| effect.id == id && effect.blood_effect.is_some())
    {
        return crate::blood_effects::advance(state, phase, id);
    }
    if phase == Phase::Update
        && state.sprite_effects.iter().any(|effect| {
            effect.id == id
                && matches!(
                    effect.poop_effect,
                    Some(crate::poop_effects::Controller::Particle5(_))
                )
        })
    {
        return crate::poop_effects::advance_blood_particle(state, id);
    }
    if let Some(effect) = state
        .sprite_effects
        .iter_mut()
        .find(|effect| effect.id == id)
    {
        if effect.poop_effect.is_some() {
            match phase {
                Phase::Update => crate::poop_effects::advance_full(
                    effect,
                    &mut state.boot_lifecycle.shared,
                    &mut state.audio,
                    state.system_clock.system_counter.wrapping_sub(1),
                )?,
                Phase::Interpolate => {
                    let grid = Grid::new(
                        state
                            .room
                            .grid
                            .clone()
                            .ok_or("poop effect requires actual grid")?,
                    )?;
                    crate::poop_effects::advance_fast(effect, &grid)?;
                }
                Phase::Skipped => {}
            }
        } else {
            effect.advance(phase)?;
        }
    }
    Ok(())
}

pub(crate) fn evaluate_cache(state: &mut State) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    if owner.unevaluated_cache_flags & 0x200 == 0 {
        return Ok(());
    }
    // 513E80: every 508D10 call in order. Unported variant AI is local to
    // each familiar, so the whole 0x200 request completes.
    let calls = familiar_roster::calls(state.player_items.as_ref().expect("checked Player owner"));
    for call in calls {
        check_familiar(state, call)?;
    }
    state
        .player_items
        .as_mut()
        .expect("checked Player owner")
        .unevaluated_cache_flags &= !0x200;
    Ok(())
}

/// 508D10(variant, count, rng): the live familiars of the variant, in
/// manager order; those past the count are removed (vtable+48, Remove
/// 466AD0); each missing one takes a seed from the RNG (5C2D70), a shared
/// direction at 20 units from the Player (42E8D0), the room clamp
/// (5C7E50), Spawn 550A40 and one explicit Update (vtable+18).
fn check_familiar(state: &mut State, call: familiar_roster::Call) -> Result<(), String> {
    let requested = call.count.max(0) as usize;
    if requested > 0 && definition(3, call.variant).is_none() {
        // 78 (Scissors) names no base-Rebirth entity row: Spawn makes nothing.
        return Ok(());
    }
    let existing: Vec<_> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Familiar)
        .filter_map(|actor| {
            state.familiars.iter().position(|familiar| {
                familiar.id == actor.id && familiar.variant == call.variant && !familiar.removed
            })
        })
        .collect();
    for &index in existing.iter().skip(requested) {
        remove(state, index);
    }
    for _ in existing.len()..requested {
        let owner = state.player_items.as_mut().expect("checked Player owner");
        let rng = if call.trinket_rng {
            owner.trinket_rngs.get_mut(call.rng as usize)
        } else {
            owner.item_rngs.get_mut(call.rng as usize)
        }
        .ok_or("Familiar RNG owner out of range")?;
        let seed = rng.raw();
        let angle =
            (f64::from(state.boot_lifecycle.shared.uniform()) * std::f64::consts::PI * 2.0) as f32;
        let (sin, cos) = (crate::fmath::sin(angle), crate::fmath::cos(angle));
        let position = crate::bombs::player_world(state) + WorldVector { x: cos, y: sin } * 20.0;
        let grid = Grid::new(
            state
                .room
                .grid
                .clone()
                .ok_or("Familiar admission requires actual room bounds")?,
        )?;
        let position = grid.clamp_room(position, 20.0);
        let id = state.next_familiar;
        state.next_familiar = id.checked_add(1).ok_or("Familiar handle exhausted")?;
        state.familiars.push(Familiar::new(
            call.variant,
            id,
            seed,
            position,
            state.game_update_count,
        ));
        init(state, state.familiars.len() - 1);
        state.actor_scheduler.admit(ActorRef {
            kind: ActorKind::Familiar,
            id,
        })?;
        // Source508eb1 explicitly advances each new Familiar before the next
        // deficit. The manager's captured old tail does not advance it twice.
        advance_actor(state, Phase::Update, id)?;
    }
    Ok(())
}

/// Init 46EE90's variant tail: the follower train (4666B0 -> 46ED90), or
/// the orbit layer (0x66C470), its respacing 466780(layer, true) and the
/// layer radius 0x8ECB00 (x1.5 for Sacrificial Dagger, 46F308).
fn init(state: &mut State, index: usize) {
    let variant = state.familiars[index].variant;
    if familiar_variants::is_follower(variant) {
        let id = state.familiars[index].id;
        add_to_followers(state, id);
        return;
    }
    if let Some(layer) = familiar_variants::orbit_layer(variant) {
        state.familiars[index].orbit_layer = layer as i32;
        familiar_ai::respace(state, layer, index, true);
        let [x, y] = familiar_variants::ORBIT_RADIUS[layer];
        let radius = WorldVector { x, y };
        state.familiars[index].orbit_radius = if variant == 35 { radius * 1.5 } else { radius };
    }
}

/// Remove 466AD0: the entity leaves the manager, its orbit layer is
/// respaced without it (466780(layer, false)) and follower links drop.
fn remove(state: &mut State, index: usize) {
    state.familiars[index].active = false;
    state.familiars[index].removed = true;
    let layer = state.familiars[index].orbit_layer;
    if layer >= 0 {
        familiar_ai::respace(state, layer as usize, index, false);
    }
    unlink_follower(state, state.familiars[index].id);
}

/// 46ED90: every follower of the same Player with no +4A0 takes the new
/// Familiar as its child (447F40), and the new one takes it as +498
/// (447FC0), in manager order.
fn add_to_followers(state: &mut State, id: u32) {
    let tails: Vec<u32> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Familiar && actor.id != id)
        .filter(|actor| {
            state.familiars.iter().any(|familiar| {
                familiar.id == actor.id
                    && familiar_variants::is_follower(familiar.variant)
                    && familiar.follow_child.is_none()
            })
        })
        .map(|actor| actor.id)
        .collect();
    for tail in tails {
        if let Some(familiar) = state.familiars.iter_mut().find(|f| f.id == tail) {
            familiar.follow_child = Some(id);
        }
        if let Some(familiar) = state.familiars.iter_mut().find(|f| f.id == id) {
            familiar.follow_parent = Some(tail);
        }
    }
}

/// A removed Familiar's EntityRef registrations (4481E0) drop the links
/// that name it.
fn unlink_follower(state: &mut State, id: u32) {
    for familiar in &mut state.familiars {
        if familiar.follow_parent == Some(id) {
            familiar.follow_parent = None;
        }
        if familiar.follow_child == Some(id) {
            familiar.follow_child = None;
        }
    }
}

pub(crate) fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let Some(index) = state
        .familiars
        .iter()
        .position(|familiar| familiar.id == id)
    else {
        return Ok(());
    };
    let familiar = &mut state.familiars[index];
    if familiar.removed || phase == Phase::Skipped {
        return Ok(());
    }
    let config =
        definition(3, familiar.variant).ok_or("Familiar variant has no authored config")?;
    if phase == Phase::Interpolate {
        // FastUpdate 46F850 by variant.
        match familiar_variants::interpolate_case(familiar.variant) {
            InterpolateCase::Base => {}
            InterpolateCase::Follow => {
                familiar_ai::follow(state, index, true)?;
                let familiar = &mut state.familiars[index];
                familiar.presented_position = familiar.position;
                return Ok(());
            }
            InterpolateCase::FollowIdle | InterpolateCase::FollowFloat => {
                let runs = if familiar_variants::interpolate_case(familiar.variant)
                    == InterpolateCase::FollowIdle
                {
                    familiar.fire_cooldown == 0
                } else {
                    // 46F89D: IsPlaying("Float").
                    familiar.sprite.animation == "Float" && familiar.sprite.playing
                };
                if runs {
                    familiar_ai::follow(state, index, true)?;
                }
                let familiar = &mut state.familiars[index];
                familiar.presented_position = familiar.position;
                return Ok(());
            }
            InterpolateCase::Orbit => {
                familiar_ai::orbit_half(state, index);
                let familiar = &mut state.familiars[index];
                familiar.presented_position = familiar.position;
                return Ok(());
            }
            InterpolateCase::Unique(_) => {
                familiar.unresolved_ai_updates = familiar.unresolved_ai_updates.saturating_add(1);
                return Ok(());
            }
        }
        let familiar = &mut state.familiars[index];
        let mut position =
            familiar.position + (familiar.velocity * familiar.current_friction) * 0.5;
        let mut velocity = familiar.velocity;
        if familiar.grid_collision_class != 0 {
            let grid = Grid::new(
                state
                    .room
                    .grid
                    .clone()
                    .ok_or("Familiar update requires room grid")?,
            )?;
            collider(familiar.variant)?.resolve_nonplayer_ground(
                &grid,
                &mut position,
                &mut velocity,
                true,
            )?;
        }
        familiar.presented_position = position;
        return Ok(());
    }
    familiar.full_updates = familiar
        .full_updates
        .checked_add(1)
        .ok_or("Familiar full callback count exhausted")?;
    familiar.sprite.advance(config)?;
    familiar.velocity = familiar.velocity * familiar.current_friction;
    familiar.position = familiar.position + familiar.velocity;
    familiar.presented_position = familiar.position;
    familiar.current_friction = familiar.configured_friction;
    familiar.sprite.scale = WorldVector { x: 1.0, y: 1.0 };
    if familiar.appearance_counter > 0 {
        familiar.appearance_counter -= 1;
        familiar.visible = familiar.appearance_counter == 0;
        if familiar.appearance_counter == 3 && familiar.flags & 4 != 0 {
            let position = familiar.position;
            let seed = state.boot_lifecycle.shared.next_u32();
            let effect = admit_poof(state, 0, seed, position, ZERO, None)?;
            // Separate real caller470809, after admission and before old-tail
            // dispatch: this advances the Poof's authored frame0 to1.
            advance_effect(state, Phase::Update, effect)?;
        }
        return Ok(());
    }
    // 47075A: the variant switch.
    match familiar_variants::update_case(familiar.variant) {
        UpdateCase::None => {}
        UpdateCase::Follower => familiar_ai::follow(state, index, false)?,
        UpdateCase::Shooter | UpdateCase::ShooterMimic => {
            familiar_ai::follow(state, index, false)?;
            familiar_ai::shoot(state, index)?;
        }
        UpdateCase::Orbital => familiar_ai::orbit(state, index),
        UpdateCase::OrbitalShooter => {
            familiar_ai::orbit(state, index);
            familiar_ai::shoot(state, index)?;
        }
        UpdateCase::Chaser => familiar_ai::chase(state, index)?,
        UpdateCase::ChaserShooter => {
            familiar_ai::chase(state, index)?;
            familiar_ai::shoot(state, index)?;
        }
        UpdateCase::DryBaby => familiar_ai::dry_baby(state, index)?,
        UpdateCase::ShooterMirror
        | UpdateCase::ShooterFree
        | UpdateCase::Spawner
        | UpdateCase::Bum
        | UpdateCase::Unique(_) => {
            // Not ported: the familiar stays admitted and visible; the
            // missing case is counted on it, never on the cache.
            let familiar = &mut state.familiars[index];
            familiar.unresolved_ai_updates = familiar.unresolved_ai_updates.saturating_add(1);
        }
    }
    tail(state, index);
    Ok(())
}

/// Update tail 47064C: 444890(interval) clears the contact hit-set, and
/// BFFS! (247) scales the sprite by 1.25 (4706E7).
fn tail(state: &mut State, index: usize) {
    let frame = crate::knives::source_game_counter(state.game_update_count);
    let bffs = state
        .player_items
        .as_ref()
        .is_some_and(|owner| owner.owned_counts.get(247).copied().unwrap_or(0) > 0);
    let sequence = state
        .actor_scheduler
        .sequence_id(ActorRef {
            kind: ActorKind::Familiar,
            id: state.familiars[index].id,
        })
        .unwrap_or(0);
    let familiar = &mut state.familiars[index];
    // The case moved the live position; the presented one follows it.
    familiar.presented_position = familiar.position;
    if familiar.flags & 0x4000 == 0
        && crate::familiar_contact::is_frame(
            frame,
            familiar_variants::hit_interval(familiar.variant),
            sequence,
        )
    {
        familiar.hits.clear();
    }
    if bffs && familiar_variants::bffs_scaled(familiar.variant) {
        familiar.sprite.scale = familiar.sprite.scale * 1.25;
    }
}

/// Each variant's grid collider from its entities2.xml row (radius and
/// grid points), for the Familiars with a grid collision class.
fn collider(variant: u32) -> Result<Collider, String> {
    let config = definition(3, variant).ok_or("Familiar variant has no authored config")?;
    Collider::new(CollisionGeometry {
        radius: config.collision_radius,
        scale: WorldVector { x: 1.0, y: 1.0 },
        points: config.grid_points,
    })
}

/// Source454b10 runs full nonplayer grid responses after entity contacts,
/// observing newly appended actors. Explicit birth Update does not own it.
pub(crate) fn grid_contacts(state: &mut State) -> Result<(), String> {
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("Familiar grid pass requires room")?,
    )?;
    for actor in state.actor_scheduler.full_phase() {
        if actor.kind != ActorKind::Familiar {
            continue;
        }
        if let Some(familiar) = state
            .familiars
            .iter_mut()
            .find(|familiar| familiar.id == actor.id && familiar.grid_collision_class != 0)
        {
            collider(familiar.variant)?.resolve_nonplayer_ground(
                &grid,
                &mut familiar.position,
                &mut familiar.velocity,
                false,
            )?;
            familiar.presented_position = familiar.position;
        }
    }
    Ok(())
}

/// The actual background pool paints and removes flags8 effects during the
/// same manager full pass. Its room texture retains the selected stopped pose.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackgroundStain {
    pub source_effect_id: u32,
    pub spawn_seed: u32,
    pub birth_game_update: u32,
    pub position: WorldVector,
    #[serde(default = "blood_variant")]
    pub variant: u32,
    pub render_offset: WorldVector,
    pub layer_rotation: Option<f32>,
    pub sprite: Sprite,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub footprint: bool,
}

/// Fresh zero-gib death445c60: caller owns the external seed and color copy.
/// Blood7 Init consumes no shared draws; the explicit immediate Update does.
pub fn admit_blood_splat(
    state: &mut State,
    seed: u32,
    position: WorldVector,
    tint: [f32; 4],
    offsets: [f32; 3],
) -> Result<u32, String> {
    NpcEffectAdmission {
        game_update_count: state.game_update_count,
        next_effect: &mut state.next_effect,
        effects: &mut state.sprite_effects,
        scheduler: &mut state.actor_scheduler,
    }
    .admit_blood(seed, position, tint, offsets)
}

pub fn set_blood_scale(state: &mut State, id: u32, scale: f32) -> Result<(), String> {
    NpcEffectAdmission {
        game_update_count: state.game_update_count,
        next_effect: &mut state.next_effect,
        effects: &mut state.sprite_effects,
        scheduler: &mut state.actor_scheduler,
    }
    .set_blood_scale(id, scale)
}

pub fn mark_footprint(state: &mut State, id: u32) -> Result<(), String> {
    let effect = state
        .sprite_effects
        .iter_mut()
        .find(|e| e.id == id && e.variant == 7)
        .ok_or("footprint requires an admitted actual7")?;
    effect.footprint = true;
    Ok(())
}

pub fn advance_blood_initial(state: &mut State, id: u32) -> Result<(), String> {
    NpcEffectAdmission {
        game_update_count: state.game_update_count,
        next_effect: &mut state.next_effect,
        effects: &mut state.sprite_effects,
        scheduler: &mut state.actor_scheduler,
    }
    .advance_blood_initial(id, &mut state.boot_lifecycle.shared)
}

/// The first update of a Blood7 whose spawner wrote its Effect+818 size
/// before Update: the stain class still follows the incoming sprite scale,
/// and the drawn scale becomes `size`.
pub fn advance_blood_initial_sized(state: &mut State, id: u32, size: f32) -> Result<(), String> {
    NpcEffectAdmission {
        game_update_count: state.game_update_count,
        next_effect: &mut state.next_effect,
        effects: &mut state.sprite_effects,
        scheduler: &mut state.actor_scheduler,
    }
    .advance_blood_initial_sized(id, &mut state.boot_lifecycle.shared, size)
}

impl NpcEffectAdmission<'_> {
    pub(crate) fn admit_blood(
        &mut self,
        seed: u32,
        position: WorldVector,
        tint: [f32; 4],
        offsets: [f32; 3],
    ) -> Result<u32, String> {
        if !position.is_finite() || tint.iter().chain(offsets.iter()).any(|v| !v.is_finite()) {
            return Err("Blood Splat requires finite ground and copied color".into());
        }
        let config = definition(1000, 7).expect("authored Blood Splat config");
        let id = *self.next_effect;
        *self.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
        let mut sprite = Sprite::new(config);
        sprite.tint = tint;
        sprite.offsets = offsets;
        self.effects.push(SpriteEffect {
            id,
            variant: 7,
            subtype: 0,
            spawn_seed: seed,
            spawner_type: 0,
            spawner_variant: 0,
            parent: None,
            birth_game_update: self.game_update_count,
            actor_rng: items::Rng::new(seed, [4, 3, 17]),
            position,
            presented_position: position,
            render_offset: ZERO,
            velocity: ZERO,
            configured_friction: config.friction,
            current_friction: config.friction,
            collision_radius: config.collision_radius,
            collision_mass: config.collision_mass,
            collision_damage: 22.0,
            hp: config.base_hp,
            max_hp: config.base_hp,
            grid_collision_class: 0,
            entity_collision_class: 0,
            flags: 8,
            visible: true,
            active: true,
            removed: false,
            render_order_offset: 0,
            lifetime_divisor: 0,
            time_to_live: -1,
            raw_7f4: 1,
            raw_7f8: 0.0,
            rotation_814: 0.0,
            raw_7d8: true,
            sprite,
            full_updates: 0,
            poop_effect: None,
            blood_effect: None,
            footprint: false,
            critter: None,
            creep_effect: None,
            fly_effect: None,
            epic_fetus: None,
            explosion_effect: None,
        });
        // Room5c6940 assigns the actual sequence first; Manager4524f0 then routes
        // flags8 to its separate background pool rather than the ordinary array.
        self.scheduler.admit_background(ActorRef {
            kind: ActorKind::Effect,
            id,
        })?;
        Ok(id)
    }

    pub(crate) fn set_blood_scale(&mut self, id: u32, scale: f32) -> Result<(), String> {
        if !scale.is_finite() || scale <= 0.0 {
            return Err("Blood scale must be positive and finite".into());
        }
        let effect = self
            .effects
            .iter_mut()
            .find(|e| e.id == id && e.variant == 7)
            .ok_or("Blood scale requires an admitted actual7")?;
        effect.sprite.scale = WorldVector { x: scale, y: scale };
        Ok(())
    }

    /// The caller moving a 1000.7 after its first update (Peep 4c341a).
    pub(crate) fn move_blood(&mut self, id: u32, position: WorldVector) -> Result<(), String> {
        let effect = self
            .effects
            .iter_mut()
            .find(|e| e.id == id && e.variant == 7)
            .ok_or("Blood move requires an admitted actual7")?;
        effect.position = position;
        effect.presented_position = position;
        Ok(())
    }

    pub(crate) fn set_blood_color(
        &mut self,
        id: u32,
        tint: [f32; 4],
        offsets: [f32; 3],
    ) -> Result<(), String> {
        if tint.iter().chain(offsets.iter()).any(|v| !v.is_finite()) {
            return Err("Blood color must be finite".into());
        }
        let effect = self
            .effects
            .iter_mut()
            .find(|e| e.id == id && e.variant == 7)
            .ok_or("Blood color requires an admitted actual7")?;
        effect.sprite.tint = tint;
        effect.sprite.offsets = offsets;
        Ok(())
    }

    pub(crate) fn advance_blood_initial(
        &mut self,
        id: u32,
        random: &mut crate::mt::Mt19937,
    ) -> Result<(), String> {
        // Init 458b3b defaults Effect+818 to 1.
        self.advance_blood_initial_sized(id, random, 1.0)
    }

    pub(crate) fn advance_blood_initial_sized(
        &mut self,
        id: u32,
        random: &mut crate::mt::Mt19937,
        size: f32,
    ) -> Result<(), String> {
        let Some(effect) = self.effects.iter_mut().find(|e| e.id == id) else {
            return Err("Blood initial callback requires admitted effect".into());
        };
        if effect.variant != 7 || effect.full_updates != 0 {
            return Err("Blood explicit initial callback must execute once".into());
        }
        let config = definition(1000, 7).expect("authored Blood Splat config");
        // BaseUpdate runs before the first7-specific manual pose and shared draws.
        effect.sprite.advance(config)?;
        effect.velocity = effect.velocity * effect.current_friction;
        effect.position = effect.position + effect.velocity;
        effect.presented_position = effect.position;
        effect.current_friction = effect.configured_friction;
        let incoming_scale = effect.sprite.scale.x;
        let name = if incoming_scale < 0.4 {
            "Size1BloodStains"
        } else if incoming_scale < 0.6 {
            "Size2BloodStains"
        } else if incoming_scale < 0.8 {
            "Size3BloodStains"
        } else if incoming_scale < 1.2 {
            "Size4BloodStains"
        } else if incoming_scale < 2.0 {
            "Size5BloodStains"
        } else {
            "Size6BloodStains"
        };
        let clip = config
            .clips
            .iter()
            .find(|c| c.name == name)
            .expect("authored stain size");
        // 45f49f..45f4d9: the class above reads the incoming Sprite scale
        // (+2c4); the scale is then replaced by (Effect+818, Effect+818).
        effect.sprite.scale = WorldVector { x: size, y: size };
        effect.sprite.animation = name.into();
        effect.sprite.frame = random.bounded(clip.frames) as f32;
        effect.sprite.playing = false;
        effect.sprite.flip_x = random.bounded(2) == 0;
        effect.sprite.flip_y = random.bounded(2) == 0;
        if effect.sprite.tint[3] == 1.0 {
            effect.sprite.tint[3] = (random.bounded(6) as f32 * 0.1) + 0.5;
        }
        effect.full_updates = 1;
        Ok(())
    }
}

thread_local! {
    static ELIDE_BACKGROUND_STAINS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Serialize `f` with every room's background stains written as `[]`. The
/// browser's per-tick snapshot uses this; stains then cost O(new) per tick
/// through `background_stains_since` instead of O(all) per tick.
pub fn with_background_stains_elided<T>(f: impl FnOnce() -> T) -> T {
    ELIDE_BACKGROUND_STAINS.with(|elide| elide.set(true));
    let result = f();
    ELIDE_BACKGROUND_STAINS.with(|elide| elide.set(false));
    result
}

pub(crate) fn serialize_background_stains<S: serde::Serializer>(
    stains: &[BackgroundStain],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if ELIDE_BACKGROUND_STAINS.with(std::cell::Cell::get) {
        serializer.collect_seq(std::iter::empty::<&BackgroundStain>())
    } else {
        serializer.collect_seq(stains)
    }
}

/// Actual454df0 current-count background pass follows actor callbacks and
/// precedes entity contacts/grid queries. Newborn flags8 blood participates.
pub fn paint_background_effects(state: &mut State) -> Result<(), String> {
    // Insert4524f0 keeps the independent render array in unsigned virtual+50
    // key order. Equal keys insert before existing entries, so later births
    // paint first. Ordinary actors also occupy that array; only flags8 paint.
    // The effect key 4586c0 uses (Entity+8 & 0x3ff) + 100 when Init set +7d8,
    // as every flags8 site does; otherwise the y-based depth key.
    let scheduler = &state.actor_scheduler;
    let mut pending = Vec::new();
    for (index, effect) in state.sprite_effects.iter().enumerate() {
        if !((matches!(effect.variant, 7 | 17 | 27 | 43 | 58)
            || (effect.variant == 18 && effect.explosion_effect.is_some()))
            && effect.flags & 8 != 0
            && effect.active)
        {
            continue;
        }
        let key = if effect.raw_7d8 {
            let sequence = scheduler
                .sequence_id(ActorRef {
                    kind: ActorKind::Effect,
                    id: effect.id,
                })
                .ok_or("background effect has no admitted Entity+8 sequence")?;
            (sequence & 0x3ff) + 100
        } else {
            (((effect.position.y + effect.raw_7f8) * 100.0 + 10030.0) + 0.5).floor() as u32
        };
        pending.push((
            index,
            key.wrapping_add(effect.render_order_offset as u32),
            effect.id,
        ));
    }
    pending.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| b.2.cmp(&a.2)));
    for (index, _, _) in pending {
        let effect = &mut state.sprite_effects[index];
        if effect.variant == 7 && effect.full_updates != 1 {
            return Err("Blood background paint requires the explicit initial Update".into());
        }
        state.room.background_stains.push(BackgroundStain {
            source_effect_id: effect.id,
            spawn_seed: effect.spawn_seed,
            birth_game_update: effect.birth_game_update,
            position: effect.position,
            variant: effect.variant,
            render_offset: effect.render_offset,
            layer_rotation: effect
                .poop_effect
                .as_ref()
                .and_then(|controller| controller.layer_rotation()),
            sprite: effect.sprite.clone(),
            footprint: effect.footprint,
        });
        // Remove441bb0 clears active and marks dead; visibility stays unchanged.
        effect.active = false;
        effect.removed = true;
    }
    Ok(())
}
