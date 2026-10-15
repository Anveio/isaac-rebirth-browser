#!/usr/bin/env bash
set -euo pipefail
# Preserve the executor's proxy and certificate settings.
if ! command -v rustup >/dev/null && [[ ! -f "$HOME/.cargo/env" ]]; then
  task_rust_installer="$(mktemp)"
  trap 'rm -f "$task_rust_installer"' EXIT
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error https://sh.rustup.rs -o "$task_rust_installer"
  sh "$task_rust_installer" -y --profile minimal --default-toolchain stable
fi
if ! command -v rustup >/dev/null; then source "$HOME/.cargo/env"; fi
rustup target add wasm32-unknown-unknown
# llvm-tools: llvm-profdata and llvm-cov for `npm run test:e2e -- --affected`.
rustup component add rustfmt clippy llvm-tools
