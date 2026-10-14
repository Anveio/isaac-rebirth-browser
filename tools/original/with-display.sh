#!/usr/bin/env bash
set -euo pipefail
task_root="$(cd "$(dirname "$0")/../.." && pwd)"
task_runtime="$task_root/.cache/original-runtime/root"
if [[ $# == 0 ]]; then echo 'Usage: with-display.sh COMMAND [ARGS...]' >&2; exit 2; fi
if [[ -x "$task_runtime/usr/bin/Xvfb" ]]; then
  export PATH="$task_runtime/usr/bin:$PATH"
  export LD_LIBRARY_PATH="$task_runtime/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
elif ! command -v Xvfb >/dev/null; then
  echo 'Run python3 tools/original/setup-runtime.py first.' >&2
  exit 1
fi
export LIBGL_ALWAYS_SOFTWARE=1
# Bound llvmpipe's worker pool on shared runners; preserve an explicit caller
# setting. This changes host scheduling, never the released presentation tape.
if [[ -z ${LP_NUM_THREADS+x} ]]; then export LP_NUM_THREADS=2; fi
# Xvfb picks and reports a free display itself (-displayfd), so concurrent
# suites never race for the same server number.
display_file="$(mktemp)"
Xvfb -displayfd 3 -screen 0 1280x720x24 -nolisten tcp +extension GLX +render -noreset \
  3>"$display_file" >/dev/null 2>&1 &
xvfb_pid=$!
trap 'kill "$xvfb_pid" 2>/dev/null; wait "$xvfb_pid" 2>/dev/null; rm -f "$display_file"' EXIT
for _ in $(seq 200); do
  [[ -s "$display_file" ]] && break
  kill -0 "$xvfb_pid" 2>/dev/null || { echo 'Xvfb failed to start' >&2; exit 1; }
  sleep 0.05
done
[[ -s "$display_file" ]] || { echo 'Xvfb did not report a display' >&2; exit 1; }
export DISPLAY=":$(head -n1 "$display_file")"
"$@"
