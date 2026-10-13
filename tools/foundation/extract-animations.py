#!/usr/bin/env python3
"""Extract authored animation assets from base Rebirth resources, not game code.

Python adaptation of the public ARCH000/LZW/animation-cache data reader at
https://github.com/gibbed/Gibbed.Rebirth. Altered source, not the original tool.
"""
# Copyright (c) 2017 Rick (rick 'at' gibbed 'dot' us)
#
# This software is provided 'as-is', without any express or implied
# warranty. In no event will the authors be held liable for any damages
# arising from the use of this software.
#
# Permission is granted to anyone to use this software for any purpose,
# including commercial applications, and to alter it and redistribute it
# freely, subject to the following restrictions:
#
# 1. The origin of this software must not be misrepresented; you must not
#    claim that you wrote the original software. If you use this software
#    in a product, an acknowledgment in the product documentation would
#    be appreciated but is not required.
#
# 2. Altered source versions must be plainly marked as such, and must not
#    be misrepresented as being the original software.
#
# 3. This notice may not be removed or altered from any source
#    distribution.

import argparse
import fnmatch
import json
import math
from pathlib import Path
import posixpath
import struct
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]


class Reader:
    def __init__(self, data):
        self.data = data
        self.at = 0

    def take(self, length):
        if length < 0 or self.at + length > len(self.data):
            raise ValueError(f'Truncated asset at {self.at}, need {length} bytes')
        output = self.data[self.at:self.at + length]
        self.at += length
        return output

    def number(self, format_):
        return struct.unpack('<' + format_, self.take(struct.calcsize('<' + format_)))[0]

    def u32(self):
        return self.number('I')

    def count(self, limit=50000):
        count = self.u32()
        if count > limit:
            raise ValueError(f'Invalid asset count {count} at {self.at}')
        return count

    def f32(self):
        number = self.number('f')
        if not math.isfinite(number):
            raise ValueError('Non-finite authored animation value')
        return number

    def boolean(self):
        value = self.number('B')
        if value > 1:
            raise ValueError(f'Invalid boolean at {self.at - 1}')
        return value == 1

    def text(self):
        return self.take(self.number('H')).decode('ascii').rstrip('\0')


class Bits:
    def __init__(self, data):
        self.data = data
        self.at = 0
        self.value = 0
        self.remaining = 0

    def read(self, length):
        while self.remaining < length:
            if self.at >= len(self.data):
                raise ValueError('Truncated LZW bit stream')
            self.value = (self.value << 8) | self.data[self.at]
            self.at += 1
            self.remaining += 8
        self.remaining -= length
        value = (self.value >> self.remaining) & ((1 << length) - 1)
        self.value &= (1 << self.remaining) - 1
        return value


def decompress_lzw(reader, length):
    if length > 64 * 1024 * 1024:
        raise ValueError('Animation asset is unexpectedly large')
    dictionary = [bytes([index]) for index in range(256)]
    width = 8
    output = bytearray()
    while len(output) < length:
        block = Bits(reader.take(reader.count(2 * 1024 * 1024)))
        previous = block.read(width)
        line = dictionary[previous]
        output.extend(line)
        first = line[0]
        while block.at < len(block.data):
            if len(dictionary) + 1 >= 4096:
                dictionary = [bytes([index]) for index in range(256)]
                width = 8
                previous = block.read(width)
                line = dictionary[previous]
                output.extend(line)
                first = line[0]
                continue
            while len(dictionary) >= 1 << width:
                width += 1
            index = block.read(width)
            if index == len(dictionary):
                dictionary.append(dictionary[previous] + bytes([first]))
                line = dictionary[index]
                first = line[0]
            elif index < len(dictionary):
                line = dictionary[index]
                first = line[0]
                dictionary.append(dictionary[previous] + bytes([first]))
            else:
                raise ValueError(f'Invalid LZW dictionary index {index}')
            output.extend(line)
            previous = index
        if len(output) > length:
            raise ValueError('LZW output exceeds declared asset size')
    if len(output) != length:
        raise ValueError('LZW output size mismatch')
    return bytes(output)


def path_hash(path):
    value = 5381
    for byte in path.encode('ascii'):
        value = (value * 33 + byte) & 0xffffffff
    return value


def archive_entries(path):
    data = path.read_bytes()
    reader = Reader(data)
    if reader.take(7) != b'ARCH000':
        raise ValueError('Expected base Rebirth ARCH000 asset archive')
    compression = reader.number('B')
    offset, count = reader.u32(), reader.number('H')
    reader.at = offset
    entries = []
    for _ in range(count):
        entries.append(struct.unpack('<IIIII', reader.take(20)))
    return data, compression, entries


def load_archive(path):
    data, compression, entries = archive_entries(path)
    if compression != 1:
        raise ValueError('Expected authored animation LZW compression')
    reader = Reader(data)
    expected = path_hash('resources/animations.b')
    for hash_a, _hash_b, entry_offset, length, _checksum in entries:
        if hash_a == expected:
            reader.at = entry_offset
            return decompress_lzw(reader, length)
    raise ValueError('animations.b asset entry is missing')


def path_hash_b(path):
    value = 0x5bb2220e
    for byte in path.encode('ascii'):
        value = ((value ^ byte) * 0x1000193) & 0xffffffff
    return value


def read_bogocrypt(reader, length, hash_b):
    """Public asset archive's reversible byte encoding, not executable DRM."""
    key = (hash_b ^ 0xf9524287) | 1
    output = bytearray()
    remaining = length
    while remaining > 0:
        count = min(1024, (remaining + 3) & ~3)
        block = bytearray(reader.take(count))
        for index in range(0, count, 4):
            for byte in range(4):
                block[index + byte] ^= (key >> (byte * 8)) & 255
            swap = key & 15
            if swap == 2:
                block[index:index + 4] = block[index:index + 4][::-1]
            elif swap == 9:
                a, b, c, d = block[index:index + 4]
                block[index:index + 4] = bytes([b, a, d, c])
            elif swap == 13:
                a, b, c, d = block[index:index + 4]
                block[index:index + 4] = bytes([c, d, a, b])
            shifted = (key << 8) & 0xffffffff
            part = (key ^ shifted) >> 9
            key = (key ^ part ^ shifted ^ ((part ^ key ^ shifted) << 23)) & 0xffffffff
        output.extend(block[:min(remaining, count)])
        remaining -= count
    return bytes(output)


def extract_named_assets(archive, paths, output):
    data, compression, entries = archive_entries(archive)
    table = {(entry[0], entry[1]): entry for entry in entries}
    found = []
    for path in sorted(set(paths)):
        path = posixpath.normpath(path.replace('\\', '/')).lower()
        if not path.startswith('resources/') or '..' in path.split('/'):
            raise ValueError(f'Invalid requested resource path {path}')
        name = (path_hash(path), path_hash_b(path))
        if name not in table:
            raise ValueError(f'Authored asset missing from Linux archive: {path}')
        _, hash_b, offset, length, _checksum = table[name]
        reader = Reader(data)
        reader.at = offset
        if compression == 0:
            contents = read_bogocrypt(reader, length, hash_b)
        elif compression == 1:
            contents = decompress_lzw(reader, length)
        else:
            raise ValueError(f'Unsupported authored asset compression {compression}')
        if path.endswith('.png') and not contents.startswith(b'\x89PNG\r\n\x1a\n'):
            raise ValueError(f'Invalid decoded Linux PNG: {path}')
        target = output / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(contents)
        found.append(path)
    return found


def frame(reader, layer=False):
    fields = ['XCrop', 'YCrop', 'Width', 'Height', 'XPosition', 'YPosition', 'XScale', 'YScale', 'XPivot', 'YPivot'] if layer else ['XPosition', 'YPosition', 'XScale', 'YScale']
    result = {field: reader.f32() for field in fields}
    result['Delay'] = reader.number('i')
    result['Visible'] = reader.boolean()
    for field in ['RedTint', 'GreenTint', 'BlueTint', 'AlphaTint', 'RedOffset', 'GreenOffset', 'BlueOffset', 'Rotation']:
        result[field] = reader.f32()
    result['Interpolated'] = reader.boolean()
    # Match authored ANM2 units: scale is percent, tint/offset is 0..255.
    for field in ['XScale', 'YScale']:
        result[field] *= 100
    for field in ['RedTint', 'GreenTint', 'BlueTint', 'AlphaTint', 'RedOffset', 'GreenOffset', 'BlueOffset']:
        result[field] *= 255
    return result


def frames(reader, layer=False):
    return [frame(reader, layer) for _ in range(reader.count(4999))]


def actor(reader):
    base = reader.text()
    spritesheets = [{'Id': reader.u32(), 'Path': reader.text()} for _ in range(reader.count(32))]
    layers = [{'Id': reader.u32(), 'SpritesheetId': reader.u32(), 'Name': reader.text()} for _ in range(reader.count(32))]
    nulls = [{'Id': reader.u32(), 'Name': reader.text()} for _ in range(reader.count(32))]
    events = [{'Id': reader.u32(), 'Name': reader.text()} for _ in range(reader.count(16))]
    default = reader.text()
    animations = []
    for _ in range(reader.count(1000)):
        animation = {'Name': reader.text(), 'FrameNum': reader.u32(), 'Loop': reader.boolean(), 'RootAnimation': frames(reader)}
        animation['LayerAnimations'] = [{'LayerId': reader.u32(), 'Visible': reader.boolean(), 'Frames': frames(reader, True)} for _ in range(reader.count(31))]
        animation['NullAnimations'] = [{'NullId': reader.u32(), 'Visible': reader.boolean(), 'Frames': frames(reader)} for _ in range(reader.count(31))]
        animation['Triggers'] = [{'EventId': reader.u32(), 'AtFrame': reader.u32()} for _ in range(reader.count(31))]
        animations.append(animation)
    return {'BaseSpritesheetPath': base, 'Spritesheets': spritesheets, 'Layers': layers, 'Nulls': nulls, 'Events': events, 'DefaultAnimation': default, 'Animations': animations}


def xml_attributes(values):
    return {key: str(value).lower() if isinstance(value, bool) else format(value, '.9g') if isinstance(value, float) else str(value) for key, value in values.items()}


def write_anm2(path, value):
    root = ET.Element('AnimatedActor')
    content = ET.SubElement(root, 'Content')
    for collection, element in [('Spritesheets', 'Spritesheet'), ('Layers', 'Layer'), ('Nulls', 'Null'), ('Events', 'Event')]:
        group = ET.SubElement(content, collection)
        for entry in value[collection]:
            ET.SubElement(group, element, xml_attributes(entry))
    animations = ET.SubElement(root, 'Animations', {'DefaultAnimation': value['DefaultAnimation']})
    for entry in value['Animations']:
        animation = ET.SubElement(animations, 'Animation', xml_attributes({key: entry[key] for key in ['Name', 'FrameNum', 'Loop']}))
        roots = ET.SubElement(animation, 'RootAnimation')
        for entry_frame in entry['RootAnimation']:
            ET.SubElement(roots, 'Frame', xml_attributes(entry_frame))
        for collection, element, id_ in [('LayerAnimations', 'LayerAnimation', 'LayerId'), ('NullAnimations', 'NullAnimation', 'NullId')]:
            group = ET.SubElement(animation, collection)
            for child in entry[collection]:
                item = ET.SubElement(group, element, xml_attributes({id_: child[id_], 'Visible': child['Visible']}))
                for entry_frame in child['Frames']:
                    ET.SubElement(item, 'Frame', xml_attributes(entry_frame))
        triggers = ET.SubElement(animation, 'Triggers')
        for trigger in entry['Triggers']:
            ET.SubElement(triggers, 'Trigger', xml_attributes(trigger))
    path.parent.mkdir(parents=True, exist_ok=True)
    ET.indent(root)
    ET.ElementTree(root).write(path, encoding='utf-8', xml_declaration=True)


def read_font(path):
    """Read standard AngelCode BMFont v3 authored glyph metrics."""
    reader = Reader(path.read_bytes())
    if reader.take(4) != b'BMF\x03':
        raise ValueError(f'Expected binary BMFont v3: {path}')
    output = {'chars': {}, 'kernings': []}
    seen = set()
    while reader.at < len(reader.data):
        type_ = reader.number('B')
        block = Reader(reader.take(reader.u32()))
        if type_ in seen:
            raise ValueError(f'Duplicate font block {type_}: {path}')
        seen.add(type_)
        if type_ == 1:
            output['fontSize'] = block.number('h')
            block.take(12)
            output['name'] = block.take(len(block.data) - block.at).rstrip(b'\0').decode('utf-8')
        elif type_ == 2:
            output.update(dict(zip(['lineHeight', 'base', 'scaleW', 'scaleH', 'pageCount'], struct.unpack('<HHHHH', block.take(10)))))
            block.take(5)
        elif type_ == 3:
            output['pages'] = [page.decode('utf-8') for page in block.data.split(b'\0') if page]
            block.take(len(block.data))
        elif type_ == 4:
            while block.at < len(block.data):
                values = struct.unpack('<IHHHHhhhBB', block.take(20))
                glyph = dict(zip(['id', 'x', 'y', 'width', 'height', 'xoffset', 'yoffset', 'xadvance', 'page', 'chnl'], values))
                if str(glyph['id']) in output['chars']:
                    raise ValueError(f'Duplicate glyph {glyph["id"]}: {path}')
                output['chars'][str(glyph['id'])] = glyph
        elif type_ == 5:
            while block.at < len(block.data):
                output['kernings'].append(dict(zip(['first', 'second', 'amount'], struct.unpack('<IIh', block.take(10)))))
        else:
            raise ValueError(f'Unknown BMFont block {type_}: {path}')
        if block.at != len(block.data):
            raise ValueError(f'Trailing bytes in font block {type_}: {path}')
    if not {1, 2, 3, 4}.issubset(seen) or len(output['pages']) != output['pageCount']:
        raise ValueError(f'Incomplete BMFont: {path}')
    for glyph in output['chars'].values():
        if glyph['page'] >= output['pageCount'] or glyph['x'] + glyph['width'] > output['scaleW'] or glyph['y'] + glyph['height'] > output['scaleH']:
            raise ValueError(f'Glyph exceeds authored font page: {path}')
    return output


def write_fonts(output, source):
    fonts = {path.stem: read_font(path) for path in sorted(source.glob('*.fnt'))}
    if not fonts:
        raise ValueError(f'No authored fonts found: {source}')
    output.mkdir(parents=True, exist_ok=True)
    (output / 'fonts.json').write_text(json.dumps(fonts, separators=(',', ':')) + '\n')
    return len(fonts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, default=ROOT / 'references/isaac/linux/depot_250903/resources/packed/animations.a')
    parser.add_argument('--filenames', type=Path, default=ROOT / 'references/isaac/resource-filenames.txt')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation/assets')
    parser.add_argument('--select', action='append', help='Actor path/glob to export; repeat for multiple selections')
    parser.add_argument('--json-output', '--out', type=Path, help='Also write the selected actor map to this JSON file')
    parser.add_argument('--fonts', type=Path, default=ROOT / 'references/isaac/rebirth/resources/font')
    parser.add_argument('--fonts-only', action='store_true', help='Only export standard BMFont authored glyph metrics')
    parser.add_argument('--graphics', action='store_true', help='Extract selected animations\' actual Linux PNG sheets, backdrop, controls and fonts')
    parser.add_argument('--graphic', action='append', help='Additional specific gfx/... Linux PNG resource; repeat as needed')
    args = parser.parse_args()
    fonts = write_fonts(args.output, args.fonts)
    if args.fonts_only:
        print(json.dumps({'fonts': fonts, 'output': str(args.output)}))
        return
    names = {}
    for path in args.filenames.read_text().splitlines():
        if path.startswith('resources/') and path.endswith('.anm2'):
            names[path_hash(path)] = path.removeprefix('resources/')
    # Base v1.05 authored name, listed publicly by Gibbed; newer inventories moved it.
    names[path_hash('resources/gfx/ui/main menu/completion_widget.anm2')] = 'gfx/ui/main menu/completion_widget.anm2'
    binary = load_archive(args.archive)
    reader = Reader(binary)
    actors = {}
    unknown = []
    count = reader.count(5000)
    for _ in range(count):
        name = reader.u32()
        value = actor(reader)
        path = names.get(name)
        if not path:
            path = f'unknown/{name:08x}.anm2'
            unknown.append(path)
        if path in actors:
            raise ValueError(f'Duplicate authored actor {path}')
        actors[path] = value
    if reader.at != len(binary):
        raise ValueError(f'Animation cache trailing bytes: {len(binary) - reader.at}')
    if args.select:
        for pattern in args.select:
            if not any(fnmatch.fnmatchcase(path, pattern) for path in actors):
                raise ValueError(f'No authored animation matches requested selection {pattern}')
        actors = {path: value for path, value in actors.items() if any(fnmatch.fnmatchcase(path, pattern) for pattern in args.select)}
        unknown = [path for path in unknown if path in actors]
    for path, value in actors.items():
        write_anm2(args.output / path, value)
    graphics = []
    if args.graphics or args.graphic:
        packed = args.archive.parent
        font_paths = ['resources/font/' + path.name for path in args.fonts.glob('*.fnt')]
        extract_named_assets(packed / 'fonts.a', font_paths, args.output)
        fonts = write_fonts(args.output, args.output / 'resources/font')
        native_fonts = json.loads((args.output / 'fonts.json').read_text())
        image_paths = ['resources/font/' + page for font in native_fonts.values() for page in font['pages']]
        image_paths.extend('resources/' + path.removeprefix('resources/') for path in args.graphic or [])
        if args.graphics:
            image_paths.extend(['resources/gfx/backdrop/01_basement.png', 'resources/gfx/backdrop/controls.png'])
            for path, value in actors.items():
                directory = value['BaseSpritesheetPath'].replace('\\', '/')
                for sheet in value['Spritesheets']:
                    image_paths.append(posixpath.normpath(posixpath.join(directory, sheet['Path'].replace('\\', '/'))))
        graphics = extract_named_assets(packed / 'graphics.a', image_paths, args.output)
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'animations.json').write_text(json.dumps(actors, separators=(',', ':')) + '\n')
    if args.json_output:
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(actors, separators=(',', ':')) + '\n')
    (args.output / 'index.json').write_text(json.dumps({'source': str(args.archive), 'format': 'authored ANM2 asset data', 'actors': sorted(actors), 'unknown': unknown}, indent=2) + '\n')
    print(json.dumps({'actors': len(actors), 'unknown': len(unknown), 'fonts': fonts, 'graphics': len(graphics), 'output': str(args.output)}))


if __name__ == '__main__':
    main()
