#!/usr/bin/env python3
"""Restore the owner's base Rebirth Linux depot from private repository parts."""
import hashlib
import json
from pathlib import Path
import tarfile
import tempfile

base = Path(__file__).resolve().parents[1] / 'references/isaac/linux'
parts_dir = base / 'transfer'
manifest = json.loads((base / 'manifest.json').read_text())
with tempfile.TemporaryFile() as archive:
    for part in json.loads((parts_dir / 'parts.json').read_text()):
        data = (parts_dir / part['path']).read_bytes()
        if len(data) != part['bytes'] or hashlib.sha256(data).hexdigest() != part['sha256']:
            raise SystemExit(f"Corrupt archive part: {part['path']}")
        archive.write(data)
    archive.seek(0)
    with tarfile.open(fileobj=archive, mode='r:gz') as tar:
        # Only regular files and directories under the expected depot are accepted.
        for member in tar.getmembers():
            path = Path(member.name)
            if path.is_absolute() or '..' in path.parts or path.parts[0] != 'depot_250903' or not (member.isfile() or member.isdir()):
                raise SystemExit(f'Unexpected archive entry: {member.name}')
            if (base / path).is_symlink():
                raise SystemExit(f'Refusing to overwrite symlink: {member.name}')
        tar.extractall(base, filter='data')
for item in manifest['files']:
    path = base / 'depot_250903' / item['path']
    if path.stat().st_size != item['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest() != item['sha256']:
        raise SystemExit(f"File verification failed: {item['path']}")
print(f"Verified {len(manifest['files'])} files: {base / 'depot_250903'}")
