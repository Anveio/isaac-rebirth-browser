//! Ordinary Player519FC0 charged-Brimstone consumers in the canonical store.
use crate::{
    actors::{ActorKind, ActorRef},
    lasers::{LaserContact, LaserTarget},
    phase::Phase,
    State,
};

pub(crate) use crate::laser_player_runtime::{
    admit_ordinary_brim, admit_ordinary_technology, advance_ordinary_brim_initial,
    advance_ordinary_technology_initial, OrdinaryBrimBirth, OrdinaryTechnologyBirth,
};

#[derive(Clone, Copy)]
enum LaserFrontier {
    ContactModifiers,
    TransformationHistory,
    EffectTargetFields,
    GridKind(u32),
}
fn record_frontier(state: &mut State, reason: LaserFrontier) {
    let channel = match reason {
        LaserFrontier::ContactModifiers => "ordinaryBrimstoneContactModifiers".into(),
        LaserFrontier::TransformationHistory => "ordinaryBrimstoneTransformationHistory".into(),
        LaserFrontier::EffectTargetFields => "ordinaryBrimstoneEffectTargetFields".into(),
        LaserFrontier::GridKind(kind) => format!("ordinaryBrimstoneGridKind{kind}"),
    };
    state
        .weapon_admission
        .record_unadmitted(state.weapon_cache.primary, &channel);
}

struct TargetQuery {
    targets: Vec<LaserTarget>,
    unadmitted_effect_fields: bool,
}

fn ordered_targets(state: &State) -> Result<TargetQuery, String> {
    let mut targets = Vec::new();
    let mut unadmitted_effect_fields = false;
    //Actual Room1638 query order is the current manager array, including
    //actors awaiting Uninit. Resolve every actor family before filtering.
    for actor in state.actor_scheduler.full_phase() {
        match actor.kind {
            ActorKind::Npc => {
                let npc = state
                    .room
                    .npcs
                    .iter()
                    .find(|npc| npc.id == actor.id)
                    .ok_or("Laser query NPC handle is unresolved")?;
                targets.push(LaserTarget {
                    actor,
                    entity_type: npc.kind,
                    variant: npc.variant,
                    entity_collision_class: npc.entity_collision_class,
                    raw_494: npc.raw_494,
                    dead: npc.dead,
                    entity_flags: npc.entity_flags(),
                    position: npc.position,
                    collision_radius: npc.collision_radius,
                });
            }
            ActorKind::Effect => {
                if let Some(effect) = state
                    .sprite_effects
                    .iter()
                    .find(|effect| effect.id == actor.id)
                {
                    if matches!(effect.variant, 21 | 33 | 63 | 64) {
                        unadmitted_effect_fields = true;
                    }
                } else if !state
                    .fire_particles
                    .iter()
                    .any(|effect| effect.id == actor.id)
                    && !crate::laser_npc_runtime::has_impact(state, actor.id)
                {
                    return Err("Laser query effect handle is unresolved".into());
                }
            }
            ActorKind::Player if actor.id != 0 => {
                return Err("Laser query requires actual admitted single Player".into());
            }
            ActorKind::Familiar => {
                if !state
                    .familiars
                    .iter()
                    .any(|actor_state| actor_state.id == actor.id)
                {
                    return Err("Laser query Familiar handle is unresolved".into());
                }
            }
            //Known source types1/2/4/5/7/8/9 do not enter the neutral target
            //set. Extra213 projectile/other-parent-family paths stay pending.
            ActorKind::Player
            | ActorKind::Tear
            | ActorKind::Bomb
            | ActorKind::Pickup
            | ActorKind::Laser
            | ActorKind::Knife
            | ActorKind::Projectile => {}
        }
    }
    Ok(TargetQuery {
        targets,
        unadmitted_effect_fields,
    })
}

fn contact_callbacks(state: &mut State, index: usize) -> Result<(), String> {
    //Validate actual numeric geometry and each reached NPC health owner before
    //a valid-unmapped history/status diagnostic can short-circuit callbacks.
    //This preflight does not mutate registry, health, queues, RNG or properties.
    let query = ordered_targets(state)?;
    let contacts = state.lasers[index].contacts(&query.targets)?;
    for contact in &contacts {
        if let LaserContact::Damage { target, .. } = contact {
            state
                .room
                .npcs
                .iter()
                .find(|npc| npc.id == target.id)
                .ok_or("Laser numeric preflight target is unresolved")?
                .health()
                .validate()?;
        }
    }
    //The pure kind3 admission also validates current Player stats/colors.
    //Compute it before history diagnostics, then retain the source-domain
    //diagnostic priority below. The real resolver still runs after enqueue.
    let transform_admission = crate::weapon_plan::admit_current_laser_contact_flags(state)?;
    let Some(counts) = state.player.transformation_counts else {
        record_frontier(state, LaserFrontier::TransformationHistory);
        return Ok(());
    };
    let owner = state
        .player_items
        .as_ref()
        .ok_or("Laser requires actual first Player items")?;
    let inputs = crate::item_pipeline::ItemInputs::new(owner)?;
    if counts[0] >= 3 {
        record_frontier(state, LaserFrontier::TransformationHistory);
        return Ok(());
    }
    if inputs.positive_owned_count(151)
        || inputs.positive_owned_count(213)
        || (!inputs.admits_neutral_kind3() && !inputs.admits_technology_cache_context())
    {
        record_frontier(state, LaserFrontier::ContactModifiers);
        return Ok(());
    }
    //This is the actual kind3 builder/status frontier, separate from476D70's
    //preceding transformation/owned151/213 damage-info branches. Numeric
    //geometry/HP validation in the actual target/overlap kernel precedes the
    //first queue write; valid unmapped modifiers preserve body continuation.
    if query.unadmitted_effect_fields {
        record_frontier(state, LaserFrontier::EffectTargetFields);
    }
    if transform_admission != crate::weapon_plan::LaserStatusAdmission::Mapped {
        record_frontier(state, LaserFrontier::ContactModifiers);
        return Ok(());
    }
    for contact in contacts {
        match contact {
            LaserContact::MarkEffectDead { target } => {
                if target.kind != ActorKind::Effect {
                    return Err("Laser direct31F callback requires actual Effect owner".into());
                }
                //No canonical main Effect31F target owner is currently admitted.
                record_frontier(state, LaserFrontier::EffectTargetFields);
            }
            LaserContact::Damage {
                target,
                source,
                amount,
            } => {
                if target.kind != ActorKind::Npc
                    || source
                        != (ActorRef {
                            kind: ActorKind::Player,
                            id: 0,
                        })
                {
                    return Err(
                        "Laser damage callback requires its actual NPC/Parent owners".into(),
                    );
                }
                let npc = state
                    .room
                    .npcs
                    .iter_mut()
                    .find(|npc| npc.id == target.id)
                    .ok_or("Laser contact target disappeared before its callback")?;
                let health = npc.health();
                //476A60 passes actual ParentPlayer DamageInfo, flags8 and
                //countdown30. Native ordinary queue keeps repeated segments.
                npc.damage.enqueue(
                    health,
                    crate::combat::DamageEvent {
                        amount,
                        kind: crate::combat::DamageKind::Laser,
                        source: crate::combat::DamageSource {
                            id: source.id,
                            entity_type: 1,
                        },
                    },
                )?;
                //476830 evaluates kind3 AFTER each real callback. Birth kind2
                //or cached flags cannot replace this current status resolver.
                let flags = crate::weapon_plan::resolve_current_laser_contact_flags(state)?;
                if flags.0 & !3 != 0 {
                    record_frontier(state, LaserFrontier::ContactModifiers);
                }
            }
        }
    }
    Ok(())
}

///5791F0 mode0's discarded query has no random demands or target writes.
///Validate every resolved actual numeric target before mutable birth/body work.
pub(crate) fn validate_mode0_query(state: &State) -> Result<(), String> {
    for target in ordered_targets(state)?.targets {
        if !target.position.is_finite()
            || !target.collision_radius.is_finite()
            || target.collision_radius < 0.0
        {
            return Err("Laser mode0 query has invalid actual target geometry".into());
        }
    }
    Ok(())
}

///Actual477BB0 grid callbacks, point simplification, then476D70/kind3.
pub(crate) fn run_line_contacts(
    state: &mut State,
    index: usize,
    grid: &crate::grid::Grid,
) -> Result<(), String> {
    state.lasers[index].validate_contact_geometry()?;
    for cell in state.lasers[index].grid_contact_indices(grid)? {
        let outcome = crate::attack_grid::callback_at_index(
            &mut state.room,
            cell,
            &mut state.boot_lifecycle.shared,
        )?;
        if let crate::attack_grid::GridCallbackOutcome::UnadmittedKind(kind) = outcome {
            record_frontier(state, LaserFrontier::GridKind(kind));
        }
    }
    state.lasers[index].finish_grid_contacts();
    contact_callbacks(state, index)
}

pub fn advance_actor(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    let origin = state
        .lasers
        .iter()
        .find(|laser| laser.id == id)
        .ok_or("scheduled Laser unresolved")?
        .origin;
    match origin {
        crate::lasers::LaserOrigin::NpcShootAngle => {
            crate::laser_npc_runtime::advance_actor(state, phase, id)
        }
        crate::lasers::LaserOrigin::PlayerCharged
        | crate::lasers::LaserOrigin::PlayerTechnology => {
            crate::laser_player_runtime::advance_actor(state, phase, id)
        }
    }
}
pub fn advance_impact(state: &mut State, phase: Phase, id: u32) -> Result<(), String> {
    if phase != Phase::Update {
        return Ok(());
    }
    let Some(index) = state
        .lasers
        .iter()
        .position(|laser| laser.impact.as_ref().is_some_and(|impact| impact.id == id))
    else {
        return Ok(());
    };
    if state.lasers[index].origin == crate::lasers::LaserOrigin::NpcShootAngle {
        return crate::laser_npc_runtime::advance_impact(state, phase, id);
    }
    let mut impact = state.lasers[index]
        .impact
        .clone()
        .ok_or("Impact identity unresolved")?;
    if !impact.parent_known {
        return Err("Player Impact actual Parent history absent".into());
    }
    crate::laser_npc_runtime::advance_impact_full(&mut impact, false)?;
    let parent_dead = match impact.parent {
        None => true,
        Some(parent)
            if parent
                == (ActorRef {
                    kind: ActorKind::Laser,
                    id: state.lasers[index].id,
                }) =>
        {
            state.lasers[index].dead
        }
        Some(_) => return Err("ordinary Impact actual Parent escaped admitted host".into()),
    };
    if parent_dead {
        impact.dead = true;
    }
    state.lasers[index].impact = Some(impact);
    Ok(())
}
pub(crate) fn restore_fast_positions(state: &mut State) {
    for laser in &mut state.lasers {
        if laser.origin.is_player() {
            laser.restore_fast_position();
        }
    }
}
pub(crate) fn uninit_actor(state: &mut State, actor: ActorRef) -> Result<(), String> {
    let laser = state
        .lasers
        .iter_mut()
        .find(|laser| laser.id == actor.id)
        .ok_or("Laser Uninit handle is unresolved")?;
    if !laser.origin.is_player() {
        return Err("Player Uninit received legacy NPC actor".into());
    }
    if laser.uninitialized == Some(true) {
        return Ok(());
    }
    if let Some(child) = laser.child {
        let impact = laser
            .impact
            .as_mut()
            .filter(|impact| child.kind == ActorKind::Effect && impact.id == child.id)
            .ok_or("Laser Uninit child identity is unresolved")?;
        if impact.parent == Some(actor) {
            impact.parent = None;
            impact.parent_known = true;
        }
        laser.child = None;
    }
    laser.parent = None;
    laser.uninitialized = Some(true);
    if state.attack_runtime.persistent == Some(actor) {
        state.attack_runtime.persistent = None;
        state.weapon_admission.c18_is_known_empty = true;
    }
    for slot in &mut state.attack_runtime.secondary {
        if *slot == Some(actor) {
            *slot = None;
        }
    }
    Ok(())
}

///Actual normal-list full callback retirement for the Player origin only.
///The old NPC both-phase adapter stays at its existing later caller.
pub(crate) fn retire_full(state: &mut State) -> Result<(), String> {
    let retired: Vec<_> = state
        .lasers
        .iter()
        .filter(|laser| {
            laser.origin.is_player() && !laser.active && laser.uninitialized == Some(false)
        })
        .map(|laser| ActorRef {
            kind: ActorKind::Laser,
            id: laser.id,
        })
        .collect();
    for actor in retired {
        uninit_actor(state, actor)?;
    }
    let mut released = Vec::new();
    for laser in &mut state.lasers {
        if !laser.origin.is_player() {
            continue;
        }
        if laser.uninitialized == Some(true) {
            released.push(ActorRef {
                kind: ActorKind::Laser,
                id: laser.id,
            });
        }
        if let Some(impact) = laser.impact.as_mut().filter(|impact| impact.removed) {
            let host = ActorRef {
                kind: ActorKind::Laser,
                id: laser.id,
            };
            if !impact.parent_known || impact.parent.is_some_and(|parent| parent != host) {
                return Err("Player Impact Uninit actual Parent history is unresolved".into());
            }
            let actor = ActorRef {
                kind: ActorKind::Effect,
                id: impact.id,
            };
            //448052 clears own Parent before448087 clears matching incoming refs.
            impact.parent = None;
            impact.parent_known = true;
            if laser.child == Some(actor) {
                laser.child = None;
            }
            released.push(actor);
        }
    }
    state
        .actor_scheduler
        .retain(|actor| !released.contains(&actor));
    state.lasers.retain(|laser| {
        !laser.origin.is_player()
            || laser.uninitialized != Some(true)
            || laser.impact.as_ref().is_some_and(|impact| !impact.removed)
    });
    Ok(())
}
pub(crate) fn reset_room(state: &mut State) -> Result<(), String> {
    let actors: Vec<_> = state
        .lasers
        .iter()
        .filter(|laser| laser.origin.is_player())
        .map(|laser| ActorRef {
            kind: ActorKind::Laser,
            id: laser.id,
        })
        .collect();
    for actor in actors {
        uninit_actor(state, actor)?;
    }
    state.lasers.clear();
    state
        .actor_scheduler
        .retain(|actor| actor.kind != ActorKind::Laser);
    Ok(())
}
