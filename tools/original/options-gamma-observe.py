"""Read-only Gamma owners on the existing directly-owned public clock gate."""
import importlib.util
import json
import math
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


identity = load('gamma_identity', ROOT/'tools/original/native-identity.py')
reader = load('gamma_maps', ROOT/'tools/original/state-observer.py')
standard = load('gamma_standard', ROOT/'tools/original/foundation-paired-observer.py')
options = load('gamma_options', ROOT/'tools/original/options-owner-reader.py')


def observe(pid, context):
    result = dict(schema=1, status='running', memoryWritten=False, executableMappingsRead=False, bytesRead=0)
    stopped = False
    try:
        out = Path(context['output']).resolve()
        case = json.loads((out.parent/'case.json').read_text())
        contract = json.loads((out.parent/'map-contract.json').read_text())
        labels = [a['label'] for a in case['tape'] if a['type'] == 'capture']
        if case.get('declarationMode') != 'gamma-main-options-source-v1' or contract.get('gamma') is not True \
                or contract.get('preferenceSchema') != 4 or labels != contract['nativeLabels'] \
                or context.get('label') not in labels or out.name != 'original':
            raise ValueError('Explicit whole Gamma declaration required')
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Directly-owned Gamma Native must be blocked at its actual public gate')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = int(next(s.split(':',1)[1] for s in status.splitlines() if s.startswith('PPid:')))
        if parent != os.getpid() or identity.native_binary_sha256(pid) != identity.NATIVE_SHA:
            raise PermissionError('Gamma Native process/executable ownership changed')
        label = context['label']
        result['standard'] = standard.observe(pid, context)
        if result['standard']['status'] != 'pass':
            raise ValueError('Actual standard observer failed')
        if label in contract['provenanceOnlyLabels']:
            result.update(status='pass', gamma=dict(status='skip', reason='Declared provenance only; no Gamma owner claimed'))
            return result
        menu = label in contract['menuLabels']
        if menu:
            result['options'] = options.observe(pid, context)
            if result['options']['status'] != 'pass':
                raise ValueError('Actual Options owner failed')
        elif label not in contract['playingLabels'] or result['standard'].get('gameplay',{}).get('status') != 'pass':
            raise ValueError('Gamma owner lifetime is undeclared or invalid')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        until = time.monotonic()+2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > until:
                raise TimeoutError('Gamma Native did not stop')
            time.sleep(.001)
        maps = reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        descriptor = os.open(f'/proc/{pid}/mem', os.O_RDONLY|os.O_CLOEXEC)
        try:
            def read(address, size):
                if result['bytesRead']+size > 128 or not any(m['start'] <= address and address+size <= m['end'] for m in maps):
                    raise PermissionError('Gamma read exceeds admitted writable owner')
                data = os.pread(descriptor, size, address)
                if len(data) != size:
                    raise OSError('Short Gamma owner read')
                result['bytesRead'] += size
                return data
            u32 = lambda p:struct.unpack('<I',read(p,4))[0]
            u64 = lambda p:struct.unpack('<Q',read(p,8))[0]
            def scalar(p):
                raw = read(p,4)
                value = struct.unpack('<f',raw)[0]
                if not math.isfinite(value):
                    raise ValueError('Nonfinite actual display owner')
                return dict(value=value,bits=struct.unpack('<I',raw)[0])
            system = u64(0x8ed560)
            enabled = read(system+0x2dfd5,1)[0]
            if enabled not in (0,1):
                raise ValueError('Invalid color correction enable byte')
            selected = dict(status='pass', systemCounter=u32(system+0x2efa0),
                value=scalar(system+0x2dfa0), exposure=scalar(system+0x2dfa4),
                brightness=scalar(system+0x2dfa8), contrast=scalar(system+0x2dfac), enabled=bool(enabled))
            if menu:
                if result['options']['systemCounter'] != selected['systemCounter']:
                    raise ValueError('Options/Gamma observations crossed the actual gate')
                menus = u64(0x8ed570)
                # SetFrame applies each layer independently even though Counter's
                # root clip has length1. The last drawn digit remains layer3's
                # selected keyframe; the complete number is checked in real RGBA.
                selected['lastCounterDigit'] = u32(menus+0x67db8+0x2f8+0x2c)
            result.update(status='pass', gamma=selected)
        finally:
            os.close(descriptor)
    except (OSError, ValueError, KeyError, TypeError, PermissionError, TimeoutError, struct.error) as error:
        result.update(status='fail', error=str(error))
    finally:
        if stopped:
            try:
                os.kill(pid,signal.SIGCONT)
            except OSError as error:
                result.update(status='fail', resumeError=repr(error))
    return result
