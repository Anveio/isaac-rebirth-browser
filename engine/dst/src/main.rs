//! Antithesis deterministic simulation testing for the Rust/WASM engine core.
//!
//! `rebirth-dst` drives logical engine units with entropy-chosen inputs and
//! checks Antithesis properties after every step. Locally a seeded stream
//! makes every case reproducible; under Antithesis (`--entropy antithesis`)
//! the platform supplies and steers every decision. See `docs/dst.md`.
mod entropy;
mod harness;
mod props;
mod units;

use entropy::Mode;
use harness::Plan;
use props::Status;
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: rebirth-dst [--units all|a,b] [--cases N] [--steps N] [--seed U64]
                   [--case-seed U64] [--entropy seeded|antithesis] [--report PATH]
                   [--allow-unmet] [--list] [--setup-complete]";

struct Options {
    units: Vec<String>,
    cases: u64,
    steps: u32,
    seed: Option<u64>,
    case_seed: Option<u64>,
    mode: Mode,
    report: Option<PathBuf>,
    allow_unmet: bool,
    list: bool,
    setup_complete: bool,
}

fn parse() -> Result<Options, String> {
    let mut options = Options {
        units: vec!["all".into()],
        cases: 8,
        steps: 1500,
        seed: None,
        case_seed: None,
        mode: Mode::Seeded,
        report: None,
        allow_unmet: false,
        list: false,
        setup_complete: false,
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
            "--report" => options.report = Some(value()?.into()),
            "--allow-unmet" => options.allow_unmet = true,
            "--list" => options.list = true,
            "--setup-complete" => options.setup_complete = true,
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

fn os_seed() -> u64 {
    use std::io::Read;
    let mut bytes = [0u8; 8];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .expect("read OS entropy for the master seed");
    u64::from_le_bytes(bytes)
}

fn main() -> ExitCode {
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
    let names: Vec<&str> = if options.units.iter().any(|unit| unit == "all") {
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
    let master_seed = options.seed.unwrap_or_else(os_seed);
    let plan = Plan {
        mode: options.mode,
        master_seed,
        case_seed: options.case_seed,
        cases: if options.case_seed.is_some() {
            1
        } else {
            options.cases
        },
        steps: options.steps,
    };
    eprintln!(
        "rebirth-dst: entropy={} seed=0x{master_seed:016x} units={} cases={} steps={}",
        plan.mode,
        names.join(","),
        plan.cases,
        plan.steps,
    );
    let records = harness::run(&plan, &selected);
    let results = props::verdict(&names);
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
    let allow_unmet =
        options.allow_unmet || options.case_seed.is_some() || plan.mode == Mode::Antithesis;
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
        }
    }
    eprintln!(
        "rebirth-dst: {} properties, {failed} failed, {unmet} unmet{} -> {}",
        results.len(),
        if allow_unmet { " (allowed)" } else { "" },
        if passed { "pass" } else { "fail" }
    );
    if let Some(path) = &options.report {
        let report = json!({
            "schema": 1,
            "status": if passed { "pass" } else { "fail" },
            "entropy": plan.mode.to_string(),
            "masterSeed": format!("0x{master_seed:016x}"),
            "units": names,
            "casesPerUnit": plan.cases,
            "steps": plan.steps,
            "allowUnmet": allow_unmet,
            "cases": records,
            "properties": results,
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
