//! Exact source attack flags, serialized without JavaScript u64 rounding.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(from = "[u32; 2]", into = "[u32; 2]")]
pub struct AttackFlags(pub u64);

impl From<[u32; 2]> for AttackFlags {
    fn from(words: [u32; 2]) -> Self {
        Self(u64::from(words[0]) | (u64::from(words[1]) << 32))
    }
}

impl From<AttackFlags> for [u32; 2] {
    fn from(flags: AttackFlags) -> Self {
        [flags.0 as u32, (flags.0 >> 32) as u32]
    }
}
