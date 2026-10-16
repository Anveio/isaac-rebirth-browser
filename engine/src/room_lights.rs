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
use crate::{actors::ActorKind, State};

pub(crate) fn present(state: &mut State) {
    let fires: Vec<u32> = state
        .actor_scheduler
        .full_phase()
        .into_iter()
        .filter(|actor| actor.kind == ActorKind::Npc)
        .filter_map(|actor| {
            state
                .room
                .npcs
                .iter()
                .find(|npc| npc.id == actor.id && npc.kind == 33 && npc.hp > 1.0)
                .map(|npc| npc.id)
        })
        .collect();
    let shared = &mut state.boot_lifecycle.shared;
    for _ in fires {
        let _offset = shared.uniform() * 3.0;
        let _direction = crate::npcs::direction(shared);
    }
}
