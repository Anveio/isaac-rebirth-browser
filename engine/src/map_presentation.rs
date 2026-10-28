//! Base Settings+10 and MinimapManager5B0F60. Only the Curse of the Maze
//! teleport shake draws from the shared MT.
use serde::{Deserialize, Serialize};

/// Independently declared settings input, admitted before menu construction.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MapPreferences {
    pub opacity: f32,
}

impl Default for MapPreferences {
    fn default() -> Self {
        // Settings5B8CD0 stores the same f32 .3 at+8,+C,+10.
        Self { opacity: 0.3_f32 }
    }
}

impl MapPreferences {
    pub fn validate(self) -> Result<Self, String> {
        if !self.opacity.is_finite() || !(0.0..=1.0).contains(&self.opacity) {
            return Err("Initial map opacity must be finite and within 0..1".into());
        }
        Ok(self)
    }
}

pub fn default_serialized_opacity() -> String {
    format!("{:.4}", MapPreferences::default().opacity)
}

/// The two actual map surfaces have separate alphas. Settings opacity is the
/// mode2 expanded-map target; it does not make the ordinary small map translucent.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MapPresentation {
    pub mode: u8,
    pub expanded_latched: bool,
    pub held_frames: u32,
    pub small_alpha: f32,
    pub expanded_alpha: f32,
    /// MinimapManager+209D0: Full updates left of the shake a Curse of the
    /// Maze teleport starts (5aeb00).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub shake: i32,
    /// MinimapManager+209D4, the shake's offset of the small map.
    #[serde(default, skip_serializing_if = "is_origin")]
    pub shake_offset: [f32; 2],
}

fn is_zero(value: &i32) -> bool {
    *value == 0
}

fn is_origin(value: &[f32; 2]) -> bool {
    *value == [0.0; 2]
}

impl MapPresentation {
    /// 5B0FC1, after the held-map request: each Full of a running shake
    /// takes one shared direction (42e8d0) and moves the offset to half its
    /// value plus that direction scaled by the remaining count over 10.
    pub fn advance_shake(&mut self, shared: &mut crate::mt::Mt19937) {
        if self.shake <= 0 {
            return;
        }
        self.shake -= 1;
        let scale = self.shake as f32 / 10.0;
        let direction = crate::boil::direction(shared);
        self.shake_offset = [
            self.shake_offset[0] * 0.5 + direction.x * scale,
            self.shake_offset[1] * 0.5 + direction.y * scale,
        ];
    }

    /// Exactly one Game Full callback, after its transition/pause guards.
    /// The return value is5B1240's ten-held-Full request to585500.
    pub fn advance_full(&mut self, held: bool, opacity: f32) -> bool {
        if held {
            self.held_frames = self.held_frames.wrapping_add(1);
        }
        let show_stage_streak = held && self.held_frames == 10;
        match self.mode {
            0 => {
                self.small_alpha = if self.small_alpha < 0.9_f32 {
                    self.small_alpha + 0.1_f32
                } else {
                    1.0
                };
                self.expanded_alpha = if self.expanded_alpha > 0.2_f32 {
                    self.expanded_alpha - 0.2_f32
                } else {
                    0.0
                };
                if held {
                    self.mode = 1;
                }
            }
            1 => {
                self.small_alpha = if !self.expanded_latched && self.small_alpha > 0.2_f32 {
                    self.small_alpha - 0.2_f32
                } else {
                    0.0
                };
                if self.held_frames > 8 {
                    self.expanded_alpha = 1.0;
                    if !held {
                        self.mode = if self.expanded_latched { 2 } else { 0 };
                    }
                } else if held {
                    self.expanded_alpha += 0.1_f32;
                } else if self.expanded_latched {
                    self.mode = 0;
                    self.expanded_latched = false;
                } else {
                    self.mode = 2;
                    self.expanded_latched = true;
                }
            }
            2 => {
                // 5B1118 clears this even on the press that enters mode1.
                // Holding from the latched map therefore starts its next
                // mode1 Full at counter1, rather than retaining this poll.
                self.held_frames = 0;
                self.small_alpha = if self.small_alpha > 0.2_f32 {
                    self.small_alpha - 0.2_f32
                } else {
                    0.0
                };
                self.expanded_alpha = if self.expanded_alpha > opacity + 0.1_f32 {
                    self.expanded_alpha - 0.1_f32
                } else if opacity - 0.1_f32 > self.expanded_alpha {
                    self.expanded_alpha + 0.1_f32
                } else {
                    opacity
                };
                if held {
                    self.mode = 1;
                }
            }
            _ => {}
        }
        if !held {
            self.held_frames = 0;
        }
        show_stage_streak
    }
}
