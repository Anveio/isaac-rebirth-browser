#!/usr/bin/env python3
"""Install portable Debian 13 display tools without modifying the host OS."""
import hashlib
import json
import pathlib
import platform
import shutil
import subprocess
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
RUNTIME = ROOT / '.cache/original-runtime'
if all(shutil.which(tool) for tool in ['Xvfb', 'xvfb-run', 'xdotool', 'glxinfo', 'glxgears']):
    print('System display/input/OpenGL tools available.')
    raise SystemExit(0)
if platform.machine() != 'x86_64' or 'VERSION_CODENAME=trixie' not in pathlib.Path('/etc/os-release').read_text():
    raise SystemExit('This portable package set requires Debian 13 (trixie), x86_64.')
packages = json.loads(pathlib.Path(__file__).with_name('runtime-packages.json').read_text())
for package in packages.values():
    path = RUNTIME / 'debs' / pathlib.Path(package['Filename']).name
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or hashlib.sha256(path.read_bytes()).hexdigest() != package['SHA256']:
        with urllib.request.urlopen('https://deb.debian.org/debian/' + package['Filename'], timeout=60) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != package['SHA256']:
            raise SystemExit(f"SHA256 mismatch for {package['Package']}")
        path.write_bytes(data)
    subprocess.run(['dpkg-deb', '-x', str(path), str(RUNTIME / 'root')], check=True)
    print(f"Ready: {package['Package']} {package['Version']}")
