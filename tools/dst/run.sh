#!/usr/bin/env bash
# Antithesis DST with seeded entropy: builds the harness and runs every unit.
# DST_CASES/DST_STEPS size the run; extra arguments pass to rebirth-dst
# (e.g. --seed 0x… or --case-seed … --units floor to reproduce a case).
set -euo pipefail
cd "$(dirname "$0")/../.."
if ! command -v cargo >/dev/null; then source "$HOME/.cargo/env"; fi
cargo build --manifest-path engine/dst/Cargo.toml --locked --release
output="${DST_OUTPUT:-.cache/dst/run-$(date +%s)-$$}"
mkdir -p "$output"
echo "DST report: $output/report.json"
exec engine/dst/target/release/rebirth-dst --units all \
  --cases "${DST_CASES:-12}" --steps "${DST_STEPS:-1500}" \
  --report "$output/report.json" "$@"
