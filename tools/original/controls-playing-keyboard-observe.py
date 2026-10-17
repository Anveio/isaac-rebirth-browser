"""Playing-only actual Keyboard/inputconfigs owners; no MenuManager access.
Reuses only the already reviewed Controls offsets/parser/ownership helpers.
No game writes, code mapping reads, function invocation or browser inputs.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('controls_existing_native_owners', ROOT / 'tools/original/controls-keyboard-observe.py')
controls = importlib.util.module_from_spec(spec)
spec.loader.exec_module(controls)

def observe(pid, context):
    result = {'schema':1, 'status':'running', 'memoryWritten':False, 'functionsInvoked':False,
              'executableMappingsRead':False, 'bytesRead':0, 'scope':'Playing InputManager/actualKeyboard/physical arrays/actual owned file only; no MenuManager read'}
    stopped = False
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Requires a directly owned blocked Native child')
        status = Path(f'/proc/{pid}/status').read_text()
        parent = next(row.split(':',1)[1].strip() for row in status.splitlines() if row.startswith('PPid:'))
        if int(parent) != os.getpid() or controls.identity.native_binary_sha256(pid) != controls.NATIVE_SHA:
            raise PermissionError('Native child/executable ownership mismatch')
        os.kill(pid,signal.SIGSTOP);stopped=True;deadline=time.monotonic()+2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic()>deadline:raise TimeoutError('Native did not stop')
            time.sleep(.001)
        mappings=controls.maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd=os.open(f'/proc/{pid}/mem',os.O_RDONLY|os.O_CLOEXEC)
        try:
            def read(address,size):
                if size<0 or result['bytesRead']+size>6*1024 or not any(m['start']<=address and address+size<=m['end']for m in mappings):raise PermissionError('Input read leaves bounded writable data')
                value=os.pread(fd,size,address);result['bytesRead']+=len(value)
                if len(value)!=size:raise OSError('Short input read')
                return value
            u32=lambda at:struct.unpack('<I',read(at,4))[0]
            u64=lambda at:struct.unpack('<Q',read(at,8))[0]
            system=u64(0x8ed560)
            if u32(system)!=2:raise ValueError('Playing keyboard owner requires actual Playing System state2')
            result['systemCounter']=u32(system+0x2efa0)
            begin,end=u64(0x934868),u64(0x934870)
            if end<begin or(end-begin)%8 or(end-begin)//8>32:raise ValueError('InputManager vector outside32-device bound')
            keyboard=None;devices=[]
            for at in range(begin,end,8):
                device=u64(at);identity=u32(device+8);vtable=u64(device);devices.append({'id':identity,'vtable':vtable})
                if vtable==0x6919d0:
                    if keyboard is not None or identity!=0:raise ValueError('Ambiguous Keyboard identity')
                    keyboard=device
            if keyboard is None or u32(keyboard+0x1c)!=24:raise ValueError('Actual Keyboard24 owner absent')
            enabled=read(keyboard+0xd,1)[0]
            if enabled not in(0,1):raise ValueError('Invalid Keyboard enabled byte')
            rows=struct.unpack('<24Q',read(u64(keyboard+0x20),192));bindings=[list(struct.unpack('<2i',read(row,8)))for row in rows]
            current,previous=read(0x936380,349),read(0x9360c0,349)
            if any(value not in(0,1)for value in current+previous):raise ValueError('Nonboolean physical arrays')
            held=[bool(enabled)and any(0<=key<=348 and current[key]for key in pair)for pair in bindings]
            triggered=[bool(enabled)and any(0<=key<=348 and current[key]and not previous[key]for key in pair)for pair in bindings]
            fresh=[key for key in range(349)if current[key]and not previous[key]]if enabled else[]
            result['input']={'actualDevices':devices,'keyboardId':0,'enabled':bool(enabled),'bindings':bindings,'currentPhysical':[key for key in range(349)if current[key]],'previousPhysical':[key for key in range(349)if previous[key]],'derivedSourceQueryResults':{'held':held,'triggered':triggered,'rawKey':fresh[0]if fresh else-1}}
            raw=read(0x8ecf60,1024)
            if b'\0'not in raw:raise ValueError('Userdata root is not bounded')
            data=Path(raw.split(b'\0',1)[0].decode('utf8'))
            if not data.is_absolute()or data.resolve()!=Path(context['output']).resolve()/'profile/data/binding of isaac rebirth':raise PermissionError('Actual userdata root differs from owned profile')
            result['file']=controls.actual_file(data.resolve()/'inputconfigs.dat')
            if result['file']['status']!='pass':raise ValueError('Playing requires actual complete saved inputconfigs.dat')
            result['status']='pass'
        finally:os.close(fd)
    except (OSError,ValueError,KeyError,TypeError,PermissionError,TimeoutError,struct.error)as error:
        result.update(status='fail',error=str(error))
    finally:
        if stopped:os.kill(pid,signal.SIGCONT)
    return result
