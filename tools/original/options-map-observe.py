"""Unlaunched Source-only Map observer: declared menus and Playing are separate.

All observations stay private and never enter Browser inputs. This module is
only loaded later by the directly owning Native gate after Root tuple review.
"""
from pathlib import Path
import hashlib
import importlib.util
import json
import math
import os
import signal
import stat
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
MENU_LABELS = ('title-ready', 'options-initial-row-0', 'options-map-row-3', 'options-map-confirm-no-adjust', 'map-left-held-1', 'map-left-held-49', 'map-left-held-50', 'map-left-held-51', 'map-left-held-71', 'map-left-held-81', 'map-right-held-1', 'map-right-held-50', 'map-right-held-51', 'map-right-held-141', 'map-final-value-tap-4', 'options-map-back-saved')
PLAYING_LABELS = ('gameplay-start', 'map-small-steady', 'map-first-tab-held-1', 'map-first-tab-held-6', 'map-first-tab-release-1', 'map-first-tab-release-2', 'map-first-tab-release-18', 'map-second-tab-held-1', 'map-second-tab-held-2', 'map-second-tab-release-1', 'map-second-tab-release-14')
PROVENANCE_LABELS = ('intro', 'provenance-character', 'provenance-seed-entry', 'character-seeded')

def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

identity = load('map_native_identity', ROOT/'tools/original/native-identity.py')
maps_reader = load('map_native_maps', ROOT/'tools/original/state-observer.py')
standard = load('map_standard_paired', ROOT/'tools/original/foundation-paired-observer.py')
options = load('map_existing_options', ROOT/'tools/original/options-owner-reader.py')
audio = load('map_existing_audio', ROOT/'tools/original/audio-observer.py')


def declaration(context):
    #The driver archives/pins these input declarations before/after either game.
    #No expected Native outputs or fixture checksums are accepted here.
    output = Path(context['output']).resolve()
    if output.name != 'original':
        raise ValueError('Map declaration requires CASE_DIR/original layout')
    case = json.loads((output.parent/'case.json').read_text())
    contract = json.loads((output.parent/'map-contract.json').read_text())
    if case.get('declarationMode') != 'map-opacity-fast31-source-v1' or contract.get('schema') != 1 \
            or contract.get('caseDeclarationMode') != case['declarationMode']:
        raise ValueError('Unadmitted Map public declaration/schema')
    if contract.get('preferenceSchema') != 3 or contract.get('storageKey') != 'isaac-rebirth.options.v3':
        raise ValueError('Map requires explicitly declared V3 preference ownership')
    if case['tape'] != case['bootstrap']['sourceOnlyPrelude'] + case['pairedTape']:
        raise ValueError('Map full public tape must retain declared intro plus paired suffix')
    native_labels = [row['label'] for row in case['tape'] if row['type'] == 'capture']
    paired_labels = [row['label'] for row in case['pairedTape'] if row['type'] == 'capture']
    source_only = [row['label'] for row in case['bootstrap']['sourceOnlyPrelude'] if row['type'] == 'capture']
    if len(native_labels) != 31 or len(set(native_labels)) != 31 or len(paired_labels) != 30 \
            or len(set(paired_labels)) != 30 or source_only != ['intro']:
        raise ValueError('Map requires all31Native/30paired labels with explicit intro lifetime')
    for key, expected in [('nativeLabels', native_labels), ('pairedLabels', paired_labels), ('sourceOnlyLabels', source_only)]:
        if contract.get(key) != expected or case['captureDeclaration'].get(key) != expected:
            raise ValueError('Map capture declarations differ from literal public tape: '+key)
    for key, expected in [('menuLabels', MENU_LABELS), ('playingLabels', PLAYING_LABELS), ('provenanceOnlyLabels', PROVENANCE_LABELS)]:
        if contract.get(key) != list(expected):
            raise ValueError('Unadmitted Map owner-lifetime partition: '+key)
    groups = [set(MENU_LABELS), set(PLAYING_LABELS), set(PROVENANCE_LABELS)]
    if any(groups[i] & groups[j] for i in range(3) for j in range(i+1, 3)) or set.union(*groups) != set(native_labels):
        raise ValueError('Map owner lifetimes must be complete and disjoint')
    if context.get('label') not in native_labels:
        raise ValueError('Undeclared Map capture')
    return contract


def actual_file_stat(file):
    #Read the actual owned file again while blocked, retaining real text/hash
    #and metadata. This is neither a save event nor a process-reload witness.
    with Path(file['path']).open('rb') as stream:
        before = os.fstat(stream.fileno())
        data = stream.read(65537)
        after = os.fstat(stream.fileno())
    if not stat.S_ISREG(before.st_mode) or len(data) > 65536 \
            or (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns) \
            or hashlib.sha256(data).hexdigest() != file['sha256'] or len(data) != file['bytes']:
        raise ValueError('Actual Options file changed or exceeded its bounded regular owner')
    return {**file, 'mode': stat.S_IMODE(before.st_mode), 'uid': before.st_uid, 'gid': before.st_gid,
            'device': before.st_dev, 'inode': before.st_ino}


def observe(pid, context):
    result = dict(schema=1, status='running', nativeBinarySha256=identity.NATIVE_SHA, memoryWritten=False, executableMappingsRead=False,
                  checkpoint=context.get('label'), bytesRead=0)
    stopped = False
    try:
        label = context.get('label')
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Owned Native child must be blocked at its acknowledged public gate')
        process_status = Path(f'/proc/{pid}/status').read_text()
        parent = next(line.split(':',1)[1].strip() for line in process_status.splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or identity.native_binary_sha256(pid) != identity.NATIVE_SHA:
            raise PermissionError('Native child or executable ownership changed')
        plan = declaration(context)
        result['standard'] = standard.observe(pid, context)
        if result['standard']['status'] != 'pass':
            raise ValueError('Mandatory standard observer failed')
        if label in plan['provenanceOnlyLabels']:
            result.update(status='pass', map=dict(status='skip', optionsReaderCalled=False, playingReaderCalled=False, audioReaderCalled=False,
                          reason='Declared intro/seed provenance only; no settings or map manager claimed'))
            return result
        menu, playing = label in plan['menuLabels'], label in plan['playingLabels']
        if menu == playing:
            raise ValueError('Map menu/Playing owner partition must be disjoint and complete')
        selected = dict(status='running', optionsReaderCalled=menu, playingReaderCalled=playing, audioReaderCalled=menu)
        if menu:
            selected['options'] = options.observe(pid, context)
            if selected['options']['status'] != 'pass':
                raise ValueError('Existing actual Options owner failed')
            selected['options']['file'] = actual_file_stat(selected['options']['file'])
            sound = audio.observe(pid, context)
            selected['audio237'] = {key: sound.get(key) for key in ('schema', 'status', 'error', 'nativeBinarySha256',
                'memoryWritten', 'executableMappingsRead', 'bytesRead', 'systemCounter', 'globalSfxVolume')}
            selected['audio237']['slots'] = [slot for slot in sound.get('slots', []) if slot['id'] == 237]
            if selected['audio237']['status'] != 'pass' or len(selected['audio237']['slots']) != 1:
                raise ValueError('Missing actual configured237 snapshot owner')
            if selected['audio237']['systemCounter'] != selected['options']['systemCounter']:
                raise ValueError('Options/audio237 crossed the public gate')
        elif result['standard'].get('gameplay', {}).get('status') != 'pass':
            raise ValueError('Playing Map requires actual standard gameplay owner')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic()+2
        while not all(p.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt' for p in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic()>deadline:
                raise TimeoutError('Native child did not stop')
            time.sleep(.001)
        mappings = maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        descriptor = os.open(f'/proc/{pid}/mem', os.O_RDONLY|os.O_CLOEXEC)
        try:
            def read(address, size):
                if size<0 or result['bytesRead']+size>2048 or not any(m['start']<=address and address+size<=m['end'] for m in mappings):
                    raise PermissionError('Map read outside bounded writable fields')
                raw = os.pread(descriptor,size,address)
                if len(raw)!=size:
                    raise OSError('Short actual Map read')
                result['bytesRead'] += size
                return raw
            u32 = lambda address:struct.unpack('<I',read(address,4))[0]
            u64 = lambda address:struct.unpack('<Q',read(address,8))[0]
            def scalar(raw):
                value = struct.unpack('<f',raw)[0]
                if not math.isfinite(value):
                    raise ValueError('Nonfinite actual Map f32 owner')
                return dict(value=value,bits=struct.unpack('<I',raw)[0])
            system = u64(0x8ed560)
            selected['systemCounter'] = u32(system+0x2efa0)
            selected['opacity'] = scalar(read(system+0x2df98,4))
            if not 0<=selected['opacity']['value']<=1:
                raise ValueError('Actual MapOpacity outside selected fresh-profile/UI scope')
            if menu:
                if selected['options']['systemCounter'] != selected['systemCounter']:
                    raise ValueError('Menu/settings observations crossed the gate')
            else:
                game = u64(0x8ecd30)
                selected['gameFullCounter'] = u32(game+0xce9f4)
                raw = read(game+0xae010,0x680)
                word = lambda offset:struct.unpack_from('<I',raw,offset)[0]
                field = lambda offset:scalar(raw[offset:offset+4])
                if word(0) not in (0,1,2) or raw[4] not in (0,1):
                    raise ValueError('Invalid actual Map mode/latch')
                selected['presentation'] = dict(mode=word(0),expandedLatched=bool(raw[4]),heldFrames=word(8),
                    scale=field(0xc),offset=[field(0x10),field(0x14)],smallAlpha=field(0x344),expandedAlpha=field(0x67c))
                for name in ['smallAlpha','expandedAlpha']:
                    if not 0<=selected['presentation'][name]['value']<=1:
                        raise ValueError('Actual Map alpha outside fresh-profile/UI scope')
                if result['standard']['gameplay']['nativePhase']['systemIterations'] != selected['systemCounter']:
                    raise ValueError('Playing/settings observations crossed the gate')
            selected['status'] = 'pass'
            result.update(status='pass',map=selected)
        finally:
            os.close(descriptor)
    except (OSError, ValueError, KeyError, TypeError, PermissionError, TimeoutError, struct.error) as error:
        result.update(status='fail',error=str(error))
    finally:
        if stopped:
            try:
                os.kill(pid, signal.SIGCONT)
            except OSError as error:
                result.update(status='fail', resumeError=repr(error))
    return result
