"""Bounded original death owner observations; no port inputs or writes."""
import hashlib
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('death_paired_observer', ROOT/'tools/original/foundation-paired-observer.py')
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)
EXECUTABLES = {}
spec_audio = importlib.util.spec_from_file_location('death_audio_observer', ROOT/'tools/original/audio-observer.py')
audio = importlib.util.module_from_spec(spec_audio)
spec_audio.loader.exec_module(audio)


def death_fields(pid):
    os.kill(pid, signal.SIGSTOP)
    try:
        deadline = time.monotonic() + 2
        while Path(f'/proc/{pid}/status').read_text().split('State:')[1].strip()[0] not in 'Tt':
            if time.monotonic() > deadline: raise TimeoutError('Original did not stop for bounded death observation')
            time.sleep(.001)
        maps = paired.combat.reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def allowed(address, count):
                return any(m['start'] <= address and address+count <= m['end'] for m in maps)
            def read(address, count):
                if not allowed(address, count): raise PermissionError('Death observer left writable source fields')
                raw = os.pread(fd, count, address)
                if len(raw) != count: raise OSError('Short death field read')
                return raw
            def u32(address): return struct.unpack('<I', read(address,4))[0]
            def u64(address): return struct.unpack('<Q', read(address,8))[0]
            def animation(address):
                frame, playing = struct.unpack('<fB',read(address+0x120,5))
                definition = u64(address+0x18)
                name = None
                if definition and allowed(definition,8):
                    text = u64(definition)
                    if text and allowed(text-0x18,8):
                        length = u64(text-0x18)
                        if 0 < length <= 80 and allowed(text,length): name = read(text,length).decode('ascii')
                return {'name':name,'frame':frame,'playing':bool(playing)}
            game = u64(0x8ecd30)
            system = u64(0x8ed560)
            system_state = u32(system)
            if system_state != 2: return {'status':'pass','systemState':system_state,'popupState':u32(game+0xa938)}
            begin,end = u64(game+0xa148),u64(game+0xa150)
            if end <= begin: raise ValueError('Gameplay has no actual primary Player owner')
            player = u64(begin)
            flags = read(player+0x31d,3)
            popup = game+0xa938
            state = u32(popup)
            if state not in (0,1,2): raise ValueError('Unknown source popup state')
            result = {'status':'pass','systemState':system_state,'controlsEnabled':bool(read(player+0x7d4,1)[0]),'playerAddress':player,'visible':bool(flags[0]),'active':bool(flags[1]),'dead':bool(flags[2]),
                'animationGuard':bool(read(player+0xbf4,1)[0]),'characterType':u32(player+0xbfc),
                'velocity':list(struct.unpack('<ff',read(player+0x464,8))),'mainAnimation':animation(player+0x28),
                'popupState':state,'systemIterations':u32(system+0x2efa0),'fullGameUpdates':u32(game+0xce9f4)}
            definition = u64(player+0x438)
            result['commonDeathFields'] = {'soundGate':u32(player+0x400), 'gibAmount':u32(definition+0x48),
                'gibFlags':u64(definition+0x50),'entityFlags':u32(player+0x318)}
            if state == 2:
                result['deathScreen'] = animation(popup+8)
                result['exitWidget'] = animation(popup+0x2e8)
                result['restartWidget'] = animation(popup+0x5c8)
            return result
        finally: os.close(fd)
    finally: os.kill(pid,signal.SIGCONT)


def observe(pid,context):
    if pid not in EXECUTABLES:
        executable = Path(f'/proc/{pid}/exe')
        EXECUTABLES[pid] = {'path': str(executable.resolve()), 'sha256': hashlib.sha256(executable.read_bytes()).hexdigest()}
    death = death_fields(pid)
    if death['systemState'] == 2:
        result = paired.observe(pid,context)
    else:
        # System dispatches Menu here; Game retains no current gameplay Room.
        # Observe the live owner rather than dereferencing cleared Room fields.
        menu = paired.menu.observe(pid,context)
        result = {'schema':1,'status':menu['status'],'menu':menu,
                  'gameplay':{'status':'skip','systemState':death['systemState'],
                              'scope':'Actual System is outside gameplay; no Room read attempted'}}
    result['death'] = death
    sounds = audio.observe(pid,context)
    result['deathSound'] = {key: sounds.get(key) for key in ('status','memoryWritten','globalSfxVolume')}
    result['deathSound']['slot30'] = next((slot for slot in sounds.get('slots',[]) if slot['id']==30),None)
    result['processExecutable'] = EXECUTABLES[pid]
    result['scope'] = 'Independent original death fields, public menu/gameplay and SFX30 wrapper; no native state supplies browser inputs'
    result['memoryWritten'] = False
    result['executableMappingsRead'] = False
    return result
