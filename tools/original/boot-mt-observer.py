"""Bounded parent-only menu-clock and GameState seed observations; no memory writes."""
import importlib.util,os,signal,struct,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
identity_spec=importlib.util.spec_from_file_location('native_identity',ROOT/'tools/original/native-identity.py')
identity=importlib.util.module_from_spec(identity_spec);identity_spec.loader.exec_module(identity)
spec=importlib.util.spec_from_file_location('state_reader',ROOT/'tools/original/state-observer.py');reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(reader)
def observe(pid,context):
 parent=next(line.split(':',1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
 if int(parent)!=os.getpid() or not context.get('gateBlocked'):raise PermissionError('Requires owned child at presentation gate')
 native_sha=identity.native_binary_sha256(pid)
 os.kill(pid,signal.SIGSTOP)
 try:
  deadline=time.monotonic()+2
  while not all(task.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt' for task in Path(f'/proc/{pid}/task').iterdir()):
   if time.monotonic()>deadline:raise TimeoutError('Original did not stop')
   time.sleep(.001)
  maps=reader.writable_maps(Path(f'/proc/{pid}/maps').read_text());fd=os.open(f'/proc/{pid}/mem',os.O_RDONLY|os.O_CLOEXEC)
  try:
   def read(addr,size):
    if not any(m['start']<=addr and addr+size<=m['end'] for m in maps):raise PermissionError(f'Read outside writable maps {addr:#x}')
    b=os.pread(fd,size,addr)
    if len(b)!=size:raise OSError('Short read')
    return b
   u32=lambda addr:struct.unpack('<I',read(addr,4))[0]
   u64=lambda addr:struct.unpack('<Q',read(addr,8))[0]
   f32=lambda addr:struct.unpack('<f',read(addr,4))[0]
   def seed_string(addr):
    pointer=u64(addr);length=u64(pointer-0x18)
    if length>9:raise ValueError('Source menu seed text exceeds bounded public code')
    return read(pointer,length).decode('ascii') if length else ''
   manager=u64(0x8ed560);menus=u64(0x8ed570);saved=manager+0x4a8
   samples=[]
   for room in [0,1,134]:
    # GameState room record0 begins+f8, stridef0; gridvector starts+58.
    pointer=u64(saved+0xf8+room*0xf0+0x58)
    for slot in [0,1,447]:
     words=struct.unpack('<7I',read(pointer+slot*0x1c,0x1c))
     samples.append({'roomRecord':room,'gridIndex':slot,'seed10':words[4],'seed18':words[6]})
   anm=menus+0x60d10+8
   loaded=bool(menus) and bool(read(anm+0x2dd,1)[0])
   title={'loaded':loaded,'menuScreen':u32(menus+0x38) if menus else None}
   if loaded:title.update({'position':f32(anm+0x120),'speed':f32(anm+0x2d8),'playing':bool(read(anm+0x124,1)[0]),'menuRunSeed':u32(menus+0x60d00),'menuSeedText':seed_string(menus+0x60d08),'characterState':u32(menus+0x62440),'characterSeedInput':seed_string(menus+0x62440+0x1758),'characterSeedCommitted':bool(read(menus+0x62440+0xc,1)[0]),'statsSamples':[{'index':i,'pose':u32(menus+0x64488+0x310+4*i)} for i in range(179)]})
   return {'schema':1,'status':'pass','scope':'bounded original writable GameState seeds, MT state and source-located title animation clock; no memory writes or gameplay input',
    'nativeBinarySha256':native_sha,'memoryWritten':False,'executableMappingsRead':False,'mtIndex':u32(0x8e6b64),'mtState':list(struct.unpack('<624Q',read(0x8ed680,624*8))),
    'managerCounter':u32(manager+0x2efa0),'stateSeed9688':u32(saved+0x9688),'gridSeedSamples':samples,'title':title}
  finally:os.close(fd)
 finally:os.kill(pid,signal.SIGCONT)
