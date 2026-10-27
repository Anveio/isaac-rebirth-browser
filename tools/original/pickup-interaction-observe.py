"""Read room entities and the Player's item/pocket/charge words for interaction cases.

Reference tooling only: bounded reads of writable data in a blocked, directly
owned original child. Offsets are the ones the pickup, item and Bomb observers
already use; nothing is written and nothing enters the port.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]


def _module(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


native_identity = _module('native_identity', 'tools/original/native-identity.py')
reader = _module('state_reader', 'tools/original/state-observer.py')
NATIVE_SHA = native_identity.NATIVE_SHA
# Collectibles whose ownership the interaction cases grant or test.
WATCHED_ITEMS = (63, 115, 286)


def observe(pid, context):
    if context.get('publicLogState', {}).get('stage') != 1:
        return {'schema': 1, 'status': 'skip', 'scope': 'gameplay-only interaction observer'}
    report = {'schema': 1, 'status': 'pass', 'checkpoint': context['label'], 'memoryWritten': False, 'bytesRead': 0}
    stopped = False
    try:
        parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines()
                      if line.startswith('PPid:'))
        if int(parent) != os.getpid() or not context.get('gateBlocked'):
            raise PermissionError('Observation requires a blocked directly owned original')
        if native_identity.native_binary_sha256(pid) != NATIVE_SHA:
            raise ValueError('Unexpected original executable')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while True:
            state = next(line for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('State:'))
            if state.split(':', 1)[1].strip()[0] in ('T', 't'):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('Original did not stop')
            time.sleep(.001)
        maps = reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, count):
                if report['bytesRead'] + count > 2 * 1024 * 1024:
                    raise ValueError('Read budget exceeded')
                if not any(m['start'] <= address and address + count <= m['end'] for m in maps):
                    raise ValueError(f'Expected writable data range missing at {address:#x}')
                value = os.pread(fd, count, address)
                if len(value) != count:
                    raise OSError('Short original data read')
                report['bytesRead'] += count
                return value

            def u64(address): return struct.unpack('<Q', read(address, 8))[0]
            def u32(address): return struct.unpack('<I', read(address, 4))[0]
            def i32(address): return struct.unpack('<i', read(address, 4))[0]

            def clip_name(pointer):
                if not pointer:
                    return None
                text = u64(pointer)
                count = u64(text - 24)
                if count > 128:
                    raise ValueError('Unexpected animation name length')
                return read(text, count).decode('ascii')

            def entity(address):
                raw = read(address, 0x7d4)
                pair = lambda offset: list(struct.unpack_from('<ff', raw, offset))
                result = {'type': struct.unpack_from('<I', raw, 0x10)[0],
                          'variant': struct.unpack_from('<I', raw, 0x14)[0],
                          'subtype': struct.unpack_from('<I', raw, 0x18)[0],
                          'dead': bool(raw[0x31f]), 'position': pair(0x448), 'velocity': pair(0x464),
                          'entityCollisionClass': struct.unpack_from('<I', raw, 0x344)[0],
                          'gridCollisionClass': struct.unpack_from('<I', raw, 0x340)[0]}
                if result['type'] == 4:
                    more = read(address + 0x7d4, 0x14)
                    # Bomb+7D4 fuse, +7DC Player-contact countdown (docs/dr-fetus-evidence.md).
                    result['bombFuse'] = struct.unpack_from('<i', more, 0)[0]
                    result['bombPlayerContactCountdown'] = struct.unpack_from('<i', more, 8)[0]
                if 10 <= result['type'] < 1000:
                    # Entity+484 HitPoints and +488 MaxHitPoints.
                    result['hp'], result['maxHp'] = pair(0x484)
                if result['type'] == 5:
                    more = read(address + 0xac0, 0x18)
                    result['pickupPrice'] = struct.unpack_from('<i', more, 0)[0]
                return result

            game = u64(0x8ecd30)
            system = u64(0x8ed560)
            report['fullGameUpdates'] = u32(game + 0xce9f4)
            room = u64(game + 0x8180)
            raw = read(room, 0x30)
            width, height = struct.unpack_from('<II', raw, 0x14)
            if not 1 <= width <= 28 or not 1 <= height <= 16:
                raise ValueError('Unexpected room grid shape')
            report['room'] = {'type': struct.unpack_from('<I', raw, 0x10)[0], 'gridWidth': width, 'gridHeight': height}
            # Room+30 GridEntity* by grid index: poop (type +8 == 14) cells
            # with their variant (+C) and collision class (+3C).
            cells = struct.unpack('<' + 'Q' * (width * height), read(room + 0x30, 8 * width * height))
            poops = []
            for index, pointer in enumerate(cells):
                if pointer:
                    grid = read(pointer, 0x40)
                    kind, variant = struct.unpack_from('<II', grid, 0x8)
                    if kind == 14:
                        poops.append([index, variant, struct.unpack_from('<I', grid, 0x3c)[0]])
            report['gridPoops'] = poops
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512:
                raise ValueError('Active entity budget exceeded')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            report['entities'] = [entity(pointer) for pointer in pointers if pointer]
            first, end = u64(game + 0xa148), u64(game + 0xa150)
            if end < first or (end - first) % 8 or end - first > 32:
                raise ValueError('Player list outside bound')
            players = []
            for at in range(first, end, 8):
                player = u64(at)
                if u64(player) != 0x670c90:
                    raise ValueError('Unexpected player class')
                item = entity(player)
                owned = struct.unpack('<347i', read(player + 0xe1c, 347 * 4))
                held = read(player + 0x3044, 16)
                item.update({
                    'health': [i32(player + offset) for offset in (0xbb4, 0xbb8, 0xbc0, 0xbc4)],
                    'inventory': {'bombs': i32(player + 0xbd4), 'keys': i32(player + 0xbcc), 'coins': i32(player + 0xbd8)},
                    'ownedItems': {str(index): owned[index] for index in WATCHED_ITEMS},
                    'ownedTotal': sum(value for value in owned if value > 0),
                    'activeId': i32(player + 0xd74), 'activeCharge': i32(player + 0xd78),
                    'activeSubordinateCharge': i32(player + 0xd7c), 'reserveCharge': i32(player + 0xd8c),
                    'controlsEnabled': read(player + 0x7d4, 1)[0] == 1, 'inputCooldown': i32(player + 0x7d8),
                    'animationGuard': read(player + 0xbf4, 1)[0] == 1,
                    'animation': clip_name(u64(player + 0x28 + 0x18)),
                    'heldPockets': [{'id': struct.unpack_from('<i', held, slot * 8)[0], 'tag': held[slot * 8 + 4]}
                                    for slot in range(2)],
                })
                players.append(item)
            report['players'] = players
            # The shared MT (audio-observer.py offsets), a diagnostic only.
            report['sharedMt'] = {'index': u32(0x8e6b64),
                                  'words': list(struct.unpack('<624Q', read(0x8ed680, 624 * 8)))}
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError, struct.error, StopIteration) as error:
        report['status'] = 'fail'
        report['error'] = str(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report
