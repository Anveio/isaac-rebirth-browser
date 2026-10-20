//! Every room the engine admits on a run's floor, entered and played.
//!
//! Ordinary play reaches only the rooms one route crosses, and a later stage
//! only after its boss. After stage/rooms' room-entry scan (`cavesentry`),
//! each case starts a seeded run through the public menus and descends: on
//! each floor it stands the player on an open trapdoor, so the trapdoor's own
//! entry check decides the next descent, until a trapdoor refuses. Every
//! stage the port admits joins without a change here. The case plays one of
//! the floors it reached, chosen uniformly: it enters each room
//! `traversal::admits` lets the player enter, through the transition a door
//! starts, and the game driver plays the room. The transition leaves the
//! neighbor behind the target's first door through that neighbor's door to
//! it, as walking in does, while the starting room stays the one the engine
//! persists; admission reads only the target and its door.
//!
//! Only a failed or panicking Tick is a violation, signed by its error. A room
//! the engine refuses is skipped, a player who walks back out has played the
//! room, and a trapdoor that does not take the player refused the descent. A
//! transition a trapdoor starts must arrive, and a Tick that fails on the way
//! is a violation too.
use super::game::{expected_rejection, seeded_run, state_invariants, Driver};
use crate::coverage;
use crate::entropy::Entropy;
use crate::harness::{self, Case};
use crate::props;
use rebirth_cleanroom::stage_transition::{Trapdoor, TrapdoorClip};
use rebirth_cleanroom::traversal::{self, RoomTransition, TransitionPhase};
use rebirth_cleanroom::{seed, Command, Game};
use serde_json::{json, Value};
use std::panic::{self, AssertUnwindSafe};

/// Full updates a stage transition may take before it counts as stuck;
/// stage/rooms measured about 860.
pub(crate) const ARRIVAL_FRAMES: u32 = 1_500;
/// Full updates the run-start hold may keep the player's controls.
pub(crate) const START_FRAMES: u32 = 600;
/// Full updates an open trapdoor under the player gets to take them.
pub(crate) const TRAPDOOR_FRAMES: u32 = 120;
/// Descents a case may chain, more than a run has stages, so only a
/// refusing trapdoor ends the chain.
pub(crate) const DESCENTS: usize = 16;

pub fn run(case: &mut Case) {
    let _census = coverage::rooms_case();
    let e = &mut *case.entropy;
    let Ok(code) = seed::encode(e.nonzero_u32()) else {
        return;
    };
    let Ok(booted) = Game::new(e.nonzero_u32()) else {
        return;
    };
    let Some(mut game) = seeded_run(booted, &code) else {
        return;
    };
    // Past the run-start hold, in the first floor's starting room.
    for _ in 0..START_FRAMES {
        if game.state.player.controls_enabled {
            break;
        }
        if !tick(
            &mut game,
            1,
            || json!({ "code": code, "at": "first floor" }),
        ) {
            return;
        }
    }
    // Every floor the run's trapdoors reach, Basement I first. The case
    // plays them all, from one it picks: every reached floor is played
    // whatever the case count, and a case seed replays its first floor
    // exactly as when a case played only that one.
    let mut floors = vec![game];
    while floors.len() <= DESCENTS {
        let Some(floor) = floors.last() else { break };
        let mut next = floor.clone();
        if !descend(&mut next, e, &code) {
            break;
        }
        floors.push(next);
    }
    let first = e.below(floors.len() as u64) as usize;
    let rest = (0..floors.len()).filter(|&index| index != first);
    for index in std::iter::once(first).chain(rest) {
        play_floor(&floors[index], e, &code);
    }
}

/// Enters and plays every room `traversal::admits` on the floor `game`
/// stands on, each from a copy of it.
fn play_floor(game: &Game, e: &mut Entropy, code: &str) {
    let stage = game.state.stage;
    props::sometimes!(stage == 1, "rooms: Basement I's rooms are played");
    props::sometimes!(stage == 2, "rooms: Basement II's rooms are played");
    props::sometimes!(stage == 3, "rooms: Caves I's rooms are played");
    props::sometimes!(stage == 4, "rooms: Caves II's rooms are played");
    let start = game.state.current_room;
    let descriptors = game.state.floor.descriptors.clone();
    for (index, descriptor) in descriptors.iter().enumerate() {
        if index == start || !traversal::admits(descriptor) {
            continue;
        }
        let Some(target_slot) = first_door(&descriptor.neighbors) else {
            continue;
        };
        // The player arrives as walking in does: from the neighbor behind
        // the target's first door, through that neighbor's door to it (the
        // one facing it, when the neighbor has several).
        let Some(from_room) = descriptor.neighbors[usize::from(target_slot)] else {
            continue;
        };
        let facing = (target_slot % 4 + 2) % 4;
        let doors_to_target = || {
            descriptors[from_room]
                .neighbors
                .iter()
                .enumerate()
                .filter(|(_, neighbor)| **neighbor == Some(index))
                .map(|(slot, _)| slot as u8)
        };
        let Some(from_slot) = doors_to_target()
            .find(|slot| slot % 4 == facing)
            .or_else(|| doors_to_target().next())
        else {
            continue;
        };
        let template = &descriptor.template;
        props::sometimes!(
            (template.width, template.height) != (13, 7),
            "rooms: a large room is played"
        );
        props::sometimes!(template.room_type == 5, "rooms: a boss room is played");
        let mut entered = game.clone();
        entered.state.room_transition = Some(RoomTransition {
            from_room,
            target_room: index,
            from_slot,
            target_slot,
            phase: TransitionPhase::Out,
            full_updates: 0,
            boss_scene: None,
        });
        entered.state.player.controls_enabled = false;
        let frames = e.range(300, 900);
        let label = || {
            json!({
                "code": code,
                "stage": stage,
                "room": index,
                "template": [template.room_type, template.variant, &template.name],
            })
        };
        if !play(&mut entered, frames, e, label) {
            continue;
        }
        props::sometimes!(
            entered.state.current_room == index,
            "rooms: the player is still in the entered room after playing it"
        );
    }
}

/// The next floor, through the trapdoor's own entry check: an open trapdoor
/// (state 1, its open clip finished) under the player, who stands on its
/// cell's centre. A trapdoor that does not take the player within
/// `TRAPDOOR_FRAMES` refused, as it refuses a stage it cannot generate; one
/// that does must arrive within `ARRIVAL_FRAMES`, on a deeper stage (an XL
/// floor skips one).
fn descend(game: &mut Game, e: &mut Entropy, code: &str) -> bool {
    let from = game.state.stage;
    if !plant_trapdoor(game, e) {
        return false;
    }
    let label = || json!({ "code": code, "at": "stage arrival", "from": from });
    let mut started = false;
    for frame in 0..ARRIVAL_FRAMES {
        if !tick(game, 1, label) {
            return false;
        }
        let state = &game.state;
        started |= state.stage_transition.active();
        // A refusal is no violation, nor something to require: refusals
        // shrink as the port admits more stages.
        if !started && frame >= TRAPDOOR_FRAMES {
            return false;
        }
        if started
            && state.stage > from
            && !state.stage_transition.active()
            && state.player.controls_enabled
        {
            props::sometimes!(true, "rooms: a run arrives on the next stage");
            break;
        }
    }
    let arrived = game.state.stage > from && !game.state.stage_transition.active();
    props::always!(
        arrived,
        "rooms: a stage transition a trapdoor starts arrives within 1,500 full updates",
        json!({ "code": code, "from": from, "stage": game.state.stage })
    );
    arrived
}

/// Stands the player on their cell's centre over an open trapdoor (state 1,
/// its open clip finished), so the trapdoor's own entry check decides the
/// descent; false outside a room grid.
pub(crate) fn plant_trapdoor(game: &mut Game, e: &mut Entropy) -> bool {
    let state = &mut game.state;
    let Some(width) = state.room.grid.as_ref().map(|grid| usize::from(grid.width)) else {
        return false;
    };
    // The player's world position (the published position is 0.65 world
    // scale, offset 32, -46) and the grid cell it stands in.
    let x = (state.player.x - 32.0) / 0.65;
    let y = (state.player.y + 46.0) / 0.65;
    let column = ((x - 40.0) / 40.0).round().max(0.0) as usize;
    let row = ((y - 120.0) / 40.0).round().max(0.0) as usize;
    state.player.x = (40 + column * 40) as f64 * 0.65 + 32.0;
    state.player.y = (120 + row * 40) as f64 * 0.65 - 46.0;
    state.player.vx = 0.0;
    state.player.vy = 0.0;
    state.room.trapdoor = Some(Trapdoor {
        grid_index: row * width + column,
        state: 1,
        spawn_seed: e.u32().max(1),
        clip: TrapdoorClip::OpenAnimation,
        frame: 10.0,
        playing: false,
    });
    true
}

/// The driver plays `frames` full updates in the entered room; false after a
/// violation.
fn play(game: &mut Game, frames: u32, e: &mut Entropy, label: impl Fn() -> Value) -> bool {
    let mut driver = Driver::new(e);
    let mut played = 0;
    while played < frames {
        let command = driver.next(&game.state, e);
        // The driver also sends Ticks the engine must reject, some of them
        // far past any frame budget.
        if let Command::Tick { count, .. } = &command {
            played = played.saturating_add(*count);
        }
        if !apply(game, command, &label) {
            return false;
        }
    }
    true
}

fn tick(game: &mut Game, count: u32, label: impl Fn() -> Value) -> bool {
    apply(
        game,
        Command::Tick {
            count,
            timestamp_millis: None,
        },
        &label,
    )
}

/// Applies one command; a Tick that fails or any command that panics is a
/// violation, and either ends the room.
fn apply(game: &mut Game, command: Command, label: &impl Fn() -> Value) -> bool {
    let expected = expected_rejection(&game.state, &command);
    let tick = matches!(command, Command::Tick { .. });
    let result = match panic::catch_unwind(AssertUnwindSafe(|| game.command(command.clone()))) {
        Ok(result) => {
            coverage::observe(&game.state);
            // The game unit's state invariants hold on every floor the
            // descent reaches; its must-hits stay the game unit's to meet.
            props::always_only(|| state_invariants(&game.state));
            result
        }
        Err(_) => {
            props::unreachable!(
                "rooms: entering and playing rooms never panics the engine",
                json!({ "case": label(), "command": command, "panic": harness::take_panic() })
            );
            return false;
        }
    };
    if tick {
        props::always!(
            expected.is_some() || result.is_ok(),
            "rooms: no Tick fails while a run enters and plays its floor's rooms",
            json!({ "case": label(), "command": command, "error": result.as_ref().err() }),
            signature: result.as_ref().err().map_or("", String::as_str)
        );
    }
    !tick || expected.is_some() || result.is_ok()
}

/// The first door slot with a neighbouring room.
fn first_door(neighbors: &[Option<usize>; 8]) -> Option<u8> {
    neighbors
        .iter()
        .position(Option::is_some)
        .and_then(|slot| u8::try_from(slot).ok())
}
