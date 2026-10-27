//! Difficulty framework: the Game difficulty is run state, and every rule the
//! original keys by it is one row of `RULES`, not an `if hard` arm.
//!
//! Game+10ECD8 is set by Game::Start 54eb20 from the System request's r8
//! (594620, stashed at System+2f2e0 for Restart 594a10), written to the save
//! at 54f294 and restored by the saved loader at 54ed4d. Every gameplay read
//! compares it with 1 (hard). A consumer reads `rules(difficulty).field`; a
//! difficulty read whose consumer the port does not implement yet has no row
//! and is listed as a gap in docs/difficulty.md.
use serde::{Deserialize, Serialize};

/// Game+10ECD8. The character menu chooses it; Challenges pass 0 (59d218).
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    #[default]
    Normal = 0,
    Hard = 1,
}

impl Difficulty {
    /// The value the original stores at Game+10ECD8.
    pub const fn source(self) -> u32 {
        self as u32
    }

    pub fn from_source(value: u32) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::Hard),
            _ => Err(format!(
                "Game difficulty {value} is not a base Rebirth difficulty"
            )),
        }
    }

    /// The character menu's Up/Down toggle.
    pub const fn toggled(self) -> Self {
        match self {
            Self::Normal => Self::Hard,
            Self::Hard => Self::Normal,
        }
    }
}

/// Serializes a difficulty as the source r8 integer, for request records that
/// keep the original's System request layout on the wire.
pub mod source {
    use super::Difficulty;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &Difficulty, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(value.source())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Difficulty, D::Error> {
        Difficulty::from_source(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Builds `Rules` and the per-difficulty table from rows of
/// `field: Type = [normal, hard]`, so a row is one line.
macro_rules! rules {
    ($($(#[$meta:meta])* $field:ident: $type:ty = [$normal:expr, $hard:expr],)*) => {
        /// Every difficulty-dependent parameter the port implements.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct Rules {
            $($(#[$meta])* pub $field: $type,)*
        }

        const RULES: [Rules; 2] = [
            Rules { $($field: $normal,)* },
            Rules { $($field: $hard,)* },
        ];
    };
}

rules! {
    /// Boss death 5c8430: the last boss of a boss (5) or Boss Rush (0x11)
    /// room drops its hearts only at normal (5c87c3).
    boss_heart: bool = [true, false],
    /// NPC Init 47f5c6/47f725: added to the champion chance (0.05, or 0.2
    /// with Champion Belt); hard adds f32 0.15 (66ad1c).
    champion_chance_bonus: f32 = [0.0, 0.15],
    /// Champion death drop 4f27a0: the NPC's own RNG draws RandomInt(3) at
    /// both difficulties (4f27c8); hard (4f27d8) drops only when it is 0.
    champion_drop_needs_zero_roll: bool = [false, true],
    /// Ordinary room-clear award 5c9240: a heart award draws RandomInt(100)
    /// on the award RNG at hard (5c9e1c, 5c9f93) and spawns only when the
    /// roll is at most this.
    clear_heart_max_roll: Option<u32> = [None, Some(34)],
    /// Level::Init 58bf90..58bfd8: RandomInt(n) == 0 curses the floor, n for
    /// a fresh profile, achievement 4, achievement 33 and counter 10 (each
    /// later one wins).
    curse_odds: [u32; 4] = [[80, 30, 10, 5], [10, 5, 3, 2]],
    /// Level::Init 58c08e: rooms added to the floor's count, plus the curse
    /// stream's RandomInt(2) (58c089) that every floor draws.
    extra_rooms: Option<u32> = [None, Some(2)],
    /// Level::Init 58c114/58c124: the normal-room difficulty range on odd
    /// stages (even stages draw 5..10 at both difficulties).
    odd_stage_room_difficulty: (u8, u8) = [(1, 5), (5, 10)],
    /// HUD 5742bd: hudpickups.anm2 frame drawn under the pickup counters.
    hud_icon: Option<u32> = [None, Some(4)],
    /// Pause render 5ba8d2: pausescreen.anm2 layer 4 (Blood) is drawn.
    pause_blood: bool = [false, true],
}

/// The rules of `difficulty`.
pub const fn rules(difficulty: Difficulty) -> &'static Rules {
    &RULES[difficulty as usize]
}

/// The rows the web renderer draws, refreshed from the run's difficulty on
/// every step so a restored snapshot cannot keep another run's rows.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Presentation {
    pub hud_icon: Option<u32>,
    pub pause_blood: bool,
}

impl Presentation {
    pub const fn of(difficulty: Difficulty) -> Self {
        let rules = rules(difficulty);
        Self {
            hud_icon: rules.hud_icon,
            pause_blood: rules.pause_blood,
        }
    }
}
