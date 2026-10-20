"""Bounded actual Save owners and own files; never game input or a writer.

Run only after an exact source/case/artifact release. No game is run by loading
or parsing this file. The explicit repository root avoids copied-script imports.
"""
import hashlib,importlib.util,json,math,os,signal,stat,struct,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,ROOT/path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
standard=load('save_canonical_paired','tools/original/foundation-paired-observer.py')
identity=load('save_native_identity','tools/original/native-identity.py')
reader=load('save_state_reader','tools/original/state-observer.py')
# prepare-offline.sh compiles the offline SDK stub locally, so its bytes and
# export offsets depend on the toolchain. The deployed stub must be the one
# unwrap.py recorded in this depot's provenance, and the export is located in
# that file's own dynamic symbol table.
SDK_PROVENANCE=ROOT/'.cache/original-runtime/offline-depot/provenance.json'

def elf_export(data,name):
    if data[:4]!=b'\x7fELF' or data[4]!=2 or data[5]!=1:raise ValueError('SDK is not a little-endian ELF64 object')
    shoff,=struct.unpack_from('<Q',data,0x28);shentsize,shnum=struct.unpack_from('<HH',data,0x3a)
    sections=[struct.unpack_from('<IIQQQQIIQQ',data,shoff+i*shentsize) for i in range(shnum)]
    dynsym=[x for x in sections if x[1]==11]
    if len(dynsym)!=1:raise ValueError('SDK has no single dynamic symbol table')
    _,_,_,_,offset,size,link,_,_,entsize=dynsym[0];strtab=sections[link][4]
    found=[]
    for at in range(offset,offset+size,entsize):
        st_name,st_info,_,st_shndx,st_value,_=struct.unpack_from('<IBBHQQ',data,at)
        end=data.index(b'\0',strtab+st_name)
        if st_shndx and st_info&0xf==2 and data[strtab+st_name:end]==name.encode():found.append(st_value)
    if len(found)!=1:raise ValueError(f'SDK does not export one {name}')
    return found[0]
RUN_HEADER=bytes.fromhex('49534141434e475f4753523030313800')
PROFILE_HEADER=bytes.fromhex('49534141434e47534156453036522020')
ARCHIVE_LABELS={'save-pre-pause','save-before-quit','save-main-settled','save-continued-playing'}

def details(pid,context):
    parent=next(x.split(':',1)[1].strip() for x in Path(f'/proc/{pid}/status').read_text().splitlines() if x.startswith('PPid:'))
    if int(parent)!=os.getpid() or context.get('gateBlocked') is not True:raise PermissionError('Requires owned blocked child')
    native_sha=identity.native_binary_sha256(pid)
    os.kill(pid,signal.SIGSTOP)
    fd=None;row={'schema':1,'status':'running','sourcePid':pid}
    try:
        deadline=time.monotonic()+2
        while not all(p.joinpath('status').read_text().split('State:')[1].strip()[0] in 'Tt' for p in Path(f'/proc/{pid}/task').iterdir()):
            if time.monotonic()>deadline:raise TimeoutError('Not all Native tasks stopped')
            time.sleep(.001)
        map_text=Path(f'/proc/{pid}/maps').read_text();maps=reader.writable_maps(map_text);fd=os.open(f'/proc/{pid}/mem',os.O_RDONLY|os.O_CLOEXEC);read_bytes=0
        def read(address,size):
            nonlocal read_bytes
            if type(size) is not int or size<0 or read_bytes+size>2*1024*1024:raise ValueError('Memory byte bound exceeded')
            if size and not any(m['start']<=address and address+size<=m['end'] for m in maps):raise PermissionError('Read outside writable mappings')
            raw=os.pread(fd,size,address)
            if len(raw)!=size:raise OSError('Short owner read')
            read_bytes+=size;return raw
        u32=lambda a:struct.unpack('<I',read(a,4))[0]
        i32=lambda a:struct.unpack('<i',read(a,4))[0]
        u64=lambda a:struct.unpack('<Q',read(a,8))[0]
        def boolean(a):
            value=read(a,1)[0]
            if value not in (0,1):raise ValueError('Non-boolean Source byte')
            return bool(value)
        def text(a,limit):
            p=u64(a);n=u64(p-0x18)
            if n>limit:raise ValueError('Source string exceeds bound')
            return read(p,n).decode('utf8')
        def floats(a,n):
            raw=read(a,n*4);values=list(struct.unpack('<'+'f'*n,raw))
            if not all(math.isfinite(x) for x in values):raise ValueError('Nonfinite Source float')
            return {'values':values,'words':list(struct.unpack('<'+'I'*n,raw))}
        system=u64(0x8ed560);menus=u64(0x8ed570);game=u64(0x8ecd30);saved=system+0x4a8
        state=u32(system);slot=u32(system+8)
        if state not in (0,1,2,3) or slot not in (1,2,3):raise ValueError('Unadmitted System/profile owner')
        prefix=read(0x8ecf60,1024)
        if b'\0' not in prefix:raise ValueError('Unterminated Source save root')
        sdk=(ROOT/'.cache/original-runtime/offline-depot/lib64/libsteam_api.so').resolve(strict=True)
        sdk_stat=sdk.stat();sdk_bytes=sdk.read_bytes();sdk_sha=hashlib.sha256(sdk_bytes).hexdigest()
        sdk_after=sdk.stat()
        if sdk_sha!=json.loads(SDK_PROVENANCE.read_text())['derivedSteamApiSha256'] or (sdk_stat.st_dev,sdk_stat.st_ino,sdk_stat.st_size,sdk_stat.st_mtime_ns,sdk_stat.st_ctime_ns)!=(sdk_after.st_dev,sdk_after.st_ino,sdk_after.st_size,sdk_after.st_mtime_ns,sdk_after.st_ctime_ns):raise ValueError('Unmatched or changed deployed SDK')
        sdk_bases=[]
        for line in map_text.splitlines():
            parts=line.split(None,5)
            if len(parts)==6 and int(parts[2],16)==0 and parts[5]==str(sdk):
                device=tuple(int(x,16) for x in parts[3].split(':'))
                if device!=(os.major(sdk_stat.st_dev),os.minor(sdk_stat.st_dev)) or int(parts[4])!=sdk_stat.st_ino:raise ValueError('Mapped SDK inode/device differs from hashed file')
                sdk_bases.append(int(parts[0].split('-')[0],16))
        if len(sdk_bases)!=1 or u64(0x8e5ef0)!=sdk_bases[0]+elf_export(sdk_bytes,'SteamRemoteStorage'):raise ValueError('Unmatched actual SteamRemoteStorage export')
        cloud=boolean(system+0x2dfb9)
        environ=Path(f'/proc/{pid}/environ').read_bytes()
        if len(environ)>128*1024:raise ValueError('Owned environment exceeds bound')
        env=dict(x.split(b'=',1) for x in environ.split(b'\0') if b'=' in x)
        cloud_text=env.get(b'ISAAC_OFFLINE_CLOUD_DIR',b'').decode('utf8')
        expected_cloud=(Path(context['output']).resolve(strict=True)/'profile/cloud').resolve(strict=True)
        if cloud and (not cloud_text or Path(cloud_text).resolve(strict=True)!=expected_cloud):raise PermissionError('Cloud adapter root is not own isolated profile')
        root_text=prefix.split(b'\0',1)[0].decode('utf8')
        actual_root=Path(root_text).resolve(strict=True)
        expected_root=(Path(context['output']).resolve(strict=True)/'profile/data/binding of isaac rebirth').resolve(strict=True)
        if actual_root!=expected_root:raise PermissionError('Source save root is not this owned output profile')
        file_owners={'runLocalPath':text(saved+0xef98,1024),'runCloudName':text(saved+0xefa8,128),'profileLocalPath':text(system+0x20,1024),'profileCloudName':text(system+0x28,128),'cloudRequestedByte':read(system+0x2dfb9,1)[0]}
        row={'schema':1,'status':'pass','nativeBinarySha256':native_sha,'sourcePid':pid,'systemState':state,'systemCounter':u32(system+0x2efa0),'selectedProfile':slot,'actualLocalRoot':root_text,'fileOwners':file_owners,'loadedRunValid':boolean(saved+0xefa0),'savedRunChecksum':u32(saved+0xef94),'profileRunChecksum':u32(system+0x49c),'profileDirty':boolean(system+0x10),'savedGameUpdateCount':u32(saved+0x50),'liveGameUpdateCount':u32(game+0xce9f4) if game and state==2 else None,'mainSelection':u32(menus+0x62154) if menus else None,'actualBackend':{'kind':'source-cloud-adapter/offline-own-directory' if cloud else 'source-local-adapter','sdkSha256':sdk_sha,'mappedExportJoin':True,'cloudDirectory':cloud_text if cloud else None,'scope':'Actual Native GOT export is the source-bound deployed SDK nonnull factory; no Native API is called and no executable mapping read'}}
        row['liveRawContext']={'game10eb10':u32(game+0x10eb10),'game10ecd8':u32(game+0x10ecd8),'seedEffectModeA1d8':i32(game+0xa1d8)} if game and state==2 else None
        row['saved']={'rawGameWord0':u32(saved),'rawGameWord8':u32(saved+8),'heldPocketWords':list(struct.unpack('<4I',read(saved+0x8924,16))),'seedText':text(saved+0xef80,9),'seededRun':boolean(saved+4),'stage':u32(saved+0x94),'currentRoomGridIndex':i32(saved+0x8880),'previousRoomGridIndex':i32(saved+0x8884),'entrySlot':i32(saved+0x8894),'entryPosition':floats(saved+0x8888,2),'playerType':u32(saved+0x889c),'health':{'red':u32(saved+0x88a0),'maxRed':u32(saved+0x88a4),'soul':u32(saved+0x88a8),'blackMask':u32(saved+0x88ac),'eternal':u32(saved+0x88b0)},'inventory':{'coins':u32(saved+0x88b8),'bombs':u32(saved+0x88c0),'keys':u32(saved+0x88bc),'goldenKey':boolean(saved+0x88c4)},'luck':{'sourceD3c':i32(saved+0x88e4),'sourceD40':i32(saved+0x88e8),'sourceD58Bits':u32(saved+0x8900)},'trinketSlots':[u32(saved+0x8ea8),u32(saved+0x8eac)],'costumeCount':u32(saved+0x9628),'transformationCounts':[i32(saved+0x8934),i32(saved+0x8938)],'managerRng':u32(saved+0xef88),'secondaryManagerRng':u32(saved+0x9688),'lastPenaltyStage':u32(saved+0xef90),'heartCollected':boolean(saved+0xac),'levelFlagBits':[boolean(saved+0xb2+i) for i in range(11)],'ownedCounts':list(struct.unpack('<347i',read(saved+0x893c,347*4))),'itemRngStates':list(struct.unpack('<347I',read(saved+0x8eb0,347*4))),'trinketRngStates':list(struct.unpack('<62I',read(saved+0x941c,62*4))),'pocketRngStates':list(struct.unpack('<41I',read(saved+0x9514,41*4))),'activeId':u32(saved+0x8908),'activeCharge':i32(saved+0x890c),'reserveCharge':i32(saved+0x8910)}
        current=row['saved']['currentRoomGridIndex']
        if not 0<=current<169:raise ValueError('Saved current room is outside ordinary grid-cell domain')
        ordinal=i32(saved+0x85dc+current*4)
        if not 0<=ordinal<135:raise ValueError('Actual saved grid map has no bounded descriptor for current cell')
        descriptor=saved+0xf8+ordinal*0xf0
        if u32(descriptor+4)!=ordinal:raise ValueError('Actual saved descriptor ordinal join failed')
        row['saved']['currentDescriptorOrdinal']=ordinal
        row['saved']['descriptorGridIndex']=i32(descriptor)
        grid=u64(descriptor+0x58)
        row['saved']['gridRecordWords']=[list(x) for x in struct.iter_unpack('<7I',read(grid,448*0x1c))]
        #Same descriptor/grid-vector owners already read above and in the
        #canonical boot-MT reader; row127 is deliberately outside serialized
        #ordinary rows[0,count), which the Source loader requires count<128.
        row['saved']['row127GridIndex']=i32(saved+0xf8+127*0xf0)
        row127=u64(saved+0xf8+127*0xf0+0x58)
        row['saved']['row127SeedSamples']=[{'roomRecord':127,'gridIndex':slot,'seeds':[u32(row127+slot*0x1c+0x10),u32(row127+slot*0x1c+0x18)]} for slot in (0,1,447)]
        row['sharedMT']={'index':u32(0x8e6b64),'words':list(struct.unpack('<624Q',read(0x8ed680,624*8)))}
        row['memoryBytesRead']=read_bytes;row['memoryWritten']=False;row['executableMappingsRead']=False
        row['files']=[]
        for kind,basename,header,owner in [('run',f'gamestate{slot}.dat',RUN_HEADER,file_owners['runCloudName' if cloud else 'runLocalPath']),('profile',f'persistentgamedata{slot}.dat',PROFILE_HEADER,file_owners['profileCloudName' if cloud else 'profileLocalPath'])]:
            expected=(expected_cloud if cloud else actual_root)/basename
            if (cloud and owner!=basename) or (not cloud and Path(owner).resolve()!=expected):raise PermissionError('Source actual file owner does not match selected backend pair')
            file_row={'kind':kind,'basename':basename,'actualPath':str(expected),'nativeFileOwner':owner}
            try:source_fd=os.open(expected,os.O_RDONLY|os.O_CLOEXEC|os.O_NOFOLLOW)
            except FileNotFoundError:file_row['status']='absent';row['files'].append(file_row);continue
            try:
                before=os.fstat(source_fd)
                if not stat.S_ISREG(before.st_mode) or not 20<=before.st_size<=4*1024*1024:raise ValueError('Own file type/size outside bound')
                raw=b''
                while len(raw)<before.st_size:
                    chunk=os.read(source_fd,min(65536,before.st_size-len(raw)))
                    if not chunk:raise OSError('Short own file read')
                    raw+=chunk
                after=os.fstat(source_fd)
                if (before.st_dev,before.st_ino,before.st_size,before.st_mtime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns):raise OSError('Own file changed during read')
            finally:os.close(source_fd)
            file_row.update({'status':'pass','bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'headerHex':raw[:16].hex(),'headerMatchesSource':raw[:16]==header,'checksumOwner':struct.unpack('<I',raw[-4:])[0]^0x96696996 if kind=='run' else struct.unpack('<I',raw[16:20])[0]})
            if context['label'] in ARCHIVE_LABELS:
                destination=Path(context['output'])/'save-own-files'/context['label']/basename;destination.parent.mkdir(parents=True,exist_ok=True)
                with destination.open('xb') as archive:archive.write(raw)
                file_row['actualBytesArchive']=str(destination)
            row['files'].append(file_row)
        row['scope']='Actual stopped own Native Save stores/files only. Presence/dirty/checksum alone is not success; mandatory guards require real loaded validity, matched pair, actual public callbacks and semantic old fields. Native bytes never enter Browser.'
        return row
    except Exception as error:
        return {'schema':1,'status':'fail','error':str(error),'partial':row,'memoryWritten':False}
    finally:
        if fd is not None:os.close(fd)
        os.kill(pid,signal.SIGCONT)

def observe(pid,context):
    if not (context['label'].startswith('save-') or context['label']=='gameplay-start'):
        return standard.observe(pid,context)
    own=details(pid,context)
    result={'schema':1,'status':'pass','memoryWritten':False,
        'scope':'Actual System-owned menu or Game observation plus unchanged bounded own-save reader; never game inputs',
        'saveOwn':own}
    if own.get('status')!='pass':
        result['status']='fail';return result
    state=own['systemState']
    if state not in (1,2):
        result['status']='fail';result['error']='Unadmitted actual System owner for this Save route';return result
    result['menu']=standard.menu.observe(pid,context)
    if state==2:
        result['gameplay']=standard.combat.observe(pid,context)
        if result['gameplay']['status']=='pass':result['pause']=standard.pause_state(pid)
    else:
        #System1 owns MenuManager after Stop destroys the live Room. A stale
        #public-log stage is not permission to dereference the former Game.
        result['gameplay']={'schema':2,'status':'skip','actualSystemState':state,
            'scope':'Actual System1 menu; no live Room/Game read admitted'}
    if any(result[key]['status'] not in ('pass','skip') for key in ('menu','gameplay')):result['status']='fail'
    return result
