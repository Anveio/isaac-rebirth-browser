//! Player main-sprite callbacks shared by the ordinary stage transition and
//! death caller. Pickup and ordinary walking presentation retain their owners.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Clip {
    Trapdoor,
    Appear,
    Death,
    UseItem,
}

impl Clip {
    pub fn name(self) -> &'static str {
        match self {
            Self::Trapdoor => "Trapdoor",
            Self::Appear => "Appear",
            Self::Death => "Death",
            Self::UseItem => "UseItem",
        }
    }
    fn frames(self) -> u32 {
        match self {
            Self::Trapdoor => crate::content::PLAYER_TRAPDOOR_FRAMES,
            Self::Appear => crate::content::PLAYER_APPEAR_FRAMES,
            Self::Death => crate::content::PLAYER_DEATH_FRAMES,
            Self::UseItem => crate::content::PLAYER_USEITEM_FRAMES,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlayerAnimation {
    /// None means the main clip is owned by walking/Pickup presentation. It
    /// cannot satisfy a completion query for this callback's guarded clips.
    pub current: Option<Clip>,
    pub frame: f32,
    pub playing: bool,
    pub guard: bool,
}

impl PlayerAnimation {
    /// Player50a5c0 sets speed1, clears the overlay, force-plays the selected
    /// main clip and sets bf4. Controls, visibility and velocity belong to
    /// its caller; this admitted slice has no overlay presentation owner.
    pub fn force_play(&mut self, clip: Clip) {
        self.current = Some(clip);
        self.frame = 0.0;
        self.playing = true;
        self.guard = true;
    }

    /// Base447190 -> Sprite40f920 advances before Player531850 tests bf4
    /// with IsFinished(""). Nonlooping clips stop on their last authored frame.
    pub fn advance_full(&mut self) {
        if let Some(clip) = self.current {
            if self.playing {
                self.frame += 1.0;
                if self.frame.floor() >= clip.frames() as f32 {
                    self.frame = clip.frames() as f32 - 1.0;
                    self.playing = false;
                }
            }
            if self.guard && self.is_finished(None) {
                self.guard = false;
            }
        }
    }

    pub fn is_playing(&self, clip: Clip) -> bool {
        self.current == Some(clip) && self.playing
    }

    /// None implements the source empty-name/current-clip query. An unowned
    /// main clip remains unknown rather than clearing a guard by default.
    pub fn is_finished(&self, requested: Option<Clip>) -> bool {
        self.current.is_some()
            && !self.playing
            && requested.is_none_or(|clip| self.current == Some(clip))
    }
}
