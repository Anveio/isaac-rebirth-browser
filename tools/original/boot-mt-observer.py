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
   manager=u64(0x8ed560);menus=u64(0x8ed570);game=u64(0x8ecd30);saved=manager+0x4a8;system_state=u32(manager)
   samples=[];level_samples=[];saved_rows=[];level_rows=[]
   for room in [0,1,134]:
    # GameState room record0 begins+f8, stridef0; gridvector starts+58.
    pointer=u64(saved+0xf8+room*0xf0+0x58)
    for slot in [0,1,447]:
     words=struct.unpack('<7I',read(pointer+slot*0x1c,0x1c))
     samples.append({'roomRecord':room,'gridIndex':slot,'seed10':words[4],'seed18':words[6]})
    saved_rows.append({'roomRecord':room,'gridIndex':struct.unpack('<i',read(saved+0xf8+room*0xf0,4))[0]})
   if game:
    for room in [0,1,134]:
     # 5881b6: record base+10, stridef0; +14 is its room index field.
     pointer=u64(game+0x10+room*0xf0+0x58)
     for slot in [0,1,447]:
      words=struct.unpack('<7I',read(pointer+slot*0x1c,0x1c))
      level_samples.append({'roomRecord':room,'gridIndex':slot,'seed10':words[4],'seed18':words[6]})
     level_rows.append({'roomRecord':room,'gridIndex':struct.unpack('<i',read(game+0x10+room*0xf0,4))[0]})
   saved_run=None;live_grid=None;saved_pools=None;current_samples=[];door_cell_records=[]
   if game and system_state==2:
    indices=[struct.unpack('<i',read(game+0x10+room*0xf0,4))[0] for room in range(135)]
    allocated=[room for room,index in enumerate(indices) if index>=0]
    copied=[room for room,index in enumerate(indices) if index!=-3]
    saved_run={'seedText':seed_string(saved+0xef80),'seededRun':bool(read(saved+4,1)[0]),'stage':u32(saved+0x94),'gameUpdateCount':u32(saved+0x50),
     'redHp':u32(saved+0x88a0),'maxRedHp':u32(saved+0x88a4),'inventory':{'coins':u32(saved+0x88b8),'bombs':u32(saved+0x88c0),'keys':u32(saved+0x88bc)},'allocatedRoomRecords':allocated,'copiedRoomRecords':copied}
    room=u64(game+0x8180);width=u32(room+0x14);height=u32(room+0x18);cells=width*height
    if not 0<cells<=448:raise ValueError('Live room grid exceeds source capacity')
    pointers=struct.unpack('<'+'Q'*cells,read(room+0x30,cells*8))
    live_grid={'width':width,'height':height,'emptyIndices':[index for index,pointer in enumerate(pointers) if not pointer]}
    descriptor=u64(room+8);current_record=u32(descriptor+4)
    if current_record>=135 or descriptor!=game+0x10+current_record*0xf0:raise ValueError('Current room descriptor identity is outside the source Level table')
    saved_run['currentRoomRecord']=current_record
    level_grid=u64(descriptor+0x58);saved_grid=u64(saved+0xf8+current_record*0xf0+0x58)
    for index in [0,1,16,17,56,85,103,134,447]:
     level_words=struct.unpack('<7I',read(level_grid+index*0x1c,0x1c));saved_words=struct.unpack('<7I',read(saved_grid+index*0x1c,0x1c))
     pointer=pointers[index] if index<cells else 0
     def record(words):return {'kind':words[0],'variant':words[1],'state':words[2],'sourceWord14':words[3],'primarySeed':words[4],'initialized':bool(words[5]&255),'secondarySeed':words[6]}
     sample={'gridIndex':index,'levelSeeds':[level_words[4],level_words[6]],'savedSeeds':[saved_words[4],saved_words[6]],'levelRecord':record(level_words),'savedRecord':record(saved_words),'occupancy':'padding' if index>=cells else 'occupied' if pointer else 'empty'}
     if pointer:sample['object']={'kind':u32(pointer+8),'variant':u32(pointer+0xc),'primarySeed':u32(pointer+0x18),'secondarySeed':u32(pointer+0x20)}
     current_samples.append(sample)
    for index in [7,60,74,127]:
     if index>=cells:raise ValueError('Starting-room door sample exceeds actual grid')
     level_words=struct.unpack('<7I',read(level_grid+index*0x1c,0x1c));saved_words=struct.unpack('<7I',read(saved_grid+index*0x1c,0x1c))
     door_cell_records.append({'gridIndex':index,'levelRecord':record(level_words),'savedRecord':record(saved_words)})
    pools=game+0x8bd0;weight_samples=[]
    for pool in [0,15]:
     record=pools+8+pool*0x48;begin=u64(record+8);end=u64(record+0x10)
     if end<begin or (end-begin)%24 or (end-begin)//24>1024:raise ValueError('Item pool weights exceed source bounds')
     if begin!=end:
      for pointer in sorted(set([begin,end-24])):
       item_id=u32(pointer)
       if item_id>=347:raise ValueError('Item pool ID exceeds source catalogue')
       weight_samples.append({'poolId':pool,'itemId':item_id,'weight':f32(saved+0x9694+4*(pool*347+item_id)),'liveWeight':f32(pointer+8)})
    saved_pools={'poolSeed':u32(saved+0xed94),'rootRngSeed':u32(saved+0xed9c),'secondaryRngSeed':u32(saved+0xeda0),'specialFactor':f32(saved+0xeda4),
     'specialCounters':[struct.unpack('<i',read(saved+0xed54+4*pool,4))[0] for pool in range(16)],
     'removedSamples':[{'itemId':item,'removed':bool(read(saved+0xeda8+item,1)[0])} for item in [0,1,346]],
     'pillEffects':list(struct.unpack('<10i',read(saved+0xef48,40))),'weightSamples':weight_samples}
   anm=menus+0x60d10+8
   loaded=bool(menus) and bool(read(anm+0x2dd,1)[0])
   title={'loaded':loaded,'menuScreen':u32(menus+0x38) if menus else None}
   if loaded:title.update({'position':f32(anm+0x120),'speed':f32(anm+0x2d8),'playing':bool(read(anm+0x124,1)[0]),'menuRunSeed':u32(menus+0x60d00),'menuSeedText':seed_string(menus+0x60d08),'characterState':u32(menus+0x62440),'characterSeedInput':seed_string(menus+0x62440+0x1758),'characterSeedCommitted':bool(read(menus+0x62440+0xc,1)[0]),'statsSamples':[{'index':i,'pose':u32(menus+0x64488+0x310+4*i)} for i in range(179)]})
   return {'schema':1,'status':'pass','scope':'bounded original writable GameState seeds, MT state and source-located title animation clock; no memory writes or gameplay input',
    'nativeBinarySha256':native_sha,'memoryWritten':False,'executableMappingsRead':False,'mtIndex':u32(0x8e6b64),'mtState':list(struct.unpack('<624Q',read(0x8ed680,624*8))),
    'managerCounter':u32(manager+0x2efa0),'systemState':system_state,'systemSeededRun':bool(read(manager+0x11,1)[0]),
    'gameRunSeed':u32(game+0xa190) if game else None,'gameSeedText':seed_string(game+0xa1e0) if game and system_state==2 else None,
    'gameSeededRun':bool(read(game+0x10ecd0,1)[0]) if game and system_state==2 else None,
    'stateSeed9688':u32(saved+0x9688),'gridSeedSamples':samples,'levelGridSeedSamples':level_samples,'savedRows':saved_rows,'levelRows':level_rows,'savedRun':saved_run,'liveGrid':live_grid,'savedItemPools':saved_pools,'currentRoomSeedSamples':current_samples,'doorCellRecords':door_cell_records,'title':title}
  finally:os.close(fd)
 finally:os.kill(pid,signal.SIGCONT)
