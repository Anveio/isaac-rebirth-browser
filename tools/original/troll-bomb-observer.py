"""Bounded read-only original bomb/pickup animation and admission metadata.

The public controller owns the stopped original child. All observations remain
reference evidence; none of these fields are provided to browser gameplay.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('troll_world', ROOT / 'tools/original/foundation-pickup-observe.py')
world = importlib.util.module_from_spec(spec)
spec.loader.exec_module(world)


def observe(pid, context):
    result = world.observe(pid, context)
    if result['status'] != 'pass':
        return result
    stopped = False
    try:
        parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or context.get('gateBlocked') is not True:
            raise PermissionError('Bomb metadata requires a blocked directly owned original')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while not all(path.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt'
                      for path in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic() >= deadline:
                raise TimeoutError('Original threads did not stop')
            time.sleep(.001)
        ranges = world.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        extra_bytes = 0
        try:
            def read(address, count):
                nonlocal extra_bytes
                if count < 0 or extra_bytes + count > 128 * 1024 or not any(
                        row['start'] <= address and address + count <= row['end'] for row in ranges):
                    raise ValueError('Bomb metadata outside bounded writable data')
                data = os.pread(fd, count, address)
                extra_bytes += count
                if len(data) != count:
                    raise OSError('Short original bomb metadata read')
                return data

            def u64(address):
                return struct.unpack('<Q', read(address, 8))[0]

            game = u64(0x8ecd30)
            room = u64(game + 0x8180)
            manager = room + 0x15d0
            count = struct.unpack('<I', read(manager + 0x74, 4))[0]
            if count > 512:
                raise ValueError('Bomb metadata actor budget exceeded')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            entities = {entity['sequenceId']: entity for entity in result['entities']}
            for pointer in pointers:
                if not pointer:
                    continue
                raw = read(pointer, 0x20)
                kind = struct.unpack_from('<I', raw, 0x10)[0]
                if kind not in (4, 5):
                    continue
                entity = entities[struct.unpack_from('<I', raw, 8)[0]]
                entity['birthGameUpdate'] = struct.unpack('<I', read(pointer + 0x42c, 4))[0]
                entity['spawnerType'] = struct.unpack_from('<I', raw, 0x1c)[0]
                entity['spawnerVariant'] = struct.unpack('<I', read(pointer + 0x20, 4))[0]
                if kind == 5:
                    entity['pickupRng'] = list(struct.unpack('<IIII', read(pointer + 0x7c0, 16)))
                    entity['pickupAnchor'] = list(struct.unpack('<ff', read(pointer + 0x440, 8)))
                    continue
                actor = pointer + 0x28
                clip = u64(actor + 0x18)
                name = u64(clip)
                length = u64(name - 24)
                if length > 128:
                    raise ValueError('Bomb animation name length exceeded')
                entity['animation'] = read(name, length).decode('ascii')
                entity['animationFrame'] = struct.unpack('<f', read(actor + 0x120, 4))[0]
                entity['animationPlaying'] = bool(read(actor + 0x124, 1)[0])
            result['bytesRead'] += extra_bytes
        finally:
            os.close(fd)
    except (OSError, ValueError, KeyError, TimeoutError, struct.error) as error:
        result['status'] = 'fail'
        result['error'] = str(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return result
