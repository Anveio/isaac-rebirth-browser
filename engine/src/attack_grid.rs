//! Actual Knife473323 -> Room5cbb30 -> grid virtual28 callbacks.
//! These receivers are distinct from collision queries and path-cost stores.
use crate::{grid::Grid, motion::WorldVector, mt::Mt19937, Room};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GridCallbackOutcome {
    Handled,
    UnadmittedKind(u32),
}

pub(crate) fn knife_callback(
    room: &mut Room,
    geometry: &Grid,
    position: WorldVector,
    shared: &mut Mt19937,
) -> Result<GridCallbackOutcome, String> {
    let Some(index) = geometry.index(position) else {
        return Ok(GridCallbackOutcome::Handled);
    };
    callback_at_index(room, index, shared)
}

///Actual Room5CBB30 invokes the same grid virtual28 receiver at an index.
pub(crate) fn callback_at_index(
    room: &mut Room,
    index: usize,
    shared: &mut Mt19937,
) -> Result<GridCallbackOutcome, String> {
    let entity = room
        .grid
        .as_mut()
        .ok_or("knife grid callback requires actual room admission")?
        .cells
        .get_mut(index)
        .ok_or("knife grid callback exceeds actual room cells")?
        .entity
        .as_mut();
    let Some(entity) = entity else {
        return Ok(GridCallbackOutcome::Handled);
    };
    match entity.kind {
        //Actual factory vtables select the literal561bf0 false-return stub.
        1..=11 | 15..=20 => Ok(GridCallbackOutcome::Handled),
        //5703b0 always takes Bounded3 before testing the existing state.
        12 => {
            let strength = if shared.bounded(3) == 0 { 2 } else { 1 };
            if (entity.state as i32) <= 3 {
                entity.state = (entity.state as i32).wrapping_add(strength).min(4) as u32;
            }
            Ok(GridCallbackOutcome::Handled)
        }
        //5660e0/56a390 have real destruction/seed/effect/drop callbacks.
        //Source WIP retains this explicit frontier until those receivers land.
        //Return the typed implementation frontier before any missing
        //receiver draw/store. The caller records an observable diagnostic
        //and continues the remaining admitted actor body/tail.
        13 | 14 => Ok(GridCallbackOutcome::UnadmittedKind(entity.kind)),
        _ => Err(format!(
            "knife grid receiver has unrecognized actual kind{}",
            entity.kind
        )),
    }
}
