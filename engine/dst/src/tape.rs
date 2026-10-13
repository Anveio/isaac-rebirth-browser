//! Public command tapes in the engine's replay format.
//!
//! A retained tape is exactly the input `engine/src/bin/replay.rs` accepts and
//! the browser's `game.reset(seed, ...)` plus commands reproduces, so a DST
//! finding replays outside the harness without translation.
use rebirth_cleanroom::Command;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tape {
    pub initial_seed: u32,
    pub boot_epoch_seed: u32,
    pub commands: Vec<Command>,
}

impl Tape {
    pub fn new(initial_seed: u32, boot_epoch_seed: u32) -> Self {
        Self {
            initial_seed,
            boot_epoch_seed,
            commands: Vec::new(),
        }
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        serde_json::from_str(&text).map_err(|error| error.to_string())
    }

    /// Writes `<directory>/<name>.json` and returns its path.
    pub fn retain(&self, directory: &Path, name: &str) -> Result<PathBuf, String> {
        std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
        let path = directory.join(format!("{name}.json"));
        let text = serde_json::to_string(self).map_err(|error| error.to_string())?;
        std::fs::write(&path, text + "\n").map_err(|error| error.to_string())?;
        Ok(path)
    }
}
