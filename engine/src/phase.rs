//! Source scheduling shared by players and nonplayer entities.
//!
//! The counter is the boundary before a system iteration, not an entity age.
//! Menus and pause still advance it. With interpolation enabled, players
//! update in both phases; ordinary nonplayers update only in the full phase.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Update,
    Interpolate,
    Skipped,
}

impl Phase {
    pub fn updates_nonplayers(self) -> bool {
        self == Self::Update
    }

    pub fn player_updates(self, interpolation_enabled: bool) -> u8 {
        match self {
            Self::Update => {
                if interpolation_enabled {
                    1
                } else {
                    2
                }
            }
            Self::Interpolate => 1,
            Self::Skipped => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Clock {
    pub system_counter: u32,
    pub interpolation_enabled: bool,
}

impl Clock {
    pub fn new(system_counter: u32, interpolation_enabled: bool) -> Self {
        Self {
            system_counter,
            interpolation_enabled,
        }
    }

    pub fn next_phase(&self) -> Phase {
        if self.system_counter.is_multiple_of(2) {
            Phase::Update
        } else if self.interpolation_enabled {
            Phase::Interpolate
        } else {
            Phase::Skipped
        }
    }

    /// Native run-start dispatch normalizes the pre-counter to a full phase.
    /// It increments an odd counter once and never resets elapsed system time.
    pub fn align_run_start(&mut self) -> Result<(), String> {
        if !self.system_counter.is_multiple_of(2) {
            self.system_counter = self
                .system_counter
                .checked_add(1)
                .ok_or("system counter exhausted")?;
        }
        Ok(())
    }

    /// Call even when gameplay is frozen. The source tests the current counter
    /// before its increment; a post-presentation observation has the next value.
    pub fn advance(&mut self) -> Result<Phase, String> {
        let next = self
            .system_counter
            .checked_add(1)
            .ok_or("system counter exhausted")?;
        let phase = self.next_phase();
        self.system_counter = next;
        Ok(phase)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independently_observed_pre_and_post_presentation_boundaries() {
        // ZG80 0TRJ: post-GL1206/Game42;1207/Game43;1208/Game43.
        let mut clock = Clock::new(1206, true);
        assert_eq!(clock.advance().unwrap(), Phase::Update);
        assert_eq!(clock.system_counter, 1207);
        assert_eq!(clock.advance().unwrap(), Phase::Interpolate);
        assert_eq!(clock.system_counter, 1208);
    }

    #[test]
    fn disabled_interpolation_preserves_two_player_updates_per_full_phase() {
        let mut clock = Clock::new(0, false);
        let full = clock.advance().unwrap();
        assert_eq!(full.player_updates(false), 2);
        assert!(full.updates_nonplayers());
        let skipped = clock.advance().unwrap();
        assert_eq!(skipped, Phase::Skipped);
        assert_eq!(skipped.player_updates(false), 0);
    }

    #[test]
    fn exhausted_counter_is_atomic() {
        let mut clock = Clock::new(u32::MAX, true);
        let before = clock;
        assert!(clock.advance().is_err());
        assert_eq!(clock, before);
        assert!(clock.align_run_start().is_err());
        assert_eq!(clock, before);
    }

    #[test]
    fn run_start_normalizes_parity_without_resetting_the_system_counter() {
        let mut clock = Clock::new(231, true);
        clock.align_run_start().unwrap();
        assert_eq!(clock.system_counter, 232);
        assert_eq!(clock.next_phase(), Phase::Update);
        clock.align_run_start().unwrap();
        assert_eq!(clock.system_counter, 232);
    }
}
