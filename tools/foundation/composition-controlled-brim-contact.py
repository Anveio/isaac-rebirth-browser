"""Typed controlled7/1/1 contact observations; callback attribution is separate.

Reuse the canonical315 NPC14/18 common-class readset and field/reference helpers.
The retained 66EF30 vtable/common+38 proof and 446CF0/447190 FIFO/applier owners
apply to these receivers. 476760/476A60 supplies ParentPlayer, flags8 and the
countdown30 callback argument; countdown is not a stored FIFO field. A Player
payload never identifies the particular Laser that appended it.
"""

import struct


def boolean(value):
    assert type(value) is bool, 'Actual boolean owner required'
    return value

def integer(value):
    assert type(value) is int, 'Actual integer owner required'
    return value

def ring_source_owner(source, state, side, common, *, variant):
    identity, reference = common['identity_tuple'], common['reference']
    if side == 'native':
        slot = source['players'][0]['actorSlots']['c18']
        if slot is None:
            return None, None
        actors = [a for a in source['attacks'] if a['identity'] == slot['identity']]
        assert len(actors) == 1, 'Actual Native C18 must resolve one source actor'
        actor = actors[0]
        actor_identity = identity(actor['identity'])
        assert actor_identity[:3] == (7, variant, 1)

    else:
        slot = state['attackRuntime']['persistent']
        if slot is None:
            return None, None
        assert slot['kind'] == 'laser'
        actors = [a for a in state['lasers'] if a['id'] == slot['id']]
        assert len(actors) == 1, 'Actual port C18 must resolve one source actor'
        actor = actors[0]
        actor_identity = identity(reference(state, slot))
        assert actor_identity[:3] == (7, variant, 1)

    return {'identity': actor_identity, 'owner': None, 'amount': None, 'eligible': False}, actor

def ring_fifo_for(target, side, state, t, common, event_flags):
    f32, reference, identity, missing = (common[k] for k in ('f32', 'reference', 'identity_tuple', 'missing'))
    records = target['pendingDamageRecords'] if side == 'native' else target['damage']['events']
    fifo = []
    for record in records:
        try:
            if side == 'native':
                attacker = record['actualAttacker']
                assert attacker is not None, 'Actual stored FIFO attacker unresolved'
                fifo.append([f32(record['amount']), record['flags'], record['sourceType'], record['sourceVariant'], identity(attacker)])
            else:
                source = record['source']
                kind = {1: 'player', 2: 'tear', 7: 'laser', 8: 'knife'}[source['entityType']]
                attacker = reference(state, {'kind': kind, 'id': source['id']})
                fifo.append([f32(record['amount']), event_flags[record['kind']], source['entityType'], attacker['variant'], identity(attacker)])
        except Exception as error:
            fifo.append(None)
            missing('controlled ring ' + side + ' actual FIFO source', t, error)
    return fifo

def compare_ring_body(n, p, owners, common, t):
    check, missing, field = (common[k] for k in ('check', 'missing', 'check_field'))
    f32, vector = common['f32'], common['vector']
    for nk, pk in (('position', 'position'), ('velocity', 'velocity'),
                   ('savedPosition450', 'savedPosition'), ('renderOffset', 'renderOffset')):
        field('controlled ring body ' + nk, lambda: vector(n[nk]), lambda: vector(p[pk]), t)
    for name in ('active', 'dead', 'visible', 'raw494', 'gridCollisionClass', 'entityCollisionClass'):
        normal = boolean if name in ('active', 'dead', 'visible', 'raw494') else integer
        field('controlled ring body ' + name, lambda: normal(n[name]), lambda: normal(p[name]), t)
    for nk, pk in (('hp', 'hp'), ('maxHp', 'maxHp'), ('collisionDamage', 'collisionDamage'),
                   ('collisionRadius', 'collisionRadius'), ('collisionMass', 'collisionMass'),
                   ('currentFriction470', 'currentFriction'), ('configuredFriction46c', 'configuredFriction')):
        field('controlled ring body ' + nk, lambda: f32(n[nk]), lambda: f32(p[pk]), t)
    field('controlled ring body actual Fast flag', lambda: boolean(n['fast322']), lambda: boolean(p['fast']), t)
    for name in ('firstUpdate', 'lineEnabled', 'contactDisabled', 'lineController'):
        normal = integer if name == 'lineController' else boolean
        field('controlled ring body ' + name, lambda: normal(n['laser'][name]), lambda: normal(p[name]), t)
    for name in ('attackFlags', 'lineFlags'):
        field('controlled ring body ' + name, lambda: str(int(n['laser'][name])), lambda: common['flags'](p[name]), t)
    def actual_parent(side):
        value = owners[side]['owner']
        assert value is not None, 'Actual nonnull ParentPlayer identity owner required'
        return value
    field('controlled ring actual Parent identity/seed', lambda: actual_parent('native'), lambda: actual_parent('port'), t)
    for name in ('points', 'priorPoints', 'history'):
        field('controlled ring actual Line ' + name,
              lambda: [vector(v) for v in n['laser']['line'][name]],
              lambda: [vector(v) for v in p['line'][name]], t)
    for name in ('origin', 'offset'):
        field('controlled ring actual Line ' + name, lambda: vector(n['laser']['line'][name]), lambda: vector(p['line'][name]), t)
    field('controlled ring actual Line controls',
          lambda: [[f32(v) for v in values] for values in n['laser']['line']['controls']],
          lambda: [vector(v['position']) + vector(v['incoming']) + vector(v['outgoing']) for v in p['line']['controls']], t)
    field('controlled ring actual Line length', lambda: f32(n['laser']['line']['length']), lambda: f32(p['line']['length']), t)

def compare_ring_receiver(original, actual, state, common, t, event_flags, fifo_for, *, unmatched_fifo=None):
    check, missing, field = (common[k] for k in ('check', 'missing', 'check_field'))
    f32, vector = common['f32'], common['vector']
    reference = common['reference']
    for name in ('hp', 'maxHp', 'pendingDamage', 'pendingFlags'):
        field('controlled ring receiver ' + name,
              lambda: original[name] if name == 'pendingFlags' else f32(original[name]),
              lambda: actual['damage'][name] if name == 'pendingFlags' else f32(actual['damage'][name] if name.startswith('pending') else actual[name]), t)
    for name in ('dead', 'visible', 'raw494', 'entityCollisionClass'):
        normal = integer if name == 'entityCollisionClass' else boolean
        field('controlled ring receiver ' + name, lambda: normal(original[name]), lambda: normal(actual[name]), t)
    field('controlled ring receiver champion', lambda: boolean(original['champion']), lambda: actual.get('champion') is not None, t)
    field('controlled ring receiver manager membership', lambda: boolean(original['inActiveManager']),
          lambda: {'kind': 'npc', 'id': actual['id']} in state['actorScheduler']['actors'], t)
    for name, port_name in (('npcState', 'state'), ('npcFrameAge', 'frameAge')):
        field('controlled ring receiver ' + name, lambda: integer(original[name]), lambda: integer(actual[port_name]), t)
    for name, port_name in (('position', 'presentedPosition'), ('velocity', 'velocity')):
        field('controlled ring receiver ' + name, lambda: vector(original[name]), lambda: vector(actual[port_name]), t)
    field('controlled ring receiver collision radius', lambda: f32(original['collisionRadius']), lambda: f32(actual['collisionRadius']), t)
    try:
        native_records, port_events = original['pendingDamageRecords'], actual['damage']['events']
        check('controlled ring ordered FIFO length', len(native_records), len(port_events), t)
        if unmatched_fifo is not None:
            unmatched_fifo(native_records, port_events, t)
        for record, event in zip(native_records, port_events):
            field('controlled ring FIFO amount', lambda: f32(record['amount']), lambda: f32(event['amount']), t)
            field('controlled ring FIFO flags', lambda: record['flags'], lambda: event_flags[event['kind']], t)
            field('controlled ring FIFO source type', lambda: record['sourceType'], lambda: event['source']['entityType'], t)
    except Exception as error:
        missing('controlled ring actual FIFO owners', t, error)
        return
    try:
        check('controlled ring ordered FIFO payload identities',
              fifo_for(original, 'native', state, t), fifo_for(actual, 'port', state, t), t)
    except Exception as error:
        missing('controlled ring ordered FIFO payload identity owners', t, error)

def observe_ring_fifo_hp(targets_by_side, source, state, counter, t, chains, owners, common, fifo_for, *, technology=False, receiver_type=None):
    check, missing, f32, identity, reference, run_flags = (common[k] for k in ('check', 'missing', 'f32', 'identity_tuple', 'reference', 'source_run_flags'))
    for side, own_targets in targets_by_side:
        for target in own_targets:
            try:
                own_identity = identity(target['identity'] if side == 'native' else reference(state, {'kind': 'npc', 'id': target['id']}))
                if receiver_type is not None:
                    assert own_identity[:3] == (receiver_type, 0, 0), 'Actual declared receiver kind/variant/subtype required'
                fifo = fifo_for(target, side, state, t)
                actual_counter = counter if side == 'native' else state['systemClock']['systemCounter']
                age = source['gameAge'] if side == 'native' else state['gameUpdateCount']
                assert type(actual_counter) is int and type(age) is int
            except Exception as error:
                missing('controlled ring ' + side + ' own receiver/FIFO/gate owner', t, error)
                continue
            key = (side, *own_identity)
            chain = chains.setdefault(key, {'side': side, 'identity': own_identity, 'rows': []})
            previous = chain['rows'][-1] if chain['rows'] else None
            if previous is not None and previous['presentation'] == t:
                continue
            owner = owners[side]
            current = {'presentation': t, 'counter': actual_counter, 'gameAge': age,
                       'hp': None, 'fifo': fifo, 'source': owner, 'applierKnown': False,
                       'emptyAppendObserved': False, 'pendingEmptyAppend': False, 'nextFullApplied': False}
            try:
                current['hp'] = f32(target['hp'])
            except Exception as error:
                missing('controlled ring ' + side + ' own receiver HP owner', t, error)
            try:
                current['applierKnown'] = (boolean(target['raw494']) is False and
                    (integer(target['entityFlags']) == 4 if side == 'native' else True) and
                    int(source['runFlags10ead8'] if side == 'native' else run_flags(state)) & 0x40 == 0)
            except Exception as error:
                missing('controlled ring ' + side + ' own receiver applier owner', t, error)
            dense = (previous is not None and previous['presentation'] + 1 == t and previous['counter'] + 1 == actual_counter)
            compatible = (owner is not None and owner['eligible'] and owner['owner'] is not None and
                owner['amount'] is not None and owner['amount'] > 0 and
                any(record == [owner['amount'], 8, 1, 0, owner['owner']] for record in fifo))
            same_source = (previous is not None and owner is not None and previous['source'] is not None and
                previous['source']['eligible'] and owner['eligible'] and
                previous['source']['identity'] == owner['identity'] and previous['source']['owner'] == owner['owner'] and
                previous['source']['amount'] == owner['amount'])
            current['emptyAppendObserved'] = bool(dense and same_source and previous['fifo'] == [] and compatible)
            current['pendingEmptyAppend'] = current['emptyAppendObserved'] or bool(
                dense and same_source and previous['pendingEmptyAppend'] and previous['fifo'] == fifo)
            chain['rows'].append(current)
            if not (dense and actual_counter % 2 == 1 and previous['pendingEmptyAppend'] and
                    age == previous['gameAge'] + 1 and previous['applierKnown'] and current['applierKnown'] and
                    previous['hp'] is not None and current['hp'] is not None):
                continue
            if fifo != []:
                missing('controlled ring ' + side + ' isolated following-Full FIFO drain', t, 'Queue is not empty at the following Full; drain versus refill is unobserved')
                continue
            try:
                assert previous['fifo'] and all(record is not None and record[0] > 0 and record[1] in (0, 8) for record in previous['fifo']), 'Complete supported positive FIFO required'
                expected = previous['hp']
                for record in previous['fifo']:
                    expected = f32(expected - record[0])
                if expected < 0:
                    if technology:
                        # Negative endpoints do not prove lifecycle callbacks.
                        check('controlled Technology ring ' + side + ' following-Full ordered negative HP endpoint', expected, current['hp'], t)
                        check('controlled Technology ring ' + side + ' actual dead31F endpoint', True, boolean(target['dead']), t)
                        current['lethalHpEndpointObserved'] = expected == current['hp'] and boolean(target['dead'])
                        current['nextFullApplied'] = current['lethalHpEndpointObserved']
                    missing('controlled ring ' + side + ' lethal subclass lifecycle', t, 'Negative-HP subclass callback and cleanup need independent actual witnesses')
                    continue
                check('controlled ring ' + side + ' following-Full ordered f32 HP application', expected, current['hp'], t)
                current['nextFullApplied'] = expected == current['hp']
            except Exception as error:
                missing('controlled ring ' + side + ' queue-to-HP owner', t, error)


def compare_contact(case, rows, common):
    """Consume actual paired rows without manufacturing a browser or events."""
    check, missing, field = (common[k] for k in ('check', 'missing', 'check_field'))
    f32, vector, reference, identity = (common[k] for k in ('f32', 'vector', 'reference', 'identity_tuple'))
    run_flags = common['source_run_flags']
    declaration = case['requires']['controlledBrimContact']
    result = {'schema': 'controlled-brim-ring-contact-v1', 'chains': [],
              'scope': 'Observed compatible ParentPlayer FIFO append and nonlethal following-Full HP application. Exact Laser callback attribution, actual collision delivery and lifecycle events are separate required witnesses.'}
    try:
        assert declaration['schema'] == result['schema']
        receiver = declaration['authoredReceiver']
        assert type(receiver['type']) is int and receiver['type'] in (14, 18)
        assert receiver['variant'] == receiver['subtype'] == 0
        assert (declaration['sourceEntityType'], declaration['sourceVariant'], declaration['damageFlags']) == (1, 0, 8)
        contract = case['requires']['sourceExpectedActorContract']
        assert (contract['primaryC08'], contract['family'], contract['variant'], contract['subtype'], contract['controlledBy329']) == (2, 7, 1, 1, True)
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
        return ring_source_owner(source, state, side, common, variant=1)

    chains = {}
    source_rows = {side: {'Full': 0, 'Fast': 0} for side in ('native', 'port')}
    event_flags = {'laser': 8, 'tear': 0, 'knife': 0, 'bomb': 4, 'fireContact': 2, 'roomEffect': 0}
    def fifo_for(target, side, state, t):
        return ring_fifo_for(target, side, state, t, common, event_flags)

    for captured in rows:
        t, source, state, counter = (captured[k] for k in ('presentation', 'source', 'state', 'nativeCounter'))
        owners, bodies = {}, {}
        for side in ('native', 'port'):
            try:
                owners[side], bodies[side] = source_owner(source, state, side)
                if owners[side] is not None:
                    actor, owner = bodies[side], owners[side]
                    try:
                        actual_parent = actor['linkedParent'] if side == 'native' else reference(state, actor['parent'])
                        expected_parent = source['players'][0]['owner'] if side == 'native' else reference(state, {'kind': 'player', 'id': 0})
                        assert actual_parent == expected_parent and identity(actual_parent)[:3] == (1, 0, 0), 'Actual controlled Laser ParentPlayer required'
                        owner['owner'] = identity(actual_parent)
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
                    if owner['eligible']:
                        actual_counter = counter if side == 'native' else state['systemClock']['systemCounter']
                        source_rows[side]['Full' if actual_counter % 2 else 'Fast'] += 1
            except Exception as error:
                owners[side], bodies[side] = None, None
                missing('controlled ring ' + side + ' actual C18 source owner', t, error)
        field('controlled ring actual C18 NULL relation',
              lambda: source['players'][0]['actorSlots']['c18'] is None,
              lambda: state['attackRuntime']['persistent'] is None, t)
        if any(owner is not None for owner in owners.values()):
            field('controlled ring actual C18 identity/seed',
                  lambda: owners['native']['identity'], lambda: owners['port']['identity'], t)
        if bodies['native'] is not None or bodies['port'] is not None:
            n, p = bodies['native'], bodies['port']
            compare_ring_body(n, p, owners, common, t)
            def native_width():
                value = n['laser']['line']
                assert value['sourceControlledLineWriterScope'] is True and n['laser']['firstUpdate'] is False
                assert len(value['points']) >= 2
                bits = value['widthRawBits']
                assert type(bits) is int and 0 <= bits <= 0xffffffff
                return f32(struct.unpack('<f', struct.pack('<I', bits))[0])
            field('controlled ring actual source-scoped Line width', native_width, lambda: f32(p['line']['width']), t)
        try:
            targets = [a for a in source['commonContactNpcs'] if a['identity']['kind'] == receiver['type']]
        except Exception as error:
            targets = []
            missing('controlled ring Native typed NPC14/18 receiver readset', t, error)
        try:
            ports = [a for a in state['room']['npcs'] if a['kind'] == receiver['type'] and not a['removed']]
        except Exception as error:
            ports = []
            missing('controlled ring port typed NPC14/18 receiver readset', t, error)
        field('controlled ring complete receiver identities',
              lambda: sorted(identity(a['identity']) for a in source['commonContactNpcs'] if a['identity']['kind'] == receiver['type']),
              lambda: sorted(identity(reference(state, {'kind': 'npc', 'id': a['id']})) for a in state['room']['npcs'] if a['kind'] == receiver['type'] and not a['removed']), t)
        for original in targets:
            try:
                actuals = [a for a in ports if (a['kind'], a['variant'], a['subtype'],
                    reference(state, {'kind': 'npc', 'id': a['id']})['sequenceId']) == identity(original['identity'])[:4]]
                assert len(actuals) == 1, 'One actual kind/variant/subtype/sequence receiver join required; no fitted position or ordinal alias'
                actual = actuals[0]
            except Exception as error:
                missing('controlled ring actual receiver correspondence', t, error)
                continue
            compare_ring_receiver(original, actual, state, common, t, event_flags, fifo_for)
        # Own queue/HP observations never depend on cross-game identity success.
        # The paired comparisons above retain strict identity failures separately.
        observe_ring_fifo_hp((('native', targets), ('port', ports)), source, state, counter, t, chains, owners, common, fifo_for)
    result['chains'] = list(chains.values())
    for side in ('native', 'port'):
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
    return result
