//! Every room the engine admits on a run's floor, entered and played.
//!
//! Ordinary play reaches only the rooms one route crosses, and a later stage
//! only after its boss. After stage/rooms' room-entry scan (`cavesentry`),
//! each case starts a seeded run through the public menus and, for Basement II
//! or Caves I, stands the player on an open trapdoor on the stage before, so
//! the trapdoor's own entry check decides the descent. On the floor it arrives
//! on, it enters each room `traversal::admits` lets the player enter, through
//! the transition a door starts, and the game driver plays the room. The
//! transition names the starting room as the room left and the target's first
//! door as the one entered; admission reads only the target and its door.
//!
//! Only a failed or panicking Tick is a violation, signed by its error. A room
//! the engine refuses is skipped, a player who walks back out has played the
//! room, and a trapdoor that does not take the player refused the descent. A
//! transition the trapdoor starts must arrive, and a Tick that fails on the
//! way is a violation too.
use super::game::{expected_rejection, seeded_run, Driver};
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
const ARRIVAL_FRAMES: u32 = 1_500;
/// Full updates the run-start hold may keep the player's controls.
const START_FRAMES: u32 = 600;
/// Full updates an open trapdoor under the player gets to take them.
const TRAPDOOR_FRAMES: u32 = 120;

pub fn run(case: &mut Case) {
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
    // 0 stays on Basement I; 1, 2 and 3 arrive on the next stage.
    let from_stage = e.below(4) as u32;
    if from_stage > 0 && !arrive(&mut game, from_stage, e, &code) {
        return;
    }
    let stage = game.state.stage;
    props::sometimes!(stage == 1, "rooms: Basement I's rooms are played");
    props::sometimes!(stage == 2, "rooms: Basement II's rooms are played");
    props::sometimes!(stage == 3, "rooms: Caves I's rooms are played");
    props::sometimes!(stage == 4, "rooms: Caves II's rooms are played");
    let start = game.state.current_room;
    let descriptors = game.state.floor.descriptors.clone();
    let Some(from_slot) = first_door(&descriptors[start].neighbors) else {
        return;
    };
    for (index, descriptor) in descriptors.iter().enumerate() {
        if index == start || !traversal::admits(descriptor) {
            continue;
        }
        let Some(target_slot) = first_door(&descriptor.neighbors) else {
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
            from_room: start,
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

/// The stage after `from_stage`, through the trapdoor's own entry check: an
/// open trapdoor (state1, its open clip finished) under the player, who stands
/// on its cell's centre. A trapdoor that does not take the player within
/// `TRAPDOOR_FRAMES` refused, as it refuses a stage it cannot generate; one
/// that does must arrive within `ARRIVAL_FRAMES`.
fn arrive(game: &mut Game, from_stage: u32, e: &mut Entropy, code: &str) -> bool {
    let state = &mut game.state;
    let Some(width) = state.room.grid.as_ref().map(|grid| usize::from(grid.width)) else {
        return false;
    };
    state.stage = from_stage;
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
    let label = || json!({ "code": code, "at": "stage arrival", "from": from_stage });
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
            && state.stage == from_stage + 1
            && !state.stage_transition.active()
            && state.player.controls_enabled
        {
            props::sometimes!(true, "rooms: a run arrives on the next stage");
            break;
        }
    }
    let arrived = game.state.stage == from_stage + 1 && !game.state.stage_transition.active();
    props::always!(
        arrived,
        "rooms: a stage transition a trapdoor starts arrives within 1,500 full updates",
        json!({ "code": code, "from": from_stage, "stage": game.state.stage })
    );
    arrived
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
        Ok(result) => result,
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
