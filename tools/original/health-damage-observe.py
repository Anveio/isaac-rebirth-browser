"""Bounded original health, Info16 and SFX wrappers; never browser inputs."""
import hashlib
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('health_death_observer', ROOT / 'tools/original/player-death-observe.py')
death = importlib.util.module_from_spec(spec)
spec.loader.exec_module(death)


def health_fields(pid, context):
    if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
        raise PermissionError('Health fields require positive directly owned child at its actual presentation gate')
    parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
    if int(parent) != os.getpid():
        raise PermissionError('Health original is not directly owned')
    os.kill(pid, signal.SIGSTOP)
    try:
        deadline = time.monotonic() + 2
        while not all(task.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt'
                      for task in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic() > deadline:
                raise TimeoutError('Original threads did not stop for bounded health fields')
            time.sleep(.001)
        mappings = death.paired.combat.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            total = 0
            def allowed(address, count):
                return any(m['start'] <= address and address + count <= m['end'] for m in mappings)
            def read(address, count):
                nonlocal total
                if count < 0 or total + count > 16384:
                    raise ValueError('Health read budget exceeded')
                if not allowed(address, count):
                    raise PermissionError('Health read left writable source data')
                raw = os.pread(fd, count, address)
                if len(raw) != count:
                    raise OSError('Short bounded health read')
                total += count
                return raw
            def u64(address): return struct.unpack('<Q', read(address, 8))[0]
            def u32(address): return struct.unpack('<I', read(address, 4))[0]
            def i32(address): return struct.unpack('<i', read(address, 4))[0]
            def text(pointer):
                if not pointer: return None
                length = u64(pointer - 24)
                if not 0 <= length <= 512: raise ValueError('Invalid bounded GNU string')
                return read(pointer, length).decode('ascii') if length else None
            system = u64(0x8ed560)
            system_state = u32(system)
            result = {'status': 'pass', 'memoryWritten': False, 'systemState': system_state}
            if system_state != 2:
                result['bytesRead'] = total
                return result
            game = u64(0x8ecd30)
            begin, end = u64(game + 0xa148), u64(game + 0xa150)
            if end <= begin or (end - begin) % 8 or end - begin > 32:
                raise ValueError('Invalid actual Player list')
            player = u64(begin)
            if u64(player) != 0x670c90:
                raise ValueError('Unexpected actual Player class')
            result.update({'characterType': u32(player + 0xbfc),
                'containers': [i32(player + offset) for offset in (0xbb4, 0xbb8, 0xbc0)] + [u32(player + 0xbc4), i32(player + 0xbbc)],
                'damageFlags': u32(player + 0xe10), 'damageCountdown': i32(player + 0xbf8),
                'activeId': i32(player + 0xd74), 'activeStateD80': i32(player + 0xd80),
                'ownedCounts': list(struct.unpack('<347i', read(player + 0xe1c, 347 * 4)))})
            first, last = u64(player + 0xd00), u64(player + 0xd08)
            if last < first or (last - first) % 16 or last - first > 347 * 16:
                raise ValueError('Invalid temporary collectible vector')
            records = []
            for at in range(first, last, 16):
                config = u64(at)
                identifier = i32(config + 4)
                if not 0 <= identifier < 347:
                    raise ValueError('Temporary record has invalid ItemConfig identity')
                records.append({'itemId': identifier, 'count': i32(at + 8), 'cooldown': i32(at + 12)})
            result['temporaryCollectibles'] = records
            active_raw = read(player + 0x3076, 1)[0]
            active_known = result['activeId'] != 0 or any(record['itemId'] == 35 for record in records)
            if active_known and active_raw not in (0, 1):
                raise ValueError('Invalid initialized active3076 byte')
            result['activeFlag3076Raw'] = active_raw
            result['activeFlag3076'] = bool(active_raw) if active_known else None
            queued = u64(player + 0x3058)
            touched_raw = read(player + 0x3064, 1)[0]
            if queued and touched_raw not in (0, 1):
                raise ValueError('Invalid initialized queued touched byte')
            result['queuedId'] = i32(queued + 4) if queued else None
            result['queuedCharge'] = i32(player + 0x3060)
            result['queuedTouched'] = bool(touched_raw) if queued else None
            result['queuedTouchedRaw'] = touched_raw
            result['collectionDelay'] = i32(player + 0x30d4)
            copy = player + 0x7e0
            selected_copy = u64(copy + 0x18)
            copy_frame, copy_playing = struct.unpack('<fB', read(copy + 0x120, 5))
            result['copiedItemSprite'] = {'actor': text(u64(copy)),
                'clip': text(u64(selected_copy)) if selected_copy else None,
                'frame': copy_frame, 'playing': bool(copy_playing)}
            info = game + 0xa1e8
            sprite = info + 8
            selected = u64(sprite + 0x18)
            frame, playing = struct.unpack('<fB', read(sprite + 0x120, 5))
            loaded = read(sprite + 0x2dd, 1)[0]
            state = u32(info)
            if state not in (0, 1, 2) or playing not in (0, 1) or loaded not in (0, 1):
                raise ValueError('Unknown actual information owner state')
            result['informationAnimation'] = {'state': state, 'delay': i32(info + 0x748),
                'actor': text(u64(sprite)), 'clip': text(u64(selected)) if selected else None,
                'frame': frame, 'playing': bool(playing), 'definitionLoaded': bool(loaded)}
            result['bytesRead'] = total
            return result
        finally:
            os.close(fd)
    finally:
        os.kill(pid, signal.SIGCONT)


def observe(pid, context):
    result = {'schema': 1, 'status': 'fail', 'memoryWritten': False, 'executableMappingsRead': False,
              'scope': 'Independent bounded health/Hit/Info16/temporary35 and original SFX wrappers; observations never supply browser state'}
    try:
        fields = health_fields(pid, context)
        if pid not in death.EXECUTABLES:
            executable = Path(f'/proc/{pid}/exe')
            death.EXECUTABLES[pid] = {'path': str(executable.resolve()), 'sha256': hashlib.sha256(executable.read_bytes()).hexdigest()}
        if death.EXECUTABLES[pid]['sha256'] != death.audio.NATIVE_SHA:
            raise ValueError('Unexpected original executable hash')
        owner = death.death_fields(pid)
        if fields['systemState'] == 2:
            result.update(death.paired.observe(pid, context))
        else:
            menu = death.paired.menu.observe(pid, context)
            result.update({'status': menu['status'], 'menu': menu,
                           'gameplay': {'status': 'skip', 'systemState': fields['systemState']}})
        result['health'] = fields
        result['death'] = owner
        sounds = death.audio.observe(pid, context)
        result['healthSounds'] = {key: sounds.get(key) for key in
            ('status', 'memoryWritten', 'systemCounter', 'globalSfxVolume', 'nativeBinarySha256',
             'soundCatalogSha256', 'catalogCountsMatch', 'pathMismatches', 'error')}
        result['healthSounds']['slots'] = [slot for slot in sounds.get('slots', []) if slot['id'] in (30, 33, 55)]
        result['processExecutable'] = death.EXECUTABLES[pid]
        if sounds['status'] != 'pass' or owner['status'] != 'pass': result['status'] = 'fail'
    except (OSError, ValueError, KeyError, TimeoutError, struct.error) as error:
        result['health'] = {'status': 'fail', 'memoryWritten': False, 'error': str(error)}
        result['error'] = str(error)
    return result
