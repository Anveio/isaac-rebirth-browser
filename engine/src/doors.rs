//! Authored door callbacks run in the full grid phase before player motion.
use crate::{combat, motion::WorldVector, phase::Phase, State};

/// Fresh ordinary Isaac's curse-door contact is Door::Update 565000. Room
/// types 7/8 suppress spikes; type10 on either remaining side charges one
/// half-heart within a strict 15-world-unit radius of the grid door position.
/// The native callback supplies damage flag 0x10000, an empty entity reference
/// and countdown argument30; its observed player immunity is60 presentations.
pub(crate) fn advance(state: &mut State, phase: Phase) -> Result<(), String> {
    if !phase.updates_nonplayers() {
        return Ok(());
    }
    // Fresh Door Init starts with class4. The first active full grid update
    // admits opened-door class5 after the inbound transition releases players.
    set_open_collision(state, 5);
    close_sounds(state);
    cracked_close_sounds(state);
    let current_type = state.floor.descriptors[state.current_room]
        .template
        .room_type;
    let position = WorldVector {
        x: ((state.player.x - 32.0) / 0.65) as f32,
        y: ((state.player.y + 46.0) / 0.65) as f32,
    };
    let curse_contact = state.room.appearance.doors.iter().any(|door| {
        let target_type = state.floor.descriptors[door.target_room as usize]
            .template
            .room_type;
        if matches!(current_type, 7 | 8)
            || matches!(target_type, 7 | 8)
            || (current_type != 10 && target_type != 10)
        {
            return false;
        }
        let dx = position.x - door.world_position.x;
        let dy = position.y - door.world_position.y;
        dx * dx + dy * dy < 225.0
    });
    if curse_contact {
        combat::try_damage_player(state, 1, 60, 0x10000)?;
    }
    Ok(())
}

/// Door::Update 565000 polls its Close animation: on frame2, two full updates
/// after the combat close, each closed door admits SFX35 (delay10).
fn close_sounds(state: &mut State) {
    if state
        .room
        .combat_shut_update
        .is_none_or(|update| update.wrapping_add(2) != state.game_update_count)
    {
        return;
    }
    for _ in 0..state.room.combat_shut_door_slots.len() {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter - 1,
            crate::audio::SoundRequest {
                id: 35,
                volume: 1.0,
                delay: 10,
                looping: false,
                pitch: 1.0,
            },
        );
    }
}

/// A cracked bedroom door's Init (563b70) plays the main sprite's Close clip
/// from frame0, so Door::Update 565000 reaches its frame2 SFX35 (delay10) on
/// the second full update of every room load, like a combat close.
fn cracked_close_sounds(state: &mut State) {
    let Some(frame) = state.room.cracked_close_frame else {
        return;
    };
    let frame = frame + 1;
    state.room.cracked_close_frame = (frame < 11).then_some(frame);
    if frame != 2 {
        return;
    }
    let cracked = state.room.grid.as_ref().map_or(0, |grid| {
        grid.cells
            .iter()
            .filter(|cell| {
                cell.entity
                    .as_ref()
                    .is_some_and(|entity| entity.kind == 16 && entity.variant == 3)
            })
            .count()
    });
    for _ in 0..cracked {
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter - 1,
            crate::audio::SoundRequest {
                id: 35,
                volume: 1.0,
                delay: 10,
                looping: false,
                pitch: 1.0,
            },
        );
    }
}

pub(crate) fn begin_room_admission(state: &mut State) {
    restore_secret_connections(state);
    set_open_collision(state, 4);
    let cracked = state.room.grid.as_ref().is_some_and(|grid| {
        grid.cells.iter().any(|cell| {
            cell.entity
                .as_ref()
                .is_some_and(|entity| entity.kind == 16 && entity.variant == 3)
        })
    });
    state.room.cracked_close_frame = cracked.then_some(0);
}

/// Room.check_player_enter_door5c71c0 runs after full actor callbacks. Fresh
/// keyboard movement selects its dominant input axis (vertical on ties) in
/// Player52981f; an ordinary key lock requires the matching outward direction,
/// a cleared room and distance strictly below60 from the displaced door point.
/// Golden BD0 and ordinary BCC use the same source TryUseKey; double-chain
/// locks and item overrides remain separate.
pub(crate) fn check_unlock(state: &mut State, dx: i32, dy: i32) {
    if state.room_transition.is_some() || !state.room.combat_clear {
        return;
    }
    let slot = if dx.abs() > dy.abs() {
        Some(if dx < 0 { 0 } else { 2 })
    } else if dy < 0 {
        Some(1)
    } else if dy > 0 {
        Some(3)
    } else {
        None
    };
    let Some(slot) = slot else { return };
    let position = WorldVector {
        x: ((state.player.x - 32.0) / 0.65) as f32,
        y: ((state.player.y + 46.0) / 0.65) as f32,
    };
    for door in &mut state.room.appearance.doors {
        if door.state != "KeyClosed" || door.slot % 4 != slot {
            continue;
        }
        let normal = match slot {
            0 => WorldVector { x: -1.0, y: 0.0 },
            1 => WorldVector { x: 0.0, y: -1.0 },
            2 => WorldVector { x: 1.0, y: 0.0 },
            _ => WorldVector { x: 0.0, y: 1.0 },
        };
        let delta_x = door.world_position.x + normal.x * 18.0 - position.x;
        let delta_y = door.world_position.y + normal.y * 18.0 - position.y;
        if (delta_x * delta_x + delta_y * delta_y).sqrt() >= 60.0
            || (state.player.golden_key != Some(true) && state.inventory.keys == 0)
        {
            continue;
        }
        let Some(grid) = &mut state.room.grid else {
            continue;
        };
        let index = door_index(door.world_position, grid.width);
        let Some(entity) = grid.cells[index].entity.as_mut() else {
            continue;
        };
        if entity.kind != 16 || entity.variant != 1 || entity.state != 1 {
            continue;
        }
        // Door.TryUnlock564D38 calls Player.TryUseKey5087F0, then selects
        // opened state2, variant7 and SFX156/delay2 only on success.
        if !state.player.try_use_key(&mut state.inventory) {
            continue;
        }
        door.state = "Opened".into();
        entity.variant = 7;
        entity.state = 2;
        entity.collision_class = 4;
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter - 1,
            crate::audio::SoundRequest {
                id: 156,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
}

fn connection(first: usize, second: usize) -> [usize; 2] {
    [first.min(second), first.max(second)]
}

fn door_index(position: WorldVector, width: u16) -> usize {
    ((position.x - 40.0) / 40.0) as usize
        + ((position.y - 120.0) / 40.0) as usize * usize::from(width)
}

/// Door.TryBlowOpen564870 uncovers the reciprocal saved variant before opening
/// the current hidden door. Fresh Basement's surprise-room flag stays clear;
/// ordinary NPC countability does not block this source secret-door policy.
/// Later surprise-room failure transitions and animated dust/rock effects
/// remain separate from this first-floor callback.
pub fn try_blow_open(state: &mut State, index: usize, _animated: bool) -> Result<bool, String> {
    let Some(grid) = &state.room.grid else {
        return Ok(false);
    };
    let Some(entity) = grid.cells.get(index).and_then(|cell| cell.entity.as_ref()) else {
        return Ok(false);
    };
    if entity.kind == 16 && entity.variant == 3 {
        return crack(state, index);
    }
    if entity.kind != 16 || entity.variant != 6 || entity.state != 1 {
        return Ok(false);
    }
    let Some(door) = state
        .room
        .appearance
        .doors
        .iter()
        .position(|door| door_index(door.world_position, grid.width) == index)
    else {
        return Ok(false);
    };
    let target = state.room.appearance.doors[door].target_room as usize;
    let current_type = state.floor.descriptors[state.current_room]
        .template
        .room_type;
    let target_type = state.floor.descriptors[target].template.room_type;
    if (!matches!(current_type, 7 | 8) && !matches!(target_type, 7 | 8))
        || current_type == 5
        || target_type == 5
        // Later stages can substitute a surprise miniboss; that lifecycle
        // remains outside this fresh Basement callback. Ordinary NPC counts
        // are not the generated flag and do not prevent secret revelation.
        || state.stage != 1
        || state.floor.descriptors[state.current_room].surprise_miniboss
    {
        return Ok(false);
    }
    state
        .revealed_secret_connections
        .insert(connection(state.current_room, target));
    let door = &mut state.room.appearance.doors[door];
    door.state = "Opened".into();
    door.blown_open = true;
    let entity = state.room.grid.as_mut().expect("admitted door grid").cells[index]
        .entity
        .as_mut()
        .expect("admitted hidden door");
    entity.variant = 7;
    entity.state = 2;
    entity.collision_class = 4;
    Ok(true)
}

/// TryBlowOpen564870 on a cracked bedroom door (variant3, 564b68): one
/// Random(5) and SpawnParticles 5515b0 of 6..10 Wood Particle27 at the door
/// with speed 8, then an intact (state1) door takes damage (state4, its
/// door_18 overlay plays Damaged) and the blast moves on. A damaged door's
/// second blast breaks it open; that branch is not admitted yet and only
/// records its frontier.
fn crack(state: &mut State, index: usize) -> Result<bool, String> {
    let door_state = state.room.grid.as_ref().and_then(|grid| grid.cells[index].entity.as_ref())
        .map_or(0, |entity| entity.state);
    if door_state != 1 {
        let primary = state.weapon_cache.primary.numeric();
        state.weapon_admission.unadmitted_controller.get_or_insert(
            crate::weapon_plan::UnadmittedController {
                primary,
                channel: "bombGridCrackedDoorBreak".into(),
            },
        );
        return Ok(false);
    }
    let width = state.room.grid.as_ref().map_or(0, |grid| grid.width);
    let Some(position) = state
        .room
        .appearance
        .doors
        .iter()
        .find(|door| door_index(door.world_position, width) == index)
        .map(|door| door.world_position)
    else {
        return Ok(false);
    };
    let count = state.boot_lifecycle.shared.bounded(5) + 6;
    crate::poop_effects::spawn_particles(state, 27, position, count, 8.0)?;
    let entity = state.room.grid.as_mut().expect("admitted door grid").cells[index]
        .entity
        .as_mut()
        .expect("admitted cracked door");
    entity.state = 4;
    Ok(false)
}

fn restore_secret_connections(state: &mut State) {
    let from_room = state
        .room_transition
        .as_ref()
        .map(|transition| transition.from_room);
    let Some(grid) = &mut state.room.grid else {
        return;
    };
    for door in &mut state.room.appearance.doors {
        if !state
            .revealed_secret_connections
            .contains(&connection(state.current_room, door.target_room as usize))
        {
            continue;
        }
        door.state = "Opened".into();
        // Door constructor56292f clears the transient+611 byte. Saved variant7
        // preserves discovery; Level.ChangeRoom58dba0 marks only the actual
        // inbound secret connection again through TryBlowOpen. Other revealed
        // doors follow ordinary admission collision4 until their next update.
        door.blown_open = Some(door.target_room as usize) == from_room;
        let index = door_index(door.world_position, grid.width);
        if let Some(entity) = &mut grid.cells[index].entity {
            entity.variant = 7;
            entity.state = 2;
            entity.collision_class = if door.blown_open { 5 } else { 4 };
        }
    }
}

fn set_open_collision(state: &mut State, collision_class: u32) {
    let blown = state
        .room
        .appearance
        .doors
        .iter()
        .filter(|door| door.blown_open)
        .map(|door| door.world_position)
        .collect::<Vec<_>>();
    if let Some(grid) = &mut state.room.grid {
        let width = grid.width;
        for (index, cell) in grid.cells.iter_mut().enumerate() {
            if let Some(entity) = &mut cell.entity {
                if entity.kind == 16 && entity.state == 2 {
                    entity.collision_class = if blown
                        .iter()
                        .any(|position| door_index(*position, width) == index)
                    {
                        5
                    } else {
                        collision_class
                    };
                }
            }
        }
    }
}
