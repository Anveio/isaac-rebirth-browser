#!/usr/bin/env python3
"""Reference-only seed relationships in bounded writable original-child data.

No executable or read-only mappings, instructions, addresses, raw dumps, or
native layouts are exported. Matches remain candidate data until validated by
the actual game log, independent seeds, and temporal observations.
"""
import importlib.util
import os
from pathlib import Path
import signal
import struct
import time

import numpy as np

_spec=importlib.util.spec_from_file_location('original_state_maps',Path(__file__).with_name('state-observer.py'))
_maps=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(_maps)

# Published xorshift triplets, not original binary instructions. Index60 has a
# conflicting public transcription; both alternatives are observed explicitly.
# https://gist.github.com/bladecoding/17b341ed08ff94d2deb704ebda8ffc5f
PUBLIC_SHIFTS = (
    (1, 3, 10), (1, 5, 16), (1, 5, 19), (1, 9, 29), (1, 11, 6), (1, 11, 16),
    (1, 19, 3), (1, 21, 20), (1, 27, 27), (2, 5, 15), (2, 5, 21), (2, 7, 7),
    (2, 7, 9), (2, 7, 25), (2, 9, 15), (2, 15, 17), (2, 15, 25), (2, 21, 9),
    (3, 1, 14), (3, 3, 26), (3, 3, 28), (3, 3, 29), (3, 5, 20), (3, 5, 22),
    (3, 5, 25), (3, 7, 29), (3, 13, 7), (3, 23, 25), (3, 25, 24), (3, 27, 11),
    (4, 3, 17), (4, 3, 27), (4, 5, 15), (5, 3, 21), (5, 7, 22), (5, 9, 7),
    (5, 9, 28), (5, 9, 31), (5, 13, 6), (5, 15, 17), (5, 17, 13), (5, 21, 12),
    (5, 27, 8), (5, 27, 21), (5, 27, 25), (5, 27, 28), (6, 1, 11), (6, 3, 17),
    (6, 17, 9), (6, 21, 7), (6, 21, 13), (7, 1, 9), (7, 1, 18), (7, 1, 25),
    (7, 13, 25), (7, 17, 21), (7, 25, 12), (7, 25, 20), (8, 7, 23), (8, 9, 23),
    (9, 5, 14), (9, 5, 25), (9, 11, 19), (9, 21, 16), (10, 9, 21), (10, 9, 25),
    (11, 7, 12), (11, 7, 16), (11, 17, 13), (11, 21, 13), (12, 9, 23), (13, 3, 17),
    (13, 3, 27), (13, 5, 19), (13, 17, 15), (14, 1, 15), (14, 13, 15), (15, 1, 29),
    (17, 15, 20), (17, 15, 23), (17, 15, 26),
)
CHUNK=1024*1024
OVERLAP=256


def next_state(state, shifts):
    a,b,c=shifts
    state ^= state >> a
    state ^= (state << b)&0xffffffff
    state ^= state >> c
    return state&0xffffffff


def inverse(state, shifts):
    a,b,c=shifts
    for shift,right in ((c,True),(b,False),(a,True)):
        original=state;value=state
        for amount in range(shift,32,shift):
            value ^= original>>amount if right else (original<<amount)&0xffffffff
        state=value
    return state


def stage_positions(stage_seed, length=4096):
    result={};state=stage_seed
    for offset in range(length+1):
        result.setdefault(state,[]).append(offset)
        state=next_state(state,PUBLIC_SHIFTS[35])
    return result


def scan_block(block, valid_from=0):
    """Find only numeric triples/tuple states; return local offsets internally."""
    words=np.frombuffer(block[:len(block)//4*4],dtype='<u4')
    descriptors=[];rng=[]
    if len(words)>=3:
        value=words[:-2]
        first=value^(value>>np.uint32(2));first=first^(first<<np.uint32(7));first=first^(first>>np.uint32(9))
        value2=words[1:-1]
        second=value2^(value2>>np.uint32(2));second=second^(second<<np.uint32(7));second=second^(second>>np.uint32(9))
        for index in np.flatnonzero((value!=0)&(first==value2)&(second==words[2:])):
            index=int(index);offset=index*4
            if offset+12<=valid_from:continue
            neighbors=words[max(0,index-24):min(len(words),index+27)]
            descriptors.append((offset,tuple(int(v) for v in words[index:index+3]),
                                {int(v) for v in neighbors if v<=168}))
    for index,shifts in enumerate(PUBLIC_SHIFTS+((9,5,1),)):
        pattern=struct.pack('<III',*shifts);offset=block.find(pattern)
        while offset>=0:
            if offset>=4 and offset+12>valid_from and offset%4==0:
                rng.append((offset-4,index,struct.unpack_from('<I',block,offset-4)[0]))
            offset=block.find(pattern,offset+1)
    return descriptors,rng


def summarize(descriptors,rng,log,positions):
    rows=[]
    for triple,metadata in sorted(descriptors.items()):
        decor,spawn,following=triple;initial=inverse(decor,PUBLIC_SHIFTS[12]);at=positions.get(initial,[])
        rows.append({'decorationSeedCandidate':decor,'spawnSeedCandidate':spawn,
                     'followingSeedCandidate':following,'derivedRoomInitSeed':initial,
                     'stage35Positions':at,'matchesActualCurrentSpawn':spawn==log.get('spawnSeed'),
                     'occurrences':metadata['occurrences'],
                     'mappingKinds':sorted(metadata['mappingKinds']),
                     'smallNeighborValues':sorted(metadata['smallNeighborValues']),
                     'smallNeighborMeaning':'untyped nearby integers0..168; no grid assignment inferred'})
    current=[row for row in rows if row['matchesActualCurrentSpawn']]
    stage_rows=[row for row in rows if row['stage35Positions']]
    return {'seedChainCandidates':rows,'publicRngTupleStates':[
                {'shiftIndex':idx if idx<81 else None,'shifts':PUBLIC_SHIFTS[idx] if idx<81 else (9,5,1),
                 'alternativePublicIndex':60 if idx==81 else None,'uniqueStates':sorted(states)}
                for idx,states in sorted(rng.items())],
            'validation':{'currentSpawnCandidates':len(current),
                          'currentSpawnStage35Candidates':sum(bool(row['stage35Positions']) for row in current),
                          'stage35MemberCandidates':len(stage_rows),'actualLogRoomCount':log.get('rooms'),
                          'candidateCountEqualsLogRoomCount':len(stage_rows)==log.get('rooms'),
                          'stage35Offsets':sorted({n for row in stage_rows for n in row['stage35Positions']}),
                          'descriptorLayoutKnown':False,'gridAssignmentsKnown':False,
                          'simulationTickKnown':False,'fullInternalState':False}}


def observe(pid,context):
    started=time.monotonic();stopped=False
    report={'schema':1,'status':'running','scope':'reference-only candidate seed relationships; no native addresses/layouts or game instructions exported',
            'checkpoint':context.get('label'),'memoryWritten':False,'executableMappingsRead':False,
            'bytesRead':0,'coverageComplete':True,'mappingKindsRead':[], 'simulationTicks':False,'fullInternalState':False}
    descriptors={};rng={};seen=set();mapping_kinds=set()
    try:
        if not context.get('gateBlocked'):raise ValueError('Requires blocked original presentation boundary')
        status=Path(f'/proc/{pid}/status').read_text()
        parent=next(line.split(':',1)[1].strip() for line in status.splitlines() if line.startswith('PPid:'))
        if int(parent)!=os.getpid():raise PermissionError('Requires the controller’s direct original child')
        if Path(os.readlink(f'/proc/{pid}/exe')).name!='isaac.x64':raise ValueError('Target is not the original child executable')
        log=context.get('publicLogState',{})
        if not all(type(log.get(k)) is int and 0<log[k]<=0xffffffff for k in ('seedValue','stageSeed','spawnSeed')):
            raise ValueError('Requires actual original run/stage/spawn log readback')
        maximum_mib=int(os.environ.get('ISAAC_DESCRIPTOR_MAX_MIB','128'))
        if not 1<=maximum_mib<=256:raise ValueError('Read budget must be1..256MiB')
        maximum=maximum_mib*1024*1024;report['maxBytes']=maximum
        positions=stage_positions(log['stageSeed'])
        os.kill(pid,signal.SIGSTOP);stopped=True
        deadline=time.monotonic()+2
        while True:
            state=next(line.split(':',1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('State:'))
            if state[0] in ('T','t'):break
            if time.monotonic()>deadline:raise TimeoutError('Original did not stop consistently')
            time.sleep(.001)
        mappings=_maps.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd=os.open(f'/proc/{pid}/mem',os.O_RDONLY|os.O_CLOEXEC)
        try:
            for mapping in mappings:
                remaining=maximum-report['bytesRead']
                if remaining<=0:report['coverageComplete']=False;break
                size=min(mapping['end']-mapping['start'],remaining);tail=b''
                kind='original writable data' if mapping['path'].endswith('/isaac.x64') else mapping['path'] or '[anonymous]'
                mapping_kinds.add(kind)
                for offset in range(0,size,CHUNK):
                    block=os.pread(fd,min(CHUNK,size-offset),mapping['start']+offset)
                    if not block:raise OSError('Short bounded memory read')
                    payload=tail+block;base=mapping['start']+offset-len(tail)
                    triples,states=scan_block(payload,len(tail))
                    for local,triple,neighbors in triples:
                        location=base+local
                        if location in seen:continue
                        seen.add(location)
                        entry=descriptors.setdefault(triple,{'occurrences':0,'mappingKinds':set(),'smallNeighborValues':set()})
                        entry['occurrences']+=1;entry['mappingKinds'].add(kind);entry['smallNeighborValues'].update(neighbors)
                    for _,index,value in states:rng.setdefault(index,set()).add(value)
                    report['bytesRead']+=len(block);tail=payload[-OVERLAP:]
                if size<mapping['end']-mapping['start']:report['coverageComplete']=False
        finally:os.close(fd)
        report.update(summarize(descriptors,rng,log,positions))
        validation=report['validation']
        if not validation['currentSpawnCandidates'] or not validation['currentSpawnStage35Candidates']:
            raise ValueError('Bounded probe did not validate current logged spawn and stage chain')
        report['status']='pass'
    except (OSError,ValueError,TimeoutError) as error:
        report['status']='fail';report['error']=str(error)
    finally:
        if stopped:os.kill(pid,signal.SIGCONT)
    report['mappingKindsRead']=sorted(mapping_kinds);report['durationMs']=(time.monotonic()-started)*1000
    return report
