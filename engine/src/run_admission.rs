//! Source run-start clock and the full-only Game54f910 fade guard.
//!
//! Elapsed time is an input owned by this runtime, separate from System's
//! phase counter and Game's active-update age. Controlled presentations use
//! their declared public interval; normal browser frames supply host clocks.
use crate::Player;
use serde::{Deserialize, Serialize};

pub const PRESENTATION_INTERVAL_NS: u32 = 16_667_000;
pub const MAX_SAFE_TIMESTAMP_MS: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WallClock {
    pub milliseconds: i64,
    pub sub_millisecond_nanoseconds: u32,
    pub nanoseconds_per_step: u32,
}

impl WallClock {
    pub fn new(epoch_seconds: u32) -> Self {
        Self {
            milliseconds: i64::from(epoch_seconds) * 1000,
            sub_millisecond_nanoseconds: 0,
            nanoseconds_per_step: PRESENTATION_INTERVAL_NS,
        }
    }

    pub fn advance(&mut self, timestamp_ms: Option<u64>) -> Result<(), String> {
        if let Some(timestamp_ms) = timestamp_ms {
            if timestamp_ms > MAX_SAFE_TIMESTAMP_MS {
                return Err("host timestamp exceeds the JavaScript safe integer range".into());
            }
            self.milliseconds = timestamp_ms as i64;
            self.sub_millisecond_nanoseconds = 0;
        } else {
            let nanos =
                u64::from(self.sub_millisecond_nanoseconds) + u64::from(self.nanoseconds_per_step);
            self.milliseconds = self
                .milliseconds
                .checked_add((nanos / 1_000_000) as i64)
                .ok_or("elapsed wall clock exhausted")?;
            self.sub_millisecond_nanoseconds = (nanos % 1_000_000) as u32;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunAdmission {
    pub clock: WallClock,
    /// System593470 writes this after the actual MenuController confirmation.
    pub requested_time_anchor_ms: Option<i64>,
    /// Game+10eb14 and +10eb1c; the subtraction stays in f32.
    pub initial_fade: f32,
    pub fade_rate: f32,
    ///593660 input reset remains pending. The caller admits the separate
    ///585500 stage streak on the actual positive-to-zero Full boundary.
    pub completion_callbacks_pending: bool,
}

impl RunAdmission {
    pub fn new(epoch_seconds: u32) -> Self {
        Self {
            clock: WallClock::new(epoch_seconds),
            requested_time_anchor_ms: None,
            initial_fade: 0.0,
            fade_rate: 0.0,
            completion_callbacks_pending: false,
        }
    }

    /// A controlled harness declares how long the original ran before the
    /// state this reset represents. The shipped game passes host timestamps
    /// instead and never sets it. Only a fresh, unadvanced clock accepts it.
    pub fn set_boot_elapsed(
        &mut self,
        epoch_seconds: u32,
        milliseconds: u32,
        sub_millisecond_nanoseconds: u32,
    ) -> Result<(), String> {
        if sub_millisecond_nanoseconds >= 1_000_000 {
            return Err("boot elapsed sub-millisecond part must be below 1ms".into());
        }
        if self.clock != WallClock::new(epoch_seconds) || self.requested_time_anchor_ms.is_some() {
            return Err("boot elapsed time applies only to a fresh reset clock".into());
        }
        self.clock.milliseconds += i64::from(milliseconds);
        self.clock.sub_millisecond_nanoseconds = sub_millisecond_nanoseconds;
        Ok(())
    }

    pub fn confirm_menu_request(&mut self) {
        self.requested_time_anchor_ms = Some(self.clock.milliseconds);
    }

    /// Fresh54eb20 ->54eaa0. Loading an owned saved run remains separate.
    pub fn initialize_fresh(&mut self, player: &mut Player) -> Result<(), String> {
        if self.requested_time_anchor_ms.is_none() {
            return Err("run initialization requires its actual menu clock anchor".into());
        }
        self.initial_fade = 1.0;
        self.fade_rate = 0.04_f32;
        player.controls_enabled = false;
        self.completion_callbacks_pending = false;
        Ok(())
    }

    ///Saved54ECE0 ->54EF5E invokes the same54EAA0 fade initializer with
    ///the actual.04 operand, after reconstruction and its immediate Room.Update.
    pub fn initialize_saved(&mut self, player: &mut Player) -> Result<(), String> {
        if self.requested_time_anchor_ms.is_none() {
            return Err("Continue initialization requires its actual menu clock anchor".into());
        }
        self.initial_fade = 1.0;
        self.fade_rate = 0.04_f32;
        player.controls_enabled = false;
        self.completion_callbacks_pending = false;
        Ok(())
    }

    /// Called only in the full branch. A completed fade allows Game age to
    /// increment and full actors to advance in this same dispatch.
    pub fn blocks_full_update(&mut self, player: &mut Player) -> Result<bool, String> {
        if self.initial_fade <= 0.0 {
            return Ok(false);
        }
        let anchor = self
            .requested_time_anchor_ms
            .ok_or("initial fade has no owned request clock anchor")?;
        // Source cmp/jle is signed. A real host clock rollback holds the fade.
        if self.clock.milliseconds.saturating_sub(anchor) > 3000 {
            self.initial_fade -= self.fade_rate;
            if self.initial_fade <= 0.0 {
                self.initial_fade = 0.0;
                player.controls_enabled = true;
                self.completion_callbacks_pending = true;
            }
        }
        Ok(self.initial_fade > 0.0)
    }
}
