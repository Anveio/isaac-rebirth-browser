//! The own-save protocol (#646, `save_profiles`) through the browser's host
//! (`save_host`): after every command the host answers the writes and menu
//! readmissions the command left pending, as `web/main.ts` does before any
//! later input sees the state. Without the host a game never stores a
//! document, so a menu readmission finds none and Continue stays unavailable.
use super::driver::Driver;
use super::seeded_run_with;
use crate::entropy::Entropy;
use crate::harness;
use crate::props;
use crate::tape::Tape;
use crate::units::rooms;
use rebirth_cleanroom::save_host::{Response, SaveHost, SaveIo, SaveStore};
use rebirth_cleanroom::save_profiles::{self, Profiles};
use rebirth_cleanroom::{seed, Command, Game, Screen};
use serde_json::json;
use std::panic::{self, AssertUnwindSafe};

/// A generated case's save storage: usually empty, as on a first visit,
/// sometimes holding a document the engine must reject, and with some early
/// write revisions whose storage fails.
pub fn generated_host(e: &mut Entropy) -> SaveHost {
    let stored = match e.below(6) {
        0 => Some(
            (*e.pick(&[
                "",
                "{}",
                "null",
                "not json",
                r#"{"version":1,"profiles":[]}"#,
                r#"{"version":2,"profiles":[]}"#,
            ]))
            .to_owned(),
        ),
        _ => None,
    };
    SaveHost {
        stored,
        failed_writes: (1..=6).filter(|_| e.chance(1, 4)).collect(),
    }
}

/// What a returning player's browser holds: a fresh game with a sound save
/// host plays `steps` driver commands, the host answering each one's save
/// IO, and the document it stored is what the next session boots with. A
/// previous session that panics or stores nothing leaves none; the same
/// driver plays every generated case, which reports those with a tape.
pub fn previous_session(e: &mut Entropy, steps: u32) -> Option<String> {
    let mut tape = Tape::new(e.nonzero_u32(), e.u32());
    let host = SaveHost::default();
    tape.save_host = Some(host.clone());
    let (mut game, mut store) = tape.session().ok()?;
    let mut driver = Driver::new(e);
    for _ in 0..steps {
        let command = driver.next(&game.state, e);
        let Ok(applied) = panic::catch_unwind(AssertUnwindSafe(|| game.command(command.clone())))
        else {
            harness::take_panic();
            return None;
        };
        if applied.is_ok() {
            serve(&mut store, Some(&host), &mut game, &command).ok()?;
        }
    }
    store?.stored
}

/// What a returning player's browser holds after a run that went deeper than
/// its first floor, which a generated case seldom plays to. A previous
/// session starts a seeded run with a sound save host and descends as the
/// `rooms` unit does, through an open trapdoor under the player whose own
/// entry check decides each descent, until one refuses. Each arrival's level
/// admission saves the run, and the document stored on one of the floors it
/// reached, chosen uniformly, is what the next session boots with: its
/// import, export and menu readmission run natively and through the WASM
/// ABI, and its Continue resumes that floor through public inputs once the
/// port admits Continue past Basement I's starting room. None when no
/// trapdoor took the player or the session failed.
pub fn previous_descent(e: &mut Entropy) -> Option<String> {
    let mut tape = Tape::new(e.nonzero_u32(), e.u32());
    let host = SaveHost::default();
    tape.save_host = Some(host.clone());
    let (booted, mut store) = tape.session().ok()?;
    let code = seed::encode(e.nonzero_u32()).ok()?;
    let mut sound = true;
    let mut game = seeded_run_with(booted, &code, &mut |game, command| {
        sound &= send(game, &mut store, &host, command);
    })?;
    let tick = || Command::Tick {
        count: 1,
        timestamp_millis: None,
    };
    for _ in 0..rooms::START_FRAMES {
        if game.state.player.controls_enabled {
            break;
        }
        sound &= send(&mut game, &mut store, &host, tick());
    }
    let mut documents = Vec::new();
    for _ in 0..rooms::DESCENTS {
        let from = game.state.stage;
        if !sound || !rooms::plant_trapdoor(&mut game, e) {
            break;
        }
        let mut started = false;
        let mut arrived = false;
        for frame in 0..rooms::ARRIVAL_FRAMES {
            sound &= send(&mut game, &mut store, &host, tick());
            let state = &game.state;
            started |= state.stage_transition.active();
            if !sound || (!started && frame >= rooms::TRAPDOOR_FRAMES) {
                break;
            }
            if started
                && state.stage > from
                && !state.stage_transition.active()
                && state.player.controls_enabled
            {
                arrived = true;
                break;
            }
        }
        match store.as_ref().and_then(|store| store.stored.clone()) {
            Some(document) if arrived && sound => documents.push(document),
            _ => break,
        }
    }
    if documents.is_empty() {
        return None;
    }
    let chosen = e.below(documents.len() as u64) as usize;
    Some(documents.swap_remove(chosen))
}

/// Sends one command and has the host answer its save IO; false after a
/// panic or a save IO the host could not answer.
fn send(game: &mut Game, store: &mut Option<SaveStore>, host: &SaveHost, command: Command) -> bool {
    match panic::catch_unwind(AssertUnwindSafe(|| game.command(command.clone()))) {
        Ok(Ok(_)) => serve(store, Some(host), game, &command).is_ok(),
        Ok(Err(_)) => true,
        Err(_) => {
            harness::take_panic();
            false
        }
    }
}

/// The document a previous session stored loads at the next session's fresh
/// Title, whatever that boot's seeds and settings, and exports back byte for
/// byte. `fresh` marks a document this engine just stored, which must load;
/// a replayed tape's document may predate a save format the engine now
/// refuses, so a replay checks only a document that loads. A replayed
/// document in an older own format that loads is a migration (#768's V1 →
/// V2): it exports in the current format, and that export reloads unchanged.
pub fn reload(tape: &Tape, document: &str, fresh: bool) {
    let Ok(mut next) = tape.boot() else {
        return;
    };
    let enabled = next.enable_save_io(1);
    let imported = next.import_save_data(document.as_bytes());
    if !fresh && imported.is_err() {
        return;
    }
    let again = next.export_save_data();
    if fresh || own_version(document.as_bytes()) == Some(u64::from(save_profiles::VERSION)) {
        props::always!(
            enabled.is_ok() && imported.is_ok() && again.as_deref().ok() == Some(document.as_bytes()),
            "game: a document a previous session stored loads at the next session's Title unchanged",
            json!({ "enabled": enabled, "imported": imported, "bytes": document.len() })
        );
    } else {
        let migrated = again.as_deref().ok();
        let owners = migrated
            .ok_or_else(|| "migration did not export a document".to_owned())
            .and_then(|bytes| migrated_v1_owners(document.as_bytes(), bytes));
        let stable = migrated.is_some_and(|bytes| {
            tape.boot().is_ok_and(|mut third| {
                third.enable_save_io(1).is_ok()
                    && third.import_save_data(bytes).is_ok()
                    && third.export_save_data().as_deref().ok() == Some(bytes)
            })
        });
        props::always!(
            enabled.is_ok()
                && imported.is_ok()
                && owners.is_ok()
                && migrated.and_then(own_version) == Some(u64::from(save_profiles::VERSION))
                && stable,
            "game: a document in an older own format loads as the current format, which then reloads unchanged",
            json!({ "enabled": enabled, "from": own_version(document.as_bytes()), "to": migrated.and_then(own_version), "stable": stable, "owners": owners })
        );
        props::sometimes!(
            true,
            "game: a session loads a document in an older own format"
        );
    }
    let saved_run = next.saved_profiles.as_ref().is_some_and(|profiles| {
        profiles
            .profiles
            .iter()
            .any(|profile| profile.saved.is_some())
    });
    props::sometimes!(
        saved_run,
        "game: a session boots with a run a previous session saved"
    );
}

/// The `version` an own profile document declares.
fn own_version(bytes: &[u8]) -> Option<u64> {
    serde_json::from_slice::<serde_json::Value>(bytes)
        .ok()?
        .get("version")?
        .as_u64()
}

/// Compare every original V1 owner, projecting only the declared V2 metadata.
fn migrated_v1_owners(original: &[u8], exported: &[u8]) -> Result<(), String> {
    let original: serde_json::Value =
        serde_json::from_slice(original).map_err(|error| error.to_string())?;
    let mut actual: serde_json::Value =
        serde_json::from_slice(exported).map_err(|error| error.to_string())?;
    if original["version"] != 1 || actual["version"] != 2 {
        return Err("own V1 must migrate to the declared V2 format".into());
    }
    let profiles = actual["profiles"]
        .as_array_mut()
        .ok_or("migrated profiles are not an array")?;
    for record in profiles {
        let profile = record["profile"]
            .as_object_mut()
            .ok_or("migrated profile is not an object")?;
        let expected = json!({ "origin": "ownVersion1Unknown", "values": vec![None::<bool>; 347] });
        if profile.remove("collectedHistory") != Some(expected) {
            return Err("own V1 migration must retain exactly 347 unknown history entries".into());
        }
    }
    actual["version"] = json!(1);
    if let Some((path, _, _)) = crate::json_diff::first(&original, &actual) {
        return Err(format!(
            "own V1 migration changed an original owner at {path}"
        ));
    }
    Ok(())
}

/// Answers the save IO a command left pending; a panic is an error.
pub fn serve(
    store: &mut Option<SaveStore>,
    host: Option<&SaveHost>,
    io: &mut impl SaveIo,
    command: &Command,
) -> Result<Vec<Response>, String> {
    let (Some(store), Some(host)) = (store.as_mut(), host) else {
        return Ok(Vec::new());
    };
    panic::catch_unwind(AssertUnwindSafe(|| store.service(host, io, command)))
        .unwrap_or_else(|_| Err(harness::take_panic()))
}

/// Contracts of one serviced command, from the game before it to the game
/// after the host answered.
pub fn check(
    before: &Game,
    command: &Command,
    result: &Result<(), String>,
    served: &Result<Vec<Response>, String>,
    after: &Game,
    imported: Option<&Profiles>,
) {
    props::always!(
        served.is_ok(),
        "game: the save host answers every save write and menu readmission a command leaves",
        json!({ "command": command, "error": served.as_ref().err() })
    );
    if before.rejected_profile_document() {
        props::always!(
            after.rejected_profile_document()
                && after.export_save_data().is_err()
                && served.as_ref().is_ok_and(Vec::is_empty)
                && before.state.save_revision == after.state.save_revision
                && before.saved_profiles == after.saved_profiles
                && !after.state.save_write_pending
                && !after.state.save_readmission_pending,
            "game: rejected profile documents stay quarantined without host writes",
            json!({ "command": command, "result": result, "served": format!("{served:?}") })
        );
        let confirms = matches!(command, Command::Key { code, down: true }
            if code == "Enter" && !before.state.pressed.contains(code)
                && (matches!(before.state.screen, Screen::Character | Screen::Challenge)
                    || (before.state.screen == Screen::Main && before.state.main_selection == 1)));
        if confirms {
            props::always!(
                result.is_err()
                    && after == before
                    && !after.state.run_start_request.input_pending
                    && !after.state.run_start_request.resume_saved
                    && !after.state.run_start_request.fading
                    && !after.state.run_start_request.system_pending,
                "game: quarantined profile confirmation is rejected before a run request",
                json!({ "command": command, "result": result })
            );
        }
    }
    for response in served.as_deref().unwrap_or_default() {
        match *response {
            Response::Write { stored, .. } => {
                props::sometimes!(stored, "game: a save write is stored");
                props::sometimes!(!stored, "game: a save write's storage fails");
            }
            Response::Readmission { document, accepted } => {
                props::sometimes!(
                    document && accepted,
                    "game: a menu readmission loads the stored document"
                );
            }
        }
    }
    continued(before, after, imported);
}

/// 54ECE0 loads the saved persistent stores into freshly constructed owners:
/// the run Continue starts is the one its profile's record saved, whether
/// this session saved it or `imported` did, the document the session booted
/// with. The record keeps the seed as Saved+EF80's text, the formatter's nine
/// bytes with a space (or a legacy compact code), while the run keeps the
/// compact code, so the two compare by the seed they spell.
fn continued(before: &Game, after: &Game, imported: Option<&Profiles>) {
    let continuing = before.state.run_start_request.resume_saved
        && before.state.screen != Screen::Playing
        && after.state.screen == Screen::Playing;
    if !continuing {
        return;
    }
    let record = before
        .saved_profiles
        .as_ref()
        .and_then(|profiles| profiles.profiles.get(usize::from(before.state.profile) - 1))
        .and_then(|profile| profile.saved.as_ref());
    let state = &after.state;
    props::always!(
        record.is_some_and(|record| {
            let (run, context) = (&record.run, &record.context);
            state.character == context.character
                && state.difficulty == context.difficulty
                && state.challenge_id == context.challenge_id
                && state.run_seed_value == context.run_seed_value
                && seed::decode(&run.run_seed)
                    .is_ok_and(|saved| seed::decode(&state.run_seed) == Ok(saved))
                && state.seeded_run == run.seeded_run
                && state.stage == run.stage
                && state.stage_seed == context.stage_seed
                && state.floor.stage == run.floor.stage
                && state.floor.stage_seed == run.floor.stage_seed
                && state.floor.grid == run.floor.grid
        }),
        "game: Continue resumes the saved run's character, seeds and floor",
        json!({
            "profile": before.state.profile,
            "saved": record.is_some(),
            "character": state.character,
            "seed": [&state.run_seed, record.map(|record| &record.run.run_seed)],
            "stage": [state.stage, state.stage_seed],
        })
    );
    props::sometimes!(true, "game: a saved run is continued");
    let slot = usize::from(before.state.profile) - 1;
    props::sometimes!(
        record.is_some_and(|record| {
            imported
                .and_then(|profiles| profiles.profiles.get(slot))
                .and_then(|profile| profile.saved.as_ref())
                == Some(record)
        }),
        "game: Continue resumes a run a previous session saved"
    );
}

/// The engine refuses an acknowledgement for a write or readmission it did
/// not request, or for another revision, and changes nothing.
pub fn stale_acknowledgement(game: &Game, e: &mut Entropy) {
    let mut probe = game.clone();
    let state = &probe.state;
    let revision = state.save_revision;
    let pending_write = state.save_write_pending;
    let pending_readmission = state.save_readmission_pending;
    // Any revision but the pending one: another, or the pending one with
    // nothing of that kind requested.
    let other = revision.wrapping_add(1 + e.below(3)).min(u64::MAX - 1);
    let (attempt, result) = match e.below(4) {
        0 => (
            "write of another revision",
            probe.complete_save_write(other, e.chance(1, 2)),
        ),
        1 if !pending_write => (
            "unrequested write",
            probe.complete_save_write(revision, true),
        ),
        2 => (
            "readmission of another revision",
            probe.complete_menu_readmission(other, None).map(|_| ()),
        ),
        _ if !pending_readmission || pending_write => (
            "unrequested readmission",
            probe.complete_menu_readmission(revision, None).map(|_| ()),
        ),
        _ => return,
    };
    props::always!(
        result.is_err() && probe == *game,
        "game: a stale or unrequested save acknowledgement is refused without change",
        json!({ "attempt": attempt, "revision": revision, "result": result })
    );
    props::sometimes!(
        pending_write || pending_readmission,
        "game: a stale acknowledgement arrives while save IO is pending"
    );
}

/// The document a game would store imports into a fresh Title of the same
/// boot and exports again byte for byte.
pub fn round_trip(tape: &Tape, game: &Game) {
    let state = &game.state;
    if state.save_write_pending || state.save_readmission_pending {
        return;
    }
    if game.rejected_profile_document() {
        props::always!(
            game.export_save_data().is_err()
                && !state.save_write_pending
                && !state.save_readmission_pending,
            "game: quarantined profile documents cannot export a fresh replacement"
        );
        return;
    }
    let Ok(document) = game.export_save_data() else {
        props::always!(
            false,
            "game: a save document exports and imports back unchanged"
        );
        return;
    };
    let Ok(mut fresh) = tape.boot() else {
        return;
    };
    let imported = fresh.import_save_data(&document);
    let again = fresh.export_save_data();
    props::always!(
        imported.is_ok() && again.as_ref().ok() == Some(&document),
        "game: a save document exports and imports back unchanged",
        json!({ "imported": imported, "bytes": document.len() })
    );
}

/// The C ABI save transport `web/engine.ts` calls.
pub struct AbiSaves;

fn abi_error() -> String {
    use rebirth_cleanroom as abi;
    // SAFETY: error_ptr/error_len describe the thread-local buffer.
    let bytes = unsafe { std::slice::from_raw_parts(abi::error_ptr(), abi::error_len()) };
    String::from_utf8_lossy(bytes).into_owned()
}

fn abi_input(document: &[u8]) -> Result<(), String> {
    let pointer = rebirth_cleanroom::save_input_ptr(document.len());
    if pointer.is_null() {
        return Err("save input allocation".into());
    }
    // SAFETY: save_input_ptr returned a writable buffer of exactly document.len().
    unsafe { std::ptr::copy_nonoverlapping(document.as_ptr(), pointer, document.len()) };
    Ok(())
}

impl SaveIo for AbiSaves {
    fn enable(&mut self) -> Result<(), String> {
        match rebirth_cleanroom::enable_saved_game_io(1) {
            0 => Ok(()),
            _ => Err(abi_error()),
        }
    }
    fn import(&mut self, document: &[u8]) -> Result<(), String> {
        abi_input(document)?;
        match rebirth_cleanroom::load_save_data() {
            0 => Ok(()),
            _ => Err(abi_error()),
        }
    }
    fn pending(&mut self) -> Result<(bool, bool, u64), String> {
        use rebirth_cleanroom as abi;
        let pointer = abi::snapshot_ptr();
        // SAFETY: snapshot_ptr/snapshot_len describe the thread-local buffer.
        let snapshot = unsafe { std::slice::from_raw_parts(pointer, abi::snapshot_len()) };
        published_io(snapshot)
    }
    fn export(&mut self) -> Result<Vec<u8>, String> {
        use rebirth_cleanroom as abi;
        let pointer = abi::save_data_ptr();
        if pointer.is_null() {
            return Err(abi_error());
        }
        // SAFETY: save_data_ptr fills the snapshot buffer snapshot_len describes.
        Ok(unsafe { std::slice::from_raw_parts(pointer, abi::snapshot_len()) }.to_vec())
    }
    fn complete_write(&mut self, revision: u64, succeeded: bool) -> Result<(), String> {
        let (low, high) = (revision as u32, (revision >> 32) as u32);
        match rebirth_cleanroom::complete_saved_game_write(low, high, u32::from(succeeded)) {
            0 => Ok(()),
            _ => Err(abi_error()),
        }
    }
    fn complete_readmission(
        &mut self,
        revision: u64,
        document: Option<&[u8]>,
    ) -> Result<bool, String> {
        if let Some(document) = document {
            abi_input(document)?;
        }
        let (low, high) = (revision as u32, (revision >> 32) as u32);
        match rebirth_cleanroom::complete_save_menu_readmission(
            low,
            high,
            u32::from(document.is_some()),
        ) {
            0 => Ok(true),
            2 => Ok(false),
            _ => Err(abi_error()),
        }
    }
}

/// The save IO a published snapshot reports pending.
pub fn published_io(snapshot: &[u8]) -> Result<(bool, bool, u64), String> {
    let state: serde_json::Value =
        serde_json::from_slice(snapshot).map_err(|error| error.to_string())?;
    let flag = |key: &str| state[key].as_bool().ok_or(format!("snapshot lacks {key}"));
    Ok((
        flag("saveWritePending")?,
        flag("saveReadmissionPending")?,
        state["saveRevision"]
            .as_u64()
            .ok_or("snapshot lacks saveRevision")?,
    ))
}
