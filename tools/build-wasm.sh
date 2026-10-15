#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if ! command -v cargo >/dev/null; then source "$HOME/.cargo/env"; fi
target_directory="$(cargo metadata --manifest-path engine/Cargo.toml --locked --no-deps --format-version 1 \
  | node -p 'JSON.parse(require("node:fs").readFileSync(0, "utf8")).target_directory')"
cargo build --manifest-path engine/Cargo.toml --locked --release --target wasm32-unknown-unknown --lib
mkdir -p web/generated
cp "$target_directory/wasm32-unknown-unknown/release/rebirth_cleanroom.wasm" web/generated/engine.wasm
