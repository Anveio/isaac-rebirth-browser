//! The exported C ABI transport (`engine/src/lib.rs`, consumed by
//! `web/engine.ts`): request framing, malformed requests, failed resets, the
//! per-thread engine slot, the settings resets the browser boots with and the
//! authored-resource boot the e2e harness uses. A resource boot with the
//! authored inputs must behave exactly like the plain boot, or a suite booted
//! with resources would not speak for the browser.
use crate::entropy::Entropy;
use crate::harness::Case;
use crate::props;
use crate::units::game::stored_input_configs;
use rebirth_cleanroom as abi;
use rebirth_cleanroom::fullscreen::FullscreenPreferences;
use rebirth_cleanroom::keyboard::{InputConfigs, ACTIONS};
use rebirth_cleanroom::map_presentation::MapPreferences;
use rebirth_cleanroom::options_menu::AudioPreferences;
use rebirth_cleanroom::{Command, State, ABI_VERSION};
use serde_json::{json, Value};

/// web/engine.ts refuses larger requests before calling `request_ptr`.
const MAX_REQUEST: usize = 65_536;
/// web/engine.ts refuses larger boot resources before `boot_resource_ptr`.
const MAX_BOOT_RESOURCE: usize = 512 * 1024;

/// The authored resources the engine embeds, as the harness supplies them.
const CHALLENGES_XML: &str =
    include_str!("../../../../references/isaac/rebirth/resources/challenges.xml");
const SPECIAL_ROOMS: &[u8] =
    include_bytes!("../../../../references/isaac/rebirth/resources/rooms/00.special rooms.stb");

fn send(bytes: &[u8]) -> Option<u32> {
    let pointer = abi::request_ptr(bytes.len());
    if pointer.is_null() {
        return None;
    }
    // SAFETY: request_ptr returned a writable buffer of exactly bytes.len().
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
    Some(abi::command())
}

/// Fills the boot resource buffer, then runs `reset` from it.
fn load(resources: &[u8], reset: impl FnOnce() -> u32) -> Option<u32> {
    let pointer = abi::boot_resource_ptr(resources.len());
    if pointer.is_null() {
        return None;
    }
    // SAFETY: boot_resource_ptr returned a writable buffer of exactly resources.len().
    unsafe { std::ptr::copy_nonoverlapping(resources.as_ptr(), pointer, resources.len()) };
    Some(reset())
}

fn boot_with(resources: &[u8], seed: u32, epoch: u32) -> Option<u32> {
    load(resources, || abi::reset_with_boot_resources(seed, epoch))
}

fn snapshot() -> Vec<u8> {
    let pointer = abi::snapshot_ptr();
    // SAFETY: snapshot_ptr/snapshot_len describe the thread-local buffer.
    unsafe { std::slice::from_raw_parts(pointer, abi::snapshot_len()) }.to_vec()
}

fn public_keys() -> Vec<u8> {
    // SAFETY: public_keys_ptr/public_keys_len describe the thread-local buffer.
    unsafe { std::slice::from_raw_parts(abi::public_keys_ptr(), abi::public_keys_len()) }.to_vec()
}

fn error() -> String {
    // SAFETY: error_ptr/error_len describe the thread-local buffer.
    let bytes = unsafe { std::slice::from_raw_parts(abi::error_ptr(), abi::error_len()) };
    String::from_utf8_lossy(bytes).into_owned()
}

/// Requests the bridge can deliver but the engine must reject.
fn malformed(e: &mut Entropy) -> Vec<u8> {
    match e.below(8) {
        0 => br#"{"type":"tick","count":1,"extra":true}"#.to_vec(),
        1 => br#"{"type":"tick","count":-1}"#.to_vec(),
        2 => br#"{"type":"audioVolume","sfx_volume":1e999}"#.to_vec(),
        3 => br#"{"type":"teleport"}"#.to_vec(),
        4 => br#"{"type":"key","code":"Enter"}"#.to_vec(),
        5 => {
            let valid = serde_json::to_vec(&Command::Tick {
                count: 1,
                timestamp_millis: None,
            })
            .expect("command");
            valid[..e.index(valid.len())].to_vec()
        }
        6 => {
            // Arbitrary bytes can encode a valid command. Keep all drawn
            // bytes, but prefix invalid UTF-8 so this arm is truly malformed.
            let mut request = vec![0xff];
            request.extend((0..e.range(1, 64)).map(|_| e.u32() as u8));
            request
        }
        _ => vec![0xff, 0xfe, b'{'],
    }
}

pub fn run(case: &mut Case) {
    props::always!(
        abi::abi_version() == ABI_VERSION,
        "abi: the exported ABI version is current"
    );

    // A fresh thread has no engine: commands fail and snapshots publish null,
    // while the key surface the browser reads before reset is available.
    let unset = std::thread::spawn(|| {
        let result = send(br#"{"type":"tick","count":1}"#);
        (result, error(), snapshot(), public_keys())
    })
    .join();
    props::always!(
        matches!(&unset, Ok((Some(1), message, snapshot, _)) if !message.is_empty() && snapshot == b"null"),
        "abi: commands before reset fail and snapshots publish null"
    );
    let published = public_keys();
    let decoded: Result<Vec<String>, _> = serde_json::from_slice(&published);
    props::always!(
        decoded
            .as_ref()
            .is_ok_and(|keys| keys.iter().map(String::as_str).eq(abi::input::PUBLIC_KEYS))
            && matches!(&unset, Ok((.., before_reset)) if *before_reset == published),
        "abi: the public_keys export publishes exactly the engine's key surface",
        json!({ "published": String::from_utf8_lossy(&published) })
    );

    let e = &mut *case.entropy;
    let seed = e.nonzero_u32();
    props::always!(
        abi::reset(seed, e.u32()) == 0 && abi::error_len() == 0,
        "abi: reset with a nonzero seed succeeds and clears the error",
        json!({ "seed": seed })
    );
    let booted = snapshot();
    props::always!(
        serde_json::from_slice::<State>(&booted).is_ok_and(|state| state.initial_seed == seed),
        "abi: the published snapshot is a valid state of the reset seed",
        json!({ "seed": seed })
    );
    props::always!(
        abi::reset(0, e.u32()) == 1 && !error().is_empty() && snapshot() == booted,
        "abi: a failed reset reports an error and keeps the previous game"
    );
    props::always!(
        abi::request_ptr(MAX_REQUEST + 1 + e.index(1024)).is_null()
            && !abi::request_ptr(MAX_REQUEST).is_null(),
        "abi: request buffers are bounded at 64 KiB"
    );

    let mut current = booted;
    for _ in 0..(case.steps / 16).max(32) {
        if e.chance(1, 2) {
            let request = malformed(e);
            let result = send(&request);
            props::always!(
                result == Some(1) && !error().is_empty() && snapshot() == current,
                "abi: malformed requests fail with an error and leave the snapshot unchanged",
                json!({ "request": String::from_utf8_lossy(&request) })
            );
        } else {
            let command = match e.below(4) {
                3 => Command::Tick {
                    count: rebirth_cleanroom::MAX_COMMAND_TICKS + 1,
                    timestamp_millis: None,
                },
                0 => Command::Tick {
                    count: e.range(0, 30),
                    timestamp_millis: None,
                },
                1 => Command::Key {
                    code: (*e.pick(&["Enter", "Escape", "ArrowLeft", "KeyD"])).into(),
                    down: e.chance(1, 2),
                },
                _ => Command::AudioVolume {
                    sfx_volume: e.range(0, 100) as f32 / 100.0,
                },
            };
            let bytes = serde_json::to_vec(&command).expect("command serializes");
            let result = send(&bytes);
            let accepted = result == Some(0);
            props::always!(
                result.is_some() && accepted == (abi::error_len() == 0),
                "abi: the return code and error buffer agree",
                json!({ "command": command, "result": result, "error": error() })
            );
            let published = snapshot();
            props::always!(
                accepted || published == current,
                "abi: rejected commands leave the snapshot unchanged",
                json!({ "command": command })
            );
            props::always!(
                serde_json::from_slice::<State>(&published).is_ok(),
                "abi: every published snapshot is a valid state"
            );
            props::sometimes!(!accepted, "abi: a well-formed command is rejected");
            current = published;
        }
    }
    let authored = resources(e);
    settings(e, &authored);
    input_configs(e, &authored);
    gamma(e, &authored);
    saves(e);
}

/// The own-save transport `web/engine.ts` calls (`save_input_ptr`,
/// `load_save_data`, `enable_saved_game_io`, `save_data_ptr`): documents are
/// bounded at 4 MiB, save IO is declared only with protocol 1 at a fresh
/// Title, and a document loads only into a fresh Title. A refused call
/// reports an error and leaves the game unchanged.
fn saves(e: &mut Entropy) {
    use rebirth_cleanroom::save_profiles::MAX_BYTES;
    props::always!(
        abi::save_input_ptr(MAX_BYTES + 1 + e.index(1024)).is_null()
            && !abi::save_input_ptr(MAX_BYTES).is_null(),
        "abi: save documents are bounded at 4 MiB"
    );
    let (seed, epoch) = (e.nonzero_u32(), e.u32());
    abi::reset(seed, epoch);
    let title = snapshot();
    // The document this fresh Title would store.
    let pointer = abi::save_data_ptr();
    let document = if pointer.is_null() {
        Vec::new()
    } else {
        // SAFETY: save_data_ptr fills the snapshot buffer snapshot_len describes.
        unsafe { std::slice::from_raw_parts(pointer, abi::snapshot_len()) }.to_vec()
    };
    let load = |bytes: &[u8]| {
        let pointer = abi::save_input_ptr(bytes.len());
        if pointer.is_null() {
            return None;
        }
        // SAFETY: save_input_ptr returned a writable buffer of exactly bytes.len().
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len()) };
        Some(abi::load_save_data())
    };
    let protocol = e.range(2, u32::MAX);
    let refused_protocol = abi::enable_saved_game_io(protocol) == 1 && !error().is_empty();
    let corrupt = *e.pick(&[
        &b""[..],
        b"{}",
        b"null",
        b"not json",
        br#"{"version":2,"profiles":[]}"#,
    ]);
    let refused_document = load(corrupt) == Some(1) && !error().is_empty();
    props::always!(
        refused_protocol && refused_document && snapshot() == title,
        "abi: a save IO protocol other than 1 and a corrupt save document are refused, keeping the game",
        json!({ "protocol": protocol, "document": String::from_utf8_lossy(corrupt) })
    );
    props::always!(
        !document.is_empty()
            && abi::enable_saved_game_io(1) == 0
            && load(&document) == Some(0)
            && abi::error_len() == 0,
        "abi: a fresh Title declares save IO and loads a save document",
        json!({ "error": error() })
    );
    // Leaving the Title for the profile menu ends both boundaries.
    let leave = serde_json::to_vec(&Command::Key {
        code: "Enter".into(),
        down: true,
    })
    .expect("command");
    let left = send(&leave) == Some(0);
    let moved = snapshot();
    props::always!(
        !left
            || (abi::enable_saved_game_io(1) == 1
                && load(&document) == Some(1)
                && !error().is_empty()
                && snapshot() == moved),
        "abi: save IO and save documents are refused outside a fresh Title, keeping the game"
    );
}

/// The input-configuration reset `web/engine.ts` boots with when the browser
/// stored an inputconfigs.dat: `reset_with_input_configs` reads the stored
/// configuration, any stored settings and whether to boot from the authored
/// resources from the request buffer. The file holds at most 32 retained
/// devices besides the Keyboard, under unique printable names, and Keyboard
/// bindings among GLFW's keys; anything else is refused, keeping the
/// previous game.
fn input_configs(e: &mut Entropy, authored: &[u8]) {
    let (seed, epoch) = (e.nonzero_u32(), e.u32());
    let configs = stored_input_configs(e);
    let reset = |with_resources: bool, configs: &Value| {
        let request = json!({ "withResources": with_resources, "inputConfigs": configs });
        from_request(&request, || abi::reset_with_input_configs(seed, epoch))
    };
    let stored = json!(configs);
    let plain = reset(false, &stored);
    let booted = snapshot();
    props::always!(
        plain == Some(0)
            && abi::error_len() == 0
            && serde_json::from_slice::<State>(&booted).is_ok_and(|state| {
                state.input_configs == configs && state.keyboard == configs.keyboard()
            }),
        "abi: an input-configuration reset boots with the stored configuration",
        json!({ "configs": configs, "result": plain, "error": error() })
    );
    let resourced = load(authored, || reset(true, &stored).unwrap_or(2));
    props::always!(
        resourced == Some(0) && snapshot() == booted,
        "abi: an input-configuration boot from the authored resources publishes the plain boot's state",
        json!({ "result": resourced, "error": error() })
    );

    let previous = snapshot();
    let device =
        |name: String, key: i32| json!({ "name": name, "bindings": vec![[key, -1]; ACTIONS] });
    let (case, result) = match e.below(5) {
        0 => {
            let devices: Vec<Value> = (0..33)
                .map(|index| device(format!("Pad {index}"), -1))
                .collect();
            (
                "33 retained devices",
                reset(false, &json!({ "devices": devices })),
            )
        }
        1 => {
            let name = (*e.pick(&["Keyboard", "Pad"])).to_owned();
            let devices = [device(name.clone(), -1), device(name, -1)];
            (
                "duplicate names",
                reset(false, &json!({ "devices": devices })),
            )
        }
        2 => {
            let name = match e.below(3) {
                0 => String::new(),
                1 => "Pad\n".to_owned(),
                _ => "P".repeat(257),
            };
            (
                "unprintable name",
                reset(false, &json!({ "devices": [device(name, -1)] })),
            )
        }
        3 => {
            let key = *e.pick(&[-2, 349, i32::MIN]);
            let devices = [device("Keyboard".into(), key)];
            ("keyboard key", reset(false, &json!({ "devices": devices })))
        }
        _ => {
            let request = json!({ "withResources": false, "inputConfigs": configs, "extra": true });
            (
                "unknown field",
                from_request(&request, || abi::reset_with_input_configs(seed, epoch)),
            )
        }
    };
    props::always!(
        result == Some(1) && !error().is_empty() && snapshot() == previous,
        "abi: an input-configuration reset refuses a configuration inputconfigs.dat cannot hold, keeping the previous game",
        json!({ "case": case, "result": result })
    );
}

/// The settings reset `web/engine.ts` boots with when the browser stored a
/// Gamma: `reset_with_settings_preferences` reads every stored setting, any
/// stored input configuration and whether to boot from the authored
/// resources from the request buffer, which must name one finite Gamma
/// within .5..1.5. A legacy reset keeps Gamma 1, so a request whose Gamma is
/// 1 boots what the input-configuration reset does; anything else is
/// refused, keeping the previous game.
fn gamma(e: &mut Entropy, authored: &[u8]) {
    let (seed, epoch) = (e.nonzero_u32(), e.u32());
    let volume = |e: &mut Entropy| e.below(10_001) as f32 / 10_000.0;
    let audio = if e.chance(1, 2) {
        Some(AudioPreferences {
            sfx_volume: volume(e),
            music_volume: volume(e),
        })
    } else {
        None
    };
    let fullscreen = e.chance(1, 2).then(|| FullscreenPreferences {
        fullscreen: e.chance(1, 2),
    });
    let map = if e.chance(1, 2) {
        Some(MapPreferences { opacity: volume(e) })
    } else {
        None
    };
    let configs = stored_input_configs(e);
    let stored_configs = e.chance(1, 2).then_some(configs.clone());
    let gamma = (5_000 + e.below(10_001)) as f32 / 10_000.0;
    let request = |with_resources: bool, configs: &Option<InputConfigs>, gamma: Value| {
        let mut request = json!({
            "withResources": with_resources,
            "audioPreferences": audio,
            "fullscreenPreferences": fullscreen,
            "mapPreferences": map,
            "inputConfigs": configs,
        });
        if !gamma.is_null() {
            request["gammaPreferences"] = gamma;
        }
        request
    };
    let reset = |request: &Value| {
        from_request(request, || {
            abi::reset_with_settings_preferences(seed, epoch)
        })
    };
    let stored = json!({ "gamma": gamma });
    let plain = reset(&request(false, &stored_configs, stored.clone()));
    let booted = snapshot();
    let defaults = stored_configs.clone().unwrap_or_default();
    props::always!(
        plain == Some(0)
            && abi::error_len() == 0
            && serde_json::from_slice::<State>(&booted).is_ok_and(|state| {
                state.gamma_settings.gamma == gamma
                    && state.map_settings == map.unwrap_or_default()
                    && state.fullscreen.requested == fullscreen.is_some_and(|f| f.fullscreen)
                    && audio.is_none_or(|audio| {
                        state.audio.sfx_volume == audio.sfx_volume
                            && state.music.volume == audio.music_volume
                    })
                    && state.input_configs == defaults
                    && state.keyboard == defaults.keyboard()
            }),
        "abi: a Gamma settings reset boots with the stored settings",
        json!({ "gamma": gamma, "inputConfigs": stored_configs.is_some(), "result": plain, "error": error() })
    );
    let resourced = load(authored, || {
        reset(&request(true, &stored_configs, stored.clone())).unwrap_or(2)
    });
    props::always!(
        resourced == Some(0) && snapshot() == booted,
        "abi: a Gamma settings boot from the authored resources publishes the plain boot's state",
        json!({ "result": resourced, "error": error() })
    );
    let configured = Some(configs);
    let legacy = from_request(&request(false, &configured, Value::Null), || {
        abi::reset_with_input_configs(seed, epoch)
    });
    let legacy_state = snapshot();
    let unit = reset(&request(false, &configured, json!({ "gamma": 1.0 })));
    props::always!(
        legacy == Some(0) && unit == Some(0) && snapshot() == legacy_state,
        "abi: a settings reset with Gamma 1 boots what the input-configuration reset does",
        json!({ "result": [legacy, unit], "error": error() })
    );

    let previous = snapshot();
    let (case, result) = match e.below(5) {
        0 => {
            let outside = *e.pick(&[0.4999, 1.5001, -1.0, 0.0, 2.0, 1.0e30]);
            (
                "Gamma outside .5..1.5",
                reset(&request(
                    false,
                    &stored_configs,
                    json!({ "gamma": outside }),
                )),
            )
        }
        1 => (
            "no Gamma",
            reset(&request(false, &stored_configs, Value::Null)),
        ),
        2 => (
            "null Gamma",
            reset(&request(false, &stored_configs, json!({ "gamma": null }))),
        ),
        3 => (
            "unknown Gamma field",
            reset(&request(
                false,
                &stored_configs,
                json!({ "gamma": gamma, "contrast": 1.0 }),
            )),
        ),
        _ => {
            let mut extra = request(false, &stored_configs, stored);
            extra["extra"] = json!(true);
            ("unknown field", reset(&extra))
        }
    };
    props::always!(
        result == Some(1) && !error().is_empty() && snapshot() == previous,
        "abi: a settings reset refuses a missing Gamma or one outside .5..1.5, keeping the previous game",
        json!({ "case": case, "result": result })
    );
}

/// Fills the request buffer with a boot request, then runs `reset` from it.
fn from_request(request: &Value, reset: impl FnOnce() -> u32) -> Option<u32> {
    let request = serde_json::to_vec(request).expect("boot request serializes");
    let pointer = abi::request_ptr(request.len());
    if pointer.is_null() {
        return None;
    }
    // SAFETY: request_ptr returned a writable buffer of exactly request.len().
    unsafe { std::ptr::copy_nonoverlapping(request.as_ptr(), pointer, request.len()) };
    Some(reset())
}

/// The settings resets `web/engine.ts` boots with from the stored Options
/// settings: a fullscreen reset with an audio presence flag, or an audio
/// reset. They admit only presence flags of zero or one and, when audio is
/// present, finite volumes within the unit interval; a refused reset keeps
/// the previous game. Booted from the authored resources, each publishes the
/// state its plain reset does.
fn settings(e: &mut Entropy, authored: &[u8]) {
    let (seed, epoch) = (e.nonzero_u32(), e.u32());
    let volume = |e: &mut Entropy| e.below(10_001) as f32 / 10_000.0;
    let (sfx, music) = (volume(e), volume(e));
    let (fullscreen, has_audio) = (e.below(2) as u32, e.below(2) as u32);
    let display =
        abi::reset_with_fullscreen_preferences(seed, epoch, fullscreen, has_audio, sfx, music);
    let plain_display = snapshot();
    let booted = serde_json::from_slice::<State>(&plain_display).ok();
    props::always!(
        display == 0
            && abi::error_len() == 0
            && booted.as_ref().is_some_and(|state| {
                state.fullscreen.requested == (fullscreen == 1)
                    && (has_audio == 0
                        || (state.audio.sfx_volume == sfx && state.music.volume == music))
            }),
        "abi: a settings reset boots with the stored settings",
        json!({ "fullscreen": fullscreen, "hasAudio": has_audio, "sfx": sfx, "music": music, "error": error() })
    );
    let resourced_display = load(authored, || {
        abi::reset_with_boot_resources_and_fullscreen_preferences(
            seed, epoch, fullscreen, has_audio, sfx, music,
        )
    });
    let display_matches = resourced_display == Some(0) && snapshot() == plain_display;
    let audio = abi::reset_with_audio_preferences(seed, epoch, sfx, music);
    let plain_audio = snapshot();
    let resourced_audio = load(authored, || {
        abi::reset_with_boot_resources_and_audio_preferences(seed, epoch, sfx, music)
    });
    props::always!(
        display_matches && audio == 0 && resourced_audio == Some(0) && snapshot() == plain_audio,
        "abi: a settings boot from the authored resources publishes the plain settings boot's state",
        json!({ "seed": seed, "epoch": epoch, "display": resourced_display, "audio": [audio, resourced_audio], "error": error() })
    );
    // The map reset carries presence flags for the display and audio
    // settings beside the stored Map opacity.
    let has_display = e.below(2) as u32;
    let opacity = volume(e);
    let map = abi::reset_with_map_preferences_v2(
        seed,
        epoch,
        has_display,
        fullscreen,
        has_audio,
        sfx,
        music,
        opacity,
    );
    let plain_map = snapshot();
    let mapped = serde_json::from_slice::<State>(&plain_map).ok();
    let resourced_map = load(authored, || {
        abi::reset_with_boot_resources_and_map_preferences_v2(
            seed,
            epoch,
            has_display,
            fullscreen,
            has_audio,
            sfx,
            music,
            opacity,
        )
    });
    props::always!(
        map == 0
            && mapped.as_ref().is_some_and(|state| {
                state.map_settings.opacity == opacity
                    && state.fullscreen.requested == (has_display == 1 && fullscreen == 1)
                    && (has_audio == 0
                        || (state.audio.sfx_volume == sfx && state.music.volume == music))
            })
            && resourced_map == Some(0)
            && snapshot() == plain_map,
        "abi: a map settings reset boots with the stored settings, from the authored resources too",
        json!({ "opacity": opacity, "hasFullscreen": has_display, "hasAudio": has_audio, "result": [map, resourced_map], "error": error() })
    );

    let previous = snapshot();
    let out_of_range = *e.pick(&[
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -0.25,
        1.0001,
        -1.0e-7,
    ]);
    let (bad_sfx, bad_music) = if e.chance(1, 2) {
        (out_of_range, music)
    } else {
        (sfx, out_of_range)
    };
    let flag = e.range(2, u32::MAX);
    let (case, result) = match e.below(6) {
        0 => (
            "fullscreen flag",
            abi::reset_with_fullscreen_preferences(seed, epoch, flag, has_audio, sfx, music),
        ),
        1 => (
            "audio flag",
            abi::reset_with_fullscreen_preferences(seed, epoch, fullscreen, flag, sfx, music),
        ),
        2 => (
            "display volume",
            abi::reset_with_fullscreen_preferences(seed, epoch, fullscreen, 1, bad_sfx, bad_music),
        ),
        3 => (
            "map opacity",
            abi::reset_with_map_preferences_v2(
                seed,
                epoch,
                has_display,
                fullscreen,
                has_audio,
                sfx,
                music,
                out_of_range,
            ),
        ),
        4 => (
            "map flag",
            abi::reset_with_map_preferences_v2(
                seed, epoch, flag, fullscreen, has_audio, sfx, music, opacity,
            ),
        ),
        _ => (
            "audio volume",
            abi::reset_with_audio_preferences(seed, epoch, bad_sfx, bad_music),
        ),
    };
    props::always!(
        result == 1 && !error().is_empty() && snapshot() == previous,
        "abi: a settings reset refuses flags other than zero or one and stored volumes or Map opacity outside the unit interval, keeping the previous game",
        json!({ "case": case, "flag": flag, "volumes": [bad_sfx, bad_music], "result": result })
    );
}

/// The commands both boots replay: through the menus into a run, then play.
fn script(e: &mut Entropy) -> Vec<Command> {
    let tap = |code: &str| {
        [true, false].map(|down| Command::Key {
            code: code.into(),
            down,
        })
    };
    let mut commands = Vec::new();
    for _ in 0..8 {
        commands.extend(tap("Enter"));
        commands.push(Command::Tick {
            count: e.range(1, 40),
            timestamp_millis: None,
        });
    }
    for _ in 0..e.range(20, 80) {
        commands.push(match e.below(3) {
            0 => Command::Key {
                code: (*e.pick(&["KeyW", "KeyA", "KeyS", "KeyD", "ArrowUp", "ArrowLeft"])).into(),
                down: e.chance(1, 2),
            },
            _ => Command::Tick {
                count: e.range(1, 30),
                timestamp_millis: None,
            },
        });
    }
    commands
}

/// Each command's return code and the snapshot after it.
fn replay(commands: &[Command]) -> Vec<(Option<u32>, Vec<u8>)> {
    commands
        .iter()
        .map(|command| {
            let bytes = serde_json::to_vec(command).expect("command serializes");
            (send(&bytes), snapshot())
        })
        .collect()
}

/// Returns the authored resources it booted from.
fn resources(e: &mut Entropy) -> Vec<u8> {
    props::always!(
        abi::boot_resource_ptr(MAX_BOOT_RESOURCE + 1 + e.index(1024)).is_null()
            && !abi::boot_resource_ptr(MAX_BOOT_RESOURCE).is_null(),
        "abi: boot resource buffers are bounded at 512 KiB"
    );
    let authored = serde_json::to_vec(&json!({
        "challengesXml": CHALLENGES_XML,
        "specialRooms": SPECIAL_ROOMS,
    }))
    .expect("resources serialize");
    let (seed, epoch) = (e.nonzero_u32(), e.u32());
    let commands = script(e);
    let plain = (abi::reset(seed, epoch), snapshot(), replay(&commands));
    let booted = boot_with(&authored, seed, epoch);
    let resourced = (booted.unwrap_or(2), snapshot(), replay(&commands));
    props::always!(
        booted == Some(0) && abi::error_len() == 0 && resourced.1 == plain.1,
        "abi: a boot from the authored resources publishes the plain boot's state",
        json!({ "seed": seed, "epoch": epoch, "result": booted, "error": error() })
    );
    let first = resourced.2.iter().zip(&plain.2).position(|(a, b)| a != b);
    props::always!(
        plain.0 == 0 && first.is_none(),
        "abi: a game booted from the authored resources plays exactly like the plain boot",
        json!({ "seed": seed, "epoch": epoch, "firstDifference": first.map(|at| (at, &commands[at])) })
    );
    let playing = plain
        .2
        .last()
        .and_then(|(_, state)| serde_json::from_slice::<State>(state).ok());
    props::sometimes!(
        playing.is_some_and(|state| state.screen == rebirth_cleanroom::Screen::Playing),
        "abi: the resource boot comparison reaches a run"
    );

    // Corrupted or malformed resources fail with an error, or (a benign
    // corruption) boot; a failure keeps the game already in the slot.
    for _ in 0..e.range(2, 6) {
        let previous = snapshot();
        let corrupted = corrupt(e, &authored);
        let result = boot_with(&corrupted, e.nonzero_u32(), e.u32());
        let failed_cleanly = result == Some(1) && !error().is_empty() && snapshot() == previous;
        props::always!(
            failed_cleanly || (result == Some(0) && abi::error_len() == 0),
            "abi: a resource boot either succeeds or fails with an error, keeping the previous game",
            json!({ "result": result, "error": error(), "bytes": corrupted.len() })
        );
        props::sometimes!(failed_cleanly, "abi: corrupted boot resources are rejected");
    }
    authored
}

/// Resource bytes the bridge could deliver: truncated or invalid JSON, an
/// unknown field, a wrong or oversized challenge XML, damaged STB bytes.
fn corrupt(e: &mut Entropy, authored: &[u8]) -> Vec<u8> {
    let stb = |bytes: Vec<u8>| {
        serde_json::to_vec(&json!({ "challengesXml": CHALLENGES_XML, "specialRooms": bytes }))
            .expect("resources serialize")
    };
    match e.below(7) {
        0 => authored[..e.index(authored.len())].to_vec(),
        1 => serde_json::to_vec(&json!({ "challengesXml": CHALLENGES_XML, "extra": true }))
            .expect("resources serialize"),
        2 => serde_json::to_vec(&json!({ "challengesXml": "<rooms/>" }))
            .expect("resources serialize"),
        3 => serde_json::to_vec(&json!({ "challengesXml": " ".repeat(65_537) }))
            .expect("resources serialize"),
        4 => {
            let mut xml = CHALLENGES_XML.as_bytes().to_vec();
            let at = e.index(xml.len());
            xml[at] = *e.pick(b"<>\"=/ x\0");
            let xml = String::from_utf8_lossy(&xml).into_owned();
            serde_json::to_vec(&json!({ "challengesXml": xml })).expect("resources serialize")
        }
        5 => stb(SPECIAL_ROOMS[..e.index(SPECIAL_ROOMS.len())].to_vec()),
        _ => {
            let mut bytes = SPECIAL_ROOMS.to_vec();
            for _ in 0..e.range(1, 8) {
                let at = e.index(bytes.len());
                bytes[at] = e.u32() as u8;
            }
            stb(bytes)
        }
    }
}
