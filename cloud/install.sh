#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Keep the managed executor's HTTP(S) proxy and CA variables unchanged.
if [[ "$(uname -s)" != Linux ]]; then
  echo 'This installer targets the Linux cloud environment.' >&2
  exit 1
fi
node_major="$(node -p 'process.versions.node.split(".")[0]')"
if [[ "$node_major" != 24 ]]; then
  echo 'Select/install Node.js 24 before running this installer.' >&2
  exit 1
fi
run_as_root=()
if [[ "$(id -u)" != 0 ]]; then run_as_root=(sudo); fi
"${run_as_root[@]}" apt-get update
"${run_as_root[@]}" apt-get install -y --no-install-recommends \
  ca-certificates curl git file unzip zip xz-utils \
  python3 python3-venv python3-dev ffmpeg \
  build-essential clang llvm lldb binutils cmake ninja-build pkg-config \
  emscripten binaryen
python3 -m venv .venv
.venv/bin/python -m pip install --disable-pip-version-check -r cloud/requirements.txt
npm ci --no-audit --no-fund
"${run_as_root[@]}" "$(command -v node)" node_modules/playwright/cli.js install-deps chromium
npx playwright install chromium
bash cloud/verify.sh

