#!/usr/bin/env python3
"""Read source-located title/fade clocks from an owned child at a public gate.

These bounded debug observations establish reference phase. They are never
production input, and do not alter original game memory or executable code.
Offsets are from the pinned restored base Rebirth ELF, SHA256
724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time

_spec = importlib.util.spec_from_file_location('menu_state_reader', Path(__file__).with_name('state-observer.py'))
_reader = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_reader)


def observe(pid, context):
    status = Path(f'/proc/{pid}/status').read_text()
    parent = next(line.split(':', 1)[1].strip() for line in status.splitlines() if line.startswith('PPid:'))
    if int(parent) != os.getpid() or not context.get('gateBlocked'):
        raise PermissionError('Menu observation requires the owned child at its presentation gate')
    os.kill(pid, signal.SIGSTOP)
    try:
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline:
                raise TimeoutError('Original child did not stop for menu observation')
            time.sleep(.001)
        maps = _reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        descriptor = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, size):
                if not any(mapping['start'] <= address and address + size <= mapping['end'] for mapping in maps):
                    raise PermissionError(f'Menu observation outside writable mappings: {address:#x}')
                data = os.pread(descriptor, size, address)
                if len(data) != size:
                    raise OSError('Short original menu-state read')
                return data

            u32 = lambda address: struct.unpack('<I', read(address, 4))[0]
            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            f32 = lambda address: struct.unpack('<f', read(address, 4))[0]
            manager, menus = u64(0x8ed560), u64(0x8ed570)
            result = {'schema': 1, 'status': 'pass', 'memoryWritten': False,
                      'scope': 'Bounded debug phase observations; not game input or full internal-state parity',
                      'systemCounter': u32(manager + 0x2efa0), 'menuInitialized': bool(menus)}
            if menus:
                actor = menus + 0x60d10 + 8
                result.update({
                    'menuScreen': u32(menus + 0x38),
                    'titleAnimation': {'loaded': bool(read(actor + 0x2dd, 1)[0]),
                                       'frame': f32(actor + 0x120), 'speed': f32(actor + 0x2d8),
                                       'playing': bool(read(actor + 0x124, 1)[0])},
                    'menuFade': {'active': bool(read(menus + 8, 1)[0]), 'alpha': f32(menus + 0xc),
                                 'color': list(struct.unpack('<4f', read(menus + 0x10, 16)))},
                })
            return result
        finally:
            os.close(descriptor)
    finally:
        os.kill(pid, signal.SIGCONT)
