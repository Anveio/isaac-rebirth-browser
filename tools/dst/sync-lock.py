#!/usr/bin/env python3
"""Pin the DST harness lockfile to the engine's own dependency versions.

engine/dst keeps its own Cargo.lock so the shipped engine's lockfile never
names the Antithesis SDK. The engine's dependency closure inside it must still
be exactly what engine/Cargo.lock ships: otherwise DST simulates a differently
built engine, and an engine dependency change breaks every --locked harness
build. Run before building the harness (tools/dst/run.sh, npm run lint:dst):
it resolves and pins each engine package the harness lock lacks, then names
the lock change to commit.
"""
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ENGINE_LOCK = ROOT / 'engine/Cargo.lock'
HARNESS_LOCK = ROOT / 'engine/dst/Cargo.lock'
HARNESS = ROOT / 'engine/dst/Cargo.toml'


def packages(lock):
    return {(package['name'], package['version'])
            for package in tomllib.loads(lock.read_text())['package']}


def unpinned():
    harness = packages(HARNESS_LOCK)
    return sorted(package for package in packages(ENGINE_LOCK) if package not in harness)


def cargo(*args):
    subprocess.run(['cargo', *args, '--manifest-path', str(HARNESS), '--quiet'],
                   check=True, stdout=subprocess.DEVNULL)


def main():
    changed = unpinned()
    if not changed:
        return 0
    # Resolve new engine dependencies into the harness lock, then pin each one
    # to the version the engine ships.
    cargo('metadata', '--format-version', '1')
    for name, version in unpinned():
        locked = sorted(v for n, v in packages(HARNESS_LOCK) if n == name)
        if len(locked) != 1:
            print(f'sync-lock: engine/dst/Cargo.lock holds {len(locked)} versions of {name}; '
                  f'pin {name} {version} by hand', file=sys.stderr)
            return 1
        cargo('update', '-p', f'{name}@{locked[0]}', '--precise', version)
    remaining = unpinned()
    if remaining:
        print(f'sync-lock: could not pin {remaining} in engine/dst/Cargo.lock', file=sys.stderr)
        return 1
    pinned = ', '.join(f'{name} {version}' for name, version in changed)
    print(f'sync-lock: engine/dst/Cargo.lock now pins the engine\'s {pinned}; '
          'commit it with your change.', file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())
