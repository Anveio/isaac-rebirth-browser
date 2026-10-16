//! Room light pass of Room render 5d1a70, once per presented frame.
//!
//! The pass walks the entity manager's list (Room+0x1638, count +0x1644) in
//! order and dispatches each entity's light by type. A fire (type 33) whose
//! HP exceeds 1 flickers its light by `uniform() * 3` (5d31b5) along a
//! `direction()` (5d31cb) of the shared stream, every presentation including
//! room-transition frames. Of the other drawing lights (type 54, 263, Bomb
//! variant 8, 273 variant 10, effect 1000.51 and a gated type 85), none is
//! admitted by this runtime, so only fires consume the stream here. The
//! browser does not draw the light offsets yet; only the draws are kept.
//! The frame that changes rooms renders the room being left once before
//! ChangeRoom (the transition's capture of the outgoing room) and the entered
//! room twice after it.
use crate::{mt::Mt19937, Room, State};

pub(crate) fn present(state: &mut State) {
    present_room(&state.room, &mut state.boot_lifecycle.shared);
}

/// One pass over `room`'s fires in manager (admission) order.
pub(crate) fn present_room(room: &Room, shared: &mut Mt19937) {
    for _ in room
        .npcs
        .iter()
        .filter(|npc| npc.kind == 33 && npc.hp > 1.0)
    {
        let _offset = shared.uniform() * 3.0;
        let _direction = crate::npcs::direction(shared);
    }
}
