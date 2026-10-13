#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if ! command -v cargo >/dev/null; then source "$HOME/.cargo/env"; fi
cargo build --manifest-path engine/Cargo.toml --locked --release --target wasm32-unknown-unknown --lib
mkdir -p web/generated
cp engine/target/wasm32-unknown-unknown/release/rebirth_cleanroom.wasm web/generated/engine.wasm
