#!/usr/bin/env bash
set -euo pipefail
umask 077
task_root="$(cd "$(dirname "$0")/../../.." && pwd)"
source "$task_root/cloud/env.sh"
task_headers="$task_root/.cache/steam-sdk-132/public/steam"
if [[ ! -f "$task_headers/steam_api.h" ]]; then
  echo 'Missing public ABI headers. Run python3 tools/original/compat/fetch-sdk.py.' >&2
  exit 1
fi
task_host_test="$(mktemp -d "${TMPDIR:-/tmp}/isaac-host-test.XXXXXXXX")"
trap 'rm -rf "$task_host_test"' EXIT
python3 "$task_root/tools/original/compat/generate.py" --headers "$task_headers" --output "$task_host_test/offline-steam.cpp" >/dev/null
clang++ -std=c++17 -shared -fPIC -O1 -g \
  -I "$task_headers" "$task_host_test/offline-steam.cpp" -o "$task_host_test/liboffline-host.so"
clang++ -std=c++17 -O1 -g \
  -I "$task_headers" "$task_root/tools/original/compat/host-test.cpp" \
  -L "$task_host_test" -loffline-host -Wl,-rpath,"$task_host_test" -o "$task_host_test/host-test"
export ISAAC_OFFLINE_CLOUD_DIR="$task_host_test/private-cloud"
for task_phase in write read reset-read; do
  "$task_host_test/host-test" "$task_phase" | tee -a "$task_host_test/results.jsonl"
done
python3 - "$task_host_test/results.jsonl" "$ISAAC_OFFLINE_CLOUD_DIR/.offline-host-stats" <<'PY'
import json
from pathlib import Path
import stat
import sys
phases=[json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert [phase['phase'] for phase in phases] == ['write','read','reset-read']
assert all(phase['passed'] for phase in phases)
assert stat.S_IMODE(Path(sys.argv[2]).stat().st_mode) == 0o600
print(json.dumps({'scenario':'offline-host-persistence','passed':True,'processes':len(phases),
    'assertions':sum(phase['assertions'] for phase in phases)+1,
    'checks':['typed-stat-roundtrip','achievement-roundtrip','reset','callbacks','local-file-read-write','explicit-store','private-file'],
    'steamAccountParity':False}))
PY
