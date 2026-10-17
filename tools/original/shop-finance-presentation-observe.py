"""Opt-in positive-price Shop logical Sprite observations; no game inputs.

Source contract: sprite-reader-source-review.json. No canonical observer edits.
"""
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools/original' / filename)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


identity = module('shop_presentation_identity', 'native-identity.py')
maps_reader = module('shop_presentation_maps', 'state-observer.py')
NATIVE_SHA = identity.NATIVE_SHA


def observe(pid, context):
    report = {'schema': 1, 'status': 'running', 'memoryWritten': False,
              'executableMappingsRead': False, 'bytesRead': 0,
              'nativeBinarySha256': NATIVE_SHA, 'checkpoint': context.get('label'),
              'scope': 'Actual Player copied asset and manager-member Shop body/price logical Sprite fields; no texture/pixel claim'}
    stopped, fd = False, None
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Requires positive original PID at blocked public gate')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = int(next(line.split(':', 1)[1] for line in status.splitlines() if line.startswith('PPid:')))
        if parent != os.getpid() or identity.native_binary_sha256(pid) != NATIVE_SHA:
            raise PermissionError('Requires pinned directly owned original child')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while not all(task.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt'
                      for task in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic() > deadline:
                raise TimeoutError('Original threads did not stop')
            time.sleep(.001)
        ranges = maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)

        def read(address, count):
            if count < 0 or report['bytesRead'] + count > 2 * 1024 * 1024:
                raise ValueError('Shop presentation read budget exceeded')
            if not any(r['start'] <= address and address + count <= r['end'] for r in ranges):
                raise PermissionError('Shop presentation field left writable source maps')
            value = os.pread(fd, count, address)
            report['bytesRead'] += count
            if len(value) != count:
                raise OSError('Short Shop presentation field read')
            return value

        def u64(address):
            return struct.unpack('<Q', read(address, 8))[0]

        def u32(address):
            return struct.unpack('<I', read(address, 4))[0]

        def i32(address):
            return struct.unpack('<i', read(address, 4))[0]

        def boolean(address):
            value = read(address, 1)[0]
            if value not in (0, 1):
                raise ValueError('Invalid source Sprite boolean')
            return value == 1

        def f32(address):
            value = struct.unpack('<f', read(address, 4))[0]
            if not math.isfinite(value):
                raise ValueError('Nonfinite source Sprite scalar')
            return value

        def string(address, bound):
            text = u64(address)
            if not text:
                raise ValueError('Null source old-ABI string backing')
            count = u64(text - 24)
            if count > bound:
                raise ValueError('Source Sprite string exceeds observer bound')
            value = read(text, count).decode('ascii')
            if '\0' in value:
                raise ValueError('Embedded NUL in source Sprite logical name')
            return value

        def clip(address):
            pointer = u64(address)
            return (pointer, string(pointer, 128)) if pointer else (0, None)

        def sprite(address, price_layers=False):
            pointer, animation = clip(address + 0x18)
            _, overlay = clip(address + 0x138)
            result = {'asset': string(address, 512), 'loaded': boolean(address + 0x2dd),
                      'animation': animation, 'animationFrame': f32(address + 0x120) if pointer else None,
                      'animationPlaying': boolean(address + 0x124) if pointer else None,
                      'overlay': overlay, 'overlayFrame': f32(address + 0x240) if overlay else None}
            if not price_layers:
                return result
            count = u32(address + 0x270)
            if count > 32:
                raise ValueError('Source Sprite global layer count exceeds observer bound')
            result.update(globalLayerCount=count, layers=[])
            if pointer:
                if animation not in ('NumbersWhite', 'NumbersRed') or count != 3:
                    raise ValueError('Opted-in price reader requires three-layer numeric price clip')
                rows, row_count = u64(pointer + 8), u32(pointer + 0x10)
                if row_count != 3:
                    raise ValueError('Numeric price clip requires exactly three layer rows')
                raw_rows = read(rows, row_count * 0x18)
                metadata = []
                for row in range(row_count):
                    offset = row * 0x18
                    layer_id = u32(rows + offset)
                    frames = struct.unpack_from('<Q', raw_rows, offset + 8)[0]
                    frame_count = struct.unpack_from('<I', raw_rows, offset + 0x10)[0]
                    if layer_id >= count or frame_count != 12:
                        raise ValueError('Numeric price layer ID/keyframe count violates scoped asset')
                    metadata.append((layer_id, frames, frame_count, boolean(rows + offset + 0x14)))
                if len({row[0] for row in metadata}) != row_count:
                    raise ValueError('Duplicate global IDs in active price clip rows')
                records = u64(address + 0x268)
                mapped = set()
                for layer_id in range(count):
                    slot = i32(pointer + 0x3c + layer_id * 4)
                    if slot < -1 or slot >= row_count:
                        raise ValueError('Price global-to-clip layer mapping exceeds active rows')
                    selected, frame_visible, row_visible = None, None, None
                    if slot >= 0:
                        if slot in mapped or metadata[slot][0] != layer_id:
                            raise ValueError('Price layer mapping is duplicate or disagrees with row ID')
                        mapped.add(slot)
                        selected = i32(address + 0x20 + slot * 4)
                        _, frames, frame_count, row_visible = metadata[slot]
                        if selected < -1 or selected >= frame_count:
                            raise ValueError('Price selected keyframe index exceeds row bounds')
                        if selected >= 0:
                            frame_visible = boolean(frames + selected * 0x6c + 0x2c)
                    result['layers'].append({'id': layer_id, 'visible': boolean(records + layer_id * 0x70 + 0x64),
                                             'clipLayerSlot': slot, 'clipLayerVisible': row_visible,
                                             'selectedKeyframeIndex': selected, 'selectedKeyframeVisible': frame_visible})
                if mapped != set(range(row_count)):
                    raise ValueError('Numeric price mapping leaves a declared clip row unused')
            elif count != 0:
                raise ValueError('Absent price clip with nonempty layer table is outside scoped clear state')
            result['clearedLogicalShapeObserved'] = (result['asset'] == '' and not result['loaded'] and
                                                      animation is None and overlay is None and count == 0)
            return result

        system = u64(0x8ed560)
        report['systemCounter'] = u32(system + 0x2efa0)
        game = u64(0x8ecd30)
        first, end = u64(game + 0xa148), u64(game + 0xa150)
        if end < first or (end - first) % 8 or not 1 <= (end - first) // 8 <= 4:
            raise ValueError('Actual Player list exceeds scoped bound')
        report['players'] = []
        for at in range(first, end, 8):
            player = u64(at)
            if u64(player) != 0x670c90:
                raise ValueError('Actual Player class differs from existing source reader')
            report['players'].append({'sequenceId': u32(player + 8), 'spawnSeed': u32(player + 0x7d0),
                                      'pickupSprite': sprite(player + 0x7e0)})
        room = u64(game + 0x8180)
        report['roomType'] = u32(room + 0x10)
        report['pickups'] = []
        if report['roomType'] == 2:
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512:
                raise ValueError('Actual manager exceeds existing entity bound')
            pointers = struct.unpack('<' + 'Q' * count, read(u64(manager + 0x68), count * 8)) if count else []
            for actor in pointers:
                if not actor or u32(actor + 0x10) != 5:
                    continue
                price = i32(actor + 0xac0)
                if price < 0:
                    raise ValueError('Negative price is outside this opted-in positive-price witness')
                report['pickups'].append({'sequenceId': u32(actor + 8), 'spawnSeed': u32(actor + 0x7d0),
                                         'priceAc0': price, 'pickupChargeAb8': i32(actor + 0xab8),
                                         'pickupTouchedAbc': boolean(actor + 0xabc),
                                         'pickupSavedFlagAbdRaw': read(actor + 0xabd, 1)[0],
                                         'bodySprite': sprite(actor + 0x28),
                                         'priceSprite': sprite(actor + 0x7d8, price_layers=True)})
        report['status'] = 'pass'
    except (OSError, ValueError, PermissionError, TimeoutError, struct.error, UnicodeError) as error:
        report.update(status='fail', error=str(error))
    finally:
        if fd is not None:
            os.close(fd)
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report
