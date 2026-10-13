#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
source cloud/env.sh
mkdir -p .cache/original-runtime
if command -v uv >/dev/null; then
  uv pip install --python .venv/bin/python -r tools/original/compat/requirements.txt
else
  .venv/bin/python -m pip install -r tools/original/compat/requirements.txt
fi
python3 tools/original/compat/fetch-sdk.py
python3 tools/original/compat/generate.py
clang++ -std=c++17 -shared -fPIC -O2 -I .cache/steam-sdk-132/public/steam \
  .cache/original-runtime/offline-steam.cpp -o .cache/original-runtime/offline-steam.so
.venv/bin/python tools/original/compat/unwrap.py
