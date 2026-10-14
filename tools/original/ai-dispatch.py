#!/usr/bin/env python3
"""Index the original Linux Entity_NPC::update_ai dispatch.

update_ai (0x4f9670) subtracts 10 from the entity type, bounds it by 0x109 and
jumps through the .rodata table at 0x66e390. Each entry is a thunk: optional
nop padding then `jmp rel32` to the type's AI routine, or `repz ret` for types
without one. This prints every authored type's AI entry, the types sharing it
and their entities2.xml names: the starting point for porting a new entity.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct

ROOT = Path(__file__).resolve().parents[2]
BINARY = ROOT / '.cache/original-runtime/offline-depot/isaac.x64'
# The one original binary pin lives in tools/e2e/lib/original.mjs.
SHA256 = re.search(r"ORIGINAL_SHA = '([0-9a-f]{64})'", (ROOT / 'tools/e2e/lib/original.mjs').read_text()).group(1)
UPDATE_AI, TABLE, FIRST_TYPE, COUNT = 0x4f9670, 0x66e390, 10, 0x10a
UPDATE_AI_BYTES = bytes.fromhex('8b4710' '83e80a' '3d09010000' '7603' 'f3c3' '90' 'ff24c5') + struct.pack('<I', TABLE)


def sections(data):
    """ELF64 section headers as (vaddr, offset, size)."""
    shoff, = struct.unpack_from('<Q', data, 0x28)
    entsize, count = struct.unpack_from('<HH', data, 0x3a)
    for index in range(count):
        _, _, _, addr, offset, size = struct.unpack_from('<IIQQQQ', data, shoff + index * entsize)
        if addr:
            yield addr, offset, size


def reader(data):
    table = list(sections(data))
    def read(address, size):
        for start, offset, length in table:
            if start <= address and address + size <= start + length:
                return data[offset + address - start:offset + address - start + size]
        raise ValueError(f'unmapped address {address:#x}')
    return read


# Standard x86-64 alignment nops (with any extra 0x66 prefixes).
NOPS = [bytes.fromhex(item) for item in ('2e0f1f840000000000', '0f1f840000000000', '0f1f8000000000',
                                         '0f1f440000', '0f1f4000', '0f1f00', '90')]


def thunk_target(read, address):
    code = read(address, 32)
    skip = 0
    while True:
        while code[skip] == 0x66:
            skip += 1
        nop = next((item for item in NOPS if code.startswith(item, skip)), None)
        if not nop:
            break
        skip += len(nop)
    if code[skip:skip + 2] == b'\xf3\xc3':
        return None
    if code[skip] != 0xe9:
        raise ValueError(f'unexpected update_ai thunk at {address:#x}: {code.hex()}')
    return address + skip + 5 + struct.unpack_from('<i', code, skip + 1)[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--type', type=int, action='append', help='only these entity types')
    args = parser.parse_args()
    data = BINARY.read_bytes()
    if hashlib.sha256(data).hexdigest() != SHA256:
        raise SystemExit('Not the pinned original Linux x64 executable')
    read = reader(data)
    if read(UPDATE_AI, len(UPDATE_AI_BYTES)) != UPDATE_AI_BYTES:
        raise SystemExit('update_ai dispatch differs from the indexed layout')
    xml = (ROOT / 'references/isaac/rebirth/resources/entities2.xml').read_text(errors='replace')
    names = {}
    for match in re.finditer(r'<entity ([^>]*)>', xml):
        attributes = dict(re.findall(r'(\w+)="([^"]*)"', match.group(1)))
        names.setdefault(int(attributes['id']), []).append(attributes['name'])
    entries = {}
    for index in range(COUNT):
        target = thunk_target(read, struct.unpack('<Q', read(TABLE + 8 * index, 8))[0])
        entries[FIRST_TYPE + index] = target
    shared = {}
    for kind, target in entries.items():
        if target is not None:
            shared.setdefault(target, []).append(kind)
    rows = [{'type': kind, 'names': names.get(kind, []), 'ai': f'{target:x}' if target else None,
             'sharedWith': [other for other in shared.get(target, []) if other != kind]}
            for kind, target in entries.items()
            if (kind in names or target) and (not args.type or kind in args.type)]
    print(json.dumps({'binarySha256': SHA256, 'updateAi': f'{UPDATE_AI:x}', 'table': f'{TABLE:x}', 'types': rows}, indent=1))


if __name__ == '__main__':
    main()
