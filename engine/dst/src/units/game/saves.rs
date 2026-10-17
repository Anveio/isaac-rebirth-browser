//! The own-save protocol (#646, `save_profiles`) through the browser's host
//! (`save_host`): after every command the host answers the writes and menu
//! readmissions the command left pending, as `web/main.ts` does before any
//! later input sees the state. Without the host a game never stores a
//! document, so a menu readmission finds none and Continue stays unavailable.
use crate::entropy::Entropy;
use crate::harness;
use crate::props;
use crate::tape::Tape;
use rebirth_cleanroom::save_host::{Response, SaveHost, SaveIo, SaveStore};
use rebirth_cleanroom::{Command, Game, Screen};
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
    served: &Result<Vec<Response>, String>,
    after: &Game,
) {
    props::always!(
        served.is_ok(),
        "game: the save host answers every save write and menu readmission a command leaves",
        json!({ "command": command, "error": served.as_ref().err() })
    );
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
    continued(before, after);
}

/// 54ECE0 loads the saved persistent stores into freshly constructed owners:
/// the run Continue starts is the one its profile's record saved.
fn continued(before: &Game, after: &Game) {
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
                && state.run_seed == run.run_seed
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
            "stage": [state.stage, state.stage_seed],
        })
    );
    props::sometimes!(true, "game: a saved run is continued");
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
