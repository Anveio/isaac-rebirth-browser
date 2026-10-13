"""Bounded native clock/MT/tear observation for the Linux reference only.

Requires the controller's owned child blocked at the public presentation gate.
Reads writable nonexecutable mappings under a whole-process stop, never writes
original memory or passes observed state to the browser. The static ELF/vtable
landmarks identify reference fields, not portable production addresses.
"""
import importlib.util, os, math, signal, struct, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('reader', ROOT/'tools/original/state-observer.py')
reader=importlib.util.module_from_spec(spec); spec.loader.exec_module(reader)
MT_INDEX=0x8e6b64
MT_STATE=0x8ed680
TEAR_VTABLE=0x671270
OBJECT_SIZE=0xb30
MAX_SCAN=512*1024*1024
MAX_OBJECTS=256
tracked=[]
owner=None

def observe(pid, context):
 global owner, tracked
 if type(pid) is not int or pid <= 0: raise PermissionError('Requires a positive owned child PID')
 parent=next(line.split(':',1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
 if int(parent)!=os.getpid() or not context.get('gateBlocked'): raise PermissionError('Requires owned child blocked at a presentation gate')
 if owner!=pid: owner=pid; tracked=[]
 os.kill(pid, signal.SIGSTOP)
 try:
  deadline=time.monotonic()+2
  while not all(task.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt' for task in Path(f'/proc/{pid}/task').iterdir()):
   if time.monotonic()>deadline: raise TimeoutError('Original did not stop')
   time.sleep(.001)
  maps=reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
  def allowed(address,length): return any(m['start']<=address and address+length<=m['end'] for m in maps)
  fd=os.open(f'/proc/{pid}/mem', os.O_RDONLY|os.O_CLOEXEC)
  try:
   def read(address,length):
    if not allowed(address,length): raise PermissionError('Read would leave native writable data')
    data=os.pread(fd,length,address)
    if len(data)!=length: raise OSError('Short bounded data read')
    return data
   mt_index=struct.unpack('<I',read(MT_INDEX,4))[0]
   mt_state=list(struct.unpack('<624Q',read(MT_STATE,624*8)))
   if mt_index>624 or any(word>0xffffffff for word in mt_state):raise ValueError('Native MT data fails source-sized integrity checks')
   game=struct.unpack('<Q',read(0x8ecd30,8))[0]
   has_players=False
   if allowed(game+0xa148,16):
    first,end=struct.unpack('<QQ',read(game+0xa148,16))
    has_players=0<end-first<=32 and (end-first)%8==0 and allowed(first,end-first)
   scan_bytes=0
   # Typed player admission, not a public-frame offset, triggers the pool scan.
   if not tracked and has_players:
    for m in maps:
     if m['path'] not in ('[heap]',''): continue
     size=min(m['end']-m['start'],MAX_SCAN-scan_bytes)
     if size<=0: break
     tail=b''
     for offset in range(0,size,reader.CHUNK):
      block=os.pread(fd,min(reader.CHUNK,size-offset),m['start']+offset)
      payload=tail+block;base=m['start']+offset-len(tail); cursor=payload.find(struct.pack('<Q',TEAR_VTABLE))
      while cursor>=0:
       address=base+cursor
       if address%8==0 and allowed(address,OBJECT_SIZE) and address not in tracked:
        tracked.append(address)
        if len(tracked)>MAX_OBJECTS: raise RuntimeError('Tear pool candidate cap exceeded')
       cursor=payload.find(struct.pack('<Q',TEAR_VTABLE),cursor+1)
      scan_bytes+=len(block);tail=payload[-7:]
   tears=[]
   for address in tracked:
    if not allowed(address,OBJECT_SIZE): continue
    b=read(address,OBJECT_SIZE)
    if struct.unpack_from('<Q',b,0)[0]!=TEAR_VTABLE: continue
    f32=lambda offset:struct.unpack_from('<f',b,offset)[0]
    vals=[f32(o) for o in (0x448,0x44c,0x450,0x454,0x458,0x45c,0x464,0x468,0x7d4,0x7d8,0x7dc)]
    if struct.unpack_from('<I',b,0x10)[0] != 2:continue
    if not all(math.isfinite(v) and abs(v)<10000 for v in vals):continue
    # Zero-position inactive pool slots are omitted from concise field output;
    # their raw constructor words are not needed to infer a runtime contract.
    if vals[0]==0 and vals[1]==0:continue
    tears.append({'referenceAddress':hex(address),'position':vals[0:2], 'previousPosition':vals[2:4],
      'renderOffset':vals[4:6],'velocity':vals[6:8], 'height':vals[8], 'fallingSpeed':vals[9], 'fallingAcceleration':vals[10],
      'type':struct.unpack_from('<I',b,0x10)[0], 'variant':struct.unpack_from('<I',b,0x14)[0],
      'frameCountCandidates': [{'offset':hex(o),'u32':struct.unpack_from('<I',b,o)[0]} for o in [0x18,0x1c,0x318,0x31c,0x340,0x344]],
      'active':bool(b[0x31e]), 'dead':bool(b[0x31f]),
      'opaqueBaseBytes':b[:32].hex()})
   players=[]
   system=struct.unpack('<Q',read(0x8ed560,8))[0]
   system_phase_counter=struct.unpack('<I',read(system+0x2efa0,4))[0] if allowed(system+0x2efa0,4) else None
   if allowed(game+0xa148,16):
    first,end=struct.unpack('<QQ',read(game+0xa148,16))
    if first<=end and end-first<=32 and allowed(first,end-first):
     for address in struct.unpack('<'+'Q'*((end-first)//8),read(first,end-first)):
      if allowed(address+0xc7c,32) and allowed(address+0xd70,1):
       vals=struct.unpack('<8f',read(address+0xc7c,32))
       players.append({'referenceAddress':hex(address),'position':list(struct.unpack('<2f',read(address+0x448,8))),
        'shotSpeed':vals[0], 'damage':vals[3], 'tearHeight':vals[4], 'fallingSpeed':vals[5], 'fallingAcceleration':vals[6],
        'eye':struct.unpack('<b',read(address+0xd70,1))[0]})
   return {'schema':1,'status':'pass','scope':'reference-only bounded native writable MT state and temporally identified tear fields; no original memory is modified or passed to the port',
      'memoryWritten':False,'executableMappingsRead':False,'simulationTicks':False,
      'players':players,'systemPhaseCounter':system_phase_counter,'gameUpdateCounter':struct.unpack('<I',read(game+0xce9f4,4))[0] if allowed(game+0xce9f4,4) else None,'mtIndex':mt_index,'mtState':mt_state,'poolCandidates':len(tracked),'scanBytes':scan_bytes,'tears':tears}
  finally:os.close(fd)
 finally:os.kill(pid,signal.SIGCONT)
