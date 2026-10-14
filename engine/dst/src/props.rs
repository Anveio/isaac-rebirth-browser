//! Antithesis properties with a local verdict computed from the same catalog.
//!
//! Each macro forwards to the Antithesis SDK assertion of the same kind, so a
//! run on the platform reports every property natively. The SDK never fails a
//! process by itself; locally each call site also records into a ledger and
//! [`verdict`] applies Antithesis' property semantics to the SDK's own
//! registration catalog: an `always` must be hit and never false, a
//! `sometimes` must be true at least once and an `unreachable` must never be
//! hit.
//!
//! Details are built only when the SDK would emit them (the first pass and the
//! first failure of each call site), keeping per-step checks cheap.
use serde::Serialize;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

pub static NO_DETAILS: Value = Value::Null;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Always,
    Sometimes,
    Unreachable,
}

pub struct Site {
    message: &'static str,
    kind: Kind,
    passes: AtomicU64,
    fails: AtomicU64,
    registered: AtomicBool,
    first_failure: Mutex<Option<Value>>,
}

static SITES: Mutex<Vec<&'static Site>> = Mutex::new(Vec::new());

impl Site {
    pub const fn new(message: &'static str, kind: Kind) -> Self {
        Self {
            message,
            kind,
            passes: AtomicU64::new(0),
            fails: AtomicU64::new(0),
            registered: AtomicBool::new(false),
            first_failure: Mutex::new(None),
        }
    }

    /// The SDK emits only a call site's first pass and first failure.
    pub fn would_emit(&self, condition: bool) -> bool {
        if condition {
            self.passes.load(Ordering::Relaxed) == 0
        } else {
            self.fails.load(Ordering::Relaxed) == 0
        }
    }

    pub fn record(&'static self, condition: bool, details: &Value) {
        if !self.registered.swap(true, Ordering::Relaxed) {
            SITES.lock().expect("property ledger").push(self);
        }
        if condition {
            self.passes.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let previous = self.fails.fetch_add(1, Ordering::Relaxed);
        if self.kind == Kind::Sometimes {
            return;
        }
        if previous != 0 {
            return;
        }
        *self.first_failure.lock().expect("property failure") = Some(details.clone());
        {
            eprintln!(
                "PROPERTY VIOLATION [{:?}] {}\n{}",
                self.kind,
                self.message,
                serde_json::to_string_pretty(details).unwrap_or_default()
            );
        }
    }
}

/// The case being simulated; attached to every emitted detail so platform
/// reports and local failures name a reproducible input.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    pub unit: &'static str,
    pub entropy: String,
    pub case: u64,
    pub case_seed: Option<u64>,
}

thread_local! {
    static CONTEXT: RefCell<Context> = RefCell::new(Context::default());
}

pub fn set_context(context: Context) {
    CONTEXT.with(|current| *current.borrow_mut() = context);
}

pub fn context() -> Context {
    CONTEXT.with(|current| current.borrow().clone())
}

pub fn with_context(details: Value) -> Value {
    let context = serde_json::to_value(context()).unwrap_or(Value::Null);
    match details {
        Value::Object(mut map) => {
            map.insert("dst".into(), context);
            Value::Object(map)
        }
        Value::Null => json!({ "dst": context }),
        other => json!({ "details": other, "dst": context }),
    }
}

macro_rules! property {
    ($sdk:ident, $kind:ident, $condition:expr, $message:literal, $details:expr) => {{
        static SITE: $crate::props::Site =
            $crate::props::Site::new($message, $crate::props::Kind::$kind);
        let condition: bool = $condition;
        let owned;
        let details: &::serde_json::Value = if SITE.would_emit(condition) {
            owned = $crate::props::with_context($details);
            &owned
        } else {
            &$crate::props::NO_DETAILS
        };
        ::antithesis_sdk::$sdk!(condition, $message, details);
        SITE.record(condition, details);
    }};
}

/// Must be hit, and false on no hit.
macro_rules! always {
    ($condition:expr, $message:literal $(,)?) => {
        $crate::props::always!($condition, $message, ::serde_json::Value::Null)
    };
    ($condition:expr, $message:literal, $details:expr $(,)?) => {
        $crate::props::property!(assert_always, Always, $condition, $message, $details)
    };
}

/// True on at least one hit: the workload actually explores this branch.
macro_rules! sometimes {
    ($condition:expr, $message:literal $(,)?) => {
        $crate::props::sometimes!($condition, $message, ::serde_json::Value::Null)
    };
    ($condition:expr, $message:literal, $details:expr $(,)?) => {
        $crate::props::property!(assert_sometimes, Sometimes, $condition, $message, $details)
    };
}

macro_rules! unreachable {
    ($message:literal $(,)?) => {
        $crate::props::unreachable!($message, ::serde_json::Value::Null)
    };
    ($message:literal, $details:expr $(,)?) => {{
        static SITE: $crate::props::Site =
            $crate::props::Site::new($message, $crate::props::Kind::Unreachable);
        let owned;
        let details: &::serde_json::Value = if SITE.would_emit(false) {
            owned = $crate::props::with_context($details);
            &owned
        } else {
            &$crate::props::NO_DETAILS
        };
        ::antithesis_sdk::assert_unreachable!($message, details);
        SITE.record(false, details);
    }};
}

pub(crate) use {always, property, sometimes, unreachable};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Passed,
    Failed,
    Unmet,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyResult {
    pub message: &'static str,
    pub kind: &'static str,
    pub class: &'static str,
    pub location: String,
    pub passes: u64,
    pub fails: u64,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_failure: Option<Value>,
}

/// Applies Antithesis property semantics to every SDK catalog entry declared
/// by a selected module: `rebirth_dst::units::<unit>` and the shared harness.
pub fn verdict(units: &[&str]) -> Vec<PropertyResult> {
    // Antithesis identifies a property by its message, so call sites sharing
    // a message (e.g. the pass and fail arms of one check) aggregate.
    let mut sites: BTreeMap<&'static str, (u64, u64, Option<Value>)> = BTreeMap::new();
    for site in SITES.lock().expect("property ledger").iter() {
        let entry = sites.entry(site.message).or_default();
        entry.0 += site.passes.load(Ordering::Relaxed);
        entry.1 += site.fails.load(Ordering::Relaxed);
        if entry.2.is_none() && site.kind != Kind::Sometimes {
            entry.2 = site.first_failure.lock().expect("property failure").clone();
        }
    }
    let selected = |class: &str| {
        class == "rebirth_dst::harness"
            || units.iter().any(|unit| {
                let module = format!("rebirth_dst::units::{unit}");
                class == module || class.starts_with(&format!("{module}::"))
            })
    };
    let mut results: BTreeMap<&'static str, PropertyResult> = BTreeMap::new();
    for info in antithesis_sdk::assert::ANTITHESIS_CATALOG.iter() {
        if !selected(info.class) || results.contains_key(info.message) {
            continue;
        }
        let (passes, fails, first_failure) = sites.get(info.message).cloned().unwrap_or_default();
        let status = match info.display_type {
            "Always" if fails > 0 => Status::Failed,
            "Always" if passes == 0 => Status::Unmet,
            "Sometimes" | "Reachable" if passes == 0 => Status::Unmet,
            "Unreachable" if fails > 0 => Status::Failed,
            _ => Status::Passed,
        };
        results.insert(
            info.message,
            PropertyResult {
                message: info.message,
                kind: info.display_type,
                class: info.class,
                location: format!("{}:{}", info.file, info.begin_line),
                passes,
                fails,
                status,
                first_failure,
            },
        );
    }
    let mut ordered: Vec<_> = results.into_values().collect();
    ordered.sort_by(|left, right| (left.class, left.message).cmp(&(right.class, right.message)));
    ordered
}
