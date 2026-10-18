//! Public command tapes in the engine's replay format.
//!
//! A retained tape is exactly the input `engine/src/bin/replay.rs` accepts and
//! the browser's `game.reset(seed, ...)` plus commands reproduces, so a DST
//! finding replays outside the harness without translation.
use rebirth_cleanroom::{
    fullscreen::FullscreenPreferences,
    keyboard::InputConfigs,
    map_presentation::MapPreferences,
    options_menu::AudioPreferences,
    save_host::{SaveHost, SaveStore},
    Command, Game,
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
    /// The input configuration file (inputconfigs.dat) the browser stored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_input_configs: Option<InputConfigs>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_map_preferences: Option<MapPreferences>,
    /// The browser's own-save storage (`save_host`); absent, no save IO.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_host: Option<SaveHost>,
    pub commands: Vec<Command>,
}

impl Tape {
    pub fn new(initial_seed: u32, boot_epoch_seed: u32) -> Self {
        Self {
            initial_seed,
            boot_epoch_seed,
            boot_audio_preferences: None,
            boot_fullscreen_preferences: None,
            boot_input_configs: None,
            boot_map_preferences: None,
            save_host: None,
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
        Game::new_with_input_configs_and_map_preferences(
            self.initial_seed,
            self.boot_epoch_seed,
            None,
            self.boot_audio_preferences,
            self.boot_fullscreen_preferences,
            self.boot_input_configs.clone(),
            self.boot_map_preferences,
        )
    }

    /// The boot and, with a save host, its declared save IO and stored
    /// document, as the browser performs them before the first command.
    pub fn session(&self) -> Result<(Game, Option<SaveStore>), String> {
        let mut game = self.boot()?;
        let store = self
            .save_host
            .as_ref()
            .map(|host| host.boot(&mut game))
            .transpose()?;
        Ok((game, store))
    }

    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
        serde_json::from_str(&text).map_err(|error| error.to_string())
    }

    /// The browser's reset export for this boot (`web/engine.ts`).
    pub fn reset(&self) -> Reset {
        let audio = self.boot_audio_preferences;
        if let Some(configs) = &self.boot_input_configs {
            let request = serde_json::json!({
                "withResources": false,
                "audioPreferences": audio,
                "fullscreenPreferences": self.boot_fullscreen_preferences,
                "mapPreferences": self.boot_map_preferences,
                "inputConfigs": configs,
            });
            return Reset::InputConfigs {
                request: serde_json::to_vec(&request).expect("boot request serializes"),
            };
        }
        if let Some(map) = self.boot_map_preferences {
            let display = self.boot_fullscreen_preferences;
            return Reset::Map {
                has_fullscreen: u32::from(display.is_some()),
                fullscreen: u32::from(display.is_some_and(|display| display.fullscreen)),
                has_audio: u32::from(audio.is_some()),
                sfx_volume: audio.map_or(0.0, |audio| audio.sfx_volume),
                music_volume: audio.map_or(0.0, |audio| audio.music_volume),
                opacity: map.opacity,
            };
        }
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

/// A reset export's arguments after the two seeds. A stored input
/// configuration takes the request-buffer reset, which carries every stored
/// setting; otherwise a stored Map opacity takes the map reset, and a stored
/// fullscreen setting the display reset, each carrying the settings before
/// it.
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
    /// `reset_with_map_preferences_v2`.
    Map {
        has_fullscreen: u32,
        fullscreen: u32,
        has_audio: u32,
        sfx_volume: f32,
        music_volume: f32,
        opacity: f32,
    },
    /// `reset_with_input_configs`, after `request_ptr` receives `request`.
    InputConfigs {
        request: Vec<u8>,
    },
}
