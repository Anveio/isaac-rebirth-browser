//! Actual Level fields consumed by the ordinary stage-clear streaks.
//! Level+81b0 is separate from the retained Game+10eae0 boss-family word.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LevelProgress {
    /// Native Level+81b0 qword. This leaf owns supported damage bits2/5;
    /// other helper534b30 bits and persistence remain separate consumers.
    pub flags: u64,
    /// Native Level+c, set only by an accepted heart-family collection.
    pub heart_collected: bool,
    ///52c4e5 writes Game+10ecdc. Its initial/restore value is not admitted.
    pub last_penalty_stage: Option<u32>,
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
