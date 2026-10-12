#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
bash cloud/managed.sh bash -c 'bash tools/original-linux.sh display; bash cloud/start.sh'
curl --fail --silent http://127.0.0.1:5173/ > .cache/dev-readiness.html
rg -q 'Isaac' .cache/dev-readiness.html
echo 'Virtual display and browser development server ready. Original launch requires an authentic Steam client/session; see docs/original-linux.md.'
