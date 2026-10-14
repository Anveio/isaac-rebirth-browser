"""Bounded read-only Player/Familiar/effect ownership at actual public gates."""
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
identity_spec = importlib.util.spec_from_file_location('native_identity', ROOT / 'tools/original/native-identity.py')
native_identity = importlib.util.module_from_spec(identity_spec)
identity_spec.loader.exec_module(native_identity)
PIN = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


maps_reader = load('familiar_maps', 'tools/original/state-observer.py')
world_reader = load('familiar_world', 'tools/original/foundation-pickup-observe.py')


def observe(pid, context):
    report = {'schema': 1, 'status': 'running', 'nativeBinarySha256': PIN,
              'memoryWritten': False, 'executableMappingsRead': False, 'bytesRead': 0,
              'scope': 'Actual Player cache/child270, complete MT, room allocator, all Familiar and sprite effects; read-only, no source values enter production'}
    stopped = False
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Requires a positive directly owned PID at a public gate')
        parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines()
                      if line.startswith('PPid:'))
        if int(parent) != os.getpid():
            raise PermissionError('Original is not a directly owned child')
        if native_identity.native_binary_sha256(pid) != PIN:
            raise ValueError('Unexpected original executable')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while not all(path.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt'
                      for path in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic() > deadline:
                raise TimeoutError('Original threads did not stop')
            time.sleep(.001)
        ranges = maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, count):
                if count < 0 or report['bytesRead'] + count > 2 * 1024 * 1024 \
                        or not any(row['start'] <= address and address + count <= row['end'] for row in ranges):
                    raise ValueError('Read outside bounded writable native data')
                data = os.pread(fd, count, address)
                report['bytesRead'] += count
                if len(data) != count:
                    raise OSError('Short original read')
                return data

            def u64(address):
                return struct.unpack('<Q', read(address, 8))[0]

            def u32(address):
                return struct.unpack('<I', read(address, 4))[0]

            def i32(address):
                return struct.unpack('<i', read(address, 4))[0]

            def pair(address):
                values = list(struct.unpack('<ff', read(address, 8)))
                if not all(math.isfinite(value) for value in values):
                    raise ValueError('Nonfinite native vector')
                return values

            system = u64(0x8ed560)
            report['systemState'] = i32(system)
            report['systemCounter'] = u32(system + 0x2efa0)
            game = u64(0x8ecd30)
            report['gamePresent'] = bool(game) and report['systemState'] == 2
            report['sharedMt'] = {'index': u32(0x8e6b64),
                                 'words': list(struct.unpack('<624Q', read(0x8ed680, 624 * 8)))}
            if report['sharedMt']['index'] > 624 or any(word > 0xffffffff for word in report['sharedMt']['words']):
                raise ValueError('Invalid complete native MT words/index')
            if report['gamePresent']:
                begin, end = u64(game + 0xa148), u64(game + 0xa150)
                if end < begin or (end - begin) % 8 or end - begin > 32:
                    raise ValueError('Player list outside bound')
                player_pointers = list(struct.unpack('<' + 'Q' * ((end - begin) // 8), read(begin, end - begin)))
                report['players'] = []
                for pointer in player_pointers:
                    if u64(pointer) != 0x670c90:
                        raise ValueError('Unexpected Player class')
                    first, last = u64(pointer + 0xd00), u64(pointer + 0xd08)
                    if last < first or (last - first) % 16 or last - first > 128 * 16:
                        raise ValueError('Temporary collectible list outside bound')
                    temporary = []
                    for at in range(first, last, 16):
                        config = u64(at)
                        if i32(config) in (1, 3, 4):
                            temporary.append({'itemId': i32(config + 4), 'count': i32(at + 8)})
                    report['players'].append({'spawnSeed': u32(pointer + 0x7d0),
                        'sequenceId': u32(pointer + 8), 'ownedCounts': list(struct.unpack('<347i', read(pointer + 0xe1c, 347 * 4))),
                        'child270': list(struct.unpack('<4I', read(pointer + 0x2504, 16))),
                        'itemRngs': [list(struct.unpack('<4I', read(pointer + 0x1424 + 16 * index, 16))) for index in range(347)],
                        'temporaryCollectibles': temporary, 'cacheFlags': u32(pointer + 0xd6c),
                        'queuedId': i32(u64(pointer + 0x3058) + 4) if u64(pointer + 0x3058) else None})
                room = u64(game + 0x8180)
                report['nextSequenceId'] = u32(room + 0x15c8)
                report['gameUpdateCount'] = u32(game + 0xce9f4)
                manager = room + 0x15d0
                count = u32(manager + 0x74)
                if count > 512:
                    raise ValueError('Actor list outside bound')
                pointers = list(struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8))) if count else []
                if any(pointer == 0 for pointer in pointers) or len(set(pointers)) != count:
                    raise ValueError('Counted active actor list contains null or duplicate pointers')
                report['actorCount'] = count
                report['actorOrder'] = [{'type': u32(pointer + 0x10), 'variant': u32(pointer + 0x14),
                    'subtype': u32(pointer + 0x18), 'spawnSeed': u32(pointer + 0x7d0), 'sequenceId': u32(pointer + 8)}
                    for pointer in pointers]
                report['actors'] = []
                for pointer in pointers:
                    if u32(pointer + 0x10) not in (3, 1000):
                        continue
                    kind = u32(pointer + 0x10)
                    raw = read(pointer, 0xfcc if kind == 3 else 0x81c)

                    def integer(offset):
                        return struct.unpack_from('<I', raw, offset)[0]

                    def scalar(offset):
                        value = struct.unpack_from('<f', raw, offset)[0]
                        if not math.isfinite(value):
                            raise ValueError('Nonfinite native scalar')
                        return value

                    def vector(offset):
                        return [scalar(offset), scalar(offset + 4)]

                    for offset in (0x31c, 0x31d, 0x31e, 0x31f, 0x14c, 0x2fc):
                        if raw[offset] not in (0, 1):
                            raise ValueError('Invalid native actor bool')
                    clip = struct.unpack_from('<Q', raw, 0x40)[0]
                    text = u64(clip) if clip else 0
                    length = u64(text - 24) if text else 0
                    if length > 128:
                        raise ValueError('Clip name outside bound')
                    actor = {'type': kind, 'variant': integer(0x14), 'subtype': integer(0x18),
                        'sequenceId': integer(8), 'spawnSeed': integer(0x7d0),
                        'parentType': integer(0x1c), 'parentVariant': integer(0x20),
                        'actorRng': list(struct.unpack_from('<4I', raw, 0x7c0)),
                        'position': vector(0x448), 'velocity': vector(0x464),
                        'configuredFriction': scalar(0x46c), 'currentFriction': scalar(0x470),
                        'collisionRadius': scalar(0x474), 'collisionMass': scalar(0x480),
                        'hp': scalar(0x484), 'maxHp': scalar(0x488), 'collisionDamage': scalar(0x48c),
                        'gridCollisionClass': integer(0x340), 'entityCollisionClass': integer(0x344),
                        'flags': integer(0x318), 'visible': bool(raw[0x31d]), 'active': bool(raw[0x31e]),
                        'removed': bool(raw[0x31f]), 'birthGameUpdate': integer(0x42c),
                        'animation': read(text, length).decode('ascii') if length else None,
                        'animationFrame': scalar(0x148), 'animationPlaying': bool(raw[0x14c]),
                        'animationSpeed': scalar(0x300), 'spriteScale': vector(0x2c4),
                        'spriteOffset': vector(0x2bc), 'flipX': bool(raw[0x2fc])}
                    if kind == 3:
                        owner = struct.unpack_from('<Q', raw, 0x7d8)[0]
                        actor.update({'ownerOrdinal': player_pointers.index(owner) if owner in player_pointers else None,
                            'ownerSpawnSeed': u32(owner + 0x7d0) if owner in player_pointers else None,
                            'followerIndex': struct.unpack_from('<i', raw, 0xf7c)[0],
                            'appearanceCounter': struct.unpack_from('<i', raw, 0xfc8)[0]})
                    else:
                        actor.update({'renderOrderOffset': struct.unpack_from('<i', raw, 0x460)[0],
                            'lifetimeDivisor': struct.unpack_from('<i', raw, 0x7d4)[0],
                            'timeToLive': struct.unpack_from('<i', raw, 0x7f0)[0],
                            'raw7f4': struct.unpack_from('<i', raw, 0x7f4)[0], 'raw7f8': scalar(0x7f8)})
                    report['actors'].append(actor)
            report['status'] = 'pass'
        finally:
            os.close(fd)
    except Exception as error:
        report['status'] = 'fail'
        report['error'] = repr(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    if report['status'] == 'pass' and report.get('gamePresent'):
        report['world'] = world_reader.observe(pid, context)
        if report['world']['status'] != 'pass':
            report['status'] = 'fail'
            report['error'] = 'Native world observer failed'
    return report
