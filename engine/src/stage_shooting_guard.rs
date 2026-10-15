//! The mapped shooting predicate5e37f0 reads Game+A0E0's first signed word.
//! This is the Stage controller, independent of fades and room traversal.
use serde::{Deserialize, Serialize};

///Only5e37a0's two constructor stores are admitted. Companion fields and
///broader reset/progression/restore callbacks need their own actual producers.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StageShootingGuard {
    pub state: i32,
    pub counter: i32,
}

impl StageShootingGuard {
    pub(crate) const fn fresh_game() -> Self {
        Self {
            state: 0,
            counter: 0,
        }
    }

    pub(crate) const fn active(self) -> bool {
        self.state != 0
    }
}
