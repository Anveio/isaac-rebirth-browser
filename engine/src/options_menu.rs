//! Main-family Options5A7840. Settings remain with their actual owners.
use crate::{audio, music, public_actions::PublicActions};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Public audio preferences loaded before MenuInit's first Music61 admission.
/// These are startup inputs, never captured runtime state or resource bytes.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioPreferences {
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl AudioPreferences {
    pub fn validate(self) -> Result<Self, String> {
        if [self.sfx_volume, self.music_volume]
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err("Initial audio preferences must be finite and within 0..1".into());
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OptionsMenu {
    pub selection: u8,
    pub left_held: u32,
    pub right_held: u32,
    pub vertical_held: u32,
    pub next_save_sequence: u64,
    pub save_requests: Vec<SaveRequest>,
    pub next_unavailable_sequence: u64,
    pub unavailable_requests: Vec<UnavailableRequest>,
}

/// Actual Settings5B8DB0 serializes current f32 owners with %.4f. These
/// request strings are file serialization, not a second mutable settings store.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRequest {
    pub sequence: u64,
    pub sfx_volume: String,
    pub music_volume: String,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default = "crate::map_presentation::default_serialized_opacity")]
    pub map_opacity: String,
}

/// An explicit port capability failure, not a native setting or callback.
/// Retaining the attempted row does not claim its native behavior ran.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UnavailableRequest {
    pub sequence: u64,
    pub row: u8,
}

pub enum Action {
    None,
    Controls,
    Fullscreen,
    SaveAndBack,
    UnsupportedSetting(u8),
}

impl OptionsMenu {
    /// Inputs precede5A7560's counter update. Both System parities poll.
    pub fn advance(
        &mut self,
        actions: &PublicActions,
        pressed: &BTreeSet<String>,
        audio: &mut audio::AudioState,
        music: &mut music::MusicState,
        map: &mut crate::map_presentation::MapPreferences,
        shared: &mut crate::mt::Mt19937,
        audio_counter: u32,
    ) -> Result<Action, String> {
        if self.selection > 7 {
            return Err("Options selection is outside its eight authored rows".into());
        }
        let held = |first: &str, second: &str| pressed.contains(first) || pressed.contains(second);
        let up = held("ArrowUp", "KeyW");
        let down = held("ArrowDown", "KeyS");
        let left = held("ArrowLeft", "KeyA");
        let right = held("ArrowRight", "KeyD");
        let action = if actions.menu_confirm {
            match self.selection {
                0 => Action::Controls,
                4 => Action::Fullscreen,
                _ => Action::None,
            }
        } else if up {
            if Self::repeat(self.vertical_held) {
                self.selection = (self.selection + 7) % 8;
            }
            Action::None
        } else if down {
            if Self::repeat(self.vertical_held) {
                self.selection = (self.selection + 1) % 8;
            }
            Action::None
        } else if left {
            self.adjust(-1, self.left_held, audio, music, map, shared, audio_counter)?
        } else if right {
            self.adjust(1, self.right_held, audio, music, map, shared, audio_counter)?
        } else if actions.menu_back {
            Action::SaveAndBack
        } else {
            Action::None
        };
        //5A7560 increments the same held group or resets it on release.
        self.left_held = if left {
            self.left_held
                .checked_add(1)
                .ok_or("Options left counter exhausted")?
        } else {
            0
        };
        self.right_held = if right {
            self.right_held
                .checked_add(1)
                .ok_or("Options right counter exhausted")?
        } else {
            0
        };
        self.vertical_held = if up || down {
            self.vertical_held
                .checked_add(1)
                .ok_or("Options vertical counter exhausted")?
        } else {
            0
        };
        Ok(action)
    }

    pub fn reinitialize(&mut self) {
        let sequence = self.next_save_sequence;
        let unavailable_sequence = self.next_unavailable_sequence;
        *self = Self::default();
        self.next_save_sequence = sequence;
        self.next_unavailable_sequence = unavailable_sequence;
    }

    pub fn begin_command(&mut self) {
        self.save_requests.clear();
        self.unavailable_requests.clear();
    }

    /// Keep an unimplemented public action visible without aborting System
    /// dispatch halfway through a polled tick. No native setting is changed.
    pub fn request_unavailable(&mut self, row: u8) -> Result<(), String> {
        let sequence = self.next_unavailable_sequence;
        self.next_unavailable_sequence = sequence
            .checked_add(1)
            .ok_or("Options unavailable-action sequence exhausted")?;
        self.unavailable_requests
            .push(UnavailableRequest { sequence, row });
        Ok(())
    }

    pub fn request_save(
        &mut self,
        audio: &audio::AudioState,
        music: &music::MusicState,
        fullscreen: &crate::fullscreen::FullscreenState,
        map: &crate::map_presentation::MapPreferences,
    ) -> Result<(), String> {
        let sequence = self.next_save_sequence;
        self.next_save_sequence = sequence
            .checked_add(1)
            .ok_or("Options save sequence exhausted")?;
        self.save_requests.push(SaveRequest {
            sequence,
            sfx_volume: format!("{:.4}", audio.sfx_volume),
            music_volume: format!("{:.4}", music.volume),
            fullscreen: fullscreen.requested,
            map_opacity: format!("{:.4}", map.opacity),
        });
        Ok(())
    }

    fn repeat(counter: u32) -> bool {
        counter == 0 || (counter > 49 && counter.is_multiple_of(10))
    }

    fn adjust(
        &self,
        direction: i32,
        held: u32,
        audio: &mut audio::AudioState,
        music: &mut music::MusicState,
        map: &mut crate::map_presentation::MapPreferences,
        shared: &mut crate::mt::Mt19937,
        audio_counter: u32,
    ) -> Result<Action, String> {
        if !Self::repeat(held) && !(self.selection == 6 && held > 49 && held.is_multiple_of(2)) {
            return Ok(Action::None);
        }
        let delta = direction as f32 * 0.1_f32;
        match self.selection {
            0 => {}
            1 => {
                //5B97A0's actual playing-sample callback precedes Play237.
                audio.set_volume((audio.sfx_volume + delta).clamp(0.0, 1.0))?;
                audio.admit(
                    shared,
                    audio_counter,
                    audio::SoundRequest {
                        id: 237,
                        volume: 1.0,
                        delay: 0,
                        looping: false,
                        pitch: 1.0,
                    },
                );
            }
            2 => {
                //5B9730 returns zero while the actual music enable is false.
                let volume = if music.music_enabled {
                    music.volume
                } else {
                    0.0
                };
                music.set_volume((volume + delta).clamp(0.0, 1.0));
            }
            3 => {
                //5A8129/5A8265 store actual Settings+10 then request237,
                //including a held repeat whose value is already clamped.
                map.opacity = (map.opacity + delta).clamp(0.0, 1.0);
                audio.admit(
                    shared,
                    audio_counter,
                    audio::SoundRequest {
                        id: 237,
                        volume: 1.0,
                        delay: 0,
                        looping: false,
                        pitch: 1.0,
                    },
                );
            }
            4 => return Ok(Action::Fullscreen),
            other => return Ok(Action::UnsupportedSetting(other)),
        }
        Ok(Action::None)
    }
}
