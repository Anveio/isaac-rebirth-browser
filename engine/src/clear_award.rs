//! Room::SpawnClearAward 5c9240, called by the clear boundary 5d19f1 after
//! the doors open and the clear SFX. Boss rooms (type 5) take the boss branch
//! in `stage_transition`; this module owns the ordinary-room branch.
//!
//! The award RNG is RNG(descriptor+50, preset 35), the same stream the boss
//! branch seeds. Two RandomFloat draws and Luck choose the variant, trinkets
//! and Guppy's Tail can replace it, Contract from Below repeats it, and each
//! award spawns at FindFreePosition 5c7f30(room centre, 0, EDX0) with the
//! award RNG's next value as its seed and a zero subtype.
use crate::{
    items::Rng,
    motion::WorldVector,
    room_spawns::{SpawnCategory, SpawnSelector},
    Pickup, State,
};

pub(crate) fn spawn(state: &mut State) -> Result<(), String> {
    let room_type = state.floor.descriptors[state.current_room]
        .template
        .room_type;
    match room_type {
        5 => return crate::stage_transition::spawn_boss_award(state),
        // 5c93d4: type 0x11 also takes the boss branch 5c9740 (PORT GAP);
        // 5c93dd: type 0x10 awards nothing.
        0x10 | 0x11 => return Ok(()),
        _ => {}
    }
    // PORT GAP: an unknown item, trinket or Luck owner leaves the award
    // unspawned rather than guessed; it draws nothing either way.
    let Some(inputs) = Inputs::read(state) else {
        return Ok(());
    };
    let Some(grid) = state.room.grid.clone() else {
        return Ok(());
    };
    let grid = crate::grid::Grid::new(grid)?;
    let Some(costs) = state.room.path_costs.clone() else {
        return Ok(());
    };
    let mut rng = Rng::new(
        state.floor.descriptors[state.current_room].following_seed,
        [5, 9, 7],
    );
    let (variant, count) = choose(&mut rng, &inputs);
    // 5c9e1c / 5c9f93: a heart award draws RandomInt(100) at hard and is
    // dropped above the rules row's limit.
    if let Some(max) = crate::difficulty::rules(state.difficulty).clear_heart_max_roll {
        if variant == 10 && rng.bounded(100) > max {
            return Ok(());
        }
    }
    if variant == 0 {
        return Ok(());
    }
    // 5c78b0: the centre cell of the room grid.
    let width = usize::from(grid.descriptor().width);
    let center_index = width / 2 + usize::from(grid.descriptor().height) / 2 * width;
    let center = grid
        .position(center_index)
        .ok_or("clear award centre cell")?;
    let mut subtype = 0;
    for _ in 0..count {
        // Every NPC room leaves the EDX0 placement history unproven (an AI
        // may have produced a type 4/5/6 actor the port lacks), so the award
        // places against the port's own ordinary actors.
        let Ok(occupied) = crate::ordinary_placement::represented_positions(state) else {
            return Ok(());
        };
        let mut shared = state.boot_lifecycle.shared.clone();
        let position = crate::roundworm::find_free_position(
            center,
            0.0,
            &mut shared,
            &grid,
            &costs,
            &occupied,
        )?;
        state.boot_lifecycle.shared = shared;
        let seed = rng.raw();
        // 5c9ea4: a repeated award reuses the first spawn's resolved subtype.
        match admit(state, variant, subtype, seed, position)? {
            Some(resolved) => subtype = resolved,
            None => return Ok(()),
        }
    }
    Ok(())
}

/// The item, trinket and Luck reads of the ordinary branch.
struct Inputs {
    /// Collectible 46 (Lucky Foot).
    lucky_foot: bool,
    /// Player+d64.
    luck: f32,
    /// Trinkets 42 (Lucky Toe), 22 (Daemon's Tail), 45, 44, 41, 34 and 36.
    lucky_toe: bool,
    daemons_tail: bool,
    replacements: [bool; 4],
    rusted_key: bool,
    /// Collectible 134 (Guppy's Tail).
    guppys_tail: bool,
    /// Collectible 241 (Contract from Below) count.
    contracts: i32,
}

impl Inputs {
    fn read(state: &State) -> Option<Self> {
        let owner = state.player_items.as_ref()?;
        let source = state.player.entity_source?;
        if source.entity_type != 1 || source.variant != 0 {
            return None;
        }
        let history = owner.query_history.as_ref()?;
        if history.cached_callbacks & 1024 == 0 {
            return None;
        }
        let queries = crate::item_queries::AppearanceQueries::new(
            owner,
            state.weapon_cache.primary,
            state.player.entity_source,
            None,
        );
        let trinket = |id| queries.has_trinket(id);
        Some(Self {
            lucky_foot: queries.have(46)?,
            luck: state.player_stats.luck?,
            lucky_toe: trinket(42)?,
            daemons_tail: trinket(22)?,
            replacements: [trinket(45)?, trinket(44)?, trinket(41)?, trinket(34)?],
            rusted_key: trinket(36)?,
            guppys_tail: queries.have(134)?,
            contracts: queries.raw_count(241)?,
        })
    }
}

/// 5c93e6..5c9e11: the award variant (0 for none) and how many to spawn.
fn choose(rng: &mut Rng, inputs: &Inputs) -> (u32, u32) {
    let mut chance = rng.uniform();
    if inputs.lucky_foot {
        chance = chance * 0.9 + 0.1;
    }
    let luck = if inputs.luck > 0.0 {
        inputs.luck.min(10.0)
    } else {
        0.0
    };
    let mut roll = rng.uniform() * luck * 0.1 + chance;
    if inputs.lucky_toe {
        roll = if inputs.lucky_foot && luck > 0.0 {
            roll * 0.98 + 0.02
        } else {
            roll * 0.9 + 0.1
        };
    }
    // 5c9511: jbe, so a NaN roll also awards nothing.
    let mut variant = if roll.partial_cmp(&0.22) != Some(std::cmp::Ordering::Greater) {
        0
    } else if roll < 0.3 {
        if rng.bounded(3) == 0 {
            300
        } else if rng.bounded(2) == 0 {
            350
        } else {
            70
        }
    } else if roll < 0.45 {
        20
    } else if roll < 0.6 {
        if inputs.daemons_tail && rng.bounded(5) != 0 {
            30
        } else {
            10
        }
    } else if roll < 0.8 {
        30
    } else if roll < 0.95 {
        40
    } else {
        50
    };
    if variant != 0 {
        variant = replace(rng, inputs, variant);
    }
    // 5c9d14: Guppy's Tail turns a third of awards into chests, a third
    // into nothing.
    if inputs.guppys_tail {
        if rng.bounded(3) == 0 {
            variant = if rng.bounded(2) == 0 { 50 } else { 60 };
        } else if rng.bounded(3) == 0 {
            variant = 0;
        }
    }
    let mut count = 1;
    if inputs.contracts > 0 && variant != 350 {
        // 5c9dc6: pow(0.666, n) * 0.5 against a RandomFloat; n + 1 awards.
        let odds = (crate::fmath::pow(0.666, f64::from(inputs.contracts)) * 0.5) as f32;
        count = if odds > rng.uniform() {
            0
        } else {
            inputs.contracts as u32 + 1
        };
    }
    (variant, count)
}

/// 5c9570: trinkets 45, 44, 41 and 34 each replace the award on a 1-in-10
/// roll (card, pill, bomb, heart); Rusted Key 36 then makes it a key.
fn replace(rng: &mut Rng, inputs: &Inputs, variant: u32) -> u32 {
    for (held, replacement) in inputs.replacements.iter().zip([300, 70, 40, 10]) {
        if *held && rng.bounded(10) == 0 {
            // 5cba2b: Child's Heart's heart yields to Daemon's Tail's key
            // check below unless its 1-in-5 roll is 0.
            if replacement == 10 && inputs.daemons_tail && rng.bounded(5) != 0 {
                break;
            }
            return replacement;
        }
    }
    if inputs.rusted_key && rng.bounded(10) == 0 {
        return 30;
    }
    variant
}

/// Game.Spawn550a40(5, variant, subtype, position, zero velocity, no spawner,
/// seed) through the room-spawn factories. Returns the resolved subtype, or
/// None when the port has no factory for the pickup (PORT GAP).
fn admit(
    state: &mut State,
    variant: u32,
    subtype: u32,
    seed: u32,
    position: WorldVector,
) -> Result<Option<u32>, String> {
    let id = state.next_pickup;
    let selector = SpawnSelector {
        group_index: 0,
        grid_index: 0,
        category: SpawnCategory::Entity,
        entity_type: 5,
        variant: u16::try_from(variant).map_err(|_| "clear award variant")?,
        subtype: u16::try_from(subtype).map_err(|_| "clear award subtype")?,
        position,
        group_seed: 0,
        spawn_seed: seed,
    };
    let pickup = match variant {
        70 | 300 => Pickup::from_pocket_selector(id, &selector, true, state)?,
        60 => crate::chests::from_locked_selector(id, &selector, true, state)?,
        50 => crate::chests::from_source_selector(id, &selector, true, state)?,
        _ => Pickup::from_source_selector(id, &selector, true, state)?,
    };
    let Some(mut pickup) = pickup else {
        if variant == 350 {
            crate::item_queries::note_unrepresented_trinket_spawn(state);
        }
        crate::ordinary_placement::invalidate(
            state,
            crate::ordinary_placement::Frontier::Factory {
                entity_type: 5,
                variant,
                subtype,
            },
        );
        return Ok(None);
    };
    pickup.birth_game_update = state.game_update_count;
    let resolved = pickup.subtype;
    state.next_pickup = id.checked_add(1).ok_or("clear award identity exhausted")?;
    state.room.pickups.push(pickup);
    let actor = crate::actors::ActorRef {
        kind: crate::actors::ActorKind::Pickup,
        id,
    };
    state.room.authored_actor_order.push(actor);
    state.actor_scheduler.admit(actor)?;
    Ok(Some(resolved))
}
