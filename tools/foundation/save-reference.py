#!/usr/bin/env python3
"""Save observed original PNGs and run metadata as portable private test fixtures."""
import argparse
import json
from pathlib import Path
import shutil

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('manifest', type=Path)
parser.add_argument('destination', type=Path)
args = parser.parse_args()
source = args.manifest.resolve()
manifest = json.loads(source.read_text())
if manifest.get('status') != 'pass' or not manifest.get('profile', {}).get('fresh') or not manifest.get('profile', {}).get('isolated'):
    raise SystemExit('Only successful isolated original captures can become reference fixtures')
if manifest.get('initialState', {}).get('seed') != 'T1PK 6Y0W':
    raise SystemExit('This foundation reference requires the observed T1PK 6Y0W seed')
output = args.destination.resolve()
(output / 'frames').mkdir(parents=True, exist_ok=True)
frames = []
for frame in manifest['frames']:
    original = Path(frame['path'])
    if not original.is_absolute():
        original = source.parent / original
    image = output / 'frames' / f"{frame['label']}.png"
    shutil.copyfile(original, image)
    frames.append({**frame, 'path': f"frames/{image.name}"})
portable = {key: value for key, value in manifest.items() if key not in ('output', 'video')}
portable.update({'source': 'offline-derived-original', 'captureId': manifest['output'], 'frames': frames})
(output / 'manifest.json').write_text(json.dumps(portable, indent=2) + '\n')
print(json.dumps({'reference': str(output), 'frames': len(frames)}))
