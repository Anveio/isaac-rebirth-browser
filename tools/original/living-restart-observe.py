"""Read-only, bounded Restart owners at the canonical paired public gate."""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


controls = load('restart_existing_controls', ROOT / 'tools/original/controls-paired-observer.py')
identity = load('restart_native_identity', ROOT / 'tools/original/native-identity.py')
maps_reader = load('restart_writable_maps', ROOT / 'tools/original/state-observer.py')


def owner(pid, context):
    result = dict(schema=1, status='running', memoryWritten=False,
                  functionsInvoked=False, executableMappingsRead=False, bytesRead=0)
    stopped = False
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Restart observation requires the directly owned blocked child')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = next(row.split(':', 1)[1].strip() for row in status.splitlines() if row.startswith('PPid:'))
        if int(parent) != os.getpid() or identity.native_binary_sha256(pid) != identity.NATIVE_SHA:
            raise PermissionError('Restart child/executable differs from canonical Native identity')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline:
                raise TimeoutError('Native did not stop for Restart observation')
            time.sleep(.001)
        mappings = maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, size):
                if size < 0 or result['bytesRead'] + size > 4096 or not any(
                        mapping['start'] <= address and address + size <= mapping['end'] for mapping in mappings):
                    raise PermissionError('Restart read leaves bounded writable owners')
                raw = os.pread(fd, size, address)
                result['bytesRead'] += len(raw)
                if len(raw) != size:
                    raise OSError('Short Restart owner read')
                return raw

            def u32(address):
                return struct.unpack('<I', read(address, 4))[0]

            def u64(address):
                return struct.unpack('<Q', read(address, 8))[0]

            def boolean(address):
                value = read(address, 1)[0]
                if value not in (0, 1):
                    raise ValueError('Restart owner has a nonboolean flag')
                return bool(value)

            system = u64(0x8ed560)
            game = u64(0x8ecd30)
            text = u64(system + 0x2f2d8)
            length = u64(text - 0x18)
            if length > 16:
                raise ValueError('Queued Restart seed exceeds bounded canonical string')
            seed_code = read(text, length).decode('ascii')
            result.update(systemState=u32(system), systemCounter=u32(system + 0x2efa0),
                          heldPolls=u32(game + 0x10eb24), fullGameUpdates=u32(game + 0xce9f4),
                          persistent=dict(dirty=boolean(system + 0x10), inhibit=boolean(system + 0x11),
                                          counter21=struct.unpack('<i', read(system + 0x138, 4))[0]),
                          pending=boolean(system + 0x2f2c8), saveEnabled=boolean(system + 0x2f2f8),
                          context=dict(characterSelector=u32(system + 0x2f2cc), challengeId=u32(system + 0x2f2d0),
                                       sourceR8=u32(system + 0x2f2e0), sourceR9=boolean(system + 0x2f2e4),
                                       seedCode=seed_code, savedLoader=boolean(system + 0x2f2c9),
                                       specialStart=boolean(system + 0x2f2e5)))
            if result['systemState'] == 2:
                begin, end = u64(game + 0xa148), u64(game + 0xa150)
                if end <= begin or (end - begin) % 8 or end - begin > 32 * 8:
                    raise ValueError('Restart has no bounded actual primary Player vector')
                player = u64(begin)
                result['primary'] = dict(characterType=u32(player + 0xbfc), dead=boolean(player + 0x31f),
                                         controlsEnabled=boolean(player + 0x7d4))
            result['status'] = 'pass'
        finally:
            os.close(fd)
    except (OSError, ValueError, KeyError, TypeError, PermissionError, TimeoutError, struct.error) as error:
        result.update(status='fail', error=str(error))
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return result


def observe(pid, context):
    label = context.get('label', '')
    if not label.startswith('living-'):
        return controls.observe(pid, context)
    actual = controls.standard.observe(pid, context)
    restart = owner(pid, context)
    result = dict(schema=1, status='pass', standard=actual, restart=restart,
                  memoryWritten=False, executableMappingsRead=False, label=label,
                  scope='Actual public gate; independent Restart/System/Persistent owners; no Native value enters Browser inputs')
    if actual.get('status') != 'pass' or restart.get('status') != 'pass':
        result['status'] = 'fail'
    if restart.get('systemState') == 2:
        result['playingKeyboard'] = controls.playing.observe(pid, context)
        if result['playingKeyboard'].get('status') != 'pass':
            result['status'] = 'fail'
    return result
