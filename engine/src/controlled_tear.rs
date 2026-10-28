//! Source-owned Ludovico type2 actors in the existing Tear store.
//! The regular ballistic owner remains separate. Parent/registry/lifecycle
//! fields are admitted only by the real controlled factory, never by restore.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_color::AttackColor,
    attack_flags::AttackFlags,
    attack_runtime::{DispatchOutcome, ShootButtons},
    grid::Grid,
    motion::WorldVector,
    phase::Phase,
    projectiles::{self, Lifetime, TearMotion},
    State, Tear,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const CONTROLLED: u64 = 0x2_0000_0000;
const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

// The selected source callback may retain C98=0 or write exactly3 for C08=8.
// Low bits1 or2 alone and every modifier bit remain outside this domain.
fn admits_kind8_flags(flags: u64) -> bool {
    matches!(flags, 0 | 3)
}

fn admits_controlled_flags(flags: u64) -> bool {
    flags & CONTROLLED != 0 && admits_kind8_flags(flags & !CONTROLLED)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ControlledTear {
    pub parent: Option<ActorRef>,
    pub active: bool,
    pub dead: bool,
    pub visible: bool,
    pub entity_flags: u32,
    pub configured_friction: f32,
    pub current_friction: f32,
    pub saved_position: WorldVector,
    pub velocity_delta_b28: WorldVector,
    pub current_color: AttackColor,
    pub birth_game_update: u32,
    /// Actual808/818 registry, keyed by Entity+8 rather than browser handles.
    pub hit_sequences: BTreeSet<u32>,
    /// Knowledge of808/818, lost by an unclosed cadence history.
    #[serde(default)]
    pub registry_known: bool,
    /// Current48C/flags/color knowledge, restored only by a real live refresh.
    #[serde(default)]
    pub current_params_known: bool,
    /// A restored or imported nonempty4A0 chain is not silently empty.
    pub child_chain_empty: Option<bool>,
}

fn frontier(state: &mut State, name: &str) {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, name);
}

/// Read actual physical448 only when the existing controlled owner is known.
pub(crate) fn referenced_position(state: &State, id: u32) -> Result<Option<WorldVector>, String> {
    let tear = state
        .tears
        .iter()
        .find(|tear| tear.id == id)
        .ok_or("controlled Tear head reference is unresolved")?;
    Ok(tear.controlled.as_ref().map(|_| tear.motion.position))
}

pub(crate) fn head_direction(
    state: &mut State,
    aim: WorldVector,
    held: bool,
) -> Result<(), String> {
    let aim = if held {
        if let Some(actor) = state.attack_runtime.persistent {
            let position = match actor.kind {
                ActorKind::Tear => state
                    .tears
                    .iter()
                    .find(|tear| tear.id == actor.id)
                    .map(|tear| tear.motion.position),
                ActorKind::Knife => state
                    .knives
                    .iter()
                    .find(|knife| knife.id == actor.id)
                    .map(|knife| knife.position),
                ActorKind::Laser => state
                    .lasers
                    .iter()
                    .find(|laser| laser.id == actor.id)
                    .map(|laser| laser.position),
                _ => None,
            };
            let Some(position) = position else {
                frontier(state, "controlledTearHeadReference");
                return Ok(());
            };
            state.head_timer = 10;
            let player = crate::bombs::player_world(state);
            WorldVector {
                x: position.x - player.x,
                y: position.y - player.y,
            }
        } else {
            aim
        }
    } else {
        aim
    };
    if !aim.is_finite() {
        return Err("controlled Tear head delta is nonfinite".into());
    }
    crate::attack_runtime::ordinary_head_direction(state, aim);
    Ok(())
}

pub(crate) fn idle_head(state: &mut State, held: bool) {
    if !held && state.head_timer < 0 {
        if let Some(direction) = state.attack_runtime.movement_direction_d9c {
            state.attack_runtime.head_direction_da0 = Some(match direction {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => 3,
            });
        } else {
            frontier(state, "controlledTearMovementHistory");
        }
    }
}

/// The47 source queries in50D090 exclude329. These exact context owners
/// bypass every modifier; this is separate from the regular kind1 builder.
fn neutral_kind8_context(state: &State) -> bool {
    state.weapon_admission.source_player_type == Some(0)
        && state.weapon_cache.primary.numeric() == 8
        && state.player_items.as_ref().is_some_and(|items| {
            let queries = crate::item_queries::AppearanceQueries::new(
                items,
                state.weapon_cache.primary,
                state.player.entity_source,
                None,
            );
            items.owned_counts.len() == 347
                && items.owned_counts[329] > 0
                && items.temporary_collectibles.is_empty()
                //50D176/1C7 precede kind dispatch, including actual kind8.
                //Only their canonical negatives skip the shared RNG arms.
                && queries.has_trinket(30) == Some(false)
                && queries.has_trinket(31) == Some(false)
                && items.queue_is_empty()
                && items
                    .owned_counts
                    .iter()
                    .enumerate()
                    .all(|(id, &count)| count == 0 || (id == 329 && count > 0))
        })
}

#[derive(Clone, Copy)]
struct Kind8Parameters {
    variant: u32,
    flags: AttackFlags,
    damage: f32,
    color: AttackColor,
}

/// Pure50D090(kind8) selected path: actual output00=0,30=C98,
///38=C88 and all44 bytes04=CA0. No random draw occurs in this path.
/// The consumer retains its separate birth/live setters and special scale.
fn resolve_kind8_parameters(state: &State) -> Option<Kind8Parameters> {
    if !neutral_kind8_context(state)
        || !admits_kind8_flags(state.weapon_cache.flags_c98.0)
        || !state
            .player_items
            .as_ref()
            .is_some_and(|items| items.unevaluated_cache_flags & (1 | 2 | 4 | 8 | 32 | 64) == 0)
        || !state.player_stats.damage.is_finite()
        || state.player_stats.damage < 0.0
        || !state.weapon_cache.color_ca0.is_finite()
    {
        return None;
    }
    Some(Kind8Parameters {
        variant: 0,
        flags: state.weapon_cache.flags_c98,
        damage: state.player_stats.damage,
        color: state.weapon_cache.color_ca0,
    })
}

/// 519FC0: a first birth performs kind8 BEFORE SharedRaw, Init, property
/// setters, the explicit full callback and finally registered C18 binding.
pub(crate) fn dispatch(
    state: &mut State,
    input: ShootButtons,
    phase: Phase,
    aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if phase == Phase::Skipped || !aim.is_finite() {
        return Err("controlled Tear requires a finite actual callback".into());
    }
    if state.room.source_tear_slowdown_2494.is_none() {
        frontier(state, "controlledTearRoomSlowdownHistory");
        return Ok(DispatchOutcome::Unadmitted);
    }
    let neutral_modifiers = neutral_kind8_context(state);
    if let Some(actor) = state.attack_runtime.persistent {
        if actor.kind == ActorKind::Tear {
            let Some(tear) = state.tears.iter_mut().find(|tear| tear.id == actor.id) else {
                return Err("controlled Tear C18 is unresolved".into());
            };
            let Some(owner) = &tear.controlled else {
                frontier(state, "controlledTearPriorActorHistory");
                return Ok(DispatchOutcome::Unadmitted);
            };
            if input.held()
                && !state
                    .player_items
                    .as_ref()
                    .is_some_and(|items| items.unevaluated_cache_flags & 4 == 0)
            {
                frontier(state, "controlledTearCurrentShotSpeed");
                return Ok(DispatchOutcome::Unadmitted);
            }
            //51D409: current C7C, double then multiply .5, then actual470.
            //Raw held remains distinct from the processed aim/DB4 producer.
            if input.held() && owner.current_friction != 0.0 {
                let aim = if crate::knives::squared_length(aim) > 1.0 {
                    crate::knives::normalized(aim)
                } else {
                    aim
                };
                let coefficient = (state.player_stats.shot_speed * 2.0) * 0.5;
                let acceleration = aim * coefficient;
                tear.motion.velocity = tear.motion.velocity
                    + WorldVector {
                        x: acceleration.x / owner.current_friction,
                        y: acceleration.y / owner.current_friction,
                    };
                if !tear.motion.velocity.is_finite() {
                    return Err("controlled Tear steering overflowed source fields".into());
                }
            }
            if owner.child_chain_empty != Some(true) {
                frontier(state, "controlledTearChildChainHistory");
                return Ok(DispatchOutcome::Unadmitted);
            }
            if !neutral_modifiers {
                tear.controlled
                    .as_mut()
                    .expect("controlled owner")
                    .child_chain_empty = None;
                frontier(state, "controlledTearChildModifiers");
                return Ok(DispatchOutcome::Unadmitted);
            }
            //Neutral153/2/245/TEMP245 requests zero4A0 extras. No global
            //secondary-slot cleanup and no per-callback property broadcast.
            return Ok(DispatchOutcome::Handled);
        }
        crate::attack_runtime::remove_referenced_actor(state, actor)?;
    }
    let Some(params) = resolve_kind8_parameters(state) else {
        frontier(state, "controlledTearKind8");
        return Ok(DispatchOutcome::Unadmitted);
    };
    let damage = params.damage;
    if !damage.is_finite() || damage < 0.0 {
        return Err("controlled Tear kind8 damage is invalid".into());
    }
    //51A3B1: regular scale has a different binary32 additive term.
    let linear = damage * 0.04 + 1.5;
    let scale =
        ((f64::from(linear) + f64::from(damage).sqrt() * f64::from(0.15_f32)) as f32).max(0.01);
    let config = crate::content::entity(2, params.variant);
    let radius = projectiles::collision_radius(scale, config.collision_radius)?;
    let tier = projectiles::regular_animation_tier(scale)?;
    let id = state.next_tear;
    let next = id
        .checked_add(1)
        .ok_or("controlled Tear handle exhausted")?;
    let seed = state.boot_lifecycle.shared.next_u32();
    let falling_speed = (state.boot_lifecycle.shared.uniform() - 0.5) * 0.2;
    state.audio.admit(
        &mut state.boot_lifecycle.shared,
        state.system_clock.system_counter.wrapping_sub(1),
        crate::audio::SoundRequest {
            id: 153,
            volume: 1.0,
            delay: 2,
            looping: false,
            pitch: 1.0,
        },
    );
    let position = crate::bombs::player_world(state) + WorldVector { x: 0.0, y: 5.0 };
    let color = params.color;
    let mut tear = Tear {
        id,
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        age: 0,
        spawn_seed: seed,
        birth_game_update: None,
        shot_ordinal: None,
        source_variant: Some(params.variant),
        //The controlled owner below retains its separate actual cadence.
        persistent_contacts: None,
        collision_damage: damage,
        base_damage: damage,
        attack_flags: AttackFlags(params.flags.0 | CONTROLLED),
        source_color: color,
        scale,
        animation_tier: tier,
        collision_radius: radius,
        grid_contact: false,
        grid_death: false,
        motion: TearMotion {
            position,
            presented_position: position,
            velocity: ZERO,
            height: -40.0,
            falling_speed,
            falling_acceleration: 0.0,
            render_offset_y: projectiles::height_offset(-40.0, 0.0),
            lifetime: Lifetime::Alive,
        },
        controlled: Some(ControlledTear {
            parent: Some(ActorRef {
                kind: ActorKind::Player,
                id: 0,
            }),
            active: true,
            dead: false,
            visible: true,
            entity_flags: 0,
            configured_friction: config.friction,
            current_friction: config.friction,
            saved_position: ZERO,
            velocity_delta_b28: ZERO,
            current_color: color,
            birth_game_update: state.game_update_count,
            hit_sequences: BTreeSet::new(),
            registry_known: true,
            current_params_known: true,
            child_chain_empty: Some(true),
        }),
    };
    tear.project();
    state.tears.push(tear);
    state.next_tear = next;
    let actor = ActorRef {
        kind: ActorKind::Tear,
        id,
    };
    state.actor_scheduler.admit(actor)?;
    advance(state, Phase::Update, id)?;
    crate::attack_runtime::replace_c18(state, Some(actor))?;
    Ok(DispatchOutcome::Handled)
}

pub(crate) fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if phase == Phase::Skipped {
        return Ok(());
    }
    let index = state
        .tears
        .iter()
        .position(|tear| tear.id == id)
        .ok_or("controlled Tear handle is unresolved")?;
    if phase == Phase::Update
        && state.tears[index]
            .controlled
            .as_ref()
            .is_some_and(|owner| !owner.active)
    {
        //454DF0 skips inactive actors before invoking virtual18. No registry,
        //property, room query or tail work runs for this fenced callback.
        return Ok(());
    }
    let Some(slowdown) = state.room.source_tear_slowdown_2494 else {
        frontier(state, "controlledTearRoomSlowdownHistory");
        return Ok(());
    };
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("controlled Tear requires actual Grid")?,
    )?;
    let player_position = crate::bombs::player_world(state);
    let empty_room = state.room.manager_counts.countable_total() <= 0;
    let period = state.max_fire_delay;
    let cadence_known = state
        .player_items
        .as_ref()
        .is_some_and(|items| items.unevaluated_cache_flags & 2 == 0);
    if phase == Phase::Update && !cadence_known {
        if !neutral_kind8_context(state) {
            state.tears[index].source_variant = None;
        }
        state.tears[index]
            .controlled
            .as_mut()
            .ok_or("controlled Tear source owner absent")?
            .registry_known = false;
        state.tears[index]
            .controlled
            .as_mut()
            .expect("controlled owner")
            .current_params_known = false;
        frontier(state, "controlledTearRegistryCadence");
    }
    if phase == Phase::Update && cadence_known && period <= 0 {
        return Err("controlled Tear requires positive current C78".into());
    }
    let periodic = phase == Phase::Update
        && cadence_known
        && state.tears[index].controlled.as_ref().is_some_and(|owner| {
            owner.entity_flags & 0x4000 == 0 && (state.game_update_count as i32) % period == 0
        });
    if phase == Phase::Update && periodic {
        state.tears[index]
            .controlled
            .as_mut()
            .expect("controlled owner")
            .hit_sequences
            .clear();
        state.tears[index]
            .controlled
            .as_mut()
            .expect("controlled owner")
            .registry_known = true;
        if let Some(cell) = grid.index(state.tears[index].motion.position) {
            let entity = state
                .room
                .grid
                .as_ref()
                .and_then(|g| g.cells[cell].entity.as_ref());
            //The nontrivial virtual28 parameter/effect paths remain explicit;
            //do not substitute Knife's argument or invent an RNG draw.
            if entity.is_some() {
                frontier(state, "controlledTearGridReceiver");
            }
        }
        //544972 runs on the real cadence. It changes current48C/color/flags,
        //not birth83C or the caller's separately chosen scale.
        if let Some(params) = resolve_kind8_parameters(state) {
            if state.tears[index].source_variant != Some(params.variant) {
                state.tears[index]
                    .controlled
                    .as_mut()
                    .expect("controlled owner")
                    .current_params_known = false;
                frontier(state, "controlledTearVariantTransitionHistory");
            } else {
                let tear = &mut state.tears[index];
                tear.collision_damage = params.damage;
                tear.source_color = params.color;
                tear.controlled
                    .as_mut()
                    .expect("controlled owner")
                    .current_color = tear.source_color;
                tear.attack_flags =
                    AttackFlags((params.flags.0 & 0xffff_fffd_fff7_ffff) | CONTROLLED);
                tear.controlled
                    .as_mut()
                    .expect("controlled owner")
                    .current_params_known = true;
            }
        } else {
            if !neutral_kind8_context(state) {
                state.tears[index].source_variant = None;
            }
            state.tears[index]
                .controlled
                .as_mut()
                .expect("controlled owner")
                .current_params_known = false;
            frontier(state, "controlledTearKind8Refresh");
        }
    }
    let tear = &mut state.tears[index];
    let owner = tear
        .controlled
        .as_mut()
        .ok_or("controlled Tear source owner absent")?;
    if phase == Phase::Update && !owner.active {
        return Ok(());
    }
    if owner.parent
        != Some(ActorRef {
            kind: ActorKind::Player,
            id: 0,
        })
        || owner.child_chain_empty != Some(true)
        || !admits_controlled_flags(tear.attack_flags.0)
    {
        frontier(state, "controlledTearActorHistory");
        return Ok(());
    }
    if phase == Phase::Interpolate {
        //5415F0: save450, then actual448 += (464*.75)/2 when slowed.
        owner.saved_position = tear.motion.position;
        let velocity = if slowdown > 0 {
            tear.motion.velocity * 0.75
        } else {
            tear.motion.velocity
        };
        tear.motion.position = tear.motion.position + velocity * 0.5;
        tear.motion.position = grid.clamp_room(tear.motion.position, 0.0);
    } else {
        //543A27 literal0x3F333333 is independent of configured friction.
        owner.current_friction *= f32::from_bits(0x3f33_3333);
        let delta = WorldVector {
            x: player_position.x - tear.motion.position.x,
            y: player_position.y - tear.motion.position.y,
        };
        if empty_room
            && crate::knives::squared_length(delta) > 10_000.0
            && owner.current_friction != 0.0
        {
            let acceleration = crate::knives::normalized(delta) * 0.2;
            tear.motion.velocity = tear.motion.velocity
                + WorldVector {
                    x: acceleration.x / owner.current_friction,
                    y: acceleration.y / owner.current_friction,
                };
        }
        let before = tear.motion.position;
        let saved_velocity = tear.motion.velocity;
        if slowdown > 0 {
            tear.motion.velocity = tear.motion.velocity * 0.75;
        }
        tear.motion.velocity = tear.motion.velocity * owner.current_friction;
        tear.motion.position = tear.motion.position + tear.motion.velocity;
        owner.current_friction = owner.configured_friction;
        if slowdown > 0 {
            tear.motion.velocity = saved_velocity * owner.current_friction;
        }
        tear.motion.position = grid.clamp_room(tear.motion.position, 0.0);
        owner.velocity_delta_b28 = WorldVector {
            x: tear.motion.position.x - before.x,
            y: tear.motion.position.y - before.y,
        };
        tear.age = state
            .game_update_count
            .wrapping_sub(owner.birth_game_update);
    }
    tear.motion.presented_position = tear.motion.position;
    if !tear.motion.position.is_finite()
        || !tear.motion.velocity.is_finite()
        || !owner.current_friction.is_finite()
    {
        return Err("controlled Tear source motion overflowed".into());
    }
    tear.project();
    Ok(())
}

///441BB0 Remove marks31E/31F; no C18/Parent weak reference is cleared here.
pub(crate) fn remove(state: &mut State, id: u32) -> Result<(), String> {
    let tear = state
        .tears
        .iter_mut()
        .find(|tear| tear.id == id)
        .ok_or("removed Tear handle is unresolved")?;
    let Some(owner) = &mut tear.controlled else {
        return Err("Tear Remove history has no source owner".into());
    };
    owner.active = false;
    owner.dead = true;
    Ok(())
}

///448040 only clears slots still registered to the actual retired identity.
pub(crate) fn uninit(state: &mut State, ids: &[u32]) {
    let retired = |actor: ActorRef| actor.kind == ActorKind::Tear && ids.contains(&actor.id);
    if state.attack_runtime.persistent.is_some_and(retired) {
        state.attack_runtime.persistent = None;
        state.weapon_admission.c18_is_known_empty = true;
    }
    for slot in &mut state.attack_runtime.secondary {
        if slot.is_some_and(retired) {
            *slot = None;
        }
    }
    for tear in &mut state.tears {
        if let Some(owner) = &mut tear.controlled {
            if owner.parent.is_some_and(retired) {
                owner.parent = None;
            }
        }
    }
    for knife in &mut state.knives {
        if knife.parent.is_some_and(retired) {
            knife.parent = None;
        }
    }
    for laser in &mut state.lasers {
        if laser.parent.is_some_and(retired) {
            laser.parent = None;
        }
    }
}

pub(crate) fn retire_full(state: &mut State) {
    let ids: Vec<_> = state
        .tears
        .iter()
        .filter_map(|tear| {
            tear.controlled
                .as_ref()
                .filter(|owner| !owner.active)
                .map(|_| tear.id)
        })
        .collect();
    uninit(state, &ids);
    state.tears.retain(|tear| !ids.contains(&tear.id));
}

pub(crate) fn clear_room(state: &mut State) {
    let ids: Vec<_> = state
        .tears
        .iter()
        .filter(|tear| tear.controlled.is_some())
        .map(|tear| tear.id)
        .collect();
    uninit(state, &ids);
}

/// Actual generic-AI dispatch4F9670 maps only type66 to481090. Existing
/// admitted NPC families exclude that owner. Future Death variant0 must own
/// its exact4822DF max160 write; an unclosed callback/restore remains unknown.
pub(crate) fn before_npc_update(state: &mut State, phase: Phase, id: u32) {
    if phase == Phase::Update
        && state
            .room
            .npcs
            .iter()
            .find(|npc| npc.id == id)
            .is_none_or(|npc| {
                (npc.kind == 66 && npc.variant == 0) || crate::entities::family(npc.kind).is_none()
            })
    {
        state.room.source_tear_slowdown_2494 = None;
    }
}

/// Selected lower type2 virtual80 path. The common flags0 receiver consumes
/// current48C and the Tear source itself. The bit33 veto is independent of
/// queue acceptance; no regular tear_hit, impulse, Dying or impact birth runs.
pub(crate) fn contacts_actor(state: &mut State, id: u32) -> Result<(), String> {
    if state
        .tears
        .iter()
        .find(|tear| tear.id == id)
        .and_then(|tear| tear.controlled.as_ref())
        .is_some_and(|owner| !owner.registry_known)
    {
        frontier(state, "controlledTearRegistryHistory");
        return Ok(());
    }
    if state
        .tears
        .iter()
        .find(|tear| tear.id == id)
        .and_then(|tear| tear.controlled.as_ref())
        .is_some_and(|owner| !owner.current_params_known)
    {
        frontier(state, "controlledTearCurrentParametersHistory");
        return Ok(());
    }
    let candidates: Vec<_> = state
        .tears
        .iter()
        .filter(|tear| tear.id == id)
        .filter_map(|tear| {
            let owner = tear.controlled.as_ref()?;
            (owner.active && !owner.dead).then_some((
                tear.id,
                tear.motion.position,
                tear.collision_radius,
            ))
        })
        .collect();
    let targets: Vec<_> = state
        .room
        .npcs
        .iter()
        .filter(|npc| !npc.dead && !npc.removed && matches!(npc.entity_collision_class, 2 | 4))
        .map(|npc| (npc.id, npc.kind, npc.position, npc.collision_radius))
        .collect();
    for (id, position, radius) in candidates {
        for &(target_id, kind, target, target_radius) in &targets {
            let dx = position.x - target.x;
            let dy = position.y - target.y;
            let combined = radius + target_radius;
            if dx.abs() > combined || dy.abs() > combined || (dx * dx + dy * dy).sqrt() >= combined
            {
                continue;
            }
            if kind <= 2 {
                frontier(state, "controlledTearContactOrdering");
                continue;
            }
            let Some(sequence) = state.actor_scheduler.sequence_id(ActorRef {
                kind: ActorKind::Npc,
                id: target_id,
            }) else {
                frontier(state, "controlledTearContactIdentity");
                continue;
            };
            let tear_index = state
                .tears
                .iter()
                .position(|tear| tear.id == id)
                .expect("retained contact Tear");
            let tear = &mut state.tears[tear_index];
            if !admits_controlled_flags(tear.attack_flags.0) {
                frontier(state, "controlledTearContactFlags");
                continue;
            }
            let owner = tear.controlled.as_mut().expect("controlled contact owner");
            if !owner.hit_sequences.insert(sequence) {
                //545EF3: a repeated actual registry hit still returns1.
                continue;
            }
            //546658/686: firstPlayer511D90(0) false is proved by actual
            //transformation ownership and no151, never assumed from flags.
            if !state
                .player
                .transformation_counts
                .is_some_and(|counts| counts[0] < 3)
                || !state
                    .player_items
                    .as_ref()
                    .is_some_and(|items| items.owned_counts.get(151) == Some(&0))
            {
                frontier(state, "controlledTearContactTransformationHistory");
                continue;
            }
            if !state
                .source_cache_run_flags
                .is_some_and(|flags| flags & 8 == 0)
            {
                frontier(state, "controlledTearContactRunFlagsHistory");
                continue;
            }
            let tear = &state.tears[tear_index];
            let amount = tear.collision_damage;
            let velocity = tear.motion.velocity;
            let flags = tear.attack_flags.0;
            let npc = state
                .room
                .npcs
                .iter_mut()
                .find(|npc| npc.id == target_id)
                .expect("retained contact receiver");
            npc.damage.enqueue(
                npc.health(),
                crate::combat::DamageEvent {
                    amount,
                    kind: crate::combat::DamageKind::Tear,
                    source: crate::combat::DamageSource { id, entity_type: 2 },
                    copied_source: None,
                },
            )?;
            //546278..546309: this separate receiver callback adds half
            //Tear velocity even when the FIFO declined this first hit.
            if matches!(kind, 14 | 18) && flags & 0x202 == 0 {
                npc.velocity = npc.velocity + velocity * 0.5;
            }
            //Neutral542030 has no effect without flag40. 54634C returns1
            //for actual bit33 even when the receiver declines the queue.
            //The existing FIFO represents consumed type/id/amount/flags.
            //Copied variant/mode/P/V historical EntityRef fields remain an
            //explicit unimplemented queue-history scope, never invented0.
        }
    }
    Ok(())
}
