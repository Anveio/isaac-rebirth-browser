//! Case scheduling, panic capture and the shared harness properties.
use crate::entropy::{splitmix64, Entropy, Mode};
use crate::props::{self, Context};
use serde_json::json;
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};

/// One logical engine unit under simulation.
pub struct Unit {
    pub name: &'static str,
    pub summary: &'static str,
    pub run: fn(&mut Case),
}

/// What a unit's case receives: its entropy and budget.
pub struct Case<'a> {
    pub entropy: &'a mut Entropy,
    /// Step budget; each unit scales its iterations from it.
    pub steps: u32,
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
    /// Entropy-driven cases per unit.
    pub cases: u64,
    pub steps: u32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseRecord {
    pub unit: &'static str,
    pub case: u64,
    pub case_seed: Option<u64>,
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
        for case in 0..plan.cases {
            let seed = match plan.mode {
                Mode::Seeded => Some(
                    plan.case_seed
                        .unwrap_or_else(|| case_seed(plan.master_seed, *unit_index, case)),
                ),
                Mode::Antithesis => None,
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
            });
            let mut context = Case {
                entropy: &mut entropy,
                steps: plan.steps,
            };
            guarded(|| (unit.run)(&mut context));
            records.push(CaseRecord {
                unit: unit.name,
                case,
                case_seed: seed,
                draws: entropy.draws(),
            });
        }
    }
    records
}
