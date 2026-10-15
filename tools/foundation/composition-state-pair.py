"""Actual original/browser compositional fields; unsupported/missing remain failures."""
import argparse, collections, gzip, hashlib, importlib.util, json, math, struct
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--case', type=Path, required=True)
parser.add_argument('--original', type=Path, required=True)
parser.add_argument('--browser', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()


def load(path):
    data = Path(path).read_bytes()
    return json.loads(gzip.decompress(data) if str(path).endswith('.gz') else data)


def f32(value):
    assert type(value) in (int, float) and math.isfinite(value), 'Actual finite numeric f32 field required'
    return struct.unpack('<f', struct.pack('<f', value))[0]


def vector(value):
    return [f32(v) for v in value] if isinstance(value, list) else [f32(value['x']), f32(value['y'])]


def color(value):
    return [f32(v) for group in ('tint', 'sourceMiddle', 'offsets') for v in value[group]]


def flags(value):
    assert len(value) == 2 and all(type(word) is int and 0 <= word <= 0xffffffff for word in value)
    return str(value[0] | value[1] << 32)


def source_run_flags(state):
    """Admit only the two actual serialized Game10EAD8 owners, without defaults."""
    owners = [key for key in ('sourceCacheRunFlags', 'sourceRunFlags') if key in state]
    assert len(owners) == 1, 'Exactly one actual Game10EAD8 owner required; missing/ambiguous is unqualified'
    if owners[0] == 'sourceCacheRunFlags':
        value = state['sourceCacheRunFlags']
        # This legacy u64 travels through a JavaScript number. Only exact safe
        # integers are observable; the newer two-limb owner preserves all64bits.
        assert type(value) is int and 0 <= value <= (1 << 53) - 1, 'Unknown/unsafe actual legacy Game10EAD8 word'
        return str(value)
    return flags(state['sourceRunFlags']['raw'])


case, native, browser = load(args.case), load(args.original), load(args.browser)
contract = case['requires']['sourceExpectedActorContract']
stages = case['requires'].get('actorStages', [])
contracts = [stage['sourceExpectedActorContract'] for stage in stages] or [contract]
ordinary_laser_required = any(c['family'] == 7 and c['variant'] == 1 and c['subtype'] == 0 for c in contracts)
controlled_knife_required = any(c['family'] == 8 and c.get('controlledBy329') for c in contracts)
ordinary_knife_required = any(c['family'] == 8 and not c.get('controlledBy329') for c in contracts)
ordinary_child_required = (any(
    stage['sourceExpectedActorContract']['family'] == 8
    and not stage['sourceExpectedActorContract'].get('controlledBy329')
    and any(item['itemId'] == 118 and item['count'] > 0 for item in stage['ownedCounts'])
    for stage in stages) if stages else 118 in case['fixture']['startingItems'])
contact_contract = case['requires'].get('actualNpcContact')
ordinary_contact_required = bool(case['requires'].get('ordinaryKnifeFunctional', {}).get('actualNpcContactDamageQueue'))
dip_contact_required = ordinary_contact_required or isinstance(contact_contract, dict) and contact_contract.get('kind') == 217
for stage in stages:
    assert isinstance(stage['requiredWitnesses'], list)
    assert all(isinstance(witness, str) for witness in stage['requiredWitnesses'])
checks = []
coverage = collections.Counter()
paired_rows = []
damage_chains = {}
common_damage_chains = {}
doorway_witnesses = []
laser_lifetimes = {}
render_witnesses = []


def reference(state, ref):
    """Join a real port handle through its actual scheduler identity."""
    if ref is None:
        return None
    identities = [i for i in state['actorScheduler']['identities'] if i['actor'] == ref]
    assert len(identities) == 1, 'Referenced actor must have one actual scheduler identity'
    if ref['kind'] == 'player':
        assert ref['id'] == 0
        seed = state['playerItems']['entitySpawnSeed']
        assert type(seed) is int and 0 <= seed <= 0xffffffff, 'Actual PlayerItems Entity+7D0 seed owner required'
        return {'kind': 1, 'variant': 0, 'subtype': 0, 'sequenceId': identities[0]['sequenceId'],
                'spawnSeed': seed}
    arrays = {'knife': 'knives', 'laser': 'lasers', 'tear': 'tears', 'npc': None, 'effect': None}
    assert ref['kind'] in arrays, 'Attack/parent reference has an unbound real actor kind'
    actors = (state['room']['npcs'] if ref['kind'] == 'npc' else
              [a['impact'] for a in state['lasers'] if a.get('impact') is not None] if ref['kind'] == 'effect' else state[arrays[ref['kind']]])
    matches = [a for a in actors if a['id'] == ref['id']]
    assert len(matches) == 1, 'Actual referenced actor unresolved; no NULL coercion'
    actor = matches[0]
    kind = {'knife': 8, 'laser': 7, 'tear': 2, 'effect': 1000}.get(ref['kind'], actor.get('kind'))
    # The nested Impact is the source-scoped real50/1 factory. Regular Tear
    # variant is an actual stored owner; missing it cannot silently become0.
    variant = actor['sourceVariant'] if ref['kind'] == 'tear' else 50 if ref['kind'] == 'effect' else actor['variant']
    subtype = 1 if ref['kind'] == 'effect' else 0 if ref['kind'] == 'tear' else actor['subtype']
    return {'kind': kind, 'variant': variant, 'subtype': subtype,
            'sequenceId': identities[0]['sequenceId'], 'spawnSeed': actor['spawnSeed']}


def identity_tuple(identity):
    return tuple(identity[key] for key in ('kind', 'variant', 'subtype', 'sequenceId', 'spawnSeed'))


def count_total(words):
    return words['countablePending'] + words['countableCurrent']


# Body correspondence uses actual C18 roles and SAME-GATE births, never fitted
# source IDs/seeds/times. Their independent equality checks remain required.
child_pairs, native_child_births, port_child_births = {}, {}, {}
child_lifetimes = []
parent_births = set()
charge_rows = []
charge_witnesses = []

def check_field(field, expected, actual, presentation):
    """Evaluate each captured owner independently; missing never means equal."""
    try:
        check(field, expected(), actual(), presentation)
    except Exception as error:
        missing(field, presentation, error)


def knife_body(original, actual, state, t, role):
    ref = {'kind': 'knife', 'id': actual['id']}
    check_field(role + ' own-manager membership', lambda: original['inActiveManager'], lambda: ref in state['actorScheduler']['actors'], t)
    check_field(role + ' actual identity/seed', lambda: identity_tuple(original['identity']), lambda: identity_tuple(reference(state, ref)), t)
    for key in ('active', 'dead', 'visible', 'gridCollisionClass', 'entityCollisionClass', 'raw494'):
        check_field(role + ' ' + key, lambda: original[key], lambda: actual[key], t)
    for key in ('collisionDamage', 'collisionRadius', 'collisionMass', 'hp', 'maxHp'):
        check_field(role + ' ' + key, lambda: f32(original[key]), lambda: f32(actual[key]), t)
    for sk, pk in [('position', 'position'), ('savedPosition450', 'savedPosition'), ('velocity', 'velocity'), ('renderOffset', 'renderOffset')]:
        check_field(role + ' ' + sk, lambda: vector(original[sk]), lambda: vector(actual[pk]), t)
    check_field(role + ' configured friction', lambda: f32(original['configuredFriction46c']), lambda: f32(actual['configuredFriction']), t)
    check_field(role + ' current friction', lambda: f32(original['currentFriction470']), lambda: f32(actual['currentFriction']), t)
    check_field(role + ' birth age', lambda: original['bornGameAge'], lambda: actual['birthGameUpdate'], t)
    check_field(role + ' fast322', lambda: original['fast322'], lambda: actual['fast'], t)
    check_field(role + ' generic flags318', lambda: original['genericFlags318'], lambda: actual['entityFlags'], t)
    check_field(role + ' attack flags7d8', lambda: original['attackFlags7d8'], lambda: flags(actual['attackFlags']), t)
    check_field(role + ' source RNG7c0', lambda: original['actorRngWords7c0'], lambda: [actual['actorRng'][k] for k in ('state', 'rightA', 'left', 'rightB')], t)
    check_field(role + ' cached spawner', lambda: (original['cachedSpawner']['kind'], original['cachedSpawner']['variant']), lambda: (actual['spawnerType'], actual['spawnerVariant']), t)
    check_field(role + ' permanent color3a0', lambda: original['permanentColor3a0'], lambda: color(actual['sourceColor']), t)
    check_field(role + ' current color2d0', lambda: original['currentColor2d0'], lambda: color(actual['currentColor']), t)
    try:
        parent = reference(state, actual['parent'])
        check_field(role + ' linked Parent NULL', lambda: original['linkedParent'] is None, lambda: parent is None, t)
        if original['linkedParent'] and parent:
            fields = ('kind', 'variant', 'subtype', 'sequenceId')
            check_field(role + ' linked Parent identity', lambda: tuple((original['linkedParent'][k] for k in fields)), lambda: tuple((parent[k] for k in fields)), t)
            if parent['kind'] != 1:
                check_field(role + ' linked Parent seed', lambda: original['linkedParent']['spawnSeed'], lambda: parent['spawnSeed'], t)
    except Exception as error:
        missing(role + ' linked Parent owner', t, error)
    try:
        for offset, value in original['knifeRawFloats'].items():
            check_field(role + ' raw' + offset, lambda: f32(value), lambda: f32(actual['raw' + offset]), t)
    except Exception as error:
        missing(role + ' raw float owners', t, error)
    try:
        for offset, value in original['knifeRawVectors'].items():
            check_field(role + ' raw' + offset, lambda: vector(value), lambda: vector(actual['raw' + offset]), t)
    except Exception as error:
        missing(role + ' raw vector owners', t, error)
    check_field(role + ' raw814', lambda: original['knifeRaw814'], lambda: actual['raw814'], t)
    check_field(role + ' raw8a0 u32', lambda: original['knifeRaw8a0'], lambda: actual['raw8a0'], t)
    check_field(role + ' Init uniform8a8', lambda: f32(original['initUniform8a8']), lambda: f32(actual['raw8a8']), t)
    for key in ('animation', 'frame', 'playing', 'rotation'):
        check_field(role + ' Sprite ' + key, lambda: f32(original['sprite'][key]) if type(original['sprite'][key]) is float else original['sprite'][key], lambda: f32(actual['sprite'][key]) if type(original['sprite'][key]) is float else actual['sprite'][key], t)
    for key in ('scale', 'offset'):
        check_field(role + ' Sprite ' + key, lambda: vector(original['sprite'][key]), lambda: vector(actual['sprite'][key]), t)
    for key in ('origin', 'offset'):
        check_field(role + ' Line ' + key, lambda: vector(original['knifeLine'][key]), lambda: vector(actual['line'][key]), t)
    for key in ('points', 'priorPoints', 'history'):
        check_field(role + ' full Line ' + key, lambda: [vector(v) for v in original['knifeLine'][key]], lambda: [vector(v) for v in actual['line'][key]], t)
    check_field(role + ' full Line controls', lambda: [[f32(x) for x in v] for v in original['knifeLine']['controls']], lambda: [vector(v['position']) + vector(v['incoming']) + vector(v['outgoing']) for v in actual['line']['controls']], t)
    try:
        if original['knifeLine']['sourceOrdinaryCurveWriterScope']:
            for key in ('subdivisions', 'mode'):
                check_field(role + ' initialized Line ' + key, lambda: original['knifeLine'][key], lambda: actual['line'][key], t)
            check_field(role + ' initialized Line width', lambda: f32(original['knifeLine']['width']), lambda: f32(actual['line']['width']), t)
        else:
            for key in ('subdivisions', 'mode', 'width'):
                check_field(role + ' unowned Line ' + key + ' stays unknown', lambda: None, lambda: actual['line'][key], t)
    except Exception as error:
        missing(role + ' Line writer scope', t, error)
    check_field(role + ' unowned Line length stays unknown', lambda: None, lambda: actual['line']['length'], t)
    # Coverage counts observed actors; every unavailable field still fails.
    coverage[role + 'Bodies'] += 1
    try:
        coverage[role + ('FullBodies' if not original['fast322'] else 'FastBodies')] += 1
    except Exception as error:
        missing(role + ' body phase owner', t, error)


def registered_knife_body(original, actual, state, t, controlled, native_counter):
    for key in ('active', 'dead', 'visible'):
        check_field('actual Knife ' + key, lambda: original[key], lambda: actual[key], t)
    check_field('actual Knife collisionDamage', lambda: f32(original['collisionDamage']), lambda: f32(actual['collisionDamage']), t)
    for key in ('collisionRadius', 'collisionMass', 'hp', 'maxHp'):
        check_field('actual Knife ' + key, lambda: f32(original[key]), lambda: f32(actual[key]), t)
    for key in ('gridCollisionClass', 'entityCollisionClass', 'raw494'):
        check_field('actual Knife ' + key, lambda: original[key], lambda: actual[key], t)
    check_field('actual Knife generic flags318', lambda: original['genericFlags318'], lambda: actual['entityFlags'], t)
    check_field('actual Knife cached spawner', lambda: (original['cachedSpawner']['kind'], original['cachedSpawner']['variant']), lambda: (actual['spawnerType'], actual['spawnerVariant']), t)
    try:
        parent = reference(state, actual['parent'])
        check_field('actual Knife linked Parent null/ref', lambda: original['linkedParent'] is None, lambda: parent is None, t)
        if original['linkedParent'] and parent:
            parent_fields = ('kind', 'variant', 'subtype', 'sequenceId')
            check_field('actual Knife Parent kind/variant/subtype/sequence', lambda: tuple((original['linkedParent'][key] for key in parent_fields)), lambda: tuple((parent[key] for key in parent_fields)), t)
    except Exception as error:
        missing('actual Knife linked Parent owner', t, error)
    check_field('actual Knife current RNG words7c0', lambda: original['actorRngWords7c0'], lambda: [actual['actorRng'][key] for key in ('state', 'rightA', 'left', 'rightB')], t)
    check_field('actual Knife current friction470', lambda: f32(original['currentFriction470']), lambda: f32(actual['currentFriction']), t)
    check_field('actual Knife birth age', lambda: original['bornGameAge'], lambda: actual['birthGameUpdate'], t)
    check_field('actual Knife physical448', lambda: vector(original['position']), lambda: vector(actual['position']), t)
    check_field('actual Knife saved450', lambda: vector(original['savedPosition450']), lambda: vector(actual['savedPosition']), t)
    check_field('actual Knife velocity464', lambda: vector(original['velocity']), lambda: vector(actual['velocity']), t)
    check_field('actual Knife fast322', lambda: original['fast322'], lambda: actual['fast'], t)
    check_field('actual Knife flags7D8', lambda: original['attackFlags7d8'], lambda: flags(actual['attackFlags']), t)
    check_field('actual Knife permanent color3A0', lambda: original['permanentColor3a0'], lambda: color(actual['sourceColor']), t)
    check_field('actual Knife current color2D0', lambda: original['currentColor2d0'], lambda: color(actual['currentColor']), t)
    check_field('actual Knife Init uniform8A8', lambda: f32(original['initUniform8a8']), lambda: f32(actual['raw8a8']), t)
    try:
        for offset, value in original['knifeRawFloats'].items():
            check_field('actual Knife raw' + offset, lambda: f32(value), lambda: f32(actual['raw' + offset]), t)
    except Exception as error:
        missing('actual Knife raw float owners', t, error)
    try:
        for offset, value in original['knifeRawVectors'].items():
            check_field('actual Knife raw' + offset, lambda: vector(value), lambda: vector(actual['raw' + offset]), t)
    except Exception as error:
        missing('actual Knife raw vector owners', t, error)
    check_field('actual Knife raw814', lambda: original['knifeRaw814'], lambda: actual['raw814'], t)
    check_field('actual Knife raw8a0 curve index', lambda: original['knifeRaw8a0'], lambda: actual['raw8a0'], t)
    check_field('actual Knife render offset458', lambda: vector(original['renderOffset']), lambda: vector(actual['renderOffset']), t)
    try:
        if original['active'] and controlled:
            coverage['controlledKnifeFullBodies' if native_counter % 2 == 1 else 'controlledKnifeFastBodies'] += 1
    except Exception as error:
        missing('actual Knife controlled phase owner', t, error)
    for source_key, port_key in [('animation', 'animation'), ('frame', 'frame'), ('playing', 'playing'), ('rotation', 'rotation')]:
        check_field('actual Knife Sprite ' + source_key, lambda: f32(original['sprite'][source_key]) if source_key == 'rotation' else original['sprite'][source_key], lambda: f32(actual['sprite'][port_key]) if source_key == 'rotation' else actual['sprite'][port_key], t)
    for key in ('scale', 'offset'):
        check_field('actual Knife Sprite ' + key, lambda: vector(original['sprite'][key]), lambda: vector(actual['sprite'][key]), t)

def ordinary_knives(source,state,t,native_counter):
    slot=source['players'][0]['actorSlots']['c18'];port_ref=state['attackRuntime']['persistent']
    native_matches=[a for a in source['attacks'] if slot and a['identity']==slot['identity']]
    port_matches=[a for a in state['knives'] if port_ref and a['id']==port_ref['id']]
    if slot:
        assert identity_tuple(slot['identity'])[:3]==(8,0,0),'Actual native C18 must own ordinary8/0/0'
        assert len(native_matches)==1,'Actual native C18 must resolve exactly once'
    if port_ref:
        assert port_ref['kind']=='knife','Actual port C18 must own Knife'
        assert len(port_matches)==1 and (port_matches[0]['entityType'],port_matches[0]['variant'],port_matches[0]['subtype'])==(8,0,0),'Actual port C18 must resolve exactly one ordinary8/0/0'
    parent_native=native_matches[0] if slot else None
    parent_port=port_matches[0] if port_ref else None
    if parent_native and parent_port:
        knife_body(parent_native,parent_port,state,t,'ordinaryParent')
        key=identity_tuple(parent_native['identity'])
        if key not in parent_births:
            parent_births.add(key);coverage['ordinaryParentBirths']+=1
        try:
            if parent_native['knifeRaw814'] and parent_port['raw814']:coverage['ordinaryParentLaunchedRows']+=1
        except Exception as error:
            missing('ordinaryParent launch owner', t, error)
    launch = {}
    for label, actor, key in [('nativeLaunched', parent_native, 'knifeRaw814'), ('portLaunched', parent_port, 'raw814')]:
        if actor is None:
            launch[label] = None
        else:
            try:
                launch[label] = actor[key]
            except Exception as error:
                missing('ordinary charge ' + label + ' owner', t, error)
                launch[label] = {'unavailable': True}
    native_charge=source['players'][0]['chargeC0c'];port_charge=state['attackRuntime']['chargeC0c']
    charge_row={'presentation':t,'nativeCounter':native_counter,'portCounter':state['systemClock']['systemCounter'],
        'nativeCharge':native_charge,'portCharge':port_charge,'nativeLaunched':launch['nativeLaunched'],
        'portLaunched':launch['portLaunched']}
    if charge_rows:
        previous=charge_rows[-1]
        if native_counter%2==state['systemClock']['systemCounter']%2==1 and native_charge==port_charge and native_charge is not None and previous['nativeCharge']==previous['portCharge'] and previous['nativeCharge'] is not None and native_charge>previous['nativeCharge']:
            charge_witnesses.append({'kind':'observedFullChargeIncrease','before':previous,'after':charge_row});coverage['ordinaryChargeIncreases']+=1
        if previous['nativeCharge'] is not None and previous['portCharge'] is not None and previous['nativeCharge']>0 and previous['portCharge']>0 and native_charge==port_charge==0 and charge_row['nativeLaunched'] is True and charge_row['portLaunched'] is True:
            charge_witnesses.append({'kind':'observedChargeResetWithLaunched814','before':previous,'after':charge_row});coverage['ordinaryChargeReleaseResets']+=1
    charge_rows.append(charge_row)
    ns=sorted([a for a in source['attacks'] if a['identity']['kind']==8 and a['identity']['subtype']==1],key=lambda a:a['identity']['sequenceId'])
    ps=sorted([a for a in state['knives'] if a['subtype']==1],key=lambda a:reference(state,{'kind':'knife','id':a['id']})['sequenceId'])
    nb=[a for a in ns if identity_tuple(a['identity']) not in native_child_births]
    pb=[a for a in ps if a['id'] not in port_child_births]
    for a in nb:native_child_births[identity_tuple(a['identity'])]=t
    for a in pb:port_child_births[a['id']]=t
    check('same-gate actual child births count',len(nb),len(pb),t)
    if len(nb)==len(pb):
        for n,p in zip(nb,pb):
            nk=identity_tuple(n['identity']);child_pairs[nk]=p['id']
            life={'nativeIdentity':n['identity'],'portId':p['id'],'nativeParentAtBirth':n['linkedParent'],'portParentAtBirth':p['parent'],'birthPresentation':t,'joinBasis':'same actual gate/type/subtype birth order; identity and seed independently graded','copyRelationSourceScope':'Selected neutral473BCE line rebuild then473DB1 Raw/Init, deep copy before447FC0 and explicit18. Actual subtype1 body473F00 reads controls/opaque copied owners without modifying Line; permanent color remains the v58 input. Source peer038fa7/finishFull3e857 scope excludes general timed-color/status histories. Both complete cross-game bodies independently graded.','rows':[],'retirement':None}
            child_lifetimes.append(life);coverage['ordinaryChildPairedBirths']+=1
            check('native child actual cached spawner Knife',8,n['cachedSpawner']['kind'],t)
            check('port child actual cached spawner Knife',8,p['spawnerType'],t)
            check('native child not registered C18',False,bool(slot and slot['identity']==n['identity']),t)
            check('port child not registered C18',False,bool(port_ref=={'kind':'knife','id':p['id']}),t)
            check('child birth actual Parent is persistent Knife on both sides',(True,True),(bool(n['linkedParent'] and slot and n['linkedParent']==slot['identity']),bool(port_ref and p['parent']==port_ref)),t)
            if parent_native and parent_port:
                check('actual child stored48c is Parent stored48c times .3',
                    (f32(f32(parent_native['collisionDamage'])*f32(.3)),f32(f32(parent_port['collisionDamage'])*f32(.3))),
                    (f32(n['collisionDamage']),f32(p['collisionDamage'])),t)
                check('actual child copied current Parent color',parent_native['currentColor2d0'],n['permanentColor3a0'],t)
                check('port child copied current Parent color',color(parent_port['currentColor']),color(p['sourceColor']),t)
                for field in ('controls','points','priorPoints','history','origin','offset','opaque18RawWords','lengthRawBits','widthRawBits','subdivisionsRawBits','modeRawBits'):
                    check('source actual child deep Line copy '+field,parent_native['knifeLine'][field],n['knifeLine'][field],t)
                for field in ('controls','points','priorPoints','history','origin','offset','width','subdivisions','mode','length'):
                    check('port actual child deep Line copy '+field,parent_port['line'][field],p['line'][field],t)
                check('port copied opaque Line unknown owner',parent_port['lineRaw18'],p['lineRaw18'],t)
    for n in ns:
        nk=identity_tuple(n['identity']);pid=child_pairs.get(nk);p=next((a for a in ps if a['id']==pid),None)
        if p is None:missing('actual child body correspondence',t,'No same-gate actual child birth correspondence; no fitted identity/time');continue
        knife_body(n,p,state,t,'ordinaryChild')
        life=next(l for l in child_lifetimes if identity_tuple(l['nativeIdentity'])==nk)
        life['rows'].append({'presentation':t,'nativeCounter':native_counter,'portCounter':state['systemClock']['systemCounter'],'nativeActive':n['active'],'nativeDead':n['dead'],'portActive':p['active'],'portDead':p['dead']})
    for life in child_lifetimes:
        if life['retirement'] or t<=life['birthPresentation']:continue
        n=next((a for a in ns if a['identity']==life['nativeIdentity']),None);p=next((a for a in state['knives'] if a['id']==life['portId']),None)
        if n is None and p is None:
            observed_remove=any(not r['nativeActive'] and r['nativeDead'] and not r['portActive'] and r['portDead'] for r in life['rows'])
            life['retirement']={'presentation':t,'fullPhaseBoth':native_counter%2==state['systemClock']['systemCounter']%2==1,'nativeManagerAbsent':True,'portStorageAndScheduleAbsent':not any(a['kind']=='knife' and a['id']==life['portId'] for a in state['actorScheduler']['actors']),'removeFlagsActuallyObserved':observed_remove,'missedShortRemoveBoundary':not observed_remove,'originalParentC18Retained':bool(slot and slot['identity']==life['nativeParentAtBirth'] and port_ref==life['portParentAtBirth']),'nativeWeaponWeakSlotsClear':not any(source['players'][0]['actorSlots'][name] and source['players'][0]['actorSlots'][name]['identity']==life['nativeIdentity'] for name in ('c18','c20','c28','c30')),'nativeObservedParentLinksClear':not any(a['linkedParent']==life['nativeIdentity'] for a in source['attacks']),'weakReferencesClear':not any(ref=={'kind':'knife','id':life['portId']} for ref in [state['attackRuntime']['persistent'],*state['attackRuntime']['secondary'],*[a['parent'] for a in state['knives']],*[a['homingTarget'] for a in state['knives']]]),'scope':'Absent native object has no invented dead/Uninit field; short Remove boundary remains a required missing witness.'}


def compare_lasers(source, state, t, native_counter):
    for original in [actor for actor in source['attacks'] if actor['identity']['kind'] == 7]:
        identity = original['identity']
        slot = source['players'][0]['actorSlots']['c18']
        if slot is None or slot['identity'] != identity:
            missing('actual nonprimary Laser body source correspondence', t, 'No declared ordinary-primary C18 relation; identity/family mismatch remains retained')
            continue
        ref = state['attackRuntime']['persistent']
        assert ref is not None and ref['kind'] == 'laser', 'Actual port primary C18 must reference Laser'
        assert identity['kind'] == 7 and identity['variant'] == 1 and identity['subtype'] == 0
        assert identity_tuple(reference(state, ref))[:3] == (7, 1, 0), 'Own source family must be actual ordinary7/1/0'
        assert len([item for item in source['attacks'] if item['identity'] == slot['identity']]) == 1, 'Own native C18 must resolve uniquely'
        matches = [actor for actor in state['lasers'] if actor['id'] == ref['id']]
        assert len(matches) == 1, 'Referenced Laser unresolved; no NULL coercion'
        actual = matches[0]
        check_field('actual Laser identity family/variant/subtype/sequence/seed', lambda: identity_tuple(identity), lambda: identity_tuple(reference(state, ref)), t)
        check_field('actual Laser own-manager membership', lambda: original['inActiveManager'], lambda: ref in state['actorScheduler']['actors'], t)
        for key in ('active', 'dead', 'visible', 'hp', 'maxHp', 'collisionRadius', 'collisionMass', 'collisionDamage',
                    'gridCollisionClass', 'entityCollisionClass', 'raw494'):
            if key in ('hp', 'maxHp'):
                check_field('actual Laser ' + key + ' Native numeric owner',
                            lambda: True, lambda: math.isfinite(f32(original[key])), t)
                check_field('actual Laser ' + key + ' port numeric owner',
                            lambda: True, lambda: math.isfinite(f32(actual[key])), t)
            check_field('actual Laser ' + key,
                        lambda: f32(original[key]) if key in ('hp', 'maxHp') or type(original[key]) is float else original[key],
                        lambda: f32(actual[key]) if key in ('hp', 'maxHp') or type(original[key]) is float else actual[key], t)
        for source_key, port_key in [('position', 'position'), ('savedPosition450', 'savedPosition'),
                                      ('velocity', 'velocity'), ('renderOffset', 'renderOffset')]:
            check_field('actual Laser ' + source_key, lambda: vector(original[source_key]), lambda: vector(actual[port_key]), t)
        check_field('actual Laser born age', lambda: original['bornGameAge'], lambda: actual['birthGameUpdate'], t)
        check_field('actual Laser fast322', lambda: original['fast322'], lambda: actual['fast'], t)
        check_field('actual Laser generic flags318', lambda: original['genericFlags318'], lambda: actual['entityFlags'], t)
        for source_key, port_key in [('currentFriction470', 'currentFriction'), ('configuredFriction46c', 'configuredFriction')]:
            check_field('actual Laser ' + source_key, lambda: f32(original[source_key]), lambda: f32(actual[port_key]), t)
        check_field('actual Laser current RNG words', lambda: original['actorRngWords7c0'],
              lambda: [actual['actorRng'][key] for key in ('state', 'rightA', 'left', 'rightB')], t)
        check_field('actual Laser permanent3a0 color', lambda: original['permanentColor3a0'], lambda: color(actual['sourceColor']), t)
        check_field('actual Laser current2d0 color', lambda: original['currentColor2d0'], lambda: color(actual['currentColor']), t)
        check_field('actual Laser cached spawner', lambda: (original['cachedSpawner']['kind'], original['cachedSpawner']['variant']),
              lambda: (actual['spawnerType'], actual['spawnerVariant']), t)
        for key in ('parent', 'child'):
            try:
                source_ref = original['linkedParent'] if key == 'parent' else original['laser']['child']
                port_ref = reference(state, actual[key])
                check_field('actual Laser ' + key + ' null/reference', lambda: source_ref is None, lambda: port_ref is None, t)
                if source_ref and port_ref:
                    fields = ('kind', 'variant', 'subtype', 'sequenceId')
                    check_field('actual Laser ' + key + ' source identity', lambda: tuple(source_ref[k] for k in fields), lambda: tuple(port_ref[k] for k in fields), t)
            except Exception as error:
                missing('actual Laser '+key+' reference group',t,error)
        for key in ('parentOffset', 'endpoint'):
            check_field('actual Laser ' + key, lambda: vector(original['laser'][key]), lambda: vector(actual[key]), t)
        for key in ('angle', 'previousAngle', 'renderSortOffset', 'beamLength', 'maxDistance'):
            check_field('actual Laser ' + key, lambda: f32(original['laser'][key]), lambda: f32(actual[key]), t)
        for key in ('remainingUpdates', 'firstUpdate', 'lineEnabled', 'contactDisabled', 'rotationEnabled', 'lineController'):
            check_field('actual Laser ' + key, lambda: original['laser'][key], lambda: actual[key], t)
        check_field('actual Laser attack flags7e8', lambda: original['laser']['attackFlags'], lambda: flags(actual['attackFlags']), t)
        check_field('actual Laser copied line flags818', lambda: original['laser']['lineFlags'], lambda: flags(actual['lineFlags']), t)
        try:
            check_field('source ordinary ParentPlayer Laser Line writer scope', lambda: True, lambda: original['laser']['line']['sourceOrdinaryLineWriterScope'], t)
            for key in ('origin', 'offset'):
                check_field('actual Laser line ' + key, lambda: vector(original['laser']['line'][key]), lambda: vector(actual['line'][key]), t)
            for key in ('subdivisions', 'mode'):
                check_field('actual Laser line ' + key, lambda: original['laser']['line'][key], lambda: actual['line'][key], t)
            check_field('actual Laser line length86c', lambda: f32(original['laser']['line']['length']), lambda: f32(actual['line']['length']), t)
            check_field('actual ordinary initialized Laser width810', lambda: f32(original['laser']['line']['width']), lambda: f32(actual['line']['width']), t)
            for key in ('points', 'priorPoints', 'history'):
                check_field('actual Laser complete line ' + key, lambda: [vector(v) for v in original['laser']['line'][key]],
                      lambda: [vector(v) for v in actual['line'][key]], t)
            controls = [[f32(v) for v in values] for values in original['laser']['line']['controls']]
            port_controls = [vector(value['position']) + vector(value['incoming']) + vector(value['outgoing']) for value in actual['line']['controls']]
            check_field('actual Laser complete line controls870', lambda: controls, lambda: port_controls, t)
        except Exception as error:
            missing('mandatory source-typed ordinary Line body group',t,error)
        for key in ('animation', 'frame', 'playing', 'rotation'):
            check_field('actual Laser Sprite ' + key,
                        lambda: f32(original['sprite'][key]) if type(original['sprite'][key]) is float else original['sprite'][key],
                        lambda: f32(actual['sprite'][key]) if type(original['sprite'][key]) is float else actual['sprite'][key], t)
        for key in ('scale', 'offset'):
            check_field('actual Laser Sprite ' + key, lambda: vector(original['sprite'][key]), lambda: vector(actual['sprite'][key]), t)
        try:
            compare_impact(source, original, actual, state, t, native_counter)
        except Exception as error:
            missing('mandatory actual Impact body group', t, error)
        key = tuple(identity[k] for k in ('sequenceId','spawnSeed'))
        laser_lifetimes.setdefault(key, {'nativeIdentity':identity,'portRef':ref,'rows':[]})['rows'].append({
            'presentation':t,'nativeCounter':native_counter,
            'portCounter':state['systemClock']['systemCounter'],'nativeRemaining':original['laser']['remainingUpdates'],
            'portRemaining':actual['remainingUpdates'],'nativeActive':original['active'],'portActive':actual['active'],
            'nativeDead':original['dead'],'portDead':actual['dead'],'portUninitialized':actual['uninitialized']})
        coverage['actualOrdinaryLaserBodies'] += 1
        coverage['ordinaryLaserFullBodies' if native_counter % 2 == 1 else 'ordinaryLaserFastBodies'] += 1


def compare_impact(source, original_laser, actual_laser, state, t, native_counter):
    source_ref = original_laser['laser']['child']
    port_ref = actual_laser['child']
    check_field('actual ordinary Impact child relation present', lambda: source_ref is None, lambda: port_ref is None, t)
    if source_ref is None:
        return
    assert tuple(source_ref[k] for k in ('kind','variant','subtype')) == (1000,50,1)
    source_rows = [a for a in source['impacts'] if a['identity'] == source_ref]
    assert len(source_rows) == 1, 'Own source Laser child must join one actual Impact50 body'
    original = source_rows[0]
    actual = actual_laser['impact']
    assert actual is not None and port_ref is not None and port_ref['kind'] == 'effect' and port_ref['id'] == actual['id']
    identity = reference(state, port_ref)
    check_field('actual Impact identity/seed', lambda: identity_tuple(source_ref), lambda: identity_tuple(identity), t)
    check_field('actual Impact dead31f', lambda: original['dead'], lambda: actual['dead'], t)
    check_field('actual Impact manager membership', lambda: original['inActiveManager'], lambda: port_ref in state['actorScheduler']['actors'], t)
    check_field('actual Impact Parent NULL relation', lambda: original['linkedParent'] is None, lambda: actual['parent'] is None, t)
    if original['linkedParent'] is not None and actual['parent'] is not None:
        check_field('actual Impact Parent source identity', lambda: identity_tuple(original['linkedParent']), lambda: identity_tuple(reference(state,actual['parent'])), t)
    for source_key,port_key in [('position','position'),('renderOffset','positionOffset')]:
        check_field('actual Impact '+source_key,lambda: vector(original[source_key]),lambda: vector(actual[port_key]),t)
    check_field('actual Impact Sprite scale',lambda: vector(original['sprite']['scale']),lambda: vector(actual['scale']),t)
    check_field('actual Impact Sprite rotation',lambda: f32(original['sprite']['rotation']),lambda: f32(actual['rotation']),t)
    check_field('actual Impact Sprite clip',lambda: original['sprite']['animation'],lambda: actual['animation'],t)
    check_field('actual Impact Sprite frame',lambda: f32(original['sprite']['frame']),lambda: f32(actual['animationClock']),t)
    check_field('actual Impact Sprite playing',lambda: original['sprite']['playing'],lambda: actual['animationPlaying'],t)
    check_field('actual Impact depth7f8',lambda: f32(original['impact']['renderSortOffset']),lambda: f32(actual['depth']),t)
    check_field('actual Impact permanent color3a0',lambda: original['permanentColor3a0'],lambda: color(actual['sourceColor']),t)
    check_field('actual Impact current color2d0',lambda: original['currentColor2d0'],lambda: color(actual['currentColor']),t)
    coverage['actualImpactBodies'] += 1
    coverage['impactFullBodies' if native_counter % 2 == 1 else 'impactFastBodies'] += 1


def compare_causal_npcs(source, world, state, t, native_counter):
    """One actual target's FIFO -> drain -> dead/active -> next full absence."""
    native_targets = source['contactNpcs']
    port_targets = [actor for actor in state['room']['npcs'] if actor['kind'] == 217 and not actor['removed']]
    native_ids = sorted((a['identity']['variant'], a['identity']['subtype'], a['identity']['sequenceId'], a['identity']['spawnSeed']) for a in native_targets)
    port_ids = sorted((a['variant'], a['subtype'], reference(state, {'kind': 'npc', 'id': a['id']})['sequenceId'], a['spawnSeed']) for a in port_targets)
    check('actual contact NPC manager identities', native_ids, port_ids, t)
    check('actual manager count words', source['managerCounts'], state['room']['managerCounts'], t)
    descriptor = state['room']['descriptor']
    check('actual room descriptor identity',
          (source['roomDescriptor']['index'], source['roomDescriptor']['gridIndex'], source['roomDescriptor']['spawnSeed']),
          (descriptor['index'], descriptor['gridIndex'], descriptor['spawnSeed']), t)
    check('actual descriptor combat clear', source['roomDescriptor']['combatClear'], state['room']['combatClear'], t)
    check('actual combat clear countdown', source['combatClearCountdown'], state['room']['combatClearCountdown'], t)
    port_counter = state['systemClock']['systemCounter']
    for original in native_targets:
        identity = original['identity']
        key = (source['roomDescriptor']['gridIndex'], identity['sequenceId'], identity['spawnSeed'])
        matches = [a for a in port_targets if reference(state, {'kind': 'npc', 'id': a['id']})['sequenceId'] == identity['sequenceId']
                   and a['variant'] == identity['variant'] and a['subtype'] == identity['subtype']]
        if len(matches) != 1:
            missing('actual contact target body', t, 'No unique source-sequence/type/variant/subtype match')
            continue
        actual = matches[0]
        native_body = [actor for actor in world['entities'] if actor['type'] == 217
                       and actor['sequenceId'] == identity['sequenceId'] and actor['spawnSeed'] == identity['spawnSeed']]
        assert len(native_body) == 1, 'Actual Dip must join its complete same-gate world body'
        check('actual starting Dip state/init and first callbacks', native_body[0]['npcState'], actual['state'], t)
        check('actual starting Dip frame age/init and first callbacks', native_body[0]['npcFrameAge'], actual['frameAge'], t)
        port_identity = reference(state, {'kind': 'npc', 'id': actual['id']})
        check('actual contact target birth seed', identity['spawnSeed'], actual['spawnSeed'], t)
        for key_field in ('hp', 'maxHp'):
            check('actual contact target ' + key_field, f32(original[key_field]), f32(actual[key_field]), t)
        for key_field in ('active', 'dead', 'visible', 'canShutDoors', 'boss'):
            check('actual contact target ' + key_field, original[key_field], actual[key_field], t)
        for field in ('entityCollisionClass','raw494'):
            check('actual217 contact eligibility '+field,original[field],actual[field],t)
        port_flags=actual['dip']['entityFlags']
        check('actual217 contact eligibility flags318',original['entityFlags'],port_flags,t)
        native_eligible=(10<=identity['kind']<=999 and original['entityCollisionClass'] not in (0,1) and not original['raw494'] and not original['dead'] and original['entityFlags']&0x10==0)
        port_eligible=(10<=actual['kind']<=999 and actual['entityCollisionClass'] not in (0,1) and not actual['raw494'] and not actual['dead'] and port_flags&0x10==0)
        check('actual444810 receiver eligibility',native_eligible,port_eligible,t)
        check('actual217 presented physical position',vector(original['position']),vector(actual['presentedPosition']),t)
        check('actual217 physical velocity',vector(original['velocity']),vector(actual['velocity']),t)
        check('actual217 collision radius',f32(original['collisionRadius']),f32(actual['collisionRadius']),t)
        check('actual contact target retired membership', False, actual['removed'], t)
        check('actual ordered pending damage total', f32(original['pendingDamage']), f32(actual['damage']['pendingDamage']), t)
        check('actual ordered pending damage flags', original['pendingFlags'], actual['damage']['pendingFlags'], t)
        native_records = [[f32(r['amount']), r['flags'], r['sourceType'], r['sourceVariant'],
                           identity_tuple(r['actualAttacker']) if r['actualAttacker'] else None]
                          for r in original['pendingDamageRecords']]
        port_records = []
        for event in actual['damage']['events']:
            kind = {2: 'tear', 8: 'knife', 7: 'laser', 1: 'player'}.get(event['source']['entityType'])
            assert kind is not None, 'Contact case source must join its actual attacker'
            attacker = reference(state, {'kind': kind, 'id': event['source']['id']})
            event_flags = {'tear': 0, 'knife': 0, 'laser': 8, 'bomb': 4, 'fireContact': 2, 'roomEffect': 0}[event['kind']]
            port_records.append([f32(event['amount']), event_flags, event['source']['entityType'], attacker['variant'], identity_tuple(attacker)])
        check('actual contact ordered FIFO and attacker identities', native_records, port_records, t)
        chain = damage_chains.setdefault(key, {'identity': identity, 'queued': [], 'drain': None,
            'deadActive': None, 'appliedFifo': [], 'intermediate': [], 'retirement': None, 'missedBoundary': False})
        row = {'presentation': t, 'nativeCounter': native_counter, 'portCounter': port_counter,
               'nativeHp': f32(original['hp']), 'portHp': f32(actual['hp']),
               'nativeCounts': source['managerCounts'], 'portCounts': state['room']['managerCounts'],
               'sameTargetIdentity': identity_tuple(identity) == identity_tuple(port_identity),
               'nativeRecords': native_records, 'portRecords': port_records,
               'actualChildRecord':any(r[4] is not None and r[4][:3]==(8,0,1) for r in native_records),
               'actualParentRecord':any(r[4] is not None and r[4][:3]==(8,0,0) for r in native_records),
               'nativeActive': original['active'], 'portActive': actual['active'],
               'nativeDead': original['dead'], 'portDead': actual['dead'], 'canShutDoors': original['canShutDoors'],
               'nativeNpcManagerIdentities': source['npcManagerIdentities'],
               'portNpcManagerIdentities': [identity_tuple(reference(state, {'kind': 'npc', 'id': actor['id']}))
                                           for actor in state['room']['npcs'] if not actor['removed']]}
        # Grade the previously captured FIFO at its next real full callback.
        # New contacts may already have enqueued another FIFO at this GL gate;
        # a nonlethal application does not require the current queue to be empty.
        previous=next((q for q in reversed(chain['queued'])
            if native_counter==q['nativeCounter']+(1 if q['nativeCounter']%2==0 else 2)
            and port_counter==q['portCounter']+(1 if q['portCounter']%2==0 else 2)),None)
        if row['sameTargetIdentity'] and original['pendingDamage'] > 0 and native_records == port_records and any(r[2] == 8 for r in native_records):
            chain['queued'].append(row)
        if previous and native_counter%2==port_counter%2==1:
            expected_native_hp,expected_port_hp=previous['nativeHp'],previous['portHp']
            for native_record,port_record in zip(previous['nativeRecords'],previous['portRecords']):
                assert native_record[1]==port_record[1]==0,'Known217 Knife flags0 common applier only'
                expected_native_hp=f32(expected_native_hp-native_record[0]);expected_port_hp=f32(expected_port_hp-port_record[0])
            check('actual next-full217 sequential captured-FIFO HP application',
                (expected_native_hp,expected_port_hp),(f32(original['hp']),f32(actual['hp'])),t)
            applied=row['sameTargetIdentity'] and f32(original['hp'])==expected_native_hp==expected_port_hp==f32(actual['hp'])
            if applied:
                witness={**row,'queuedWitness':previous,'expectedNativeHpSequentialF32':expected_native_hp,'expectedPortHpSequentialF32':expected_port_hp}
                chain['appliedFifo'].append(witness)
                if original['pendingDamage']==actual['damage']['pendingDamage']==0 and original['hp']<0 and actual['hp']<0:
                    chain['drain']=chain['drain'] or witness
        if chain['drain'] and row['sameTargetIdentity'] and original['dead'] and actual['dead'] and original['active'] and actual['active']:
            chain['deadActive'] = chain['deadActive'] or row
        death = chain['deadActive']
        if death and native_counter == death['nativeCounter'] + 1 and port_counter == death['portCounter'] + 1 and native_counter % 2 == port_counter % 2 == 0:
            if original['dead'] and actual['dead'] and original['active'] and actual['active']:
                chain['intermediate'].append(row)
    for key, chain in damage_chains.items():
        death = chain['deadActive']
        if death is None or chain['retirement'] is not None or chain['missedBoundary']:
            continue
        if source['roomDescriptor']['gridIndex'] != key[0]:
            continue
        if native_counter > death['nativeCounter'] + 2 or port_counter > death['portCounter'] + 2:
            chain['missedBoundary'] = True
            continue
        if native_counter != death['nativeCounter'] + 2 or port_counter != death['portCounter'] + 2:
            continue
        native_absent = all(identity_tuple(a['identity']) != identity_tuple(chain['identity']) for a in native_targets)
        port_absent = all(identity_tuple(reference(state, {'kind': 'npc', 'id': a['id']})) != identity_tuple(chain['identity']) for a in port_targets)
        counts_match = source['managerCounts'] == state['room']['managerCounts'] and death['nativeCounts'] == death['portCounts']
        native_disappeared = set(tuple(i) for i in death['nativeNpcManagerIdentities']) - set(tuple(i) for i in source['npcManagerIdentities'])
        current_port_manager = [identity_tuple(reference(state, {'kind': 'npc', 'id': actor['id']}))
                                for actor in state['room']['npcs'] if not actor['removed']]
        port_disappeared = set(tuple(i) for i in death['portNpcManagerIdentities']) - set(current_port_manager)
        sole_removal = native_disappeared == port_disappeared == {identity_tuple(chain['identity'])}
        decrease = (count_total(death['nativeCounts']) - count_total(source['managerCounts'])
                    == count_total(death['portCounts']) - count_total(state['room']['managerCounts'])
                    == int(death['canShutDoors']))
        chain['retirement'] = {'presentation': t, 'nativeCounter': native_counter, 'portCounter': port_counter,
            'nativeAbsent': native_absent, 'portAbsent': port_absent, 'countsMatch': counts_match, 'exactCountDecrease': decrease,
            'soleTargetRemoval': sole_removal, 'nativeDisappeared': sorted(native_disappeared), 'portDisappeared': sorted(port_disappeared),
            'multiRemovalScope': None if sole_removal else 'Other/missing manager removals observed; actor-attributable count change is not qualified.',
            'passed': native_counter % 2 == port_counter % 2 == 1 and native_absent and port_absent and counts_match and decrease and sole_removal,
            'scope': 'Actual next-full manager absence/counts; inactive store inferred from sole Remove, never an invented absent actor row.'}


def common_contact_spec():
    """Two explicit source payloads; never reinterpret the Dip217 contract."""
    brim = case['requires'].get('ordinaryBrimContact')
    tear = case['requires'].get('controlledTearContact')
    assert not (brim is not None and tear is not None), 'One typed contact producer required'
    if brim is not None:
        receiver = brim['authoredReceiver']
        assert receiver['type'] in (14, 18)
        assert contract['family'] == 7 and contract['variant'] == 1 and contract['subtype'] == 0
        return brim, [receiver['type']], 1, 0, 8
    if tear is not None:
        assert tear['receiverTypes'] and all(type(k) is int and k in (14, 18) for k in tear['receiverTypes'])
        assert (tear['sourceEntityType'], tear['sourceVariant'], tear['damageFlags'], tear['countdown']) == (2, 0, 0, 30)
        assert contract['family'] == 2 and contract['controlledBy329'] is True
        return tear, tear['receiverTypes'], 2, 0, 0
    return None


def contact_source(source, state, spec):
    """Resolve each own C18/source; a Parent payload is not a Laser identity."""
    _, _, source_type, source_variant, _ = spec
    slot = source['players'][0]['actorSlots']['c18']
    assert slot is not None, 'Contact source requires its actual registered C18'
    native_bodies = [a for a in source['attacks'] if a['identity'] == slot['identity']]
    assert len(native_bodies) == 1, 'Native C18 must resolve one real source actor'
    native_body = native_bodies[0]
    ref = state['attackRuntime']['persistent']
    assert ref is not None and ref['kind'] == ('laser' if source_type == 1 else 'tear')
    port_body = [a for a in state['lasers' if source_type == 1 else 'tears'] if a['id'] == ref['id']]
    assert len(port_body) == 1, 'Port C18 must resolve one real source actor'
    if source_type == 1:
        assert identity_tuple(native_body['identity'])[:3] == (7, 1, 0)
        native_owner = native_body['linkedParent']
        assert native_owner == source['players'][0]['owner'], 'Laser damage source must be its actual ParentPlayer'
        port_owner = reference(state, port_body[0]['parent'])
        assert port_owner == reference(state, {'kind': 'player', 'id': 0})
    else:
        native_owner, port_owner = native_body['identity'], reference(state, ref)
        assert int(native_body['attackFlags7f0']) & (1 << 33) and int(flags(port_body[0]['attackFlags'])) & (1 << 33)
    assert identity_tuple(native_owner)[:2] == identity_tuple(port_owner)[:2] == (source_type, source_variant)
    return native_owner, port_owner, f32(native_body['collisionDamage']), f32(port_body[0]['collisionDamage'])


def compare_common_contacts(source, state, t, native_counter, spec):
    """Observe real common NPC FIFO and nonlethal next-Full application.

    This does not infer callback counts, historical EntityRef values, registry
    hits, impulse order, lethal subclass behavior or Remove from later absence.
    """
    declaration, kinds, source_type, source_variant, damage_flags = spec
    native_targets = [a for a in source['commonContactNpcs'] if a['identity']['kind'] in kinds]
    port_targets = [a for a in state['room']['npcs'] if a['kind'] in kinds and not a['removed']]
    check_field('typed contact complete receiver identities',
        lambda: sorted(identity_tuple(a['identity']) for a in native_targets),
        lambda: sorted(identity_tuple(reference(state, {'kind': 'npc', 'id': a['id']})) for a in port_targets), t)
    for original in native_targets:
        identity = original['identity']
        matches = [a for a in port_targets if (a['kind'], a['variant'], a['subtype'],
            reference(state, {'kind': 'npc', 'id': a['id']})['sequenceId']) == identity_tuple(identity)[:4]]
        if len(matches) != 1:
            missing('typed contact receiver correspondence', t, 'No unique actual kind/variant/subtype/source-sequence join')
            continue
        actual = matches[0]
        for field in ('hp', 'maxHp', 'pendingDamage', 'pendingFlags'):
            check_field('typed contact receiver ' + field,
                lambda: f32(original[field]) if field != 'pendingFlags' else original[field],
                lambda: f32(actual['damage'][field] if field.startswith('pending') else actual[field]) if field != 'pendingFlags' else actual['damage'][field], t)
        for field in ('dead', 'visible', 'raw494', 'entityCollisionClass'):
            check_field('typed contact receiver ' + field, lambda: original[field], lambda: actual[field], t)
        check_field('typed contact receiver own-manager membership', lambda: original['inActiveManager'],
            lambda: {'kind': 'npc', 'id': actual['id']} in state['actorScheduler']['actors'], t)
        # Npc.champion serializes Some only; absence is that explicit Option
        # schema, not an assumed authored variant/HP or native champion value.
        check_field('typed contact receiver champion', lambda: original['champion'], lambda: actual.get('champion') is not None, t)
        for field in ('npcState', 'npcFrameAge'):
            check_field('typed contact receiver ' + field, lambda: original[field],
                lambda: actual['state' if field == 'npcState' else 'frameAge'], t)
        for field, port_field in (('position', 'presentedPosition'), ('velocity', 'velocity')):
            check_field('typed contact receiver ' + field, lambda: vector(original[field]), lambda: vector(actual[port_field]), t)
        check_field('typed contact receiver radius', lambda: f32(original['collisionRadius']), lambda: f32(actual['collisionRadius']), t)
        native_records, port_events = original['pendingDamageRecords'], actual['damage']['events']
        check('typed contact ordered FIFO length', len(native_records), len(port_events), t)
        event_flag_values = {'laser': 8, 'tear': 0, 'knife': 0, 'bomb': 4, 'fireContact': 2, 'roomEffect': 0}
        for record, event in zip(native_records, port_events):
            check_field('typed contact FIFO amount', lambda: f32(record['amount']), lambda: f32(event['amount']), t)
            check_field('typed contact FIFO flags', lambda: record['flags'], lambda: event_flag_values[event['kind']], t)
            check_field('typed contact FIFO source type', lambda: record['sourceType'], lambda: event['source']['entityType'], t)
        native_fifo, port_fifo = [], []
        for record in native_records:
            native_fifo.append([f32(record['amount']), record['flags'], record['sourceType'], record['sourceVariant'],
                identity_tuple(record['actualAttacker']) if record['actualAttacker'] is not None else None])
        for index, event in enumerate(port_events):
            try:
                kind = {1: 'player', 2: 'tear', 7: 'laser', 8: 'knife'}[event['source']['entityType']]
                attacker = reference(state, {'kind': kind, 'id': event['source']['id']})
                event_flags = event_flag_values[event['kind']]
                port_fifo.append([f32(event['amount']), event_flags, event['source']['entityType'], attacker['variant'], identity_tuple(attacker)])
            except Exception as error:
                port_fifo.append(None)
                missing('typed contact actual FIFO source ' + str(index), t, error)
        check('typed contact ordered FIFO amount/flags/source identity', native_fifo, port_fifo, t)
        row = {'presentation': t, 'nativeCounter': native_counter, 'portCounter': state['systemClock']['systemCounter'],
            'nativeGameAge': source['gameAge'], 'portGameAge': state['gameUpdateCount'],
            'nativeHp': f32(original['hp']), 'portHp': f32(actual['hp']),
            'nativeFifo': native_fifo, 'portFifo': port_fifo, 'nativeRawRecords': native_records,
            'nativeQueued': False, 'portQueued': False, 'nativeApplied': False, 'portApplied': False}
        try:
            native_owner, port_owner, native_damage, port_damage = contact_source(source, state, spec)
            check('typed contact own source identities', identity_tuple(native_owner), identity_tuple(port_owner), t)
            for side, fifo, owner, amount in (('native', native_fifo, native_owner, native_damage), ('port', port_fifo, port_owner, port_damage)):
                row[side + 'Queued'] = any(r is not None and r == [amount, damage_flags, source_type, source_variant, identity_tuple(owner)] and amount > 0 for r in fifo)
        except Exception as error:
            missing('typed contact actual registered damage producer', t, error)
        # 447729 skips damage for494; Game bit40 chooses a different applier.
        # No literal0 is supplied for an unknown serialized owner.
        row['nativeApplierKnown'] = original['raw494'] is False and original['entityFlags'] == 4 and int(source['runFlags10ead8']) & 0x40 == 0
        try:
            row['portApplierKnown'] = actual['raw494'] is False and int(source_run_flags(state)) & 0x40 == 0
        except Exception as error:
            row['portApplierKnown'] = False
            missing('typed contact port damage run-flag owner', t, error)
        key = (source['roomDescriptor']['gridIndex'], *identity_tuple(identity))
        chain = common_damage_chains.setdefault(key, {'identity': identity, 'rows': []})
        previous = chain['rows'][-1] if chain['rows'] else None
        if previous and previous['presentation'] == t:
            continue  # Same recorded gate is not another accepted enqueue.
        if previous and native_counter % 2 == row['portCounter'] % 2 == 1:
            for side in ('native', 'port'):
                if (row[side + 'GameAge'] != previous[side + 'GameAge'] + 1
                        or not previous[side + 'Queued'] or not previous[side + 'ApplierKnown'] or not row[side + 'ApplierKnown']):
                    continue
                expected = previous[side + 'Hp']
                fifo = previous[side + 'Fifo']
                if not fifo or any(r is None or r[1] not in (0, 8) or r[0] <= 0 for r in fifo):
                    missing('typed contact ' + side + ' unsupported queued applier', t, 'Unknown flags/nonpositive amount in complete FIFO')
                    continue
                for record in fifo:
                    expected = f32(expected - record[0])
                if expected < 0:
                    missing('typed contact ' + side + ' lethal subclass lifecycle', t, 'Captured lethal queue needs its separate callback/cleanup witness')
                    continue
                check('typed contact ' + side + ' next-Full ordered f32 HP application', expected, row[side + 'Hp'], t)
                row[side + 'Applied'] = expected == row[side + 'Hp']
        chain['rows'].append(row)


def equal(expected, actual):
    if isinstance(expected, bool) or isinstance(actual, bool):
        return type(expected) is type(actual) and expected == actual
    if isinstance(expected, (list, tuple)) and isinstance(actual, (list, tuple)):
        return len(expected) == len(actual) and all(equal(a, b) for a, b in zip(expected, actual))
    if isinstance(expected, dict) and isinstance(actual, dict):
        return expected.keys() == actual.keys() and all(equal(expected[key], actual[key]) for key in expected)
    return expected == actual


def check(field, expected, actual, presentation):
    row = {'field': field, 'presentation': presentation, 'expected': expected, 'actual': actual,
           'passed': equal(expected, actual)}
    if field.endswith('Sprite rotation'):
        # The source Sprite+2CC writer is f32. Compare both serialized values
        # as that exact type, retaining bit differences without an epsilon.
        left, right = struct.pack('<f', f32(expected)), struct.pack('<f', f32(actual))
        row.update({'expectedF32Bits': left.hex(), 'actualF32Bits': right.hex(), 'passed': left == right})
    checks.append(row)


def missing(field, presentation, error):
    checks.append({'field': field, 'presentation': presentation, 'passed': False, 'error': str(error)})


fetus_module, fetus_context = None, None
if 'ordinaryDrFetus' in case['requires']:
    fetus_path = Path(__file__).with_name('composition-dr-fetus.py')
    fetus_spec = importlib.util.spec_from_file_location('ordinary_dr_fetus', fetus_path)
    fetus_module = importlib.util.module_from_spec(fetus_spec)
    fetus_spec.loader.exec_module(fetus_module)
    fetus_context = fetus_module.begin(case)

mixed_fetus_comparer = None
if case['requires'].get('mixedDrFetusCricket4') is not None:
    mixed_fetus_spec = importlib.util.spec_from_file_location(
        'mixed_dr_fetus_stats', Path(__file__).with_name('composition-dr-fetus-stats.py'))
    mixed_fetus_module = importlib.util.module_from_spec(mixed_fetus_spec)
    mixed_fetus_spec.loader.exec_module(mixed_fetus_module)
    mixed_fetus_comparer = mixed_fetus_module.MixedDrFetusComparer(globals(), case)

common_spec = None
try:
    common_spec = common_contact_spec()
except Exception as error:
    missing('mandatory typed NPC14/18 contact declaration', None, error)

technology_comparer = None
technology_result = None
if case['requires'].get('ordinaryTechnologyFunctional') is not None:
    technology_spec = importlib.util.spec_from_file_location(
        'ordinary_technology_comparer', Path(__file__).with_name('composition-technology.py'))
    technology_module = importlib.util.module_from_spec(technology_spec)
    technology_spec.loader.exec_module(technology_module)
    technology_comparer = technology_module.TechnologyComparer(globals(), case)
controlled_technology_comparer = None
controlled_technology_result = None
if case['requires'].get('controlledTechnologyFunctional') is not None:
    controlled_technology_spec = importlib.util.spec_from_file_location(
        'controlled_technology_comparer', Path(__file__).with_name('composition-controlled-technology.py'))
    controlled_technology_module = importlib.util.module_from_spec(controlled_technology_spec)
    controlled_technology_spec.loader.exec_module(controlled_technology_module)
    controlled_technology_comparer = controlled_technology_module.ControlledTechnologyComparer(globals(), case)
controlled_fetus_comparer = None
controlled_fetus_result = None
if case['requires'].get('controlledDrFetusFunctional') is not None:
    controlled_fetus_spec = importlib.util.spec_from_file_location(
        'controlled_dr_fetus_comparer', Path(__file__).with_name('composition-controlled-dr-fetus.py'))
    controlled_fetus_module = importlib.util.module_from_spec(controlled_fetus_spec)
    controlled_fetus_spec.loader.exec_module(controlled_fetus_module)
    controlled_fetus_comparer = controlled_fetus_module.ControlledDrFetusComparer(globals(), case)
check('actual completed original', ('pass', []), (native.get('status'), native.get('failures')), None)
check('actual completed browser', 'complete', browser.get('captureStatus'), None)
trace = browser.get('trace') or {}
check('complete actual backend states', (False, None), (trace.get('backendStateTruncated'), trace.get('backendStateTruncationReason')), None)
check('actual trace command delivery', browser['expectedCommands'], trace.get('commands'), None)
check('actual independent boot epoch', case['bootEpochSeed'], trace.get('bootEpochSeed'), None)
check('actual independently declared reset placeholder', 1, trace.get('initialSeed'), None)
start = next(row['context']['presentedFrame'] for row in native['debugObservations'] if row['context']['label'] == 'gameplay-start')
native_title = next(row['context']['presentedFrame'] for row in native['debugObservations'] if row['context']['label'] == 'title')
browser_title = next(row['presentation'] for row in browser['captures'] if row['label'] == 'title')
native_rows = {}
duplicate_observations = []
for row in native['debugObservations']:
    t = row['context']['presentedFrame'] - native_title
    if t in native_rows:
        def mechanical_bytes(observation):
            # These two fields are literal provenance labels set from the
            # context, whose complete original rows remain in the archives.
            # No game/memory/status/error field is omitted or normalized.
            value = dict(observation)
            value.pop('label', None)
            if 'world' in value:
                value['world'] = dict(value['world'])
                value['world'].pop('checkpoint', None)
            return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()
        same = mechanical_bytes(native_rows[t]['observation']) == mechanical_bytes(row['observation'])
        check('same-gate duplicate observations byteequal apart from explicit label/checkpoint provenance', True, same, t)
        duplicate_observations.append({'presentation': t, 'labels': [native_rows[t]['context']['label'], row['context']['label']], 'passed': same})
        # Keep first observation; a conflicting duplicate is a retained failure,
        # never silently overwritten by its later capture.
    else:
        native_rows[t] = row
check('actual generated Challenge seed', native['seedValue'], browser['states'][-1]['state'].get('runSeedValue'), None)
check('actual generated Challenge code', native['seed'].replace(' ', ''), browser['states'][-1]['state'].get('runSeed'), None)
check('source Challenge admission mode', 'source-challenge-menu', native.get('seedMode'), None)
expected_counts = [0] * 347
for item in case['fixture']['startingItems']:
    expected_counts[item] += 1
all_captured = [*browser['preludeStates'], *browser['states']]
for captured in sorted(all_captured, key=lambda row: row['presentation']):
    t = captured['presentation'] - browser_title
    state = captured['state']
    try:
        row = native_rows[t]
        observed = row['observation']
        assert observed['status'] == 'pass' and type(observed['gamePresent']) is bool
        assert observed['memoryWritten'] is False and observed['executableMappingsRead'] is False
        check('actual run admission presence', observed['gamePresent'], state['screen'] == 'playing', t)
        if observed['gamePresent'] is False:
            continue
        source = observed['composition']
        assert source['status'] == 'pass' and len(source['players']) == len(observed['players']) == 1
        paired_rows.append({'presentation': t, 'source': source, 'state': state,
                            'nativeCounter': observed['systemCounter'], 'nativeContext': row['context'],
                            'nativePlayer': observed['players'][0], 'gameplayPresentation': captured.get('gameplayPresentation')})
        if controlled_fetus_comparer is not None:
            try:
                controlled_fetus_comparer.snapshot(source, state, t, observed['systemCounter'], observed['players'][0],
                    native_gameplay_presentation=t - (start - native_title),
                    port_gameplay_presentation=captured.get('gameplayPresentation'))
            except Exception as error:
                missing('mandatory actual controlled DrFetus C18/body group', t, error)
        if mixed_fetus_comparer is not None:
            try:
                mixed_fetus_comparer.snapshot(source, state, t, observed['systemCounter'], observed['players'][0])
            except Exception as error:
                missing('mandatory actual mixed DrFetus stats group', t, error)
        if fetus_context is not None:
            try:
                fetus_module.snapshot(fetus_context, source, state, t, observed['systemCounter'],
                                      globals(), observed['players'][0])
            except Exception as error:
                missing('mandatory typed DrFetus Bomb group', t, error)
        if dip_contact_required:
            try:
                compare_causal_npcs(source, observed['world'], state, t, observed['systemCounter'])
            except Exception as error:
                missing('mandatory actual same-target contact group', t, error)
        if common_spec is not None:
            try:
                compare_common_contacts(source, state, t, observed['systemCounter'], common_spec)
            except Exception as error:
                missing('mandatory typed NPC14/18 contact group', t, error)
        native_families = sorted(identity_tuple(actor['identity']) for actor in source['attacks'] if actor['inActiveManager'])
        port_families = []
        try:
            port_attack_refs = [ref for ref in state['actorScheduler']['actors'] if ref['kind'] in ('tear', 'knife', 'laser')]
            for ref in port_attack_refs:
                try:
                    port_families.append(identity_tuple(reference(state, ref)))
                except Exception as error:
                    missing('actual scheduled attack reference resolves', t, error)
            check('actual complete active attack families/sequence/seeds', native_families, sorted(port_families), t)
        except Exception as error:
            missing('mandatory active attack scheduler group', t, error)
        if controlled_technology_comparer is not None:
            try:
                controlled_technology_comparer.snapshot(source, state, t, observed['systemCounter'], observed['players'][0])
            except Exception as error:
                missing('mandatory actual controlled Technology C18/body/history group', t, error)
        elif technology_comparer is not None:
            try:
                technology_comparer.snapshot(source, state, t, observed['systemCounter'])
            except Exception as error:
                missing('mandatory actual Technology manager-role body group', t, error)
        elif any(actor['identity']['kind'] == 7 for actor in source['attacks']):
            try:
                compare_lasers(source, state, t, observed['systemCounter'])
            except Exception as error:
                missing('mandatory actual Laser body group', t, error)
        for actor in observed['world']['entities']:
            if actor['type'] in (2, 7, 8):
                coverage['nativeFamilyKind' + str(actor['type'])] += 1
        for family, reader in [
            (2, lambda: len([actor for actor in state['tears'] if actor['motion']['lifetime'] != 'removed'])),
            (8, lambda: len(state['knives'])),
            (7, lambda: len(state['lasers'])),
        ]:
            try:
                coverage['browserFamilyKind' + str(family)] += reader()
            except Exception as error:
                missing('actual browser family storage ' + str(family), t, error)
        player = observed['players'][0]
        weapon = source['players'][0]
        if weapon['primaryWeaponC08'] == 4 and any(a['identity']['kind'] == 8 for a in source['attacks']):
            if not any(player['ownedCounts'][329:330]):
                try:
                    ordinary_knives(source, state, t, observed['systemCounter'])
                except Exception as error:
                    missing('mandatory ordinary Knife parent/child body group', t, error)
        try:
            source_slot = weapon['actorSlots']['c18']
            port_slot = state['attackRuntime']['persistent']
            identity = source_slot['identity'] if source_slot else None
            if source_slot and identity['kind'] == 8 and port_slot:
                assert port_slot['kind'] == 'knife'
                source_actor = [actor for actor in source['attacks'] if actor['identity'] == identity]
                port_actor = [actor for actor in state['knives'] if actor['id'] == port_slot['id']]
                assert len(source_actor) == len(port_actor) == 1, 'Actual referenced actor unresolved; no NULL coercion'
                original, actual = source_actor[0], port_actor[0]
                try:
                    controlled = bool(player['ownedCounts'][329] and state['playerItems']['ownedCounts'][329]
                                      and int(original['attackFlags7d8']) & (1 << 33) and int(flags(actual['attackFlags'])) & (1 << 33))
                except Exception as error:
                    missing('actual Knife controlled classification owners', t, error)
                    controlled = None  # Explicitly unknown; cannot establish a controlled witness.
                if controlled:
                    coverage['actualControlledKnifeBodies'] += 1
                    try:
                        check('controlled actual childChain4A0 eligibility', (True, 'empty'),
                              (original['childChain4a0Null'], actual['childChain4a0']), t)
                    except Exception as error:
                        missing('controlled actual childChain4A0 eligibility', t, error)
                check_field('actual Knife own-manager membership', lambda: original['inActiveManager'],
                            lambda: port_slot in state['actorScheduler']['actors'], t)
                check_field('actual primary source sequence', lambda: identity['sequenceId'],
                            lambda: next(i for i in state['actorScheduler']['identities'] if i['actor'] == port_slot)['sequenceId'], t)
                check_field('actual primary birth seed', lambda: identity['spawnSeed'], lambda: actual['spawnSeed'], t)
                registered_knife_body(original, actual, state, t, controlled, observed['systemCounter'])
        except Exception as error:
            missing('mandatory registered Knife body group', t, error)
        for field, expected, getter in [
            ('actual Player movement direction D9C', weapon.get('movementDirectionD9c'), lambda: state['attackRuntime']['movementDirectionD9c']),
            ('actual Luck D64', weapon.get('luckD64'), lambda: state['playerStats']['luck']),
            ('actual Flight D68', weapon.get('flyingD68'), lambda: state['player']['flying']),
            ('actual Player grid class340', weapon.get('gridCollisionClass340'), lambda: state['player']['gridCollisionClass']),
            ('actual Game flags10EAD8', source.get('runFlags10ead8'), lambda: source_run_flags(state)),
            ('actual C0C charge owner', weapon['chargeC0c'], lambda: state['attackRuntime']['chargeC0c']),
            ('actual DA0 head direction', weapon['headDirectionDa0'], lambda: state['attackRuntime']['headDirectionDa0']),
            ('actual retained DB4 shooting direction', vector(weapon['shootingAimDb4']), lambda: vector(state['attackRuntime']['shootingAimDb4'])),
            ('actual DC4 shooting owner', vector(weapon['playerDc4']), lambda: vector(state['attackRuntime']['playerDc4'])),
        ]:
            try:
                assert expected is not None, 'Native output owner is not observed'
                actual_value = getter()
                if field == 'actual Luck D64':
                    expected, actual_value = f32(expected), f32(actual_value)
                check(field, expected, actual_value, t)
            except Exception as error:
                missing(field, t, error)
        check('actual Playing frontdoor', 'playing', state['screen'], t)
        check('actual Challenge ID', (1, 1), (source['challengeId'], state['challengeId']), t)
        check('actual source phase', observed['systemCounter'] % 2, state['systemClock']['systemCounter'] % 2, t)
        if not stages and fetus_context is None:
            check('actual native declared counts', expected_counts, player['ownedCounts'], t)
            check('actual browser declared counts', expected_counts, state['playerItems']['ownedCounts'], t)
        check('actual owned counts at the same public gate', player['ownedCounts'], state['playerItems']['ownedCounts'], t)
        check('actual queued collectible config', player['queuedId'], state['playerItems']['queuedId'], t)
        check('actual cache pending', (0, 0), (player['cacheFlags'], state['playerItems']['unevaluatedCacheFlags']), t)
        check('actual final initialization3074', (True, True), (weapon['initialized3074'], state['player']['initialized3074']), t)
        check('actual canShoot3075', weapon['canShoot3075'], state['player']['canShoot'], t)
        check('actual player controls gate7d4', player['controlsEnabled'], state['player']['controlsEnabled'], t)
        check('actual player input cooldown7d8', player['inputCooldown'], state['player']['inputCooldown'], t)
        check('actual Player firing ordinal counter', player['shotsFired'], state['player']['shotsFired'], t)
        native_player = [actor for actor in observed['world']['entities'] if actor['type'] == 1]
        assert len(native_player) == 1, 'Actual first Player must join the current manager'
        check('actual Player physical world position', vector(native_player[0]['position']),
              [f32((state['player']['x'] - 32) / 0.65), f32((state['player']['y'] + 46) / 0.65)], t)
        check('actual Player world velocity', vector(native_player[0]['velocity']),
              [f32(state['player']['vx'] / 0.65), f32(state['player']['vy'] / 0.65)], t)
        if captured.get('gameplayPresentation') == 0:
            check('declared ready controls enabled', (True, True), (player['controlsEnabled'], state['player']['controlsEnabled']), t)
        check('actual primary cache C08', weapon['primaryWeaponC08'], state['weaponCache']['primary'], t)
        if not stages and (fetus_context is None or player['ownedCounts'][52] > 0):
            check('declared primary cache C08', contract['primaryC08'], state['weaponCache']['primary'], t)
        check('actual cached C98', weapon['attackFlagsC98'], flags(state['weaponCache']['flagsC98']), t)
        for source_field, port_field in [('cachedColorCA0', 'colorCa0'), ('cachedColorCCC', 'colorCcc')]:
            check('actual ' + source_field, weapon[source_field], color(state['weaponCache'][port_field]), t)
        check('actual C74 firing cooldown', player['fireCooldown'], state['fireCooldown'], t)
        check('actual BB0 head timer', player['headTimer'], state['headTimer'], t)
        check('accepted controller frontier', None, state['weaponAdmission']['unadmittedController'], t)
        check('actual pending primary cleanup frontier', None, state['weaponAdmission']['pendingPrimaryCleanup'], t)
        check('actual C18 known-NULL owner', weapon['actorSlots']['c18'] is None, state['weaponAdmission']['c18IsKnownEmpty'], t)
        check('actual transformation counts', weapon['transformationCounts'], state['player']['transformationCounts'], t)
        for key in ('shotSpeed', 'damage', 'tearHeight', 'tearFallingSpeed', 'tearFallingAcceleration', 'moveSpeed'):
            check('actual cached ' + key, f32(player['cachedStats'][key]), f32(state['playerStats'][key]), t)
        check('actual cached fire delay', player['cachedStats']['maxFireDelay'], state['maxFireDelay'], t)
        check('actual modified curse mask', (0, 0), (source['levelCurseMask'], state['floor']['challengeCurse']['filteredMask']), t)
        for name, reader in [
            ('actual room loaded', lambda: (source['roomLoaded'], state['room']['loaded'])),
            ('actual room init timestamp', lambda: (source['roomInitializedGameAge159c'], state['room']['initializationGameUpdate'])),
        ]:
            try:
                expected, actual_value = reader()
                check(name, expected, actual_value, t)
            except Exception as error:
                missing(name, t, error)
        source_slot = weapon['actorSlots']['c18']
        port_slot = state['attackRuntime']['persistent']
        check('actual C18 null versus referenced actor', source_slot is None, port_slot is None, t)
        for source_name, port_ref in zip(('c20', 'c28', 'c30'), state['attackRuntime']['secondary']):
            source_ref = weapon['actorSlots'][source_name]
            check('actual secondary ' + source_name + ' null/reference', source_ref is None, port_ref is None, t)
            if source_ref and port_ref:
                check('actual secondary ' + source_name + ' identity', identity_tuple(source_ref['identity']),
                      identity_tuple(reference(state, port_ref)), t)
        if source_slot:
            identity = source_slot['identity']
            coverage['nativePrimaryKind' + str(identity['kind'])] += 1
            if not stages:
                check('declared actual primary family', (contract['family'], contract['variant'], contract['subtype']),
                      (identity['kind'], identity['variant'], identity['subtype']), t)
    except Exception as error:
        missing('mandatory source/browser field and actor execution', t, error)
check('actual declared native family must execute', True, coverage['nativeFamilyKind' + str(contract['family'])] > 0, None)
check('actual declared browser family must execute', True, coverage['browserFamilyKind' + str(contract['family'])] > 0, None)
if ordinary_laser_required:
    check('actual ordinary Laser body must execute', True, coverage['actualOrdinaryLaserBodies'] > 0, None)
    check('actual ordinary Laser full bodies must execute', True, coverage['ordinaryLaserFullBodies'] > 0, None)
    check('actual ordinary Laser fast bodies must execute', True, coverage['ordinaryLaserFastBodies'] > 0, None)
    check('actual ordinary Impact50 body must execute', True, coverage['actualImpactBodies'] > 0, None)
    check('actual ordinary Impact50 full bodies must execute', True, coverage['impactFullBodies'] > 0, None)
    check('actual ordinary Impact50 fast bodies must execute', True, coverage['impactFastBodies'] > 0, None)
if controlled_knife_required:
    check('actual controlled Knife body must execute', True, coverage['actualControlledKnifeBodies'] > 0, None)
    check('actual controlled Knife normal-full bodies must execute', True, coverage['controlledKnifeFullBodies'] > 0, None)
    check('actual controlled Knife intermediate bodies must execute', True, coverage['controlledKnifeFastBodies'] > 0, None)
if ordinary_knife_required:
    for name in ('ordinaryParentBirths', 'ordinaryParentFullBodies', 'ordinaryParentFastBodies',
                 'ordinaryParentLaunchedRows', 'ordinaryChargeIncreases', 'ordinaryChargeReleaseResets'):
        check('required actual ' + name, True, coverage[name] > 0, None)
    if ordinary_child_required:
        for name in ('ordinaryChildPairedBirths', 'ordinaryChildFullBodies', 'ordinaryChildFastBodies'):
            check('required actual ' + name, True, coverage[name] > 0, None)
    else:
        check('source actual child absent without118', 0, len(native_child_births), None)
        check('port actual child absent without118', 0, len(port_child_births), None)
if case['requires'].get('ordinaryKnifeFunctional', {}).get('actualNpcContactDamageQueue'):
    parent_contacts = [w for chain in damage_chains.values() for w in chain['appliedFifo']
                       if w['queuedWitness']['actualParentRecord']]
    check('required actual subtype0 parent same-target FIFO damage and next-full drain', True, bool(parent_contacts), None)
    if ordinary_child_required:
        child_contacts = [w for chain in damage_chains.values() for w in chain['appliedFifo']
                          if w['queuedWitness']['actualChildRecord']]
        check('required actual subtype1 child same-target FIFO damage and next-full drain', True, bool(child_contacts), None)
        retired = [life for life in child_lifetimes if life['retirement'] and all(life['retirement'][key]
                   for key in ('fullPhaseBoth', 'portStorageAndScheduleAbsent', 'removeFlagsActuallyObserved',
                               'originalParentC18Retained', 'nativeWeaponWeakSlotsClear',
                               'nativeObservedParentLinksClear', 'weakReferencesClear'))]
        check('required actual child Remove then full Uninit with parentC18 retained', True, bool(retired), None)
if dip_contact_required:
    qualified = [chain for chain in damage_chains.values() if chain['queued'] and chain['drain'] and chain['deadActive']
                 and chain['intermediate'] and chain['retirement'] and chain['retirement']['passed']]
    check('required actual same-target Knife FIFO strict-negative dead/active next-full retirement/count chain', True, bool(qualified), None)
    check('actual authored starting Dip must execute before first gameplay interval', True,
          any(row['source']['contactNpcs'] for row in paired_rows if row['presentation'] < start - native_title), None)
if contact_contract and not dip_contact_required:
    missing('declared receiver lifecycle comparison', None,
            'Receiver kind is not the source217 strict-negative lifecycle; its own lifecycle is not yet admitted by this comparer')
if case['requires'].get('actualDoorwayReplacement'):
    # A doorway is a persistent-pool witness. It cannot stand in for Remove.
    previous = None
    for row in paired_rows:
        source, state = row['source'], row['state']
        source_slot = source['players'][0]['actorSlots']['c18']
        port_slot = state['attackRuntime']['persistent']
        if not source['roomLoaded'] or not state['room']['loaded'] or not source_slot or not port_slot:
            continue
        source_attacks = [a for a in source['attacks'] if a['identity'] == source_slot['identity']]
        port_attacks = [a for a in state['knives'] if port_slot['kind'] == 'knife' and a['id'] == port_slot['id']]
        if len(source_attacks) != 1 or len(port_attacks) != 1:
            continue  # The per-row unresolved relation is already a failure.
        original, actual = source_attacks[0], port_attacks[0]
        current = {'presentation': row['presentation'], 'nativeRoom': source['roomDescriptor']['gridIndex'],
                   'portRoom': state['room']['descriptor']['gridIndex'], 'nativeSlot': source_slot['identity'], 'portSlot': port_slot,
                   'nativeBirth': (original['identity']['spawnSeed'], original['bornGameAge'], original['initUniform8a8']),
                   'portBirth': (actual['spawnSeed'], actual['birthGameUpdate'], actual['raw8a8']),
                   'nativeActorRng': original['actorRngWords7c0'], 'portActorRng': actual['actorRng'],
                   'nativeActive': original['active'], 'portActive': actual['active']}
        if previous and previous['nativeRoom'] != current['nativeRoom']:
            witness = {'before': previous, 'after': current}
            witness['passed'] = (previous['portRoom'] != current['portRoom'] and current['nativeRoom'] == current['portRoom']
                and previous['nativeSlot'] == current['nativeSlot'] and previous['portSlot'] == current['portSlot']
                and previous['nativeBirth'] == current['nativeBirth'] and previous['portBirth'] == current['portBirth']
                and previous['nativeActorRng'] == current['nativeActorRng'] and previous['portActorRng'] == current['portActorRng']
                and current['nativeActive'] and current['portActive'])
            doorway_witnesses.append(witness)
            check('actual doorway persistent Knife identity/C18/birth/RNG', True, witness['passed'], row['presentation'])
        previous = current
    check('required actual changed-room persistent Knife doorway witness', True, any(w['passed'] for w in doorway_witnesses), None)
history = {'checks': [], 'witnesses': [], 'coverageGaps': []}
if stages:
    try:
        history_path = Path(__file__).with_name('composition-history.py')
        spec = importlib.util.spec_from_file_location('composition_history', history_path)
        history_module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(history_module)
        history = history_module.compare_history(case, paired_rows)
        assert all(type(check['passed']) is bool for check in history['checks'])
        checks.extend(history['checks'])
    except Exception as error:
        missing('required actual history comparison execution', None, error)

# Qualify real prior admission -> transient full-Uninit/registered NULL.
# Absent objects are not given fabricated native inactive/body records.
qualified_lifetimes = []
for lifetime in laser_lifetimes.values():
    identity, ref = lifetime['nativeIdentity'], lifetime['portRef']
    born = lifetime['rows'][0]
    retirement = None
    for row in paired_rows:
        if row['presentation'] <= born['presentation']:
            continue
        source, state = row['source'], row['state']
        original = [a for a in source['attacks'] if a['identity'] == identity]
        actual = [a for a in state['lasers'] if a['id'] == ref['id']]
        native_slot = source['players'][0]['actorSlots']['c18']
        port_slot = state['attackRuntime']['persistent']
        native_absent = not original
        port_unscheduled = ref not in state['actorScheduler']['actors']
        port_uninitialized = actual[0]['uninitialized'] if len(actual) == 1 else None
        slot_cleared = native_slot is None and port_slot is None and state['weaponAdmission']['c18IsKnownEmpty'] is True
        if native_absent and port_unscheduled and slot_cleared:
            retirement = {'presentation':row['presentation'],'nativeCounter':row['nativeCounter'],
                'portCounter':state['systemClock']['systemCounter'],'nativeActorAbsent':True,
                'portUnscheduled':True,'portUninitialized':port_uninitialized,
                'portStorageAbsent':not actual,'registeredSlotsActuallyNull':True,
                'fullPhaseBoth':row['nativeCounter']%2==state['systemClock']['systemCounter']%2==1,
                'scope':'Actual previously admitted own C18 actor absent, slots NULL and port unscheduled. Absent native actor has no synthetic inactive/Uninit field.'}
            retirement['passed'] = retirement['fullPhaseBoth'] and (port_uninitialized is True or not actual)
            if retirement['passed']:
                qualified_lifetimes.append(retirement)
                break
    lifetime['retirementWitness'] = retirement
if ordinary_laser_required:
    check('required actual ordinary transient full-Uninit and registered C18 NULL witness',True,bool(qualified_lifetimes),None)
if common_spec is not None:
    declaration = common_spec[0]
    rows = [row for chain in common_damage_chains.values() for row in chain['rows']]
    for side in ('native', 'port'):
        for key, field in (('minimumAcceptedEnqueuesPerGame', 'Queued'), ('minimumNextFullDamageApplicationsPerGame', 'Applied')):
            minimum = declaration[key]
            # A retained FIFO proves an accepted enqueue, not an exact dynamic
            # append count across unobserved boundaries or repeated GL gates.
            check('typed contact supported minimum ' + key, 1, minimum, None)
            check('required typed contact ' + side + ' ' + field, True, any(row[side + field] for row in rows), None)
    if case['requires'].get('ordinaryBrimContact') is not None:
        missing('required typed contact exact Laser callback attribution', None,
            'Accepted sourceType1/flags8 FIFO identifies ParentPlayer. A retained record plus current C18/amount does not uniquely identify the Laser that enqueued it; an actual bounded enqueue window is still required.')
        check('typed contact supported natural-retirement minimum', 1, declaration['minimumNaturalLaserRetirementsPerGame'], None)
        natural = [life for life in laser_lifetimes.values() if life['retirementWitness'] and life['retirementWitness']['passed']
            and any(r['nativeRemaining'] > 0 and r['portRemaining'] > 0 for r in life['rows'])
            and any(r['nativeRemaining'] == r['portRemaining'] == 0 for r in life['rows'])]
        check('required typed contact ordinary Laser natural timer and following-Full C18 NULL', True, bool(natural), None)
    if case['requires'].get('controlledTearContact') is not None:
        for key in ('firstRegistryHit', 'bit33Veto', 'halfTearVelocityReceiverTypes', 'repeatedRegistryHitSkipsBothUntilRealClear'):
            if declaration.get(key):
                missing('typed Tear contact ' + key + ' coverage', None,
                    'FIFO/HP observations do not expose the pre-callback registry or isolate callback impulse/veto ordering')

# Fixed observation PNGs are chosen solely from the predeclared interval.
for captured in browser['captures']:
    if '-observe-step' not in captured['label']:
        continue
    t = captured['presentation'] - browser_title
    native_record = native_rows.get(t)
    if native_record is None:
        missing('mandatory rendered-frame original observation',t,'Fixed PNG has no same-gate native observation')
        continue
    original_row = native_record['observation']
    check('actual rendered-frame original observation status','pass',original_row.get('status'),t)
    if original_row.get('gamePresent') is False:
        continue
    source, state = original_row['composition'], captured['state']
    native_living = [a for a in source['attacks'] if a['identity']['kind']==7 and a['identity']['variant']==1 and a['identity']['subtype']==0 and a['active'] and a['visible'] and not a['dead']]
    port_living = [a for a in state['lasers'] if a['entityType']==7 and a['variant']==1 and a['subtype']==0 and a['active'] and a['visible'] and not a['dead']]
    if native_living or port_living:
        witness={'label':captured['label'],'presentation':t,'actualBrowserPNG':captured['path'],'actualBrowserPNGSha256':captured['sha256'],
            'nativeLivingOrdinaryLasers':len(native_living),'portLivingOrdinaryLasers':len(port_living)}
        witness['passed']=bool(native_living) and bool(port_living)
        render_witnesses.append(witness)
        check('actual PNG capture during native and port ordinary Laser life',True,witness['passed'],t)
if ordinary_laser_required:
    check('required actual ordinary Laser rendered-frame witness',True,any(w['passed'] for w in render_witnesses),None)

# These are actual captured-frame/body witnesses, not pixel-parity claims.
for label, controlled, required in [('ordinary Knife', False, ordinary_knife_required),
                                    ('controlled Knife', True, controlled_knife_required)]:
    if not required:
        continue
    seen = []
    for captured in browser['captures']:
        if '-observe-step' not in captured['label']:
            continue
        t = captured['presentation'] - browser_title
        try:
            observed = native_rows[t]['observation']
            if observed['gamePresent'] is False:
                continue
            assert observed['status'] == 'pass'
            source, state = observed['composition'], captured['state']
            native_living = [a for a in source['attacks'] if identity_tuple(a['identity'])[:3] == (8, 0, 0)
                             and a['active'] and a['visible'] and not a['dead']
                             and bool(int(a['attackFlags7d8']) & (1 << 33)) == controlled]
            port_living = [a for a in state['knives'] if (a['entityType'], a['variant'], a['subtype']) == (8, 0, 0)
                           and a['active'] and a['visible'] and not a['dead']
                           and bool(int(flags(a['attackFlags'])) & (1 << 33)) == controlled]
            if not native_living and not port_living:
                continue
            frames = [f for f in native['frames'] if f['label'] == captured['label']]
            assert len(frames) == 1, 'Exactly one actual same-label Native PNG required'
            native_png, port_png = Path(frames[0]['path']), Path(captured['path'])
            assert native_png.is_file() and port_png.is_file()
            witness = {'kind': label, 'label': captured['label'], 'presentation': t,
                       'nativeLiving': len(native_living), 'portLiving': len(port_living),
                       'actualNativePNG': str(native_png), 'actualBrowserPNG': str(port_png),
                       'actualNativePNGSha256': hashlib.sha256(native_png.read_bytes()).hexdigest(),
                       'actualBrowserPNGSha256': hashlib.sha256(port_png.read_bytes()).hexdigest(),
                       'passed': bool(native_living) and bool(port_living)}
            render_witnesses.append(witness)
            seen.append(witness)
            check('actual PNG capture during native and port ' + label + ' life', True, witness['passed'], t)
        except Exception as error:
            missing('mandatory actual ' + label + ' rendered-frame group', t, error)
    check('required actual ' + label + ' rendered-frame witness', True, any(w['passed'] for w in seen), None)

controlled_ring_contact = None
if 'controlledBrimContact' in case['requires']:
    try:
        contact_path = Path(__file__).with_name('composition-controlled-brim-contact.py')
        contact_spec = importlib.util.spec_from_file_location('controlled_brim_ring_contact', contact_path)
        contact_module = importlib.util.module_from_spec(contact_spec)
        contact_spec.loader.exec_module(contact_module)
        controlled_ring_contact = contact_module.compare_contact(case, paired_rows, globals())
    except Exception as error:
        missing('mandatory controlled ring typed contact consumer', None, error)
if controlled_technology_comparer is not None:
    controlled_technology_result = controlled_technology_comparer.finish()
if technology_comparer is not None:
    technology_result = technology_comparer.finish()

if controlled_fetus_comparer is not None:
    controlled_fetus_result = controlled_fetus_comparer.finish()

mixed_dr_fetus = mixed_fetus_comparer.finish() if mixed_fetus_comparer is not None else None
ordinary_dr_fetus = None
if fetus_context is not None:
    ordinary_dr_fetus = fetus_module.finish(fetus_context, globals())

result = {'schema': 2, 'status': 'fail' if any(not item['passed'] for item in checks) else 'pass', 'checks': checks,
          'coverage': dict(coverage), 'damageChains': list(damage_chains.values()), 'doorwayWitnesses': doorway_witnesses, 'laserLifetimes':list(laser_lifetimes.values()), 'renderWitnesses':render_witnesses,
          'sameGateDuplicateObservations': duplicate_observations, 'history': history, 'childLifetimes': child_lifetimes, 'chargeWitnesses': charge_witnesses,
          'backendAcceptanceStatus': 'incomplete',
          'requiredBackendCoverageGaps': ['History requirements are graded by their explicit versioned raw witnesses; missing transient Remove flags remain failures even when later manager absence is observed.', 'Body comparison covers ordinary7/1/0 with linked1000/50/1 and actual registered Knife/ParentKnife roles. Other Laser subtypes and general modifier domains require separate admitted comparisons; no whole-backend acceptance is implied.', 'Player birth seed uses the actual PlayerItems.entitySpawnSeed owner; missing or unknown fields remain failures. Native regular-Tear physical-position phase binding remains unclosed.'],
          'scope': 'Public resource game-level composition. Actual declared fields and complete active attack identities/slots. New body correspondence uses each own actual registered C18, no crossside fitted IDs/positions; all independent identity/seed mismatches remain failures. Case PASS cannot accept the whole backend while its separate required gaps remain. Every mismatch/missing field retained; syntax checks are not game acceptance.',
          'sourceSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
if common_spec is not None:
    result['commonDamageChains'] = list(common_damage_chains.values())
    result['commonContactScope'] = 'Actual common-class NPC14/18 FIFO and nonlethal next-Full HP. Player-source FIFO is not a particular Laser callback witness. Native copied EntityRef raw words/P/V are retained, not reconstructed from current sources. Countdown30 is a source callback argument, not a stored queue field. Full copied-history, registry/impulse order, lethal subclass cleanup and explicit Remove/Uninit callbacks remain separately unqualified.'
if controlled_ring_contact is not None:
    result['controlledRingContact'] = controlled_ring_contact
if technology_result is not None:
    result['ordinaryTechnology'] = technology_result
if controlled_technology_result is not None:
    result['controlledTechnology'] = controlled_technology_result
if ordinary_dr_fetus is not None:
    result['ordinaryDrFetus'] = ordinary_dr_fetus
if controlled_fetus_result is not None:
    result['controlledDrFetus'] = controlled_fetus_result
if mixed_dr_fetus is not None:
    result['mixedDrFetusCricket4'] = mixed_dr_fetus
payload = (json.dumps(result, separators=(',', ':')) + '\n').encode()
with args.output.open('xb') as stream:
    stream.write(gzip.compress(payload, mtime=0) if str(args.output).endswith('.gz') else payload)
print(json.dumps({'status': result['status'], 'checks': len(checks), 'differences': sum(not item['passed'] for item in checks)}))
raise SystemExit(0 if result['status'] == 'pass' else 1)
