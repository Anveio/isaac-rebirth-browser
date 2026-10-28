//! Actual545E80 ballistic bit9 receiver registry and overkill continuation.
//! The born flag owns this callback even after inventory changes. Current
//! FIRSTPlayer proc/run inputs and current receiver HP remain separate reads.
use crate::{
    actors::{ActorKind, ActorRef},
    combat::{DamageEvent, DamageKind, DamageSource},
    projectiles::{self, Lifetime},
    weapon_plan::UnadmittedController,
    State,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TearHitRegistry {
    ///Actual Entity+8 values, independent of browser handles and HP/death.
    pub hit_sequences: BTreeSet<u32>,
}
impl TearHitRegistry {
    pub(crate) fn fresh() -> Self {
        Self {
            hit_sequences: BTreeSet::new(),
        }
    }
}

fn frontier(state: &mut State, channel: &str) {
    //A ballistic Tear callback does not write Player C18. Preserve that owner
    //while exposing an unimplemented independent contact query/history.
    state
        .weapon_admission
        .unadmitted_controller
        .get_or_insert(UnadmittedController {
            primary: state.weapon_cache.primary.numeric(),
            channel: channel.into(),
        });
}

///49B680's finite current velocity/animation predicate. Other NPC kinds
///return true directly. This is neither the incoming Player aim nor a queue
///drain; the direction-sensitive owners stay with the actual receiver.
fn accepts_direction(npc: &crate::npcs::Npc, incoming: crate::motion::WorldVector) -> bool {
    let direction = match npc.kind {
        97 => {
            if npc.velocity.x.abs() > npc.velocity.y.abs() {
                if npc.velocity.x < 0.0 {
                    0
                } else {
                    2
                }
            } else if npc.velocity.y < 0.0 {
                1
            } else {
                3
            }
        }
        41 | 254 => match npc.animation.as_str() {
            "Up" => 1,
            "Down" => 3,
            _ if npc.flip_x => 0,
            _ => 2,
        },
        _ => return true,
    };
    match direction {
        0 => incoming.x <= 0.0,
        1 => incoming.y <= 0.0,
        2 => incoming.x >= 0.0,
        3 => incoming.y >= 0.0,
        _ => unreachable!("source direction is bounded to0..3"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContactPass {
    Unused,
    Handled,
    ///Poly owns an explicit unknown context. Preserve existing regular and
    ///controlled callbacks; only the unadmitted Poly actor is withheld.
    Unadmitted,
}

///Snapshot source partition membership in actual manager insertion order.
///The shared primitive emits global pairs, including cross-Tear interleaving.
///Each actual Poly pair rechecks the current circle after earlier callbacks.
pub(crate) fn contacts_all(state: &mut State) -> Result<ContactPass, String> {
    use crate::contact_candidates::{self, Bounds};
    if !state.tears.iter().any(|tear| {
        tear.controlled.is_none()
            && tear.attack_flags.0 == 0x200
            && tear.motion.lifetime == Lifetime::Alive
    }) {
        return Ok(ContactPass::Unused);
    }
    //Every actual member affects bucket capacity and global pair order,
    //including currently dead actors. Do not invent unknown Tear geometry.
    if state.tears.iter().any(|tear| {
        tear.controlled.is_some()
            || tear.source_variant != Some(0)
            || !matches!(tear.attack_flags.0, 0 | 0x200)
    }) {
        frontier(state, "ballisticTearMixedContactGeometryHistory");
        return Ok(ContactPass::Unadmitted);
    }
    //Only these fresh ordinary receiver owners have closed unit478/47C
    //bounds here. Unsupported/champion geometry never receives a default1.
    if state.room.npcs.iter().any(|npc| {
        !npc.removed
            && npc.entity_collision_class != 0
            && (!matches!(npc.kind, 14 | 18 | 217)
                || npc.variant != 0
                || npc.subtype != 0
                || npc.boss
                || npc.champion.is_some())
    }) {
        frontier(state, "ballisticTearContactGeometryHistory");
        return Ok(ContactPass::Unadmitted);
    }
    let mut tears = Vec::new();
    let mut npcs = Vec::new();
    let mut index = 0;
    while let Some(actor) = state.actor_scheduler.get(index) {
        index += 1;
        match actor.kind {
            ActorKind::Tear => {
                if let Some(tear) = state.tears.iter().find(|tear| tear.id == actor.id) {
                    //Source4545D0 inserts nonzero-class members before its
                    //callback dead31F guard. Selected Tear Init owns class4.
                    tears.push(Bounds {
                        id: tear.id,
                        position: tear.motion.position,
                        half_extents: crate::motion::WorldVector {
                            x: tear.collision_radius,
                            y: tear.collision_radius,
                        },
                    });
                }
            }
            ActorKind::Npc => {
                if let Some(npc) = state.room.npcs.iter().find(|npc| npc.id == actor.id) {
                    if !npc.removed && npc.entity_collision_class != 0 {
                        //The explicit selected receiver owners above bind
                        //unit478/47C. Sprite scale is not applied here.
                        npcs.push(Bounds {
                            id: npc.id,
                            position: npc.position,
                            half_extents: crate::motion::WorldVector {
                                x: npc.collision_radius,
                                y: npc.collision_radius,
                            },
                        });
                    }
                }
            }
            _ => {}
        }
    }
    for (tear, receiver) in contact_candidates::pairs(&tears, &npcs)? {
        let Some(index) = state.tears.iter().position(|actor| actor.id == tear) else {
            continue;
        };
        let actor = &state.tears[index];
        if actor.motion.lifetime != Lifetime::Alive {
            continue;
        }
        if actor.attack_flags.0 == 0x200 {
            contacts_pair(state, tear, receiver)?;
        } else if actor.attack_flags.0 == 0 {
            //A regular Tear born before Add169 remains a real independent
            //actor. Its old callback executes at THIS global pair position.
            let mut motion = actor.motion;
            let radius = actor.collision_radius;
            let damage = actor.collision_damage;
            if crate::npcs::tear_hit_target(state, &mut motion, radius, tear, damage, receiver)?
                .is_some()
            {
                let actor = &mut state.tears[index];
                motion.lifetime = Lifetime::Dying;
                actor.motion = motion;
                actor.project();
            }
        } else {
            frontier(state, "ballisticTearMixedContactFlags");
        }
    }
    Ok(ContactPass::Handled)
}

///Selected545E80 bit9 callback only. The concrete result1 vetoes all
///ordinary mass response; the global owner handles independent flag0 pairs.
fn contacts_pair(state: &mut State, id: u32, target_id: u32) -> Result<(), String> {
    let Some(index) = state.tears.iter().position(|tear| tear.id == id) else {
        return Ok(());
    };
    let Some(receiver) = state.room.npcs.iter().position(|npc| npc.id == target_id) else {
        return Ok(());
    };
    let tear = &state.tears[index];
    if tear.controlled.is_some()
        || tear.attack_flags.0 != 0x200
        || tear.motion.lifetime != Lifetime::Alive
    {
        return Ok(());
    }
    let npc = &state.room.npcs[receiver];
    //4453E7..4453FE tests actual31F on both actors; class4/4 is admitted
    //by4453B6..445490, and class2 meets a tear at 445478. HP does not
    //decide callback eligibility.
    if npc.dead || npc.removed || !matches!(npc.entity_collision_class, 2 | 4) {
        return Ok(());
    }
    //445710's strict circle check uses CURRENT radius and P after any
    //earlier contact, while partition membership remains the snapshot.
    let position = tear.motion.position;
    let radius = tear.collision_radius;
    let dx = position.x - npc.position.x;
    let dy = position.y - npc.position.y;
    let combined = radius + npc.collision_radius;
    if dx.abs() > combined || dy.abs() > combined || (dx * dx + dy * dy).sqrt() >= combined {
        return Ok(());
    }
    let Some(sequence) = state.actor_scheduler.sequence_id(ActorRef {
        kind: ActorKind::Npc,
        id: npc.id,
    }) else {
        frontier(state, "ballisticTearContactIdentity");
        return Ok(());
    };
    let Some(registry) = state.tears[index].persistent_contacts.as_mut() else {
        frontier(state, "ballisticTearRegistryHistory");
        return Ok(());
    };
    if !registry.hit_sequences.insert(sequence) {
        //545EF3's mask includes bit9: repeat returns1 before damage.
        return Ok(());
    }
    //54612E/511D90(0) and5461AF read CURRENT FIRSTPlayer ownership.
    //These capabilities are independent of cache, kind1 and born169.
    if !state
        .player
        .transformation_counts
        .is_some_and(|counts| counts[0] < 3)
        || !state
            .player_items
            .as_ref()
            .is_some_and(|items| items.owned_counts.get(151) == Some(&0))
    {
        frontier(state, "ballisticTearContactTransformationHistory");
        return Ok(());
    }
    if !state
        .source_cache_run_flags
        .is_some_and(|flags| flags & 8 == 0)
    {
        frontier(state, "ballisticTearContactRunFlagsHistory");
        return Ok(());
    }
    let tear = &state.tears[index];
    let damage = tear.collision_damage;
    let incoming = tear.motion.velocity;
    let scale = tear.scale;
    //5178A9 ->550AC8..550ADA binds these spawner words at this real
    //Player factory birth. P/V and variant are read from the ACTUAL Tear at
    //this callback; later inventory/current Player values do not supply them.
    let copied_source = crate::combat::CopiedDamageSource {
        entity_type: 2,
        variant: tear
            .source_variant
            .ok_or("tracked ballistic Tear variant absent")?,
        spawner_type: 1,
        spawner_variant: 0,
        position: tear.motion.position,
        velocity: tear.motion.velocity,
        actor_at_copy: Some(ActorRef {
            kind: ActorKind::Tear,
            id,
        }),
    };
    //445423 invokes the receiver's second(false) callback even after the
    //Tear returns1. These source49B8F0 paths have no effect; other actual
    //receiver callbacks retain a separate observable frontier below.
    let reciprocal_callback_closed = matches!(state.room.npcs[receiver].kind, 14 | 18 | 217)
        && state.room.npcs[receiver].entity_flags() & 0x100 == 0
        && state
            .floor
            .descriptors
            .get(state.current_room)
            .is_some_and(|descriptor| descriptor.template.room_type == 1);
    let npc = &mut state.room.npcs[receiver];
    //446CF0 appends the actual Tear source and old current48C. It never
    //writes HP484; source545E80 ignores the enqueue return value here.
    npc.damage.enqueue(
        npc.health(),
        DamageEvent {
            amount: damage,
            kind: DamageKind::Tear,
            source: DamageSource { id, entity_type: 2 },
            copied_source: Some(copied_source),
        },
    )?;
    //5468AF..546915 reads current HP AFTER that callback, before the
    //receiver's later Full drain. Pending damage is not deducted here.
    if damage > npc.hp && accepts_direction(npc, incoming) {
        let remaining = damage - npc.hp;
        let next_scale = ((remaining * scale) / damage).max(0.01_f32);
        let tier = projectiles::regular_animation_tier(next_scale)?;
        let radius = projectiles::collision_radius(next_scale, 8.0_f32)?;
        let tear = &mut state.tears[index];
        tear.collision_damage = remaining;
        tear.scale = next_scale;
        tear.animation_tier = tier;
        tear.collision_radius = radius;
        //541FF0 updates7E8/7E4 and clip/radius; it never writes83C.
    } else {
        //54698C sets31F. Exact equality takes this branch too.
        state.tears[index].motion.lifetime = Lifetime::Dying;
    }
    if !reciprocal_callback_closed {
        //This point follows the real Tear callback. No unknown receiver
        //effect is claimed as a no-op and no Player C18 owner is touched.
        frontier(state, "ballisticTearReceiverCallback");
    }
    Ok(())
}
