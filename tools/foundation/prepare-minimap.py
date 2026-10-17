#!/usr/bin/env python3
"""Prepare ignored minimap2 metadata from the owner's local Rebirth archive.

The selected actor is generated locally, never embedded in this source file.
No PNG, ANM2, font, capture or other actor is extracted or copied.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import posixpath
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
ACTOR = 'gfx/ui/minimap2.anm2'
SHEET = 'gfx/ui/minimap2.png'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path,
                        default=ROOT / 'references/isaac/linux/depot_250903/resources/packed/animations.a')
    args = parser.parse_args()
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location('rebirth_animation_reader', ROOT / 'tools/foundation/extract-animations.py')
    decoder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(decoder)
    binary = decoder.load_archive(args.archive)
    reader = decoder.Reader(binary)
    selected = None
    for _ in range(reader.count(5000)):
        name = reader.u32()
        actor = decoder.actor(reader)
        if name == decoder.path_hash('resources/' + ACTOR):
            if selected is not None:
                raise ValueError('Duplicate minimap2 authored actor')
            selected = actor
    if reader.at != len(binary) or selected is None:
        raise ValueError('Incomplete base Rebirth minimap2 animation cache')
    paths = [posixpath.normpath((selected['BaseSpritesheetPath'] + '/' + sheet['Path']).replace('\\', '/'))
             .removeprefix('resources/').lower() for sheet in selected['Spritesheets']]
    if paths != [SHEET] or selected['Layers'] != [{'Id': 0, 'SpritesheetId': 0, 'Name': 'main'}]:
        raise ValueError('Minimap2 requires an unadmitted spritesheet or layer')
    image = (ROOT / 'references/isaac/rebirth/resources' / SHEET).read_bytes()
    if image[:16] != b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR':
        raise ValueError('Existing minimap2 PNG is invalid')
    width, height = struct.unpack('>II', image[16:24])
    names = set()
    for clip in selected['Animations']:
        if clip['Name'] in names or clip['FrameNum'] <= 0:
            raise ValueError('Invalid minimap2 clip identity or duration')
        names.add(clip['Name'])
        for layer in clip['LayerAnimations']:
            if layer['LayerId'] != 0:
                raise ValueError('Unadmitted minimap2 animation layer')
            for frame in layer['Frames']:
                x, y, w, h = (frame[key] for key in ['XCrop', 'YCrop', 'Width', 'Height'])
                if min(x, y, w, h) < 0 or x + w > width or y + h > height:
                    raise ValueError('Minimap2 frame exceeds the existing PNG')
    if not {'RoomOutline', 'RoomVisited', 'RoomUnvisited', 'RoomCurrent', 'IconLockedRoom'}.issubset(names):
        raise ValueError('Missing minimap2 presentation clips')
    output = ROOT / 'web/generated/minimap-data.json'
    encoded = (json.dumps({ACTOR: selected}, separators=(',', ':')) + '\n').encode()
    output.parent.mkdir(parents=True, exist_ok=True)
    # Replace the generated path; never rewrite a possibly shared inode.
    temporary = output.with_suffix('.json.tmp')
    with temporary.open('xb') as stream:
        stream.write(encoded)
    temporary.replace(output)
    print(f'Prepared {ACTOR} locally ({len(names)} clips); existing PNG {width}x{height}')


if __name__ == '__main__':
    main()
