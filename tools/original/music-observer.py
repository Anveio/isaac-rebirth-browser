"""Read-only Music controller snapshots in a directly owned, presentation-blocked original.

Fields are source-located and bounded; observations
are snapshots, not an inferred event stream, and never become browser inputs.
"""
import hashlib
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]
NATIVE_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
identity_spec = importlib.util.spec_from_file_location('native_identity', ROOT / 'tools/original/native-identity.py')
native_identity = importlib.util.module_from_spec(identity_spec)
identity_spec.loader.exec_module(native_identity)
MUSIC_SHA = '679d268e0c1fb9b57b079c6a1476d6ee6b6634a87d70a293bce1e518fcf1d92a'
MAX_BYTES = 2 * 1024 * 1024
static_image = (ROOT / '.cache/original-runtime/offline-depot/isaac.x64').read_bytes()
if hashlib.sha256(static_image).hexdigest() != NATIVE_SHA:
    raise ValueError('Unexpected pinned ELF for source-class validation')
section_at = struct.unpack_from('<Q', static_image, 0x28)[0]
section_stride, section_count = struct.unpack_from('<HH', static_image, 0x3a)
static_data_sections = []
for index in range(section_count):
    at = section_at + index * section_stride
    kind = struct.unpack_from('<I', static_image, at + 4)[0]
    flags, address, offset, length = struct.unpack_from('<QQQQ', static_image, at + 8)
    if kind == 1 and flags & 2 and not flags & 5:
        static_data_sections.append((address, offset, length))


def source_vtable_slot(address, slot):
    for base, offset, length in static_data_sections:
        if base <= address + slot and address + slot + 8 <= base + length:
            return struct.unpack_from('<Q', static_image, offset + address + slot - base)[0]
    raise ValueError('Source class vtable is not in the pinned readonly ELF data')

spec = importlib.util.spec_from_file_location('audio_source_maps', ROOT / 'tools/original/state-observer.py')
maps_reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(maps_reader)
music_bytes = (ROOT / 'references/isaac/rebirth/resources/music.xml').read_bytes()
if hashlib.sha256(music_bytes).hexdigest() != MUSIC_SHA:
    raise ValueError('Unexpected pinned music catalogue')
music_xml = ET.fromstring(music_bytes)
music_catalog = {int(track.attrib['id']): dict(track.attrib) for track in music_xml}
if len(music_catalog) != 45:
    raise ValueError('Unexpected music catalogue count')
backend_owner = None
backend_ids = {}


def observe(pid, context):
    global backend_owner, backend_ids
    report = {
        'schema': 1, 'status': 'running', 'memoryWritten': False,
        'executableMappingsRead': False, 'bytesRead': 0,
        'nativeBinarySha256': NATIVE_SHA, 'musicCatalogSha256': MUSIC_SHA,
        'checkpoint': context.get('label'), 'originalPid': pid,
        'scope': 'Bounded XML-verified Music controller, wrapper and queued-path snapshots; no inferred event multiplicity or fixed completion timing',
    }
    stopped = False
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Requires positive original child PID at its public presentation gate')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = next(line.split(':', 1)[1].strip() for line in status.splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid():
            raise PermissionError('Requires directly owned original child')
        if native_identity.native_binary_sha256(pid) != NATIVE_SHA:
            raise ValueError('Unexpected original executable hash')
        if backend_owner != pid:
            backend_owner, backend_ids = pid, {}
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while not all(
            task.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt'
            for task in Path(f'/proc/{pid}/task').iterdir()
        ):
            if time.monotonic() > deadline:
                raise TimeoutError('Original threads did not stop')
            time.sleep(.001)
        report['hostMonotonicNs'] = time.monotonic_ns()
        mappings = maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def allowed(address, length):
                return any(m['start'] <= address and address + length <= m['end'] for m in mappings)

            def read(address, length):
                if length < 0 or report['bytesRead'] + length > MAX_BYTES:
                    raise ValueError('Audio observation read budget exceeded')
                if not allowed(address, length):
                    raise PermissionError('Audio observation left readable writable nonexecutable source data')
                value = os.pread(fd, length, address)
                if len(value) != length:
                    raise OSError('Short bounded audio read')
                report['bytesRead'] += length
                return value

            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            system = u64(0x8ed560)
            game = u64(0x8ecd30)
            counter = struct.unpack('<I', read(system + 0x2efa0, 4))[0]
            report['systemCounter'] = counter
            menu = u64(0x8ed570)
            system_state = struct.unpack('<I', read(system, 4))[0]
            system_pending = read(system + 0x2f2c8, 1)[0]
            if system_pending not in (0, 1) or (system_state == 1 and not menu):
                raise ValueError('Invalid source pending/menu ownership')
            report['runStartRequest'] = {'menuPresent': bool(menu), 'systemPending': bool(system_pending)}
            if menu:
                fade_requested = read(menu + 1, 1)[0]
                fade_alpha = struct.unpack('<f', read(menu + 4, 4))[0]
                if fade_requested not in (0, 1) or not math.isfinite(fade_alpha):
                    raise ValueError('Invalid source menu request/fade')
                report['runStartRequest'].update({'fading': bool(fade_requested), 'fadeAlpha': fade_alpha})
            def gnu_string(pointer, maximum=512):
                if not pointer:
                    raise ValueError('Missing configured GNU string')
                length = u64(pointer - 24)
                if length > maximum:
                    raise ValueError('Unexpected configured GNU string length')
                return read(pointer, length).decode('ascii')

            music = system + 0x114d0
            manager = read(music, 0xf8)
            active, current, queued = struct.unpack_from('<i', manager, 0)[0], *struct.unpack_from('<ii', manager, 0xe4)
            pitch, target_pitch = struct.unpack_from('<ff', manager, 0xec)
            if active not in (0, 1) or not all(-1 <= value <= 101 for value in (current, queued)) \
                    or not all(math.isfinite(value) for value in (pitch, target_pitch)):
                raise ValueError('Invalid bounded native music controller fields')
            music_report = {'managerRoot': 'SystemManager + 0x114d0', 'catalogSha256': MUSIC_SHA,
                'activeChannel': active, 'currentId': current, 'queuedId': queued,
                'pitch': pitch, 'targetPitch': target_pitch, 'layerEnabled': bool(manager[0xe0]),
                'channels': [], 'tracks': []}
            if manager[0xe0] not in (0, 1):
                raise ValueError('Invalid native music layer enable flag')
            for name, offset in [('main0', 8), ('main1', 0x50), ('layer', 0x98)]:
                channel = manager[offset:offset + 0x48]
                pointer = struct.unpack_from('<Q', channel, 0)[0]
                values = struct.unpack_from('<fff', channel, 0x30)
                wrapper_gain, wrapper_pitch = struct.unpack_from('<f', channel, 0x18)[0], struct.unpack_from('<f', channel, 0x20)[0]
                if not all(math.isfinite(value) for value in (*values, wrapper_gain, wrapper_pitch)) \
                        or channel[0x3c] not in (0, 1) or channel[0x14] not in (0, 1) or wrapper_pitch <= 0:
                    raise ValueError('Invalid native music channel fields')
                row = {'name': name, 'volume': values[0], 'targetVolume': values[1], 'fadeRate': values[2],
                    'playing': bool(channel[0x3c]), 'updatesSincePrepare': struct.unpack_from('<i', channel, 0x40)[0],
                    'wrapperLoop': bool(channel[0x14]), 'wrapperGain': wrapper_gain,
                    'wrapperPitch': wrapper_pitch, 'backendPresent': bool(pointer)}
                if pointer:
                    vtable = u64(pointer)
                    row['backendIdentity'] = f"sample-source-{backend_ids.setdefault(pointer, len(backend_ids) + 1)}"
                    row['backendClassVtable'] = hex(vtable)
                    row['backendCallbacks'] = {hex(slot): hex(source_vtable_slot(vtable, slot))
                                              for slot in (0x58, 0x60, 0x68, 0x70, 0x80, 0xa0)}
                    if vtable != 0x67d070 or source_vtable_slot(vtable, 0x60) != 0x610cf0:
                        raise ValueError('Unexpected source-backed streaming class layout')
                    # 610cf0 reads this inherited public OpenAL source ID.
                    backend = read(pointer, 0x80)
                    if backend[8] not in (0, 1) or backend[9] not in (0, 1):
                        raise ValueError('Invalid source streaming decoder flags')
                    row['openalSource'] = struct.unpack_from('<I', backend, 0x78)[0]
                    row['backendLoop'] = bool(backend[9])
                    row['backendLoaded'] = bool(backend[8])
                    # 629e30/629a90 identify the deque's begin/end data and
                    # block-map pointers. Queue records are 24 bytes; their
                    # interned paths are ordinary bounded C strings, not GNU.
                    begin, end = struct.unpack_from('<Q', backend, 0x28)[0], struct.unpack_from('<Q', backend, 0x48)[0]
                    begin_node, end_node = struct.unpack_from('<Q', backend, 0x40)[0], struct.unpack_from('<Q', backend, 0x60)[0]
                    row['queuedStreams'] = []
                    if begin_node != end_node:
                        raise ValueError('Streaming queue spans multiple blocks outside this bounded probe scope')
                    if end < begin or (end - begin) % 24 or (end - begin) // 24 > 8:
                        raise ValueError('Unexpected bounded source streaming queue')
                    for at in range(begin, end, 24):
                        record = read(at, 24)
                        text = struct.unpack_from('<Q', record, 0)[0]
                        path = bytearray()
                        for index in range(512):
                            byte = read(text + index, 1)
                            if byte == b'\0':
                                break
                            path.extend(byte)
                        else:
                            raise ValueError('Source queued path lacks bounded terminator')
                        if record[8] not in (0, 1):
                            raise ValueError('Invalid source queued stream loop')
                        row['queuedStreams'].append({'path': path.decode('ascii'), 'loop': bool(record[8])})
                music_report['channels'].append(row)
            records = read(music + 0xf8, 102 * 56)
            for identifier in range(102):
                offset = identifier * 56
                native_id = struct.unpack_from('<I', records, offset)[0]
                expected = music_catalog.get(identifier)
                if not expected:
                    if native_id != 0:
                        raise ValueError('Unexpected configured native music ID')
                    continue
                if native_id != identifier:
                    raise ValueError('Native music ID differs from pinned XML')
                track = {'id': identifier}
                for field, at in [('name', 8), ('intro', 0x10), ('path', 0x18), ('layerintro', 0x20), ('layer', 0x28)]:
                    value = gnu_string(struct.unpack_from('<Q', records, offset + at)[0])
                    authored = expected.get(field, '')
                    if field != 'name' and authored:
                        authored = music_xml.attrib['root'] + authored
                    if value.lower() != authored.lower():
                        raise ValueError(f'Native music {identifier}/{field} differs from pinned XML')
                    track[field] = value
                loop = records[offset + 0x30]
                if loop not in (0, 1) or bool(loop) != (expected['loop'] == 'true'):
                    raise ValueError('Native music loop differs from pinned XML')
                track['loop'] = bool(loop)
                music_report['tracks'].append(track)
            if len(music_report['tracks']) != 45:
                raise ValueError('Native music catalogue accounting incomplete')
            report['music'] = music_report
            report['music']['globalMusicVolume'] = struct.unpack('<f', read(system + 0x2df90, 4))[0]
            report['music']['gainMultiplier'] = struct.unpack('<f', read(0x8ed5b0, 4))[0]
            report['music']['systemState'] = struct.unpack('<I', read(system, 4))[0]
            music_enabled = read(music + 0x1748, 1)[0]
            odd_dispatch = read(system + 0x2dfd4, 1)[0]
            if music_enabled not in (0, 1) or odd_dispatch not in (0, 1) \
                    or not math.isfinite(report['music']['globalMusicVolume']) or not 0 <= report['music']['globalMusicVolume'] <= 1 \
                    or report['music']['gainMultiplier'] not in (0.0, 1.0) or report['music']['systemState'] not in (1, 2, 3):
                raise ValueError('Invalid source music global options/dispatch fields')
            report['music']['musicEnabled'] = bool(music_enabled)
            report['music']['oddSystemDispatchEnabled'] = bool(odd_dispatch)
            if game and context.get('publicLogState', {}).get('stage') == 1:
                level_stage = struct.unpack('<I', read(game, 4))[0]
                alternate = read(game + 4, 1)[0]
                transition = game + 0x95c0
                transition_raw = read(transition, 0x1c)
                animation_enabled = read(transition + 0x3f8, 1)[0]
                if alternate not in (0, 1) or animation_enabled not in (0, 1):
                    raise ValueError('Invalid source stage/transition flags')
                report['level'] = {'stage': level_stage, 'alternate': bool(alternate)}
                report['roomEntry'] = {
                    'state': struct.unpack_from('<i', transition_raw, 0)[0],
                    'counter': struct.unpack_from('<i', transition_raw, 4)[0],
                    'duration': struct.unpack_from('<i', transition_raw, 8)[0],
                    'kind': struct.unpack_from('<i', transition_raw, 0x18)[0],
                    'animationEnabled': bool(animation_enabled),
                }
                if animation_enabled:
                    sprite = transition + 0x40
                    animation = u64(sprite + 0x18)
                    if not animation:
                        raise ValueError('Enabled transition lacks source animation')
                    frame_count = struct.unpack('<I', read(animation + 0x34, 4))[0]
                    animation_loop = read(animation + 0x38, 1)[0]
                    playing = read(sprite + 0x124, 1)[0]
                    frame, speed = struct.unpack('<f', read(sprite + 0x120, 4))[0], struct.unpack('<f', read(sprite + 0x2d8, 4))[0]
                    if not 1 <= frame_count <= 36000 or animation_loop not in (0, 1) or playing not in (0, 1) \
                            or not math.isfinite(frame) or not math.isfinite(speed):
                        raise ValueError('Invalid bounded source transition animation')
                    report['roomEntry']['animation'] = {'name': gnu_string(u64(animation)),
                        'frames': frame_count, 'loop': bool(animation_loop), 'playing': bool(playing), 'frame': frame, 'speed': speed}
                state, popup, selection = struct.unpack('<III', read(game + 0x8ce18, 12))
                if state not in (0, 1, 2):
                    raise ValueError('Unknown source pause controller state')
                report['nativePause'] = {'state': state, 'paused': state != 0,
                                        'popup': bool(popup & 255), 'selection': selection}
            report['status'] = 'pass'
        finally:
            os.close(fd)
    except (OSError, ValueError, KeyError, TypeError, TimeoutError, struct.error) as error:
        report['status'] = 'fail'
        report['error'] = str(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report
