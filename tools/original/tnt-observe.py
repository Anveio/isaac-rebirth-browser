"""Actual bounded TNT fuse/Sprite, existing gameplay/effects and SFX182 owners.

Presentation-end RAM cannot witness transient crater18's same-Full callback.
That gap is explicit; no missing object implies a zero or successful callback.
"""
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


paired = load('tnt_paired', 'tools/original/foundation-paired-observer.py')
PIN = paired.combat.native_identity.NATIVE_SHA
effects = load('tnt_effects', 'tools/original/familiar-observer.py')
audio = load('tnt_audio', 'tools/original/audio-observer.py')


def tnt_owner(pid, context):
    result = {'schema': 1, 'status': 'running', 'nativeBinarySha256': PIN,
              'memoryWritten': False, 'executableMappingsRead': False,
              'bytesRead': 0, 'maxBytes': 16384, 'grids': []}
    stopped = False
    try:
        if context.get('gateBlocked') is not True or type(pid) is not int or pid <= 0:
            raise PermissionError('TNT requires a presentation-blocked owned child')
        parent = next(line.split(':', 1)[1] for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or paired.combat.native_identity.native_binary_sha256(pid) != PIN:
            raise PermissionError('TNT original parent/binary identity differs')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while not all(p.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt' for p in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic() > deadline:
                raise TimeoutError('Original threads did not stop')
            time.sleep(.001)
        maps = paired.combat.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, size):
                if size < 0 or result['bytesRead'] + size > result['maxBytes'] or not any(row['start'] <= address and address + size <= row['end'] for row in maps):
                    raise ValueError('TNT writable data/read budget exceeded')
                value = os.pread(fd, size, address)
                result['bytesRead'] += size
                if len(value) != size:
                    raise OSError('Short TNT read')
                return value
            def u64(address): return struct.unpack('<Q', read(address, 8))[0]
            def u32(address): return struct.unpack('<I', read(address, 4))[0]
            game = u64(0x8ecd30)
            room = u64(game + 0x8180)
            width, height = struct.unpack('<II', read(room + 0x14, 8))
            if not 3 <= width <= 28 or not 3 <= height <= 16 or width * height > 448:
                raise ValueError('Invalid actual TNT room grid')
            pointers = struct.unpack('<' + 'Q' * (width * height), read(room + 0x30, width * height * 8))
            for index, pointer in enumerate(pointers):
                if not pointer or u32(pointer + 8) != 12:
                    continue
                if u64(pointer) != 0x674530 or u32(pointer + 0x24) != index:
                    raise ValueError('Actual TNT class/room slot differs')
                raw = read(pointer + 0x40, 0x12d)
                definition = struct.unpack_from('<Q', raw, 0x20)[0]
                if not definition:
                    raise ValueError('Initialized TNT has no Sprite definition')
                text = u64(definition)
                length = u64(text - 24)
                if not 1 <= length <= 32:
                    raise ValueError('TNT animation name outside bound')
                name = read(text, length).decode('ascii')
                frame = struct.unpack_from('<f', raw, 0x128)[0]
                playing = raw[0x12c]
                if name not in ('Idle', 'IdleMedium', 'ReadyToExplode', 'Blown') or not math.isfinite(frame) or playing not in (0, 1):
                    raise ValueError('Invalid actual TNT animation owner')
                result['grids'].append({'index': index, 'countdown': struct.unpack_from('<i', raw)[0],
                    'animation': name, 'frame': frame, 'playing': bool(playing)})
            result['status'] = 'pass'
        finally:
            os.close(fd)
    except Exception as error:
        result['status'] = 'fail'
        result['error'] = repr(error)
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return result


def observe(pid, context):
    result = paired.observe(pid, context)
    if result['gameplay']['status'] == 'skip':
        result['tnt'] = {'schema': 1, 'status': 'skip', 'scope': 'explicit pre-run owner lifetime'}
        return result
    if result['gameplay']['status'] != 'pass':
        return result
    result['tnt'] = tnt_owner(pid, context)
    result['tntEffects'] = effects.observe(pid, context)
    sound = audio.observe(pid, context)
    result['tntAudio'] = {key: sound.get(key) for key in ('schema', 'status', 'error', 'nativeBinarySha256',
        'memoryWritten', 'executableMappingsRead', 'bytesRead', 'systemCounter', 'globalSfxVolume')}
    result['tntAudio']['slots'] = [slot for slot in sound.get('slots', []) if slot['id'] == 182]
    if any(result[key].get('status') != 'pass' for key in ('tnt', 'tntEffects', 'tntAudio')) or len(result['tntAudio']['slots']) != 1:
        result['status'] = 'fail'
    return result
