#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p .cache
if curl --fail --silent http://127.0.0.1:5173/ >/dev/null; then
  echo 'Development server is ready on port 5173.'
  exit 0
fi
nohup npm run dev > .cache/vite.log 2>&1 < /dev/null &
echo "$!" > .cache/vite.pid
for attempt in {1..30}; do
  if curl --fail --silent http://127.0.0.1:5173/ >/dev/null; then
    echo 'Development server is ready on port 5173.'
    exit 0
  fi
  sleep 1
done
tail -40 .cache/vite.log >&2
exit 1

