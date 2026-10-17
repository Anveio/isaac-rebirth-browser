"""Source-bound Native Controls observation at an owned public gate.
No Browser inputs, game writes, function injection or executable mapping reads.
"""
import hashlib
import importlib.util
import os
from pathlib import Path
import re
import signal
import stat
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
MEMORY_LIMIT = 12 * 1024
FILE_LIMIT = 128 * 1024

def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result

identity = module('controls_native_identity', ROOT / 'tools/original/native-identity.py')
NATIVE_SHA = identity.NATIVE_SHA
maps_reader = module('controls_writable_maps', ROOT / 'tools/original/state-observer.py')

def actual_file(path):
    """Retain actual bytes and independently parse only the complete Source save.
    Numeric attribute names are TinyXML language, rejected by standard XML parsers.
    This reader imports no Browser decoder, bindings or fixture expectations.
    """
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    except FileNotFoundError:
        return {'status': 'absent', 'path': str(path), 'scope': 'Actual absence at this gate; no save success inferred'}
    try:
        info = os.fstat(descriptor)
        if not stat.S_ISREG(info.st_mode) or info.st_size > FILE_LIMIT:
            raise ValueError('Actual inputconfigs.dat is not a bounded regular file')
        data = os.read(descriptor, FILE_LIMIT + 1)
        if len(data) != info.st_size or len(data) > FILE_LIMIT:
            raise ValueError('Actual inputconfigs.dat read is incomplete or exceeds bound')
    finally:
        os.close(descriptor)
    text = data.decode('utf8')
    report = {'status': 'read', 'path': str(path), 'bytes': len(data),
              'sha256': hashlib.sha256(data).hexdigest(), 'text': text}
    try:
        body = text.lstrip('\ufeff').strip()
        body = re.sub(r'^<\?xml\s+[^?]*\?>\s*', '', body)
        root = re.fullmatch(r'<KeyConfigs\s*>([\s\S]*)</KeyConfigs\s*>', body)
        if root is None:
            raise ValueError('Actual file lacks the Source KeyConfigs root')
        body = root.group(1).strip()
        devices = []
        names = set()
        while body:
            node = re.match(r'''<Controller\b((?:[^"'<>]|"[^"]*"|'[^']*')*)(?:/>|>\s*</Controller\s*>)''', body)
            if node is None:
                raise ValueError('Actual file contains unsupported Controller syntax')
            attributes = {}
            source = node.group(1)
            while source.strip():
                attribute = re.match(r'''\s+(Name|\d+-[01])\s*=\s*(?:"([^"]*)"|'([^']*)')''', source)
                if attribute is None or attribute.group(1) in attributes:
                    raise ValueError('Actual file contains invalid or duplicate attributes')
                attributes[attribute.group(1)] = attribute.group(2) if attribute.group(2) is not None else attribute.group(3)
                source = source[len(attribute.group(0)):]
            if len(attributes) != 49 or 'Name' not in attributes:
                raise ValueError('Actual file must contain Name plus all48 Source columns')
            #No default filling: all48 actual saved attributes must exist. Named
            #entities are decoded independently, once, without Browser admission.
            def entity(match):
                value = match.group(1)
                named = {'amp': '&', 'lt': '<', 'gt': '>', 'quot': '"', 'apos': "'"}
                if value in named:
                    return named[value]
                if value.startswith('#x'):
                    return chr(int(value[2:], 16))
                if value.startswith('#'):
                    return chr(int(value[1:], 10))
                raise ValueError('Unsupported actual Name entity')
            raw_name = attributes['Name']
            if re.search(r'&(?!(?:amp|lt|gt|quot|apos|#\d+|#x[\da-fA-F]+);)', raw_name):
                raise ValueError('Unrecognized actual Name entity')
            name = re.sub(r'&([^;]+);', entity, raw_name)
            if not name or name in names:
                raise ValueError('Missing or duplicate actual device Name')
            names.add(name)
            bindings = []
            for action in range(24):
                pair = []
                for column in range(2):
                    value = attributes[f'{action}-{column}']
                    if re.fullmatch(r'-?\d+', value) is None or not -2147483648 <= int(value) <= 2147483647:
                        raise ValueError('Actual saved binding is not signed i32')
                    pair.append(int(value))
                bindings.append(pair)
            devices.append({'name': name, 'bindings': bindings})
            if len(devices) > 64:
                raise ValueError('Actual file exceeds bounded64-device observation')
            body = body[len(node.group(0)):].strip()
        report.update({'status': 'pass', 'devices': devices,
                       'scope': 'Actual complete file at this owned gate; not a write event or process-reload claim'})
    except (ValueError, KeyError, OverflowError) as error:
        report.update({'status': 'fail', 'error': str(error)})
    return report


def observe(pid, context):
    report = {'schema': 1, 'status': 'running', 'nativeBinarySha256': NATIVE_SHA,
              'memoryWritten': False, 'functionsInvoked': False, 'executableMappingsRead': False,
              'bytesRead': 0, 'checkpoint': context.get('label'), 'originalPid': pid,
              'scope': 'Actual Menu9/10/device24x2/physical poll operands and actual inputconfigs.dat; Source query results derived independently from these Native operands. No Browser runtime inputs or save/reload/pixel/controller parity inferred.'}
    stopped = False
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Requires directly owned Native at acknowledged presentation gate')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = next(line.split(':', 1)[1].strip() for line in status.splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or identity.native_binary_sha256(pid) != NATIVE_SHA:
            raise PermissionError('Native child/executable ownership mismatch')
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
                if type(address) is not int or address <= 0 or size < 0 or report['bytesRead'] + size > MEMORY_LIMIT or not any(m['start'] <= address and address + size <= m['end'] for m in mappings):
                    raise PermissionError('Bounded Controls read outside admitted writable data')
                value = os.pread(descriptor, size, address)
                if len(value) != size:
                    raise OSError('Short Controls owner read')
                report['bytesRead'] += size
                return value
            u32 = lambda address: struct.unpack('<I', read(address, 4))[0]
            i32 = lambda address: struct.unpack('<i', read(address, 4))[0]
            u64 = lambda address: struct.unpack('<Q', read(address, 8))[0]
            def boolean(address):
                value = read(address, 1)[0]
                if value not in (0, 1):
                    raise ValueError('Invalid actual Source boolean')
                return bool(value)
            def string(address, bound):
                value = read(address, bound)
                if b'\0' not in value:
                    raise ValueError('Source string is not terminated within admitted bound')
                return value.split(b'\0', 1)[0].decode('utf8')
            system, menus = u64(0x8ed560), u64(0x8ed570)
            if not system or not menus:
                if context.get('label') != 'intro':
                    raise ValueError('Missing System/MenuManager at declared Controls checkpoint')
                report.update({'status': 'skip', 'reason': 'Declared intro before input/menu construction'})
                return report
            page = u32(menus + 0x38)
            if page > 12:
                raise ValueError('Unadmitted actual menu page')
            report['systemCounter'] = u32(system + 0x2efa0)
            report['menu'] = {'page': page, 'selectedDevice': i32(menus + 0x34)}
            picker = menus + 0x683a0
            begin, end = u64(picker + 0x2f0), u64(picker + 0x2f8)
            if end < begin or (end - begin) % 16 or (end - begin) // 16 > 6:
                raise ValueError('Actual device-picker rows exceed Source six-record view')
            count = (end - begin) // 16
            report['devices'] = {'selection': i32(picker + 4), 'held': u32(picker), 'rows': []}
            for index in range(count):
                row = begin + index * 16
                report['devices']['rows'].append({'id': i32(row + 8), 'name': string(u64(row), 64)})
            #An opening SetMenu9 precedes its first callback; an empty actual
            #row vector on that first presentation is retained, not invented.
            config = menus + 0x88700
            report['configuration'] = {'waiting': boolean(config), 'selection': u32(config + 4), 'column': u32(config + 8)}
            if report['configuration']['selection'] > 17 or report['configuration']['column'] > 1:
                raise ValueError('Actual keyboard configuration row/column outside Source')
            begin, end = u64(0x934868), u64(0x934870)
            if end < begin or (end - begin) % 8 or (end - begin) // 8 > 32:
                raise ValueError('Actual InputManager device list exceeds bounded32-device read')
            keyboard = None
            devices = []
            for index in range((end - begin) // 8):
                pointer = u64(begin + index * 8)
                actual_id, vtable = u32(pointer + 8), u64(pointer)
                devices.append({'id': actual_id, 'vtable': vtable})
                if vtable == 0x6919d0:
                    if keyboard is not None or actual_id != 0:
                        raise ValueError('Ambiguous actual Keyboard identity')
                    keyboard = pointer
            if keyboard is None:
                raise ValueError('Actual Keyboard owner is absent')
            enabled = boolean(keyboard + 0xd)
            action_count = u32(keyboard + 0x1c)
            if action_count != 24:
                raise ValueError('Actual Keyboard action count differs from Source24')
            rows = struct.unpack('<24Q', read(u64(keyboard + 0x20), 24 * 8))
            bindings = [list(struct.unpack('<2i', read(row, 8))) for row in rows]
            current, previous = read(0x936380, 349), read(0x9360c0, 349)
            if any(value not in (0, 1) for value in current + previous):
                raise ValueError('Actual physical keyboard arrays contain non-boolean bytes')
            held = [enabled and any(0 <= key <= 348 and bool(current[key]) for key in pair) for pair in bindings]
            triggered = [enabled and any(0 <= key <= 348 and bool(current[key]) and not previous[key] for key in pair) for pair in bindings]
            fresh = [key for key in range(349) if current[key] and not previous[key]] if enabled else []
            report['input'] = {'actualDevices': devices, 'keyboardId': 0, 'enabled': enabled, 'bindings': bindings,
                               'currentPhysical': [key for key in range(349) if current[key]],
                               'previousPhysical': [key for key in range(349) if previous[key]],
                               'derivedSourceQueryResults': {'held': held, 'triggered': triggered, 'rawKey': fresh[0] if fresh else -1}}
            #43C570 builds %s/inputconfigs.dat from actual writable userdata root.
            #Validate that actual root against the directly owned capture profile;
            #do not guess a path or read a different profile's retained file.
            raw_data_root = Path(string(0x8ecf60, 1024))
            if not raw_data_root.is_absolute():
                raise PermissionError('Actual userdata root is relative; Native cwd semantics remain unqualified')
            data_root = raw_data_root.resolve()
            expected = Path(context['output']).resolve() / 'profile/data/binding of isaac rebirth'
            if data_root != expected:
                raise PermissionError('Actual Native userdata path differs from owned capture profile')
            report['file'] = actual_file(data_root / 'inputconfigs.dat')
            if report['file']['status'] == 'fail':
                raise ValueError('Actual inputconfigs.dat does not match admitted complete Source format')
            report['status'] = 'pass'
        finally:
            os.close(descriptor)
    except (OSError, ValueError, KeyError, TypeError, PermissionError, TimeoutError, struct.error) as error:
        report.update({'status': 'fail', 'error': str(error)})
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report
