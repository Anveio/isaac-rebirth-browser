//! Antithesis deterministic simulation testing for the Rust/WASM engine core.
//!
//! `rebirth-dst` drives logical engine units with entropy-chosen inputs and
//! checks Antithesis properties after every step. Locally a seeded stream
//! makes every case reproducible; under Antithesis (`--entropy antithesis`)
//! the platform supplies and steers every decision. See `docs/dst.md`.
mod coverage;
mod entropy;
mod harness;
mod json_diff;
mod props;
mod tape;
mod units;
mod wasm;

use entropy::Mode;
use harness::Plan;
use props::Status;
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: rebirth-dst [--units all|a,b] [--cases N] [--steps N] [--seed U64]
                   [--case-seed U64] [--entropy seeded|antithesis] [--tapes DIR]
                   [--replay TAPE.json] [--resume TAPE.json] [--corpus DIR]
                   [--wasm ENGINE.wasm]
                   [--report PATH] [--allow-unmet] [--retain-all] [--jobs N]
                   [--list] [--setup-complete]";

struct Options {
    units: Vec<String>,
    cases: u64,
    steps: u32,
    seed: Option<u64>,
    case_seed: Option<u64>,
    mode: Mode,
    tapes: Option<PathBuf>,
    replay: Option<PathBuf>,
    resume: Option<PathBuf>,
    corpus: Option<PathBuf>,
    wasm: Option<PathBuf>,
    report: Option<PathBuf>,
    allow_unmet: bool,
    list: bool,
    setup_complete: bool,
    retain_all: bool,
    jobs: usize,
}

fn parse() -> Result<Options, String> {
    let mut options = Options {
        units: vec!["all".into()],
        cases: 8,
        steps: 1500,
        seed: None,
        case_seed: None,
        mode: Mode::Seeded,
        tapes: None,
        replay: None,
        resume: None,
        corpus: None,
        wasm: None,
        report: None,
        allow_unmet: false,
        list: false,
        setup_complete: false,
        retain_all: false,
        jobs: std::thread::available_parallelism().map_or(1, usize::from),
    };
    let mut arguments = std::env::args().skip(1);
    while let Some(flag) = arguments.next() {
        let mut value = || {
            arguments
                .next()
                .ok_or_else(|| format!("{flag} requires a value"))
        };
        match flag.as_str() {
            "--units" => options.units = value()?.split(',').map(str::to_string).collect(),
            "--cases" => options.cases = value()?.parse().map_err(|_| "invalid --cases")?,
            "--steps" => options.steps = value()?.parse().map_err(|_| "invalid --steps")?,
            "--seed" => options.seed = Some(parse_seed(&value()?)?),
            "--case-seed" => options.case_seed = Some(parse_seed(&value()?)?),
            "--entropy" => {
                options.mode = match value()?.as_str() {
                    "seeded" => Mode::Seeded,
                    "antithesis" => Mode::Antithesis,
                    other => return Err(format!("unknown entropy source {other}")),
                }
            }
            "--tapes" => options.tapes = Some(value()?.into()),
            "--replay" => options.replay = Some(value()?.into()),
            "--resume" => options.resume = Some(value()?.into()),
            "--corpus" => options.corpus = Some(value()?.into()),
            "--wasm" => options.wasm = Some(value()?.into()),
            "--report" => options.report = Some(value()?.into()),
            "--allow-unmet" => options.allow_unmet = true,
            "--list" => options.list = true,
            "--setup-complete" => options.setup_complete = true,
            "--retain-all" => options.retain_all = true,
            "--jobs" => options.jobs = value()?.parse().map_err(|_| "invalid --jobs")?,
            "--help" | "-h" => return Err(USAGE.into()),
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }
    Ok(options)
}

fn parse_seed(text: &str) -> Result<u64, String> {
    let parsed = match text.strip_prefix("0x") {
        Some(hex) => u64::from_str_radix(hex, 16),
        None => text.parse(),
    };
    parsed.map_err(|_| format!("invalid seed {text}"))
}

/// `--replay` names one tape; `--corpus` adds every tape in a directory
/// (regression and coverage tapes), replayed after the generated game cases.
fn load_replays(options: &Options) -> Result<Vec<(String, tape::Tape)>, String> {
    let mut paths = Vec::new();
    if let Some(path) = &options.replay {
        paths.push(path.clone());
    }
    if let Some(directory) = &options.corpus {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect();
        entries.sort();
        paths.extend(entries);
    }
    paths
        .into_iter()
        .map(|path| {
            let name = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            tape::Tape::read(&path)
                .map(|tape| (name, tape))
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect()
}

fn os_seed() -> u64 {
    use std::io::Read;
    let mut bytes = [0u8; 8];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .expect("read OS entropy for the master seed");
    u64::from_le_bytes(bytes)
}

fn main() -> ExitCode {
    std::thread::Builder::new()
        .name("rebirth-dst".into())
        .stack_size(harness::STACK_BYTES)
        .spawn(simulate)
        .expect("spawn the simulation thread")
        .join()
        .unwrap_or(ExitCode::FAILURE)
}

fn simulate() -> ExitCode {
    let options = match parse() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    if options.list {
        for unit in units::UNITS {
            println!("{:<12} {}", unit.name, unit.summary);
        }
        return ExitCode::SUCCESS;
    }
    antithesis_sdk::antithesis_init();
    if options.setup_complete {
        // The Antithesis entrypoint signals readiness once; Test Composer
        // drivers then run units with `--entropy antithesis`.
        antithesis_sdk::lifecycle::setup_complete(&json!({ "units": units::UNITS.len() }));
        return ExitCode::SUCCESS;
    }
    harness::install_panic_hook();
    let mut selected = Vec::new();
    let names: Vec<&str> = if options.replay.is_some() {
        vec!["game"]
    } else if options.units.iter().any(|unit| unit == "all") {
        units::UNITS.iter().map(|unit| unit.name).collect()
    } else {
        options.units.iter().map(String::as_str).collect()
    };
    for name in &names {
        match units::UNITS.iter().position(|unit| unit.name == *name) {
            Some(index) => selected.push((index, &units::UNITS[index])),
            None => {
                eprintln!("unknown unit {name}; --list shows the registry");
                return ExitCode::from(2);
            }
        }
    }
    let replays = match load_replays(&options) {
        Ok(replays) => replays,
        Err(error) => {
            eprintln!("cannot read replay tapes: {error}");
            return ExitCode::from(2);
        }
    };
    // Generated game cases branch from the end of this tape.
    let resume = match options.resume.as_deref().map(tape::Tape::read).transpose() {
        Ok(resume) => resume,
        Err(error) => {
            eprintln!("cannot read resume tape: {error}");
            return ExitCode::from(2);
        }
    };
    let wasm = match options.wasm.as_deref().map(wasm::Module::load).transpose() {
        Ok(wasm) => wasm,
        Err(error) => {
            eprintln!("cannot load WASM engine: {error}");
            return ExitCode::from(2);
        }
    };
    let master_seed = options.seed.unwrap_or_else(os_seed);
    let replaying = options.replay.is_some();
    let plan = Plan {
        mode: options.mode,
        master_seed,
        case_seed: options.case_seed,
        cases: if replaying {
            0
        } else if options.case_seed.is_some() {
            1
        } else {
            options.cases
        },
        steps: options.steps,
        tapes: options.tapes.clone(),
        replays,
        resume,
        wasm,
        retain_all: options.retain_all,
        jobs: options.jobs,
    };
    eprintln!(
        "rebirth-dst: entropy={} seed=0x{master_seed:016x} units={} cases={} steps={}{}",
        plan.mode,
        names.join(","),
        plan.cases,
        plan.steps,
        if plan.wasm.is_some() {
            " wasm=differential"
        } else {
            ""
        }
    );
    let records = harness::run(&plan, &selected);
    let results = props::verdict(&names, plan.wasm.is_some() && names.contains(&"game"));
    let failed = results
        .iter()
        .filter(|result| result.status == Status::Failed)
        .count();
    let unmet = results
        .iter()
        .filter(|result| result.status == Status::Unmet)
        .count();
    // Antithesis judges must-hit properties across its whole exploration, so
    // one driver invocation never fails merely for leaving some unmet.
    let allow_unmet = options.allow_unmet
        || replaying
        || options.case_seed.is_some()
        || plan.mode == Mode::Antithesis;
    let passed = failed == 0 && (allow_unmet || unmet == 0);
    for result in &results {
        if result.status != Status::Passed {
            eprintln!(
                "{:?}: [{}] {} ({} passes, {} failures) {}",
                result.status,
                result.kind,
                result.message,
                result.passes,
                result.fails,
                result.location
            );
            for signed in &result.signatures {
                eprintln!("  {} failures: {}", signed.fails, signed.signature);
            }
        }
    }
    eprintln!(
        "rebirth-dst: {} properties, {failed} failed, {unmet} unmet{} -> {}",
        results.len(),
        if allow_unmet { " (allowed)" } else { "" },
        if passed { "pass" } else { "fail" }
    );
    let census = names.contains(&"game").then(coverage::report);
    if let Some(census) = &census {
        eprintln!(
            "rebirth-dst coverage: {} game cases; NPC kinds never simulated: {} (and {} that no authored room places); simulated but never removed: {}; room types on floors never entered: {}",
            census["gameCases"], census["unreachedNpcKinds"], census["unplacedNpcKinds"], census["unremovedNpcKinds"], census["unenteredRoomTypes"]
        );
    }
    if let Some(path) = &options.report {
        let report = json!({
            "schema": 1,
            "status": if passed { "pass" } else { "fail" },
            "entropy": plan.mode.to_string(),
            "masterSeed": format!("0x{master_seed:016x}"),
            "units": names,
            "casesPerUnit": plan.cases,
            "steps": plan.steps,
            "wasmDifferential": options.wasm.as_ref().map(|path| path.display().to_string()),
            "allowUnmet": allow_unmet,
            "cases": records,
            "retainedTapes": harness::retained_tapes(),
            "properties": results,
            "coverage": census,
        });
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(error) = std::fs::write(
            path,
            serde_json::to_string_pretty(&report).expect("serializable report") + "\n",
        ) {
            eprintln!("cannot write report: {error}");
            return ExitCode::from(2);
        }
    }
    if passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
