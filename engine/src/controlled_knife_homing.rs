//! Controlled bit33+4 uses the previous mask8 partition and strict nearest
//! acquisition. Ordinary Tear's manager scan/last-write policy is independent.
use crate::{
    actors::{ActorKind, ActorRef},
    contact_candidates::{Bounds, QuerySnapshot},
    knives::Knife,
    motion::WorldVector,
    target_eligibility::{eligible_target, TargetEligibility},
    State,
};

const RADIUS: f32 = 150.0;
const DISTANCE_SQUARED: f32 = 22_500.0;

/// A transient read plan: no mutation or target permission comes from counts.
/// The actual actor flag/registered slot selects the source branch.
pub(crate) enum Step {
    Disabled,
    Retained {
        actor: ActorRef,
        position: WorldVector,
    },
    Acquire(Option<ActorRef>),
}

///4545D0 runs after fenced Full actors/Uninit and before contact callbacks.
/// Unsupported member geometry invalidates this query owner alone. Ineligible
/// members still affect source bucket capacity; they cannot be filtered here.
pub(crate) fn rebuild_partition(state: &mut State) {
    state.knife_query_partition = partition_members(state)
        .and_then(|members| QuerySnapshot::rebuild(state.game_update_count, &members).ok());
}

fn partition_members(state: &State) -> Option<Vec<(ActorRef, Bounds)>> {
    let mut members = Vec::new();
    for actor in state.actor_scheduler.full_phase() {
        match actor.kind {
            ActorKind::Npc => {
                let npc = state.room.npcs.iter().find(|npc| npc.id == actor.id)?;
                //45483C/444800. Dead31F and class1 remain real members.
                if npc.entity_collision_class == 0 || !(10..=999).contains(&npc.kind) {
                    continue;
                }
                //Only these source owners currently prove actual unit478/47C.
                //Do not borrow visual/sprite scale or a family fallback.
                if !matches!(npc.kind, 14 | 18 | 217)
                    || npc.variant != 0
                    || npc.subtype != 0
                    || npc.boss
                    || npc.champion.is_some()
                    || !npc.position.is_finite()
                    || !npc.collision_radius.is_finite()
                    || npc.collision_radius < 0.0
                {
                    return None;
                }
                members.push((
                    actor,
                    Bounds {
                        id: 0,
                        position: npc.position,
                        half_extents: WorldVector {
                            x: npc.collision_radius,
                            y: npc.collision_radius,
                        },
                    },
                ));
            }
            //454B53 inserts actual type4 into this same mask8 partition.
            //Its source AABB owner is separate; omitting it changes cap40/order.
            ActorKind::Bomb => return None,
            //Slot6 membership in this partition is not decoded.
            ActorKind::Slot => return None,
            _ => {}
        }
    }
    Some(members)
}

///None is an explicit unavailable source plan, never Disabled or an empty
///query. Its caller records the frontier before mutating this actor.
pub(crate) fn prepare(state: &State, knife: &Knife) -> Result<Option<Step>, String> {
    if knife.attack_flags.0 & 4 == 0 {
        return Ok(Some(Step::Disabled));
    }
    if !knife.homing_target_known {
        return Ok(None);
    }
    if let Some(actor) = knife.homing_target {
        if actor.kind != ActorKind::Npc || state.actor_scheduler.sequence_id(actor).is_none() {
            return Ok(None);
        }
        let Some(npc) = state.room.npcs.iter().find(|npc| npc.id == actor.id) else {
            return Ok(None);
        };
        if !npc.position.is_finite() {
            return Err("controlled Knife target position is nonfinite".into());
        }
        //Retained steering does not rerun acquisition eligibility/HP rules.
        return Ok(Some(Step::Retained {
            actor,
            position: npc.position,
        }));
    }
    let Some(snapshot) = state.knife_query_partition.as_ref() else {
        return Ok(None);
    };
    if snapshot.game_counter > state.game_update_count {
        return Err("controlled Knife query epoch is ahead of current Game".into());
    }
    let mut nearest = None;
    let mut bound = DISTANCE_SQUARED;
    for actor in snapshot.query(knife.position, RADIUS)? {
        //The mask8 includes Bombs, but its current geometry is not owned yet.
        if actor.kind != ActorKind::Npc {
            return Ok(None);
        }
        let Some(npc) = state.room.npcs.iter().find(|npc| npc.id == actor.id) else {
            return Ok(None);
        };
        if !npc.position.is_finite()
            || !npc.collision_radius.is_finite()
            || npc.collision_radius < 0.0
        {
            return Err("controlled Knife query current circle is invalid".into());
        }
        let delta = WorldVector {
            x: knife.position.x - npc.position.x,
            y: knife.position.y - npc.position.y,
        };
        let squared = crate::knives::squared_length(delta);
        let radius = RADIUS + npc.collision_radius;
        //453790..453803 preserves strict current-circle filtering before444810.
        if squared >= radius * radius {
            continue;
        }
        let Ok(eligible) = eligible_target(TargetEligibility {
            kind: npc.kind,
            collision_class: npc.entity_collision_class,
            raw_494: npc.raw_494,
            dead: npc.dead,
            entity_flags: crate::entities::family(npc.kind).map(|f| (f.entity_flags)(npc)),
        }) else {
            //444810's only error is its unowned Entity318 input. Do not
            //filter that candidate or borrow a default family flag.
            return Ok(None);
        };
        if !eligible {
            continue;
        }
        let pending = npc.damage.pending_damage();
        if !pending.is_finite() || !npc.hp.is_finite() || !squared.is_finite() {
            return Err("controlled Knife target damage/distance owner is invalid".into());
        }
        //444950 is pending378 > HP484. Equality remains eligible. Strict
        //nearest comparison retains the first query candidate for exact ties.
        if pending > npc.hp || squared >= bound {
            continue;
        }
        bound = squared;
        nearest = Some(actor);
    }
    Ok(Some(Step::Acquire(nearest)))
}

///473615 follows rotation/flip and precedes room clamp/satellite formation.
pub(crate) fn apply(knife: &mut Knife, step: Step) -> Result<(), String> {
    match step {
        Step::Disabled => {
            if knife.attack_flags.0 & 4 != 0 {
                return Err("controlled Knife homing read plan is missing".into());
            }
        }
        Step::Acquire(target) => {
            if knife.attack_flags.0 & 4 == 0
                || !knife.homing_target_known
                || knife.homing_target.is_some()
            {
                return Err("controlled Knife acquisition slot changed after preflight".into());
            }
            //474C98→447110. A fresh acquisition does not steer this call.
            knife.set_homing_target(target);
        }
        Step::Retained { actor, position } => {
            if knife.attack_flags.0 & 4 == 0
                || !knife.homing_target_known
                || knife.homing_target != Some(actor)
            {
                return Err("controlled Knife retained target changed after preflight".into());
            }
            let away = WorldVector {
                x: knife.position.x - position.x,
                y: knife.position.y - position.y,
            };
            let unit = crate::knives::normalized(away) * 1.0;
            //Exact612A10 is componentwise SUBSS, not source6129B0 addition.
            knife.velocity = WorldVector {
                x: knife.velocity.x - unit.x,
                y: knife.velocity.y - unit.y,
            };
            if crate::knives::squared_length(away) > DISTANCE_SQUARED {
                //4736E7→447110(NULL) occurs AFTER this call's steering.
                knife.set_homing_target(None);
            }
        }
    }
    Ok(())
}

///The canonical NPC purge is the existing448040 owner, as for Tear4A8.
///Do not clear on dead/Remove before actual registered-slot retirement.
pub(crate) fn unlink_removed_targets(state: &mut State) {
    crate::tear_homing::unlink_removed(state);
    let retired: Vec<_> = state
        .room
        .npcs
        .iter()
        .filter(|npc| npc.removed)
        .map(|npc| ActorRef {
            kind: ActorKind::Npc,
            id: npc.id,
        })
        .collect();
    for knife in &mut state.knives {
        if knife.homing_target_known
            && knife
                .homing_target
                .is_some_and(|target| retired.contains(&target))
        {
            knife.set_homing_target(None);
        }
    }
    //Current removed bodies will disappear from the canonical store below.
    //A later query cannot reconstruct stale bucket pointer membership from Vec.
    //Keep this dependent owner unknown until the next real4545D0 rebuild.
    if state
        .knife_query_partition
        .as_ref()
        .is_some_and(|snapshot| retired.iter().any(|&actor| snapshot.contains(actor)))
    {
        state.knife_query_partition = None;
    }
}
