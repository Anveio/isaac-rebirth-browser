//! Public command tapes in the engine's replay format.
//!
//! A retained tape is exactly the input `engine/src/bin/replay.rs` accepts and
//! the browser's `game.reset(seed, ...)` plus commands reproduces, so a DST
//! finding replays outside the harness without translation.
use rebirth_cleanroom::{
    fullscreen::FullscreenPreferences, options_menu::AudioPreferences, Command, Game,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tape {
    pub initial_seed: u32,
    pub boot_epoch_seed: u32,
    /// The settings the browser stored from its last Options save.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_audio_preferences: Option<AudioPreferences>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_fullscreen_preferences: Option<FullscreenPreferences>,
    pub commands: Vec<Command>,
}

impl Tape {
    pub fn new(initial_seed: u32, boot_epoch_seed: u32) -> Self {
        Self {
            initial_seed,
            boot_epoch_seed,
            boot_audio_preferences: None,
            boot_fullscreen_preferences: None,
            commands: Vec::new(),
        }
    }

    /// The same boot with no commands yet.
    pub fn start(&self) -> Self {
        Self {
            commands: Vec::new(),
            ..self.clone()
        }
    }

    /// The boot `replay.rs` and the browser's reset perform for this tape.
    pub fn boot(&self) -> Result<Game, String> {
        Game::new_with_boot_preferences(
            self.initial_seed,
            self.boot_epoch_seed,
            None,
            self.boot_audio_preferences,
            self.boot_fullscreen_preferences,
        )
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        serde_json::from_str(&text).map_err(|error| error.to_string())
    }

    /// The browser's reset export for this boot (`web/engine.ts`).
    pub fn reset(&self) -> Reset {
        let audio = self.boot_audio_preferences;
        match (self.boot_fullscreen_preferences, audio) {
            (Some(display), _) => Reset::Display {
                fullscreen: u32::from(display.fullscreen),
                has_audio: u32::from(audio.is_some()),
                sfx_volume: audio.map_or(0.0, |audio| audio.sfx_volume),
                music_volume: audio.map_or(0.0, |audio| audio.music_volume),
            },
            (None, Some(audio)) => Reset::Audio {
                sfx_volume: audio.sfx_volume,
                music_volume: audio.music_volume,
            },
            (None, None) => Reset::Plain,
        }
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

/// A reset export's arguments after the two seeds. A stored fullscreen
/// setting takes the display reset, which also carries any audio settings.
pub enum Reset {
    Plain,
    Audio {
        sfx_volume: f32,
        music_volume: f32,
    },
    Display {
        fullscreen: u32,
        has_audio: u32,
        sfx_volume: f32,
        music_volume: f32,
    },
}
