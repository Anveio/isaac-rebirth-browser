#!/usr/bin/env bash
# Antithesis DST with seeded entropy: builds the harness, runs every unit and
# replays the corpus tapes through the game unit.
# DST_CASES/DST_STEPS size the run; extra arguments pass to rebirth-dst
# (e.g. --seed 0x… or --case-seed … --units game to reproduce a case).
set -euo pipefail
cd "$(dirname "$0")/../.."
if ! command -v cargo >/dev/null; then source "$HOME/.cargo/env"; fi
cargo build --manifest-path engine/dst/Cargo.toml --locked --release
output="${DST_OUTPUT:-.cache/dst/run-$(date +%s)-$$}"
mkdir -p "$output"
echo "DST report and retained tapes: $output"
exec engine/dst/target/release/rebirth-dst --units all \
  --cases "${DST_CASES:-12}" --steps "${DST_STEPS:-1500}" \
  --corpus engine/dst/corpus --tapes "$output/tapes" \
  --report "$output/report.json" "$@"
