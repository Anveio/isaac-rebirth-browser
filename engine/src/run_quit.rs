//! Pause Quit's outgoing Game fade and deferred System menu request.
//! SaveRun copying and unimplemented binary persistence/Continue are separate.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SaveOutcome {
    SupportedRunCopied,
    DeadRunCleared,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunQuit {
    pub input_pending: bool,
    /// Game+10EB18, separate from the incoming +10EB14 and menu alpha.
    pub fade_alpha: f32,
    pub fade_rate: f32,
    /// System+2F2FC is consumed before the next non-skipped dispatch.
    pub system_pending: bool,
    /// System+2F2F8: unknown legacy history cannot manufacture a save flag.
    pub save_enabled: Option<bool>,
    pub save_outcome: Option<SaveOutcome>,
}

impl RunQuit {
    /// NewRun dispatcher594AF6 writes the actual save-enable byte.
    pub fn fresh() -> Self {
        Self {
            save_enabled: Some(true),
            ..Self::default()
        }
    }
    pub fn active(&self) -> bool {
        self.fade_alpha > 0.0
    }
    /// Pause5BC1E9 ->54EB00 stores .08, target menu3, and adds immediately.
    /// System595497 calls PauseInput54F7DE/5BBB40 before GameFull5954E9.
    /// Thus a Full input adds here and again in Full; interpolation adds here only.
    pub fn begin(&mut self) {
        self.input_pending = false;
        self.fade_rate = 0.08_f32;
        self.fade_alpha += self.fade_rate;
    }
    /// GameFull54FB20 adds again after HUD and before ordinary actors.
    pub fn advance_full(&mut self) {
        self.fade_alpha += self.fade_rate;
        if self.fade_alpha >= 1.0_f32 {
            self.fade_alpha = 1.0;
            self.system_pending = true;
        }
    }
}
