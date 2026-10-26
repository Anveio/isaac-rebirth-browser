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
            _ => Err(format!("Game difficulty {value} is not a base Rebirth difficulty")),
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
