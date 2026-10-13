#!/usr/bin/env python3
"""Workspace-local Debian 13 toolchain; never alters host package state."""
import hashlib
import json
import pathlib
import platform
import subprocess
import urllib.request
from concurrent.futures import ThreadPoolExecutor
ROOT = pathlib.Path(__file__).resolve().parents[1]
DEST = ROOT / '.cache/toolchain'
if platform.machine() != 'x86_64' or 'VERSION_CODENAME=trixie' not in pathlib.Path('/etc/os-release').read_text():
    raise SystemExit('Portable toolchain requires Debian 13 x86_64; use root/sudo install on other hosts.')
packages = json.loads((ROOT / 'cloud/portable-packages.json').read_text())
def fetch(p):
    path = DEST / 'debs' / pathlib.Path(p['Filename']).name
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or hashlib.sha256(path.read_bytes()).hexdigest() != p['SHA256']:
        with urllib.request.urlopen('https://deb.debian.org/debian/' + p['Filename'], timeout=60) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != p['SHA256']:
            raise RuntimeError('Package checksum mismatch: ' + p['Package'])
        path.write_bytes(data)
    return p, path
with ThreadPoolExecutor(max_workers=4) as pool:
    for p, path in pool.map(fetch, packages.values()):
        subprocess.run(['dpkg-deb', '-x', str(path), str(DEST / 'root')], check=True)
        print('Ready:', p['Package'], p['Version'], flush=True)
