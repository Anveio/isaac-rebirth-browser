#!/usr/bin/env python3
"""Verify the original resources against the recorded extraction inventory."""
import hashlib
import importlib.util
import json
from pathlib import Path
import xml.etree.ElementTree as ET

root = Path(__file__).resolve().parents[1]
manifest = json.loads((root / 'references/isaac/manifest.json').read_text())
errors = []
for entry in manifest['files']:
    path = root / entry['path']
    if not path.is_file():
        errors.append(f"Missing: {entry['path']}")
    elif path.stat().st_size != entry['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
        errors.append(f"Hash mismatch: {entry['path']}")
resources = root / 'references/isaac/rebirth/resources'
for name in ['players.xml', 'items.xml', 'entities2.xml', 'stages.xml']:
    ET.parse(resources / name)
if errors:
    raise SystemExit('\n'.join(errors[:20]))
audio_spec = importlib.util.spec_from_file_location('authored_audio_assets', root / 'tools/foundation/extract-audio.py')
audio = importlib.util.module_from_spec(audio_spec)
audio_spec.loader.exec_module(audio)
audio.complete(check_only=True)
print(f"Verified {len(manifest['files'])} original reference files and complete authored audio references.")

