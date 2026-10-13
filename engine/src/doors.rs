//! Authored door callbacks run in the full grid phase before player motion.
use crate::{combat, motion::WorldVector, phase::Phase, State};

/// Fresh ordinary Isaac's curse-door contact is Door::Update 565000. Room
/// types 7/8 suppress spikes; type10 on either remaining side charges one
/// half-heart within a strict 15-world-unit radius of the grid door position.
/// The native callback supplies damage flag 0x10000, an empty entity reference
/// and countdown argument30; its observed player immunity is60 presentations.
pub(crate) fn advance(state: &mut State, phase: Phase) {
    if !phase.updates_nonplayers() {
        return;
    }
    // Fresh Door Init starts with class4. The first active full grid update
    // admits opened-door class5 after the inbound transition releases players.
    set_open_collision(state, 5);
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
        combat::try_damage_player(state, 1, 60);
    }
}

pub(crate) fn begin_room_admission(state: &mut State) {
    restore_secret_connections(state);
    set_open_collision(state, 4);
}

fn connection(first: usize, second: usize) -> [usize; 2] {
    [first.min(second), first.max(second)]
}

fn door_index(position: WorldVector, width: u16) -> usize {
    ((position.x - 40.0) / 40.0) as usize
        + ((position.y - 120.0) / 40.0) as usize * usize::from(width)
}

/// Door.TryBlowOpen564870 uncovers the reciprocal saved variant before opening
/// the current hidden door. This slice admits fresh Basement secret endpoints
/// in clear rooms; hostile secret closure and its failed-open saved variant
/// remain a separate lifecycle. Animated dust/rock effects are not admitted.
pub fn try_blow_open(state: &mut State, index: usize, _animated: bool) -> bool {
    let Some(grid) = &state.room.grid else {
        return false;
    };
    let Some(entity) = grid.cells.get(index).and_then(|cell| cell.entity.as_ref()) else {
        return false;
    };
    if entity.kind != 16 || entity.variant != 6 || entity.state != 1 {
        return false;
    }
    let Some(door) = state
        .room
        .appearance
        .doors
        .iter()
        .position(|door| door_index(door.world_position, grid.width) == index)
    else {
        return false;
    };
    let target = state.room.appearance.doors[door].target_room as usize;
    let current_type = state.floor.descriptors[state.current_room]
        .template
        .room_type;
    let target_type = state.floor.descriptors[target].template.room_type;
    if (!matches!(current_type, 7 | 8) && !matches!(target_type, 7 | 8))
        || current_type == 5
        || target_type == 5
        // This initial route supports cleared rooms. Source's broader policy
        // also reads descriptor byte+40; its producer is still being recovered.
        || !state.room.combat_clear
    {
        return false;
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
    true
}

fn restore_secret_connections(state: &mut State) {
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
        door.blown_open = true;
        let index = door_index(door.world_position, grid.width);
        if let Some(entity) = &mut grid.cells[index].entity {
            entity.variant = 7;
            entity.state = 2;
            entity.collision_class = 5;
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
