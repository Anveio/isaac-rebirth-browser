//! Actual Level fields consumed by the ordinary stage-clear streaks.
//! Level+81b0 is separate from the retained Game+10eae0 boss-family word.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LevelProgress {
    /// Native Level+81b0 qword. This leaf owns supported damage bits2/5;
    /// other helper534b30 bits and persistence remain separate consumers.
    pub flags: u64,
    /// Native Level+c, set only by an accepted heart-family collection.
    pub heart_collected: bool,
    /// Game.Init54E769 owns zero; damage52C4E5 records the current stage.
    /// Save54F32F/Load54EE22 copy this owner; legacy unknown stays None.
    pub last_penalty_stage: Option<u32>,
}

impl Default for LevelProgress {
    fn default() -> Self {
        Self {
            flags: 0,
            heart_collected: false,
            last_penalty_stage: Some(0),
        }
    }
}

impl LevelProgress {
    /// Generation58be4e/58be59 clears the two per-level owners.
    pub(crate) fn begin_level(&mut self) {
        self.flags = 0;
        self.heart_collected = false;
    }

    /// Already-admitted52b080 reaches52bde4 before health-container routing.
    /// Flag0x20 suppresses the bit5 and companion stage stores; HP loss does
    /// not determine this callback, including Soul/Eternal/NO_KILL cases.
    pub(crate) fn admitted_damage(&mut self, flags: u32, stage: u32) {
        if flags & 0x20 == 0 {
            self.flags |= 0x20;
            self.last_penalty_stage = Some(stage);
        }
    }

    ///52c6de sets bit2 only after the admitted red-container path.
    pub(crate) fn red_damage(&mut self, flags: u32) {
        if flags & 0x20 == 0 {
            self.flags |= 0x4;
        }
    }
}
