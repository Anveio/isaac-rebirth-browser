//! Ordinary Basement Rock destruction reached through a real bomb blast;
//! poop takes the blast as Hurt(4) (`attack_grid`).
//!
//! The callback changes collision, the room's destruction marker and linked
//! authored frames. Tinted rocks (kind4) also run their reward 56d440.
//! Rubble/worm effect actors and alt-rock drops remain unadmitted; no guessed
//! loot or placeholder random draws run.
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
        let grid = state
            .room
            .grid
            .as_mut()
            .ok_or("bomb rock requires a room grid")?;
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
        // Rock.Destroy56c8ec/56ca46: the same crater and explosion spawns as
        // a bomb. The explosion's light owner is keyed apart from bomb ids.
        crate::bombs::spawn_blast_effects(state, 0x8000_0000 | cell as u32, position);
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
        let kind = state
            .room
            .grid
            .as_ref()
            .and_then(|grid| grid.cells[index].entity.as_ref())
            .map(|entity| entity.kind);
        if kind == Some(14) {
            // DestroyGrid5cbb70 -> Poop's base Destroy561c00 is Hurt(4).
            let outcome = crate::attack_grid::callback_at_index(state, index, 4)?;
            outcome.record_frontier(state, "bombGridHurt");
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
        if !matches!(rock.kind, 2 | 4) || rock.state != 1 {
            continue;
        }
        let tinted = rock.kind == 4;
        // Destroy56c670: the rock-local RNG35 on the spawn seed. Its state1
        // pre-step always draws Int(50); a zero rolls rare Worm effects
        // (1000.63, shared MT), unadmitted like the rubble particles.
        let mut local = crate::items::Rng::new(rock.spawn_seed, [5, 9, 7]);
        if local.bounded(50) == 0 {
            let worms = local.bounded(5);
            local.bounded(worms + 1);
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
        if tinted {
            tinted_reward(state, index, &mut local)?;
        }
    }
    Ok(())
}

/// Tinted rock reward 56d440 for a fresh character without trinkets: one or
/// two soul hearts, keys or bombs, or a single (locked) chest. Seeds come from
/// the rock-local RNG; velocities from the shared Chest scatter.
fn tinted_reward(
    state: &mut State,
    index: usize,
    local: &mut crate::items::Rng,
) -> Result<(), String> {
    let grid = Grid::new(
        state
            .room
            .grid
            .clone()
            .ok_or("tinted reward requires room grid")?,
    )?;
    let position = grid.position(index).ok_or("tinted rock outside its grid")?;
    // Int(10)==0 offers Small Rock only once Persistent+0x3c unlocks it; a
    // fresh profile has not destroyed the tinted rocks that unlock it.
    local.bounded(10);
    let mut count = 2;
    if local.bounded(3) != 0 {
        // Its velocity only feeds trinket bonuses, which need trinkets.
        crate::chests::scatter(position, &grid, &mut state.boot_lifecycle.shared);
    }
    let mut roll = 0;
    while roll < count {
        let choice = local.uniform();
        let velocity = crate::chests::scatter(position, &grid, &mut state.boot_lifecycle.shared);
        let (variant, subtype, last) = if choice < 0.6 {
            (10, Some(3), false)
        } else if choice < 0.7 {
            (30, None, false)
        } else if roll == 0 && choice < 0.71 {
            (50, None, true)
        } else if roll == 0 && choice < 0.9 {
            (60, Some(1), true)
        } else {
            (40, None, false)
        };
        let seed = local.raw();
        admit_reward(state, variant, subtype, seed, position, velocity)?;
        if last {
            return Ok(());
        }
        if variant == 10 && local.bounded(2) == 0 {
            count -= 1;
        }
        roll += 1;
    }
    Ok(())
}

/// Spawns one tinted-rock child immediately, as the source does before the
/// next local draw.
pub(crate) fn admit_reward(
    state: &mut State,
    variant: u32,
    subtype: Option<u32>,
    seed: u32,
    position: WorldVector,
    velocity: WorldVector,
) -> Result<(), String> {
    let id = state.next_pickup;
    let mut pickup = match variant {
        50 => {
            let selector = crate::room_spawns::SpawnSelector {
                group_index: 0,
                grid_index: 0,
                category: crate::room_spawns::SpawnCategory::Entity,
                entity_type: 5,
                variant: 50,
                subtype: 0,
                position,
                group_seed: seed,
                spawn_seed: seed,
            };
            // Pickup::Init's Random20/Random50 replacements stay unsupported.
            let Some(chest) = crate::chests::from_selector(id, &selector, true)? else {
                return Err("tinted rock chest replacement is unsupported".into());
            };
            chest
        }
        60 => {
            // Init4ffef2 forces the closed subtype1; Locked Chest opening
            // needs a key and is not admitted.
            let mut chest = crate::pickups::Pickup::spawn(id, 60, 1, position, true)?;
            chest.spawn_seed = seed;
            chest.chest = Some(crate::chests::Chest {
                rng: crate::items::Rng::new(seed, [4, 3, 17]),
                grid_collision_class: 5,
                entity_collision_class: 0,
                price: 0,
                cooldown: 0,
                loot_births: Vec::new(),
            });
            chest
        }
        _ => {
            let subtype = match subtype {
                Some(subtype) => subtype,
                None => crate::pickups::regular_subtype(variant, 0, seed)?,
            };
            let mut pickup = crate::pickups::Pickup::spawn(id, variant, subtype, position, true)?;
            pickup.initialize_spawn_seed(seed)?;
            pickup
        }
    };
    pickup.velocity = velocity;
    pickup.birth_game_update = state.game_update_count;
    state.next_pickup = id
        .checked_add(1)
        .ok_or("tinted reward identity exhausted")?;
    state.room.pickups.push(pickup);
    let actor = crate::actors::ActorRef {
        kind: crate::actors::ActorKind::Pickup,
        id,
    };
    state.room.authored_actor_order.push(actor);
    state.actor_scheduler.admit(actor)
}
