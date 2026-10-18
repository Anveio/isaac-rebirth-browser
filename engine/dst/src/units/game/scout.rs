//! Seed scouting for the seeded persona.
//!
//! Lasers, knives, fetuses and the weapon plans behind them run only once
//! the player owns a collectible that changes the primary weapon, and an
//! unseeded run seldom offers one. Before a seeded case starts, the scout starts runs for
//! candidate codes on copies of the booted game, through the same public
//! commands the browser sends, and asks the engine's own pedestal rule
//! (`Pickup::from_collectible_selector`) what the floor's treasure room will
//! offer. Codes whose collectible requests the weapon cache (authored
//! `cacheFlags`) are handed to the driver. A changed collectible order on
//! the way to the treasure room only lowers the hit rate.
use crate::entropy::Entropy;
use crate::props;
use rebirth_cleanroom::pickups::Pickup;
use rebirth_cleanroom::{items, room_assets, room_spawns, seed, Command, Game, Screen};
use std::collections::VecDeque;

/// The ItemConfig cache bit that re-resolves the primary weapon
/// (`weapon_plan::resolve_primary`).
const WEAPON_CACHE: u32 = 0x100;
const CANDIDATES: usize = 96;
const MENU_COMMANDS: usize = 400;

/// Up to `wanted` seed codes whose first treasure room offers a weapon
/// collectible.
pub fn codes(booted: &Game, e: &mut Entropy, wanted: usize) -> Vec<String> {
    // Every candidate starts from the same open seed dialog.
    let Some(dialog) = walk(booted.clone(), None) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for _ in 0..CANDIDATES {
        let Ok(code) = seed::encode(e.nonzero_u32()) else {
            continue;
        };
        let Some(game) = walk(dialog.clone(), Some(&code)) else {
            continue;
        };
        props::sometimes!(true, "game: the seed scout starts a seeded run");
        let scouted = treasure_collectible(&game).and_then(items::definition);
        if scouted.is_some_and(|config| config.cache_flags & WEAPON_CACHE != 0) {
            props::sometimes!(true, "game: a scouted seed offers a weapon collectible");
            found.push(code);
            if found.len() == wanted {
                break;
            }
        }
    }
    found
}

/// A seeded run of `code`, started from `booted` through the public menus.
pub(crate) fn seeded_run(booted: Game, code: &str) -> Option<Game> {
    walk(walk(booted, None)?, Some(code))
}

fn tap(queue: &mut VecDeque<Command>, code: &str) {
    for down in [true, false] {
        queue.push_back(Command::Key {
            code: code.into(),
            down,
        });
    }
    queue.push_back(Command::Tick {
        count: 1,
        timestamp_millis: None,
    });
}

/// Walks the menus by their published state: without a code, to the open
/// seed dialog; with one, to a seeded run of it.
fn walk(mut game: Game, code: Option<&str>) -> Option<Game> {
    let mut queue = VecDeque::new();
    for _ in 0..MENU_COMMANDS {
        if let Some(command) = queue.pop_front() {
            let _ = game.command(command);
            continue;
        }
        let state = &game.state;
        // The run's floor and item pools exist once it is Playing, before
        // its start fade ends.
        if state.screen == Screen::Playing {
            return (code.is_some() && state.seeded_run).then_some(game);
        }
        let request = &state.run_start_request;
        if request.input_pending || request.fading || request.system_pending {
            queue.push_back(Command::Tick {
                count: 1,
                timestamp_millis: None,
            });
            continue;
        }
        let entered = code.is_some_and(|code| state.seed_text == code);
        match state.screen {
            Screen::Title | Screen::Profile => tap(&mut queue, "Enter"),
            Screen::Main if state.main_selection == 0 => tap(&mut queue, "Enter"),
            Screen::Main => tap(&mut queue, "ArrowUp"),
            Screen::Character if state.seeded_run && entered => tap(&mut queue, "Enter"),
            Screen::Character => tap(&mut queue, "Tab"),
            Screen::Seed => match code {
                None => return Some(game),
                Some(_) if entered => tap(&mut queue, "Enter"),
                Some(code) => {
                    for _ in 0..state.seed_text.len() {
                        tap(&mut queue, "Backspace");
                    }
                    queue.push_back(Command::Text { text: code.into() });
                }
            },
            _ => tap(&mut queue, "Escape"),
        }
    }
    None
}

/// The collectible the floor's treasure room pedestal draws on admission.
fn treasure_collectible(game: &Game) -> Option<u32> {
    let state = &game.state;
    let descriptor = state
        .floor
        .descriptors
        .iter()
        .find(|descriptor| descriptor.template.room_type == 4)?;
    let asset = room_assets::for_descriptor(descriptor).ok()?;
    let spawns = room_spawns::select(&asset, descriptor.spawn_seed).ok()?;
    let mut pools = state.item_pools.clone()?;
    let mut shared = state.boot_lifecycle.shared.clone();
    spawns.selectors.iter().find_map(|selector| {
        Pickup::from_collectible_selector(0, selector, &mut pools, 4, &mut shared)
            .ok()
            .flatten()
            .map(|pickup| pickup.subtype)
    })
}
