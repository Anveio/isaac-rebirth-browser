//!Neutral555300 children and the independent persistent crater record.
//!The damage caller and ordinary Bomb helper retain their own owners.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    audio::SoundRequest,
    familiars::{definition, Sprite, SpriteEffect},
    motion::WorldVector,
    phase::Phase,
    State,
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

///Only the fixed1000/18/0 projection consumed by5CF58E is known. The
///partly initialized72-byte native row and ordinary RNG30 are not invented.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicCraterRecord {
    pub position: WorldVector,
    pub spawn_seed: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Controller {
    pub permanent_color: AttackColor,
    pub current_color: AttackColor,
    pub frame_count_7d4: u32,
    ///The local35 generator is separate from Entity7C0 and shared MT.
    pub local_frame_rng: Option<crate::items::Rng>,
    pub saved_position: Option<WorldVector>,
    pub fast_pending: bool,
}

fn frontier(state: &mut State, channel: &str) {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
}

fn admits_room(state: &State) -> bool {
    //459195 has a separate roomtype11 replacement sheet. No default room
    //type or unowned texture selection is substituted for that predicate.
    state
        .room
        .descriptor
        .as_ref()
        .is_some_and(|d| d.template.room_type != 11)
}

fn base(state: &mut State, variant: u32, seed: u32, position: WorldVector) -> Result<u32, String> {
    if !matches!(variant, 1 | 18) || !position.is_finite() {
        return Err("neutral explosion child requires actual family/position".into());
    }
    let config = definition(1000, variant).ok_or("authored explosion child config absent")?;
    let id = state.next_effect;
    state.next_effect = id.checked_add(1).ok_or("effect handles exhausted")?;
    state.sprite_effects.push(SpriteEffect {
        id,
        variant,
        subtype: 0,
        spawn_seed: seed,
        spawner_type: 0,
        spawner_variant: 0,
        parent: None,
        birth_game_update: state.game_update_count,
        actor_rng: crate::items::Rng::new(seed, [4, 3, 17]),
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
        flags: 0,
        visible: true,
        active: true,
        removed: false,
        render_order_offset: 0,
        lifetime_divisor: 0,
        time_to_live: -1,
        raw_7f4: 1,
        raw_7f8: 0.0,
        raw_7d8: false,
        sprite: Sprite::new(config),
        full_updates: 0,
        poop_effect: None,
        blood_effect: None,
        creep_effect: None,
        fly_effect: None,
        epic_fetus: None,
        explosion_effect: Some(Controller {
            permanent_color: AttackColor::WHITE,
            current_color: AttackColor::WHITE,
            frame_count_7d4: 0,
            local_frame_rng: None,
            saved_position: None,
            fast_pending: false,
        }),
    });
    Ok(id)
}

///This factory never appends a record. Reentry consumes stored seed/P and
///still runs the real InitUniform; there is no replacement SharedRaw.
fn admit_crater(state: &mut State, seed: u32, position: WorldVector) -> Result<u32, String> {
    if seed == 0 {
        return Err("crater local35 seed0 reaches an unadmitted source assertion".into());
    }
    let config = definition(1000, 18).ok_or("authored crater config absent")?;
    let frames = config
        .clips
        .iter()
        .find(|c| c.name == config.default_clip)
        .ok_or("crater default clip absent")?
        .frames;
    if frames == 0 {
        return Err("crater clip is empty".into());
    }
    let id = base(state, 18, seed, position)?;
    //4590B0 local constructor index35, independent from Entity.actor_rng.
    let mut local = crate::items::Rng::new(seed, [5, 9, 7]);
    let frame = local.bounded(frames);
    let alpha = state.boot_lifecycle.shared.uniform() * 0.2 + 0.8;
    let effect = state.sprite_effects.last_mut().ok_or("new crater absent")?;
    effect.flags |= 8;
    effect.raw_7d8 = true;
    effect.sprite.frame = frame as f32;
    effect.sprite.playing = false;
    effect.sprite.tint[3] = alpha;
    let owner = effect
        .explosion_effect
        .as_mut()
        .expect("fresh crater owner");
    owner.current_color.tint[3] = alpha;
    owner.local_frame_rng = Some(local);
    state.actor_scheduler.admit_background(ActorRef {
        kind: ActorKind::Effect,
        id,
    })?;
    Ok(id)
}

fn admit_explosion(
    state: &mut State,
    seed: u32,
    position: WorldVector,
    color: AttackColor,
) -> Result<u32, String> {
    let id = base(state, 1, seed, position)?;
    let flip = state.boot_lifecycle.shared.bounded(2) == 0;
    let effect = state
        .sprite_effects
        .last_mut()
        .ok_or("new explosion absent")?;
    effect.sprite.flip_x = flip;
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Effect,
        id,
    })?;
    let effect = state
        .sprite_effects
        .last_mut()
        .ok_or("admitted explosion absent")?;
    //55550C negative-duration virtual58 applies the full permanent color.
    //Fresh ChildNULL and the literal441C10 copier keep the two owners
    //distinct while copying all44 bytes; there is no timed override here.
    let owner = effect
        .explosion_effect
        .as_mut()
        .expect("fresh explosion owner");
    owner.permanent_color = color;
    owner.current_color = color;
    effect.sprite.tint = color.tint;
    effect.sprite.offsets = color.offsets;
    Ok(id)
}

///Actual common child demands occur AFTER554490 and its nested callbacks.
///False is an explicit unperformed branch, never an accepted partial blast.
pub(crate) fn explode_neutral(
    state: &mut State,
    position: WorldVector,
    damage: f32,
    flags: AttackFlags,
    color: AttackColor,
    source: crate::combat::DamageSource,
    copied_source: crate::combat::CopiedDamageSource,
) -> Result<bool, String> {
    if !position.is_finite() || !damage.is_finite() || damage < 0.0 || !color.is_finite() {
        return Err("neutral555300 contains nonfinite actual inputs".into());
    }
    //The first caller footprint produces WHITE. Full storage is retained,
    //but arbitrary colorize rendering is outside this neutral leaf.
    if flags.0 != 0 || color != AttackColor::WHITE {
        frontier(state, "neutralExplosionFlagsOrColor");
        return Ok(false);
    }
    if !admits_room(state) || state.room.grid.is_none() {
        frontier(state, "neutralExplosionRoomHistory");
        return Ok(false);
    }
    if state.room.dynamic_crater_records.is_none() {
        frontier(state, "neutralExplosionCraterRecordHistory");
        return Ok(false);
    }
    state
        .next_effect
        .checked_add(2)
        .ok_or("explosion child handles exhausted")?;
    let radius = if damage >= 105.0 {
        105.0
    } else if damage > 85.0 {
        90.0
    } else {
        75.0
    };
    crate::bombs::explode_with_copied_source(
        state,
        position,
        radius,
        damage,
        source,
        Some(copied_source),
    )?;
    let seed = state.boot_lifecycle.shared.next_u32();
    let id = admit_crater(state, seed, position)?;
    let actual_seed = state
        .sprite_effects
        .iter()
        .find(|e| e.id == id)
        .ok_or("actual crater absent before record append")?
        .spawn_seed;
    state
        .room
        .dynamic_crater_records
        .as_mut()
        .expect("preflight record owner")
        .push(DynamicCraterRecord {
            position,
            spawn_seed: actual_seed,
        });
    let seed = state.boot_lifecycle.shared.next_u32();
    admit_explosion(state, seed, position, color)?;
    //550B30's three flags predicates are all false for actualflags0.
    //Neither child has an immediate Full at this555300 call site.
    Ok(true)
}

pub(crate) fn replay_dynamic_craters(state: &mut State) -> Result<(), String> {
    let Some(records) = state.room.dynamic_crater_records.clone() else {
        frontier(state, "craterRecordReentryHistory");
        return Ok(());
    };
    if records.is_empty() {
        return Ok(());
    }
    if !admits_room(state) {
        frontier(state, "craterRecordReentryRoomHistory");
        return Ok(());
    }
    if records.iter().any(|r| !r.position.is_finite()) {
        return Err("stored crater position is nonfinite".into());
    }
    //Old18 paint is a visit projection; persistent records are the owner.
    state
        .room
        .background_stains
        .retain(|stain| stain.variant != 18);
    for record in records {
        admit_crater(state, record.spawn_seed, record.position)?;
    }
    Ok(())
}

pub(crate) fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if phase == Phase::Skipped {
        return Ok(());
    }
    let index = state
        .sprite_effects
        .iter()
        .position(|e| e.id == id)
        .ok_or("canonical explosion child is unresolved")?;
    if state.sprite_effects[index].explosion_effect.is_none()
        || !matches!(state.sprite_effects[index].variant, 1 | 18)
        || state.sprite_effects[index].subtype != 0
    {
        return Err("explosion child callback history is not admitted".into());
    }
    if phase == Phase::Interpolate {
        if state.sprite_effects[index].variant == 18 {
            return Err("background-only crater has no admitted live Fast callback".into());
        }
        let effect = &mut state.sprite_effects[index];
        if effect.flags & 2 != 0 {
            effect.flags &= !2;
        } else {
            let owner = effect.explosion_effect.as_mut().expect("validated owner");
            owner.saved_position = Some(effect.position);
            effect.position = effect.position + (effect.velocity * effect.current_friction) * 0.5;
            effect.presented_position = effect.position;
            owner.fast_pending = true;
        }
        return Ok(());
    }
    if !state.sprite_effects[index].active {
        return Ok(());
    }
    if state.sprite_effects[index].variant == 18 {
        return Err("flags8 crater has no admitted ordinary Full callback".into());
    }
    let effect = &mut state.sprite_effects[index];
    effect.full_updates = effect
        .full_updates
        .checked_add(1)
        .ok_or("explosion Full count exhausted")?;
    effect
        .sprite
        .advance(definition(1000, 1).ok_or("authored explosion config absent")?)?;
    effect.velocity = effect.velocity * effect.current_friction;
    effect.position = effect.position + effect.velocity;
    effect.presented_position = effect.position;
    effect.current_friction = effect.configured_friction;
    effect.sprite.rotation = if effect.sprite.flip_x { 0.0 } else { -0.0 };
    let owner = effect.explosion_effect.as_mut().expect("validated owner");
    if owner.frame_count_7d4 == 0 {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter.wrapping_sub(1),
            SoundRequest {
                id: 182,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
        //45ABDC is inside the first-sound branch, including cooldown misses.
        owner.frame_count_7d4 = owner.frame_count_7d4.wrapping_add(1);
    }
    if !effect.sprite.playing {
        effect.active = false;
        effect.removed = true;
    }
    Ok(())
}

pub(crate) fn restore_fast_positions(state: &mut State) -> Result<(), String> {
    for effect in &mut state.sprite_effects {
        //550860 uses the ordinary68/74 pool, not flags8 background80/8C.
        if effect.variant != 1 {
            continue;
        }
        if let Some(owner) = &mut effect.explosion_effect {
            if owner.fast_pending {
                effect.position = owner
                    .saved_position
                    .ok_or("explosion pending Fast has unknown450")?;
                effect.presented_position = effect.position;
                owner.fast_pending = false;
            }
        }
    }
    Ok(())
}

pub(crate) fn retire_full(state: &mut State) {
    let retired: Vec<_> = state
        .sprite_effects
        .iter()
        .filter(|e| e.explosion_effect.is_some() && !e.active)
        .map(|e| e.id)
        .collect();
    if retired.is_empty() {
        return;
    }
    let is_retired =
        |actor: ActorRef| actor.kind == ActorKind::Effect && retired.contains(&actor.id);
    for effect in &mut state.sprite_effects {
        if retired.contains(&effect.id) || effect.parent.is_some_and(|p| is_retired(p.actor)) {
            effect.parent = None;
        }
    }
    if state.attack_runtime.persistent.is_some_and(is_retired) {
        state.attack_runtime.persistent = None;
        state.weapon_admission.c18_is_known_empty = true;
    }
    for slot in &mut state.attack_runtime.secondary {
        if slot.is_some_and(is_retired) {
            *slot = None;
        }
    }
    state.sprite_effects.retain(|e| !retired.contains(&e.id));
    state.actor_scheduler.retain(|actor| !is_retired(actor));
}
