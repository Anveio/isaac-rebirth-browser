"""Read-only SFX table snapshots in a directly owned, presentation-blocked original.

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
identity_spec = importlib.util.spec_from_file_location('native_identity', ROOT / 'tools/original/native-identity.py')
native_identity = importlib.util.module_from_spec(identity_spec)
identity_spec.loader.exec_module(native_identity)
NATIVE_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
SOUNDS_SHA = '8e4a4ce10de0835c86df68006b156a62ce3b9d2a9be40ee8b9b541a30b42e30d'
SLOT_COUNT = 324
SLOT_STRIDE = 0x158
TABLE_OFFSET = 0x12c20
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
catalog_bytes = (ROOT / 'references/isaac/rebirth/resources/sounds.xml').read_bytes()
if hashlib.sha256(catalog_bytes).hexdigest() != SOUNDS_SHA:
    raise ValueError('Unexpected pinned base sound catalogue')
catalog = {
    int(sound.attrib['id']): [
        'resources/sfx/' + sample.attrib['path'].lower()
        for sample in sound
    ]
    for sound in ET.fromstring(catalog_bytes)
}
if len(catalog) != 169 or sum(map(len, catalog.values())) != 303:
    raise ValueError('Unexpected base sound catalogue identities or sample references')
backend_owner = None
backend_ids = {}


def observe(pid, context):
    global backend_owner, backend_ids
    report = {
        'schema': 1, 'status': 'running', 'memoryWritten': False,
        'executableMappingsRead': False, 'bytesRead': 0,
        'nativeBinarySha256': NATIVE_SHA, 'soundCatalogSha256': SOUNDS_SHA,
        'checkpoint': context.get('label'), 'originalPid': pid,
        'scope': 'Bounded SFX table/sample-wrapper snapshots; no inferred event stream or music lifecycle claim',
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
            global_volume = struct.unpack('<f', read(system + 0x2df94, 4))[0]
            if not math.isfinite(global_volume):
                raise ValueError('Invalid source global SFX volume')
            table = system + TABLE_OFFSET
            raw = read(table, SLOT_COUNT * SLOT_STRIDE)
            report.update({
                'systemCounter': counter, 'cooldownTick': struct.unpack('<i', struct.pack('<I', counter))[0] >> 1,
                'mtIndex': struct.unpack('<I', read(0x8e6b64, 4))[0],
                'mtState': list(struct.unpack('<624Q', read(0x8ed680, 624 * 8))),
                'globalSfxVolume': global_volume, 'managerRoot': 'SystemManager + 0x12c20',
                'gamePresent': bool(game), 'configuredSounds': 0, 'sampleReferences': 0,
                'catalogCountsMatch': True, 'pathMismatches': [], 'slots': [],
            })
            for identifier in range(SLOT_COUNT):
                at = identifier * SLOT_STRIDE
                count = struct.unpack_from('<I', raw, at + 0x148)[0]
                expected = catalog.get(identifier, [])
                if count != len(expected) or count > 5:
                    raise ValueError(f'Source sound {identifier} sample count {count} differs from pinned authored count {len(expected)}')
                if not count:
                    continue
                last_tick, next_allowed = struct.unpack_from('<ii', raw, at)
                last_volume = struct.unpack_from('<f', raw, at + 0x14c)[0]
                loaded = raw[at + 0x150]
                if loaded not in (0, 1) or not math.isfinite(last_volume):
                    raise ValueError(f'Invalid source admission fields for sound {identifier}')
                slot = {'id': identifier, 'lastTick': last_tick, 'nextAllowed': next_allowed,
                        'count': count, 'lastVolume': last_volume, 'loaded': bool(loaded), 'samples': []}
                for index in range(count):
                    sample_at = at + 0x10 + index * 64
                    loop = raw[sample_at + 0x14]
                    gain = struct.unpack_from('<f', raw, sample_at + 0x18)[0]
                    pitch = struct.unpack_from('<f', raw, sample_at + 0x20)[0]
                    if loop not in (0, 1) or not all(math.isfinite(v) for v in (gain, pitch)):
                        raise ValueError(f'Invalid source sample wrapper {identifier}/{index}')
                    # Config-owned GNU strings precede their sample wrapper by 8.
                    pointer = struct.unpack_from('<Q', raw, at + 8 + index * 64)[0]
                    if not pointer or not allowed(pointer - 24, 8):
                        raise ValueError(f'Source sample path {identifier}/{index} has no bounded GNU string')
                    length = u64(pointer - 24)
                    if not 1 <= length <= 512:
                        raise ValueError('Unexpected source sample path length')
                    path = read(pointer, length).decode('ascii')
                    if path.lower() != expected[index]:
                        report['pathMismatches'].append({'id': identifier, 'index': index,
                                                       'expected': expected[index], 'actual': path})
                    source_pointer = struct.unpack_from('<Q', raw, sample_at)[0]
                    sample = {'index': index, 'path': path, 'loop': bool(loop),
                              'gain': gain, 'pitch': pitch, 'nativeSourcePresent': bool(source_pointer)}
                    if source_pointer:
                        backend = read(source_pointer, 8)
                        vtable = struct.unpack_from('<Q', backend, 0)[0]
                        callbacks = {hex(offset): hex(source_vtable_slot(vtable, offset))
                                     for offset in (0x58, 0x60, 0x70, 0x80, 0xa0)}
                        # Keep ownership identity distinct from backend-layout
                        # claims. The SoundSource interface can wrap a platform
                        # object; only the actual callback table is recorded.
                        identity = backend_ids.setdefault(source_pointer, len(backend_ids) + 1)
                        sample.update({'backendIdentity': f'sample-source-{identity}',
                                       'backendClassVtable': hex(vtable),
                                       'backendCallbacks': callbacks})
                    slot['samples'].append(sample)
                report['slots'].append(slot)
                report['configuredSounds'] += 1
                report['sampleReferences'] += count
            if report['pathMismatches']:
                raise ValueError('Source sample paths differ from pinned authored base catalogue')
            if report['configuredSounds'] != 169 or report['sampleReferences'] != 303:
                raise ValueError('Source audio catalogue accounting is incomplete')
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
