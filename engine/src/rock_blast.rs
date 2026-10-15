//! Ordinary Basement Rock destruction reached through a real bomb blast.
//!
//! The callback changes collision, the room's destruction marker and linked
//! authored frames. Modifier drops, tinted rocks and emitted effect actors have
//! separate source callbacks; no guessed loot or placeholder random draws run.
use crate::{grid::Grid, motion::WorldVector, State};

/// Room.Update ages destruction markers before actor dispatch every third full
/// game update. Paused gameplay and doorway freezes do not advance this clock.
/// The gate reads native Game+ce9f4, which is one more than game_update_count.
pub fn advance_markers(state: &mut State) {
    if !(state.game_update_count + 1).is_multiple_of(3) {
        return;
    }
    if let Some(costs) = &mut state.room.path_costs {
        let count = state.room.grid.as_ref().map_or(0, |grid| grid.cells.len());
        for cost in &mut costs.entries_mut()[..count] {
            if (1..=900).contains(cost) {
                *cost = (*cost - 100).max(0);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BombRockFuse {
    pub index: u32,
    pub remaining: u8,
}

/// Rock.Update56bf00 runs before Room.Update ages markers. State3 holds
/// collision class3 and post-decrements Rock+348; when it reads zero, the
/// fourth update, Destroy56c670 sets state2, collision0 and marker900, then
/// explodes through Game554490 with damage40, radius60 and flags4 and no
/// source entity. Its 1000/1 explosion and particle draws stay unadmitted
/// effect families, as for ordinary rock debris.
pub fn advance_bomb_rocks(state: &mut State) -> Result<(), String> {
    // Grid updates run in ascending cell order; rocks that a secondary blast
    // moves to state3 start counting on the next update.
    let mut due = Vec::new();
    state.room.bomb_rock_fuses.retain_mut(|fuse| {
        if fuse.remaining == 0 {
            due.push(fuse.index as usize);
            return false;
        }
        fuse.remaining -= 1;
        true
    });
    due.sort_unstable();
    for cell in due {
        let grid = state.room.grid.as_mut().ok_or("bomb rock requires a room grid")?;
        let rock = grid.cells[cell]
            .entity
            .as_mut()
            .filter(|rock| rock.kind == 5 && rock.state == 3)
            .ok_or("bomb rock fuse lost its state3 rock")?;
        rock.state = 2;
        rock.collision_class = 0;
        let position = Grid::new(grid.clone())?
            .position(cell)
            .ok_or("bomb rock outside its grid")?;
        state
            .room
            .path_costs
            .as_mut()
            .ok_or("Bomb rock destruction requires source Room.Init path costs")?
            .store(cell, 900)?;
        crate::bombs::explode(
            state,
            position,
            60.0,
            40.0,
            crate::combat::DamageSource {
                id: 0,
                entity_type: 0,
            },
        )?;
    }
    Ok(())
}

/// Source 555300 selects a75-unit regular blast tier before554490 collects
/// ascending grid indices. Each Rock.Destroy56c670 may repair surviving links
/// before the next selected object's callback observes its current variant.
pub fn destroy(state: &mut State, origin: WorldVector, radius: f32) -> Result<(), String> {
    let Some(descriptor) = &state.room.grid else {
        return Ok(());
    };
    let selected = Grid::new(descriptor.clone())?.blast_candidates(origin, radius)?;
    let decoration_seed = state
        .room
        .descriptor
        .as_ref()
        .map_or(0, |descriptor| descriptor.decoration_seed);
    for index in selected {
        if crate::doors::try_blow_open(state, index, true) {
            continue;
        }
        let grid = state.room.grid.as_mut().expect("admitted room grid");
        let Some(rock) = grid.cells[index].entity.as_mut() else {
            continue;
        };
        // Destroy56c670: a blast (immediate0) only moves an intact type5 bomb
        // rock to state3. Init stored its Rock+348 countdown as 3.
        if rock.kind == 5 && rock.state == 1 {
            rock.state = 3;
            state.room.bomb_rock_fuses.push(BombRockFuse {
                index: index as u32,
                remaining: 3,
            });
            continue;
        }
        if rock.kind != 2 || rock.state != 1 {
            continue;
        }
        let variant = rock.variant;
        let width = usize::from(grid.width);
        let repairs: &[(isize, i32)] = match variant {
            0..=2 => &[],
            1000 => &[(1, -1)],
            1001 => &[(-1, -1)],
            1002 => &[(width as isize, -1)],
            1003 => &[(-(width as isize), -1)],
            1004 => &[(width as isize + 1, 1), (width as isize, 0), (1, -1)],
            1005 => &[(width as isize, 1), (width as isize - 1, 0), (-1, -1)],
            1006 => &[(1, 3), (1 - width as isize, 2), (-(width as isize), -1)],
            1007 => &[(-1, 3), (-(width as isize) - 1, 2), (-(width as isize), -1)],
            _ => continue,
        };
        let rock = grid.cells[index].entity.as_mut().expect("selected rock");
        rock.state = 2;
        rock.collision_class = 0;
        state
            .room
            .path_costs
            .as_mut()
            .ok_or("Rock destruction requires source Room.Init path costs")?
            .store(index, 900)?;
        for &(offset, frame) in repairs {
            let Some(neighbor) = index.checked_add_signed(offset) else {
                continue;
            };
            let Some(rock) = grid
                .cells
                .get_mut(neighbor)
                .and_then(|cell| cell.entity.as_mut())
                .filter(|entity| entity.kind == 2)
            else {
                continue;
            };
            // SetType56c240(-1) chooses the normal authored frame from a new
            // local RNG35; it does not consume the shared effects/audio stream.
            rock.variant = if frame < 0 {
                crate::room_spawns::next35(decoration_seed.wrapping_add(neighbor as u32) | 1) % 3
            } else {
                1000 + frame as u32
            };
        }
        // Normal Basement Rock emits this real sound request before particles.
        // Unsupported effect families remain unadmitted, without cursor shifts.
        state.audio.admit(
            &mut state.boot_lifecycle.shared,
            state.system_clock.system_counter - 1,
            crate::audio::SoundRequest {
                id: 137,
                volume: 1.0,
                delay: 2,
                looping: false,
                pitch: 1.0,
            },
        );
    }
    Ok(())
}
