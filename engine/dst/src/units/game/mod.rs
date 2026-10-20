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
pub(crate) mod saves;
mod scout;

pub(crate) use driver::Driver;
pub(crate) use invariants::{expected_rejection, state as state_invariants};
pub(crate) use scout::{seeded_run, seeded_run_with};

use crate::coverage;
use crate::entropy::Entropy;
use crate::harness::{self, Case};
use crate::json_diff;
use crate::props;
use crate::tape::{BootElapsed, Reset, Tape};
use rebirth_cleanroom::audio::{SoundEvent, VolumeCallback};
use rebirth_cleanroom::controls_menu;
use rebirth_cleanroom::display_settings::GammaPreferences;
use rebirth_cleanroom::fullscreen::{self, FullscreenPreferences};
use rebirth_cleanroom::input;
use rebirth_cleanroom::keyboard::{
    DeviceConfig, InputConfigs, Keyboard, CONFIGURABLE, DEFAULT_PAIRS,
};
use rebirth_cleanroom::map_presentation::MapPreferences;
use rebirth_cleanroom::music::MusicEvent;
use rebirth_cleanroom::options_menu::{AudioPreferences, SaveRequest, UnavailableRequest};
use rebirth_cleanroom::save_host::SaveStore;
use rebirth_cleanroom::{Command, Game, Screen, State};
use serde::Serialize;
use serde_json::json;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::panic::{self, AssertUnwindSafe};

pub fn run(case: &mut Case) {
    let replay = case.replay.take();
    let replaying = replay.is_some();
    // `--resume`: replay a prefix (a boss room entered, say), then branch.
    let resume = case.resume.take();
    let prefix = resume.as_ref().map_or(&[][..], |tape| &tape.commands[..]);
    // The document a generated case's previous session stored, if any.
    let mut previous = None;
    let tape = match (replay, &resume) {
        (Some(tape), _) => tape,
        (None, Some(resume)) => resume.start(),
        (None, None) => {
            let e = &mut *case.entropy;
            let initial_seed = e.nonzero_u32();
            let boot_epoch_seed = match e.below(4) {
                0 => 0,
                1 => 1_700_000_000 + e.range(0, 200_000_000),
                _ => e.u32(),
            };
            let mut tape = Tape::new(initial_seed, boot_epoch_seed);
            // The browser boots with the settings its last Options save stored.
            if e.chance(1, 2) {
                tape.boot_audio_preferences = Some(AudioPreferences {
                    sfx_volume: stored_volume(e),
                    music_volume: stored_volume(e),
                });
            }
            if e.chance(1, 3) {
                tape.boot_fullscreen_preferences = Some(FullscreenPreferences {
                    fullscreen: e.chance(1, 2),
                });
            }
            if e.chance(1, 3) {
                tape.boot_input_configs = Some(stored_input_configs(e));
            }
            if e.chance(1, 3) {
                tape.boot_map_preferences = Some(MapPreferences {
                    opacity: stored_volume(e),
                });
            }
            if e.chance(1, 3) {
                tape.boot_gamma_preferences = Some(GammaPreferences {
                    gamma: stored_gamma(e),
                });
            }
            // The browser's own-save storage, which a menu readmission reads:
            // sometimes what a previous session stored, as after a reload,
            // and sometimes what one stored on a floor it descended to.
            if e.chance(1, 2) {
                let mut host = saves::generated_host(e);
                match e.below(3) {
                    0 => {
                        let steps = e.range(150, 1_500);
                        previous = saves::previous_session(e, steps);
                        host.stored.clone_from(&previous);
                    }
                    1 => {
                        previous = saves::previous_descent(e);
                        host.stored.clone_from(&previous);
                    }
                    _ => {}
                }
                tape.save_host = Some(host);
            }
            // A controlled harness declares how long the original ran before
            // its reset (#660); the shipped game never does.
            if e.chance(1, 4) {
                tape.boot_elapsed = Some(declared_boot_elapsed(e));
            }
            tape
        }
    };
    let violations = props::violations();
    let Some(mut simulation) = Simulation::boot(&tape) else {
        return;
    };
    // A tape's stored document is a previous session's too, unless it is
    // one of the corrupt documents the generator stores (which never load).
    let stored = tape
        .save_host
        .as_ref()
        .and_then(|host| host.stored.as_deref());
    if let Some(document) = previous.as_deref().or(stored) {
        saves::reload(&tape, document, previous.is_some());
    }
    let mut driver = driver::Driver::new(case.entropy);
    if !replaying {
        // Every generated case scouts, so the scout's own must-hits never hang
        // on how many cases drew the seeded persona; only that persona types
        // the codes.
        let wanted = if driver.seeded() { 3 } else { 1 };
        let codes = scout::codes(&simulation.game, case.entropy, wanted);
        if driver.seeded() {
            driver.scout(codes);
        }
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
        if !simulation.step(command) {
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

/// Batched Ticks up to this length are re-run as single Ticks when sampled
/// and whenever they change screens or wait on save IO.
const BATCH_SAMPLE_TICKS: u32 = 32;

struct Simulation {
    game: Game,
    /// The save host's storage, with the tape's save host.
    store: Option<SaveStore>,
    /// The profiles the session booted with: a previous session's document.
    imported: Option<std::sync::Arc<rebirth_cleanroom::save_profiles::Profiles>>,
    tape: Tape,
    /// Ok/Err outcome and state digest after each command.
    trace: Vec<(bool, u64)>,
    /// The full update at which the player lost controls outside any hold
    /// the state contract names.
    uncontrolled_since: Option<u32>,
}

/// Full updates a living player may stay without controls outside the holds
/// the state contract names (a hold the port has yet to name, say) before
/// the controls count as lost for good.
const CONTROLS_RETURN: u32 = 600;

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

/// What commands publish for the host, in order (`State::begin_command`
/// clears it as each command starts).
#[derive(Default, PartialEq, Serialize)]
struct Published {
    sounds: Vec<SoundEvent>,
    music: Vec<MusicEvent>,
    volume_callbacks: Vec<VolumeCallback>,
    saves: Vec<SaveRequest>,
    unavailable: Vec<UnavailableRequest>,
    fullscreen: Vec<fullscreen::Request>,
    input_saves: Vec<controls_menu::SaveRequest>,
    unavailable_actions: Vec<usize>,
}

impl Published {
    fn extend(&mut self, state: &State) {
        self.sounds.extend(state.audio.events.iter().cloned());
        self.music.extend(state.music.events.iter().cloned());
        self.volume_callbacks
            .extend(state.audio.volume_callbacks.iter().cloned());
        self.saves
            .extend(state.options_menu.save_requests.iter().cloned());
        self.unavailable
            .extend(state.options_menu.unavailable_requests.iter().cloned());
        self.fullscreen
            .extend(state.fullscreen.requests.iter().cloned());
        let controls = &state.controls_menu;
        self.input_saves
            .extend(controls.save_requests.iter().cloned());
        self.unavailable_actions
            .extend(controls.unavailable_actions.iter().copied());
    }
}

/// An input configuration file as the browser stores it: devices retained
/// from earlier sessions, with whatever bindings they held, and usually the
/// Keyboard's: the defaults, second keys no default uses, or every
/// configurable action rebound to a distinct key.
pub(crate) fn stored_input_configs(e: &mut Entropy) -> InputConfigs {
    let mut devices: Vec<DeviceConfig> = (0..e.below(3))
        .map(|index| DeviceConfig {
            name: format!("Controller {index}"),
            bindings: std::array::from_fn(|_| {
                std::array::from_fn(|_| match e.below(8) {
                    0 => e.u32() as i32,
                    _ => e.range(0, 349) as i32 - 1,
                })
            }),
        })
        .collect();
    if e.chance(3, 4) {
        let mut keyboard = Keyboard::default();
        let defaults: Vec<i32> = DEFAULT_PAIRS.iter().map(|(_, key)| *key).collect();
        let mut free: Vec<i32> = input::PUBLIC_KEYS
            .iter()
            .filter_map(|code| input::glfw_key(code))
            .filter(|key| !defaults.contains(key))
            .collect();
        if e.chance(1, 3) {
            let mut keys: Vec<i32> = CONFIGURABLE
                .iter()
                .map(|action| keyboard.bindings[*action][0])
                .chain(free.iter().copied().take(CONFIGURABLE.len()))
                .collect();
            for action in CONFIGURABLE {
                keyboard.bindings[action] = [keys.swap_remove(e.index(keys.len())), -1];
            }
        } else {
            for action in CONFIGURABLE {
                if !free.is_empty() && e.chance(1, 4) {
                    let key = free.swap_remove(e.index(free.len()));
                    keyboard.bindings[action][1] = key;
                }
            }
        }
        let at = e.index(devices.len() + 1);
        devices.insert(
            at,
            DeviceConfig {
                name: "Keyboard".into(),
                bindings: keyboard.bindings,
            },
        );
    }
    InputConfigs { devices }
}

/// A boot clock a harness declares: the live suites' Native intro (30
/// bootstrap, 180 intro and 9 Title presentations: 3,850 ms + 77,000 ns),
/// none, any day's worth, or the widest the ingress takes.
fn declared_boot_elapsed(e: &mut Entropy) -> BootElapsed {
    let (milliseconds, sub_millisecond_nanoseconds) = match e.below(4) {
        0 => (3_850, 77_000),
        1 => (0, 0),
        2 => (e.range(0, 86_400_000), e.range(0, 1_000_000)),
        _ => (u32::MAX, 999_999),
    };
    BootElapsed {
        milliseconds,
        sub_millisecond_nanoseconds,
    }
}

/// Whether `Game::set_boot_elapsed` refuses `declared` on `game`, leaving it
/// unchanged.
fn boot_clock_refuses(game: &Game, declared: (u32, u32)) -> bool {
    let mut probe = game.clone();
    probe.set_boot_elapsed(declared.0, declared.1).is_err() && probe == *game
}

/// A volume as the browser stores it: an Options step, an endpoint, or any
/// four-decimal value within the unit interval.
fn stored_volume(e: &mut Entropy) -> f32 {
    match e.below(4) {
        0 => *e.pick(&[0.0, 1.0]),
        1 => e.below(11) as f32 / 10.0,
        _ => e.below(10_001) as f32 / 10_000.0,
    }
}

/// A Gamma an Options save stored (`%.4f` within .5..1.5): a bound, a
/// whole number of Options steps from 1, or any four-decimal value.
fn stored_gamma(e: &mut Entropy) -> f32 {
    match e.below(4) {
        0 => *e.pick(&[0.5, 1.0, 1.5]),
        1 => (50 + e.below(101)) as f32 / 100.0,
        _ => (5_000 + e.below(10_001)) as f32 / 10_000.0,
    }
}

fn command_engine(game: &mut Game, command: Command) -> Result<Result<(), String>, String> {
    panic::catch_unwind(AssertUnwindSafe(|| game.command(command)))
        .map_err(|_| harness::take_panic())
}

impl Simulation {
    fn boot(tape: &Tape) -> Option<Self> {
        // The census case opens before boot, so it records the properties
        // boot meets.
        coverage::begin_case();
        let (initial_seed, boot_epoch_seed) = (tape.initial_seed, tape.boot_epoch_seed);
        let booted = panic::catch_unwind(|| tape.session());
        let details = json!({ "initialSeed": initial_seed, "bootEpochSeed": boot_epoch_seed });
        match booted {
            Ok(Ok((game, store))) => {
                props::always!(true, "game: every nonzero seed and epoch boots to Title");
                props::always!(
                    game.state.screen == rebirth_cleanroom::Screen::Title
                        && game.state.initial_seed == initial_seed
                        && game.state.boot_epoch_seed == boot_epoch_seed,
                    "game: boot publishes the first Title with its seeds",
                    details
                );
                // Settings5B9930 applies the stored settings to their owners
                // before MenuInit's first Music.
                // The Keyboard owner starts from the stored Keyboard device's
                // bindings, or the defaults, and every device is retained.
                let state = &game.state;
                let configs = tape.boot_input_configs.clone().unwrap_or_default();
                props::always!(
                    tape.boot_audio_preferences.is_none_or(|audio| {
                        state.audio.sfx_volume == audio.sfx_volume
                            && state.music.volume == audio.music_volume
                    }) && tape
                        .boot_fullscreen_preferences
                        .is_none_or(|display| state.fullscreen.requested == display.fullscreen)
                        && state.keyboard == configs.keyboard()
                        && state.input_configs == configs
                        && state.map_settings == tape.boot_map_preferences.unwrap_or_default()
                        && state.gamma_settings == tape.boot_gamma_preferences.unwrap_or_default(),
                    "game: boot starts with the stored settings",
                    json!({
                        "audio": tape.boot_audio_preferences,
                        "fullscreen": tape.boot_fullscreen_preferences,
                        "inputConfigs": tape.boot_input_configs,
                        "map": [tape.boot_map_preferences, state.map_settings],
                        "gamma": [tape.boot_gamma_preferences, state.gamma_settings],
                        "sfx": state.audio.sfx_volume,
                        "music": state.music.volume,
                        "requested": state.fullscreen.requested,
                        "keyboard": state.keyboard,
                    })
                );
                props::sometimes!(
                    tape.boot_audio_preferences.is_some()
                        && tape.boot_fullscreen_preferences.is_some()
                        && tape.boot_input_configs.is_some(),
                    "game: a game boots with stored audio, fullscreen and input settings"
                );
                // #660's harness ingress: a fresh reset clock takes exactly the
                // declared time, and refuses a sub-millisecond part of 1 ms or
                // more, or a second declaration once it moved, unchanged.
                let clock = &state.run_admission.clock;
                props::always!(
                    tape.boot_elapsed.is_none_or(|elapsed| {
                        clock.milliseconds
                            == i64::from(boot_epoch_seed) * 1000 + i64::from(elapsed.milliseconds)
                            && clock.sub_millisecond_nanoseconds
                                == elapsed.sub_millisecond_nanoseconds
                    }),
                    "game: a declared boot elapsed time advances the fresh reset clock by exactly that time",
                    json!({ "declared": tape.boot_elapsed, "clock": clock })
                );
                props::sometimes!(
                    tape.boot_elapsed.is_some(),
                    "game: a game boots with a declared boot elapsed time"
                );
                let moved = tape.boot_elapsed.is_some_and(|elapsed| {
                    elapsed.milliseconds != 0 || elapsed.sub_millisecond_nanoseconds != 0
                });
                if moved {
                    props::always!(
                        boot_clock_refuses(&game, (0, 0)),
                        "game: a boot elapsed declaration is refused once the reset clock moved",
                        json!({ "declared": tape.boot_elapsed })
                    );
                } else {
                    // Any sub-millisecond part from 1 ms up to u32::MAX.
                    let invalid = 1_000_000 + initial_seed % (u32::MAX - 999_999);
                    props::always!(
                        boot_clock_refuses(&game, (boot_epoch_seed, invalid)),
                        "game: a boot elapsed time with a sub-millisecond part of 1 ms or more is refused",
                        json!({ "declared": [boot_epoch_seed, invalid] })
                    );
                }
                let again = tape.session();
                props::always!(
                    again.as_ref().ok() == Some(&(game.clone(), store.clone())),
                    "game: boot is a pure function of its seeds and stored settings"
                );
                invariants::state(&game.state);
                coverage::observe(&game.state);
                Some(Self {
                    imported: game.saved_profiles.clone(),
                    game,
                    store,
                    tape: tape.start(),
                    trace: Vec::new(),
                    uncontrolled_since: None,
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

    /// Controls a living player lost outside every named hold come back:
    /// a stuck flag freezes the player for good.
    fn controls_return(&mut self) {
        let state = &self.game.state;
        let lost = state.screen == Screen::Playing
            && !state.player.controls_enabled
            && !invariants::controls_hold(state)
            && !state.player.dead
            && state.player_death.game_over.is_none();
        let now = state.game_update_count;
        let since = if lost {
            *self.uncontrolled_since.get_or_insert(now)
        } else {
            self.uncontrolled_since = None;
            now
        };
        props::always!(
            now.saturating_sub(since) <= CONTROLS_RETURN,
            "game: controls lost outside a named hold come back within 600 full updates",
            json!({ "since": since, "now": now })
        );
    }

    /// Applies one command with every per-step check; false ends the case.
    /// The draws of the checks a command samples. They come from the state
    /// before the command and its index, not from the case's entropy, so a
    /// retained tape replays exactly the checks its case ran.
    fn checks(&self) -> Entropy {
        let before = self
            .trace
            .last()
            .map_or_else(|| digest(&self.game.state), |&(_, digest)| digest);
        let index = self.tape.commands.len() as u64;
        Entropy::seeded(before ^ index.wrapping_mul(0x9e37_79b9_7f4a_7c15))
    }

    fn step(&mut self, command: Command) -> bool {
        let e = &mut self.checks();
        let before = self.game.clone();
        let before_store = self.store.clone();
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
        if before.state.tick == 0 && self.game.state.tick > 0 {
            props::always!(
                boot_clock_refuses(&self.game, (0, 0)),
                "game: a boot elapsed declaration is refused once a Tick ran",
                json!({ "command": command, "tick": self.game.state.tick })
            );
        }
        // The browser answers the save IO a command leaves before any later
        // input sees the state, so the contracts below see both.
        let host = self.tape.save_host.as_ref();
        let pending =
            self.game.state.save_write_pending || self.game.state.save_readmission_pending;
        if host.is_some() && (pending || e.chance(1, 8)) {
            saves::stale_acknowledgement(&self.game, e);
        }
        let served = match &result {
            Ok(()) => saves::serve(&mut self.store, host, &mut self.game, &command),
            Err(_) => Ok(Vec::new()),
        };
        saves::check(
            &before,
            &command,
            &served,
            &self.game,
            self.imported.as_deref(),
        );
        if host.is_some() && e.chance(1, 40) {
            saves::round_trip(&self.tape, &self.game);
        }
        invariants::outcome(&before.state, &command, &result, &self.game.state);
        match &result {
            Ok(()) => invariants::transition(&before.state, &command, &self.game.state),
            Err(_) => props::always!(
                self.game == before,
                "game: a rejected command leaves the whole game unchanged",
                json!({ "command": command })
            ),
        }
        invariants::state(&self.game.state);
        self.controls_return();
        coverage::observe(&self.game.state);
        let sampled = e.chance(1, 50);
        if sampled {
            self.snapshot_round_trip();
        }
        // A batch that changes screens or waits on save IO crosses what a
        // command boundary resets (a menu reconstruction, a System iteration
        // the host resumes), so it is always re-run as single Ticks.
        let crosses = before.state.screen != self.game.state.screen
            || served.as_ref().is_ok_and(|responses| !responses.is_empty());
        if let (
            Ok(()),
            Command::Tick {
                count,
                timestamp_millis: None,
            },
        ) = (&result, &command)
        {
            if (sampled || crosses) && (2..=BATCH_SAMPLE_TICKS).contains(count) {
                self.batch_matches_single_ticks(&before, before_store, *count);
            }
        }
        self.trace.push((result.is_ok(), digest(&self.game.state)));
        true
    }

    /// A batched Tick runs the same system iterations as that many single
    /// Ticks. Only command-level publication differs: the batch publishes
    /// every iteration's outputs (`State::begin_command`) in order, and a
    /// screen change resets the menu animation counter at the command
    /// boundary.
    fn batch_matches_single_ticks(&self, before: &Game, mut store: Option<SaveStore>, count: u32) {
        let mut singles = before.clone();
        let host = self.tape.save_host.as_ref();
        let mut published = Published::default();
        let mut screen_changed = false;
        let mut failed_at = None;
        for index in 0..count {
            let screen = singles.state.screen;
            let single = Command::Tick {
                count: 1,
                timestamp_millis: None,
            };
            if !matches!(command_engine(&mut singles, single.clone()), Ok(Ok(())))
                || saves::serve(&mut store, host, &mut singles, &single).is_err()
            {
                failed_at = Some(index);
                break;
            }
            screen_changed |= singles.state.screen != screen;
            published.extend(&singles.state);
        }
        let mut batched = Published::default();
        batched.extend(&self.game.state);
        let mut batch = self.game.clone();
        batch.state.begin_command();
        singles.state.begin_command();
        if screen_changed {
            singles.state.menu_animation_tick = batch.state.menu_animation_tick;
        }
        let equal = failed_at.is_none() && singles == batch && published == batched;
        props::always!(
            equal,
            "game: a batched Tick simulates exactly what single Ticks do",
            json!({
                "count": count,
                "singleFailedAt": failed_at,
                "difference": (!equal && failed_at.is_none()).then(|| {
                    let (batch, single) = if singles == batch {
                        (json!(batched), json!(published))
                    } else {
                        (json!(batch), json!(singles))
                    };
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
        self.continuation_matches(&mut self.checks());
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
        let Ok((mut game, mut store)) = self.tape.session() else {
            return;
        };
        let host = self.tape.save_host.as_ref();
        let mut divergence = None;
        for (index, command) in self.tape.commands.iter().enumerate() {
            let outcome = matches!(command_engine(&mut game, command.clone()), Ok(Ok(())));
            if outcome {
                let _ = saves::serve(&mut store, host, &mut game, command);
            }
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
        let Ok((mut game, mut store)) = self.tape.session() else {
            return;
        };
        let host = self.tape.save_host.as_ref();
        let midpoint = if commands.is_empty() {
            0
        } else {
            e.index(commands.len() + 1)
        };
        for command in &commands[..midpoint] {
            if let Ok(Ok(())) = command_engine(&mut game, command.clone()) {
                let _ = saves::serve(&mut store, host, &mut game, command);
            }
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
            if let Ok(Ok(())) = command_engine(&mut resumed, command.clone()) {
                let _ = saves::serve(&mut store, host, &mut resumed, command);
            }
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
        let (seed, epoch) = (self.tape.initial_seed, self.tape.boot_epoch_seed);
        let reset = match self.tape.reset() {
            Reset::Plain => abi::reset(seed, epoch),
            Reset::Audio {
                sfx_volume,
                music_volume,
            } => abi::reset_with_audio_preferences(seed, epoch, sfx_volume, music_volume),
            Reset::Display {
                fullscreen,
                has_audio,
                sfx_volume,
                music_volume,
            } => abi::reset_with_fullscreen_preferences(
                seed,
                epoch,
                fullscreen,
                has_audio,
                sfx_volume,
                music_volume,
            ),
            Reset::Map {
                has_fullscreen,
                fullscreen,
                has_audio,
                sfx_volume,
                music_volume,
                opacity,
            } => abi::reset_with_map_preferences_v2(
                seed,
                epoch,
                has_fullscreen,
                fullscreen,
                has_audio,
                sfx_volume,
                music_volume,
                opacity,
            ),
            Reset::InputConfigs { request } => {
                if !abi_request(&request) {
                    props::always!(false, "game: the C ABI agrees with direct dispatch");
                    return;
                }
                abi::reset_with_input_configs(seed, epoch)
            }
            Reset::Settings { request } => {
                if !abi_request(&request) {
                    props::always!(false, "game: the C ABI agrees with direct dispatch");
                    return;
                }
                abi::reset_with_settings_preferences(seed, epoch)
            }
        };
        // web/engine.ts declares a harness's boot clock inside reset.
        let declared = self.tape.boot_elapsed.is_none_or(|elapsed| {
            abi::set_boot_elapsed(elapsed.milliseconds, elapsed.sub_millisecond_nanoseconds) == 0
        });
        if reset != 0 || !declared {
            props::always!(false, "game: the C ABI agrees with direct dispatch");
            return;
        }
        let host = self.tape.save_host.as_ref();
        let mut store = match host.map(|host| host.boot(&mut saves::AbiSaves)).transpose() {
            Ok(store) => store,
            Err(error) => {
                props::always!(
                    false,
                    "game: the C ABI agrees with direct dispatch",
                    json!({ "saveHost": error })
                );
                return;
            }
        };
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
            if !failed && saves::serve(&mut store, host, &mut saves::AbiSaves, command).is_err() {
                divergence = Some((index, "save host"));
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

/// Copies a reset request into the C ABI's request buffer.
fn abi_request(request: &[u8]) -> bool {
    let pointer = rebirth_cleanroom::request_ptr(request.len());
    if pointer.is_null() {
        return false;
    }
    // SAFETY: request_ptr returned a buffer of exactly request.len().
    unsafe { std::ptr::copy_nonoverlapping(request.as_ptr(), pointer, request.len()) };
    true
}
