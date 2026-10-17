"""Bounded read-only Main-family Options owner at an owned public gate.
No browser inputs, game writes, executable-mapping reads or rendered-layer proof.
"""
import hashlib
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
# The authored public route selects only these Source-located menu pages.
ADMITTED_PAGES = frozenset((1, 2, 3, 4, 8))
SOURCE_ONLY_LABELS = frozenset(('intro',))

def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

identity = module('options_native_identity', ROOT / 'tools/original/native-identity.py')
NATIVE_SHA = identity.NATIVE_SHA
maps_reader = module('options_writable_maps', ROOT / 'tools/original/state-observer.py')

def observe(pid, context):
    report = {'schema': 1, 'status': 'running', 'nativeBinarySha256': NATIVE_SHA,
              'memoryWritten': False, 'executableMappingsRead': False, 'bytesRead': 0,
              'checkpoint': context.get('label'), 'originalPid': pid,
              'scope': 'Source-located Main-family Options, real settings and actual options.ini readback. Raw/gauge operands are not actual rendered layers; menu view is raw Native coordinate frame. No backend event or WM geometry claim.'}
    stopped = False
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Requires directly owned Native at acknowledged presentation gate')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = next(line.split(':', 1)[1].strip() for line in status.splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or identity.native_binary_sha256(pid) != NATIVE_SHA:
            raise PermissionError('Native child/executable ownership mismatch')
        if context.get('label') in SOURCE_ONLY_LABELS:
            report.update({'status': 'skip', 'reason': 'Declared source-only intro; no Options/menu/settings/file observation claimed'})
            return report
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline:
                raise TimeoutError('Original child did not stop at public gate')
            time.sleep(.001)
        mappings = maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        descriptor = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, size):
                if report['bytesRead'] + size > 4096 or not any(m['start'] <= address and address + size <= m['end'] for m in mappings):
                    raise PermissionError('Bounded Options read outside admitted writable data')
                value = os.pread(descriptor, size, address)
                if len(value) != size:
                    raise OSError('Short Options read')
                report['bytesRead'] += size
                return value
            u32 = lambda address: struct.unpack('<I', read(address, 4))[0]
            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            def boolean(address):
                value = read(address, 1)[0]
                if value not in (0, 1):
                    raise ValueError('Invalid actual source boolean')
                return bool(value)
            def scalar(address):
                raw = read(address, 4)
                value, bits = struct.unpack('<f', raw)[0], struct.unpack('<I', raw)[0]
                if not math.isfinite(value):
                    raise ValueError('Nonfinite actual f32 owner')
                return {'value': value, 'bits': bits}
            system, menus = u64(0x8ed560), u64(0x8ed570)
            report['systemCounter'] = u32(system + 0x2efa0)
            if not menus:
                raise ValueError('Missing MenuManager at selected paired Options-family checkpoint')
            page = u32(menus + 0x38)
            if page not in ADMITTED_PAGES:
                raise ValueError('Unadmitted Native MenuManager page for this declared Main-family route')
            options = menus + 0x67db8
            selection = u32(options + 4)
            if selection > 7:
                raise ValueError('Native Options selection outside source eight rows')
            report['menu'] = {'screen': page, 'moving': boolean(menus),
                              'rawView': [scalar(menus + 0x3c), scalar(menus + 0x40)],
                              'rawRequestedPageTarget': [scalar(menus + 0x88cf0 + 8 * page), scalar(menus + 0x88cf4 + 8 * page)]}
            report['options'] = {'inGame': boolean(options), 'selection': selection,
                                 'leftHeld': u32(options + 0x5d8), 'rightHeld': u32(options + 0x5dc),
                                 'verticalHeld': u32(options + 0x5e0)}
            if report['options']['inGame']:
                raise ValueError('This observer admits Main-family Options only')
            music, sfx = scalar(system + 0x2df90), scalar(system + 0x2df94)
            if not all(0 <= owner['value'] <= 1 for owner in (music, sfx)):
                raise ValueError('Native audio setting outside source clamp')
            enabled = boolean(system + 0x12c18)
            report['settings'] = {'sfxVolume': sfx, 'rawMusicVolume': music, 'musicEnabled': enabled,
                                  'effectiveMusicVolume': music if enabled else {'value': 0.0, 'bits': 0},
                                  'fullscreenRequested': boolean(system + 0x2df9c)}
            # Read the directly owned process's actual XDG-backed file, not an
            # input declaration or a copy synthesized from observed values.
            environment = dict(part.split(b'=', 1) for part in Path(f'/proc/{pid}/environ').read_bytes().split(b'\0') if b'=' in part)
            output = Path(context['output']).resolve()
            data = Path(os.fsdecode(environment[b'XDG_DATA_HOME'])).resolve()
            if data != output / 'profile/data':
                raise PermissionError('Actual Native XDG profile differs from owned capture profile')
            file = data / 'binding of isaac rebirth/options.ini'
            with file.open('rb') as stream:
                text = stream.read(65537)
            if len(text) > 65536:
                raise ValueError('Actual Options file exceeds admitted read bound')
            report['file'] = {'path': str(file), 'bytes': len(text), 'sha256': hashlib.sha256(text).hexdigest(),
                              'text': text.decode('utf8'), 'scope': 'Actual pre-exit read at this public gate; no inferred save event or process-restart qualification'}
            report['status'] = 'pass'
        finally:
            os.close(descriptor)
    except (OSError, ValueError, KeyError, TypeError, PermissionError, TimeoutError, struct.error) as error:
        report.update({'status': 'fail', 'error': str(error)})
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report
