//! Entity sprite colour (+2d0) and its timed colour stack (+388).
//! SetColor 446f00 inserts an entry before the first one of equal or higher
//! priority and copies the front colour; BaseUpdate 447190 counts entries
//! down at the start of every full update, erases those reaching zero and
//! then copies the new front colour or, with none left, ResetColor (for NPCs
//! 47f430: the champion table colour, otherwise the plain base colour).
use crate::combat::AppliedDamage;
use crate::phase::Phase;
use serde::{Deserialize, Serialize};

/// KColor: tint RGBA, colorize RGBA and offset RGB (0x2c bytes).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Color {
    pub tint: [f32; 4],
    pub colorize: [f32; 4],
    pub offset: [f32; 3],
}

impl Color {
    pub const WHITE: Color = Color {
        tint: [1.0, 1.0, 1.0, 1.0],
        colorize: [0.0; 4],
        offset: [0.0; 3],
    };
    pub const fn tinted(tint: [f32; 4], offset: f32) -> Color {
        Color {
            tint,
            colorize: [0.0; 4],
            offset: [offset, offset, offset],
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ColorEntry {
    pub priority: i32,
    pub color: Color,
    pub duration: i32,
    pub remaining: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpriteColor {
    pub current: Color,
    pub stack: Vec<ColorEntry>,
}

impl Default for SpriteColor {
    fn default() -> Self {
        Self::reset(Color::WHITE)
    }
}

/// SetOffset 42f510 scales integer offsets by the stored f32 1/255.
pub const OFFSET_200: f32 = 200.0 * (1.0 / 255.0);

/// BaseUpdate 447926/44793c damage flash: tint 0.5, red offset 200.
const FLASH: Color = Color {
    tint: [0.5, 0.5, 0.5, 1.0],
    colorize: [0.0; 4],
    offset: [OFFSET_200, 0.0, 0.0],
};

impl SpriteColor {
    pub fn reset(base: Color) -> Self {
        Self {
            current: base,
            stack: Vec::new(),
        }
    }

    /// SetColor 446f00 with a positive duration and no fade.
    pub(crate) fn set(&mut self, color: Color, duration: i32, priority: i32, base: Color) {
        // SetColor446f00's nonpositive branch updates the reset colour and
        // invokes ResetColor; it never inserts a timed entry.
        if duration <= 0 {
            self.current = base;
            return;
        }
        let at = self
            .stack
            .iter()
            .position(|entry| entry.priority >= priority)
            .unwrap_or(self.stack.len());
        self.stack.insert(
            at,
            ColorEntry {
                priority,
                color,
                duration,
                remaining: duration,
            },
        );
        self.current = self.stack[0].color;
    }

    /// One BaseUpdate: count the stack down, then flash once per hit the
    /// damage drain applied without killing (4478b4: HP >= 0 continues Hit).
    pub fn advance(&mut self, phase: Phase, applied: &[AppliedDamage], base: Color) {
        self.tick(phase, base);
        self.admit_hits(phase, applied, base);
    }

    /// BaseUpdate colour countdown, before status timers and damage.
    pub(crate) fn tick(&mut self, phase: Phase, base: Color) {
        if !phase.updates_nonplayers() {
            return;
        }
        let before = self.stack.len();
        for entry in &mut self.stack {
            entry.remaining -= 1;
        }
        self.stack.retain(|entry| entry.remaining != 0);
        if self.stack.len() != before {
            self.current = self.stack.first().map_or(base, |entry| entry.color);
        }
    }

    /// Damage flashes are inserted after the colour/status countdown.
    pub(crate) fn admit_hits(&mut self, phase: Phase, applied: &[AppliedDamage], base: Color) {
        if !phase.updates_nonplayers() {
            return;
        }
        for hit in applied.iter().filter(|hit| !hit.lethal) {
            // d = (trunc(amount + 0.5) + 2) / 3, clamped to 1..=6 (4478c0).
            let duration = match (hit.event.amount + 0.5) as i32 + 2 {
                value if value / 3 > 5 => 6,
                value if value / 3 <= 0 => 1,
                value => value / 3,
            };
            self.set(FLASH, duration, 0, base);
        }
    }
}
