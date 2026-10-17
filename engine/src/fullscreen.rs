//! Settings5B97C0 owns requested intent; the host owns actual window state.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullscreenPreferences {
    pub fullscreen: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    Startup,
    Options,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub sequence: u64,
    pub requested: bool,
    pub origin: Origin,
}

/// The source setting survives menu reconstruction and new runs. Requests are
/// effects, not acknowledgements; denied host operations never undo intent.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullscreenState {
    pub requested: bool,
    pub next_sequence: u64,
    pub requests: Vec<Request>,
}

impl FullscreenState {
    pub fn from_preferences(values: Option<FullscreenPreferences>) -> Result<Self, String> {
        let mut state = Self::default();
        if let Some(values) = values {
            state.request(values.fullscreen, Origin::Startup)?;
        }
        Ok(state)
    }

    pub fn begin_command(&mut self) {
        self.requests.clear();
    }

    pub fn toggle(&mut self) -> Result<(), String> {
        self.request(!self.requested, Origin::Options)
    }

    fn request(&mut self, requested: bool, origin: Origin) -> Result<(), String> {
        let sequence = self.next_sequence;
        let next = sequence
            .checked_add(1)
            .filter(|value| *value <= 9_007_199_254_740_991)
            .ok_or("Fullscreen request sequence exceeds exact host integer range")?;
        //5B97C1 stores the setting before invoking the actual window owner.
        self.requested = requested;
        self.next_sequence = next;
        self.requests.push(Request {
            sequence,
            requested,
            origin,
        });
        Ok(())
    }
}
