//! Global Gamma setting and its admitted ordinary Options input domain.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GammaPreferences {
    pub gamma: f32,
}

impl Default for GammaPreferences {
    fn default() -> Self {
        Self { gamma: 1.0 }
    }
}

impl GammaPreferences {
    pub fn validate(self) -> Result<Self, String> {
        if !self.gamma.is_finite() || !(0.5..=1.5).contains(&self.gamma) {
            return Err("Initial Gamma must be finite and within .5..1.5".into());
        }
        Ok(self)
    }

    pub fn adjust(&mut self, direction: i32) {
        // Each callback performs exactly one f32 operation and one bound.
        self.gamma = if direction < 0 {
            (self.gamma - 0.01_f32).max(0.5)
        } else {
            (self.gamma + 0.01_f32).min(1.5)
        };
    }
}

pub fn default_serialized_gamma() -> String {
    format!("{:.4}", GammaPreferences::default().gamma)
}
