//! Source menu initializers interleaved with admitted public sound requests.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Initialization {
    LoadFreshProfile { profile: u8 },
    ResetCharacterSeed,
}

/// The boundary counts actual pending sound requests, including requests that
/// may be rejected by the sound manager. It is never an RNG draw/cursor count.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pending {
    pub after_audio_requests: usize,
    pub initialization: Initialization,
}

pub fn validate(pending: &[Pending], audio_requests: usize) -> Result<(), String> {
    let mut previous = 0;
    for event in pending {
        if event.after_audio_requests < previous || event.after_audio_requests > audio_requests {
            return Err("menu initializer lies outside the ordered audio request queue".into());
        }
        match event.initialization {
            Initialization::LoadFreshProfile { profile } if !(1..=3).contains(&profile) => {
                return Err("menu initializer requires a real profile slot".into());
            }
            _ => {}
        }
        previous = event.after_audio_requests;
    }
    Ok(())
}
