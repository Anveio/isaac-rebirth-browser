"""Bounded read-only original ItemPool/configuration/player acquisition evidence."""
import importlib.util
import math
import os
from pathlib import Path
import signal
import struct
import time
import xml.etree.ElementTree as ET
ROOT=Path(__file__).resolve().parents[2]
NATIVE_SHA='724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
identity_spec=importlib.util.spec_from_file_location('native_identity',ROOT/'tools/original/native-identity.py')
native_identity=importlib.util.module_from_spec(identity_spec);identity_spec.loader.exec_module(native_identity)
spec=importlib.util.spec_from_file_location('pool_maps',ROOT/'tools/original/state-observer.py')
maps_reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(maps_reader)
items=ET.parse(ROOT/'references/isaac/rebirth/resources/items.xml').getroot()
expected={int(x.attrib['id']):dict(x.attrib,kind=x.tag) for x in items if x.tag!='trinket'}
names=['treasure','shop','boss','devil','angel','secret','library','challenge','goldenChest','redChest','beggar','demonBeggar','curse','keyMaster','bossrush','dungeon']
spec=importlib.util.spec_from_file_location('item_world',ROOT/'tools/original/foundation-pickup-observe.py')
world_reader=importlib.util.module_from_spec(spec);spec.loader.exec_module(world_reader)
def observe(pid,context):
 report={'schema':1,'status':'running','scope':'Actual16 pool slots, sparse authored config, eligibility and acquisition fields; readonly, no native values enter production','nativeBinarySha256':NATIVE_SHA,'originalPid':pid,'memoryWritten':False,'executableMappingsRead':False,'bytesRead':0,'label':context.get('label')}
 stopped=False
 try:
  if type(pid)is not int or pid<=0 or context.get('gateBlocked')is not True:raise PermissionError('Requires positive direct-child PID at public gate')
  status=Path(f'/proc/{pid}/status').read_text();ppid=next(x.split(':',1)[1].strip() for x in status.splitlines() if x.startswith('PPid:'))
  if int(ppid)!=os.getpid():raise PermissionError('Not directly owned')
  if native_identity.native_binary_sha256(pid)!=NATIVE_SHA:raise ValueError('Unexpected original ELF')
  os.kill(pid,signal.SIGSTOP);stopped=True;deadline=time.monotonic()+2
  while not all(p.joinpath('status').read_text().split('State:')[1].strip()[0]in'Tt' for p in Path(f'/proc/{pid}/task').iterdir()):
   if time.monotonic()>deadline:raise TimeoutError('Threads did not stop')
   time.sleep(.001)
  ranges=maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text());fd=os.open(f'/proc/{pid}/mem',os.O_RDONLY|os.O_CLOEXEC)
  try:
   def read(a,n):
    if n<0 or report['bytesRead']+n>2*1024*1024 or not any(r['start']<=a and a+n<=r['end'] for r in ranges):raise ValueError(f'Outside bounded writable data {a:x}/{n}')
    b=os.pread(fd,n,a);report['bytesRead']+=n
    if len(b)!=n:raise OSError('Short read')
    return b
   def u64(a):return struct.unpack('<Q',read(a,8))[0]
   def i32(a):return struct.unpack('<i',read(a,4))[0]
   def f32(a):
    v=struct.unpack('<f',read(a,4))[0]
    if not math.isfinite(v):raise ValueError('Nonfinite native scalar')
    return v
   def bools(a,n):
    b=list(read(a,n))
    if any(x not in(0,1)for x in b):raise ValueError('Invalid native bool')
    return b
   def clip_name(actor,offset=0x18):
    pointer=u64(actor+offset)
    if not pointer:return None
    text=u64(pointer);count=u64(text-24)
    if count>128:raise ValueError('Animation name outside bound')
    return read(text,count).decode('ascii')
   def sprite(actor):
    animation=clip_name(actor);overlay=clip_name(actor,0x138)
    # Empty Sprite channels have no active clip/clock contract. Observe only
    # meaningful active fields, as native Update40f920 tests the clip first.
    return {'animation':animation,'animationFrame':f32(actor+0x120)if animation else None,
            'animationPlaying':bools(actor+0x124,1)[0]==1 if animation else None,
            'overlay':overlay,'overlayFrame':f32(actor+0x240)if overlay else None}
   system=u64(0x8ed560);report['systemState']=i32(system);report['systemCounter']=struct.unpack('<I',read(system+0x2efa0,4))[0]
   report['achievements']=bools(system+0x30,179)
   report['sharedMt']={'index':struct.unpack('<I',read(0x8e6b64,4))[0],
                       'words':list(struct.unpack('<624Q',read(0x8ed680,624*8)))}
   report['collectionSounds']=[]
   for sound in (128,171,218):
    slot=system+0x12c20+sound*0x158
    report['collectionSounds'].append({'id':sound,'lastTick':i32(slot),'nextAllowed':i32(slot+4),
                                      'lastVolume':f32(slot+0x14c),'loaded':bools(slot+0x150,1)[0]==1})
   configs=[]
   for item_id in range(347):
    pointer=u64(system+0x2dfe0+8*item_id)
    if not pointer:
     if item_id in expected:raise ValueError(f'Missing authored config{item_id}')
     continue
    data=read(pointer,0x50);kind,actual_id=struct.unpack_from('<ii',data)
    if item_id not in expected or actual_id!=item_id or kind!={'passive':1,'active':3,'familiar':4}[expected[item_id]['kind']]:raise ValueError('Authored config kind/ID mismatch')
    achievement=struct.unpack_from('<i',data,0x20)[0]
    if achievement< -1 or achievement>178:raise ValueError('Invalid achievement')
    eligible=achievement<0 or bools(system+0x30+achievement,1)[0]==1
    configs.append({'id':item_id,'kind':kind,'achievement':achievement,'cacheFlags':struct.unpack_from('<I',data,0x24)[0],'inventory':list(struct.unpack_from('<7i',data,0x28)),'maxCharges':struct.unpack_from('<i',data,0x44)[0],'cooldown':struct.unpack_from('<i',data,0x48)[0],'special':bool(data[0x4c]),'eligible':eligible})
   report['configs']=configs
   game=u64(0x8ecd30);report['gamePresent']=bool(game)and report['systemState']==2
   if report['gamePresent']:
    root=game+0x8bd0;pool_rows=[]
    report['poolSeed']=struct.unpack('<I',read(root,4))[0]
    report['rootRngWords']=list(struct.unpack('<4I',read(root+0x48c,16)))
    report['secondaryRngWords']=list(struct.unpack('<4I',read(root+0x49c,16)))
    report['pillEffects']=list(struct.unpack('<10i',read(root+0x9b8,40)))
    report['runSeeds']={'startSeed':struct.unpack('<I',read(game+0xa190,4))[0], 'rngWords':list(struct.unpack('<4I',read(game+0xa194,16))), 'initializedFields':list(struct.unpack('<13I',read(game+0xa1a4,52)))}
    for index,name in enumerate(names):
     record=root+8+0x48*index;begin,end=u64(record+8),u64(record+0x10)
     if end<begin or(end-begin)%24 or(end-begin)//24>1024:raise ValueError('Pool array outside bound')
     entries=[]
     for at in range(begin,end,24):
      data=read(at,24);item_id=struct.unpack_from('<i',data)[0]
      if item_id not in expected or data[0x14]not in(0,1)or data[0x15]not in(0,1):raise ValueError('Invalid pool entry')
      vals=list(struct.unpack_from('<4f',data,4))
      if not all(math.isfinite(x)for x in vals):raise ValueError('Invalid pool weights')
      entries.append({'id':item_id,'baseWeight':vals[0],'currentWeight':vals[1],'decreaseBy':vals[2],'removeOn':vals[3],'eligible':bool(data[0x14]),'special':bool(data[0x15])})
     pool_rows.append({'id':index,'name':name,'baseWeightTotal':f32(record),'rngWords':list(struct.unpack('<8I',read(record+0x20,32))),'specialCounter':i32(record+0x40),'entries':entries})
    report['pools']=pool_rows;report['specialFactor']=f32(root+0x4ac);report['removed']=bools(root+0x4b0,347);report['selected']=bools(root+0x60b,347);report['roomRoutes']=list(struct.unpack('<23i',read(root+0x768,92)))
    first,end=u64(game+0xa148),u64(game+0xa150)
    if end<first or(end-first)%8 or end-first>32:raise ValueError('Player list outside bound')
    players=[]
    for at in range(first,end,8):
     player=u64(at)
     if u64(player)!=0x670c90:raise ValueError('Unexpected player class')
     queued=u64(player+0x3058)
     # 52f310 initializes touched when storing an actual queued config.
     # 52ee40 tests the pointer first and never uses an inactive touched byte.
     # Record that raw byte without asserting a nonexistent active bool.
     touched_raw=read(player+0x3064,1)[0]
     touched=bools(player+0x3064,1)[0]if queued else None
     players.append({'ownedCounts':list(struct.unpack('<347i',read(player+0xe1c,347*4))),'queuedId':i32(queued+4)if queued else None,'queuedCharge':i32(player+0x3060),'queuedTouched':touched,'queuedTouchedRaw':touched_raw,'cacheFlags':struct.unpack('<I',read(player+0xd6c,4))[0],'activeId':i32(player+0xd74),'activeCharge':i32(player+0xd78),'pickupCooldown':i32(player+0x30d4),
                     'itemRngWords':[list(struct.unpack('<4I',read(player+0x1424+16*i,16)))for i in range(347)],
                     'trinketRngWords':[list(struct.unpack('<4I',read(player+0x29d4+16*i,16)))for i in range(62)],
                     'pocketRngWords':[list(struct.unpack('<4I',read(player+0x2db4+16*i,16)))for i in range(41)],
                     'managerRngWords':list(struct.unpack('<4I',read(game+0xa138,16))),
                     'entitySpawnSeed':struct.unpack('<I',read(player+0x7d0,4))[0],
                     'health':[i32(player+offset)for offset in (0xbb4,0xbb8,0xbc0,0xbc4)],
                     'inventory':{'bombs':i32(player+0xbd4),'keys':i32(player+0xbcc),'coins':i32(player+0xbd8)},
                     'pickupAnimationGuard':bools(player+0xbf4,1)[0]==1,'mainSprite':sprite(player+0x28),
                     'pickupSprite':sprite(player+0x7e0),
                     'activeSubordinateCharge':i32(player+0xd7c),'activeStateD80':i32(player+0xd80),
                     'activeStateD84':i32(player+0xd84),'reserveCharge':i32(player+0xd8c),
                     'activeFlag3076':bools(player+0x3076,1)[0]==1 if i32(player+0xd74)else None})
    report['players']=players
    room=u64(game+0x8180);manager=room+0x15d0
    count=struct.unpack('<I',read(manager+0x74,4))[0]
    if count>512:raise ValueError('Active entity budget exceeded')
    pointers=struct.unpack('<'+'Q'*count,read(u64(manager+0x68),count*8))if count else[]
    pedestals=[]
    for pointer in pointers:
     if not pointer:continue
     entity=read(pointer+0x10,12);kind,variant,subtype=struct.unpack('<III',entity)
     if kind!=5 or variant!=100:continue
     charge=read(pointer+0xab8,8)
     born=struct.unpack('<I',read(pointer+0x42c,4))[0]
     full=struct.unpack('<I',read(game+0xce9f4,4))[0]
     pedestals.append({'subtype':subtype,'charge':struct.unpack_from('<i',charge)[0],'touched':bool(charge[4]),'age':(full-born)&0xffffffff,'bornGameUpdate':born,'currentGameUpdate':full})
    report['pedestals']=pedestals
   report['status']='pass'
  finally:os.close(fd)
 except Exception as error:report['status']='fail';report['error']=repr(error)
 finally:
  if stopped:os.kill(pid,signal.SIGCONT)
 if report['status']=='pass'and report.get('gamePresent'):
  report['world']=world_reader.observe(pid,context)
  if report['world']['status']!='pass':report['status']='fail';report['error']='Native world observer failed'
 return report
