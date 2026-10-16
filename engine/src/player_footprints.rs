//! Player50c000: a walking player leaves a small Blood7 footprint, tinted
//! from the floor beneath it, on every sixth full game update.
use crate::{bombs, familiars, motion::WorldVector, State};

/// 50c04a sets Color(0,0,0,255) then alpha .7 (Player+30b0); 50c27a lerps
/// that alpha toward .7, so it never changes. Both ColorMods' offsets are the
/// sampled floor color *.9 (42f500). The runtime has no backdrop pixels, so
/// those offsets stay zero and the stain is flagged as a footprint.
const TINT: [f32; 4] = [0.0, 0.0, 0.0, 0.7];

/// Player531850 calls 50c000 at 531c8f, after its shot callbacks. Only the
/// full update spawns: the intermediate pass sharing that counter does not.
/// `velocity` is the player's velocity after input and before this update's
/// grid collision, which 531c8f precedes.
pub(crate) fn advance(state: &mut State, velocity: WorldVector) -> Result<(), String> {
    // 50bef0(-1): IsFrame444890(6, 0) on Game+ce9f4 (game_update_count) and
    // a clear Player+d68 byte that no supported player sets.
    let counter = state.game_update_count;
    if !counter.is_multiple_of(6) {
        return Ok(());
    }
    // 50c1a0: the velocity's perpendicular, normalized and doubled. 50bef0(0)
    // is IsFrame(12), which places it on the other side.
    let side = crate::npcs::normalized(WorldVector {
        x: velocity.y,
        y: -velocity.x,
    }) * 2.0;
    let side = if counter.is_multiple_of(12) {
        side * -1.0
    } else {
        side
    };
    // 612950 is the squared length.
    if velocity.x * velocity.x + velocity.y * velocity.y <= 0.16 {
        return Ok(());
    }
    // 50c3d8: shared seed, Game::Spawn1000/7 with no spawner, Sprite scale
    // .1, SetColor (virtual58), then the immediate Update (virtual18).
    let seed = state.boot_lifecycle.shared.next_u32();
    let position = bombs::player_world(state) + side;
    let id = familiars::admit_blood_splat(state, seed, position, TINT, [0.0; 3])?;
    familiars::mark_footprint(state, id)?;
    familiars::set_blood_scale(state, id, 0.1)?;
    familiars::advance_blood_initial(state, id)
}
