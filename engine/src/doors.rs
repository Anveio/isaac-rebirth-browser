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
    set_open_collision(state, 4);
}

fn set_open_collision(state: &mut State, collision_class: u32) {
    if let Some(grid) = &mut state.room.grid {
        for cell in &mut grid.cells {
            if let Some(entity) = &mut cell.entity {
                if entity.kind == 16 && entity.state == 2 {
                    entity.collision_class = collision_class;
                }
            }
        }
    }
}
