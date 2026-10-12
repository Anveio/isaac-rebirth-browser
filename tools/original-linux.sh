#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
root="$PWD"
export DISPLAY="${DISPLAY:-:99}" LIBGL_ALWAYS_SOFTWARE=1 ALSOFT_DRIVERS=null
mkdir -p .cache/original-linux
case "${1:-start}" in
  display)
    if ! xdpyinfo >/dev/null 2>&1; then
      nohup Xvfb "$DISPLAY" -screen 0 1280x720x24 -nolisten tcp > .cache/original-linux/xvfb.log 2>&1 < /dev/null &
      echo "$!" > .cache/original-linux/xvfb.pid
    fi
    for attempt in {1..30}; do
      if xdpyinfo >/dev/null 2>&1; then glxinfo -B; exit 0; fi
      sleep 1
    done
    cat .cache/original-linux/xvfb.log >&2; exit 1 ;;
  start)
    bash "$0" display
    depot="$root/references/isaac/linux/depot_250903"
    test -f "$depot/isaac.x64"
    LD_LIBRARY_PATH="$depot/lib64" ldd "$depot/isaac.x64" > .cache/original-linux/ldd.txt
    if rg -q 'not found' .cache/original-linux/ldd.txt; then cat .cache/original-linux/ldd.txt; exit 1; fi
    if ! xdotool search --onlyvisible --name 'Binding of Isaac' >/dev/null 2>&1; then
      cd "$depot"
      nohup bash ./run-x64.sh > "$root/.cache/original-linux/game.log" 2>&1 < /dev/null &
      game_pid="$!"
      echo "$game_pid" > "$root/.cache/original-linux/game.pid"
    fi
    for attempt in {1..30}; do
      if xdotool search --onlyvisible --name 'Binding of Isaac' > "$root/.cache/original-linux/window" 2>/dev/null; then exit 0; fi
      if [[ -n "${game_pid:-}" ]] && { ! kill -0 "$game_pid" 2>/dev/null || [[ "$(ps -p "$game_pid" -o stat=)" == Z* ]]; }; then
        status=0; wait "$game_pid" || status=$?
        cat "$root/.cache/original-linux/game.log" >&2
        echo "Original launch exited $status before a visible game window." >&2
        if [[ "$status" == 0 ]]; then status=1; fi
        exit "$status"
      fi
      sleep 1
    done
    cat "$root/.cache/original-linux/game.log" >&2; exit 1 ;;
  capture)
    xdotool search --onlyvisible --name 'Binding of Isaac' >/dev/null
    import -display "$DISPLAY" -window root "${2:-.cache/original-linux/screen.png}" ;;
  key)
    window="$(head -1 .cache/original-linux/window)"
    xdotool windowfocus --sync "$window"
    xdotool key --clearmodifiers "${@:2}" ;;
  exercise)
    bash "$0" start
    bash "$0" capture .cache/original-linux/before.png
    bash "$0" key "${2:-Return}"
    sleep 2
    bash "$0" capture .cache/original-linux/after.png ;;
  *) echo 'Usage: original-linux.sh display|start|capture [path]|key KEY...|exercise [KEY]' >&2; exit 2 ;;
esac
