#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
source cloud/env.sh
for tool in node npm python3 cargo clang xvfb-run xdotool; do
  command -v "$tool" >/dev/null || { echo "Missing tool: $tool" >&2; exit 1; }
done
.venv/bin/python -c 'import PIL, numpy, lxml.etree, capstone, macholib, elftools; print("Python asset and binary libraries: ready")'
python3 tools/check-assets.py
python3 tools/original/check-dependencies.py
npm run check

