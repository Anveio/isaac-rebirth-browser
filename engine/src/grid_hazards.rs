//! Grid contact damage (561100) in the full grid pass before actors: active
//! spikes here, red poop from `grid_records::advance_poops`. The helper
//! compares strict centre distance, then a successful Player hit admits a
//! four-unit radial impulse. Retracting and sacrifice rules are separate.
use crate::{bombs, motion::WorldVector, phase::Phase, State};

pub fn advance(state: &mut State, phase: Phase) -> Result<(), String> {
    if phase != Phase::Update {
        return Ok(());
    }
    let Some(grid) = state.room.grid.as_ref() else {
        return Ok(());
    };
    let spikes: Vec<usize> = (grid.cells.iter().enumerate())
        .filter(|(_, cell)| {
            cell.entity
                .as_ref()
                .is_some_and(|entity| entity.kind == 8 && entity.state == 0)
        })
        .map(|(index, _)| index)
        .collect();
    for index in spikes {
        // 56fcc4: radius 21, two half hearts, DAMAGE_SPIKES.
        if contact(
            state,
            index,
            21.0,
            2,
            120,
            crate::player_death::DAMAGE_SPIKES,
        )? {
            break;
        }
    }
    Ok(())
}

/// 561100's Player branch for the grid entity at `index`: a Player strictly
/// within `radius` of the cell centre (no collision radius added) takes
/// `damage` half hearts with `flags`, and a successful hit adds a four-unit
/// impulse away from the centre. Returns whether the hit was taken.
/// PORT GAP: the helper's NPC branch (its f32 damage to every other entity in
/// range) is not run.
pub(crate) fn contact(
    state: &mut State,
    index: usize,
    radius: f32,
    damage: u8,
    immunity: u32,
    flags: u32,
) -> Result<bool, String> {
    if state.player.dead || state.damage_cooldown != 0 {
        return Ok(false);
    }
    let Some(grid) = state.room.grid.as_ref() else {
        return Ok(false);
    };
    let center = WorldVector {
        x: 40.0 + (index % usize::from(grid.width)) as f32 * 40.0,
        y: 120.0 + (index / usize::from(grid.width)) as f32 * 40.0,
    };
    let position = bombs::player_world(state);
    let delta = WorldVector {
        x: position.x - center.x,
        y: position.y - center.y,
    };
    let squared = delta.x * delta.x + delta.y * delta.y;
    if !squared.is_finite() {
        return Err("grid contact requires bounded native world positions".into());
    }
    if squared >= radius * radius {
        return Ok(false);
    }
    let normal = if squared == 0.0 {
        WorldVector { x: 0.0, y: 0.0 }
    } else {
        delta * (1.0 / squared.sqrt())
    };
    let velocity = bombs::player_velocity(state) + normal * 4.0;
    if !velocity.is_finite() {
        return Err("grid contact impulse overflowed native velocity".into());
    }
    if !crate::combat::try_damage_player(state, damage, immunity, flags)? {
        return Ok(false);
    }
    state.player.vx = f64::from(velocity.x) * 0.65;
    state.player.vy = f64::from(velocity.y) * 0.65;
    Ok(true)
}
