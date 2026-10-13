#!/usr/bin/env python3
"""Check the real launcher library path; never disassemble original game code."""
import json
import os
from pathlib import Path
import subprocess
root = Path(__file__).resolve().parents[2]
game = root / 'references/isaac/linux/depot_250903'
env = dict(os.environ)
env['LD_LIBRARY_PATH'] = str(game / 'lib64') + (':' + env['LD_LIBRARY_PATH'] if env.get('LD_LIBRARY_PATH') else '')
result = subprocess.run(['ldd', str(game / 'isaac.x64')], env=env, capture_output=True, text=True)
output = root / '.cache/original-runtime'
output.mkdir(parents=True, exist_ok=True)
(output / 'game-dependencies.txt').write_text(result.stdout + result.stderr)
missing = [line.strip() for line in result.stdout.splitlines() if 'not found' in line]
print(json.dumps({'nativeDependenciesResolved': result.returncode == 0 and not missing, 'missing': missing}))
if result.returncode or missing:
    raise SystemExit(1)
