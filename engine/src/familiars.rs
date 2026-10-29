//! Player cache511f10 -> Familiar reconciliation508d10, Leech56 appearance
//! and Brother Bobby1 (follow 466B50, fire 4702E0/467170).
//! Leech navigation/target and Familiar contacts remain explicit pending work.
use crate::{
    actors::{ActorKind, ActorRef},
    grid::{Collider, CollisionGeometry, Grid},
    items,
    motion::WorldVector,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

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
}

/// 513E80 calls 508D10 for each Familiar variant in this order; these are
/// the supported (variant, collectible) pairs, in that order.
const SUPPORTED: [(u32, usize); 2] = [(BROTHER_BOBBY, 8), (LEECH, 270)];
const BROTHER_BOBBY: u32 = 1;
const LEECH: u32 = 56;

/// 4666B0: the variants Init 46EFD7 adds to the follower train.
fn is_follower(variant: u32) -> bool {
    matches!(
        variant,
        1..=7 | 9..=12 | 20..=23 | 25..=28 | 40 | 41 | 51 | 52 | 54 | 55 | 57 | 59 | 61 | 74
    )
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
            // Init 46EFCB gives Leech56 class 5; 46F16B gives the others 0.
            grid_collision_class: if variant == LEECH { 5 } else { 0 },
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

fn reconcile_selected_zero_requests(state: &State) -> Result<bool, String> {
    const VARIANTS: [u32; 68] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 20, 21, 22, 23, 24, 30, 31, 32, 17, 16, 25, 41, 42, 35,
        50, 53, 68, 51, 52, 54, 75, 57, 64, 56, 76, 77, 65, 55, 58, 59, 66, 67, 74, 62, 79, 63, 61,
        33, 40, 14, 60, 44, 45, 46, 47, 69, 70, 71, 72, 48, 78, 28, 26, 27, 12, 13, 15,
    ];
    let mut empty_queries = true;
    for variant in VARIANTS {
        // 508D10 queries 455680 even when the requested count is zero. Retain
        // inactive and dead records until Uninit. An unowned lookup is an
        // error; nonempty excess-removal paths remain requested here.
        let query = state
            .actor_scheduler
            .full_phase()
            .into_iter()
            .filter(|actor| actor.kind == ActorKind::Familiar)
            .map(|actor| {
                state
                    .familiars
                    .iter()
                    .find(|familiar| familiar.id == actor.id)
                    .ok_or("Familiar manager query contains an unowned actor")
            })
            .collect::<Result<Vec<_>, _>>()?;
        if query
            .into_iter()
            .any(|familiar| familiar.variant == variant)
        {
            empty_queries = false;
        }
    }
    Ok(empty_queries)
}

pub(crate) fn evaluate_cache(state: &mut State) -> Result<(), String> {
    let Some(owner) = state.player_items.as_ref() else {
        return Ok(());
    };
    if owner.unevaluated_cache_flags & 0x200 == 0 {
        return Ok(());
    }
    if state.weapon_admission.source_player_type == Some(0)
        && crate::item_pipeline::ItemInputs::new(owner)?.admits_selected_zero_familiar_requests()
    {
        if reconcile_selected_zero_requests(state)? {
            state
                .player_items
                .as_mut()
                .expect("checked Player owner")
                .unevaluated_cache_flags &= !0x200;
        }
        return Ok(());
    }
    for (variant, item) in SUPPORTED {
        reconcile(state, variant, item)?;
    }
    let owner = state.player_items.as_mut().expect("checked Player owner");
    // unevaluatedCacheFlags is a port diagnostic, not native d6c. Source clears
    // all bits only after all selected callbacks; unsupported flags stay here.
    let unsupported_familiar = owner.owned_counts.iter().enumerate().any(|(id, &count)| {
        !SUPPORTED.iter().any(|&(_, item)| item == id)
            && (count != 0 || owner.temporary_count(id as u32) != 0)
            && items::definition(id as u32).is_some_and(|config| config.kind == 4)
    });
    if !unsupported_familiar {
        owner.unevaluated_cache_flags &= !0x200;
    }
    Ok(())
}

/// 508D10 for one variant: the owned plus temporary count of its
/// collectible, excess removal through vtable+48, then each deficit.
fn reconcile(state: &mut State, variant: u32, item: usize) -> Result<(), String> {
    let owner = state.player_items.as_ref().expect("checked Player owner");
    let requested = owner.owned_counts[item]
        .wrapping_add(owner.temporary_count(item as u32))
        .max(0) as usize;
    let existing: Vec<_> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Familiar)
        .filter_map(|actor| {
            state.familiars.iter().position(|familiar| {
                familiar.id == actor.id && familiar.variant == variant && !familiar.removed
            })
        })
        .collect();
    for &index in existing.iter().skip(requested) {
        state.familiars[index].active = false;
        state.familiars[index].removed = true;
        unlink_follower(state, state.familiars[index].id);
    }
    for _ in existing.len()..requested {
        let seed = state
            .player_items
            .as_mut()
            .expect("checked Player owner")
            .item_rngs[item]
            .raw();
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
            variant,
            id,
            seed,
            position,
            state.game_update_count,
        ));
        if is_follower(variant) {
            add_to_followers(state, id);
        }
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
                    && is_follower(familiar.variant)
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
    let config = definition(3, familiar.variant).ok_or("Familiar variant has no authored config")?;
    if phase == Phase::Interpolate {
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
            collider().resolve_nonplayer_ground(&grid, &mut position, &mut velocity, true)?;
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
    match familiar.variant {
        LEECH => advance_leech(state, index),
        BROTHER_BOBBY => {
            // 4708E6: 466B50(false), then 4702E0.
            follow(state, index)?;
            shoot(state, index)
        }
        _ => Err("Familiar variant has no ported AI".into()),
    }
}

fn advance_leech(state: &mut State, index: usize) -> Result<(), String> {
    // Source469a00's empty-room owner deadband requires no path request or draw.
    // Other target/path consumers stay observable instead of a fitted vector.
    let player = crate::bombs::player_world(state);
    let familiar = &mut state.familiars[index];
    let delta = WorldVector {
        x: player.x - familiar.position.x,
        y: player.y - familiar.position.y,
    };
    if state.room.npcs.iter().any(|npc| !npc.removed)
        || delta.x * delta.x + delta.y * delta.y > 6400.0
    {
        familiar.unresolved_ai_updates = familiar
            .unresolved_ai_updates
            .checked_add(1)
            .ok_or("unresolved Familiar callback count exhausted")?;
    }
    familiar.current_friction *= 0.85;
    if (familiar.velocity.x * familiar.velocity.x + familiar.velocity.y * familiar.velocity.y)
        .sqrt()
        < 2.0
    {
        familiar.sprite.frame = 0.0;
    }
    Ok(())
}

/// 466B50(false): follow the first eligible follower ahead (+498), or the
/// Player. Past 20 units the position moves a tenth of the excess toward
/// it; the current friction then takes the 0.8 factor.
fn follow(state: &mut State, index: usize) -> Result<(), String> {
    let mut ahead = state.familiars[index].follow_parent;
    let mut target = None;
    while let Some(id) = ahead {
        let parent = state
            .familiars
            .iter()
            .find(|familiar| familiar.id == id)
            .ok_or("Familiar +498 names no Familiar")?;
        // 466B92..466C60: Little Chubby3 and 59 while their cooldown runs,
        // and 25 of subtype 1, are passed over.
        let skipped = (matches!(parent.variant, 3 | 59) && parent.fire_cooldown != 0)
            || (parent.variant == 25 && parent.subtype == 1);
        if !skipped {
            target = Some(parent.position);
            break;
        }
        ahead = parent.follow_parent;
    }
    let target = target.unwrap_or_else(|| crate::bombs::player_world(state));
    let familiar = &mut state.familiars[index];
    let delta = WorldVector {
        x: target.x - familiar.position.x,
        y: target.y - familiar.position.y,
    };
    let length = (delta.x * delta.x + delta.y * delta.y).sqrt();
    if length > 20.0 {
        let step = (length - 20.0) / length * 0.1;
        familiar.position = familiar.position + delta * step;
    }
    familiar.current_friction *= 0.8;
    if !familiar.position.is_finite() {
        return Err("Familiar follow overflowed".into());
    }
    Ok(())
}

/// 467BA0 (Float*) and 4683B0 (FloatShoot*): direction 0 left (flipped
/// FloatSide), 1 up, 2 right, 3 down. Play keeps a clip already playing.
fn pose(familiar: &mut Familiar, direction: i32, shooting: bool) -> Result<(), String> {
    let side = match direction {
        0 | 2 => "Side",
        1 => "Up",
        3 => "Down",
        _ => return Ok(()),
    };
    familiar.sprite.flip_x = direction == 0;
    let clip = format!("Float{}{side}", if shooting { "Shoot" } else { "" });
    if familiar.sprite.animation != clip {
        familiar.sprite.animation = clip;
        familiar.sprite.frame = 0.0;
        familiar.sprite.playing = true;
    }
    Ok(())
}

/// 4702E0 for an ordinary shooter: +F58 counts down; with the Player's
/// shooting direction (+DA4) and a spent cooldown, 467170 fires and the
/// cooldown restarts at 30 with an 18-update pose hold.
fn shoot(state: &mut State, index: usize) -> Result<(), String> {
    let direction = state.player_fire_direction;
    let familiar = &mut state.familiars[index];
    if familiar.fire_cooldown > 0 {
        familiar.fire_cooldown -= 1;
    }
    if direction == -1 {
        if familiar.shoot_hold >= 0 {
            familiar.shoot_hold -= 1;
            if familiar.shoot_hold == -1 {
                pose(familiar, 3, false)?;
            }
        }
        return Ok(());
    }
    if familiar.fire_cooldown <= 0 {
        fire(state, index, direction)?;
        let familiar = &mut state.familiars[index];
        familiar.shoot_hold = 18;
        familiar.fire_cooldown = 30;
    }
    let familiar = &mut state.familiars[index];
    if familiar.shoot_hold > 8 {
        pose(familiar, direction, true)?;
        familiar.shoot_hold -= 1;
    } else {
        pose(familiar, direction, false)?;
        familiar.shoot_hold = 8;
    }
    Ok(())
}

/// 467170 for Brother Bobby: the direction times 10 plus 1.2 times the
/// Player's velocity, renormalized to 10 below length 100; damage 3.5
/// (doubled by BFFS 247) and scale 0.8.
fn fire(state: &mut State, index: usize, direction: i32) -> Result<(), String> {
    let aim = match direction {
        0 => WorldVector { x: -1.0, y: 0.0 },
        1 => WorldVector { x: 0.0, y: -1.0 },
        2 => WorldVector { x: 1.0, y: 0.0 },
        3 => WorldVector { x: 0.0, y: 1.0 },
        _ => return Err("Familiar fire direction outside 0..3".into()),
    };
    let player_velocity = crate::bombs::player_velocity(state);
    let mut velocity = aim * 10.0 + (player_velocity * 0.6_f32) * 2.0;
    let length = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
    if length < 100.0 {
        velocity = crate::knives::normalized(velocity) * 10.0;
    }
    let bffs = state
        .player_items
        .as_ref()
        .is_some_and(|owner| owner.owned_counts[247] > 0 || owner.temporary_count(247) > 0);
    let damage = 3.5 * if bffs { 2.0 } else { 1.0 };
    let origin = state.familiars[index].position;
    crate::tear_factory::fire_familiar(state, origin, velocity, damage, 0.8)?;
    Ok(())
}

fn collider() -> &'static Collider {
    static COLLIDER: OnceLock<Collider> = OnceLock::new();
    COLLIDER.get_or_init(|| {
        let config = definition(3, 56).expect("authored Leech collider");
        Collider::new(CollisionGeometry {
            radius: config.collision_radius,
            scale: WorldVector { x: 1.0, y: 1.0 },
            points: config.grid_points,
        })
        .expect("authored Leech geometry")
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
            collider().resolve_nonplayer_ground(
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
