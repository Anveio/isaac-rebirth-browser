"""Read source-identified current room, grid and active entity descriptors.

All reads are bounded writable data in a directly owned blocked original child.
The ELF Game singleton and field offsets are reference tooling only.
"""
import hashlib
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('reader', ROOT / 'tools/original/state-observer.py')
reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)
NATIVE_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'


def observe(pid, context):
    if context.get('publicLogState', {}).get('stage') != 1:
        return {'schema': 2, 'status': 'skip', 'scope': 'combat gameplay-only observer'}
    report = {'schema': 2, 'status': 'pass', 'scope': 'source-identified bounded writable room/grid/entity fields',
              'memoryWritten': False, 'executableMappingsRead': False, 'checkpoint': context['label'],
              'nativeBinarySha256': NATIVE_SHA, 'bytesRead': 0}
    stopped = False
    try:
        parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or not context.get('gateBlocked'):
            raise PermissionError('Observation requires a blocked directly owned original')
        if hashlib.sha256(Path(f'/proc/{pid}/exe').read_bytes()).hexdigest() != NATIVE_SHA:
            raise ValueError('Unexpected original executable')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while True:
            state = next(line for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('State:'))
            if state.split(':', 1)[1].strip()[0] in ('T', 't'): break
            if time.monotonic() >= deadline: raise TimeoutError('Original did not stop')
            time.sleep(.001)
        maps = reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, count):
                if report['bytesRead'] + count > 1024 * 1024: raise ValueError('Read budget exceeded')
                if not any(m['start'] <= address and address + count <= m['end'] for m in maps): raise ValueError(f'Expected writable data range missing at {address:#x}')
                value = os.pread(fd, count, address)
                if len(value) != count: raise OSError('Short original data read')
                report['bytesRead'] += count
                return value
            def u64(address): return struct.unpack('<Q', read(address, 8))[0]
            def u32(address): return struct.unpack('<I', read(address, 4))[0]
            def entity(address):
                raw = read(address, 0x7d4)
                pair = lambda offset: list(struct.unpack_from('<ff', raw, offset))
                result = {'sequenceId': struct.unpack_from('<I', raw, 8)[0],
                    'type': struct.unpack_from('<I', raw, 0x10)[0],
                    'variant': struct.unpack_from('<I', raw, 0x14)[0],
                    'subtype': struct.unpack_from('<I', raw, 0x18)[0],
                    'removed': bool(raw[0x31f]), 'active': bool(raw[0x31e]), 'visible': bool(raw[0x31d]), 'position': pair(0x448), 'velocity': pair(0x464),
                    'collisionRadius': struct.unpack_from('<f', raw, 0x474)[0],
                    'collisionScale': pair(0x478), 'collidedWithGrid': bool(raw[0x34c]),
                    'preGridCollisionVelocity': pair(0x350), 'gridCollisionNormal': pair(0x358),
                    'collisionMass': struct.unpack_from('<f', raw, 0x480)[0],
                    'friction': struct.unpack_from('<f', raw, 0x470)[0],
                    'hp': struct.unpack_from('<f', raw, 0x484)[0],
                    'maxHp': struct.unpack_from('<f', raw, 0x488)[0],
                    'collisionDamage': struct.unpack_from('<f', raw, 0x48c)[0],
                    'pendingDamage': struct.unpack_from('<f', raw, 0x378)[0],
                    'gridCollisionClass': struct.unpack_from('<I', raw, 0x340)[0],
                    'entityCollisionClass': struct.unpack_from('<I', raw, 0x344)[0],
                    'spawnSeed': struct.unpack_from('<I', raw, 0x7d0)[0]}
                definition = struct.unpack_from('<Q', raw, 0x40)[0]
                text_pointer = u64(definition) if definition else 0
                length = u64(text_pointer - 24) if text_pointer else 0
                if length > 128: raise ValueError('Unexpected entity animation name length')
                result['animation'] = read(text_pointer, length).decode('ascii') if length else None
                result['animationFrame'] = struct.unpack_from('<f', raw, 0x148)[0]
                result['animationPlaying'] = bool(raw[0x14c])
                result['animationSpeed'] = struct.unpack_from('<f', raw, 0x300)[0]
                damage_begin, damage_end = struct.unpack_from('<QQ', raw, 0x360)
                damage_count = (damage_end - damage_begin) // 0x30
                if damage_end < damage_begin or (damage_end - damage_begin) % 0x30 or not 0 <= damage_count <= 128:
                    raise ValueError('Unexpected ordered pending damage list')
                result['pendingFlags'] = struct.unpack_from('<I', raw, 0x37c)[0]
                result['pendingDamageRecords'] = []
                records = read(damage_begin, damage_count * 0x30) if damage_count else b''
                for index in range(damage_count):
                    offset = index * 0x30
                    attacker = struct.unpack_from('<Q', records, offset + 0x28)[0]
                    identity = read(attacker, 0x1c) if attacker else None
                    result['pendingDamageRecords'].append({
                        'amount': struct.unpack_from('<f', records, offset)[0],
                        'flags': struct.unpack_from('<I', records, offset + 4)[0],
                        'sourceType': struct.unpack_from('<I', records, offset + 8)[0],
                        'sourceVariant': struct.unpack_from('<I', records, offset + 12)[0],
                        'sourceActorType': struct.unpack_from('<I', identity, 0x10)[0] if identity else None,
                        'sourceActorSequenceId': struct.unpack_from('<I', identity, 8)[0] if identity else None,
                        'sourceActorSpawnSeed': u32(attacker + 0x7d0) if attacker else None})
                begin, end = struct.unpack_from('<QQ', raw, 0x328)
                sample_count = (end - begin) // 8
                if not 0 <= sample_count <= 128 or end < begin or (end - begin) % 8:
                    raise ValueError('Unexpected collision sample list')
                samples = read(begin, sample_count * 8) if sample_count else b''
                result['gridCollisionSamples'] = [list(struct.unpack_from('<ff', samples, i * 8)) for i in range(sample_count)]
                if result['type'] == 4:
                    more = read(address + 0x7d4, 0x14)
                    result['bombFuseUpdatesRemaining'] = struct.unpack_from('<i', more, 0)[0]
                    result['bombExplosionDamage'] = struct.unpack_from('<f', more, 4)[0]
                if result['type'] == 9:
                    more = read(address + 0x7d4, 0x24)
                    result['projectileHeight'], result['projectileFallingSpeed'], result['projectileFallingAcceleration'] = struct.unpack_from('<fff', more, 0)
                    result['projectileScale'] = struct.unpack_from('<f', more, 0x10)[0]
                    result['projectileParameter1'] = struct.unpack_from('<f', more, 0x14)[0]
                    result['projectileParameter2'] = struct.unpack_from('<f', more, 0x18)[0]
                    result['projectileFullUpdates'] = struct.unpack_from('<I', more, 0x1c)[0]
                    result['projectileFlags'] = struct.unpack_from('<I', more, 0x20)[0]
                if struct.unpack_from('<Q', raw, 0)[0] == 0x66ef30:
                    more = read(address + 0xf50, 0x138)
                    result['npcState'] = struct.unpack_from('<I', more, 0)[0]
                    result['npcSavedInitializedState'] = struct.unpack_from('<I', more, 4)[0]
                    result['npcStateFrame'] = struct.unpack_from('<I', more, 8)[0]
                    result['npcFrameAge'] = struct.unpack_from('<I', more, 0x10)[0]
                    result['npcCountdown'] = struct.unpack_from('<I', more, 0x1c)[0]
                    result['npcTarget'] = pair(0x440)
                    result['npcWanderVelocity'] = list(struct.unpack_from('<ff', more, 0x28))
                    result['canShutDoors'] = bool(more[0xec])
                    result['boss'] = bool(more[0xed])
                return result

            game = u64(0x8ecd30)
            system = u64(0x8ed560)
            report['nativePhase'] = {'systemIterations': u32(system + 0x2efa0), 'fullGameUpdates': u32(game + 0xce9f4)}
            room = u64(game + 0x8180)
            raw = read(room, 0x30)
            width, height = struct.unpack_from('<II', raw, 0x14)
            if not 1 <= width <= 28 or not 1 <= height <= 16 or width * height > 448:
                raise ValueError('Unexpected room grid shape')
            descriptor = u64(room + 8)
            report['room'] = {'type': struct.unpack_from('<I', raw, 0x10)[0], 'gridWidth': width, 'gridHeight': height,
                'descriptorIndex': u32(descriptor + 4), 'descriptorGridIndex': u32(descriptor),
                'descriptorSpawnSeed': u32(descriptor + 0x4c),
                'boundsTopLeft': list(struct.unpack_from('<ff', raw, 0x1c)),
                'boundsBottomRight': list(struct.unpack_from('<ff', raw, 0x24)),
                'combatClear': bool(read(descriptor + 0x3d, 1)[0]),
                'descriptorFlag40': bool(read(descriptor + 0x40, 1)[0]),
                'combatClearCountdown': struct.unpack('<i', read(room + 0x1598, 4))[0],
                'countablePending': struct.unpack('<i', read(room + 0x16ac, 4))[0],
                'countableCurrent': struct.unpack('<i', read(room + 0x16b0, 4))[0]}
            grid_pointers = struct.unpack('<' + 'Q' * (width * height), read(room + 0x30, width * height * 8))
            cells = []
            for index, pointer in enumerate(grid_pointers):
                if not pointer: continue
                raw = read(pointer, 0x40)
                actual_index = struct.unpack_from('<I', raw, 0x24)[0]
                if actual_index != index: raise ValueError('Grid object index does not match owning room slot')
                cells.append({'index': index, 'type': struct.unpack_from('<I', raw, 8)[0],
                    'variant': struct.unpack_from('<I', raw, 0xc)[0], 'state': struct.unpack_from('<I', raw, 0x10)[0],
                    'varData': struct.unpack_from('<I', raw, 0x14)[0], 'spawnSeed': struct.unpack_from('<I', raw, 0x18)[0],
                    'collisionClass': struct.unpack_from('<I', raw, 0x3c)[0],
                    'position': [40 + index % width * 40, 120 + index // width * 40]})
            report['grid'] = cells
            report['gridSelectors'] = list(struct.unpack('<' + 'I' * (width * height),
                read(room + 0xe98, width * height * 4)))
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512: raise ValueError('Active entity budget exceeded')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            report['entities'] = [entity(pointer) for pointer in pointers if pointer]
            player_begin, player_end = u64(game + 0xa148), u64(game + 0xa150)
            count = (player_end - player_begin) // 8
            if not 0 <= count <= 4 or player_end < player_begin or (player_end - player_begin) % 8:
                raise ValueError('Unexpected original player list')
            players = []
            for pointer in struct.unpack('<' + 'Q' * count, read(player_begin, count * 8)) if count else []:
                item = entity(pointer)
                raw = read(pointer + 0xbb8, 0x44)
                item['redHp'] = struct.unpack_from('<I', raw, 0)[0]
                item['damageCountdown'] = struct.unpack_from('<I', raw, 0x40)[0]
                damage = read(pointer + 0xde8, 0x2c)
                attacker_pointer = struct.unpack_from('<Q', damage, 0x20)[0]
                joined = next((actor for address, actor in zip([value for value in pointers if value], report['entities'])
                               if address == attacker_pointer), None)
                item['lastDamageSource'] = {'type': struct.unpack_from('<I', damage, 0)[0],
                    'variant': struct.unpack_from('<I', damage, 4)[0],
                    'raw1c': struct.unpack_from('<I', damage, 8)[0], 'raw20': struct.unpack_from('<I', damage, 12)[0],
                    'position': list(struct.unpack_from('<ff', damage, 0x10)),
                    'velocity': list(struct.unpack_from('<ff', damage, 0x18)),
                    'flags': struct.unpack_from('<I', damage, 0x28)[0],
                    'activeActorSequenceId': joined['sequenceId'] if joined else None,
                    'activeActorSpawnSeed': joined['spawnSeed'] if joined else None}
                players.append(item)
            report['players'] = players
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError, struct.error) as error:
        report['status'] = 'fail'; report['error'] = str(error)
    finally:
        if stopped: os.kill(pid, signal.SIGCONT)
    return report
