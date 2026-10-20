//! Persistent callbacks over the existing Profile achievement/counter arrays.
//! Prepared mutations prove unknown-history independence before effects escape.
//! This module does not supply platform or unlock-notification implementations.
use crate::{challenges, Profile};
use serde::{Deserialize, Serialize};

pub const ACHIEVEMENT_COUNT: usize = 179;
pub const COUNTER_COUNT: usize = 95;
pub const COLLECTED_COUNT: usize = 347;

///Live active-Persistent words. Own profile files must not serialize these as
///permanent slot flags. No Default: reset/load/run callers own their lifetime.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Runtime {
    ///Some(false/true) retains the legacy runtime JSON bool. None serializes
    ///as null and remains unknown when absent; only real reset owns clean.
    #[serde(default)]
    pub dirty: Option<bool>,
    pub inhibit: bool,
}

impl Runtime {
    pub fn fresh() -> Self {
        Self {
            dirty: Some(false),
            inhibit: false,
        }
    }

    ///54EBFD copies the actual queued sourceR9 byte, including Challenge R9.
    pub fn set_run_inhibit(&mut self, source_r9: bool) {
        self.inhibit = source_r9;
    }

    ///Stop54F555, after its earlier conditional dirty-profile flush.
    pub fn stop_clear_inhibit(&mut self) {
        self.inhibit = false;
    }

    ///5BD3A0 clears dirty BEFORE attempting the selected writer. Failure must
    ///not restore dirty or turn the attempt into an automatic retry.
    pub fn take_dirty_flush(&mut self) -> Result<bool, String> {
        let requested = self
            .dirty
            .ok_or("Persistent conditional flush depends on unknown legacy dirty history")?;
        self.dirty = Some(false);
        Ok(requested)
    }

    ///5BF010 sets dirty even for checksum0 during the real SavedGame Clear.
    pub fn mark_run_checksum_write(&mut self) {
        self.dirty = Some(true);
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CollectionOrigin {
    ///Only an actual absent own document or explicit profile reset owns this.
    SourceReset,
    ///Current own V1 has no collection-history field. Its absence is unknown.
    OwnVersion1Unknown,
}

///One separate collected-history array, never inferred from owned counts or
///the run acquisition-history set. Null entries remain genuinely unknown.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectedHistory {
    pub origin: CollectionOrigin,
    pub values: Vec<Option<bool>>,
}

impl CollectedHistory {
    pub fn source_reset() -> Self {
        Self {
            origin: CollectionOrigin::SourceReset,
            values: vec![Some(false); COLLECTED_COUNT],
        }
    }

    pub fn legacy_own_v1() -> Self {
        Self {
            origin: CollectionOrigin::OwnVersion1Unknown,
            values: vec![None; COLLECTED_COUNT],
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.values.len() != COLLECTED_COUNT {
            return Err("Persistent collected history requires exactly347 entries".into());
        }
        if self.origin == CollectionOrigin::SourceReset && self.values.iter().any(Option::is_none) {
            return Err(
                "constructor-known collected history cannot contain unknown entries".into(),
            );
        }
        Ok(())
    }
}

///Required real callback owners. Validation is side-effect free and checks all
///ordered new-award calls before apply. Implementations must retain platform
///gates/backend status, actual Popups, authored notification metadata, immediate
///manager updates and their same-call SFX/shared-RNG effects. No no-op default.
pub trait AwardCallbacks {
    fn validate_awards(&self, ordered_ids: &[u32]) -> Result<(), String>;
    fn report_platform(&mut self, id: u32) -> Result<(), String>;
    fn enqueue_and_update_notification(&mut self, id: u32) -> Result<(), String>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Knowledge {
    False,
    True,
    Unknown,
}

#[derive(Clone, Debug)]
enum Step {
    Counter { index: usize, value: i32 },
    Achievement(u32),
    Collected(usize),
    Platform(u32),
    Notification(u32),
    Dirty(Option<bool>),
}

///A fully proved source-ordered callback chain. Preparation only clones actual
///owners; it neither installs new stats arrays nor executes backend callbacks.
#[derive(Clone, Debug)]
pub struct Mutation {
    achievements_before: Vec<bool>,
    counters_before: Vec<i32>,
    history_before: CollectedHistory,
    runtime_before: Runtime,
    steps: Vec<Step>,
}

impl Mutation {
    pub fn ordered_new_awards(&self) -> Vec<u32> {
        self.steps
            .iter()
            .filter_map(|step| match step {
                Step::Achievement(id) => Some(*id),
                _ => None,
            })
            .collect()
    }

    ///The public command's existing clone/commit boundary must contain this
    ///entire apply, including required callback state. Backend requests are
    ///recorded admissions; actual host IO occurs after that command commits.
    pub fn apply(
        self,
        achievements: &mut Vec<bool>,
        counters: &mut Vec<i32>,
        history: &mut CollectedHistory,
        runtime: &mut Runtime,
        callbacks: &mut impl AwardCallbacks,
    ) -> Result<(), String> {
        if *achievements != self.achievements_before
            || *counters != self.counters_before
            || *history != self.history_before
            || *runtime != self.runtime_before
        {
            return Err("stale Persistent mutation preimage".into());
        }
        let awards = self.ordered_new_awards();
        if !awards.is_empty() {
            callbacks.validate_awards(&awards)?;
        }
        for step in self.steps {
            match step {
                Step::Counter { index, value } => counters[index] = value,
                Step::Achievement(id) => achievements[id as usize] = true,
                Step::Collected(id) => history.values[id] = Some(true),
                Step::Platform(id) => callbacks.report_platform(id)?,
                Step::Notification(id) => callbacks.enqueue_and_update_notification(id)?,
                Step::Dirty(value) => runtime.dirty = value,
            }
        }
        Ok(())
    }
}

struct Builder<'a> {
    profile: Profile,
    history: CollectedHistory,
    runtime: Runtime,
    catalog: &'a challenges::Catalog,
    steps: Vec<Step>,
}

impl<'a> Builder<'a> {
    fn new(
        profile: &Profile,
        history: &CollectedHistory,
        runtime: Runtime,
        catalog: &'a challenges::Catalog,
    ) -> Self {
        Self {
            profile: profile.clone(),
            history: history.clone(),
            runtime,
            catalog,
            steps: Vec::new(),
        }
    }

    fn validate_owners(&self) -> Result<(), String> {
        if self.profile.achievements.len() != ACHIEVEMENT_COUNT
            || self.profile.counters.len() != COUNTER_COUNT
        {
            return Err("Persistent requires actual179 achievements and95 known counters".into());
        }
        self.history.validate()
    }

    fn dirty(&mut self) {
        self.runtime.dirty = Some(true);
        self.steps.push(Step::Dirty(Some(true)));
    }

    fn add_counter(&mut self, index: usize, delta: i32) -> Result<(), String> {
        if self.runtime.inhibit {
            return Ok(());
        }
        self.validate_owners()?;
        let old = *self
            .profile
            .counters
            .get(index)
            .ok_or("Persistent counter index outside95 words")?;
        let value = if delta.wrapping_neg() > old {
            0
        } else {
            old.wrapping_add(delta)
        };
        self.profile.counters[index] = value;
        self.steps.push(Step::Counter { index, value });
        //Even old21==0 owns this dirty write and the complete ordered scan.
        self.dirty();
        for &(counter, threshold, award) in THRESHOLDS_BEFORE_PAIR {
            if self.profile.counters[counter] as u32 > threshold {
                self.award(award)?;
            }
        }
        if self.profile.counters[23] != 0 && self.profile.counters[24] != 0 {
            self.award(41)?;
        }
        for &(counter, threshold, award) in THRESHOLDS_AFTER_PAIR {
            if self.profile.counters[counter] as u32 > threshold {
                self.award(award)?;
            }
        }
        if [36, 69, 80, 91, 47, 58]
            .into_iter()
            .all(|counter| self.profile.counters[counter] as u32 > 1)
        {
            //The source's final predicate tail-calls this same award owner.
            self.award(156)?;
        }
        Ok(())
    }

    fn award(&mut self, id: u32) -> Result<bool, String> {
        if self.runtime.inhibit || id == 0 {
            return Ok(false);
        }
        let awarded = *self
            .profile
            .achievements
            .get(id as usize)
            .ok_or("Persistent award id outside179 bits")?;
        if awarded {
            return Ok(false);
        }
        self.validate_owners()?;
        self.profile.achievements[id as usize] = true;
        self.steps.push(Step::Achievement(id));
        self.steps.push(Step::Platform(id));
        self.steps.push(Step::Notification(id));
        self.dirty();
        for challenge_id in 1..=20 {
            let definition = self.catalog.get(challenge_id)?;
            let mut unlocked = true;
            for &required in &definition.achievements {
                let bit = self
                    .profile
                    .achievements
                    .get(required as usize)
                    .ok_or("Challenge prerequisite outside actual179 achievement bits")?;
                if !bit {
                    unlocked = false;
                    break;
                }
            }
            if unlocked {
                if let Some(award) = challenge_award(challenge_id) {
                    self.award(award)?;
                }
            }
        }
        self.completion()?;
        Ok(true)
    }

    fn completion(&mut self) -> Result<(), String> {
        let (qualifying, missing_special_zero) = self.item_gates();
        if qualifying == Knowledge::False {
            return Ok(());
        }
        //Captured before either recursive award, just like the source locals.
        let awarded = self.profile.achievements[1..]
            .iter()
            .filter(|bit| **bit)
            .count();
        let missing_exempt = EXEMPT_ACHIEVEMENTS
            .iter()
            .filter(|&&id| !self.profile.achievements[id])
            .count();
        if !self.profile.achievements[69] && awarded + missing_exempt > 176 {
            if qualifying == Knowledge::Unknown {
                return Err(
                    "Persistent award69 depends on unknown legacy collected history".into(),
                );
            }
            self.award(69)?;
        }
        //Retain captured awarded even if recursive69 changed current bits.
        if !self.profile.achievements[84]
            && awarded > 176
            && missing_special_zero != Knowledge::False
        {
            if qualifying == Knowledge::Unknown || missing_special_zero == Knowledge::Unknown {
                return Err(
                    "Persistent award84 depends on unknown legacy collected history".into(),
                );
            }
            self.award(84)?;
        }
        Ok(())
    }

    fn item_gates(&self) -> (Knowledge, Knowledge) {
        let mut qualifying = Knowledge::True;
        let mut missing_special_zero = Knowledge::True;
        for id in 1..COLLECTED_COUNT {
            let value = self.history.values[id];
            if SPECIAL_ITEMS.contains(&id) {
                //A missing special contributes to Q through M, so Q is always1.
                match value {
                    Some(false) => missing_special_zero = Knowledge::False,
                    None if missing_special_zero != Knowledge::False => {
                        missing_special_zero = Knowledge::Unknown
                    }
                    _ => {}
                }
            } else if !EXEMPT_ITEMS.contains(&id) {
                match value {
                    Some(false) => qualifying = Knowledge::False,
                    None if qualifying != Knowledge::False => qualifying = Knowledge::Unknown,
                    _ => {}
                }
            }
        }
        (qualifying, missing_special_zero)
    }

    fn collected(&mut self, id: usize) -> Result<(), String> {
        if self.runtime.inhibit {
            return Ok(());
        }
        self.validate_owners()?;
        let previous = *self
            .history
            .values
            .get(id)
            .ok_or("collected id outside347 entries")?;
        if previous == Some(true) {
            return Ok(());
        }
        //Preflight the source old-false branch. A true old bit returns
        //without these callbacks; both branches still guarantee target=true.
        let previous_dirty = self.runtime.dirty;
        self.history.values[id] = Some(true);
        let start = self.steps.len();
        self.steps.push(Step::Collected(id));
        self.dirty();
        self.completion()?;
        if previous.is_none() {
            if self.steps[start..].iter().any(|step| {
                matches!(
                    step,
                    Step::Achievement(_) | Step::Platform(_) | Step::Notification(_)
                )
            }) {
                return Err(
                    "collected setter's award callback depends on its unknown old history bit"
                        .into(),
                );
            }
            //No award callback can distinguish the two source branches.
            //Known dirty stays dirty; otherwise the old-true/old-false dirty
            //outcomes are unresolved until an actual controlling flush or a
            //real unconditional dirty write. Do not infer the ORIGINAL bit.
            let dirty = if previous_dirty == Some(true) {
                Some(true)
            } else {
                None
            };
            self.runtime.dirty = dirty;
            for step in &mut self.steps[start..] {
                if let Step::Dirty(value) = step {
                    *value = dirty;
                }
            }
        }
        Ok(())
    }

    fn finish(self, profile: &Profile, history: &CollectedHistory, runtime: Runtime) -> Mutation {
        Mutation {
            achievements_before: profile.achievements.clone(),
            counters_before: profile.counters.clone(),
            history_before: history.clone(),
            runtime_before: runtime,
            steps: self.steps,
        }
    }
}

///Living R's helper computes -old21 only after the real inhibit early return.
///Wrapping negation resets every possible original i32 word, including MIN.
pub fn prepare_restart_counter21(
    profile: &Profile,
    history: &CollectedHistory,
    runtime: Runtime,
    catalog: &challenges::Catalog,
) -> Result<Mutation, String> {
    let mut builder = Builder::new(profile, history, runtime, catalog);
    if !runtime.inhibit {
        builder.validate_owners()?;
        let delta = builder.profile.counters[21].wrapping_neg();
        builder.add_counter(21, delta)?;
    }
    Ok(builder.finish(profile, history, runtime))
}

pub fn prepare_counter(
    profile: &Profile,
    history: &CollectedHistory,
    runtime: Runtime,
    catalog: &challenges::Catalog,
    index: usize,
    delta: i32,
) -> Result<Mutation, String> {
    let mut builder = Builder::new(profile, history, runtime, catalog);
    builder.add_counter(index, delta)?;
    Ok(builder.finish(profile, history, runtime))
}

pub fn prepare_award(
    profile: &Profile,
    history: &CollectedHistory,
    runtime: Runtime,
    catalog: &challenges::Catalog,
    id: u32,
) -> Result<Mutation, String> {
    let mut builder = Builder::new(profile, history, runtime, catalog);
    builder.award(id)?;
    Ok(builder.finish(profile, history, runtime))
}

pub fn prepare_collected(
    profile: &Profile,
    history: &CollectedHistory,
    runtime: Runtime,
    catalog: &challenges::Catalog,
    id: usize,
) -> Result<Mutation, String> {
    let mut builder = Builder::new(profile, history, runtime, catalog);
    builder.collected(id)?;
    Ok(builder.finish(profile, history, runtime))
}

fn challenge_award(id: u32) -> Option<u32> {
    match id {
        4..=11 => Some(id + 153),
        19 => Some(165),
        20 => Some(166),
        _ => None,
    }
}

const THRESHOLDS_BEFORE_PAIR: &[(usize, u32, u32)] = &[
    (2, 99, 85),
    (3, 9, 28),
    (3, 99, 12),
    (6, 3, 36),
    (8, 9, 26),
    (4, 99, 145),
    (10, 9, 68),
    (10, 4, 57),
    (12, 4, 78),
];
const THRESHOLDS_AFTER_PAIR: &[(usize, u32, u32)] = &[
    (9, 99, 30),
    (1, 0, 81),
    (1, 1, 150),
    (1, 2, 8),
    (1, 3, 139),
    (1, 4, 33),
    (1, 5, 140),
    (1, 6, 141),
    (1, 7, 10),
    (1, 8, 11),
    (1, 9, 32),
    (1, 10, 34),
    (11, 19, 61),
    (13, 99, 64),
    (14, 9, 66),
    (15, 19, 142),
    (16, 29, 147),
    (17, 29, 148),
];
const SPECIAL_ITEMS: [usize; 6] = [333, 276, 335, 334, 331, 283];
const EXEMPT_ITEMS: [usize; 5] = [59, 43, 235, 61, 263];
const EXEMPT_ACHIEVEMENTS: [usize; 9] = [129, 130, 131, 132, 133, 156, 84, 82, 175];
