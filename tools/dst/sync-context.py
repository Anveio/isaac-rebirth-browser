#!/usr/bin/env python3
"""Keep the Antithesis image's build context holding every engine build input.

antithesis/Dockerfile copies its whole build context, which
antithesis/Dockerfile.dockerignore limits to the engine build's inputs so the
image never carries local build output or unrelated private material. Those
inputs are whatever the engine and the harness read at compile time: their
sources, `include_str!`/`include_bytes!` files and every path build.rs reads.
Cargo records them in the dep-info files of the two builds the image runs.
Run after building both (tools/dst/run.sh): it re-includes each recorded
input the context drops, then names the change to commit.
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
IGNORE = ROOT / 'antithesis/Dockerfile.dockerignore'


def target_directory(manifest):
    metadata = subprocess.run(
        ['cargo', 'metadata', '--manifest-path', str(ROOT / manifest), '--locked', '--no-deps',
         '--format-version', '1'], check=True, capture_output=True, text=True).stdout
    return Path(json.loads(metadata)['target_directory'])


def dependencies(dep_info):
    """The paths a Makefile-style dep-info file names, with escaped spaces kept."""
    text = dep_info.read_text()
    paths = set()
    for line in text.splitlines():
        if not line or line.startswith('#'):
            continue
        _, _, listed = line.partition(': ')
        for path in re.split(r'(?<!\\) ', listed.strip()):
            if path:
                paths.add(Path(os.path.normpath(path.replace('\\ ', ' '))))
    return paths


def rule(pattern):
    """A .dockerignore line as (re-included, regex), with Go filepath.Match plus `**`."""
    negated = pattern.startswith('!')
    pattern = os.path.normpath(pattern.removeprefix('!').strip('/'))
    regex = ''
    index = 0
    while index < len(pattern):
        character = pattern[index]
        if pattern.startswith('**', index):
            regex += '.*'
            index += 2
            continue
        if character == '*':
            regex += '[^/]*'
        elif character == '?':
            regex += '[^/]'
        elif character == '\\' and index + 1 < len(pattern):
            index += 1
            regex += re.escape(pattern[index])
        else:
            regex += re.escape(character)
        index += 1
    return negated, re.compile(regex)


def in_context(path, rules):
    """Docker keeps a path unless the last rule matching it or a parent excludes it."""
    parts = path.split('/')
    prefixes = ['/'.join(parts[:length]) for length in range(1, len(parts) + 1)]
    kept = True
    for negated, regex in rules:
        if any(regex.fullmatch(prefix) for prefix in prefixes):
            kept = negated
    return kept


def main():
    engine_target = target_directory('engine/Cargo.toml')
    harness_target = target_directory('engine/dst/Cargo.toml')
    dep_infos = [engine_target / 'wasm32-unknown-unknown/release/rebirth_cleanroom.d',
                 harness_target / 'release/rebirth-dst.d']
    inputs = set()
    for dep_info in dep_infos:
        if not dep_info.exists():
            print(f'sync-context: {dep_info} is missing; build the engine WASM and the harness first',
                  file=sys.stderr)
            return 1
        inputs |= dependencies(dep_info)
    # Generated files (build.rs output) live in the target directories, which
    # the image builds itself; dependencies outside the repository come from
    # the registry.
    inputs = sorted(path.relative_to(ROOT).as_posix() for path in inputs
                    if path.is_relative_to(ROOT)
                    and not path.is_relative_to(engine_target) and not path.is_relative_to(harness_target))
    lines = IGNORE.read_text().splitlines()
    rules = [rule(line) for line in lines if line.strip() and not line.startswith('#')]
    dropped = [path for path in inputs if not in_context(path, rules)]
    if not dropped:
        return 0
    escaped = ['!' + re.sub(r'([*?\[\\])', r'\\\1', path) for path in dropped]
    IGNORE.write_text('\n'.join(lines + escaped) + '\n')
    print(f'sync-context: {IGNORE.relative_to(ROOT)} now keeps the engine build\'s '
          f'{", ".join(dropped)}; commit it with your change.', file=sys.stderr)
    return 0


if __name__ == '__main__':
    sys.exit(main())
