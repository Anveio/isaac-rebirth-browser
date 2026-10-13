//! Ordinary active spike damage runs in the full grid pass before actors.
//! Source 0x561100 compares strict center distance, then successful player damage
//! admits a four-unit radial impulse. Retracting and sacrifice rules are separate.
use crate::{bombs, motion::WorldVector, phase::Phase, State};

pub fn advance(state: &mut State, phase: Phase) -> Result<(), String> {
    if phase != Phase::Update || state.player.hp == 0 || state.damage_cooldown != 0 {
        return Ok(());
    }
    let Some(grid) = state.room.grid.as_ref() else {
        return Ok(());
    };
    for (index, cell) in grid.cells.iter().enumerate() {
        if !cell
            .entity
            .as_ref()
            .is_some_and(|entity| entity.kind == 8 && entity.state == 0)
        {
            continue;
        }
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
            return Err("spike contact requires bounded native world positions".into());
        }
        // This grid helper does not add the entity's collision radius.
        if squared >= 21.0 * 21.0 {
            continue;
        }
        let normal = if squared == 0.0 {
            WorldVector { x: 0.0, y: 0.0 }
        } else {
            delta * (1.0 / squared.sqrt())
        };
        let velocity = bombs::player_velocity(state) + normal * 4.0;
        if !velocity.is_finite() {
            return Err("spike impulse overflowed native velocity".into());
        }
        state.player.hp = state.player.hp.saturating_sub(2);
        state.damage_cooldown = 120;
        state.player.vx = f64::from(velocity.x) * 0.65;
        state.player.vy = f64::from(velocity.y) * 0.65;
        break;
    }
    Ok(())
}
