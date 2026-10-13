"""Read-only native descriptor validation for recovered layout contracts.

Field offsets derive from the hashed local plaintext ELF, not browser input.
Only logical values are exported. Requires actual original log/parent ownership.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('readonly_maps', ROOT / 'tools/original/state-observer.py')
maps = importlib.util.module_from_spec(spec)
spec.loader.exec_module(maps)
spec = importlib.util.spec_from_file_location('descriptor_rng', ROOT / 'tools/original/foundation-descriptor-observer.py')
rng = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rng)
NATIVE_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
identity_spec = importlib.util.spec_from_file_location('native_identity', ROOT / 'tools/original/native-identity.py')
native_identity = importlib.util.module_from_spec(identity_spec)
identity_spec.loader.exec_module(native_identity)

def observe(pid, context):
    if context.get('label') not in ('gameplay-start', 'right-entry', 'left-entry', 'topology'):
        return {'status': 'skip', 'scope': 'layout gameplay-only observer'}
    stopped = False
    fd = None
    report = {'schema': 2, 'status': 'running', 'memoryWritten': False,
              'scope': 'source-derived descriptor values; no native addresses exported',
              'nativeBinarySha256': NATIVE_SHA, 'checkpoint': context.get('label')}
    try:
        status = Path(f'/proc/{pid}/status').read_text()
        parent = int(next(line.split(':', 1)[1] for line in status.splitlines() if line.startswith('PPid:')))
        if parent != os.getpid() or not context.get('gateBlocked'):
            raise ValueError('Requires blocked direct original child')
        if native_identity.native_binary_sha256(pid) != NATIVE_SHA:
            raise ValueError('Unexpected original executable')
        log = context.get('publicLogState', {})
        if not all(type(log.get(key)) is int for key in ('stageSeed', 'spawnSeed', 'seedValue')):
            raise ValueError('Requires actual original seed/stage/spawn log')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while True:
            state = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('State:'))
            if state[0] in ('T', 't'):
                break
            if time.monotonic() > deadline:
                raise TimeoutError('Original did not stop')
            time.sleep(.001)
        mappings = maps.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        def read(address, size):
            if size < 0 or size > 0x9000 or not any(m['start'] <= address and address + size <= m['end'] for m in mappings):
                raise ValueError('Read is outside bounded writable original data')
            result = os.pread(fd, size, address)
            if len(result) != size:
                raise ValueError('Short original data read')
            return result
        game, = struct.unpack('<Q', read(0x8ecd30, 8))
        level = read(game, 0x81c0)
        stage, = struct.unpack_from('<I', level, 0)
        count, starting = struct.unpack_from('<II', level, 0x8144)
        current, previous = struct.unpack_from('<ii', level, 0x8188)
        if stage != log.get('stage') or not 1 <= count <= 128 or not 0 <= current < 169:
            raise ValueError(f'Invalid or log-mismatching level header: {stage}/{count}/{current}; log={log}')
        grid = list(struct.unpack_from('<169i', level, 0x7ea0))
        if any(value < -1 or value >= count for value in grid):
            raise ValueError('Grid refers outside descriptor table')
        positions = rng.stage_positions(log['stageSeed'])
        descriptors = []
        for index in range(count):
            at = 0x10 + index * 0xf0
            grid_index, list_index, config = struct.unpack_from('<iiQ', level, at)
            if list_index != index:
                raise ValueError('Descriptor index field disagrees with table')
            template = read(config, 0x40)
            asset_stage, template_type, variant = struct.unpack_from('<III', template)
            name_address, = struct.unpack_from('<Q', template, 0x10)
            name = read(name_address, 256).split(b'\0', 1)[0].decode('ascii')
            decoration, spawn, following = struct.unpack_from('<III', level, at + 0x48)
            if rng.next_state(decoration, rng.PUBLIC_SHIFTS[12]) != spawn or rng.next_state(spawn, rng.PUBLIC_SHIFTS[12]) != following:
                raise ValueError('Descriptor seeds violate source RNG12 chain')
            initial = rng.inverse(decoration, rng.PUBLIC_SHIFTS[12])
            cells = [i for i, value in enumerate(grid) if value == index]
            descriptors.append({'index': index, 'gridIndex': grid_index, 'cells': cells,
                'doorGridIndices': list(struct.unpack_from('<8i', level, at + 0x18)),
                'doorMask': struct.unpack_from('<Q', level, at + 0x10)[0],
                'displayFlags': struct.unpack_from('<I', level, at + 0x38)[0],
                'visited': bool(level[at + 0x3c]),
                'combatClear': bool(level[at + 0x3d]),
                'doorPolicyFlag': bool(level[at + 0x40]),
                'template': {'assetStage': asset_stage, 'type': template_type, 'variant': variant, 'name': name,
                             'width': template[0x3a], 'height': template[0x3b],
                             'shapeIndex': struct.unpack_from('<I', template, 0x3c)[0]},
                'decorationSeed': decoration, 'spawnSeed': spawn, 'followingSeed': following,
                'roomInitSeed': initial, 'stage35DrawPositions': positions.get(initial, [])})
        actual = descriptors[grid[current]]
        if actual['spawnSeed'] != log['spawnSeed']:
            raise ValueError('Current descriptor spawn does not match actual original log')
        primary = struct.unpack_from('<4I', level, 0x814c)
        secondary = struct.unpack_from('<4I', level, 0x815c)
        if primary[1:] != rng.PUBLIC_SHIFTS[35] or secondary[1:] != rng.PUBLIC_SHIFTS[2]:
            raise ValueError('Stage RNG tuples disagree with source-derived initialization')
        report.update({'status': 'pass', 'stage': stage, 'altStage': level[4],
            'curseMask': struct.unpack_from('<I', level, 8)[0], 'descriptorCount': count,
            'startingGridIndex': starting, 'currentGridIndex': current,
            'previousGridIndex': previous, 'currentDescriptorIndex': grid[current],
            'grid': grid, 'descriptors': descriptors,
            'stage35CurrentState': primary[0], 'stage35DrawPositions': positions.get(primary[0], []),
            'stage2CurrentState': secondary[0], 'actualPublicLog': log})
    except (OSError, ValueError, TimeoutError) as error:
        report.update({'status': 'fail', 'error': str(error)})
    finally:
        if fd is not None:
            os.close(fd)
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report
