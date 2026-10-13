"""Compose existing read-only source observers for the live paired game gate."""
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


menu = load('paired_menu_observer', 'foundation-menu-observer.py')
combat = load('paired_combat_observer', 'foundation-combat-observe.py')


def pause_state(pid):
    # Source controller is embedded in Game, not a port-injected fixture. The
    # source pause-update branch checks finished Disappear before actor Update;
    # its clear phase remains frozen and the next intermediate phase resumes.
    os.kill(pid, signal.SIGSTOP)
    try:
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline:
                raise TimeoutError('Original did not stop for pause observation')
            time.sleep(.001)
        maps = combat.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def allowed(address, length):
                return any(m['start'] <= address and address + length <= m['end'] for m in maps)
            def read(address, length):
                if not allowed(address, length):
                    raise PermissionError('Pause observation left writable source fields')
                raw = os.pread(fd, length, address)
                if len(raw) != length:
                    raise OSError('Short pause field read')
                return raw
            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            game = u64(0x8ecd30)
            controller = game + 0x8ce18
            state, popup, selection = struct.unpack('<III', read(controller, 12))
            if state not in (0, 1, 2):
                raise ValueError('Unknown source pause controller state')
            actor = controller + 0x28
            frame, playing = struct.unpack('<fB', read(actor + 0x120, 5))
            definition = u64(actor + 0x18)
            # Hidden controller actors can have no current definition. The
            # controller state/cursor are always observed; a definition name
            # is supplemental phase evidence only when it exists in data.
            name = None
            if allowed(definition, 0x40):
                name_pointer = u64(definition)
                if allowed(name_pointer - 0x18, 8):
                    length = u64(name_pointer - 0x18)
                    if length <= 80 and allowed(name_pointer, length):
                        name = read(name_pointer, length).decode('ascii')
            return {'state': state, 'paused': state != 0, 'popup': bool(popup & 255),
                    'selection': selection, 'animation': name, 'frame': frame, 'playing': bool(playing)}
        finally:
            os.close(fd)
    finally:
        os.kill(pid, signal.SIGCONT)


def observe(pid, context):
    result = {'schema': 1, 'status': 'pass', 'memoryWritten': False,
              'scope': 'Source-only bounded title/fader and current-room actor fields; never browser inputs'}
    result['menu'] = menu.observe(pid, context)
    result['gameplay'] = combat.observe(pid, context)
    if result['gameplay']['status'] == 'pass':
        result['pause'] = pause_state(pid)
    if any(result[k]['status'] not in ('pass', 'skip') for k in ('menu', 'gameplay')):
        result['status'] = 'fail'
    return result
