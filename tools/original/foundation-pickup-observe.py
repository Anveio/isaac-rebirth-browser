"""Read source-identified current room, grid and active entity descriptors.

All reads are bounded writable data in a directly owned blocked original child.
The ELF Game singleton and field offsets are reference tooling only.
"""
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
        if native_identity.native_binary_sha256(pid) != NATIVE_SHA:
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
            def clip_name(pointer):
                if not pointer: return None
                text = u64(pointer)
                count = u64(text - 24)
                if count > 128: raise ValueError('Unexpected animation name length')
                return read(text, count).decode('ascii')
            def entity(address):
                raw = read(address, 0x7d4)
                pair = lambda offset: list(struct.unpack_from('<ff', raw, offset))
                result = {'sequenceId': struct.unpack_from('<I', raw, 8)[0],
                    'type': struct.unpack_from('<I', raw, 0x10)[0],
                    'variant': struct.unpack_from('<I', raw, 0x14)[0],
                    'subtype': struct.unpack_from('<I', raw, 0x18)[0],
                    'removed': bool(raw[0x31f]), 'position': pair(0x448), 'velocity': pair(0x464),
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
                begin, end = struct.unpack_from('<QQ', raw, 0x328)
                sample_count = (end - begin) // 8
                if not 0 <= sample_count <= 128 or end < begin or (end - begin) % 8:
                    raise ValueError('Unexpected collision sample list')
                samples = read(begin, sample_count * 8) if sample_count else b''
                result['gridCollisionSamples'] = [list(struct.unpack_from('<ff', samples, i * 8)) for i in range(sample_count)]
                if result['type'] == 2:
                    if struct.unpack_from('<Q', raw, 0)[0] != 0x671270:
                        raise ValueError('Unexpected original Tear class')
                    # Current manager pointer membership is the owner of this
                    # list. Entity+31f is a separate death flag, not that list.
                    result['managerMember'] = True
                    result['dead'] = bool(raw[0x31f])
                    more = read(address + 0x7d4, 0x6c)
                    result['shotOrdinal'] = struct.unpack_from('<I', more, 0x64)[0]
                    result['baseDamage'] = struct.unpack_from('<f', more, 0x68)[0]
                    result['tearScale'] = struct.unpack_from('<f', more, 0x10)[0]
                    parent = struct.unpack_from('<Q', raw, 0x498)[0]
                    result['tearOwner'] = ({'type': u32(parent + 0x10), 'variant': u32(parent + 0x14),
                                           'sequenceId': u32(parent + 8)} if parent else None)
                    actor = address + 0x28
                    result['animation'] = clip_name(u64(actor + 0x18))
                if result['type'] == 5:
                    # GenericInit441cb0/441cbf owns these actual persistent
                    # RNG30 words. Read them directly, independently of the
                    # entity spawn seed and Pickup Init's temporary RNG35.
                    result['localRngWords'] = list(struct.unpack_from('<4I', raw, 0x7c0))
                    if context.get('pickupResourceSound') is True:
                        result['pickupManagerMember'] = True
                    more = read(address + 0xac0, 0x18)
                    result['pickupPrice'] = struct.unpack_from('<i', more, 0)[0]
                    result['collectionDelay'] = struct.unpack_from('<i', more, 0xc)[0]
                    result['pickupCooldown'] = struct.unpack_from('<i', more, 0x10)[0]
                    actor = address + 0x28
                    result['animation'] = clip_name(u64(actor + 0x18))
                    result['animationFrame'] = struct.unpack('<f', read(actor + 0x120, 4))[0]
                    result['animationPlaying'] = bool(read(actor + 0x124, 1)[0])
                    result['overlay'] = clip_name(u64(actor + 0x138))
                    result['overlayFrame'] = struct.unpack('<f', read(actor + 0x240, 4))[0]
                if result['type'] == 4:
                    more = read(address + 0x7d4, 0x14)
                    result['bombFuseUpdatesRemaining'] = struct.unpack_from('<i', more, 0)[0]
                    result['bombExplosionDamage'] = struct.unpack_from('<f', more, 4)[0]
                if struct.unpack_from('<Q', raw, 0)[0] == 0x66ef30:
                    more = read(address + 0xf50, 0x20)
                    result['npcState'] = struct.unpack_from('<I', more, 0)[0]
                    result['npcStateFrame'] = struct.unpack_from('<I', more, 8)[0]
                    result['npcFrameAge'] = struct.unpack_from('<I', more, 0x10)[0]
                    result['npcCountdown'] = struct.unpack_from('<I', more, 0x1c)[0]
                    if result['type'] == 17:
                        result['npcEntityFlags'] = struct.unpack_from('<I', raw, 0x318)[0]
                        result['npcAnchor'] = pair(0x440)
                        result['npcColor'] = list(struct.unpack_from('<11f', raw, 0x3cc))
                        result['canShutDoors'], result['boss'] = [bool(value) for value in read(address + 0x103c, 2)]
                        actor = address + 0x28
                        pointer = u64(actor + 0x18)
                        text = u64(pointer)
                        count = u64(text - 24)
                        if count > 128: raise ValueError('Unexpected Keeper animation name length')
                        result['animation'] = read(text, count).decode('ascii')
                        result['animationFrame'] = struct.unpack('<f', read(actor + 0x120, 4))[0]
                        result['animationPlaying'] = bool(read(actor + 0x124, 1)[0])
                return result

            game = u64(0x8ecd30)
            system = u64(0x8ed560)
            report['nativePhase'] = {'systemIterations': u32(system + 0x2efa0), 'fullGameUpdates': u32(game + 0xce9f4)}
            if context.get('pickupResourceSound') is True:
                report['systemCounter'] = u32(system + 0x2efa0)
            room = u64(game + 0x8180)
            raw = read(room, 0x30)
            width, height = struct.unpack_from('<II', raw, 0x14)
            if not 1 <= width <= 28 or not 1 <= height <= 16 or width * height > 448:
                raise ValueError('Unexpected room grid shape')
            report['room'] = {'type': struct.unpack_from('<I', raw, 0x10)[0], 'gridWidth': width, 'gridHeight': height,
                              'backdropType': u32(room + 0x1ae0),
                              'boundsMin': list(struct.unpack_from('<ff', raw, 0x1c)),
                              'boundsMax': list(struct.unpack_from('<ff', raw, 0x24)),
                              'renderOrigin': list(struct.unpack('<ff', read(room + 0x15b0, 8))),
                              'renderScroll': list(struct.unpack('<ff', read(room + 0x15b8, 8))),
                              'cameraStyle': u32(system + 0x2dfb4)}
            if (width, height) == (15, 16):
                # Source camera diagnostics are independent readbacks for the
                # following camera leaf; they never supply runtime state.
                camera = u64(room + 0x15a8)
                if u64(camera + 0x98) != room:
                    raise ValueError('Native camera controller must own the current room')
                histories = []
                for offset, expected in ((0, 60), (0x18, 90), (0x30, 300), (0x48, 60), (0x60, 4), (0x78, 120)):
                    pointer, capacity, size, head = struct.unpack('<Qiii', read(camera + offset, 20))
                    if capacity != expected or not 0 <= size <= capacity or not 0 <= head < capacity:
                        raise ValueError('Unexpected source camera history bounds')
                    data = read(pointer, capacity * 8)
                    histories.append({'offset': offset, 'capacity': capacity, 'size': size, 'head': head,
                        'samples': [list(struct.unpack_from('<ff', data, ((head + i) % capacity) * 8)) for i in range(size)]})
                report['room']['cameraDiagnostics'] = {
                    'lastPlayerPosition': list(struct.unpack('<ff', read(camera + 0x90, 8))),
                    'position': list(struct.unpack('<ff', read(camera + 0xa0, 8))),
                    'alternatePosition': list(struct.unpack('<ff', read(camera + 0xa8, 8))),
                    'alternateVelocity': list(struct.unpack('<ff', read(camera + 0xb0, 8))),
                    'specialTargetSupplied': bool(read(camera + 0xb8, 1)[0]), 'histories': histories}
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
                if cells[-1]['type'] == 16:
                    cells[-1]['blownOpen'] = bool(read(pointer + 0x611, 1)[0])
            report['grid'] = cells
            report['gridSelectors'] = list(struct.unpack('<' + 'I' * (width * height),
                read(room + 0xe98, width * height * 4)))
            report['pathCosts'] = list(struct.unpack('<' + 'i' * 448,
                read(room + 0xe98, 0x700)))
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
                item['bombInventory'] = struct.unpack_from('<I', raw, 0x1c)[0]
                item['bombPlacementCooldown'] = struct.unpack('<i', read(pointer + 0xd90, 4))[0]
                if context.get('pickupResourceSound') is True:
                    if raw[0x18] not in (0, 1):
                        raise ValueError('Actual PlayerBD0 boolean representation unavailable')
                    item['goldenKeyBd0Byte'] = raw[0x18]
                    item['goldenKey'] = raw[0x18] == 1
                item['keyInventory'] = u32(pointer + 0xbcc)
                item['coinInventory'] = u32(pointer + 0xbd8)
                item['maxRedHp'] = u32(pointer + 0xbb4)
                item['controlsEnabled'] = bool(read(pointer + 0x7d4, 1)[0])
                if (width, height) == (15, 16):
                    item['cameraInputDiagnostics'] = {
                        'vectorDa8': list(struct.unpack('<ff', read(pointer + 0xda8, 8))),
                        'vectorDb4': list(struct.unpack('<ff', read(pointer + 0xdb4, 8))),
                        'modeC08': u32(pointer + 0xc08), 'modeD80': u32(pointer + 0xd80)}
                players.append(item)
            report['players'] = players
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError, struct.error) as error:
        report['status'] = 'fail'; report['error'] = str(error)
    finally:
        if stopped: os.kill(pid, signal.SIGCONT)
    return report
