//! Familiar contacts: the manager pass 454940 pairs the Familiar partition
//! (+0) with projectiles (+18), then NPCs and bombs (+10), before any tear
//! reaches an NPC. Each overlapping pair runs 4453A0: the Familiar's
//! collision callback 46FA50(other, true) first.
//!
//! 46FA50 for a vulnerable enemy (444810): with collision damage above 0,
//! an enemy not yet in the hit-set F98 is inserted and takes
//! TakeDamage(BFFS x damage, 0, source, 30). The Update tail clears the set
//! every `hit_interval` frames (444890). For a projectile, a blocking
//! variant sets its remove flag; Dry Baby first rolls Random(4) and with 0
//! and a spent F58 plays Hit and takes F58 = 150 (470198).
use crate::{
    actors::{ActorKind, ActorRef},
    combat,
    contact_candidates::{self, Bounds},
    familiar_variants,
    motion::WorldVector,
    State,
};

/// 444890(interval, own Entity+8): whether the current Game frame plus the
/// entity index is a multiple of the interval.
pub(crate) fn is_frame(frame: u32, interval: u32, index: u32) -> bool {
    interval != 0 && (frame as i32).wrapping_add(index as i32) % interval as i32 == 0
}

fn bounds(id: u32, position: WorldVector, radius: f32) -> Bounds {
    Bounds {
        id,
        position,
        half_extents: WorldVector {
            x: radius,
            y: radius,
        },
    }
}

/// 4453A0's class test for a Familiar (class 4) against `other_class`:
/// classes 0 and 1 never pair with it.
fn pairs_with(other_class: u32) -> bool {
    !matches!(other_class, 0 | 1)
}

fn familiar_bounds(state: &State) -> Vec<Bounds> {
    state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Familiar)
        .filter_map(|actor| {
            state
                .familiars
                .iter()
                .find(|familiar| familiar.id == actor.id && familiar.active && !familiar.removed)
        })
        .filter(|familiar| familiar.entity_collision_class != 0)
        .map(|familiar| bounds(familiar.id, familiar.position, familiar.collision_radius))
        .collect()
}

fn overlap(a: WorldVector, ra: f32, b: WorldVector, rb: f32) -> bool {
    let delta = WorldVector {
        x: b.x - a.x,
        y: b.y - a.y,
    };
    let radius = ra + rb;
    if delta.x.abs() > radius || delta.y.abs() > radius {
        return false;
    }
    (delta.x * delta.x + delta.y * delta.y).sqrt() < radius
}

/// 46FA64: the contact multiplier is 2 with BFFS! (247) held, or with Hive
/// Mind (248) for Blue Flies 43 and Blue Spiders 73.
fn multiplier(state: &State, variant: u32) -> f32 {
    let item = if matches!(variant, 43 | 73) { 248 } else { 247 };
    let held = state
        .player_items
        .as_ref()
        .is_some_and(|owner| owner.owned_counts.get(item).copied().unwrap_or(0) > 0);
    if held {
        2.0
    } else {
        1.0
    }
}

/// The Familiar side of the contact pass, at 454938/454949.
pub(crate) fn contacts(state: &mut State, npc_partition: &[Bounds]) -> Result<(), String> {
    if state.familiars.is_empty() {
        return Ok(());
    }
    projectile_contacts(state)?;
    npc_contacts(state, npc_partition)
}

fn projectile_contacts(state: &mut State) -> Result<(), String> {
    let familiars = familiar_bounds(state);
    let projectiles: Vec<Bounds> = state
        .enemy_projectiles
        .iter()
        .filter(|projectile| !projectile.dead && !projectile.removed)
        .map(|projectile| {
            bounds(
                projectile.id,
                projectile.position,
                projectile.collision_radius,
            )
        })
        .collect();
    if familiars.is_empty() || projectiles.is_empty() {
        return Ok(());
    }
    for (familiar_id, projectile_id) in contact_candidates::pairs(&familiars, &projectiles)? {
        let Some(index) = state.familiars.iter().position(|f| f.id == familiar_id) else {
            continue;
        };
        let Some(target) = state
            .enemy_projectiles
            .iter()
            .position(|projectile| projectile.id == projectile_id)
        else {
            continue;
        };
        let familiar = &state.familiars[index];
        let projectile = &state.enemy_projectiles[target];
        if familiar.removed
            || projectile.dead
            || projectile.removed
            || !pairs_with(projectile.entity_collision_class)
            || !overlap(
                familiar.position,
                familiar.collision_radius,
                projectile.position,
                projectile.collision_radius,
            )
        {
            continue;
        }
        let variant = familiar.variant;
        // 470188: Smart Fly blocks only with Entity+18 = 0.
        if !familiar_variants::blocks_projectiles(variant)
            || (variant == 50 && familiar.subtype != 0)
        {
            continue;
        }
        if variant == 51 {
            // 46FAF8: Random(4) on the shared stream, then the Hit proc.
            let roll = state.boot_lifecycle.shared.bounded(4);
            let familiar = &mut state.familiars[index];
            if roll == 0 && familiar.fire_cooldown <= 0 {
                crate::familiar_ai::play(familiar, "Hit");
                familiar.fire_cooldown = 150;
            }
        }
        // 46FB26: the projectile's +31F remove flag.
        state.enemy_projectiles[target].dead = true;
    }
    Ok(())
}

/// Against the NPC/Bomb partition (+10) as 4545D0 built it for this pass
/// (`npcs::contact_partition`); 4453A0 skips a dead NPC.
fn npc_contacts(state: &mut State, npcs: &[Bounds]) -> Result<(), String> {
    let familiars = familiar_bounds(state);
    if familiars.is_empty() || npcs.is_empty() {
        return Ok(());
    }
    for (familiar_id, npc_id) in contact_candidates::pairs(&familiars, npcs)? {
        let Some(index) = state.familiars.iter().position(|f| f.id == familiar_id) else {
            continue;
        };
        let Some(target) = state.room.npcs.iter().position(|npc| npc.id == npc_id) else {
            continue;
        };
        let familiar = &state.familiars[index];
        let npc = &state.room.npcs[target];
        if familiar.removed
            || npc.dead
            || npc.removed
            || !pairs_with(npc.entity_collision_class)
            || !overlap(
                familiar.position,
                familiar.collision_radius,
                npc.position,
                npc.collision_radius,
            )
        {
            continue;
        }
        // 46FAD0: 444810, then collision damage +48C above 0 (or Halo of
        // Flies 33).
        let vulnerable = crate::target_eligibility::eligible_target(
            crate::target_eligibility::TargetEligibility {
                kind: npc.kind,
                collision_class: npc.entity_collision_class,
                raw_494: npc.raw_494,
                dead: npc.dead,
                entity_flags: crate::entities::family(npc.kind).map(|_| npc.entity_flags()),
            },
        )?;
        if !vulnerable || !(familiar.collision_damage > 0.0 || familiar.variant == 33) {
            continue;
        }
        let Some(sequence) = state.actor_scheduler.sequence_id(ActorRef {
            kind: ActorKind::Npc,
            id: npc_id,
        }) else {
            continue;
        };
        if familiar.hits.contains(&sequence) {
            continue;
        }
        let amount = multiplier(state, familiar.variant) * familiar.collision_damage;
        state.familiars[index].hits.push(sequence);
        let npc = &mut state.room.npcs[target];
        let health = npc.health();
        npc.damage.enqueue(
            health,
            combat::DamageEvent {
                amount,
                kind: combat::DamageKind::Familiar,
                source: combat::DamageSource {
                    id: familiar_id,
                    entity_type: 3,
                },
                copied_source: None,
            },
        )?;
    }
    Ok(())
}
