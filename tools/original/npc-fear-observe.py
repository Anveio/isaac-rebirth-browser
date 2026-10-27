"""Read-only Fear owners, composed with the unchanged Freeze/active reader.

Only declared activeItems.npcFear cases select this wrapper. The actual room
list is joined by sequenceId/spawnSeed; no observation supplies game inputs.
"""
import os
from pathlib import Path
import signal
import struct
import time
import importlib.util


spec = importlib.util.spec_from_file_location('fear_existing_npc_reader',
    Path(__file__).with_name('npc-freeze-observe.py'))
freeze = importlib.util.module_from_spec(spec)
spec.loader.exec_module(freeze)


def fear_owners(pid, context):
    report = dict(schema='source-public-npc-fear-observation-v1', status='pass',
                  memoryWritten=False, executableMappingsRead=False, bytesRead=0, actors=[])
    stopped = False
    try:
        parent = next(line.split(':', 1)[1].strip()
                      for line in Path(f'/proc/{pid}/status').read_text().splitlines()
                      if line.startswith('PPid:'))
        if int(parent) != os.getpid() or not context.get('gateBlocked'):
            raise PermissionError('Fear observation needs a blocked directly owned Original')
        if freeze.combat.native_identity.native_binary_sha256(pid) != freeze.combat.NATIVE_SHA:
            raise ValueError('Unexpected Original executable')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while True:
            state = next(line for line in Path(f'/proc/{pid}/status').read_text().splitlines()
                         if line.startswith('State:'))
            if state.split(':', 1)[1].strip()[0] in ('T', 't'):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('Original did not stop')
            time.sleep(.001)
        maps = freeze.combat.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, count):
                if count < 0 or report['bytesRead'] + count > 128 * 1024:
                    raise ValueError('Fear owner read budget exceeded')
                if not any(row['start'] <= address and address + count <= row['end'] for row in maps):
                    raise ValueError(f'Actual writable Fear owner missing at {address:#x}')
                value = os.pread(fd, count, address)
                if len(value) != count:
                    raise OSError('Short Original Fear owner read')
                report['bytesRead'] += count
                return value

            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            u32 = lambda address: struct.unpack('<I', read(address, 4))[0]
            i32 = lambda address: struct.unpack('<i', read(address, 4))[0]
            game = u64(0x8ecd30)
            if not game:
                raise ValueError('Actual gameplay Game owner required')
            # Seeds+48 is independent of ChallengeID and is not inferred from it.
            report['sourceEffectMode'] = i32(game + 0xa1d8)
            # Existing boot-MT observer owners. This is shared-stream evidence,
            # not a per-Horf call trace or complete MT qualification.
            report['sharedRandom'] = dict(index=u32(0x8e6b64),
                words=list(struct.unpack('<624Q', read(0x8ed680, 624 * 8))))
            if not 0 <= report['sharedRandom']['index'] <= 624 or any(
                    word > 0xffffffff for word in report['sharedRandom']['words']):
                raise ValueError('Actual shared MT owner outside source bounds')
            room = u64(game + 0x8180)
            if not room:
                raise ValueError('Actual room owner required')
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512:
                raise ValueError('Actual room-list bound exceeded')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            for pointer in pointers:
                if not pointer:
                    continue
                raw = read(pointer, 0x20)
                kind = struct.unpack_from('<I', raw, 0x10)[0]
                if not 10 <= kind <= 999:
                    continue
                if struct.unpack_from('<Q', raw)[0] != 0x66ef30:
                    raise ValueError('Actual ordinary NPC vtable required')
                path = read(pointer + 0x7d8, 0x30)
                vector = lambda offset: list(struct.unpack_from('<ff', path, offset))
                tail = read(pointer + 0x7d8 + 0x730, 0x18)
                if any(tail[offset] not in (0, 1) for offset in (0, 1)):
                    raise ValueError('Actual path direct/found outside boolean range')
                report['actors'].append(dict(
                    sequenceId=struct.unpack_from('<I', raw, 8)[0],
                    spawnSeed=u32(pointer + 0x7d0),
                    entityFlags=u32(pointer + 0x318), fearTimer=i32(pointer + 0x418),
                    applicationLockout=i32(pointer + 0x420),
                    vectors=dict(target=vector(8), wanderDirection10=vector(0x10)),
                    # Independent Native-only preservation witness. Horf's Port
                    # owns just the two vectors, never a fabricated full path.
                    preservedPath=dict(direction=struct.unpack_from('<i', path, 0x18)[0],
                        stuck=struct.unpack_from('<i', path, 0x1c)[0],
                        lastUpdate=struct.unpack_from('<I', path, 0x20)[0],
                        cell=struct.unpack_from('<i', path, 0x24)[0],
                        cellDelay=struct.unpack_from('<i', path, 0x28)[0],
                        replan=struct.unpack_from('<i', path, 0x2c)[0],
                        direct=bool(tail[0]), found=bool(tail[1]),
                        lastTarget=list(struct.unpack_from('<ff', tail, 4)),
                        panic=struct.unpack_from('<i', tail, 0xc)[0],
                        slack=struct.unpack_from('<f', tail, 0x10)[0],
                        pressure=struct.unpack_from('<f', tail, 0x14)[0])))
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError, struct.error) as error:
        report.update(status='fail', error=repr(error))
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report


def observe(pid, context):
    result = freeze.observe(pid, context)
    actual = result.get('combat', {})
    if actual.get('status') == 'pass':
        result['npcFear'] = fear_owners(pid, context)
        expected = sorted((row['sequenceId'], row['spawnSeed']) for row in actual['entities']
                          if 10 <= row['type'] <= 999)
        measured = sorted((row['sequenceId'], row['spawnSeed']) for row in result['npcFear']['actors'])
        if expected != measured:
            result['npcFear'].update(status='fail', error='Independent Fear/room-list identity join failed')
    else:
        # The composed combat reader skips before actual Level::Init. Preserve
        # that menu boundary; an unavailable gameplay owner still fails there.
        result['npcFear'] = dict(status=actual['status'], reason='No complete actual combat owners',
                                memoryWritten=False, executableMappingsRead=False)
    if result['npcFear'].get('status') not in ('pass', 'skip'):
        result['status'] = 'fail'
    result['scope'] += ' Additive actual Fear418/800, shared420, Seeds+48, two path vectors and shared MT.'
    return result
