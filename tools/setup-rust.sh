#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# rust-toolchain.toml pins the compiler, rustfmt, clippy and the wasm target.
task_toolchain="$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)"
[[ -n "$task_toolchain" ]] || { echo 'rust-toolchain.toml names no channel.' >&2; exit 1; }
# Preserve the executor's proxy and certificate settings.
if ! command -v rustup >/dev/null && [[ ! -f "$HOME/.cargo/env" ]]; then
  task_rust_installer="$(mktemp)"
  trap 'rm -f "$task_rust_installer"' EXIT
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error https://sh.rustup.rs -o "$task_rust_installer"
  sh "$task_rust_installer" -y --profile minimal --default-toolchain "$task_toolchain"
fi
if ! command -v rustup >/dev/null; then source "$HOME/.cargo/env"; fi
# Installs the pinned toolchain with its components and targets; once
# installed, builds work offline.
rustup toolchain install
rustc --version
