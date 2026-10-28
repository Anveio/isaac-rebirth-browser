//! Port-only debug sandbox floor (cargo feature `debug-sandbox`, never in a
//! release build). Base Rebirth has no item room, so a playtest build can lay
//! out a page of collectible pedestals in the current room and
//! spawn a chosen enemy, then record the set as a live original-vs-port case
//! (`npm run sandbox:case`). Nothing here is evidence of original behaviour.
use crate::{
    actors::{ActorKind, ActorRef},
    motion::WorldVector,
    pickups::Pickup,
    room_spawns::{SpawnCategory, SpawnSelector},
    State,
};

/// Marker the release bundle check (tools/e2e/policy.mjs) rejects.
pub const MARKER: &str = "debug-sandbox-floor";

fn grid(state: &State) -> Result<(usize, usize), String> {
    let grid = state
        .room
        .grid
        .as_ref()
        .ok_or("debug sandbox requires a room grid")?;
    Ok((usize::from(grid.width), usize::from(grid.height)))
}

fn cell(width: usize, column: usize, row: usize) -> WorldVector {
    let index = row * width + column;
    WorldVector {
        x: (40 + index % width * 40) as f32,
        y: (120 + index / width * 40) as f32,
    }
}

/// Pedestal slots: every other interior column on the first and third
/// interior rows, skipping any cell within two tiles of Isaac.
pub fn slots(state: &State) -> Result<Vec<WorldVector>, String> {
    let (width, height) = grid(state)?;
    let isaac = WorldVector {
        x: ((state.player.x - 32.0) / 0.65) as f32,
        y: ((state.player.y + 46.0) / 0.65) as f32,
    };
    let mut output = Vec::new();
    for row in [1, 3].into_iter().filter(|row| row + 1 < height) {
        for column in (1..width.saturating_sub(1)).step_by(2) {
            let position = cell(width, column, row);
            let (dx, dy) = (position.x - isaac.x, position.y - isaac.y);
            if dx * dx + dy * dy > 6400.0 {
                output.push(position);
            }
        }
    }
    Ok(output)
}

/// Remove every collectible pedestal not yet taken.
pub fn clear(state: &mut State) {
    for pickup in &mut state.room.pickups {
        if pickup.variant == 100 && !pickup.collected {
            pickup.removed = true;
        }
    }
}

/// A collectible pedestal (5.100) at a slot. Loose trinkets (5.350) are not
/// ported, so a trinket reaches a case only through `--trinket`.
pub fn place(state: &mut State, subtype: u32, slot: u32) -> Result<(), String> {
    let position = *slots(state)?
        .get(slot as usize)
        .ok_or("debug sandbox slot is outside the room")?;
    let id = state.next_pickup;
    let mut pickup = Pickup::spawn(id, 100, subtype, position, true)?;
    pickup.charge = crate::items::definition(subtype)
        .ok_or("collectible has no ItemConfig")?
        .max_charges;
    pickup.spawn_seed = state.boot_lifecycle.shared.next_u32();
    pickup.birth_game_update = state.game_update_count;
    state.next_pickup = id.checked_add(1).ok_or("pickup identity exhausted")?;
    state.room.pickups.push(pickup);
    let actor = ActorRef {
        kind: ActorKind::Pickup,
        id,
    };
    state.room.authored_actor_order.push(actor);
    state.actor_scheduler.admit(actor)
}

/// An enemy or boss in the room's upper right, through the ordinary families.
pub fn enemy(state: &mut State, entity_type: u16, variant: u16) -> Result<(), String> {
    let (width, _) = grid(state)?;
    let seed = state.boot_lifecycle.shared.next_u32();
    let selector = SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: SpawnCategory::Entity,
        entity_type,
        variant,
        subtype: 0,
        position: cell(width, width - 3, 2),
        group_seed: seed,
        spawn_seed: seed,
    };
    let id = state
        .room
        .npcs
        .iter()
        .map(|npc| npc.id.checked_add(1))
        .chain([state.actor_scheduler.next_free_id(ActorKind::Npc)])
        .try_fold(0_u32, |next, id| id.map(|id| next.max(id)))
        .ok_or("NPC identity exhausted")?;
    let context = crate::entities::SpawnContext {
        stage: state.stage,
        game_update_count: state.game_update_count,
    };
    let npc = crate::entities::spawn(id, &selector, &context)?
        .ok_or_else(|| format!("enemy {entity_type}.{variant} is not ported"))?;
    let (countable, boss) = (npc.can_shut_doors, npc.boss);
    if boss {
        state.room.boss_count = state.room.boss_count.wrapping_add(1);
    }
    state.room.npcs.push(npc);
    state.actor_scheduler.admit(ActorRef {
        kind: ActorKind::Npc,
        id,
    })?;
    state.room.manager_counts.admit(countable, boss);
    Ok(())
}
