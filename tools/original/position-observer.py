#!/usr/bin/env python3
"""Track candidate f32 position pairs in bounded original writable data.

Starting vectors are search hypotheses, not a native coordinate/state contract.
Only temporal deltas can validate a candidate; no addresses enter the port.
"""
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

spec = importlib.util.spec_from_file_location('reference_data_reader', Path(__file__).with_name('state-observer.py'))
reader = importlib.util.module_from_spec(spec); spec.loader.exec_module(reader)
INITIAL_X = (320., 240.)
PRIMARY_PAIRS = ((320., 400.), (240., 213.))
MAX_BYTES = 128 * 1024 * 1024
MAX_CANDIDATES = 512
tracked = {}
owner = None


def observe(pid, context):
    global owner
    parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
    if int(parent) != os.getpid() or not context.get('gateBlocked'):
        raise PermissionError('Original must be a directly owned child blocked at a presentation boundary')
    if owner != pid:
        tracked.clear(); owner = pid
    first = not tracked
    started = time.monotonic(); stopped = False
    report = {'schema': 1, 'status': 'pass', 'scope': 'reference-only f32 pair hypotheses; no typed player state or full state parity claim',
              'checkpoint': context['label'], 'executableMappingsRead': False, 'memoryWritten': False,
              'maxBytes': MAX_BYTES, 'bytesRead': 0, 'initialScan': first, 'matches': [],
              'consistency': 'all original threads SIGSTOP during bounded reads; resume preserves presentation gate'}
    try:
        os.kill(pid, signal.SIGSTOP); stopped = True
        deadline = time.monotonic() + 2
        while True:
            state = next(line for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('State:'))
            if state.split(':', 1)[1].strip()[0] in ('T', 't'):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('Original did not stop for position observation')
            time.sleep(.001)
        maps = reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            if first:
                for mapping in maps:
                    size = min(mapping['end'] - mapping['start'], MAX_BYTES - MAX_CANDIDATES * 144 - report['bytesRead'])
                    if size <= 0:
                        break
                    tail = b''
                    for offset in range(0, size, reader.CHUNK):
                        block = os.pread(fd, min(reader.CHUNK, size - offset), mapping['start'] + offset)
                        payload = tail + block; base = mapping['start'] + offset - len(tail)
                        for x in INITIAL_X:
                            pattern = struct.pack('<f', x); cursor = payload.find(pattern)
                            while cursor >= 0:
                                address = base + cursor
                                if address % 4 == 0 and cursor + 8 <= len(payload):
                                    pair = struct.unpack_from('<ff', payload, cursor)
                                    if math.isfinite(pair[1]) and 100 <= pair[1] <= 500:
                                        primary = pair in PRIMARY_PAIRS
                                        if primary and len(tracked) >= MAX_CANDIDATES:
                                            secondary = next((key for key, value in tracked.items() if not value['primary']), None)
                                            if secondary is not None:
                                                del tracked[secondary]
                                        if len(tracked) < MAX_CANDIDATES:
                                            tracked[address] = {'address': hex(address), 'initial': pair, 'primary': primary,
                                                                'mapping': mapping['path'] or '[anonymous]'}
                                cursor = payload.find(pattern, cursor + 1)
                        report['bytesRead'] += len(block); tail = payload[-8:]
            for address, candidate in tracked.items():
                if not any(mapping['start'] <= address and address + 8 <= mapping['end'] for mapping in maps):
                    report['matches'].append({**candidate, 'status': 'unmapped'}); continue
                block = os.pread(fd, 8, address); report['bytesRead'] += len(block)
                if len(block) != 8:
                    raise OSError('Short tracked position read')
                values = struct.unpack('<ff', block)
                if not all(math.isfinite(value) for value in values):
                    report['matches'].append({**candidate, 'status': 'nonfinite'}); continue
                result = {**candidate, 'status': 'candidate', 'f32': values,
                          'delta': [value - initial for value, initial in zip(values, candidate['initial'])]}
                if candidate['initial'] == (320., 400.):
                    mapping = next(mapping for mapping in maps if mapping['start'] <= address and address + 8 <= mapping['end'])
                    low, high = max(mapping['start'], address - 64), min(mapping['end'], address + 72)
                    nearby = os.pread(fd, high - low, low); report['bytesRead'] += len(nearby)
                    result['nearbyF32Candidates'] = [{'relativeBytes': offset + low - address, 'f32': value}
                        for offset in range(0, len(nearby) - 3, 4)
                        for value in [struct.unpack_from('<f', nearby, offset)[0]]
                        if math.isfinite(value) and abs(value) <= 10 and (value == 0 or abs(value) >= 1e-7)]
                report['matches'].append(result)
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError) as error:
        report['status'] = 'fail'; report['error'] = str(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    report['durationMs'] = (time.monotonic() - started) * 1000
    return report
