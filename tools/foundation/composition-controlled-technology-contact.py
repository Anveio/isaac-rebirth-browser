"""Side-scoped canonical7/2/1 NPC contact evidence with exact actor joins.

Canonical Brim source/FIFO/material/HP helpers are reused. Own mode never creates a
Native side; paired material grades require two complete current maps and the
actual five-field identity. FIFO/HP endpoints are state compatibility only.
"""
import importlib.util
from pathlib import Path

_shared_spec = importlib.util.spec_from_file_location('composition_controlled_ring_shared', Path(__file__).with_name('composition-controlled-brim-contact.py'))
_shared = importlib.util.module_from_spec(_shared_spec)
_shared_spec.loader.exec_module(_shared)


def boolean(value):
    assert type(value) is bool, 'Actual boolean owner required'
    return value


def integer(value):
    assert type(value) is int, 'Actual integer owner required'
    return value



def uint32(value):
    assert type(value) is int and 0 <= value <= 0xffffffff, 'Actual unsigned 32-bit owner required'
    return value


def strict_identity(value):
    assert type(value) is dict, 'Actual identity object required'
    return tuple(uint32(value[key]) for key in ('kind', 'variant', 'subtype', 'sequenceId', 'spawnSeed'))


def handle(value):
    assert type(value) is dict and type(value['kind']) is str, 'Actual typed ActorRef required'
    uint32(value['id'])
    return value


def validate_port_handles(state):
    for row in state['actorScheduler']['identities']:
        handle(row['actor'])
        uint32(row['sequenceId'])
    for value in state['actorScheduler']['actors']:
        handle(value)
    slot = state['attackRuntime']['persistent']
    if slot is not None:
        handle(slot)


def strict_reference(state, value, common):
    validate_port_handles(state)
    if value is None:
        return None
    handle(value)
    arrays = {'knife': 'knives', 'laser': 'lasers', 'tear': 'tears', 'bomb': 'bombs'}
    candidates = (state['room']['npcs'] if value['kind'] == 'npc' else
        [a['impact'] for a in state['lasers'] if a.get('impact') is not None] if value['kind'] == 'effect' else
        state[arrays[value['kind']]] if value['kind'] in arrays else [])
    for actor in candidates:
        uint32(actor['id'])
    result = common['reference'](state, value)
    strict_identity(result)
    return result


def strict_common(common):
    result = dict(common)
    result['identity_tuple'] = strict_identity
    result['reference'] = lambda state, value: strict_reference(state, value, common)
    return result


BEAM_FIELDS = (
    'position', 'velocity', 'savedPosition450', 'renderOffset', 'active', 'dead', 'visible', 'raw494',
    'gridCollisionClass', 'entityCollisionClass', 'hp', 'maxHp', 'collisionDamage', 'collisionRadius',
    'collisionMass', 'currentFriction470', 'configuredFriction46c', 'actualFastFlag', 'firstUpdate',
    'lineEnabled', 'contactDisabled', 'lineController', 'attackFlags', 'lineFlags', 'ParentIdentity',
    'LinePoints', 'LinePriorPoints', 'LineHistory', 'LineOrigin', 'LineOffset', 'LineControls',
    'LineLength', 'sourceScopedLineWidth', 'LineSubdivisions', 'LineMode', 'persistentTTL0')
NPC_FIELDS = (
    'hp', 'maxHp', 'pendingDamage', 'pendingFlags', 'dead', 'visible', 'raw494',
    'entityCollisionClass', 'champion', 'managerMembership', 'npcState', 'npcFrameAge',
    'position', 'velocity', 'collisionRadius', 'orderedFifoLength', 'orderedFifoAmount',
    'orderedFifoFlags', 'orderedFifoSourceType', 'orderedFifoPayloadIdentities')


def slot_is_empty(side, source, state):
    if side == 'native':
        return source['players'][0]['actorSlots']['c18'] is None
    slot = state['attackRuntime']['persistent']
    if slot is None:
        assert state['weaponAdmission']['c18IsKnownEmpty'] is True, 'Actual port NULL C18 owner unavailable'
    return slot is None


def slot_identity(side, source, state, owners):
    if slot_is_empty(side, source, state):
        return None
    owner = owners[side]
    assert owner is not None, 'Actual nonnull C18 identity owner unavailable'
    return owner['identity']


def collect_current_beams(side, source, state, common, t):
    """Current scheduler map and observations remain separate from stored retirement."""
    identity, reference = common['identity_tuple'], common['reference']
    result, observations, known = {}, [], True
    label = 'controlled Technology ' + side + ' '
    try:
        if side == 'port':
            validate_port_handles(state)
        actors = ([a for a in source['attacks'] if identity(a['identity'])[:3] == (7, 2, 1)] if side == 'native'
            else [a for a in state['lasers'] if uint32(a['variant']) == 2 and uint32(a['subtype']) == 1])
    except Exception as error:
        common['missing'](label + 'complete current beam roster', t, error)
        return None, []
    for actor in actors:
        record = dict(identity=None, actor=actor, current=False)
        try:
            if side == 'native':
                if not boolean(actor['inActiveManager']):
                    continue
                key = identity(actor['identity'])
            else:
                ref = handle(dict(kind='laser', id=actor['id']))
                manager = ref in state['actorScheduler']['actors']
                refs = [i for i in state['actorScheduler']['identities'] if i['actor'] == ref]
                if boolean(actor['uninitialized']):
                    if manager or refs:
                        raise ValueError('Retired stored Laser contradicts current manager/identity owners')
                    continue
                key = identity(reference(state, ref))
                assert manager, 'Nonretired Laser is missing current manager membership'
            assert key[:3] == (7, 2, 1), 'Actual current canonical7/2/1 identity required'
            record.update(identity=key, current=True)
            if key in result:
                raise ValueError('Duplicate exact current beam identity')
            result[key] = actor
        except Exception as error:
            record['current'] = False
            known = False
            common['missing'](label + 'current beam identity/manager owner', t, error)
        observations.append(record)
    return result if known else None, observations


def collect_current_npcs(side, source, state, receiver, common, t):
    identity, reference = common['identity_tuple'], common['reference']
    result, observations, known = {}, [], True
    try:
        if side == 'port':
            validate_port_handles(state)
        actors = ([a for a in source['commonContactNpcs'] if identity(a['identity'])[0] == receiver['type']] if side == 'native'
            else [a for a in state['room']['npcs'] if uint32(a['kind']) == receiver['type'] and not boolean(a['removed'])])
    except Exception as error:
        common['missing']('controlled Technology ' + side + ' complete current NPC roster', t, error)
        return None, []
    for actor in actors:
        record = dict(identity=None, actor=actor, current=False)
        try:
            if side == 'native':
                if not boolean(actor['inActiveManager']):
                    continue
                key = identity(actor['identity'])
            else:
                ref = handle(dict(kind='npc', id=actor['id']))
                key = identity(reference(state, ref))
                assert ref in state['actorScheduler']['actors'], 'Actual current NPC manager owner required'
            assert key[:3] == (receiver['type'], 0, 0), 'Actual declared typed receiver required'
            record.update(identity=key, current=True)
            if key in result:
                raise ValueError('Duplicate exact current NPC identity')
            result[key] = actor
        except Exception as error:
            record['current'] = False
            known = False
            common['missing']('controlled Technology ' + side + ' current NPC identity/manager owner', t, error)
        observations.append(record)
    return result if known else None, observations


def strict_rows(kind, maps, observations, fields, common, t):
    complete = maps['native'] is not None and maps['port'] is not None
    joined = set(maps['native']) & set(maps['port']) if complete else set()
    common['check']('controlled Technology strict ' + kind + ' complete current maps', True, complete, t)
    if complete:
        common['check']('controlled Technology strict ' + kind + ' exact five-field roster',
            sorted(maps['native']), sorted(maps['port']), t)
    report = dict(presentation=t, kind=kind, exactIdentities=sorted(joined), sides={})
    for side in ('native', 'port'):
        unmatched = [row for row in observations[side] if row['identity'] not in joined]
        if maps[side] is None and not observations[side]:
            for field in fields:
                common['missing']('controlled Technology ' + side + ' UNGRADED ' + kind + ' ' + field, t,
                    'Current roster unavailable; no exact actor or material observation can be manufactured')
        for row in unmatched:
            for field in fields:
                common['missing']('controlled Technology ' + side + ' UNGRADED ' + kind + ' ' + field, t,
                    dict(identity=row['identity'], reason='No complete exact five-field partner; source roles do not establish correspondence'))
        report['sides'][side] = dict(currentMapKnown=maps[side] is not None,
            availableCurrentRows=len(observations[side]), exactIdentityJoinRows=len(joined),
            unmatchedRows=len(unmatched), ungradedMaterialObligations=len(unmatched)*len(fields) +
                (len(fields) if maps[side] is None and not observations[side] else 0))
    return report


def collect_impact(side, source, state, common, t, observations):
    identity, reference = common['identity_tuple'], common['reference']
    try:
        native = side == 'native'
        if not native:
            validate_port_handles(state)
            for actor in state['lasers']:
                uint32(actor['id'])
                if actor['impact'] is not None:
                    uint32(actor['impact']['id'])
        else:
            for actor in source['attacks'] + source['impacts']:
                identity(actor['identity'])
        slot = source['players'][0]['actorSlots']['c18'] if native else state['attackRuntime']['persistent']
        if slot is None:
            if not native:
                assert state['weaponAdmission']['c18IsKnownEmpty'] is True, 'Port C18 NULL owner unknown'
            return
        if native:
            identity(slot['identity'])
        actors = ([a for a in source['attacks'] if a['identity'] == slot['identity']] if native
            else [a for a in state['lasers'] if slot['kind'] == 'laser' and a['id'] == slot['id']])
        assert len(actors) == 1, 'One actual C18 Laser required'
        actor = actors[0]
        parent = identity(actor['identity'] if native else reference(state, slot))
        assert parent[:3] == (7, 2, 1), 'Actual Technology7/2/1 owner required'
        child = actor['laser']['child'] if native else actor['child']
        assert child is not None, 'Actual registered Impact50/0 required'
        if native:
            identity(child)
        else:
            handle(child)
        key = identity(child if native else impact_reference(state, child))
        assert key[:3] == (1000, 50, 0), 'Actual canonical Impact50/0 role required'
        bodies = ([a for a in source['impacts'] if a['identity'] == child] if native
            else [actor['impact']] if actor['impact'] is not None and actor['impact']['id'] == child['id'] else [])
        assert len(bodies) == 1, 'Actual registered child body required'
        impact = bodies[0]
        linked_parent = identity(impact['linkedParent'] if native else reference(state, impact['parent']))
        manager = boolean(impact['inActiveManager'] if native else child in state['actorScheduler']['actors'])
        common['check']('controlled Technology ' + side + ' Impact actual ParentLaser', parent, linked_parent, t)
        common['check']('controlled Technology ' + side + ' Impact actual registration', True, manager, t)
        entry = observations.setdefault(key, dict(identity=key, rows=0, registeredRows=0))
        entry['rows'] += 1
        entry['registeredRows'] += int(manager and linked_parent == parent)
    except Exception as error:
        common['missing']('controlled Technology ' + side + ' actual registered Impact owner', t, error)


def impact_reference(state, ref):
    """Canonical nested Impact50 with an explicit stored sourceSubtype0 owner."""
    validate_port_handles(state)
    handle(ref)
    for actor in state['lasers']:
        if actor['impact'] is not None:
            uint32(actor['impact']['id'])
    assert ref['kind'] == 'effect', 'Actual registered Effect handle required'
    bodies = [a['impact'] for a in state['lasers']
        if a['impact'] is not None and a['impact']['id'] == ref['id']]
    identities = [i for i in state['actorScheduler']['identities'] if i['actor'] == ref]
    assert len(bodies) == len(identities) == 1, 'Actual Impact reference must resolve uniquely'
    actual = bodies[0]
    assert type(actual['sourceSubtype']) is int and actual['sourceSubtype'] == 0
    #Nested Impact is the canonical Effect50 store. Its subtype is stored;
    #the older shared reference() remains source-scoped to Brim subtype1.
    return dict(kind=1000, variant=50, subtype=actual['sourceSubtype'],
        sequenceId=identities[0]['sequenceId'], spawnSeed=actual['spawnSeed'])


def compare_beam_fields(n, p, owners, common, t):
    _shared.compare_ring_body(n, p, owners, common, t)
    check, missing, field = (common[k] for k in ('check', 'missing', 'check_field'))
    f32, vector = common['f32'], common['vector']
    technology = True
    def native_width():
        value = n['laser']['controlledTechnologyLine']
        assert value['sourceWriterScope'] is True and n['laser']['firstUpdate'] is False
        assert len(n['laser']['line']['points']) >= 2
        assert value['subdivisions'] == 50 and value['mode'] == 1
        return f32(value['width'])
    field('controlled ring actual source-scoped Line width', native_width, lambda: f32(p['line']['width']), t)
    if technology:
        for name, expected in (('subdivisions', 50), ('mode', 1)):
            field('controlled Technology contact actual Line ' + name,
                lambda: integer(n['laser']['controlledTechnologyLine'][name]), lambda: integer(p['line'][name]), t)
            field('controlled Technology contact source-owned Line ' + name,
                lambda: expected, lambda: integer(n['laser']['controlledTechnologyLine'][name]), t)
        field('controlled Technology contact persistent TTL0', lambda: integer(n['laser']['remainingUpdates']), lambda: integer(p['remainingUpdates']), t)
        field('controlled Technology contact source TTL0', lambda: 0, lambda: integer(n['laser']['remainingUpdates']), t)

def compare_npc_fields(original, actual, state, common, t, fifo_for):
    event_flags = {'laser': 8, 'tear': 0, 'knife': 0, 'bomb': 4, 'fireContact': 2, 'roomEffect': 0}
    missing = common['missing']
    def unmatched_fifo(native_records, port_events, t):
        for side, records, other in (('native', native_records, port_events), ('port', port_events, native_records)):
            for index in range(len(other), len(records)):
                for name in ('amount', 'flags', 'sourceType', 'sourceVariant', 'attackerIdentity'):
                    missing('controlled Technology ' + side + ' UNGRADED unmatched FIFO ' + name, t,
                        dict(index=index, reason='No actual ordered FIFO partner on the same exact receiver'))
    _shared.compare_ring_receiver(original, actual, state, common, t, event_flags, fifo_for, unmatched_fifo=unmatched_fifo)

def compare_contact(case, rows, common, *, sides=('native', 'port')):
    """Consume real rows only for the declared side scope."""
    common = strict_common(common)
    marker, schema, primary, variant = 'controlledTechnologyContact', 'controlled-technology-ring-contact-v1', 3, 2
    if not sides or any(s not in ('native', 'port') for s in sides) or len(set(sides)) != len(sides):
        raise ValueError('Actual unique side scope required')
    check, missing, field = (common[k] for k in ('check', 'missing', 'check_field'))
    f32, vector, reference, identity = (common[k] for k in ('f32', 'vector', 'reference', 'identity_tuple'))
    run_flags = common['source_run_flags']
    technology = variant == 2
    declaration = case['requires'][marker]
    result = {'schema': schema, 'chains': [],
              'scope': 'Observed compatible ParentPlayer FIFO append and nonlethal following-Full HP application. Exact Laser callback attribution, actual collision delivery and lifecycle events are separate required witnesses.'}
    try:
        assert declaration['schema'] == result['schema']
        receiver = declaration['authoredReceiver']
        assert type(receiver['type']) is int and receiver['type'] in (14, 18)
        assert receiver['variant'] == receiver['subtype'] == 0
        assert (declaration['sourceEntityType'], declaration['sourceVariant'], declaration['damageFlags']) == (1, 0, 8)
        contract = case['requires']['sourceExpectedActorContract']
        assert (contract['primaryC08'], contract['family'], contract['variant'], contract['subtype'], contract['controlledBy329']) == (primary, 7, variant, 1, True)
        assert 'npc14And18DamageQueue' in case['requires']['compositionSourceFields']
        assert declaration['missingToolingMustFail'] is True
        window = declaration['uniqueSourceWindow']
        assert window['required'] is True and window['missingAttributionIsFailure'] is True
        assert window['minimumEligibleControlledLasers'] == window['maximumEligibleControlledLasers'] == 1
        assert window['maximumCompetingDamageSourceCallbacks'] == 0
        for key in ('minimumAcceptedEnqueuesPerGame', 'minimumNextFullDamageApplicationsPerGame'):
            assert type(declaration[key]) is int and declaration[key] == 1
    except Exception as error:
        missing('controlled ring typed contact declaration', None, error)
        return result

    def boolean(value):
        assert type(value) is bool, 'Actual boolean owner required'
        return value

    def integer(value):
        assert type(value) is int, 'Actual integer owner required'
        return value

    def source_owner(source, state, side):
        slot_is_empty(side, source, state)
        if side == 'native':
            slot = source['players'][0]['actorSlots']['c18']
            if slot is not None:
                strict_identity(slot['identity'])
            for actor in source['attacks']:
                strict_identity(actor['identity'])
        else:
            validate_port_handles(state)
            for actor in state['lasers']:
                uint32(actor['id'])
        return _shared.ring_source_owner(source, state, side, common, variant=2)

    chains = {}
    impact_observations = {side: {} for side in sides}
    correspondence = []
    metrics = {side: dict(canonicalLasers=set(), registeredImpactRows=0) for side in sides}
    source_rows = {side: {'Full': 0, 'Fast': 0} for side in sides}
    event_flags = {'laser': 8, 'tear': 0, 'knife': 0, 'bomb': 4, 'fireContact': 2, 'roomEffect': 0}
    def fifo_for(target, side, state, t):
        records = target['pendingDamageRecords'] if side == 'native' else target['damage']['events']
        for record in records:
            if side == 'native':
                for key in ('flags', 'sourceType', 'sourceVariant'):
                    uint32(record[key])
                strict_identity(record['actualAttacker'])
            else:
                uint32(record['source']['entityType'])
                uint32(record['source']['id'])
        return _shared.ring_fifo_for(target, side, state, t, common, event_flags)

    for captured in rows:
        t, source, state, counter = (captured[k] for k in ('presentation', 'source', 'state', 'nativeCounter'))
        if len(chains) > 256 or sum(len(chain['rows']) for chain in chains.values()) > 4096 or len(common['checks']) > 200000:
            raise RuntimeError('Bounded complete Technology contact history/check count exceeded')
        owners, bodies = {}, {}
        for side in sides:
            try:
                current_primary = integer(source['players'][0]['primaryWeaponC08'] if side == 'native' else state['weaponCache']['primary'])
                check('controlled Technology ' + side + ' actual primary C08', 3, current_primary, t)
                slot_is_empty(side, source, state)
            except Exception as error:
                missing('controlled Technology ' + side + ' independent Player/nullable C18 owner', t, error)
            try:
                owners[side], bodies[side] = source_owner(source, state, side)
                if owners[side] is not None:
                    actor, owner = bodies[side], owners[side]
                    try:
                        actual_parent = actor['linkedParent'] if side == 'native' else reference(state, actor['parent'])
                        expected_parent = source['players'][0]['owner'] if side == 'native' else reference(state, {'kind': 'player', 'id': 0})
                        actual_parent_key, expected_parent_key = identity(actual_parent), identity(expected_parent)
                        assert actual_parent_key == expected_parent_key and actual_parent_key[:3] == (1, 0, 0), 'Actual controlled Laser ParentPlayer required'
                        owner['owner'] = actual_parent_key
                    except Exception as error:
                        missing('controlled ring ' + side + ' actual ParentPlayer owner', t, error)
                    try:
                        owner['amount'] = f32(actor['collisionDamage'])
                    except Exception as error:
                        missing('controlled ring ' + side + ' actual damage amount owner', t, error)
                    try:
                        manager = actor['inActiveManager'] if side == 'native' else state['attackRuntime']['persistent'] in state['actorScheduler']['actors']
                        assert type(manager) is bool and type(actor['active']) is type(actor['dead']) is bool
                        owner['eligible'] = manager and actor['active'] and not actor['dead']
                    except Exception as error:
                        missing('controlled ring ' + side + ' actual source eligibility owner', t, error)
                    try:
                        laser = actor['laser'] if side == 'native' else actor
                        line = laser['controlledTechnologyLine'] if side == 'native' else actor['line']
                        if side == 'native':
                            assert line['sourceWriterScope'] is True and laser['firstUpdate'] is False, 'Actual source Line writer required'
                            assert len(laser['line']['points']) >= 2, 'Actual source Line points required'
                        for name, expected in (('subdivisions', 50), ('mode', 1)):
                            check('controlled Technology ' + side + ' source Line ' + name, expected, integer(line[name]), t)
                        check('controlled Technology ' + side + ' source Line width', f32(60), f32(line['width']), t)
                        check('controlled Technology ' + side + ' persistent source TTL', 0, integer(laser['remainingUpdates']), t)
                    except Exception as error:
                        missing('controlled Technology ' + side + ' independent actual Line/TTL owner', t, error)
                    if owner['eligible']:
                        actual_counter = counter if side == 'native' else state['systemClock']['systemCounter']
                        if technology:
                            fast = boolean(actor['fast322'] if side == 'native' else actor['fast'])
                            source_rows[side]['Fast' if fast else 'Full'] += 1
                        else:
                            source_rows[side]['Full' if actual_counter % 2 else 'Fast'] += 1
            except Exception as error:
                owners[side], bodies[side] = None, None
                missing('controlled ring ' + side + ' actual C18 source owner', t, error)
        current_maps, current_rows = {}, {}
        npc_maps, npc_rows = {}, {}
        for side in sides:
            current_maps[side], current_rows[side] = collect_current_beams(side, source, state, common, t)
            for row in current_rows[side]:
                if row['identity'] is not None:
                    metrics[side]['canonicalLasers'].add(row['identity'])
            # Child collection is independent of parent body fields and NPCs.
            collect_impact(side, source, state, common, t, impact_observations[side])
        if set(sides) == {'native', 'port'}:
            field('controlled Technology actual nullable C18 relation',
                lambda: slot_is_empty('native', source, state),
                lambda: slot_is_empty('port', source, state), t)
            field('controlled Technology actual C18 identity/seed',
                lambda: slot_identity('native', source, state, owners),
                lambda: slot_identity('port', source, state, owners), t)
            record = strict_rows('beam', current_maps, current_rows, BEAM_FIELDS, common, t)
            correspondence.append(record)
            for key in record['exactIdentities']:
                n, p = current_maps['native'][key], current_maps['port'][key]
                own_parents = {}
                for side, actor in (('native', n), ('port', p)):
                    try:
                        parent = actor['linkedParent'] if side == 'native' else reference(state, actor['parent'])
                        own_parents[side] = dict(owner=identity(parent))
                    except Exception as error:
                        own_parents[side] = dict(owner=None)
                        missing('controlled Technology ' + side + ' strict body Parent owner', t, error)
                compare_beam_fields(n, p, own_parents, common, t)
        own_targets = {}
        for side in sides:
            npc_maps[side], npc_rows[side] = collect_current_npcs(side, source, state, receiver, common, t)
            own_targets[side] = [row['actor'] for row in npc_rows[side] if row['identity'] is not None and row['current']]
        if set(sides) == {'native', 'port'}:
            record = strict_rows('receiver', npc_maps, npc_rows, NPC_FIELDS, common, t)
            correspondence.append(record)
            for key in record['exactIdentities']:
                compare_npc_fields(npc_maps['native'][key], npc_maps['port'][key], state, common, t, fifo_for)
        # Own queue/HP observations never depend on cross-game identity success.
        # The paired comparisons above retain strict identity failures separately.
        _shared.observe_ring_fifo_hp(((side, own_targets[side]) for side in sides), source, state, counter, t, chains, owners, common, fifo_for, technology=True, receiver_type=receiver['type'])
    result['chains'] = list(chains.values())
    for side in sides:
        for phase in ('Full', 'Fast'):
            check('required controlled ring ' + side + ' actual registered ' + phase + ' source', True, source_rows[side][phase] > 0, None)
        captured_rows = [r for chain in chains.values() if chain['side'] == side for r in chain['rows']]
        check('required controlled ring ' + side + ' empty-to-compatible-ParentPlayer enqueue', True,
              any(r['emptyAppendObserved'] for r in captured_rows), None)
        check('required controlled ring ' + side + ' same-receiver following-Full HP application', True,
              any(r['nextFullApplied'] for r in captured_rows), None)
        missing('required controlled ring ' + side + ' exact Laser callback attribution', None,
                'Snapshot C18 membership and ParentPlayer FIFO do not exclude competing or retired source callbacks throughout the enqueue window. Actual bounded unique-source callback evidence remains missing.')
        missing('required controlled ring ' + side + ' actual ring collision delivery', None,
                'FIFO/HP and current Line geometry do not expose the accepted collision callback or its segment/eligibility inputs.')
    missing('required controlled ring actual Remove/BaseUninit boundaries', None,
            'This FIFO/HP consumer does not infer Remove or BaseUninit from later absence.')
    result['sides'] = {}
    for side in sides:
        values = [r for chain in chains.values() if chain['side'] == side for r in chain['rows']]
        impacts = list(impact_observations[side].values())
        check('required controlled Technology ' + side + ' registered canonical Impact', True,
            any(entry['registeredRows'] > 0 for entry in impacts), None)
        missing('required controlled Technology ' + side + ' complete copied Parent payload', None,
            'The retained contact collector exposes actual Player FIFO attacker fields; it does not observe every cached-spawner/P/V source-at-copy owner. No Parent reconstruction.')
        result['sides'][side] = dict(metrics=dict(
            canonicalLasers=len(metrics[side]['canonicalLasers']),
            fullBodyRows=source_rows[side]['Full'], fastBodyRows=source_rows[side]['Fast'],
            compatibleEmptyEnqueues=sum(bool(r['emptyAppendObserved']) for r in values),
            followingFullHpApplications=sum(bool(r['nextFullApplied']) for r in values),
            negativeHpEndpoints=sum(bool(r.get('lethalHpEndpointObserved', False)) for r in values),
            registeredImpactRows=sum(entry['registeredRows'] for entry in impacts)), registeredImpacts=impacts)
    result['correspondence'] = dict(status='observed' if len(sides) == 2 else 'unobserved', rows=correspondence,
        scope='Complete current rosters and exact five-field identities only. Unmatched/unavailable rows retain every material field as UNGRADED; no role/position/ordinal match.')
    result['scope'] = ('Own actual C18/Impact and ParentPlayer-compatible FIFO/ordered following-Full HP endpoints, independently collected per game. '
        'Exact actor matching is separate. Snapshot compatibility never proves unique Laser attribution, collision delivery, copied-source completion or Remove/BaseUninit.')
    return result


def compare_own_contact(case, port_rows, common):
    """Only actual closed public State rows, with no synthetic reference side."""
    return compare_contact(case, port_rows, common, sides=('port',))
