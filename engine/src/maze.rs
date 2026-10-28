//! Curse of the Maze (Level+8 bit 0x20) in Level::ChangeRoom 58d900.
//!
//! After Room.Save of the room being left, 58d95d tests the curse on every
//! change between two valid grid cells. RandomInt(10) on the shared MT picks
//! the teleport (58dc90) on a zero; RandomInt(5) then picks the room swap
//! (58dbf0) on a zero. The teleport shuffles the rooms within two doors of the
//! target with the shared MT and enters the first that has a door on the
//! entry side; the swap shuffles descriptors with the process-global glibc
//! rand and exchanges two visited rooms of the same type, shape and doors.
//!
//! Rooms are descriptor indices here; the original's Level+8188/818C and
//! Descriptor+18 neighbors are grid cells, which `cell` and `neighbor` map.
//! Descriptor+3C is the visited flag Room.Init sets (5cf624), which
//! `State::visited_rooms` holds.
use crate::{
    audio::SoundRequest, floor::Descriptor, mt::DescriptorSeeds, traversal::RoomTransition, State,
};
use std::collections::BTreeSet;

pub const MAZE: u32 = 0x20;
const COLUMNS: usize = 13;
/// 5dd120 starts the minimap shake (5aeb00: MinimapManager+209D0 = 30).
pub const SHAKE_UPDATES: i32 = 30;
/// 5dd120 plays SFX 22 at volume 0.5, delay 2.
const TELEPORT_SOUND: u32 = 22;

fn room_at(state: &State, cell: usize) -> Option<usize> {
    state
        .floor
        .grid
        .get(cell)
        .and_then(|index| usize::try_from(*index).ok())
}

/// Descriptor+18[slot]: the grid cell beyond a door slot, if it is on the grid.
fn neighbor(descriptor: &Descriptor, slot: u8) -> Option<usize> {
    let (x, y) = crate::traversal::slot_cell(descriptor, slot);
    ((0..COLUMNS as i32).contains(&x) && (0..COLUMNS as i32).contains(&y))
        .then_some(y as usize * COLUMNS + x as usize)
}

fn has_door(descriptor: &Descriptor, slot: u8) -> bool {
    descriptor.door_mask & (1 << slot) != 0
}

fn room_type(state: &State, index: usize) -> u32 {
    state.floor.descriptors[index].template.room_type
}

/// 587070: the slot of the door a room is entered through at `cell` from
/// direction `dir`, falling back through the authored door mask.
fn entry_slot(descriptor: &Descriptor, cell: usize, dir: u8) -> u8 {
    let offset = cell as i32 - descriptor.grid_index as i32;
    let second = match dir {
        1 | 3 => offset % COLUMNS as i32 > 0,
        _ => offset > 12,
    };
    let mut slot = if second { dir + 4 } else { dir };
    let allowed = descriptor.template.door_mask;
    if allowed & (1 << slot) == 0 {
        slot = dir + 4;
        for _ in 0..8 {
            if allowed & (1 << slot) != 0 {
                break;
            }
            slot = (slot + 1) & 7;
        }
    }
    slot
}

/// 589660: grid cells through the target's doors and through theirs (the
/// target cell excepted), each a normal room (type 1) or a visited one.
fn candidates(state: &State, target_cell: usize) -> BTreeSet<usize> {
    let eligible =
        |index: usize| room_type(state, index) == 1 || state.visited_rooms.contains(&index);
    let mut cells = BTreeSet::new();
    let Some(target) = room_at(state, target_cell) else {
        return cells;
    };
    let target = &state.floor.descriptors[target];
    for slot in 0..8 {
        let Some(first) = neighbor(target, slot).filter(|_| has_door(target, slot)) else {
            continue;
        };
        let Some(room) = room_at(state, first).filter(|room| eligible(*room)) else {
            continue;
        };
        cells.insert(first);
        let room = &state.floor.descriptors[room];
        for slot in 0..8 {
            let Some(second) = neighbor(room, slot).filter(|_| has_door(room, slot)) else {
                continue;
            };
            if second != target_cell && room_at(state, second).is_some_and(eligible) {
                cells.insert(second);
            }
        }
    }
    cells
}

struct Teleport {
    cell: usize,
    slot: u8,
    previous: usize,
}

/// 58dc90: the shuffled candidates, the first with a door on the entry side
/// whose far room is not a secret room.
fn teleport(state: &mut State, exit_slot: u8, target_cell: usize) -> Option<Teleport> {
    let mut cells: Vec<_> = candidates(state, target_cell).into_iter().collect();
    for index in (1..cells.len()).rev() {
        let pick = state.boot_lifecycle.shared.bounded(index as u32 + 1) as usize;
        cells.swap(index, pick);
    }
    let dir = (exit_slot + 2) & 3;
    cells.into_iter().find_map(|cell| {
        let room = room_at(state, cell)?;
        if matches!(room_type(state, room), 7 | 8 | 10) {
            return None;
        }
        let descriptor = &state.floor.descriptors[room];
        let slot = entry_slot(descriptor, cell, dir);
        if slot & 3 != dir || !has_door(descriptor, slot) {
            return None;
        }
        let previous = neighbor(descriptor, slot)?;
        let beyond = room_at(state, previous)?;
        (!matches!(room_type(state, beyond), 7 | 8)).then_some(Teleport {
            cell,
            slot,
            previous,
        })
    })
}

/// 58d160 with its visited filter: a shuffled other visited room with the same
/// door mask, shape and type, or None.
fn partner(state: &mut State, room: usize) -> Option<usize> {
    let mut order: Vec<_> = (0..state.floor.descriptors.len()).collect();
    state.libc_rand.shuffle(&mut order);
    if !state.visited_rooms.contains(&room) {
        return None;
    }
    let descriptor = &state.floor.descriptors[room];
    order.into_iter().find(|other| {
        let candidate = &state.floor.descriptors[*other];
        *other != room
            && state.visited_rooms.contains(other)
            && candidate.door_mask == descriptor.door_mask
            && candidate.template.shape_index() == descriptor.template.shape_index()
            && candidate.template.room_type == descriptor.template.room_type
    })
}

fn door_cells(room: &crate::Room) -> Result<BTreeSet<usize>, String> {
    let grid = room
        .grid
        .as_ref()
        .ok_or("maze room swap requires saved grids")?;
    Ok(grid
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.entity.as_ref().is_some_and(|entity| entity.kind == 16))
        .map(|(index, _)| index)
        .collect())
}

/// 588490: exchange two rooms' contents. Each descriptor keeps its grid cell,
/// index and neighbors, and each saved grid keeps its door records (588bac),
/// so the doors stay where they are while everything else moves.
fn exchange(
    state: &mut State,
    seeds: &mut DescriptorSeeds,
    a: usize,
    b: usize,
) -> Result<(), String> {
    let (first, second) = (&state.floor.descriptors[a], &state.floor.descriptors[b]);
    if (first.template.width, first.template.height)
        != (second.template.width, second.template.height)
    {
        return Ok(());
    }
    let mut moved = [second.clone(), first.clone()];
    for (descriptor, place) in moved.iter_mut().zip([first, second]) {
        descriptor.index = place.index;
        descriptor.generated_index = place.generated_index;
        descriptor.grid_index = place.grid_index;
        descriptor.neighbors = place.neighbors;
    }
    let [into_a, into_b] = moved;
    let mut room_a = state
        .persisted_rooms
        .remove(&b)
        .ok_or("maze room swap requires both saved rooms")?;
    let mut room_b = state
        .persisted_rooms
        .remove(&a)
        .ok_or("maze room swap requires both saved rooms")?;
    let mut doors = door_cells(&room_a)?;
    doors.extend(door_cells(&room_b)?);
    std::mem::swap(&mut room_a.appearance.doors, &mut room_b.appearance.doors);
    std::mem::swap(
        &mut room_a.combat_shut_door_slots,
        &mut room_b.combat_shut_door_slots,
    );
    if let (Some(grid_a), Some(grid_b)) = (room_a.grid.as_mut(), room_b.grid.as_mut()) {
        for &cell in &doors {
            if let (Some(left), Some(right)) =
                (grid_a.cells.get_mut(cell), grid_b.cells.get_mut(cell))
            {
                std::mem::swap(&mut left.entity, &mut right.entity);
            }
        }
    }
    seeds.swap_rooms(a, b, &doors)?;
    room_a.descriptor = room_a.descriptor.is_some().then(|| into_a.clone());
    room_b.descriptor = room_b.descriptor.is_some().then(|| into_b.clone());
    state.persisted_rooms.insert(a, room_a);
    state.persisted_rooms.insert(b, room_b);
    state.floor.descriptors[a] = into_a;
    state.floor.descriptors[b] = into_b;
    state.room_display_flags.swap(a, b);
    Ok(())
}

/// 58dbf0: the first shuffled visited room with a partner swaps with it.
fn swap(state: &mut State, seeds: &mut DescriptorSeeds) -> Result<(), String> {
    let mut order: Vec<_> = (0..state.floor.descriptors.len()).collect();
    state.libc_rand.shuffle(&mut order);
    for room in order {
        if !state.visited_rooms.contains(&room) {
            continue;
        }
        if let Some(other) = partner(state, room) {
            return exchange(state, seeds, room, other);
        }
    }
    Ok(())
}

/// 58d95d..58da35 for a door transition, after the left room's Room.Save.
/// The target room is not yet visited, so its flag reads as before Room.Init.
pub(crate) fn change_room(
    state: &mut State,
    seeds: &mut DescriptorSeeds,
    transition: &mut RoomTransition,
) -> Result<(), String> {
    if state.floor.curses & MAZE == 0 {
        return Ok(());
    }
    let left = &state.floor.descriptors[transition.from_room];
    let target_cell =
        neighbor(left, transition.from_slot).ok_or("maze door target is off the grid")?;
    if state.boot_lifecycle.shared.bounded(10) == 0 {
        if let Some(teleport) = teleport(state, transition.from_slot, target_cell) {
            // 58d99b re-enters from the far side of the chosen door.
            let beyond = room_at(state, teleport.previous)
                .ok_or("maze teleport door leads off the floor")?;
            let beyond = &state.floor.descriptors[beyond];
            let entered = if has_door(beyond, transition.from_slot) {
                neighbor(beyond, transition.from_slot).unwrap_or(teleport.cell)
            } else {
                teleport.cell
            };
            let room = room_at(state, teleport.cell).ok_or("maze teleport cell has no room")?;
            if room_at(state, entered) != Some(room) {
                return Err(
                    "maze teleport re-entry source frontier: 58d99b enters another room".into(),
                );
            }
            transition.target_room = room;
            transition.target_slot = teleport.slot;
            // 5dd120: a style-0 door transition takes style 4, which follows
            // style 0's legs, with the minimap shake and SFX 22.
            state.minimap_presentation.shake = SHAKE_UPDATES;
            state.audio.admit(
                &mut state.boot_lifecycle.shared,
                state.system_clock.system_counter.wrapping_sub(1),
                SoundRequest {
                    id: TELEPORT_SOUND,
                    volume: 0.5,
                    delay: 2,
                    looping: false,
                    pitch: 1.0,
                },
            );
        }
    }
    if state.boot_lifecycle.shared.bounded(5) == 0 {
        swap(state, seeds)?;
    }
    Ok(())
}
