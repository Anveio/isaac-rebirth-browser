//! Actual GiantBook5805c0/580330 lifecycle for the Eternal NewStage and Black-heart callbacks.
//! Other configured books and Heal-event callers retain their own admission.
use serde::{Deserialize, Serialize};

pub struct Definition {
    pub actor: &'static str,
    pub clip: &'static str,
    pub frames: u32,
    pub spritesheet: &'static str,
}
include!(concat!(env!("OUT_DIR"), "/information_catalog.rs"));

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InformationAnimation {
    pub state: u32,
    pub delay: i32,
    pub current_actor: Option<String>,
    pub current_clip: Option<String>,
    pub spritesheet: Option<String>,
    pub definition_loaded: bool,
    pub frame: f32,
    pub playing: bool,
}

impl Default for InformationAnimation {
    fn default() -> Self {
        Self {
            state: 0,
            delay: 0,
            current_actor: None,
            current_clip: None,
            spritesheet: None,
            definition_loaded: false,
            frame: -1.0,
            playing: false,
        }
    }
}

impl InformationAnimation {
    pub fn active(&self) -> bool {
        self.state != 0
    }

    /// Game stop/restart callers own580310: Sprite.Clear and state0, while
    /// preserving the information manager's caller delay word.
    pub fn clear(&mut self) {
        *self = Self {
            delay: self.delay,
            ..Self::default()
        };
    }

    /// Source5805c0 reloads the definition, clears its channels, then plays the
    /// configured name and stores the actual caller delay. Definition readiness
    /// comes from this build's successfully decoded authored catalogue.
    pub fn request_eternal(&mut self) {
        self.request(&ETERNAL_DEFINITION);
    }

    /// Damage52c143 calls5805c0(16,0) before the forced item35 callback.
    pub fn request_black_heart(&mut self) {
        self.request(&BLACK_HEART_DEFINITION);
    }

    fn request(&mut self, definition: &Definition) {
        self.current_actor = Some(definition.actor.into());
        self.current_clip = Some(definition.clip.into());
        self.spritesheet = Some(definition.spritesheet.into());
        self.definition_loaded = true;
        self.frame = 0.0;
        self.playing = true;
        self.state = 1;
        self.delay = 0;
    }

    /// Stage state4 and ordinary Game call580330 only on full updates.
    /// State1 exits before Sprite.Update; state2 checks completion before it.
    pub fn advance_full(&mut self) -> Result<(), String> {
        match self.state {
            0 => return Ok(()),
            1 => {
                self.delay = self.delay.wrapping_sub(1);
                if self.delay <= 0 {
                    self.state = 2;
                }
                return Ok(());
            }
            2 => {}
            _ => return Err("unadmitted information animation state".into()),
        }
        let definition = [&ETERNAL_DEFINITION, &BLACK_HEART_DEFINITION]
            .into_iter()
            .find(|definition| {
                self.current_actor.as_deref() == Some(definition.actor)
                    && self.current_clip.as_deref() == Some(definition.clip)
            })
            .filter(|_| self.definition_loaded)
            .ok_or("information callback requires its owned authored definition")?;
        if !self.playing {
            self.state = 0;
        }
        if self.playing {
            self.frame += 1.0;
            if self.frame.floor() >= definition.frames as f32 {
                self.frame = definition.frames as f32 - 1.0;
                self.playing = false;
            }
        }
        // Unlike Nightmare Clear, completion retains this main definition,
        // current clip and final frame. Both admitted clips have no Heal event.
        Ok(())
    }
}
