//! The browser's own-save host (`web/save-games.ts` and the save IO loop of
//! `web/main.ts`'s dispatch) as a deterministic model, so a run that saves and
//! continues replays outside the browser: `bin/replay.rs` and the DST harness
//! service the same protocol. After every public command the host answers each
//! pending write by storing the document that save exported (unless that
//! revision's storage fails) and each pending menu readmission with the stored
//! document, before any later input sees the state.
use crate::{save_profiles, Command, Game};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A replay's save host: the document the browser had stored when it booted,
/// and the write revisions whose storage fails.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveHost {
    #[serde(default)]
    pub stored: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub failed_writes: BTreeSet<u64>,
}

/// The host's storage while a replay runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveStore {
    pub stored: Option<String>,
    consumed_revision: u64,
}

/// One answer the host gave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    /// A write's document was stored, or its storage failed.
    Write { revision: u64, stored: bool },
    /// A menu readmission was answered with the stored document, if any, and
    /// the engine accepted or rejected it.
    Readmission { document: bool, accepted: bool },
}

/// The save transport a host drives: the engine itself, or the C ABI and the
/// shipped WASM that `web/engine.ts` calls.
pub trait SaveIo {
    fn enable(&mut self) -> Result<(), String>;
    fn import(&mut self, document: &[u8]) -> Result<(), String>;
    /// The published `saveWritePending`, `saveReadmissionPending` and
    /// `saveRevision`.
    fn pending(&mut self) -> Result<(bool, bool, u64), String>;
    fn export(&mut self) -> Result<Vec<u8>, String>;
    fn complete_write(&mut self, revision: u64, succeeded: bool) -> Result<(), String>;
    fn complete_readmission(
        &mut self,
        revision: u64,
        document: Option<&[u8]>,
    ) -> Result<bool, String>;
}

impl SaveIo for Game {
    fn enable(&mut self) -> Result<(), String> {
        self.enable_save_io(1)
    }
    fn import(&mut self, document: &[u8]) -> Result<(), String> {
        self.import_save_data(document)
    }
    fn pending(&mut self) -> Result<(bool, bool, u64), String> {
        let state = &self.state;
        Ok((
            state.save_write_pending,
            state.save_readmission_pending,
            state.save_revision,
        ))
    }
    fn export(&mut self) -> Result<Vec<u8>, String> {
        self.export_save_data()
    }
    fn complete_write(&mut self, revision: u64, succeeded: bool) -> Result<(), String> {
        self.complete_save_write(revision, succeeded)
    }
    fn complete_readmission(
        &mut self,
        revision: u64,
        document: Option<&[u8]>,
    ) -> Result<bool, String> {
        self.complete_menu_readmission(revision, document)
    }
}

impl SaveHost {
    /// SaveGameIO.load: declares save IO at the fresh Title, then imports the
    /// stored document. A document the engine rejects stays stored, as the
    /// browser's storage keeps it, and the game stays as booted.
    pub fn boot(&self, io: &mut impl SaveIo) -> Result<SaveStore, String> {
        io.enable()?;
        if let Some(text) = self.stored.as_ref().filter(|text| bounded(text)) {
            let _rejected = io.import(text.as_bytes());
        }
        Ok(SaveStore {
            stored: self.stored.clone(),
            consumed_revision: 0,
        })
    }
}

impl SaveStore {
    /// Answers the save IO `command` left pending. One write and one menu read
    /// can belong to one public System interval, so a Tick allows two
    /// responses per tick and any other command one.
    pub fn service(
        &mut self,
        host: &SaveHost,
        io: &mut impl SaveIo,
        command: &Command,
    ) -> Result<Vec<Response>, String> {
        let budget = match command {
            Command::Tick { count, .. } => u64::from(*count) * 2,
            _ => 1,
        };
        let mut responses = Vec::new();
        loop {
            let (write, readmission, revision) = io.pending()?;
            if !write && !readmission {
                return Ok(responses);
            }
            if responses.len() as u64 >= budget {
                return Err("Save IO made no bounded tick progress".into());
            }
            if write {
                if Some(revision) != self.consumed_revision.checked_add(1) {
                    return Err("Save IO requires the exact next actual callback revision".into());
                }
                let text = String::from_utf8(io.export()?).map_err(|error| error.to_string())?;
                if !bounded(&text) {
                    return Err("Saved profiles exceed the supported 4 MiB format".into());
                }
                let stored = !host.failed_writes.contains(&revision);
                if stored {
                    self.stored = Some(text);
                }
                self.consumed_revision = revision;
                io.complete_write(revision, stored)?;
                responses.push(Response::Write { revision, stored });
            } else {
                let document = self.stored.as_ref().filter(|text| bounded(text));
                let accepted =
                    io.complete_readmission(revision, document.map(|text| text.as_bytes()))?;
                responses.push(Response::Readmission {
                    document: document.is_some(),
                    accepted,
                });
            }
        }
    }
}

fn bounded(text: &str) -> bool {
    text.len() <= save_profiles::MAX_BYTES
}
