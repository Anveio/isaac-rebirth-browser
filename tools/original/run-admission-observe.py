"""Bounded original run-clock/fade fields, read only from an owned blocked child.

Historical research called Player+7d4 visibility. It is the controls-enabled
byte; actual entity visibility+31d is retained separately and never inferred.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result
identity = module('admission_identity', ROOT / 'tools/original/native-identity.py')
reader = module('admission_reader', ROOT / 'tools/original/state-observer.py')

def observe(pid, context):
    if not context.get('gateBlocked'):
        raise PermissionError('Run admission observation requires the blocked public frame gate')
    digest = identity.native_binary_sha256(pid)
    stopped = False
    try:
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline:
                raise TimeoutError('Owned original did not stop')
            time.sleep(.001)
        maps = reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            total = 0
            def read(address, size):
                nonlocal total
                total += size
                if total > 16 * 1024:
                    raise ValueError('Admission observer read budget exceeded')
                if not any(m['start'] <= address and address + size <= m['end'] for m in maps):
                    raise PermissionError('Admission read left writable data')
                value = os.pread(fd, size, address)
                if len(value) != size:
                    raise OSError('Short admission read')
                return value
            def u32(address): return struct.unpack('<I', read(address, 4))[0]
            def u64(address): return struct.unpack('<Q', read(address, 8))[0]
            def f32(address): return struct.unpack('<f', read(address, 4))[0]
            game, system = u64(0x8ecd30), u64(0x8ed560)
            report = {'schema': 1, 'status': 'pass', 'nativeBinarySha256': digest,
                'scope': 'Actual bounded initial admission clocks/full guards; no source value enters production',
                'memoryWritten': False, 'executableMappingsRead': False,
                'systemState': u32(system), 'systemIterations': u32(system + 0x2efa0),
                'requestedTimeAnchorMs': u64(system + 0x2f2b8),
                'gameAge': u32(game + 0xce9f4), 'explicitFreeze': u32(game + 0xce9f0),
                'popupState': u32(game + 0xa938), 'bossSceneState': u32(game + 0xa0e0),
                'roomTransitionState': u32(game + 0x95c0), 'pauseState': u32(game + 0x8ce18),
                'initialFade': f32(game + 0x10eb14), 'exitFade': f32(game + 0x10eb18),
                'fadeRate': f32(game + 0x10eb1c)}
            if report['systemState'] == 2:
                begin, end = u64(game + 0xa148), u64(game + 0xa150)
                if end < begin or (end - begin) % 8 or not 1 <= (end - begin) // 8 <= 4:
                    raise ValueError('Unexpected primary player list')
                player = u64(begin)
                report['primaryControlsEnabled'] = bool(read(player + 0x7d4, 1)[0])
                report['primaryEntityVisible'] = bool(read(player + 0x31d, 1)[0])
                report['primaryPosition'] = list(struct.unpack('<ff', read(player + 0x448, 8)))
                report['primaryVelocity'] = list(struct.unpack('<ff', read(player + 0x464, 8)))
                report['primaryBirthGameAge'] = u32(player + 0x42c)
            report['bytesRead'] = total
            return report
        finally:
            os.close(fd)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
