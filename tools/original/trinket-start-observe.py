"""Additive Challenge startup Kind2 data observer; no observations are inputs.

Uses the existing owned presentation gate. No Native function is invoked and
no executable/read-only mapping, resource payload or process memory is written.
Presentation-end snapshots do not witness intra-Full call order or transient18.
"""
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
_composition_spec = importlib.util.spec_from_file_location('trinket_start_composition', ROOT/'tools/original/composition-observe.py')
composition = importlib.util.module_from_spec(_composition_spec)
_composition_spec.loader.exec_module(composition)
PIN = composition.base.native_identity.NATIVE_SHA


def rare_owner(pid, context):
    result = {'schema': 1, 'status': 'running', 'nativeBinarySha256': PIN,
              'memoryWritten': False, 'executableMappingsRead': False,
              'bytesRead': 0, 'maxBytes': 131072,
              'eventScope': 'Actual presentation-end fields only; intra-Full pool/contact calls unobserved',
              'positionScope': 'Raw448 and raw450 retained; no label-based Full/Fast projection',
              'rawAB8Scope': 'Raw observed integer; not a runtime-owned loose350 Save value'}
    stopped = False
    try:
        if context.get('gateBlocked') is not True or type(pid) is not int or pid <= 0:
            raise PermissionError('Rare TNT requires a presentation-blocked owned child')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = next(line.split(':', 1)[1] for line in status.splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or composition.base.native_identity.native_binary_sha256(pid) != PIN:
            raise PermissionError('Rare TNT parent/binary identity differs')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while not all(task.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt'
                      for task in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic() > deadline:
                raise TimeoutError('Original threads did not stop')
            time.sleep(.001)
        maps = composition.base.maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, size):
                if type(size) is not int or size < 0 or result['bytesRead'] + size > result['maxBytes']:
                    raise ValueError('Rare TNT read budget exceeded')
                if not any(row['start'] <= address and address + size <= row['end'] for row in maps):
                    raise PermissionError('Rare TNT read outside writable mappings')
                value = os.pread(fd, size, address)
                result['bytesRead'] += size
                if len(value) != size:
                    raise OSError('Short rare TNT read')
                return value

            def u64(address): return struct.unpack('<Q', read(address, 8))[0]
            def u32(address): return struct.unpack('<I', read(address, 4))[0]
            def i32(address): return struct.unpack('<i', read(address, 4))[0]
            def boolean(address):
                value = read(address, 1)[0]
                if value not in (0, 1):
                    raise ValueError('Nonboolean actual owner')
                return bool(value)
            def floats(address, count):
                raw = read(address, count * 4)
                values = list(struct.unpack('<' + 'f' * count, raw))
                if not all(math.isfinite(value) for value in values):
                    raise ValueError('Nonfinite actual float owner')
                return {'values': values, 'words': list(struct.unpack('<' + 'I' * count, raw))}
            def string(address, maximum=128):
                pointer = u64(address)
                if not pointer:
                    raise ValueError('Missing actual string owner')
                length = u64(pointer - 24)
                if length > maximum:
                    raise ValueError('Actual string exceeds bound')
                return read(pointer, length).decode('ascii') if length else ''
            def rng(address): return list(struct.unpack('<4I', read(address, 16)))
            def sprite(address):
                loaded = boolean(address + 0x2dd)
                definition = u64(address + 0x18)
                overlay = u64(address + 0x138)
                sheet_count = u32(address + 0x270)
                if sheet_count > 32:
                    raise ValueError('Sprite sheet count exceeds bound')
                sheets = u64(address + 0x268)
                return {'loadedFlag': loaded,
                        'animation': string(definition) if definition else None,
                        'frame': floats(address + 0x120, 1) if definition else None,
                        'playing': boolean(address + 0x124) if definition else None,
                        'overlay': string(overlay) if overlay else None,
                        'overlayFrame': floats(address + 0x240, 1) if overlay else None,
                        'overlayPlaying': boolean(address + 0x244) if overlay else None,
                        'sheets': [string(sheets + index * 0x70 + 8, 256)
                                   for index in range(sheet_count)],
                        'inactiveScope': None if definition else 'No current clip owner; frame/playing unobserved'}

            system, game = u64(0x8ed560), u64(0x8ecd30)
            result['systemState'] = i32(system)
            result['selectedProfile'] = u32(system + 8)
            profile = list(read(system + 0x30, 179))
            if any(value not in (0, 1) for value in profile):
                raise ValueError('Actual179 profile bytes outside bool domain')
            result['currentProfileAchievements'] = profile
            words = list(struct.unpack('<624Q', read(0x8ed680, 624 * 8)))
            cursor = u32(0x8e6b64)
            if cursor > 624 or any(word > 0xffffffff for word in words):
                raise ValueError('Actual MT owner outside32-bit624-word domain')
            result['sharedMt'] = {'index': cursor, 'words': words}
            result['systemCounter'] = u32(system + 0x2efa0)
            result['gamePresent'] = bool(game) and result['systemState'] == 2
            if not result['gamePresent']:
                result['status'] = 'skip'
                result['scope'] = 'Explicit pre/post-run lifetime; live pool/Player/body unobserved'
                return result
            result['gameUpdateCount'] = u32(game + 0xce9f4)
            result['pauseState'] = u32(game + 0x8ce18)
            pool, config_root = game + 0x8bd0, system + 0x2dfe0
            result['trinketSelectionRngWords'] = rng(pool + 0x49c)
            entries = []
            for index in range(62):
                entry = pool + 0x7c4 + index * 8
                actual_id = i32(entry)
                if actual_id != index:
                    raise ValueError('Actual62 pool IDs differ from initialized source layout')
                config = u64(config_root + 0xad8 + index * 8)
                kind, config_id, achievement = (i32(config), i32(config + 4), i32(config + 0x20)) if config else (None, None, None)
                if config and (kind != 2 or config_id != index or not -1 <= achievement < 179):
                    raise ValueError('Actual Kind2 config identity/prerequisite differs')
                entries.append({'id': actual_id, 'available': boolean(entry + 4),
                                'eligible': boolean(entry + 5), 'configKind': kind,
                                'configId': config_id, 'achievement': achievement,
                                'configPresent': bool(config)})
            result['trinketPool'] = {'entries': entries, 'remainingCount': i32(pool + 0x9b4)}
            begin, end = u64(game + 0xa148), u64(game + 0xa150)
            if end < begin or (end - begin) % 8 or end - begin > 4 * 8:
                raise ValueError('Player vector outside bounded Source domain')
            player_pointers = list(struct.unpack('<' + 'Q' * ((end - begin) // 8), read(begin, end - begin)))
            result['playerCount'] = len(player_pointers)
            result['players'] = []
            for pointer in player_pointers:
                if u64(pointer) != 0x670c90:
                    raise ValueError('Unexpected actual Player class')
                held = list(struct.unpack('<2i', read(pointer + 0xe14, 8)))
                if any(not 0 <= item_id < 62 for item_id in held):
                    raise ValueError('Actual held trinket ID outside domain')
                capacity_count = i32(pointer + 0xe1c + 139 * 4)
                queued = u64(pointer + 0x3058)
                config = {'present': False, 'kind': None, 'id': None, 'charge': None, 'touched': None}
                touched_raw = read(pointer + 0x3064, 1)[0]
                if queued:
                    kind, item_id = i32(queued), i32(queued + 4)
                    if kind not in (1, 2, 3, 4) or not 1 <= item_id <= (61 if kind == 2 else 346):
                        raise ValueError('Actual queued config kind/ID outside domain')
                    config = {'present': True, 'kind': kind, 'id': item_id,
                              'charge': i32(pointer + 0x3060), 'touched': boolean(pointer + 0x3064)}
                result['players'].append({'sequenceId': u32(pointer + 8), 'spawnSeed': u32(pointer + 0x7d0),
                    'entityType': u32(pointer + 0x10), 'variant': u32(pointer + 0x14),
                    'sourcePlayerType': u32(pointer + 0xbfc), 'heldTrinketSlots': held,
                    'rawHave139Count': capacity_count, 'activeTrinketCapacity': 2 if capacity_count > 0 else 1,
                    'child41RngWords': rng(pointer + 0x29d4 + 41 * 16),
                    'collectible41CountRaw': i32(pointer + 0xe1c + 41 * 4),
                    'queued': config, 'queuedChargeRaw': i32(pointer + 0x3060), 'queuedTouchedRaw': touched_raw,
                    'pendingCacheFlags': u32(pointer + 0xd6c), 'pickupAnimationGuard': boolean(pointer + 0xbf4),
                    'mainSprite': sprite(pointer + 0x28), 'copiedPickupSprite': sprite(pointer + 0x7e0)})
            room = u64(game + 0x8180)
            result['roomAge'] = {'loaded': boolean(room), 'firstVisitByte': read(room + 1, 1)[0],
                                 'initializationGameUpdate': u32(room + 0x159c)}
            width, height = struct.unpack('<II', read(room + 0x14, 8))
            if not 3 <= width <= 28 or not 3 <= height <= 16 or width * height > 448:
                raise ValueError('Actual room grid dimensions outside bound')
            grids = struct.unpack('<' + 'Q' * (width * height), read(room + 0x30, width * height * 8))
            result['tntGrids'] = [{'index': index, 'state': u32(pointer + 0x10), 'spawnSeed': u32(pointer + 0x18)}
                                 for index, pointer in enumerate(grids) if pointer and u32(pointer + 8) == 12]
            count = u32(room + 0x1644)
            if count > 512:
                raise ValueError('Actual room actor list outside bound')
            pointers = list(struct.unpack('<' + 'Q' * count, read(u64(room + 0x1638), count * 8))) if count else []
            if any(pointer == 0 for pointer in pointers) or len(set(pointers)) != count:
                raise ValueError('Actual counted actor list null/duplicate owner')
            result['actorOrder'] = [{'ordinal': ordinal, 'sequenceId': u32(pointer + 8),
                'type': u32(pointer + 0x10), 'variant': u32(pointer + 0x14),
                'subtype': u32(pointer + 0x18), 'spawnSeed': u32(pointer + 0x7d0)}
                for ordinal, pointer in enumerate(pointers)]
            bodies = []
            for pointer, owner in zip(pointers, result['actorOrder']):
                if owner['type'] != 5 or owner['variant'] != 350:
                    continue
                if len(bodies) >= 16 or u64(pointer) != 0x66f470:
                    raise ValueError('Actual350 actor bound/class differs')
                bodies.append({**owner, 'position448': floats(pointer + 0x448, 2),
                    'backupPosition450': floats(pointer + 0x450, 2), 'velocity464': floats(pointer + 0x464, 2),
                    'anchor440': floats(pointer + 0x440, 2), 'configuredFriction': floats(pointer + 0x46c, 1),
                    'currentFriction': floats(pointer + 0x470, 1), 'collisionRadius': floats(pointer + 0x474, 1),
                    'collisionMass': floats(pointer + 0x480, 1), 'gridCollisionClass': u32(pointer + 0x340),
                    'entityCollisionClass': u32(pointer + 0x344), 'active': boolean(pointer + 0x31e),
                    'dead31F': boolean(pointer + 0x31f), 'visible': boolean(pointer + 0x31d),
                    'birthGameUpdate': u32(pointer + 0x42c), 'persistentRng30': rng(pointer + 0x7c0),
                    'sourceSpawnerType': u32(pointer + 0x1c), 'sourceSpawnerVariant': u32(pointer + 0x20),
                    'rawAB8Charge': i32(pointer + 0xab8), 'rawABCTouched': read(pointer + 0xabc, 1)[0],
                    'rawABD': read(pointer + 0xabd, 1)[0], 'priceAC0': i32(pointer + 0xac0),
                    'collectionDelayACC': i32(pointer + 0xacc), 'contactDelayAD0': i32(pointer + 0xad0),
                    'sprite': sprite(pointer + 0x28)})
            result['bodies350'] = bodies
            result['status'] = 'pass'
        finally:
            os.close(fd)
    except Exception as error:
        result['status'] = 'fail'
        result['error'] = repr(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return result


def observe(pid, context):
    result = composition.observe(pid, context)
    #Retain all common observations and failures; the additive owner never
    #changes prior wire fields or converts an unavailable owner into zero.
    if result.get('gamePresent') is True and result.get('status') == 'pass':
        result['trinketStartup'] = rare_owner(pid, context)
        if result['trinketStartup']['status'] != 'pass':
            result['status'] = 'fail'
    else:
        result['trinketStartup'] = {'schema': 1, 'status': 'skip', 'scope': 'Gameplay owner unavailable'}
    return result
