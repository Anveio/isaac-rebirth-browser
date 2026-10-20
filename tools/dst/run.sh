#!/usr/bin/env bash
# Antithesis DST with seeded entropy: builds the production engine.wasm and the
# harness, keeps the Antithesis image's build context holding their inputs,
# runs every unit, replays the corpus tapes through the game unit, plays the
# pinned rooms through the rooms unit and checks every game tape against the
# shipped WASM.
# DST_CASES/DST_STEPS size the run; extra arguments pass to rebirth-dst
# (e.g. --seed 0x… or --case-seed … --units game to reproduce a case).
set -euo pipefail
cd "$(dirname "$0")/../.."
if ! command -v cargo >/dev/null; then source "$HOME/.cargo/env"; fi
bash tools/build-wasm.sh
python3 tools/dst/sync-lock.py
cargo build --manifest-path engine/dst/Cargo.toml --locked --release
python3 tools/dst/sync-context.py
# CARGO_TARGET_DIR or Cargo configuration may move the build output.
target_directory="$(cargo metadata --manifest-path engine/dst/Cargo.toml --locked --no-deps --format-version 1 \
  | node -p 'JSON.parse(require("node:fs").readFileSync(0, "utf8")).target_directory')"
output="${DST_OUTPUT:-.cache/dst/run-$(date +%s)-$$}"
mkdir -p "$output"
echo "DST report and retained tapes: $output"
exec "$target_directory/release/rebirth-dst" --units all \
  --cases "${DST_CASES:-12}" --steps "${DST_STEPS:-1500}" \
  --wasm web/generated/engine.wasm --corpus engine/dst/corpus \
  --pins engine/dst/pinned-rooms.json \
  --tapes "$output/tapes" \
  --report "$output/report.json" "$@"
