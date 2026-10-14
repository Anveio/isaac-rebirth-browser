"""Read-only current17/42 bodies and SFX72 in a directly owned original."""
import hashlib
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('blood_start_paired', Path(__file__).with_name('foundation-paired-observer.py'))
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)
NATIVE_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'


def observe(pid, context):
    result = paired.observe(pid, context)
    if result['gameplay']['status'] != 'pass':
        result['bloodStart'] = {'status': 'skip', 'memoryWritten': False}
        return result
    report = {'status': 'pass', 'memoryWritten': False, 'executableMappingsRead': False,
              'bytesRead': 0, 'nativeBinarySha256': NATIVE_SHA, 'effects': []}
    stopped = False
    try:
        status = Path(f'/proc/{pid}/status').read_text()
        parent = next(row.split(':', 1)[1].strip() for row in status.splitlines() if row.startswith('PPid:'))
        if int(parent) != os.getpid() or context.get('gateBlocked') is not True:
            raise PermissionError('Requires a directly owned blocked original')
        if hashlib.sha256(Path(f'/proc/{pid}/exe').read_bytes()).hexdigest() != NATIVE_SHA:
            raise ValueError('Original executable identity changed')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline:
                raise TimeoutError('Original did not stop')
            time.sleep(.001)
        maps = paired.combat.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, length):
                if length < 0 or report['bytesRead'] + length > 256 * 1024:
                    raise ValueError('BloodStart read budget exceeded')
                if not any(row['start'] <= address and address + length <= row['end'] for row in maps):
                    raise PermissionError('BloodStart left bounded writable data')
                raw = os.pread(fd, length, address)
                if len(raw) != length:
                    raise OSError('Short BloodStart read')
                report['bytesRead'] += length
                return raw

            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            u32 = lambda address: struct.unpack('<I', read(address, 4))[0]
            game, system = u64(0x8ecd30), u64(0x8ed560)
            report['systemCounter'] = u32(system + 0x2efa0)
            report['gameUpdateCount'] = u32(game + 0xce9f4)
            room = u64(game + 0x8180)
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512:
                raise ValueError('Actual actor budget exceeded')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            for pointer in pointers:
                if not pointer:
                    continue
                identity = read(pointer + 0x10, 12)
                kind, variant, subtype = struct.unpack('<III', identity)
                if kind != 1000 or variant not in (17, 42):
                    continue
                raw = read(pointer, 0x7fc)
                pair = lambda at: list(struct.unpack_from('<ff', raw, at))
                definition = u64(pointer + 0x40)
                text = u64(definition)
                length = u64(text - 24)
                if not 1 <= length <= 128:
                    raise ValueError('Invalid actual effect clip')
                parent_pointer = struct.unpack_from('<Q', raw, 0x498)[0]
                parent = {'type': u32(parent_pointer + 0x10), 'variant': u32(parent_pointer + 0x14),
                          'spawnSeed': u32(parent_pointer + 0x7d0)} if parent_pointer else None
                # Exact410810: Sprite+268 is a112-byte layer-state array,
                # Sprite+270 count; Entity.Sprite starts+28. Controller460a36
                # writes returnedLayer+22/+23. Observe that actual owner only.
                layer_begin = struct.unpack_from('<Q', raw, 0x290)[0]
                layer_count = struct.unpack_from('<I', raw, 0x298)[0]
                if not 0 <= layer_count <= 32:
                    raise ValueError('Invalid actual Sprite layer count')
                layer_flips = [list(map(bool, read(layer_begin + index * 112 + 0x22, 2)))
                               for index in range(layer_count)]
                report['effects'].append({'variant': variant, 'subtype': subtype,
                    'spawnSeed': struct.unpack_from('<I', raw, 0x7d0)[0],
                    'birthGameUpdate': struct.unpack_from('<I', raw, 0x42c)[0],
                    'spawner': list(struct.unpack_from('<II', raw, 0x1c)), 'parent': parent,
                    'active': bool(raw[0x31e]), 'dead': bool(raw[0x31f]), 'visible': bool(raw[0x31d]),
                    'position': pair(0x448), 'velocity': pair(0x464),
                    'spriteOffset': pair(0x2bc), 'scale': pair(0x2c4),
                    'rotation': struct.unpack_from('<f', raw, 0x2cc)[0],
                    'flipX': bool(raw[0x2fc]), 'flipY': bool(raw[0x2fd]),
                    'layerFlips': layer_flips,
                    'tint': list(struct.unpack_from('<ffff', raw, 0x2d0)),
                    'colorOffsets': list(struct.unpack_from('<fff', raw, 0x2e0)),
                    'animation': read(text, length).decode('ascii'),
                    'frame': struct.unpack_from('<f', raw, 0x148)[0], 'playing': bool(raw[0x14c]),
                    'flags': struct.unpack_from('<I', raw, 0x318)[0],
                    'targetFrame': struct.unpack_from('<i', raw, 0x7d4)[0],
                    'ttl': struct.unpack_from('<i', raw, 0x7f0)[0],
                    'raw7f4': struct.unpack_from('<i', raw, 0x7f4)[0],
                    'raw7f8': struct.unpack_from('<f', raw, 0x7f8)[0]})
            raw = read(system + 0x12c20 + 72 * 0x158, 0x158)
            last_tick, next_allowed = struct.unpack_from('<ii', raw)
            volume = struct.unpack_from('<f', raw, 0x14c)[0]
            if not math.isfinite(volume) or raw[0x150] not in (0, 1):
                raise ValueError('Invalid source SFX72 admission')
            report['sound72'] = {'lastTick': last_tick, 'nextAllowed': next_allowed,
                                 'volume': volume, 'loaded': bool(raw[0x150]),
                                 'sampleCount': struct.unpack_from('<I', raw, 0x148)[0]}
            sample_count = report['sound72']['sampleCount']
            if not 0 <= sample_count <= 5:
                raise ValueError('Invalid bounded SFX72 sample count')
            report['sound72']['samples'] = [
                {'looping': bool(raw[0x10 + index * 64 + 0x14]),
                 'gain': struct.unpack_from('<f', raw, 0x10 + index * 64 + 0x18)[0],
                 'pitch': struct.unpack_from('<f', raw, 0x10 + index * 64 + 0x20)[0]}
                for index in range(sample_count)]
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError, struct.error) as error:
        report['status'] = 'fail'
        report['error'] = str(error)
        result['status'] = 'fail'
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    result['bloodStart'] = report
    return result
