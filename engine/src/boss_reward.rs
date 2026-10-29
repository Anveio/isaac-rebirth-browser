//! Room::TriggerBossDeath 5c8430, which 4f2d10 runs for a dead boss (+103d)
//! before it removes it.
//!
//! The callback sets the boss's final flag (0x100000) and decrements the
//! room's boss count (Room+2490, refreshed from the manager's boss words by
//! every Room::Update and raised by each boss Init). Only the death that takes
//! the count to zero or below rewards:
//!
//! - A sin (46..52) drops from its own RNG: RandomInt(4) == 0 (or, for a
//!   super sin, a further RandomInt(7) == 0) spawns the sin's collectible,
//!   otherwise its pickups (jump table 67a768).
//! - Any other boss in a boss (5) or Boss Rush (0x11) room drops a heart at
//!   normal (5c87c3), and a second while achievement 33 is unset (5c88a0).
//!
//! Every drop takes the boss's own RNG's next value as its seed and the boss
//! as its spawner.
use crate::{items::Rng, motion::WorldVector, pickups::SourceSpawner, State};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

/// The boss callback for the dead boss at `index` in the room's NPCs.
pub(crate) fn trigger(state: &mut State, index: usize) -> Result<(), String> {
    // 5c84ae..5c84d2: TriggerBossDeath logs and stores the decremented count.
    state.room.boss_count = state.room.boss_count.wrapping_sub(1);
    if state.room.boss_count as i32 > 0 {
        return Ok(());
    }
    let npc = &state.room.npcs[index];
    let (kind, variant, subtype) = (npc.kind, npc.variant, npc.subtype);
    let mut rng = crate::champions::entity_rng(npc);
    let room_type = state.floor.descriptors[state.current_room]
        .template
        .room_type;
    if (46..=52).contains(&kind) {
        sin(state, index, &mut rng, kind, variant)?;
    } else {
        if kind == 81 && variant == 1 {
            return Err("Krampus's boss callback is not admitted".into());
        }
        // 5c879b..5c87cb: a boss or Boss Rush room drops hearts at normal.
        if matches!(room_type, 5 | 0x11) && crate::difficulty::rules(state.difficulty).boss_heart {
            let Some(daemons_tail) = daemons_tail(state) else {
                // PORT GAP: an unknown trinket owner leaves the hearts unspawned.
                return Ok(());
            };
            heart(state, index, &mut rng, daemons_tail)?;
            // 5c88a0: Persistent+51 is achievement 33.
            if !achievement(state, 33)? {
                heart(state, index, &mut rng, daemons_tail)?;
            }
        }
        if subtype != 0 {
            // 5c88b8: a boss champion's heart (soul for colours 3 and 6); the
            // port admits no boss champions.
            return Err("boss champion heart is not admitted".into());
        }
        if room_type == 15 {
            // 5c8fd0: the angel room's FindFreePosition and key pieces.
            return Err("angel room boss callback is not admitted".into());
        }
    }
    // 5bec80 / 5bed50: the sin or boss kill record marks Persistent dirty.
    // PORT GAP: the kill records themselves and the achievements they
    // complete (22, 86, ...) have no profile owner.
    if let Some(runtime) = &mut state.persistent_runtime {
        if !runtime.inhibit && runtime.dirty.is_some() {
            runtime.dirty = Some(true);
        }
    }
    Ok(())
}

/// 5c8750..5c8f83: a sin's drop.
fn sin(
    state: &mut State,
    index: usize,
    rng: &mut Rng,
    kind: u32,
    variant: u32,
) -> Result<(), String> {
    if kind == 46 && variant == 2 {
        // 5c89e8: Ultra Pride's trinket and achievement.
        return Err("Ultra Pride's boss callback is not admitted".into());
    }
    let sup = variant == 1;
    // 5c86b0 / 5c8a90.
    let collectible = rng.bounded(4) == 0 || rng.bounded(7) == 0 && sup;
    let position = state.room.npcs[index].position;
    if collectible {
        // Jump table 67a7a0: the sin's collectible, its super form's for a
        // super sin. Greed's super form needs achievement 10 (Persistent+3a).
        let id = match (kind, sup) {
            (46, false) => 42,
            (46, true) => 140,
            (47, false) => 13,
            (47, true) => 45,
            (48, false) => 37,
            (48, true) => 106,
            (49, false) => 15,
            (49, true) => 129,
            (50, false) => 64,
            (50, true) => {
                if achievement(state, 10)? {
                    74
                } else {
                    64
                }
            }
            (51, false) => 49,
            (51, true) => 128,
            (52, false) => 65,
            _ => 81,
        };
        let Some(position) = free_position(state, position)? else {
            return Ok(());
        };
        let seed = rng.raw();
        crate::pickups::admit_collectible(state, id, position, seed)?;
        let source = spawner(state, index);
        if let Some(pickup) = state.room.pickups.last_mut() {
            pickup.source_spawner = Some(source);
        }
        // 581750: the collectible leaves the item pools.
        state
            .item_pools
            .as_mut()
            .ok_or("sin collectible requires the run item pools")?
            .removed[id as usize] = true;
        return Ok(());
    }
    match kind {
        // Sloth, Lust and Pride: a card or pill at FindFreePosition.
        46 | 47 | 52 => {
            let variant = if kind == 47 { 70 } else { 300 };
            let Some(position) = free_position(state, position)? else {
                return Ok(());
            };
            let seed = rng.raw();
            spawn(state, index, variant, 0, seed, position, ZERO)?;
        }
        // Wrath: two bombs; Gluttony: two hearts, the second kept by
        // Daemon's Tail only on RandomInt(2) == 0 of its trinket RNG. Each
        // pair flies apart at RandomVector * 5 from the sin's position.
        48 | 49 => {
            let variant = if kind == 48 { 40 } else { 10 };
            let velocity = crate::boil::direction(&mut state.boot_lifecycle.shared) * 5.0;
            let seed = rng.raw();
            spawn(state, index, variant, 0, seed, position, velocity)?;
            if kind == 49 {
                let Some(daemons_tail) = daemons_tail(state) else {
                    // PORT GAP: an unknown trinket owner leaves it unspawned.
                    return Ok(());
                };
                if daemons_tail && trinket_rng(state)?.bounded(2) != 0 {
                    return Ok(());
                }
            }
            let seed = rng.raw();
            spawn(state, index, variant, 0, seed, position, velocity * -1.0)?;
        }
        // Greed: RandomInt(5) + 4 coins, each at RandomVector * (RandomFloat
        // * 5 + 2).
        50 => {
            let count = rng.bounded(5) + 4;
            for _ in 0..count {
                let speed = state.boot_lifecycle.shared.uniform() * 5.0 + 2.0;
                let velocity = crate::boil::direction(&mut state.boot_lifecycle.shared) * speed;
                let seed = rng.raw();
                spawn(state, index, 20, 0, seed, position, velocity)?;
            }
        }
        // Envy: a 5.40.3 bomb at the sin's position.
        _ => {
            let seed = rng.raw();
            spawn(state, index, 40, 3, seed, position, ZERO)?;
        }
    }
    Ok(())
}

/// 5c87ee..5c88a0: unless Daemon's Tail's RandomInt(5) of its trinket RNG
/// is nonzero, a 5.10 heart flies from FindFreePosition(boss) at
/// RandomVector * 7.
fn heart(state: &mut State, index: usize, rng: &mut Rng, daemons_tail: bool) -> Result<(), String> {
    if daemons_tail && trinket_rng(state)?.bounded(5) != 0 {
        return Ok(());
    }
    let seed = rng.raw();
    let velocity = crate::boil::direction(&mut state.boot_lifecycle.shared) * 7.0;
    let anchor = state.room.npcs[index].position;
    let Some(position) = free_position(state, anchor)? else {
        return Ok(());
    };
    spawn(state, index, 10, 0, seed, position, velocity)
}

/// FindFreePosition 5c7f30(anchor, 0, EDX0) against the port's ordinary
/// actors. None when the room has no grid or path costs.
pub(crate) fn free_position(
    state: &mut State,
    anchor: WorldVector,
) -> Result<Option<WorldVector>, String> {
    let (Some(grid), Some(costs)) = (state.room.grid.clone(), state.room.path_costs.clone()) else {
        return Ok(None);
    };
    let grid = crate::grid::Grid::new(grid)?;
    let Ok(occupied) = crate::ordinary_placement::represented_positions(state) else {
        return Ok(None);
    };
    crate::roundworm::find_free_position(
        anchor,
        0.0,
        &mut state.boot_lifecycle.shared,
        &grid,
        &costs,
        &occupied,
    )
    .map(Some)
}

fn spawn(
    state: &mut State,
    index: usize,
    variant: u32,
    subtype: u32,
    seed: u32,
    position: WorldVector,
    velocity: WorldVector,
) -> Result<(), String> {
    let spawner = Some(spawner(state, index));
    crate::clear_award::admit(state, variant, subtype, seed, position, velocity, spawner)?;
    Ok(())
}

fn spawner(state: &State, index: usize) -> SourceSpawner {
    let npc = &state.room.npcs[index];
    SourceSpawner {
        entity_type: npc.kind,
        variant: npc.variant,
    }
}

/// Trinket 22 (Daemon's Tail) on the first player, or None when the player's
/// trinket owner is unknown.
fn daemons_tail(state: &State) -> Option<bool> {
    let owner = state.player_items.as_ref()?;
    crate::item_queries::AppearanceQueries::new(
        owner,
        state.weapon_cache.primary,
        state.player.entity_source,
        None,
    )
    .has_trinket(22)
}

/// Player+2b34: trinket 22's RNG.
fn trinket_rng(state: &mut State) -> Result<&mut Rng, String> {
    state
        .player_items
        .as_mut()
        .and_then(|owner| owner.trinket_rngs.get_mut(22))
        .ok_or_else(|| "Daemon's Tail requires the player's trinket RNGs".into())
}

/// Persistent+30+id: the active profile's achievement.
fn achievement(state: &State, id: usize) -> Result<bool, String> {
    state
        .profiles
        .get(usize::from(state.profile.saturating_sub(1)))
        .and_then(|profile| profile.achievements.get(id).copied())
        .ok_or_else(|| "boss callback requires the active profile's achievements".into())
}
