#!/usr/bin/env python3
"""Prepare ignored notification metadata from the owner's local Base resources.

Both the browser UI and Rust catalogue builder consume this same actor/catalogue
pair. No PNG, ANM2, XML, font or generated metadata is added to public source.
"""
import argparse
import fnmatch
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, default=ROOT / 'references/isaac/linux/depot_250903/resources/packed/animations.a')
    parser.add_argument('--filenames', type=Path, default=ROOT / 'references/isaac/resource-filenames.txt')
    parser.add_argument('--xml', type=Path, default=ROOT / 'references/isaac/rebirth/resources/achievements.xml')
    parser.add_argument('--output', type=Path, default=ROOT / 'web/generated/unlock-notifications')
    args = parser.parse_args()
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('rebirth_animation_reader', ROOT / 'tools/foundation/extract-animations.py')
    decoder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(decoder)
    filenames = args.filenames.read_text().splitlines()
    names = {decoder.path_hash(path): path.removeprefix('resources/') for path in filenames
             if path.startswith('resources/') and path.endswith('.anm2')}
    patterns = [line.strip() for line in (ROOT / 'tools/foundation/information-animations.txt').read_text().splitlines()
                if line.strip() and not line.lstrip().startswith('#')]
    if not patterns:
        raise ValueError('Information animation selection is empty')
    binary = decoder.load_archive(args.archive)
    reader = decoder.Reader(binary)
    actors = {}
    for _ in range(reader.count(5000)):
        name, actor = reader.u32(), decoder.actor(reader)
        path = names.get(name)
        if path is None:
            continue
        if path.replace('\\', '/').lower().rsplit('/', 1)[-1] == 'achievements.anm2' or any(fnmatch.fnmatchcase(path, p) for p in patterns):
            if path in actors:
                raise ValueError(f'Duplicate selected authored actor {path}')
            actors[path] = actor
    if reader.at != len(binary):
        raise ValueError('Incomplete Base animation cache')
    actual, catalogue, _ = decoder.unlock_notification_catalog(actors, filenames, args.xml)
    actor_path = catalogue['actor']
    if actual != actor_path:
        if actor_path in actors:
            raise ValueError('Normalized notification actor is ambiguous')
        actors[actor_path] = actors.pop(actual)
    for pattern in patterns:
        if not any(fnmatch.fnmatchcase(path, pattern) for path in actors):
            raise ValueError(f'Missing selected information actor {pattern}')
    args.output.mkdir(parents=True, exist_ok=True)
    for name, value in [('information-data.json', actors), ('catalogue.json', catalogue)]:
        output = args.output / name
        # Match the existing private minimap preparation convention: replace
        # this generated path without modifying a potentially shared inode.
        temporary = output.with_suffix('.json.tmp')
        with temporary.open('x') as stream:
            stream.write(json.dumps(value, separators=(',', ':')) + '\n')
        temporary.replace(output)
    print(f'Prepared private notification actors/catalogue in {args.output}')


if __name__ == '__main__':
    main()
