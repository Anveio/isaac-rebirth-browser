//! Ordinary 1x1 doorway admission and descriptor-owned room persistence.
//! The public floor graph and authored doors are runtime inputs; source maps
//! and captured room/actor geometry never enter the production route.
use crate::{motion::WorldVector, phase::Phase, Player, Room, State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TransitionPhase {
    Out,
    In,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomTransition {
    pub from_room: usize,
    pub target_room: usize,
    pub from_slot: u8,
    pub target_slot: u8,
    pub phase: TransitionPhase,
    pub full_updates: u8,
}

fn outward(slot: u8) -> WorldVector {
    match slot % 4 {
        0 => WorldVector { x: -1.0, y: 0.0 },
        1 => WorldVector { x: 0.0, y: -1.0 },
        2 => WorldVector { x: 1.0, y: 0.0 },
        _ => WorldVector { x: 0.0, y: 1.0 },
    }
}

fn world(player: &Player) -> WorldVector {
    WorldVector {
        x: ((player.x - 32.0) / 0.65) as f32,
        y: ((player.y + 46.0) / 0.65) as f32,
    }
}

/// Boundary collision geometry includes each authored opened doorway. The
/// grid factory replaces this geometry with fully seeded authored objects.
pub(crate) fn doorway_grid(room: &Room) -> crate::grid::Grid {
    let mut descriptor = crate::grid::Grid::closed_1x1_boundary()
        .descriptor()
        .clone();
    for door in &room.appearance.doors {
        if door.state != "Opened" {
            continue;
        }
        let column = ((door.world_position.x - 40.0) / 40.0) as usize;
        let row = ((door.world_position.y - 120.0) / 40.0) as usize;
        let index = column + row * usize::from(descriptor.width);
        if let Some(cell) = descriptor.cells.get_mut(index) {
            cell.entity = Some(crate::grid::GridEntity {
                kind: 16,
                variant: 0,
                state: 2,
                collision_class: 5,
                spawn_seed: 0,
            });
        }
    }
    crate::grid::Grid::new(descriptor).expect("authored ordinary door boundary geometry")
}

/// Room::check_player_enter_door 5c71c0 uses Euclidean distance below 25
/// from the authored grid position displaced outward by 18 world units.
/// This check runs after entities in a full Room update.
pub(crate) fn check_departure(state: &mut State) {
    if state.room_transition.is_some() {
        return;
    }
    let position = world(&state.player);
    for door in &state.room.appearance.doors {
        if door.state != "Opened" {
            continue;
        }
        let target_room = door.target_room as usize;
        let destination = &state.floor.descriptors[target_room];
        // Large-room camera/backdrop and combat locks need their owning source
        // routes. Preserve those descriptors without claiming this small route.
        if !matches!(destination.template.room_type, 1 | 4 | 10)
            || (destination.template.width, destination.template.height) != (13, 7)
        {
            continue;
        }
        let normal = outward(door.slot);
        let delta_x = (door.world_position.x + normal.x * 18.0) - position.x;
        let delta_y = (door.world_position.y + normal.y * 18.0) - position.y;
        if (delta_x * delta_x + delta_y * delta_y).sqrt() >= 25.0 {
            continue;
        }
        let opposite = (door.slot + 2) % 4;
        let Some(target_slot) =
            destination
                .neighbors
                .iter()
                .enumerate()
                .find_map(|(slot, value)| {
                    (*value == Some(state.current_room) && slot % 4 == usize::from(opposite))
                        .then_some(slot as u8)
                })
        else {
            continue;
        };
        state.room_transition = Some(RoomTransition {
            from_room: state.current_room,
            target_room,
            from_slot: door.slot,
            target_slot,
            phase: TransitionPhase::Out,
            // Game::Update chooses its transition branch before Room::Update.
            full_updates: 0,
        });
        state.player.vx = 0.0;
        state.player.vy = 0.0;
        break;
    }
}

/// The next full Game update admits a normal style0 door (5ddf20/5de26f).
/// The inbound state3 pre-counter then uses limit7 (5dde10).
/// Native room admission clears velocity and puts the player one 40-unit cell
/// inside the incoming authored doorway. Mutable Room state is restored once.
pub(crate) fn advance_transition(state: &mut State, phase: Phase) -> bool {
    let Some(mut transition) = state.room_transition.take() else {
        return false;
    };
    if !phase.updates_nonplayers() {
        state.room_transition = Some(transition);
        return true;
    }
    if transition.phase == TransitionPhase::In && transition.full_updates < 7 {
        transition.full_updates += 1;
        state.room_transition = Some(transition);
        return true;
    }
    if transition.phase == TransitionPhase::In {
        // The state3 completion branch dispatches one Player::Update at
        // 5dde9e while the room's grid/nonplayer callbacks remain frozen.
        crate::combat::advance_player_immunity(state);
        return true;
    }
    state
        .persisted_rooms
        .insert(state.current_room, state.room.clone());
    state.current_room = transition.target_room;
    state.room = state
        .persisted_rooms
        .remove(&state.current_room)
        .unwrap_or_else(|| Room::from_floor_descriptor(&state.floor, state.current_room));
    state.visited_rooms.insert(state.current_room);
    let door = state
        .room
        .appearance
        .doors
        .iter()
        .find(|door| door.slot == transition.target_slot)
        .expect("reciprocal generated ordinary doorway is authored");
    let normal = outward(transition.target_slot);
    state.player.x = f64::from(door.world_position.x - normal.x * 40.0) * 0.65 + 32.0;
    state.player.y = f64::from(door.world_position.y - normal.y * 40.0) * 0.65 - 46.0;
    state.player.vx = 0.0;
    state.player.vy = 0.0;
    state.tears.clear();
    state.impacts.clear();
    state.bombs.clear();
    state.explosions.clear();
    state
        .actor_scheduler
        .retain(|actor| actor.kind == crate::actors::ActorKind::Player);
    transition.phase = TransitionPhase::In;
    transition.full_updates = 0;
    state.room_transition = Some(transition);
    true
}
