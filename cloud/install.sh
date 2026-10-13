#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Preserve inherited proxy, certificate, authentication and network configuration.
[[ "$(uname -s)" == Linux ]] || { echo 'Linux required.' >&2; exit 1; }
[[ "$(node -p 'process.versions.node.split(".")[0]')" == 24 ]] || { echo 'Node 24 required.' >&2; exit 1; }
if [[ "$(id -u)" == 0 ]] || { command -v sudo >/dev/null && sudo -n true; }; then
  task_root=()
  [[ "$(id -u)" == 0 ]] || task_root=(sudo -n)
  "${task_root[@]}" apt-get update
  "${task_root[@]}" apt-get install -y --no-install-recommends \
    ca-certificates curl git file unzip zip xz-utils \
    python3 python3-venv python3-dev ffmpeg \
    build-essential clang llvm lldb binutils cmake ninja-build pkg-config \
    emscripten binaryen xvfb xauth xdotool mesa-utils \
    libgl1 libgl1-mesa-dri libglu1-mesa libx11-6 libxxf86vm1 \
    libxrandr2 libxi6 libopenal1
else
  echo 'No root/sudo: installing workspace-local Debian 13 toolchain.'
  python3 tools/install-portable-tools.py
fi
python3 tools/original/setup-runtime.py
# Never reuse a venv copied from another Python version.
if [[ -x .venv/bin/python ]] && ! .venv/bin/python -c 'import sys; assert sys.version_info[:2] == tuple(map(int,sys.argv[1].split(".")))' "$(python3 -c 'import sys;print(f"{sys.version_info.major}.{sys.version_info.minor}")')"; then
  mv .venv ".venv.previous-$(date +%s)"
fi
if command -v uv >/dev/null; then
  [[ -x .venv/bin/python ]] || uv venv --python python3 .venv
  uv pip install --python .venv/bin/python -r cloud/requirements.txt
else
  python3 -m venv .venv
  .venv/bin/python -m pip install --disable-pip-version-check -r cloud/requirements.txt
fi
bash tools/setup-rust.sh
source cloud/env.sh
npm ci --no-audit --no-fund
if [[ ! -x /usr/bin/chromium ]] && [[ -z "${PLAYWRIGHT_CHROMIUM_EXECUTABLE:-}" ]]; then
  npx playwright install chromium
fi
python3 tools/restore-linux-depot.py
bash tools/original/prepare-offline.sh
npm run typecheck
npm run build
python3 tools/original/check-dependencies.py
bash tools/original/with-display.sh python3 tools/original/probe.py
echo 'Dependencies, WASM build and display are ready. Run npm run check for all game verification scenarios.'
