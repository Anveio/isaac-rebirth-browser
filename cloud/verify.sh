#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
for tool in node npm python3 ffmpeg clang llvm-objdump cmake ninja emcc wasm-opt; do
  command -v "$tool" >/dev/null || { echo "Missing tool: $tool" >&2; exit 1; }
done
.venv/bin/python -c 'import PIL, numpy, lxml.etree, capstone, macholib, elftools; print("Python asset and binary libraries: ready")'
python3 tools/check-assets.py
llvm-objdump --file-headers references/isaac/native/isaac-rebirth.macho
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
CI=1 npm run check

