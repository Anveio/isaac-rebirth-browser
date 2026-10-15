//! The whole engine through its public command boundary (`engine/src/lib.rs`
//! and every system it dispatches).
//!
//! Each case boots a game from entropy-chosen seeds, drives public commands,
//! and checks state invariants, transition contracts and rejection
//! semantics after every step. At the end it replays the tape for
//! determinism, resumes from a serialized midpoint for snapshot completeness,
//! drives the C ABI used by the browser, and (with `--wasm`) the shipped WASM.
mod driver;
mod invariants;
mod scout;

use crate::coverage;
use crate::entropy::Entropy;
use crate::harness::{self, Case};
use crate::json_diff;
use crate::props;
use crate::tape::Tape;
use rebirth_cleanroom::{Command, Game, State};
use serde_json::json;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::panic::{self, AssertUnwindSafe};

pub fn run(case: &mut Case) {
    let replay = case.replay.take();
    let replaying = replay.is_some();
    // `--resume`: replay a prefix (a boss room entered, say), then branch.
    let resume = case.resume.take();
    let prefix = resume.as_ref().map_or(&[][..], |tape| &tape.commands[..]);
    let tape = match (replay, &resume) {
        (Some(tape), _) => tape,
        (None, Some(resume)) => Tape::new(resume.initial_seed, resume.boot_epoch_seed),
        (None, None) => {
            let e = &mut *case.entropy;
            let initial_seed = e.nonzero_u32();
            let boot_epoch_seed = match e.below(4) {
                0 => 0,
                1 => 1_700_000_000 + e.range(0, 200_000_000),
                _ => e.u32(),
            };
            Tape::new(initial_seed, boot_epoch_seed)
        }
    };
    let violations = props::violations();
    let Some(mut simulation) = Simulation::boot(tape.initial_seed, tape.boot_epoch_seed) else {
        return;
    };
    let mut driver = driver::Driver::new(case.entropy);
    if !replaying && driver.seeded() {
        let codes = scout::codes(&simulation.game, case.entropy);
        driver.scout(codes);
    }
    let budget = if replaying {
        tape.commands.len()
    } else {
        prefix.len() + case.steps as usize
    };
    for step in 0..budget {
        props::set_step(step as u64);
        let command = if replaying {
            tape.commands[step].clone()
        } else if let Some(command) = prefix.get(step) {
            command.clone()
        } else {
            driver.next(&simulation.game.state, case.entropy)
        };
        if !simulation.step(command, case.entropy) {
            case.retain(&simulation.tape, "panic");
            return;
        }
    }
    simulation.finish(case);
    if props::violations() != violations {
        case.retain(&simulation.tape, "violation");
    } else if case.retain_all && !replaying {
        case.retain(&simulation.tape, "case");
    }
}

/// Batched Ticks up to this length are re-run as single Ticks when sampled.
const BATCH_SAMPLE_TICKS: u32 = 32;

struct Simulation {
    game: Game,
    tape: Tape,
    /// Ok/Err outcome and state digest after each command.
    trace: Vec<(bool, u64)>,
}

fn digest(state: &State) -> u64 {
    let mut hasher = DefaultHasher::new();
    serde_json::to_vec(state)
        .expect("state serializes")
        .hash(&mut hasher);
    hasher.finish()
}

/// Names what a lossy round trip lost: the first JSON pointer where the
/// re-serialized value differs from the original text or, when both texts
/// agree, where the values' `Debug` forms first differ (a skipped field, or a
/// NaN that serializes as `null`).
fn round_trip_difference<T: std::fmt::Debug>(
    original: &str,
    round_tripped: Option<&str>,
    value: &T,
    restored: Option<&T>,
) -> serde_json::Value {
    let (Some(round_tripped), Some(restored)) = (round_tripped, restored) else {
        return serde_json::Value::Null;
    };
    let (Ok(original), Ok(round_tripped)) = (
        serde_json::from_str::<serde_json::Value>(original),
        serde_json::from_str::<serde_json::Value>(round_tripped),
    ) else {
        return serde_json::Value::Null;
    };
    match json_diff::first(&original, &round_tripped) {
        Some((path, original, round_tripped)) => {
            json!({ "path": path, "original": original, "roundTripped": round_tripped })
        }
        None => {
            let (value, restored) = (format!("{value:?}"), format!("{restored:?}"));
            let at = value
                .char_indices()
                .zip(restored.chars())
                .find(|((_, a), b)| a != b)
                .map_or(value.len().min(restored.len()), |((index, _), _)| index);
            let window = |text: &str| {
                let start = text.floor_char_boundary(at.saturating_sub(160));
                let end = text.ceil_char_boundary((at + 80).min(text.len()));
                text[start..end].to_owned()
            };
            json!({
                "identicalJson": true,
                "value": window(&value),
                "restored": window(&restored),
            })
        }
    }
}

fn command_engine(game: &mut Game, command: Command) -> Result<Result<(), String>, String> {
    panic::catch_unwind(AssertUnwindSafe(|| game.command(command)))
        .map_err(|_| harness::take_panic())
}

impl Simulation {
    fn boot(initial_seed: u32, boot_epoch_seed: u32) -> Option<Self> {
        let booted =
            panic::catch_unwind(|| Game::new_with_boot_epoch(initial_seed, boot_epoch_seed));
        let details = json!({ "initialSeed": initial_seed, "bootEpochSeed": boot_epoch_seed });
        match booted {
            Ok(Ok(game)) => {
                props::always!(true, "game: every nonzero seed and epoch boots to Title");
                props::always!(
                    game.state.screen == rebirth_cleanroom::Screen::Title
                        && game.state.initial_seed == initial_seed
                        && game.state.boot_epoch_seed == boot_epoch_seed,
                    "game: boot publishes the first Title with its seeds",
                    details
                );
                let again = Game::new_with_boot_epoch(initial_seed, boot_epoch_seed);
                props::always!(
                    again.as_ref().ok() == Some(&game),
                    "game: boot is a pure function of its seeds"
                );
                invariants::state(&game.state);
                coverage::begin_case();
                coverage::observe(&game.state);
                Some(Self {
                    game,
                    tape: Tape::new(initial_seed, boot_epoch_seed),
                    trace: Vec::new(),
                })
            }
            Ok(Err(error)) => {
                props::always!(
                    false,
                    "game: every nonzero seed and epoch boots to Title",
                    json!({ "initialSeed": initial_seed, "bootEpochSeed": boot_epoch_seed, "error": error })
                );
                None
            }
            Err(_) => {
                props::unreachable!(
                    "game: public commands never panic the engine",
                    json!({ "initialSeed": initial_seed, "panic": harness::take_panic(), "command": "boot" })
                );
                None
            }
        }
    }

    /// Applies one command with every per-step check; false ends the case.
    fn step(&mut self, command: Command, e: &mut Entropy) -> bool {
        let before = self.game.clone();
        self.tape.commands.push(command.clone());
        let result = match command_engine(&mut self.game, command.clone()) {
            Ok(result) => result,
            Err(panic) => {
                props::unreachable!(
                    "game: public commands never panic the engine",
                    json!({ "command": command, "panic": panic, "screen": before.state.screen })
                );
                return false;
            }
        };
        invariants::outcome(&before.state, &command, &result);
        match &result {
            Ok(()) => invariants::transition(&before.state, &command, &self.game.state),
            Err(_) => props::always!(
                self.game == before,
                "game: a rejected command leaves the whole game unchanged",
                json!({ "command": command })
            ),
        }
        invariants::state(&self.game.state);
        coverage::observe(&self.game.state);
        if e.chance(1, 50) {
            self.snapshot_round_trip();
            if let (
                Ok(()),
                Command::Tick {
                    count,
                    timestamp_millis: None,
                },
            ) = (&result, &command)
            {
                if (2..=BATCH_SAMPLE_TICKS).contains(count) {
                    self.batch_matches_single_ticks(&before, *count);
                }
            }
        }
        self.trace.push((result.is_ok(), digest(&self.game.state)));
        true
    }

    /// A batched Tick runs the same system iterations as that many single
    /// Ticks. Only command-level publication differs: the batch carries every
    /// iteration's sound and music events, and a screen change resets the
    /// menu animation counter at the command boundary.
    fn batch_matches_single_ticks(&self, before: &Game, count: u32) {
        let mut singles = before.clone();
        let mut sounds = Vec::new();
        let mut music = Vec::new();
        let mut screen_changed = false;
        let mut failed_at = None;
        for index in 0..count {
            let screen = singles.state.screen;
            let single = Command::Tick {
                count: 1,
                timestamp_millis: None,
            };
            if !matches!(command_engine(&mut singles, single), Ok(Ok(()))) {
                failed_at = Some(index);
                break;
            }
            screen_changed |= singles.state.screen != screen;
            sounds.extend(singles.state.audio.events.iter().cloned());
            music.extend(singles.state.music.events.iter().cloned());
        }
        singles.state.audio.events = sounds;
        singles.state.music.events = music;
        if screen_changed {
            singles.state.menu_animation_tick = self.game.state.menu_animation_tick;
        }
        let equal = failed_at.is_none() && singles == self.game;
        props::always!(
            equal,
            "game: a batched Tick simulates exactly what single Ticks do",
            json!({
                "count": count,
                "singleFailedAt": failed_at,
                "difference": (!equal && failed_at.is_none()).then(|| {
                    let batch = serde_json::to_value(&self.game).unwrap_or_default();
                    let single = serde_json::to_value(&singles).unwrap_or_default();
                    json_diff::first(&batch, &single).map(|(path, batch, single)| {
                        json!({ "path": path, "batch": batch, "single": single })
                    })
                }),
            })
        );
    }

    /// The browser reads states only through JSON; it must be lossless and
    /// pass the engine's own deserialization validators.
    fn snapshot_round_trip(&self) {
        let text = self.game.json();
        let restored = serde_json::from_str::<State>(&text);
        let republished = restored
            .as_ref()
            .ok()
            .and_then(|state| serde_json::to_string(state).ok());
        let lossless = restored.as_ref().ok() == Some(&self.game.state)
            && republished.as_deref() == Some(text.as_str());
        props::always!(
            lossless,
            "game: published states round-trip losslessly through their validators",
            json!({
                "error": restored.as_ref().err().map(|error| error.to_string()),
                "difference": (!lossless).then(|| round_trip_difference(
                    &text,
                    republished.as_deref(),
                    &self.game.state,
                    restored.as_ref().ok(),
                )),
            })
        );
        let game_text = serde_json::to_string(&self.game).expect("game serializes");
        let restored = serde_json::from_str::<Game>(&game_text);
        let lossless = restored.as_ref().ok() == Some(&self.game);
        let reserialized = restored
            .as_ref()
            .ok()
            .and_then(|game| serde_json::to_string(game).ok());
        props::always!(
            lossless,
            "game: complete games round-trip losslessly through their validators",
            json!({
                "error": restored.as_ref().err().map(|error| error.to_string()),
                "difference": (!lossless).then(|| round_trip_difference(
                    &game_text,
                    reserialized.as_deref(),
                    &self.game,
                    restored.as_ref().ok(),
                )),
            })
        );
    }

    fn finish(&mut self, case: &mut Case) {
        coverage::end_case(&case.label);
        self.snapshot_round_trip();
        self.replay_matches();
        self.continuation_matches(case.entropy);
        self.abi_matches();
        if let Some(module) = case.wasm {
            module.differential(&self.tape, &self.trace);
        }
        if self.game.state.screen == rebirth_cleanroom::Screen::Playing {
            props::sometimes!(
                self.game.state.run_tick > 3_000,
                "game: a run lasts more than 3,000 ticks"
            );
        }
    }

    /// The same tape on a fresh boot reproduces every outcome and state.
    fn replay_matches(&self) {
        let Ok(mut game) =
            Game::new_with_boot_epoch(self.tape.initial_seed, self.tape.boot_epoch_seed)
        else {
            return;
        };
        let mut divergence = None;
        for (index, command) in self.tape.commands.iter().enumerate() {
            let outcome = matches!(command_engine(&mut game, command.clone()), Ok(Ok(())));
            if (outcome, digest(&game.state)) != self.trace[index] {
                divergence = Some(index);
                break;
            }
        }
        let equal = divergence.is_none() && game == self.game;
        props::always!(
            equal,
            "game: replaying a tape reproduces every outcome and state",
            json!({ "firstDivergence": divergence, "commands": self.tape.commands.len() })
        );
    }

    /// Resuming from a serialized midpoint reaches the same final game, so
    /// no simulation state lives outside the serialized game.
    fn continuation_matches(&self, e: &mut Entropy) {
        let commands = &self.tape.commands;
        let Ok(mut game) =
            Game::new_with_boot_epoch(self.tape.initial_seed, self.tape.boot_epoch_seed)
        else {
            return;
        };
        let midpoint = if commands.is_empty() {
            0
        } else {
            e.index(commands.len() + 1)
        };
        for command in &commands[..midpoint] {
            let _ = command_engine(&mut game, command.clone());
        }
        let saved = serde_json::to_string(&game).expect("game serializes");
        let Ok(mut resumed) = serde_json::from_str::<Game>(&saved) else {
            props::always!(
                false,
                "game: a game resumed from its serialized midpoint reaches the same end",
                json!({ "midpoint": midpoint, "error": "midpoint does not deserialize" })
            );
            return;
        };
        for command in &commands[midpoint..] {
            let _ = command_engine(&mut resumed, command.clone());
        }
        let equal = resumed == self.game;
        props::always!(
            equal,
            "game: a game resumed from its serialized midpoint reaches the same end",
            json!({ "midpoint": midpoint, "commands": commands.len() })
        );
    }

    /// The exported C ABI (what the WASM bridge calls) agrees with direct
    /// dispatch on every outcome and on the final published snapshot.
    fn abi_matches(&self) {
        use rebirth_cleanroom as abi;
        if abi::reset(self.tape.initial_seed, self.tape.boot_epoch_seed) != 0 {
            props::always!(false, "game: the C ABI agrees with direct dispatch");
            return;
        }
        let mut divergence = None;
        for (index, command) in self.tape.commands.iter().enumerate() {
            let bytes = serde_json::to_vec(command).expect("command serializes");
            let pointer = abi::request_ptr(bytes.len());
            if pointer.is_null() {
                divergence = Some((index, "request allocation"));
                break;
            }
            // SAFETY: request_ptr returned a buffer of exactly bytes.len().
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
            let failed = abi::command() != 0;
            let error_published = abi::error_len() > 0;
            if failed == self.trace[index].0 || failed != error_published {
                divergence = Some((index, "outcome"));
                break;
            }
        }
        let pointer = abi::snapshot_ptr();
        // SAFETY: snapshot_ptr/snapshot_len describe the thread-local buffer.
        let snapshot = unsafe { std::slice::from_raw_parts(pointer, abi::snapshot_len()) };
        let equal = divergence.is_none() && snapshot == self.game.json().as_bytes();
        props::always!(
            equal,
            "game: the C ABI agrees with direct dispatch",
            json!({ "divergence": divergence })
        );
    }
}
