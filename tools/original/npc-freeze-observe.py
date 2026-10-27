"""Add bounded read-only NPC status owners to the canonical active-item reader.

Only target-bearing activeItems.npcFreeze declarations select this wrapper.
The existing readers retain their independent observations and admission rules.
Extra reads join actual room-list actors by sequenceId/spawnSeed; no Native
value supplies Browser input or state. Missing owners fail the observation.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


canonical = load('freeze_canonical_active_owners', 'pocket-collection-observe.py')
combat = load('freeze_actual_combat_owners', 'foundation-combat-observe.py')


def status_owners(pid, context):
    report = dict(schema='source-public-npc-freeze-observation-v1', status='pass',
                  memoryWritten=False, executableMappingsRead=False, bytesRead=0, actors=[])
    stopped = False
    try:
        parent = next(line.split(':', 1)[1].strip()
                      for line in Path(f'/proc/{pid}/status').read_text().splitlines()
                      if line.startswith('PPid:'))
        if int(parent) != os.getpid() or not context.get('gateBlocked'):
            raise PermissionError('Freeze observation needs a blocked directly owned Original')
        if combat.native_identity.native_binary_sha256(pid) != combat.NATIVE_SHA:
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
        maps = combat.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, count):
                if count < 0 or report['bytesRead'] + count > 128 * 1024:
                    raise ValueError('Freeze owner read budget exceeded')
                if not any(row['start'] <= address and address + count <= row['end'] for row in maps):
                    raise ValueError(f'Actual writable freeze owner missing at {address:#x}')
                value = os.pread(fd, count, address)
                if len(value) != count:
                    raise OSError('Short Original freeze owner read')
                report['bytesRead'] += count
                return value

            def u64(address):
                return struct.unpack('<Q', read(address, 8))[0]

            def u32(address):
                return struct.unpack('<I', read(address, 4))[0]

            game = u64(0x8ecd30)
            room = u64(game + 0x8180)
            if not game or not room:
                raise ValueError('Actual gameplay room required')
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512:
                raise ValueError('Actual room-list bound exceeded')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            for pointer in pointers:
                if not pointer:
                    continue
                raw = read(pointer, 0x4a8)
                kind = struct.unpack_from('<I', raw, 0x10)[0]
                if not 10 <= kind <= 999:
                    continue
                if struct.unpack_from('<Q', raw)[0] != 0x66ef30:
                    raise ValueError('Actual ordinary NPC vtable required for bounded freeze fixture')
                pair = lambda offset: list(struct.unpack_from('<ff', raw, offset))
                if raw[0x322] not in (0, 1) or raw[0x494] not in (0, 1):
                    raise ValueError('Actual preview/eligibility byte outside boolean range')
                begin, end = struct.unpack_from('<QQ', raw, 0x388)
                if end < begin or (end - begin) % 0x3c or end - begin > 0x3c * 128:
                    raise ValueError('Actual timed-colour vector bound invalid')
                entries = read(begin, end - begin) if end != begin else b''
                if any(entries[at + offset] not in (0, 1)
                       for at in range(0, len(entries), 0x3c) for offset in (0x38, 0x39)):
                    raise ValueError('Actual timed-colour fade/propagate byte outside boolean range')
                report['actors'].append(dict(
                    sequenceId=struct.unpack_from('<I', raw, 8)[0],
                    spawnSeed=u32(pointer + 0x7d0), type=kind,
                    variant=struct.unpack_from('<I', raw, 0x14)[0],
                    subtype=struct.unpack_from('<I', raw, 0x18)[0],
                    preview=bool(raw[0x322]), backupPosition=pair(0x450),
                    physicalPosition=pair(0x450 if raw[0x322] else 0x448),
                    baseFriction=struct.unpack_from('<f', raw, 0x46c)[0],
                    applicationLockout=struct.unpack_from('<i', raw, 0x420)[0],
                    raw494=bool(raw[0x494]),
                    parentNull=struct.unpack_from('<Q', raw, 0x498)[0] == 0,
                    childNull=struct.unpack_from('<Q', raw, 0x4a0)[0] == 0,
                    # SetColor446f00: 3c stride, priority0, KColor4,
                    # duration30, remaining34, fade38, propagate39.
                    timedColors=[dict(priority=struct.unpack_from('<i', entries, at)[0],
                        color=list(struct.unpack_from('<11f', entries, at + 4)),
                        duration=struct.unpack_from('<i', entries, at + 0x30)[0],
                        remaining=struct.unpack_from('<i', entries, at + 0x34)[0],
                        fade=bool(entries[at + 0x38]), propagate=bool(entries[at + 0x39]))
                        for at in range(0, len(entries), 0x3c)]))
        finally:
            os.close(fd)
    except (OSError, ValueError, TimeoutError, struct.error) as error:
        report.update(status='fail', error=repr(error))
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report


def observe(pid, context):
    result = canonical.observe(pid, context)
    actual = combat.observe(pid, context)
    result['combat'] = actual
    if actual.get('status') == 'pass':
        result['npcFreeze'] = status_owners(pid, context)
        # Status readback must describe exactly the same room-list identities.
        expected = sorted((row['sequenceId'], row['spawnSeed']) for row in actual['entities']
                          if 10 <= row['type'] <= 999)
        measured = sorted((row['sequenceId'], row['spawnSeed']) for row in result['npcFreeze']['actors'])
        if expected != measured:
            result['npcFreeze'].update(status='fail', error='Independent freeze/room-list identity join failed')
    else:
        result['npcFreeze'] = dict(status=actual['status'], reason='No complete actual combat owners',
                                  memoryWritten=False, executableMappingsRead=False)
    if any(result[key].get('status') not in ('pass', 'skip') for key in ('combat', 'npcFreeze')):
        result['status'] = 'fail'
    result['scope'] += ' Additive actual NPC freeze, full/Fast phase, sprite and motion owners.'
    return result
