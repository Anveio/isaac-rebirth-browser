//! Complete source ColorMod42f450 storage shared by item plans and attacks.
use serde::{Deserialize, Serialize};

/// Source CA0 contains eleven f32 components. Rendering currently projects
/// its tint and final offset components; unnamed storage retains every word.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AttackColor {
    pub tint: [f32; 4],
    pub source_middle: [f32; 4],
    pub offsets: [f32; 3],
}
impl AttackColor {
    pub const WHITE: Self = Self {
        tint: [1.0; 4],
        source_middle: [0.0; 4],
        offsets: [0.0; 3],
    };
    pub fn is_finite(self) -> bool {
        self.tint
            .into_iter()
            .chain(self.source_middle)
            .chain(self.offsets)
            .all(f32::is_finite)
    }
}
impl Default for AttackColor {
    fn default() -> Self {
        Self::WHITE
    }
}
