//!Actual Game551DA0 duration and independent camera-produced pixel offset.
//!The shared generator, Camera+60 ring and game age retain their own owners.
use crate::{motion::WorldVector, State};
use serde::{Deserialize, Serialize};

const ZERO: WorldVector = WorldVector { x: 0.0, y: 0.0 };

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameShake {
    ///None means an unknown restored CE9FC, never an assumed inactive shake.
    pub duration_ce9fc: Option<i32>,
    ///42D0B0 writes this independently, in screen pixels rather than world units.
    pub offset_cea00: Option<WorldVector>,
    pub frontier: Option<String>,
}

impl GameShake {
    ///Fresh System5984BB/594C1F invokes54E670 after actual Game construction.
    ///54E68F clears duration;54E77D copies the actual zero vector to output.
    pub(crate) fn fresh() -> Self {
        Self {
            duration_ce9fc: Some(0),
            offset_cea00: Some(ZERO),
            frontier: None,
        }
    }

    ///54F471/54F635 write CE9FC only; restart never heals unknown CEA00.
    pub(crate) fn reset_duration(&mut self) {
        self.duration_ce9fc = Some(0);
    }

    ///551DA4 overwrites rather than taking max(old,new). Its rumble5EF2D0
    ///consumer has no shared RNG and remains a separate controller frontier.
    pub(crate) fn overwrite_duration(&mut self, duration: i32) {
        self.duration_ce9fc = Some(duration);
    }

    ///54F938 runs before popup/transition guards and the later GameAge increment.
    pub(crate) fn advance_full_clock(&mut self) {
        if let Some(duration) = &mut self.duration_ce9fc {
            if *duration > 0 {
                *duration -= 1;
                if *duration == 0 {
                    //The following real camera pass may write a residual mean.
                    self.offset_cea00 = Some(ZERO);
                }
            }
        }
    }

    pub(crate) fn unknown(&mut self, channel: &str) {
        self.offset_cea00 = None;
        self.frontier.get_or_insert_with(|| channel.to_owned());
    }
}

///Source554490's actual >=100 branch. The caller preflights its independent
///Camera history before this real SharedBound7 draw; no cursor-fitting exists.
pub(crate) fn from_blast(state: &mut State, damage: f32) {
    if damage >= 100.0 {
        let duration = state.boot_lifecycle.shared.bounded(7) as i32 + 7;
        state
            .game_shake
            .get_or_insert_with(Default::default)
            .overwrite_duration(duration);
    }
}
