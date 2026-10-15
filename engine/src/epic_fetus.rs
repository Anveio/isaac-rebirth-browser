//!519FC0 C08=6: a canonical target Effect30 and its timed Rocket31.
//!The rocket reads FIRSTPlayer kind6 when it strikes, not at either birth.
use crate::{
    actors::{ActorKind, ActorRef},
    attack_runtime::{DispatchOutcome, ShootButtons},
    familiars::{definition, Parent, Sprite, SpriteEffect},
    item_pipeline::ItemInputs,
    motion::WorldVector,
    phase::Phase,
    weapon_plan::{ControllerChannel, PrimaryWeaponC08, ResolvedWeaponPlan},
    State,
};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

///This optional owner proves the admitted30/31 callback history.
///Other SpriteEffects do not acquire these callbacks from variant alone.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Controller {
    ///BaseInit does not initialize source450; first actual Fast owns it.
    pub saved_position: Option<WorldVector>,
    pub fast_pending: bool,
}

fn frontier(state: &mut State, channel: &str) -> DispatchOutcome {
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, channel);
    DispatchOutcome::Unadmitted
}

fn admitted(effect: &SpriteEffect) -> bool {
    let Some(owner) = &effect.epic_fetus else {
        return false;
    };
    (!owner.fast_pending || owner.saved_position.is_some())
        && owner
            .saved_position
            .is_none_or(|position| position.is_finite())
        && matches!(effect.variant, 30 | 31)
        && effect.subtype == 0
        && effect.position.is_finite()
        && effect.velocity.is_finite()
        && effect.current_friction.is_finite()
}

pub(crate) fn admits_referenced_remove(state: &State, id: u32) -> Result<bool, String> {
    Ok(state
        .sprite_effects
        .iter()
        .find(|effect| effect.id == id)
        .is_some_and(admitted))
}

pub(crate) fn referenced_position(state: &State, id: u32) -> Result<Option<WorldVector>, String> {
    Ok(state
        .sprite_effects
        .iter()
        .find(|effect| effect.id == id)
        .filter(|effect| admitted(effect))
        .map(|effect| effect.position))
}

///441BB0 preserves lifetime, age and registered links until actual Uninit.
pub(crate) fn remove_referenced(state: &mut State, id: u32) -> Result<(), String> {
    let effect = state
        .sprite_effects
        .iter_mut()
        .find(|effect| effect.id == id)
        .ok_or("registered Epic Effect is unresolved")?;
    if !admitted(effect) {
        return Err("Epic Effect Remove requires owned callback history".into());
    }
    effect.active = false;
    effect.removed = true;
    Ok(())
}

///Actual Spawn550A40: cached spawner1C/20 is independent of Parent498.
fn admit(
    state: &mut State,
    variant: u32,
    seed: u32,
    position: WorldVector,
    ttl: i32,
) -> Result<u32, String> {
    if !matches!((variant, ttl), (30, 50 | 70) | (31, 10 | 20 | 30)) || !position.is_finite() {
        return Err("Epic Effect factory requires its actual variant/lifetime/position".into());
    }
    let config = definition(1000, variant).ok_or("authored Epic Effect config absent")?;
    let id = state.next_effect;
    state.next_effect = id.checked_add(1).ok_or("effect handle exhausted")?;
    let mut sprite = Sprite::new(config);
    if variant == 31 {
        //4591C8 SetFrame("Falling",0), not animation playback.
        sprite.animation = "Falling".into();
        sprite.frame = 0.0;
        sprite.playing = false;
    }
    state.sprite_effects.push(SpriteEffect {
        id,
        variant,
        subtype: 0,
        spawn_seed: seed,
        spawner_type: 1,
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
        time_to_live: ttl,
        raw_7f4: ttl,
        raw_7f8: 0.0,
        raw_7d8: false,
        sprite,
        full_updates: 0,
        poop_effect: None,
        blood_effect: None,
        creep_effect: None,
        fly_effect: None,
        explosion_effect: None,
        epic_fetus: Some(Controller {
            saved_position: None,
            fast_pending: false,
        }),
    });
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Effect,
        id,
    })?;
    Ok(id)
}

pub(crate) fn dispatch_player(
    state: &mut State,
    plan: &ResolvedWeaponPlan,
    input: ShootButtons,
    phase: Phase,
    callback_aim: WorldVector,
) -> Result<DispatchOutcome, String> {
    if plan.primary != PrimaryWeaponC08::EpicFetus
        || plan.controller != ControllerChannel::EpicFetus
        || phase == Phase::Skipped
    {
        return Err("Epic Fetus requires actual C08=6 callback".into());
    }
    let inputs = ItemInputs::new(state.player_items.as_ref().ok_or("Epic items absent")?)?;
    if state.weapon_admission.source_player_type != Some(0) || !inputs.admits_epic_fetus_caller() {
        return Ok(frontier(state, "epicFetusCallerContext"));
    }
    if !callback_aim.is_finite() {
        return Err("Epic Fetus current aim is nonfinite".into());
    }
    if input.held() {
        //51A182 precedes the actual C18 NULL/dead tests.
        state.head_timer = 10;
    }
    if let Some(actor) = state.attack_runtime.persistent {
        if actor.kind != ActorKind::Effect {
            return Ok(frontier(state, "epicFetusC18FamilyHistory"));
        }
        let effect = state
            .sprite_effects
            .iter_mut()
            .find(|effect| effect.id == actor.id)
            .ok_or("Epic Fetus C18 is unresolved")?;
        if !admitted(effect) || effect.variant != 30 || effect.parent.is_some() {
            return Ok(frontier(state, "epicFetusTargetHistory"));
        }
        if effect.removed {
            state.fire_cooldown = plan.max_fire_delay;
        } else if input.held() {
            let mut aim = callback_aim;
            let squared = aim.x * aim.x + aim.y * aim.y;
            if squared > 1.0 {
                aim = aim * (1.0 / squared.sqrt());
            }
            //51A1F8..260: no current C7C multiplier or friction division.
            effect.velocity = effect.velocity + aim * 8.0;
            if !effect.velocity.is_finite() {
                return Err("Epic target steering overflowed".into());
            }
        }
        return Ok(DispatchOutcome::Handled);
    }
    if !state.weapon_admission.c18_is_known_empty {
        return Ok(frontier(state, "epicFetusC18History"));
    }
    if !input.held() || state.fire_cooldown >= 0 {
        return Ok(DispatchOutcome::Handled);
    }
    //The caller footprint closes Have153=false.51E370 independently reads
    //actual shooting Player Have2: false gives50; true gives70.
    let ttl = if inputs.positive_owned_count(2) {
        70
    } else {
        50
    };
    let seed = state.boot_lifecycle.shared.next_u32();
    let position = crate::bombs::player_world(state);
    let id = admit(state, 30, seed, position, ttl)?;
    crate::attack_runtime::replace_c18(
        state,
        Some(ActorRef {
            kind: ActorKind::Effect,
            id,
        }),
    )?;
    //51B9A1 after the optional55/87 branches, both absent in this readset.
    state.fire_cooldown = plan.max_fire_delay;
    Ok(DispatchOutcome::Handled)
}

fn set_frame(effect: &mut SpriteEffect, animation: &str, frame: u32) {
    effect.sprite.animation = animation.into();
    effect.sprite.frame = frame as f32;
    effect.sprite.playing = false;
}

fn room_clamp(state: &State, point: WorldVector) -> Result<Option<WorldVector>, String> {
    let Some(descriptor) = state.room.grid.clone() else {
        return Ok(None);
    };
    let grid = crate::grid::Grid::new(descriptor)?;
    Ok(Some(grid.clamp_room(point, 0.0)))
}

///45AAB0 runs after the selected body, independently of actual active31E.
fn ttl_tail(effect: &mut SpriteEffect) {
    if effect.time_to_live > 0 {
        effect.time_to_live -= 1;
        if effect.time_to_live == 0 {
            effect.removed = true;
        }
    }
}

pub(crate) fn advance(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if phase == Phase::Skipped {
        return Ok(());
    }
    let index = state
        .sprite_effects
        .iter()
        .position(|effect| effect.id == id)
        .ok_or("Epic Effect callback is unresolved")?;
    if !admitted(&state.sprite_effects[index]) {
        return Err("Epic Effect callback history is not admitted".into());
    }
    if phase == Phase::Interpolate {
        return advance_fast(state, index);
    }
    if !state.sprite_effects[index].active {
        return Ok(());
    }
    let parent_position = if state.sprite_effects[index].variant == 31 {
        match state.sprite_effects[index].parent {
            None => None,
            Some(parent) if parent.actor.kind == ActorKind::Effect => {
                let parent = state
                    .sprite_effects
                    .iter()
                    .find(|e| e.id == parent.actor.id)
                    .ok_or("Epic Rocket Parent is unresolved")?;
                if !admitted(parent) || parent.variant != 30 {
                    frontier(state, "epicFetusRocketParentHistory");
                    return Ok(());
                }
                (!parent.removed).then_some(parent.position)
            }
            Some(_) => {
                frontier(state, "epicFetusRocketParentFamily");
                return Ok(());
            }
        }
    } else {
        None
    };
    {
        let effect = &mut state.sprite_effects[index];
        effect.full_updates = effect
            .full_updates
            .checked_add(1)
            .ok_or("Epic full count exhausted")?;
        effect
            .sprite
            .advance(definition(1000, effect.variant).ok_or("Epic Effect config absent")?)?;
        effect.velocity = effect.velocity * effect.current_friction;
        effect.position = effect.position + effect.velocity;
        effect.presented_position = effect.position;
        effect.current_friction = effect.configured_friction;
    }
    let variant = state.sprite_effects[index].variant;
    if variant == 30 {
        state.sprite_effects[index].current_friction *= 0.5;
        let Some(position) = room_clamp(state, state.sprite_effects[index].position)? else {
            frontier(state, "epicFetusTargetRoomBounds");
            return Ok(());
        };
        let effect = &mut state.sprite_effects[index];
        effect.position = position;
        effect.presented_position = position;
        if effect.removed {
            set_frame(effect, "Blink", 0);
            effect.active = false;
        } else {
            let ttl = effect.time_to_live;
            let period = (f64::from(ttl as f32).sqrt() as i32).max(2);
            set_frame(effect, "Blink", u32::from(ttl % period < period / 2));
            //45E66C and462674 query CURRENT FIRSTPlayer, after Blink.
            //5090DE..EA for IDs153/2 tests signed E1C counts >0 only;
            //neither query enters327/328's trinket/age branch or reads TEMP.
            let inputs = ItemInputs::new(
                state
                    .player_items
                    .as_ref()
                    .ok_or("FIRSTPlayer Epic target items absent")?,
            )?;
            if inputs.positive_owned_count(153) {
                frontier(state, "epicFetusCurrentFirstPlayerTargetModifiers");
                return Ok(());
            }
            let (trigger_ttl, rocket_count) = if inputs.positive_owned_count(2) {
                (30, 3)
            } else {
                (10, 1)
            };
            if ttl == trigger_ttl {
                //45E692..724 births the whole burst in source order. Each
                //rocket has its own Raw, lifetime and immediate Full; the
                //later strikes are not controller timers or extra targets.
                for shot in 1..=rocket_count {
                    let seed = state.boot_lifecycle.shared.next_u32();
                    let rocket = admit(state, 31, seed, position, shot * 10)?;
                    //45E713 invokes Full while Parent498 is still NULL.
                    advance(state, Phase::Update, rocket)?;
                    state
                        .sprite_effects
                        .iter_mut()
                        .find(|e| e.id == rocket)
                        .ok_or("new Epic Rocket disappeared before Parent setter")?
                        .parent = Some(Parent {
                        actor: ActorRef {
                            kind: ActorKind::Effect,
                            id,
                        },
                        entity_type: 1000,
                        variant: 30,
                    });
                }
            }
        }
    } else {
        let effect = &mut state.sprite_effects[index];
        effect.render_offset.y = (effect.time_to_live.wrapping_neg() as f32) * 100.0;
        if effect.time_to_live <= 2 {
            set_frame(effect, "Falling", 1);
        }
        if let Some(position) = parent_position {
            effect.position = position;
            effect.presented_position = position;
        }
        if effect.removed {
            let position = effect.position;
            let inputs = ItemInputs::new(
                state
                    .player_items
                    .as_ref()
                    .ok_or("FIRSTPlayer Epic items absent")?,
            )?;
            if state.weapon_admission.source_player_type != Some(0)
                || !inputs.admits_epic_fetus_kind6_context()
            {
                frontier(state, "epicFetusCurrentFirstPlayerKind6");
                return Ok(());
            }
            let properties = crate::weapon_plan::resolve_epic_fetus_kind6(
                inputs,
                state.weapon_admission.source_player_type,
                state.weapon_cache,
                state.player_stats,
            )?;
            let damage = properties.collision_damage * 20.0;
            let effect = &state.sprite_effects[index];
            //5547DC..554840 copies the actual Rocket EntityRef before V38.
            //These are current Rocket fields, independent of current Get6 and
            //of the target Parent or the later effect/actor retirement.
            let copied_source = crate::combat::CopiedDamageSource {
                entity_type: 1000,
                variant: effect.variant,
                spawner_type: effect.spawner_type,
                spawner_variant: effect.spawner_variant,
                position: effect.position,
                velocity: effect.velocity,
                actor_at_copy: Some(ActorRef {
                    kind: ActorKind::Effect,
                    id,
                }),
            };
            if !crate::explosion_effects::explode_neutral(
                state,
                position,
                damage,
                properties.flags,
                properties.color,
                crate::combat::DamageSource {
                    id,
                    entity_type: 1000,
                },
                copied_source,
            )? {
                return Ok(());
            }
            state.sprite_effects[index].active = false;
        }
    }
    ttl_tail(&mut state.sprite_effects[index]);
    Ok(())
}

fn advance_fast(state: &mut State, index: usize) -> Result<(), String> {
    {
        let effect = &mut state.sprite_effects[index];
        if effect.flags & 2 != 0 {
            effect.flags &= !2;
        } else {
            let owner = effect.epic_fetus.as_mut().ok_or("Epic Fast owner absent")?;
            //4446E9 saves CURRENT448 on every Fast, even after another Fast.
            owner.saved_position = Some(effect.position);
            effect.position = effect.position + (effect.velocity * effect.current_friction) * 0.5;
            owner.fast_pending = true;
        }
    }
    if state.sprite_effects[index].variant == 30 {
        let Some(clamped) = room_clamp(state, state.sprite_effects[index].position)? else {
            frontier(state, "epicFetusTargetFastRoomBounds");
            return Ok(());
        };
        state.sprite_effects[index].position = clamped;
    }
    state.sprite_effects[index].presented_position = state.sprite_effects[index].position;
    Ok(())
}

///System550860/445370 before PlayerFull, including actors already Remove'd.
pub(crate) fn restore_fast_positions(state: &mut State) -> Result<(), String> {
    for effect in &mut state.sprite_effects {
        if let Some(owner) = &mut effect.epic_fetus {
            if owner.fast_pending {
                effect.position = owner
                    .saved_position
                    .ok_or("Epic pending Fast has unknown source450")?;
                effect.presented_position = effect.position;
                owner.fast_pending = false;
            }
        }
    }
    Ok(())
}

fn uninit(state: &mut State, retired: &[u32]) {
    let retired_ref =
        |actor: ActorRef| actor.kind == ActorKind::Effect && retired.contains(&actor.id);
    for effect in &mut state.sprite_effects {
        if retired.contains(&effect.id) || effect.parent.is_some_and(|p| retired_ref(p.actor)) {
            effect.parent = None;
        }
    }
    if state.attack_runtime.persistent.is_some_and(retired_ref) {
        state.attack_runtime.persistent = None;
        state.weapon_admission.c18_is_known_empty = true;
    }
    for slot in &mut state.attack_runtime.secondary {
        if slot.is_some_and(retired_ref) {
            *slot = None;
        }
    }
}

pub(crate) fn retire_full(state: &mut State) {
    let retired: Vec<_> = state
        .sprite_effects
        .iter()
        .filter(|effect| effect.epic_fetus.is_some() && !effect.active)
        .map(|effect| effect.id)
        .collect();
    uninit(state, &retired);
    state
        .sprite_effects
        .retain(|effect| !retired.contains(&effect.id));
}

pub(crate) fn reset_room(state: &mut State) {
    let retired: Vec<_> = state
        .sprite_effects
        .iter()
        .filter(|effect| effect.epic_fetus.is_some())
        .map(|effect| effect.id)
        .collect();
    //453EC0 Uninit precedes virtualRemove for the transient pool.
    uninit(state, &retired);
    for effect in &mut state.sprite_effects {
        if retired.contains(&effect.id) {
            effect.active = false;
            effect.removed = true;
        }
    }
}
