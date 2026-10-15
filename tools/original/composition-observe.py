"""Source-identified native composition observations; read-only.

Common Knife/Laser fields come from the actor owner's full-FDE/vtable receipt.
Player C0C is observed separately from the existing C74 firing cooldown.
This wrapper only adds comparisons; it never supplies production inputs.
"""
import importlib.util
import math
import os
import re
from pathlib import Path
import signal
import struct
import time

ROOT = Path(__file__).resolve().parents[2]
EXPECTED_SHA = os.environ.get('ISAAC_ORIGINAL_SHA')
if not isinstance(EXPECTED_SHA, str) or re.fullmatch(r'[0-9a-f]{64}', EXPECTED_SHA) is None:
    raise ValueError('ISAAC_ORIGINAL_SHA must provide the driver-verified canonical binary SHA256')
SOURCE_FIELDS = frozenset(('playerMovementD9c', 'knifeChildChain4a0', 'npc14And18DamageQueue', 'kind8TearFields', 'ordinaryTechnology', 'ordinaryDrFetusBombs', 'controlledTechnology7x2x1', 'controlledDrFetusBombs', 'monstrosLung229'))
spec = importlib.util.spec_from_file_location('composition_items', ROOT / 'tools/original/item-observer.py')
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)
VTABLES = {2: 0x671270, 8: 0x66ccf0, 7: 0x66cf70, 1000: 0x66c0d0}


def observe(pid, context):
    # The public Node driver validates its canonical original first. Independently
    # bind this directly owned observation to that required identity before reads.
    try:
        if type(pid) is not int or pid <= 0 or context.get('gateBlocked') is not True:
            raise PermissionError('Requires positive directly owned original at a blocked public gate')
        parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid():
            raise PermissionError('Original is not directly owned')
        if base.native_identity.native_binary_sha256(pid) != EXPECTED_SHA:
            raise ValueError('Original identity differs from ISAAC_ORIGINAL_SHA')
        requested_fields = context.get('compositionSourceFields', [])
        if (not isinstance(requested_fields, list)
                or any(not isinstance(name, str) or name not in SOURCE_FIELDS for name in requested_fields)
                or len(set(requested_fields)) != len(requested_fields)):
            raise ValueError('Unknown or duplicate compositionSourceFields request')
    except Exception as error:
        return {'schema': 1, 'status': 'fail', 'scope': 'composition owner/source-field admission',
                'nativeBinarySha256': EXPECTED_SHA, 'memoryWritten': False,
                'executableMappingsRead': False, 'bytesRead': 0, 'error': repr(error)}
    report = base.observe(pid, context)
    if report.get('status') != 'pass' or not report.get('gamePresent'):
        return report
    supplemental = {'schema': 1, 'scope': 'source Player weapon slots, C0C charge and common attack actors; full controller laws remain separately scoped',
                    'memoryWritten': False, 'executableMappingsRead': False, 'bytesRead': 0,
                    'nativeBinarySha256': EXPECTED_SHA, 'requestedSourceFields': requested_fields, 'players': [], 'attacks': [], 'impacts': [], 'contactNpcs': []}
    if 'npc14And18DamageQueue' in requested_fields:
        supplemental['commonContactNpcs'] = []
    if 'ordinaryDrFetusBombs' in requested_fields or 'controlledDrFetusBombs' in requested_fields:
        supplemental['fetusBombs'] = []
    stopped = False
    try:
        parent = next(line.split(':', 1)[1].strip() for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('PPid:'))
        if int(parent) != os.getpid() or context.get('gateBlocked') is not True:
            raise PermissionError('Requires directly owned original at a blocked public gate')
        if base.native_identity.native_binary_sha256(pid) != EXPECTED_SHA:
            raise ValueError('Unexpected original binary')
        os.kill(pid, signal.SIGSTOP)
        stopped = True
        deadline = time.monotonic() + 2
        while True:
            state = next(line for line in Path(f'/proc/{pid}/status').read_text().splitlines() if line.startswith('State:'))
            if state.split(':', 1)[1].strip()[0] in ('T', 't'):
                break
            if time.monotonic() >= deadline:
                raise TimeoutError('Original did not stop')
            time.sleep(.001)
        maps = base.maps_reader.writable_maps(Path(f'/proc/{pid}/maps').read_text())
        fd = os.open(f'/proc/{pid}/mem', os.O_RDONLY | os.O_CLOEXEC)
        try:
            def read(address, count):
                if supplemental['bytesRead'] + count > 256 * 1024:
                    raise ValueError('Composition read budget exceeded')
                if not any(row['start'] <= address and address + count <= row['end'] for row in maps):
                    raise ValueError(f'Source writable range missing at {address:#x}')
                data = os.pread(fd, count, address)
                if len(data) != count:
                    raise OSError('Short source data read')
                supplemental['bytesRead'] += count
                return data

            def u64(at): return struct.unpack('<Q', read(at, 8))[0]
            def u32(at): return struct.unpack('<I', read(at, 4))[0]
            def i32(at): return struct.unpack('<i', read(at, 4))[0]

            def floats(at, count=1):
                values = list(struct.unpack('<' + 'f' * count, read(at, 4 * count)))
                if not all(math.isfinite(value) for value in values):
                    raise ValueError('Nonfinite source scalar')
                return values[0] if count == 1 else values

            def flags(at, count):
                values = list(read(at, count))
                if any(value not in (0, 1) for value in values):
                    raise ValueError('Invalid source bool')
                return [bool(value) for value in values]

            def actor_ref(at):
                if not at:
                    return None
                return {'kind': u32(at + 0x10), 'variant': u32(at + 0x14), 'subtype': u32(at + 0x18),
                        'sequenceId': u32(at + 8), 'spawnSeed': u32(at + 0x7d0)}

            def actor_slot(at):
                identity = actor_ref(at)
                if identity is None:
                    return None
                visible, active, dead = flags(at + 0x31d, 3)
                return {'identity': identity, 'visible': visible, 'active': active, 'dead': dead}

            def vector_storage(at, stride=8):
                begin, end, capacity = u64(at), u64(at + 8), u64(at + 16)
                if end < begin or capacity < end or (end - begin) % stride or (capacity - begin) % stride or end - begin > 32768:
                    raise ValueError('Laser vector outside actual bounded storage')
                data = read(begin, end - begin) if end > begin else b''
                values = []
                for offset in range(0, len(data), stride):
                    value = list(struct.unpack_from('<' + 'f' * (stride // 4), data, offset))
                    if not all(math.isfinite(x) for x in value):
                        raise ValueError('Nonfinite initialized Laser vector')
                    values.append(value)
                return values

            def animation_name(at):
                definition = u64(at + 0x40)
                if not definition:
                    return None
                text = u64(definition)
                length = u64(text - 24) if text else 0
                if length > 128:
                    raise ValueError('Attack animation name outside bound')
                return read(text, length).decode('ascii') if length else None

            game = u64(0x8ecd30)
            supplemental['challengeId'] = i32(game + 0x10eb10)
            supplemental['levelCurseMask'] = u32(game + 8)
            supplemental['runFlags10ead8'] = str(u64(game + 0x10ead8))
            supplemental['gameAge'] = u32(game + 0xce9f4)
            supplemental['rawRunFlag10ecd8'] = i32(game + 0x10ecd8)
            supplemental['rawRunFlag10ecd0'] = flags(game + 0x10ecd0, 1)[0]
            supplemental['stageShootingGuard'] = {'state': i32(game + 0xa0e0), 'counter': i32(game + 0xa0e4)}
            start, end = u64(game + 0xa148), u64(game + 0xa150)
            if end < start or (end - start) % 8 or end - start > 32:
                raise ValueError('Player list outside source bound')
            slot_pointers = []
            for entry in range(start, end, 8):
                player = u64(entry)
                if u64(player) != 0x670c90:
                    raise ValueError('Unexpected Player class')
                # Slots stay distinguishable from actor active/dead flags.
                # Primary switching marks a referenced actor removed; it does
                # not itself prove that the registered weak slot was cleared.
                slots = {offset: u64(player + offset) for offset in (0xc18, 0xc20, 0xc28, 0xc30)}
                slot_pointers.extend(pointer for pointer in slots.values() if pointer)
                supplemental['players'].append({'owner': actor_ref(player), 'primaryWeaponC08': i32(player + 0xc08),
                    'chargeC0c': i32(player + 0xc0c),
                    'luckD64': floats(player + 0xd64), 'flyingD68': flags(player + 0xd68, 1)[0],
                    'gridCollisionClass340': u32(player + 0x340),
                    'headDirectionDa0': i32(player + 0xda0), 'shootingAimDb4': floats(player + 0xdb4, 2),
                    'playerDc4': floats(player + 0xdc4, 2),
                    'attackFlagsC98': str(u64(player + 0xc98)),
                    'cachedColorCA0': floats(player + 0xca0, 11), 'cachedColorCCC': floats(player + 0xccc, 11),
                    'initialized3074': flags(player + 0x3074, 1)[0], 'canShoot3075': flags(player + 0x3075, 1)[0],
                    'transformationCounts': [i32(player + 0x3068), i32(player + 0x306c)],
                    'actorSlots': {f'{offset:x}': actor_slot(pointer) for offset, pointer in slots.items()}})
                if 'ordinaryDrFetusBombs' in requested_fields or 'controlledDrFetusBombs' in requested_fields:
                    supplemental['players'][-1]['fetusCaller'] = {
                        'playerTypeBfc': i32(player + 0xbfc),
                        'trinketSlotsE14': [i32(player + 0xe14), i32(player + 0xe18)],
                        'previousVelocityDd4': floats(player + 0xdd4, 2),
                        'fireCooldownC74': i32(player + 0xc74), 'headTimerBb0': i32(player + 0xbb0)}
                if 'monstrosLung229' in requested_fields:
                    supplemental['players'][-1]['monstroCaller'] = {
                        'playerTypeBfc': i32(player + 0xbfc),
                        'trinketSlotsE14': [i32(player + 0xe14), i32(player + 0xe18)],
                        'playerFlags318': u32(player + 0x318),
                        'shootingLock7dc': u32(player + 0x7dc),
                        'position448': floats(player + 0x448, 2),
                        'previousVelocityDd4': floats(player + 0xdd4, 2)}
                if 'playerMovementD9c' in requested_fields:
                    supplemental['players'][-1]['movementDirectionD9c'] = i32(player + 0xd9c)
            room = u64(game + 0x8180)
            if 'kind8TearFields' in requested_fields:
                # 5414DA..5414E8/542E08..542E16 dereference the actual Room;
                # zero/negative/positive are read values, never a default.
                supplemental['sourceTearSlowdown2494'] = i32(room + 0x2494)
            supplemental['roomInitializedGameAge159c'] = u32(room + 0x159c)
            supplemental['roomLoaded'] = flags(room, 1)[0]
            age_delta = (supplemental['gameAge'] - supplemental['roomInitializedGameAge159c']) & 0xffffffff
            supplemental['roomAge'] = (age_delta if age_delta < 0x80000000 else age_delta - 0x100000000) if supplemental['roomLoaded'] else -1
            # Literal fields already owned by foundation-combat-observe.py.
            # Observe them here at the SAME blocked gate; they are evidence,
            # never input to either game or a derived !dead active flag.
            descriptor = u64(room + 8)
            supplemental['roomDescriptor'] = {'index': u32(descriptor + 4), 'gridIndex': u32(descriptor),
                'spawnSeed': u32(descriptor + 0x4c), 'combatClear': flags(descriptor + 0x3d, 1)[0]}
            supplemental['managerCounts'] = dict(zip(
                ('countablePending', 'countableCurrent', 'bossPending', 'bossCurrent'),
                (i32(room + offset) for offset in (0x16ac, 0x16b0, 0x16b4, 0x16b8))))
            supplemental['combatClearCountdown'] = i32(room + 0x1598)
            manager = room + 0x15d0
            count = u32(manager + 0x74)
            if count > 512:
                raise ValueError('Active entity count outside source bound')
            begin = u64(manager + 0x68)
            pointers = struct.unpack('<' + 'Q' * count, read(begin, 8 * count)) if count else []
            active_pointers = set(pointers)
            supplemental['npcManagerIdentities'] = [list(actor_ref(pointer).values()) for pointer in pointers
                if pointer and 10 <= u32(pointer + 0x10) < 1000]
            # Retain actual referenced objects even after Remove excludes them
            # from an active view; registered cleanup is a different producer.
            # Retain each actual Laser child relation even after active-manager Remove;
            # actual registered Uninit, not !dead, is the producer of NULL.
            child_pointers = [u64(pointer + 0x4a0) for pointer in [*pointers, *slot_pointers]
                if pointer and u32(pointer + 0x10) == 7 and u64(pointer) == VTABLES[7]]
            all_pointers = dict.fromkeys([*pointers, *slot_pointers, *child_pointers])
            for actor in all_pointers:
                if not actor:
                    continue
                identity = actor_ref(actor)
                common_receiver = ('npc14And18DamageQueue' in requested_fields and identity['kind'] in (14, 18))
                if identity['kind'] == 217 or common_receiver:
                    # 555700 routes14/18 through the generic1088 NPC pool;
                    # 49EA17 stores66EF30, whose+38 is446CF0. Require the
                    # actual class before interpreting its common damage FIFO.
                    if u64(actor) != 0x66ef30:
                        raise ValueError('Contact receiver must own the actual common NPC class')
                    visible, active, dead = flags(actor + 0x31d, 3)
                    begin, end = u64(actor + 0x360), u64(actor + 0x368)
                    if end < begin or (end - begin) % 0x30 or (end - begin) // 0x30 > 128:
                        raise ValueError('Contact NPC ordered queue outside source bound')
                    records = []
                    for record in range(begin, end, 0x30):
                        records.append({'amount': floats(record), 'flags': u32(record + 4),
                            'sourceType': u32(record + 8), 'sourceVariant': u32(record + 12),
                            'actualAttacker': actor_ref(u64(record + 0x28))})
                        if common_receiver:
                            # 446E24..446EAB copies the actual EntityRef. These
                            # historical values are not reread from its source.
                            records[-1].update({'sourceRaw1c': u32(record + 0x10), 'sourceRaw20': u32(record + 0x14),
                                'sourcePosition': floats(record + 0x18, 2), 'sourceVelocity': floats(record + 0x20, 2)})
                    row = {'identity': identity, 'inActiveManager': actor in active_pointers,
                        'active': active, 'dead': dead, 'visible': visible, 'hp': floats(actor + 0x484),
                        'maxHp': floats(actor + 0x488), 'pendingDamage': floats(actor + 0x378),
                        'pendingFlags': u32(actor + 0x37c), 'pendingDamageRecords': records,
                        'canShutDoors': flags(actor + 0x103c, 1)[0], 'boss': flags(actor + 0x103d, 1)[0],
                        'position': floats(actor + 0x448, 2), 'velocity': floats(actor + 0x464, 2),
                        'collisionRadius': floats(actor + 0x474), 'entityCollisionClass': u32(actor + 0x344),
                        'raw494': flags(actor + 0x494, 1)[0], 'entityFlags': u32(actor + 0x318)}
                    if common_receiver:
                        row.update({'npcState': u32(actor + 0xf50), 'npcFrameAge': u32(actor + 0xf60),
                            'champion': flags(actor + 0x103f, 1)[0]})
                        supplemental['commonContactNpcs'].append(row)
                    else:
                        supplemental['contactNpcs'].append(row)
                if identity['kind'] == 4 and ('ordinaryDrFetusBombs' in requested_fields or 'controlledDrFetusBombs' in requested_fields):
                    # 66B4D0: Init456E00/Full457060/Fast4446A0. This is a
                    # distinct Bomb owner, never a Knife/Laser field alias.
                    if u64(actor) != 0x66b4d0:
                        raise ValueError('Typed Bomb4 requires its actual virtual class')
                    visible, active, dead = flags(actor + 0x31d, 3)
                    fast = flags(actor + 0x322, 1)[0]
                    supplemental['fetusBombs'].append({
                        'identity': identity, 'inActiveManager': actor in active_pointers,
                        'visible': visible, 'active': active, 'dead': dead,
                        'bornGameAge': u32(actor + 0x42c),
                        'position448': floats(actor + 0x448, 2),
                        'savedPosition450': floats(actor + 0x450, 2) if fast else None,
                        'velocity464': floats(actor + 0x464, 2),
                        'fast322': fast,
                        'currentFriction470': floats(actor + 0x470),
                        'gridCollisionClass340': u32(actor + 0x340),
                        'gridCollided34c': flags(actor + 0x34c, 1)[0],
                        'actorRngWords7c0': [u32(actor + 0x7c0 + 4*i) for i in range(4)],
                        'fuse7d4': i32(actor + 0x7d4),
                        'explosionDamage7d8': floats(actor + 0x7d8),
                        'playerContactCountdown7dc': i32(actor + 0x7dc),
                        'attackFlags7e0': str(u64(actor + 0x7e0)),
                        'shotMarker7e8': read(actor + 0x7e8, 1)[0],
                        'cachedSpawner': {'kind': u32(actor + 0x1c), 'variant': u32(actor + 0x20)},
                        'linkedParent': actor_ref(u64(actor + 0x498)),
                        'sourceColor3a0': floats(actor + 0x3a0, 11),
                        'currentColor2d0': floats(actor + 0x2d0, 11)})
                    if 'controlledDrFetusBombs' in requested_fields:
                        #519FC0 follows the real reused Bomb4's Child4A0. The
                        #66B4D0/type4 guard above precedes this typed pointer.
                        supplemental['fetusBombs'][-1]['controlledChild4a0'] = actor_ref(u64(actor + 0x4a0))
                    #7EC's initial/global ordinal writer is unowned. Its raw
                    # padding is not read or substituted with a numeric zero.
                if identity['kind'] not in VTABLES or (identity['kind'] == 1000 and (identity['variant'],identity['subtype']) != (50,1)):
                    if not (('ordinaryTechnology' in requested_fields or 'controlledTechnology7x2x1' in requested_fields)
                            and identity['kind'] == 1000 and (identity['variant'], identity['subtype']) == (50, 0)):
                        continue
                if u64(actor) != VTABLES[identity['kind']]:
                    raise ValueError('Source attack family has an unexpected virtual class')
                visible, active, dead = flags(actor + 0x31d, 3)
                row = {'identity': identity, 'inActiveManager': actor in active_pointers,
                    'visible': visible, 'active': active, 'dead': dead,
                    'bornGameAge': u32(actor + 0x42c), 'position': floats(actor + 0x448, 2),
                    'savedPosition450': floats(actor + 0x450, 2), 'fast322': flags(actor + 0x322, 1)[0],
                    'velocity': floats(actor + 0x464, 2), 'renderOffset': floats(actor + 0x458, 2),
                    'collisionDamage': floats(actor + 0x48c), 'genericFlags318': u32(actor + 0x318),
                    'collisionRadius': floats(actor + 0x474), 'collisionMass': floats(actor + 0x480),
                    'hp': floats(actor + 0x484), 'maxHp': floats(actor + 0x488),
                    'currentFriction470': floats(actor + 0x470), 'actorRngWords7c0': [u32(actor + 0x7c0 + 4*i) for i in range(4)],
                    'configuredFriction46c': floats(actor + 0x46c),
                    'gridCollisionClass': u32(actor + 0x340), 'entityCollisionClass': u32(actor + 0x344),
                    'raw494': flags(actor + 0x494, 1)[0],
                    'currentColor2d0': floats(actor + 0x2d0, 11),
                    'permanentColor3a0': floats(actor + 0x3a0, 11),
                    'cachedSpawner': {'kind': u32(actor + 0x1c), 'variant': u32(actor + 0x20)},
                    'linkedParent': actor_ref(u64(actor + 0x498)),
                    'sprite': {'animation': animation_name(actor), 'frame': floats(actor + 0x148), 'playing': flags(actor + 0x14c, 1)[0],
                               'rotation': floats(actor + 0x2cc), 'offset': floats(actor + 0x2bc, 2), 'scale': floats(actor + 0x2c4, 2)}}
                if identity['kind'] == 2:
                    # 7D8 is falling speed in Tear; its attack flags live at
                    # 7F0. Knife's distinct attack flags are at 7D8 below.
                    row['attackFlags7f0'] = str(u64(actor + 0x7f0))
                    if 'kind8TearFields' in requested_fields:
                        writer = ((identity['variant'], identity['subtype']) == (0, 0)
                                  and bool(int(row['attackFlags7f0']) & (1 << 33)))
                        row['sourceKind8WriterScope'] = writer
                        # Completed public gates expose initialized actors:
                        # 54240F/5411E8 own83C; Init5424FB ownsB28, then
                        # Full542DDD..542DF5 writes the actual velocity delta.
                        # Fast5414A0 retainsB28; do not synthesize a Fast zero.
                        row['baseDamage83c'] = floats(actor + 0x83c) if writer else None
                        row['velocityDeltaB28'] = floats(actor + 0xb28, 2) if writer else None
                    if ('monstrosLung229' in requested_fields
                            and (identity['variant'], identity['subtype']) == (1, 0)):
                        #Canonical2/1/0 passes actual671270 before these reads.
                        #5411E8/54240F owns83C;5179DB copies actualPlayerC84
                        #into Tear838. Neither field is a correspondence alias.
                        row['monstroBaseDamage83c'] = floats(actor + 0x83c)
                        row['monstroShotOrdinal838'] = u32(actor + 0x838)
                    row['height7d4'] = floats(actor + 0x7d4)
                    row['fallingSpeed7d8'] = floats(actor + 0x7d8)
                    row['fallingAcceleration7dc'] = floats(actor + 0x7dc)
                elif identity['kind'] == 8:
                    if 'knifeChildChain4a0' in requested_fields:
                        child_chain = u64(actor + 0x4a0)
                        row['childChain4a0Null'] = child_chain == 0
                        row['childChain4a0RawPointer'] = f'{child_chain:#x}'
                    row['attackFlags7d8'] = str(u64(actor + 0x7d8))
                    row['initUniform8a8'] = floats(actor + 0x8a8)
                    row['knifeRawFloats'] = {f'{offset:x}': floats(actor + offset) for offset in
                        (0x7e0, 0x7e4, 0x7e8, 0x7ec, 0x7f0, 0x7f4, 0x7f8, 0x80c, 0x810, 0x8a4)}
                    row['knifeRaw8a0'] = u32(actor + 0x8a0)
                    row['knifeRawVectors'] = {f'{offset:x}': floats(actor + offset, 2) for offset in (0x7fc, 0x804)}
                    row['knifeRaw814'] = flags(actor + 0x814, 1)[0]
                    # Actual Knife Line818.576F50 initializes all four vectors;
                    # 473BCE/473C9C initializes mode/subdivisions/width and controls
                    # before the source subtype1 deep copy. Unowned scalar words
                    # remain raw evidence; never read them as initialized floats.
                    line = actor + 0x818
                    controls = vector_storage(line + 0x70, 24)
                    writer = (identity['variant'] == 0 and identity['subtype'] in (0, 1)
                              and u32(line + 0x14) == 30 and u32(line + 0x68) == 0
                              and len(controls) >= 2)
                    row['knifeLine'] = {
                        'origin': floats(line, 2), 'offset': floats(line + 8, 2),
                        'widthRawBits': u32(line + 0x10), 'subdivisionsRawBits': u32(line + 0x14),
                        'opaque18RawWords': [u32(line + 0x18), u32(line + 0x1c)],
                        'modeRawBits': u32(line + 0x68), 'lengthRawBits': u32(line + 0x6c),
                        'sourceOrdinaryCurveWriterScope': writer,
                        'width': floats(line + 0x10) if writer else None,
                        'subdivisions': u32(line + 0x14) if writer else None,
                        'mode': u32(line + 0x68) if writer else None,
                        'length': None,  # No admitted Knife writer owns+6C.
                        'controls': controls, 'points': vector_storage(line + 0x20),
                        'priorPoints': vector_storage(line + 0x38),
                        'history': vector_storage(line + 0x50)}
                elif identity['kind'] == 7:
                    # Exact native-laser7-layout-source-contract-v1.json,
                    # source fa208fe4. Actual Line base800, four owned vectors.
                    # Width is untouched by the admitted constructor/body: keep
                    # its raw bits without calling it a known initialized value.
                    points = vector_storage(actor + 0x820)
                    # The sealed controlled subtype1 first callback initializes
                    # Line through577120/579fa0. Ordinary subtype0 is a different
                    # unadmitted producer: retain raw words instead of treating
                    # uninitialized length/mode as finite typed values.
                    controlled_line = identity['subtype'] == 1 and len(points) >= 2
                    ordinary_line = (identity['variant'] == 1 and identity['subtype'] == 0
                        and row['linkedParent'] is not None and row['linkedParent']['kind'] == 1
                        and not flags(actor + 0x7f4, 1)[0] and len(points) >= 2)
                    known_line = controlled_line or ordinary_line
                    row['laser'] = {'parentOffset': floats(actor + 0x7d4, 2), 'angle': floats(actor + 0x7dc),
                        'previousAngle': floats(actor + 0x7e0), 'renderSortOffset': floats(actor + 0x7e4),
                        'attackFlags': str(u64(actor + 0x7e8)), 'remainingUpdates': i32(actor + 0x7f0),
                        'firstUpdate': flags(actor + 0x7f4, 1)[0], 'lineEnabled': flags(actor + 0x7f5, 1)[0],
                        'contactDisabled': flags(actor + 0x7f6, 1)[0], 'beamLength': floats(actor + 0x7f8),
                        'child': actor_ref(u64(actor + 0x4a0)), 'lineFlags': str(u64(actor + 0x818)),
                        'rotationEnabled': flags(actor + 0x888, 1)[0], 'maxDistance': floats(actor + 0x898),
                        'lineController': i32(actor + 0x89c), 'endpoint': floats(actor + 0x8a0, 2),
                        'line': {'origin': floats(actor + 0x800, 2), 'offset': floats(actor + 0x808, 2),
                            'widthRawBits': u32(actor + 0x810), 'subdivisionsRawBits': u32(actor + 0x814),
                            'modeRawBits': u32(actor + 0x868), 'lengthRawBits': u32(actor + 0x86c),
                            'sourceControlledLineWriterScope': controlled_line,
                            'sourceOrdinaryLineWriterScope': ordinary_line,
                            'width': floats(actor + 0x810) if ordinary_line else None,
                            'subdivisions': u32(actor + 0x814) if known_line else None,
                            'mode': u32(actor + 0x868) if known_line else None,
                            'length': floats(actor + 0x86c) if known_line else None,
                            'points': points, 'priorPoints': vector_storage(actor + 0x838),
                            'history': vector_storage(actor + 0x850),
                            'controls': vector_storage(actor + 0x870, 24)}}
                    if 'ordinaryTechnology' in requested_fields:
                        #479648 initializes50/mode0/width120;478320/579FA0
                        #own the resampled7/2/0 Line. Preserve old typed fields.
                        technology_writer = (identity['variant'] == 2 and identity['subtype'] == 0
                            and row['linkedParent'] is not None and row['linkedParent']['kind'] == 1
                            and not row['laser']['firstUpdate'] and row['laser']['lineEnabled']
                            and len(points) >= 2)
                        row['laser']['technologyLine'] = {
                            'sourceWriterScope': technology_writer,
                            'width': floats(actor + 0x810) if technology_writer else None,
                            'subdivisions': u32(actor + 0x814) if technology_writer else None,
                            'mode': u32(actor + 0x868) if technology_writer else None,
                            'length': floats(actor + 0x86c) if technology_writer else None}
                    if 'controlledTechnology7x2x1' in requested_fields:
                        #4799D0 initializes actual7/2/1 Line50/mode1/width60.
                        #477DC0 owns the ring vectors; preserve other typed fields.
                        controlled_technology_writer = (identity['variant'] == 2 and identity['subtype'] == 1
                            and row['linkedParent'] is not None and row['linkedParent']['kind'] == 1
                            and not row['laser']['firstUpdate'] and row['laser']['lineEnabled']
                            and len(points) >= 2)
                        row['laser']['controlledTechnologyLine'] = {
                            'sourceWriterScope': controlled_technology_writer,
                            'width': floats(actor + 0x810) if controlled_technology_writer else None,
                            'subdivisions': u32(actor + 0x814) if controlled_technology_writer else None,
                            'mode': u32(actor + 0x868) if controlled_technology_writer else None,
                            'length': floats(actor + 0x86c) if controlled_technology_writer else None}
                if identity['kind'] == 1000:
                    # v2 ordinary Impact contract5187908b:7D8 is a raw u8,
                    # render depth lives at7F8; remaining7F0 starts-1.
                    row['impact'] = {'raw7d8': read(actor + 0x7d8, 1)[0],
                        'remainingUpdates': i32(actor + 0x7f0), 'renderSortOffset': floats(actor + 0x7f8)}
                    if ('ordinaryTechnology' in requested_fields
                            and identity['variant'] == 50 and identity['subtype'] == 0):
                        #Actual479732 and479773 owners;814 is not Sprite2CC.
                        row['impact']['technology'] = {'spriteFlipX': flags(actor + 0x2fc, 1)[0],
                                                       'raw814': floats(actor + 0x814)}
                    if ('controlledTechnology7x2x1' in requested_fields
                            and identity['variant'] == 50 and identity['subtype'] == 0):
                        #479732/479773 own flip2FC and raw814 before childFull.
                        #raw814 is independent of Sprite.rotation2CC.
                        row['impact']['controlledTechnology'] = {
                            'spriteFlipX': flags(actor + 0x2fc, 1)[0], 'raw814': floats(actor + 0x814)}
                    supplemental['impacts'].append(row)
                else:
                    supplemental['attacks'].append(row)
        finally:
            os.close(fd)
        supplemental['status'] = 'pass'
        report['composition'] = supplemental
    except Exception as error:
        supplemental['status'] = 'fail'
        supplemental['error'] = repr(error)
        report['composition'] = supplemental
        report['status'] = 'fail'
        report['error'] = 'Source composition observer failed'
    finally:
        if stopped:
            os.kill(pid, signal.SIGCONT)
    return report
