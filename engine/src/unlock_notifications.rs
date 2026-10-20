//! System's unlock-notification manager. Each award can call it immediately;
//! callbacks are not coalesced to one update per System interval.
use crate::{audio, mt};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub struct Definition {
    pub id: u32,
    pub name: &'static str,
    pub gfx: Option<&'static str>,
}

pub struct Clip {
    pub name: &'static str,
    pub frames: u32,
    pub looping: bool,
}
include!(concat!(env!("OUT_DIR"), "/unlock_notification_catalog.rs"));

fn definition(id: u32) -> Result<&'static Definition, String> {
    DEFINITIONS
        .iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("unlock notification requires authored definition {id}"))
}

fn clip(name: &str) -> Result<&'static Clip, String> {
    CLIPS
        .iter()
        .find(|entry| entry.name == name)
        .ok_or_else(|| format!("unlock notification requires authored clip {name}"))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Runtime {
    pub queue: VecDeque<u32>,
    pub state: u32,
    pub idle_counter: u32,
    pub current_id: Option<u32>,
    pub current_clip: Option<String>,
    pub frame: f32,
    pub playing: bool,
    pub gfx: Option<String>,
    pub definition_loaded: bool,
}

/// Rendering placement remains the System render owner's responsibility.
/// Empty graphics disable layer3; other authored layers remain visible.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    pub actor: &'static str,
    pub clip: String,
    pub frame: f32,
    pub layer3_spritesheet: u32,
    pub gfx: Option<String>,
}

impl Runtime {
    pub fn fresh() -> Result<Self, String> {
        for name in ["Appear", "Idle", "Dissapear"] {
            clip(name)?;
        }
        Ok(Self {
            queue: VecDeque::new(),
            state: 0,
            idle_counter: 0,
            current_id: None,
            current_clip: None,
            frame: -1.0,
            playing: false,
            gfx: None,
            definition_loaded: true,
        })
    }

    /// Call with actual new awards, after inhibit/ID0/already-set no-op checks.
    /// Unknown Popups is controlling only when a notification could be emitted.
    pub fn validate_awards(&self, ids: &[u32], popups: Option<bool>) -> Result<(), String> {
        if ids.is_empty() || popups == Some(false) {
            return Ok(());
        }
        if popups.is_none() {
            return Err("new awards require the actual Popups setting owner".into());
        }
        if !self.definition_loaded {
            return Err("unlock notification requires its loaded authored actor".into());
        }
        for &id in ids {
            definition(id)?;
        }
        Ok(())
    }

    pub fn active(&self) -> bool {
        self.state != 0
    }

    /// The caller has established Popups=true. False skips this entire call;
    /// unknown is refused by validate_awards before any actual award effects.
    pub fn enqueue_and_update(
        &mut self,
        id: u32,
        system_pre_counter: u32,
        loading: bool,
        audio: &mut audio::AudioState,
        shared: &mut mt::Mt19937,
    ) -> Result<(), String> {
        definition(id)?;
        self.queue.push_back(id);
        self.advance(system_pre_counter, loading, audio, shared)
    }

    /// Sprite first, then manager state. Full advances happen on EACH callback
    /// at an even System pre-counter, including multiple awards in one interval.
    /// Loading suppresses Sprite advancement, not the manager's state branches.
    pub fn advance(
        &mut self,
        system_pre_counter: u32,
        loading: bool,
        audio: &mut audio::AudioState,
        shared: &mut mt::Mt19937,
    ) -> Result<(), String> {
        if self.state > 3 {
            return Err("invalid unlock notification state".into());
        }
        if self.active() && system_pre_counter.is_multiple_of(2) && !loading {
            self.advance_sprite()?;
        }
        match self.state {
            1 if !self.playing => {
                self.state = 2;
                self.idle_counter = 0;
                self.play("Idle")?;
                audio.admit(
                    shared,
                    system_pre_counter,
                    audio::SoundRequest {
                        id: 8,
                        volume: 1.0,
                        delay: 2,
                        looping: false,
                        pitch: 1.0,
                    },
                );
            }
            2 => {
                let previous = self.idle_counter;
                self.idle_counter = self.idle_counter.wrapping_add(1);
                if previous > 90 {
                    self.state = 3;
                    self.play("Dissapear")?;
                }
            }
            3 if !self.playing => self.state = 0,
            _ => {}
        }
        if self.state == 0 {
            if let Some(id) = self.queue.pop_front() {
                let selected = definition(id)?;
                self.current_id = Some(id);
                self.gfx = selected.gfx.map(str::to_owned);
                self.state = 1;
                self.play("Appear")?;
            }
        }
        Ok(())
    }

    fn play(&mut self, name: &str) -> Result<(), String> {
        clip(name)?;
        self.current_clip = Some(name.into());
        self.frame = 0.0;
        self.playing = true;
        Ok(())
    }

    fn advance_sprite(&mut self) -> Result<(), String> {
        if !self.definition_loaded {
            return Err("unlock notification requires its loaded authored actor".into());
        }
        let selected = clip(
            self.current_clip
                .as_deref()
                .ok_or("active unlock notification has no current clip")?,
        )?;
        if self.playing {
            self.frame += 1.0;
            if self.frame.floor() >= selected.frames as f32 {
                if selected.looping {
                    self.frame = 0.0;
                } else {
                    self.frame = selected.frames as f32 - 1.0;
                    self.playing = false;
                }
            }
        }
        Ok(())
    }

    pub fn presentation(&self, loading: bool) -> Option<Presentation> {
        if !self.active() || loading || !self.definition_loaded {
            return None;
        }
        Some(Presentation {
            actor: ACTOR,
            clip: self.current_clip.clone()?,
            frame: self.frame,
            layer3_spritesheet: LAYER3_SPRITESHEET,
            gfx: self.gfx.clone(),
        })
    }
}
