#!/usr/bin/env python3
"""Decode structurally validated base-Rebirth STB room assets.

All 13 original base stage sets consume every byte with this little-endian
layout. Type/variant/subtype integers and grid coordinates remain authored
selectors: this tool does not infer runtime entity types or generation rules.
Afterbirth/Afterbirth+ layouts are outside this decoder's verified scope.
"""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path, PurePosixPath
import struct
import tempfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
MAX_ASSET_BYTES = 32 * 1024 * 1024
MAX_ROOMS = 100_000


class Reader:
    def __init__(self, data, source):
        if len(data) > MAX_ASSET_BYTES:
            raise ValueError(f'{source}: STB exceeds {MAX_ASSET_BYTES} bytes')
        self.data = data
        self.source = source
        self.cursor = 0

    def take(self, size):
        if size < 0 or self.cursor + size > len(self.data):
            raise ValueError(f'{self.source}: truncated STB at byte {self.cursor}; need {size} bytes')
        result = self.data[self.cursor:self.cursor + size]
        self.cursor += size
        return result

    def number(self, fmt):
        values = struct.unpack('<' + fmt, self.take(struct.calcsize('<' + fmt)))
        return values[0] if len(values) == 1 else values

    def weight(self, value, label):
        if not math.isfinite(value) or value < 0:
            raise ValueError(f'{self.source}: invalid {label} weight at byte {self.cursor}')
        return value


def decode_bytes(data, *, source='<memory>'):
    reader = Reader(data, source)
    count = reader.number('I')
    if count > MAX_ROOMS:
        raise ValueError(f'{source}: room count exceeds {MAX_ROOMS}')
    rooms = []
    for index in range(count):
        room_type, variant, difficulty, length = reader.number('IIBH')
        try:
            name = reader.take(length).decode('ascii')
        except UnicodeDecodeError as error:
            raise ValueError(f'{source}: non-ASCII base room name in record {index}') from error
        weight = reader.weight(reader.number('f'), 'room')
        width, height, door_count, spawn_count = reader.number('BBBH')
        doors = []
        for _ in range(door_count):
            x, y, enabled = reader.number('hhB')
            if enabled not in (0, 1):
                raise ValueError(f'{source}: invalid door enabled flag at byte {reader.cursor - 1}')
            doors.append({'x': x, 'y': y, 'enabled': bool(enabled)})
        spawns = []
        for _ in range(spawn_count):
            x, y, entry_count = reader.number('hhB')
            entries = []
            for _ in range(entry_count):
                spawn_type, spawn_variant, subtype, spawn_weight = reader.number('HHHf')
                reader.weight(spawn_weight, 'spawn')
                entries.append({'type': spawn_type, 'variant': spawn_variant,
                                'subtype': subtype, 'weight': spawn_weight})
            spawns.append({'x': x, 'y': y, 'entries': entries})
        rooms.append({'type': room_type, 'variant': variant, 'difficulty': difficulty,
                      'name': name, 'weight': weight, 'width': width, 'height': height,
                      'doors': doors, 'spawns': spawns})
    if reader.cursor != len(data):
        raise ValueError(f'{source}: trailing STB bytes at {reader.cursor} of {len(data)}')
    return rooms


def read_asset(path):
    if path.stat().st_size > MAX_ASSET_BYTES:
        raise ValueError(f'{path}: STB exceeds {MAX_ASSET_BYTES} bytes')
    return path.read_bytes()


def decode(path):
    return decode_bytes(read_asset(path), source=str(path))


def encode_bytes(rooms):
    """Encode the structural fields for exact reference round-trip checks."""
    output = bytearray(struct.pack('<I', len(rooms)))
    for room in rooms:
        name = room['name'].encode('ascii')
        output.extend(struct.pack('<IIBH', room['type'], room['variant'], room['difficulty'], len(name)))
        output.extend(name)
        output.extend(struct.pack('<fBBBH', room['weight'], room['width'], room['height'],
                                  len(room['doors']), len(room['spawns'])))
        for door in room['doors']:
            output.extend(struct.pack('<hhB', door['x'], door['y'], int(door['enabled'])))
        for spawn in room['spawns']:
            output.extend(struct.pack('<hhB', spawn['x'], spawn['y'], len(spawn['entries'])))
            for entry in spawn['entries']:
                output.extend(struct.pack('<HHHf', entry['type'], entry['variant'], entry['subtype'], entry['weight']))
    result = bytes(output)
    decode_bytes(result, source='encoded STB')
    return result


def base_assets(root=ROOT):
    """Read named original archive entries without modifying reference files."""
    spec = importlib.util.spec_from_file_location('room_archive_reader', root / 'tools/foundation/extract-animations.py')
    archive = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(archive)
    resources = root / 'references/isaac/rebirth/resources'
    packed = root / 'references/isaac/linux/depot_250903/resources/packed/rooms.a'
    data, compression, entries = archive.archive_entries(packed)
    manifest = json.loads((root / 'references/isaac/linux/manifest.json').read_text())
    records = [entry for entry in manifest['files'] if entry['path'] == 'resources/packed/rooms.a']
    if len(records) != 1 or records[0]['bytes'] != len(data) or records[0]['sha256'] != hashlib.sha256(data).hexdigest():
        raise ValueError('Original rooms.a does not match the pinned Linux depot manifest')
    table = {(entry[0], entry[1]): entry for entry in entries}
    declared = ET.parse(resources / 'stages.xml').getroot()
    paths = []
    for stage in declared:
        authored = stage.attrib['path'].replace('\\', '/')
        path = PurePosixPath(authored)
        if path.suffix.lower() != '.xml' or len(path.parts) != 1 or path.name in ('', '.', '..'):
            raise ValueError(f'Unsupported base stage resource path: {authored}')
        paths.append('resources/rooms/' + path.with_suffix('.stb').name.lower())
    if len(paths) != len(set(paths)):
        raise ValueError('Duplicate declared base stage room asset')
    result = []
    for path in paths:
        key = (archive.path_hash(path), archive.path_hash_b(path))
        if key not in table:
            raise ValueError(f'Original base stage asset missing: {path}')
        _, hash_b, offset, length, _checksum = table[key]
        if length > MAX_ASSET_BYTES:
            raise ValueError(f'{path}: STB exceeds {MAX_ASSET_BYTES} bytes')
        reader = archive.Reader(data)
        reader.at = offset
        if compression == 0:
            contents = archive.read_bogocrypt(reader, length, hash_b)
        elif compression == 1:
            contents = archive.decompress_lzw(reader, length)
        else:
            raise ValueError(f'Unsupported base room archive compression: {compression}')
        result.append((path, contents))
    return result


# stages.xml room sets the engine embeds from the owner archive: Caves (3) and
# Depths (5). Basement and the special rooms are checked in.
PRIVATE_ROOM_SETS = ('03.caves.stb', '05.depths.stb')


def prepare_private_caves(root=ROOT):
    """Derive only the private room sets in this Source tree's ignored build cache."""
    assets = base_assets(root)
    for name in PRIVATE_ROOM_SETS:
        prepare_private_room_set(root, assets, name)


def prepare_private_room_set(root, assets, name):
    matches = [contents for path, contents in assets if path == f'resources/rooms/{name}']
    if len(matches) != 1:
        raise ValueError(f'Pinned base archive must contain exactly one {name} room set')
    contents = matches[0]
    target = root / f'.cache/private-base-assets/rooms/{name}'

    def verify():
        if not target.is_file() or read_asset(target) != contents:
            raise ValueError(f'Private {name} cache does not match the pinned base archive')

    if target.exists() or target.is_symlink():
        verify()
        return
    target.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f'.{name}-', dir=target.parent)
    try:
        with os.fdopen(descriptor, 'wb') as stream:
            stream.write(contents)
        try:
            os.link(temporary, target)
        except FileExistsError:
            pass  # A concurrent creator must still match; never overwrite it.
        verify()
    finally:
        Path(temporary).unlink(missing_ok=True)


def summary(source, rooms, data):
    entries = [entry for room in rooms for spawn in room['spawns'] for entry in spawn['entries']]
    return {'source': source, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'rooms': len(rooms),
            'roomTypes': dict(sorted(Counter(room['type'] for room in rooms).items())),
            'weightedSpawnEntries': len(entries),
            'scope': 'complete structural base-STB decoding; no runtime or generation parity'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path, nargs='?', help='An extracted base-Rebirth STB file')
    parser.add_argument('--all-base', action='store_true', help='Decode every declared stage from the original Linux rooms.a')
    parser.add_argument('--prepare-private-caves', action='store_true', help='Prepare only the ignored Caves and Depths build inputs from the pinned owner archive')
    parser.add_argument('--output', type=Path, help='JSON file for one STB, or a private JSON directory for --all-base')
    args = parser.parse_args()
    if args.prepare_private_caves:
        if args.source or args.all_base or args.output:
            parser.error('--prepare-private-caves cannot be combined with decoding arguments')
        prepare_private_caves()
        print('Verified private Caves and Depths build inputs.')
        return 0
    if bool(args.source) == args.all_base:
        parser.error('Choose an STB source or --all-base')
    assets = base_assets() if args.all_base else [(str(args.source), read_asset(args.source))]
    reports = []
    for source, data in assets:
        rooms = decode_bytes(data, source=source)
        if encode_bytes(rooms) != data:
            raise ValueError(f'{source}: structural decoding did not round-trip the original bytes')
        reports.append(summary(source, rooms, data))
        if args.output:
            target = args.output / (PurePosixPath(source).stem + '.json') if args.all_base else args.output
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(json.dumps({'schema': 1, 'source': source,
                'scope': 'authored structural room records; selectors are not concrete runtime entities',
                'rooms': rooms}, indent=2) + '\n')
    print(json.dumps({'schema': 1, 'status': 'pass', 'assets': reports,
                      'rooms': sum(report['rooms'] for report in reports)}, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
