//! The authored System-state4 nightmare interstitial. Its sprite readiness is
//! successful definition loading, not a native bit or a GPU-completion timer.
use serde::{Deserialize, Serialize};

pub struct Definition {
    pub actor: &'static str,
    pub frames: u32,
}
include!(concat!(env!("OUT_DIR"), "/nightmare_catalog.rs"));

pub fn definition(actor: &str) -> Option<&'static Definition> {
    if actor == PROGRESS_DEFINITION.actor {
        Some(&PROGRESS_DEFINITION)
    } else {
        DEFINITIONS.iter().find(|entry| entry.actor == actor)
    }
}

/// The imported glibc rand algorithm with its default initial state, separate
/// from run xorshift and shared MT. Direct game callers occur after System init;
/// external library consumption before initialization still needs live readback.
struct LibcRand {
    words: [u32; 31],
    front: usize,
    rear: usize,
}
impl LibcRand {
    fn new() -> Self {
        let mut words = [0; 31];
        words[0] = 1;
        for index in 1..words.len() {
            words[index] = ((u64::from(words[index - 1]) * 16_807) % 2_147_483_647) as u32;
        }
        let mut rng = Self {
            words,
            front: 3,
            rear: 0,
        };
        for _ in 0..310 {
            rng.next();
        }
        rng
    }
    fn next(&mut self) -> u32 {
        let value = self.words[self.front].wrapping_add(self.words[self.rear]);
        self.words[self.front] = value;
        self.front = (self.front + 1) % self.words.len();
        self.rear = (self.rear + 1) % self.words.len();
        value >> 1
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Nightmares {
    pub order: Vec<usize>,
    pub path_index: usize,
    pub current_actor: Option<String>,
    pub request_pending: bool,
    /// Systemstate4 persists through the presentation that clears managerstate.
    pub system_active: bool,
    pub active: bool,
    pub definition_loaded: bool,
    pub progress_definition_loaded: bool,
    /// Source manager+5ec: actual Level585600 ->5b8340 resource routing.
    pub stage_resources: [u32; 12],
    pub counter: u32,
    pub frame: f32,
    pub playing: bool,
    pub stage_index: u32,
    pub alternate: bool,
    /// Floors Render5B76C0 lays out on the progress bar: 8, or 6 while the
    /// Womb achievement is locked or the challenge ends at stage 6.
    #[serde(default)]
    pub progress_floors: u32,
}
impl Nightmares {
    pub fn new() -> Result<Self, String> {
        if DEFINITIONS.is_empty() {
            return Err("nightmare initialization requires authored definitions".into());
        }
        let mut order: Vec<_> = (0..DEFINITIONS.len()).collect();
        let mut rng = LibcRand::new();
        // 5b83e0 shuffles only after the authored progress definition loads.
        let progress_definition_loaded = definition(PROGRESS_ACTOR).is_some();
        if progress_definition_loaded {
            for index in 1..order.len() {
                order.swap(index, rng.next() as usize % (index + 1));
            }
        }
        Ok(Self {
            order,
            path_index: 0,
            current_actor: None,
            request_pending: false,
            system_active: false,
            active: false,
            definition_loaded: false,
            progress_definition_loaded,
            stage_resources: [12; 12],
            counter: 0,
            // Sprite40dc60 -> Clear40d9e0 owns the empty-channel -1 sentinel.
            frame: -1.0,
            playing: false,
            stage_index: 0,
            alternate: false,
            progress_floors: 0,
        })
    }

    pub fn request(&mut self, alternate: bool) {
        self.request_pending = true;
        self.alternate = alternate;
    }

    pub fn select_normal_basement(&mut self, stage: u32) -> Result<(), String> {
        if crate::levels::node(stage).is_none() {
            return Err(
                "stage resource routing admits only the level framework's stage nodes".into(),
            );
        }
        // 5b8345..5b835b selects ((stage-1)&~1)+alternate. Ordinary Basement
        // floors route 0 and Caves floors 2; twelve constructor entries start at 12.
        self.stage_resources[stage as usize] = (stage - 1) & !1;
        Ok(())
    }

    /// System5950a0 runs before ordinary Game dispatch on both presentations.
    /// Return true when this presentation belongs to Systemstate4; callers
    /// suppress Game/room/camera dispatch while retaining System/music updates.
    /// System5950a0 applies the pending request before the released input poll.
    ///
    /// `womb_unlocked` is Persistent achievement 4 (Manager+34 = PGD+20+4) and
    /// `challenge_end_stage` the current challenge Config+70.
    pub fn apply_request(
        &mut self,
        stage: u32,
        womb_unlocked: bool,
        challenge_end_stage: Option<u32>,
    ) -> bool {
        if self.request_pending {
            self.request_pending = false;
            let actor = DEFINITIONS[self.order[self.path_index]].actor;
            self.current_actor = Some(actor.into());
            self.definition_loaded = definition(actor).is_some();
            self.system_active = true;
            self.active = true;
            self.counter = 0;
            self.frame = 0.0;
            self.playing = true;
            self.stage_index = if stage == 11 { 9 } else { stage - 1 };
            self.path_index = (self.path_index + 1) % self.order.len();
            // 5B77F0..5B7CB3: Womb locked limits the bar to Depths II except
            // a fresh Womb I arrival; 5B7E10 a stage-6 challenge always does.
            self.progress_floors = if challenge_end_stage == Some(6) {
                6
            } else if !womb_unlocked
                && self.stage_index <= 7
                && (self.stage_index != 6 || self.alternate)
            {
                6
            } else {
                8
            };
            return true;
        }
        false
    }

    /// Advance System state4 after the same presentation's action poll.
    pub fn advance_system(&mut self, skip_triggered: bool) -> bool {
        if !self.system_active {
            return false;
        }
        if !self.active {
            self.system_active = false;
            return false;
        }
        if !self.definition_loaded {
            return true;
        }
        let previous_counter = self.counter;
        self.counter = self.counter.wrapping_add(1);
        if previous_counter.is_multiple_of(2) && self.playing {
            let clip = definition(self.current_actor.as_deref().expect("loaded actor"))
                .expect("own loaded definition");
            self.frame += 1.0;
            if self.frame.floor() >= clip.frames as f32 {
                self.frame = clip.frames as f32 - 1.0;
                self.playing = false;
            }
        }
        if !self.playing || skip_triggered {
            self.active = false;
            self.current_actor = None;
            self.definition_loaded = false;
            // Clear40da3e resets the main clock even on an input-driven skip.
            self.frame = -1.0;
            self.playing = false;
        }
        true
    }
}
