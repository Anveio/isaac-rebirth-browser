#!/usr/bin/env bash
set -euo pipefail
task_root="$(cd "$(dirname "$0")/../.." && pwd)"
task_runtime="$task_root/.cache/original-runtime/root"
if [[ $# == 0 ]]; then echo 'Usage: with-display.sh COMMAND [ARGS...]' >&2; exit 2; fi
if [[ -x "$task_runtime/usr/bin/Xvfb" ]]; then
  export PATH="$task_runtime/usr/bin:$PATH"
  export LD_LIBRARY_PATH="$task_runtime/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
elif ! command -v Xvfb >/dev/null || ! command -v xvfb-run >/dev/null; then
  echo 'Run python3 tools/original/setup-runtime.py first.' >&2
  exit 1
fi
export LIBGL_ALWAYS_SOFTWARE=1
exec xvfb-run --auto-servernum --server-args='-screen 0 1280x720x24 -nolisten tcp +extension GLX +render -noreset' "$@"
