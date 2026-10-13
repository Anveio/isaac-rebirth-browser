#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source cloud/env.sh
[[ -x references/isaac/linux/depot_250903/isaac.x64 ]] || python3 tools/restore-linux-depot.py
python3 tools/original/check-dependencies.py
mkdir -p .cache
npm run build:wasm
if curl --fail --silent http://127.0.0.1:5173/ >/dev/null; then
  echo 'Browser development server is ready on port 5173. Offline reference: bash tools/original/run-offline.sh.'
  exit 0
fi
nohup npm run dev > .cache/vite.log 2>&1 < /dev/null &
echo "$!" > .cache/vite.pid
for attempt in {1..30}; do
  if curl --fail --silent http://127.0.0.1:5173/ >/dev/null; then
    echo 'Browser development server is ready on port 5173. Offline reference: bash tools/original/run-offline.sh.'
    exit 0
  fi
  sleep 1
done
tail -40 .cache/vite.log >&2
exit 1

