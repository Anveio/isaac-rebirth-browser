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
    let mut tapes = RETAINED.lock().expect("retained tapes").clone();
    tapes.sort();
    tapes
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
    /// Worker threads for seeded cases.
    pub jobs: usize,
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

/// One case to run: its unit and index, and the tape it replays, if any.
struct Job<'a> {
    unit_index: usize,
    unit: &'a Unit,
    case: u64,
    replay: Option<&'a (String, Tape)>,
}

/// Runs every case. Seeded cases are independent, so they spread over
/// `plan.jobs` threads; the report is identical for any thread count.
/// Antithesis drives one thread so the platform owns every decision's order.
pub fn run(plan: &Plan, units: &[(usize, &Unit)]) -> Vec<CaseRecord> {
    let mut jobs = Vec::new();
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
            .map(|(index, replay)| (plan.cases + index as u64, Some(replay)));
        for (case, replay) in generated.chain(replayed) {
            jobs.push(Job {
                unit_index: *unit_index,
                unit,
                case,
                replay,
            });
        }
    }
    let threads = if plan.mode == Mode::Antithesis {
        1
    } else {
        plan.jobs.clamp(1, jobs.len().max(1))
    };
    let next = std::sync::atomic::AtomicUsize::new(0);
    let records = std::sync::Mutex::new(Vec::new());
    let work = || loop {
        let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let Some(job) = jobs.get(index) else { break };
        let record = run_case(plan, job);
        records.lock().expect("case records").push((index, record));
    };
    if threads == 1 {
        work();
    } else {
        std::thread::scope(|scope| {
            for worker in 0..threads {
                std::thread::Builder::new()
                    .name(format!("rebirth-dst-{worker}"))
                    .stack_size(STACK_BYTES)
                    .spawn_scoped(scope, work)
                    .expect("spawn a case worker");
            }
        });
    }
    let mut records = records.into_inner().expect("case records");
    records.sort_by_key(|(index, _)| *index);
    records.into_iter().map(|(_, record)| record).collect()
}

/// The WASM interpreter's translator recurses deeply on the engine's large
/// serde functions; every case runs with a generous stack.
pub const STACK_BYTES: usize = 512 * 1024 * 1024;

fn run_case(plan: &Plan, job: &Job<'_>) -> CaseRecord {
    let Job {
        unit_index,
        unit,
        case,
        replay,
    } = *job;
    // Replays take their commands from the tape; their remaining choices
    // (e.g. the continuation midpoint) stay seeded.
    let seed = match (plan.mode, replay) {
        (Mode::Seeded, None) => Some(
            plan.case_seed
                .unwrap_or_else(|| case_seed(plan.master_seed, unit_index, case)),
        ),
        (_, Some(_)) => Some(case_seed(plan.master_seed, unit_index, case)),
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
    CaseRecord {
        unit: unit.name,
        case,
        case_seed: seed,
        replay: replay.map(|(name, _)| name.clone()),
        draws: entropy.draws(),
    }
}
