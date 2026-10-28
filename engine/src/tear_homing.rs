//! Homing tears (flag bit 2, Spoon Bender 3): Tear.Update 542530 at
//! 542675, before the spectral branch (542710) and the integration 447190.
//!
//! Without a target (+4A8), 5445D8 walks Room+1638 and, for each entity that
//! passes 444810, draws Random(2) on the shared stream. A candidate is taken
//! when that draw is nonzero, 444890(period 5, own Entity+8) admits this
//! update, CheckLine 5C74A0(mode 3) is clear from the tear to it, the tear's
//! position five updates ahead lies within 100 units of it, and its Entity+8
//! is not in the tear's 808/818 hit registry. 447110 then stores it; the walk
//! goes on, so the last accepted candidate stays the target.
//!
//! With a target, 542687 draws Random(2); on zero 543777 steers: velocity
//! minus normalize(position - target) * 2, then times the homing friction
//! +7E0 (0.85 from Init 542467).
use crate::{
    actors::{ActorKind, ActorRef},
    knives::{normalized, squared_length},
    motion::WorldVector,
    projectiles::Lifetime,
    target_eligibility::{eligible_target, TargetEligibility},
    State,
};

const HOMING: u64 = 0x4;

/// 6129D0 / 612A10: a - b per f32 component.
fn sub(a: WorldVector, b: WorldVector) -> WorldVector {
    WorldVector {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}
/// Tear+7E0 from Init 542467.
const HOMING_FRICTION: f32 = f32::from_bits(0x3f59_999a);

pub(crate) fn update(state: &mut State, id: u32) -> Result<(), String> {
    let Some(tear) = state.tears.iter().find(|tear| {
        tear.id == id
            && tear.controlled.is_none()
            && tear.motion.lifetime == Lifetime::Alive
            && tear.attack_flags.0 & HOMING != 0
    }) else {
        return Ok(());
    };
    let (position, velocity, target) = (
        tear.motion.position,
        tear.motion.velocity,
        tear.homing_target,
    );
    if let Some(target) = target {
        let target_position = state
            .room
            .npcs
            .iter()
            .find(|npc| npc.id == target)
            .map(|npc| npc.position)
            .ok_or("homing tear target +4A8 names no NPC")?;
        if state.boot_lifecycle.shared.bounded(2) != 0 {
            return Ok(());
        }
        let steer = normalized(sub(position, target_position)) * 2.0;
        let tear = state
            .tears
            .iter_mut()
            .find(|tear| tear.id == id)
            .ok_or("homing tear vanished")?;
        tear.motion.velocity = sub(velocity, steer) * HOMING_FRICTION;
        if !tear.motion.velocity.is_finite() {
            return Err("homing tear velocity overflowed".into());
        }
        return Ok(());
    }
    let own_sequence = state
        .actor_scheduler
        .sequence_id(ActorRef {
            kind: ActorKind::Tear,
            id,
        })
        .ok_or("homing tear has no Entity+8")?;
    let registry = state
        .tears
        .iter()
        .find(|tear| tear.id == id)
        .and_then(|tear| tear.persistent_contacts.as_ref())
        .map(|registry| registry.hit_sequences.clone())
        .unwrap_or_default();
    let ahead = position + velocity * 5.0;
    let grid = state
        .room
        .grid
        .clone()
        .map(crate::grid::Grid::new)
        .transpose()?;
    let counter = state.native_game_frame();
    let mut chosen = None;
    for index in 0..state.room.npcs.len() {
        let npc = &state.room.npcs[index];
        if npc.removed {
            continue;
        }
        let eligible = eligible_target(TargetEligibility {
            kind: npc.kind,
            collision_class: npc.entity_collision_class,
            raw_494: npc.raw_494,
            dead: npc.dead,
            entity_flags: crate::entities::family(npc.kind)
                .map(|family| (family.entity_flags)(npc)),
        })?;
        if !eligible {
            continue;
        }
        let (npc_id, npc_position) = (npc.id, npc.position);
        let distance = squared_length(sub(ahead, npc_position));
        if state.boot_lifecycle.shared.bounded(2) == 0 {
            continue;
        }
        // 444890: (Entity+8 + Game+CE9F4) % 5 == 0; ordinary tears carry
        // no Entity+318 bit 0x4000.
        if (counter.wrapping_add(own_sequence) as i32) % 5 != 0 {
            continue;
        }
        let clear = match (&grid, &state.room.path_costs) {
            (Some(grid), Some(costs)) => grid.projectile_line_clear(position, npc_position, costs),
            _ => return Err("homing tear CheckLine requires the room grid".into()),
        };
        if !clear || distance >= 10000.0 {
            continue;
        }
        let sequence = state
            .actor_scheduler
            .sequence_id(ActorRef {
                kind: ActorKind::Npc,
                id: npc_id,
            })
            .ok_or("homing candidate has no Entity+8")?;
        if !registry.contains(&sequence) {
            chosen = Some(npc_id);
        }
    }
    if let Some(target) = chosen {
        if let Some(tear) = state.tears.iter_mut().find(|tear| tear.id == id) {
            tear.homing_target = Some(target);
        }
    }
    Ok(())
}

/// 448040 nulls Tear+4A8 when its NPC is purged.
pub(crate) fn unlink_removed(state: &mut State) {
    let removed: Vec<u32> = state
        .room
        .npcs
        .iter()
        .filter(|npc| npc.removed)
        .map(|npc| npc.id)
        .collect();
    for tear in &mut state.tears {
        if tear
            .homing_target
            .is_some_and(|target| removed.contains(&target))
        {
            tear.homing_target = None;
        }
    }
}
