#!/usr/bin/env bash
set -euo pipefail
task_root="$(cd "$(dirname "$0")/../.." && pwd)"
task_game="${1:-$task_root/references/isaac/linux/depot_250903}"
if [[ ! -f "$task_game/run-x64.sh" ]]; then
  echo "Missing Linux installation: $task_game/run-x64.sh" >&2
  echo 'Place Steam base depot 250903 here; see README.md.' >&2
  exit 2
fi
task_game="$(cd "$task_game" && pwd)"
shift "$(( $# > 0 ? 1 : 0 ))"
# Launch from the original installation directory so relative libraries/assets resolve.
# Do not change HOME, the original executable, or Steam authentication behavior.
cd "$task_game"
exec "$task_root/tools/original/with-display.sh" bash ./run-x64.sh "$@"
