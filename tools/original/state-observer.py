#!/usr/bin/env python3
"""Observe bounded writable data in the controller's original-game child.

This module is reference tooling only. It never reads executable mappings,
disassembles instructions, writes original state, or supplies browser behavior.
Numeric matches are candidates until independent seed and temporal validation.
"""
import json
import os
from pathlib import Path
import signal
import struct
import time
import numpy as np

ROOT = Path(__file__).resolve().parents[2]
CHUNK = 1024 * 1024
MAX_READ = 64 * 1024 * 1024
MAX_MATCHES = 256
PUBLIC_SHIFTS = ((3, 23, 25), (5, 9, 7))


def writable_maps(text):
    """Only readable, writable, nonexecutable original data/anonymous maps."""
    result = []
    for line in text.splitlines():
        fields = line.split(None, 5)
        if len(fields) < 5:
            raise ValueError('Malformed process mapping')
        permissions = fields[1]
        if 'r' not in permissions or 'w' not in permissions or 'x' in permissions:
            continue
        path = fields[5] if len(fields) == 6 else ''
        if path and path not in ('[heap]', '[stack]') and not path.endswith('/isaac.x64'):
            continue
        start, end = (int(part, 16) for part in fields[0].split('-'))
        result.append({'start': start, 'end': end, 'permissions': permissions,
                       'path': path, 'fileOffset': int(fields[2], 16)})
    data_ends = {entry['end'] for entry in result if entry['path'].endswith('/isaac.x64')}
    priority = lambda entry: (0 if entry['path'].endswith('/isaac.x64') else
                             1 if not entry['path'] and entry['start'] in data_ends else
                             2 if entry['path'] == '[stack]' else
                             3 if entry['path'] == '[heap]' else 4, entry['start'])
    return sorted(result, key=priority)


def numeric_watch(context):
    values = {}
    def add(value, label):
        if type(value) is int and 0 < value <= 0xffffffff:
            values.setdefault(value, []).append(label)
    add(context.get('seedValue'), 'requested-start-seed')
    for key in ('seedValue', 'stageSeed', 'spawnSeed'):
        add(context.get('publicLogState', {}).get(key), 'actual-public-log:' + key)
    candidate_path = Path(os.environ.get('ISAAC_STATE_OBSERVER_CANDIDATES',
                          str(ROOT / '.cache/foundation/fx-float-inversion.json')))
    candidate_source = None
    if candidate_path.is_file():
        models = json.loads(candidate_path.read_text())
        for model in models:
            if model.get('model') != 'worldSprite':
                continue
            for row in model.get('rows', []):
                observed = row['observation']['seed']
                if observed.get('seedValue') != context.get('seedValue'):
                    continue
                candidate_source = str(candidate_path)
                for key in ('seedValue', 'stageSeed', 'spawnSeed'):
                    add(observed.get(key), 'paired-graphics-log:' + key)
                for candidate in row['candidates']:
                    if candidate.get('shiftIndex') != 35:
                        continue
                    for key in ('previousState', 'xState', 'yState'):
                        add(candidate[key], 'public-FX35-float-inversion:' + key)
    return values, candidate_source


def scan_block(block, base, watches, shifts=PUBLIC_SHIFTS):
    """Return matching numbers and public shift tuples; not native field types."""
    observations = []
    def append(index, labels, value):
        low = max(0, index - 4); high = min(len(block), index + 20)
        low += (index - low) % 4
        observations.append({'address': base + index, 'u32': value, 'labels': labels,
                             'wordAlignment': (base + index) % 4,
                             'neighborStart': base + low,
                             'neighborU32': list(struct.unpack('<' + 'I' * ((high - low) // 4),
                                                     block[low:low + (high - low) // 4 * 4]))})
    for value, labels in watches.items():
        pattern = struct.pack('<I', value)
        offset = block.find(pattern)
        while offset >= 0:
            append(offset, labels, value)
            if len(observations) >= MAX_MATCHES:
                return observations, True
            offset = block.find(pattern, offset + 1)
    for shift in shifts:
        pattern = struct.pack('<III', *shift)
        offset = block.find(pattern)
        while offset >= 0:
            if offset >= 4:
                append(offset - 4, ['public-shift-tuple:' + ','.join(map(str, shift))],
                       struct.unpack_from('<I', block, offset - 4)[0])
            if len(observations) >= MAX_MATCHES:
                return observations, True
            offset = block.find(pattern, offset + 1)
    # Discover data relationships rather than importing native descriptor layouts.
    words = np.frombuffer(block[:len(block) // 4 * 4], dtype='<u4')
    def public12(values):
        values = values ^ (values >> np.uint32(2))
        values = values ^ (values << np.uint32(7))
        return values ^ (values >> np.uint32(9))
    if len(words) >= 3:
        valid = (words[:-2] != 0) & (public12(words[:-2]) == words[1:-1]) & (public12(words[1:-1]) == words[2:])
        for index in np.flatnonzero(valid):
            append(int(index) * 4, ['public-u32-seed-chain:12,12'], int(words[index]))
            if len(observations) >= MAX_MATCHES:
                return observations, True
    return observations, False


def observe(pid, context):
    started = time.monotonic()
    status = Path(f'/proc/{pid}/status').read_text()
    parent = next(line.split(':', 1)[1].strip() for line in status.splitlines() if line.startswith('PPid:'))
    if int(parent) != os.getpid():
        raise PermissionError('State observer requires the original to be its direct child')
    if not context.get('gateBlocked'):
        raise ValueError('State observation requires a blocked presentation boundary')
    watches, candidate_source = numeric_watch(context)
    maximum_mib = int(os.environ.get('ISAAC_STATE_OBSERVER_MAX_MIB', '64'))
    if not 1 <= maximum_mib <= 256:
        raise ValueError('Observer read cap must be 1..256 MiB')
    maximum = maximum_mib * 1024 * 1024
    report = {'schema': 1, 'status': 'pass', 'scope': 'reference-only writable data observations; numeric matches are not typed game state',
              'processId': pid, 'parentId': os.getpid(), 'checkpoint': context['label'],
              'consistency': 'all original threads SIGSTOP during bounded reads; resume preserves blocked main-thread gate',
              'executableMappingsRead': False, 'memoryWritten': False, 'maxBytes': maximum,
              'candidateSource': candidate_source, 'watches': [{'u32': value, 'labels': labels} for value, labels in watches.items()],
              'maps': [], 'matches': [], 'bytesRead': 0, 'coverageComplete': True, 'matchesTruncated': False}
    stopped = False
    try:
        os.kill(pid, signal.SIGSTOP); stopped = True
        deadline = time.monotonic() + 2
        while True:
            state = next(line for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('State:'))
            if state.split(':', 1)[1].strip()[0] in ('T', 't'):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('Original did not stop for consistent state observation')
            time.sleep(.001)
        maps = writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            for mapping in maps:
                remaining = maximum - report['bytesRead']
                if remaining <= 0:
                    report['coverageComplete'] = False
                    break
                size = min(mapping['end'] - mapping['start'], remaining)
                entry = {**mapping, 'start': hex(mapping['start']), 'end': hex(mapping['end']), 'bytesRead': 0}
                report['maps'].append(entry)
                tail = b''
                for offset in range(0, size, CHUNK):
                    block = os.pread(fd, min(CHUNK, size - offset), mapping['start'] + offset)
                    if not block:
                        raise OSError('Short writable-memory read')
                    # Overlap retains uint32/tuple observations crossing chunk boundaries.
                    payload = tail + block; base = mapping['start'] + offset - len(tail)
                    hits, truncated = scan_block(payload, base, watches)
                    for hit in hits:
                        hit['mapping'] = mapping['path'] or '[anonymous]'
                        hit['mappingOffset'] = hit['address'] - mapping['start']
                        hit['address'] = hex(hit['address']); hit['neighborStart'] = hex(hit['neighborStart'])
                        if hit not in report['matches']:
                            report['matches'].append(hit)
                    report['matchesTruncated'] |= truncated
                    report['bytesRead'] += len(block); entry['bytesRead'] += len(block)
                    tail = payload[-48:]
                if size < mapping['end'] - mapping['start']:
                    report['coverageComplete'] = False
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError) as error:
        report['status'] = 'fail'; report['error'] = str(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    report['durationMs'] = (time.monotonic() - started) * 1000
    return report
