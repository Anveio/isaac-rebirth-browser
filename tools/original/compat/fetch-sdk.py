#!/usr/bin/env python3
"""Fetch checksum-pinned public Steam ABI declarations; no gameplay sources."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import urllib.request
root=Path(__file__).resolve().parents[3]
manifest=json.loads(Path(__file__).with_name('sdk-manifest.json').read_text())
base=root/'.cache/steam-sdk-132'
def get(item):
    dest=base/item['path']
    if dest.exists() and hashlib.sha256(dest.read_bytes()).hexdigest()==item['sha256']:return
    url=f"https://raw.githubusercontent.com/{manifest['repository']}/{manifest['commit']}/{item['path']}"
    with urllib.request.urlopen(url,timeout=30) as stream:data=stream.read()
    if hashlib.sha256(data).hexdigest()!=item['sha256']:raise RuntimeError('SDK header checksum mismatch')
    dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
with ThreadPoolExecutor(max_workers=4) as pool:list(pool.map(get,manifest['files']))
print('Verified public Steamworks ABI headers:',len(manifest['files']))
