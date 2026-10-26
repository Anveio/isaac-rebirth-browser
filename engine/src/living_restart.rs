//! Configurable living Restart's retained System request and held-poll owner.
//! Old snapshots keep missing context unknown; public NewRun/Continue owns it.
use crate::difficulty::Difficulty;
use crate::{mt, seed};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Context {
    pub character_selector: u32,
    pub challenge_id: u32,
    /// The System request's r8 (Game+10ECD8), kept on the wire as sourceR8.
    #[serde(rename = "sourceR8", with = "crate::difficulty::source")]
    pub difficulty: Difficulty,
    pub source_r9: bool,
    pub seed_code: String,
    pub saved_loader: bool,
    pub special_start: bool,
}

impl Context {
    ///The existing ordinary Isaac front door owns selector0 and the run's r8
    ///difficulty. This constructor is used at its actual request/Continue
    ///completion, never inferred from a later menu selection or live inventory.
    pub fn isaac(code: String, source_r9: bool, difficulty: Difficulty) -> Result<Self, String> {
        let context = Self {
            character_selector: 0,
            challenge_id: 0,
            difficulty,
            source_r9,
            seed_code: code,
            saved_loader: false,
            special_start: false,
        };
        context.validate()?;
        Ok(context)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.character_selector != 0
            || self.challenge_id != 0
            || self.saved_loader
            || self.special_start
        {
            return Err("living Restart requires the owned ordinary Isaac request context".into());
        }
        let value = seed::decode(&self.seed_code)?;
        if seed::encode(value)? != self.seed_code {
            return Err("living Restart retains its actual canonical seed code".into());
        }
        Ok(())
    }

    ///594710's empty argument preserves only R9true/Challenge0. The one raw
    ///sample belongs here, before the old Game's remaining callbacks.
    pub fn select_empty_request_seed(&mut self, shared: &mut mt::Mt19937) -> Result<(), String> {
        self.validate()?;
        if !(self.source_r9 && self.challenge_id == 0) {
            self.seed_code = seed::encode(shared.next_u32().max(1))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LivingRestart {
    ///None is an old snapshot without the actual hold owner, not count0.
    pub held_polls: Option<u32>,
    pub context: Option<Context>,
    pub pending: Option<Context>,
}

impl LivingRestart {
    pub fn source_reset() -> Self {
        Self {
            held_polls: Some(0),
            context: None,
            pending: None,
        }
    }

    ///Called only by the actual Systemstate2 Game input head, on Full/Fast.
    ///Skipped/menu/Nightmare/notification-gated dispatch does not call this.
    pub fn poll(&mut self, held: bool, primary_type: Option<u32>) -> Result<bool, String> {
        let counter = self
            .held_polls
            .as_mut()
            .ok_or("living Restart hold history is unknown")?;
        if !held {
            *counter = 0;
            return Ok(false);
        }
        let primary_type = primary_type.ok_or("living Restart primary Player type is unknown")?;
        if primary_type == 9 {
            *counter = 0;
            return Ok(false);
        }
        *counter = counter.wrapping_add(1);
        if *counter > 60 {
            *counter = 0;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn request_after_counter(&mut self, shared: &mut mt::Mt19937) -> Result<(), String> {
        if self.pending.is_some() {
            return Err("living Restart already has a pending System request".into());
        }
        let mut context = self
            .context
            .clone()
            .ok_or("living Restart queued context is unknown")?;
        context.select_empty_request_seed(shared)?;
        self.context = Some(context.clone());
        self.pending = Some(context);
        Ok(())
    }
}

///Each synchronous own-storage attempt can suspend this same System dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResumeStage {
    AfterProfileFlush,
    AfterClear,
    AfterInitialSave,
    ///Character59F59B flush occurs after this interval's System.advance,
    ///before its request. Resume the menu tail, never poll/advance it again.
    AfterCharacterProfileFlush {
        phase: crate::phase::Phase,
        pre_counter: u32,
        bomb_triggered: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    ///Includes the consumed-clock interval whose System.advance is pending.
    pub remaining_ticks: u32,
    pub timestamp_millis: Option<u64>,
    pub presented: bool,
    pub room_before: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteResume {
    pub stage: ResumeStage,
    pub interval: Option<Interval>,
}
