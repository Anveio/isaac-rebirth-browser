#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source cloud/env.sh
for tool in node npm python3 ffmpeg clang llvm-objdump cmake ninja emcc wasm-opt; do
  command -v "$tool" >/dev/null || { echo "Missing tool: $tool" >&2; exit 1; }
done
.venv/bin/python -c 'import PIL, numpy, lxml.etree, capstone, macholib, elftools; print("Python asset and binary libraries: ready")'
python3 tools/check-assets.py
llvm-objdump --macho --private-header references/isaac/native/isaac-rebirth.macho
task_wasm_dir="$(mktemp -d)"
trap 'rm -rf "$task_wasm_dir"' EXIT
echo 'int add(int a, int b) { return a + b; }' > "$task_wasm_dir/probe.c"
emcc "$task_wasm_dir/probe.c" -O2 -sSTANDALONE_WASM --no-entry -Wl,--export=add -o "$task_wasm_dir/probe.wasm"
node --input-type=module - "$task_wasm_dir/probe.wasm" <<'JS'
import fs from 'node:fs';
const { instance } = await WebAssembly.instantiate(fs.readFileSync(process.argv[2]), { wasi_snapshot_preview1: { proc_exit() {} } });
if (instance.exports.add(20, 22) !== 42) throw new Error('WASM execution failed');
console.log('C to WebAssembly compilation and execution: ready');
JS
task_project_status=0
npm run check || task_project_status=$?
python3 tools/restore-linux-depot.py
python3 tools/original/check-dependencies.py
bash tools/original/with-display.sh python3 tools/original/probe.py
if [[ "${1:-}" == --launch-original ]]; then
  bash tools/original/with-display.sh python3 tools/original/verify-launch.py
elif [[ "${1:-}" == --launch-offline ]]; then
  python3 tools/original/debug-smoke.py --output ".cache/original-runtime/offline-verified-$(date +%s)-$$"
else
  echo "Toolchain, depot and display checks passed. Project verification exit status: $task_project_status. Use --launch-offline for seeded original gameplay checks."
fi
exit "$task_project_status"

