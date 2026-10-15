//! Case scheduling, panic capture and the shared harness properties.
use crate::entropy::{splitmix64, Entropy, Mode};
use crate::props::{self, Context};
use crate::tape::Tape;
use serde_json::json;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;

/// One logical engine unit under simulation.
pub struct Unit {
    pub name: &'static str,
    pub summary: &'static str,
    pub run: fn(&mut Case),
}

/// What a unit's case receives: its entropy, budget and replay inputs.
pub struct Case<'a> {
    pub entropy: &'a mut Entropy,
    /// Game-command budget for stateful units; kernels scale from it.
    pub steps: u32,
    pub tapes: Option<PathBuf>,
    /// Replays this tape instead of generating commands (game unit only).
    pub replay: Option<Tape>,
    /// Generated commands continue from the end of this tape (game unit).
    pub resume: Option<Tape>,
    pub wasm: Option<&'a crate::wasm::Module>,
    pub label: String,
    /// Retain every game tape (corpus harvesting), not only violating ones.
    pub retain_all: bool,
}

impl Case<'_> {
    /// Retains `tape` under the configured directory, if any.
    pub fn retain(&self, tape: &Tape, suffix: &str) -> Option<String> {
        let directory = self.tapes.as_ref()?;
        match tape.retain(directory, &format!("{}-{suffix}", self.label)) {
            Ok(path) => {
                let path = path.display().to_string();
                eprintln!("retained tape: {path}");
                RETAINED.lock().expect("retained tapes").push(path.clone());
                props::set_tape(Some(path.clone()));
                Some(path)
            }
            Err(error) => {
                eprintln!("could not retain tape: {error}");
                None
            }
        }
    }
}

static RETAINED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

pub fn retained_tapes() -> Vec<String> {
    RETAINED.lock().expect("retained tapes").clone()
}

thread_local! {
    static LAST_PANIC: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Records panic text instead of printing it; the property carries it.
pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_default();
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|text| text.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-text panic payload".into());
        LAST_PANIC.with(|last| *last.borrow_mut() = Some(format!("{payload} at {location}")));
    }));
}

pub fn take_panic() -> String {
    LAST_PANIC
        .with(|last| last.borrow_mut().take())
        .unwrap_or_else(|| "unknown panic".into())
}

/// Runs `body`, reporting a panic as an `unreachable` property violation.
pub fn guarded<T>(body: impl FnOnce() -> T) -> Option<T> {
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(value) => Some(value),
        Err(_) => {
            let message = take_panic();
            props::unreachable!("harness: no unit case panics", json!({ "panic": message }));
            None
        }
    }
}

pub struct Plan {
    pub mode: Mode,
    pub master_seed: u64,
    pub case_seed: Option<u64>,
    /// Entropy-driven cases per unit; zero when only replaying.
    pub cases: u64,
    pub steps: u32,
    pub tapes: Option<PathBuf>,
    /// Named tapes the game unit replays after its generated cases.
    pub replays: Vec<(String, Tape)>,
    /// A tape every generated game case replays before driving on.
    pub resume: Option<Tape>,
    pub wasm: Option<crate::wasm::Module>,
    pub retain_all: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseRecord {
    pub unit: &'static str,
    pub case: u64,
    pub case_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay: Option<String>,
    pub draws: u64,
}

pub fn case_seed(master: u64, unit_index: usize, case: u64) -> u64 {
    let mut state = master ^ (unit_index as u64).wrapping_mul(0xd1b5_4a32_d192_ed03);
    state ^= case.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    splitmix64(&mut state)
}

pub fn run(plan: &Plan, units: &[(usize, &Unit)]) -> Vec<CaseRecord> {
    let mut records = Vec::new();
    for (unit_index, unit) in units {
        let replays: &[(String, Tape)] = if unit.name == "game" {
            &plan.replays
        } else {
            &[]
        };
        let generated = (0..plan.cases).map(|case| (case, None));
        let replayed = replays
            .iter()
            .enumerate()
            .map(|(index, (name, tape))| (plan.cases + index as u64, Some((name, tape))));
        for (case, replay) in generated.chain(replayed) {
            // Replays take their commands from the tape; their remaining
            // choices (e.g. the continuation midpoint) stay seeded.
            let seed = match (plan.mode, replay) {
                (Mode::Seeded, None) => Some(
                    plan.case_seed
                        .unwrap_or_else(|| case_seed(plan.master_seed, *unit_index, case)),
                ),
                (_, Some(_)) => Some(case_seed(plan.master_seed, *unit_index, case)),
                (Mode::Antithesis, None) => None,
            };
            let mut entropy = match seed {
                Some(seed) => Entropy::seeded(seed),
                None => Entropy::antithesis(),
            };
            props::set_context(Context {
                unit: unit.name,
                entropy: plan.mode.to_string(),
                case,
                case_seed: seed,
                step: 0,
                tape: replay.map(|(name, _)| name.clone()),
            });
            let label = match (replay, seed) {
                (Some((name, _)), _) => format!("{}-replay-{name}", unit.name),
                (None, Some(seed)) => format!("{}-{seed:016x}", unit.name),
                (None, None) => format!("{}-antithesis-{case}", unit.name),
            };
            let mut context = Case {
                entropy: &mut entropy,
                steps: plan.steps,
                tapes: plan.tapes.clone(),
                replay: replay.map(|(_, tape)| tape.clone()),
                resume: plan
                    .resume
                    .clone()
                    .filter(|_| replay.is_none() && unit.name == "game"),
                wasm: plan.wasm.as_ref(),
                label,
                retain_all: plan.retain_all,
            };
            guarded(|| (unit.run)(&mut context));
            records.push(CaseRecord {
                unit: unit.name,
                case,
                case_seed: seed,
                replay: replay.map(|(name, _)| name.clone()),
                draws: entropy.draws(),
            });
        }
    }
    records
}
