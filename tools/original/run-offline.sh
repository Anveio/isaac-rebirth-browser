#!/usr/bin/env bash
set -euo pipefail
task_root="$(cd "$(dirname "$0")/../.." && pwd)"
task_game="$task_root/.cache/original-runtime/offline-depot"
[[ -f "$task_game/provenance.json" ]] || { echo 'Run bash tools/original/prepare-offline.sh first.' >&2; exit 1; }
export ISAAC_OFFLINE_CLOUD_DIR="$task_root/.cache/original-runtime/offline-cloud"
# OpenAL's documented null backend keeps the application responsive without an audio device.
export ALSOFT_DRIVERS=null
cd "$task_game"
exec "$task_root/tools/original/with-display.sh" bash ./run-x64.sh "$@"
