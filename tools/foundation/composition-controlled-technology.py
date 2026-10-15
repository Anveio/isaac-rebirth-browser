"""Explicit7/2/1 C18/Impact fields and each game's own public property history.

Cross-game identities, seeds and every numeric field stay strict. Own histories
use only that game's actual C18 and registered child; state transitions are not
callbacks, initialization order, RNG-demand or complete cleanup evidence.
"""
import math
import struct

SCHEMA = 'controlled-technology-properties-live-reapply-v1'


def integer(value):
    if type(value) is not int:
        raise ValueError('Actual integer owner required')
    return value


def boolean(value):
    if type(value) is not bool:
        raise ValueError('Actual boolean owner required')
    return value


class ControlledTechnologyComparer:
    def __init__(self, shared, case):
        self.b, self.case = shared, case
        self.declaration = case['requires']['controlledTechnologyFunctional']
        if self.declaration['schema'] != SCHEMA:
            raise ValueError('Explicit controlled Technology property marker required')
        contract = case['requires']['sourceExpectedActorContract']
        if tuple(contract[k] for k in ('primaryC08', 'family', 'variant', 'subtype', 'controlledBy329')) != (3, 7, 2, 1, True):
            raise ValueError('Exact controlled7/2/1 contract required')
        self.impacts = {}
        self.own = {side: dict(grants=[], queued=set(), priorCounts=None, last=None,
                    bodies=[], births={}, impacts={}, transitions=[], stages=set(), stageActors={})
                    for side in ('native', 'port')}

    def reference(self, state, ref):
        if ref is None or ref['kind'] != 'effect':
            return self.b['reference'](state, ref)
        # Old Brim reference() deliberately owns subtype1. This actual typed
        # subtype0 owner is independent; never patch a1-shaped view to fit it.
        rows = [a['impact'] for a in state['lasers']
                if a['impact'] is not None and a['impact']['id'] == ref['id']]
        identities = [i for i in state['actorScheduler']['identities'] if i['actor'] == ref]
        assert len(rows) == len(identities) == 1, 'Actual Impact reference must resolve uniquely'
        actual = rows[0]
        assert type(actual['sourceSubtype']) is int and actual['sourceSubtype'] == 0
        return dict(kind=1000, variant=50, subtype=actual['sourceSubtype'],
                    sequenceId=identities[0]['sequenceId'], spawnSeed=actual['spawnSeed'])

    def relation(self, label, native, port, state, t):
        b = self.b
        b['check_field'](label + ' NULL', lambda: native is None, lambda: port is None, t)
        if native is not None and port is not None:
            b['check_field'](label + ' identity/seed', lambda: b['identity_tuple'](native),
                             lambda: b['identity_tuple'](self.reference(state, port)), t)

    def body(self, original, actual, state, t):
        b, cf, f, v = self.b, self.b['check_field'], self.b['f32'], self.b['vector']
        ref = dict(kind='laser', id=actual['id'])
        cf('Controlled Technology independent identity/seed', lambda: b['identity_tuple'](original['identity']),
           lambda: b['identity_tuple'](self.reference(state, ref)), t)
        cf('Controlled Technology actual manager membership', lambda: original['inActiveManager'],
           lambda: ref in state['actorScheduler']['actors'], t)
        cf('Controlled Technology actual ParentPlayer', lambda: (1, 1),
           lambda: (original['linkedParent']['kind'], self.reference(state, actual['parent'])['kind']), t)
        for key in ('active', 'dead', 'visible', 'gridCollisionClass', 'entityCollisionClass', 'raw494'):
            cf('Controlled Technology ' + key, lambda: original[key], lambda: actual[key], t)
        for key in ('hp', 'maxHp', 'collisionDamage', 'collisionRadius', 'collisionMass'):
            cf('Controlled Technology ' + key + ' numeric Native owner', lambda: True,
               lambda: b['math'].isfinite(f(original[key])), t)
            cf('Controlled Technology ' + key + ' numeric port owner', lambda: True,
               lambda: b['math'].isfinite(f(actual[key])), t)
            cf('Controlled Technology ' + key, lambda: f(original[key]), lambda: f(actual[key]), t)
        for nk, pk in [('position', 'position'), ('savedPosition450', 'savedPosition'),
                       ('velocity', 'velocity'), ('renderOffset', 'renderOffset')]:
            cf('Controlled Technology ' + nk, lambda: v(original[nk]), lambda: v(actual[pk]), t)
        for nk, pk in [('bornGameAge', 'birthGameUpdate'), ('fast322', 'fast'),
                       ('genericFlags318', 'entityFlags')]:
            cf('Controlled Technology ' + nk, lambda: original[nk], lambda: actual[pk], t)
        for nk, pk in [('currentFriction470', 'currentFriction'), ('configuredFriction46c', 'configuredFriction')]:
            cf('Controlled Technology ' + nk, lambda: f(original[nk]), lambda: f(actual[pk]), t)
        cf('Controlled Technology actual RNG', lambda: original['actorRngWords7c0'],
           lambda: [actual['actorRng'][k] for k in ('state', 'rightA', 'left', 'rightB')], t)
        cf('Controlled Technology actual cached spawner', lambda: (original['cachedSpawner']['kind'], original['cachedSpawner']['variant']),
           lambda: (actual['spawnerType'], actual['spawnerVariant']), t)
        for nk, pk in [('permanentColor3a0', 'sourceColor'), ('currentColor2d0', 'currentColor')]:
            cf('Controlled Technology ' + nk, lambda: original[nk], lambda: b['color'](actual[pk]), t)
        for key in ('parentOffset', 'endpoint'):
            cf('Controlled Technology ' + key, lambda: v(original['laser'][key]), lambda: v(actual[key]), t)
        for key in ('angle', 'previousAngle', 'renderSortOffset', 'beamLength', 'maxDistance'):
            cf('Controlled Technology ' + key, lambda: f(original['laser'][key]), lambda: f(actual[key]), t)
        for key in ('remainingUpdates', 'firstUpdate', 'lineEnabled', 'contactDisabled', 'rotationEnabled', 'lineController'):
            cf('Controlled Technology ' + key, lambda: original['laser'][key], lambda: actual[key], t)
        for key in ('attackFlags', 'lineFlags'):
            cf('Controlled Technology ' + key, lambda: original['laser'][key], lambda: b['flags'](actual[key]), t)
        for key in ('animation', 'frame', 'playing', 'rotation'):
            cf('Controlled Technology Sprite ' + key, lambda: f(original['sprite'][key]) if key in ('frame', 'rotation') else original['sprite'][key],
               lambda: f(actual['sprite'][key]) if key in ('frame', 'rotation') else actual['sprite'][key], t)
        for key in ('scale', 'offset'):
            cf('Controlled Technology Sprite ' + key, lambda: v(original['sprite'][key]), lambda: v(actual['sprite'][key]), t)
        cf('Controlled Technology actual typed Line writer', lambda: True,
           lambda: original['laser']['controlledTechnologyLine']['sourceWriterScope'], t)
        for key in ('subdivisions', 'mode'):
            cf('Controlled Technology Line ' + key + ' Native integer owner', lambda: True,
               lambda: type(original['laser']['controlledTechnologyLine'][key]) is int and original['laser']['controlledTechnologyLine'][key] >= 0, t)
            cf('Controlled Technology Line ' + key + ' port integer owner', lambda: True,
               lambda: type(actual['line'][key]) is int and actual['line'][key] >= 0, t)
        for key in ('width', 'length', 'subdivisions', 'mode'):
            cf('Controlled Technology initialized Line ' + key,
               lambda: f(original['laser']['controlledTechnologyLine'][key]) if key in ('width', 'length') else original['laser']['controlledTechnologyLine'][key],
               lambda: f(actual['line'][key]) if key in ('width', 'length') else actual['line'][key], t)
        for key in ('origin', 'offset'):
            cf('Controlled Technology Line ' + key, lambda: v(original['laser']['line'][key]), lambda: v(actual['line'][key]), t)
        for key in ('points', 'priorPoints', 'history'):
            cf('Controlled Technology complete Line ' + key, lambda: [v(a) for a in original['laser']['line'][key]],
               lambda: [v(a) for a in actual['line'][key]], t)
        cf('Controlled Technology complete Line controls', lambda: [[f(x) for x in a] for a in original['laser']['line']['controls']],
           lambda: [v(a['position']) + v(a['incoming']) + v(a['outgoing']) for a in actual['line']['controls']], t)
        for label, native_read, port_read in [
                ('Parent498', lambda: original['linkedParent'], lambda: actual['parent']),
                ('Child4A0', lambda: original['laser']['child'], lambda: actual['child'])]:
            try:
                self.relation('Controlled Technology actual ' + label, native_read(), port_read(), state, t)
            except Exception as error:
                b['missing']('Controlled Technology actual ' + label + ' owner', t, error)
        try:
            child, port_child = original['laser']['child'], actual['child']
            if child is not None and port_child is not None:
                assert (child['kind'], child['variant'], child['subtype']) == (1000, 50, 0)
                assert port_child['kind'] == 'effect'
                key = b['identity_tuple'](child)
                target = dict(nativeIdentity=child, portRef=port_child,
                              nativeHostIdentity=original['identity'], portHostRef=ref)
                cf('Controlled Technology Impact relation stable', lambda: self.impacts.get(key, target), lambda: target, t)
                self.impacts.setdefault(key, target)
        except Exception as error:
            b['missing']('Controlled Technology parent/child actual ownership', t, error)

    def impact(self, original, actual, state, t):
        b, cf, f, v = self.b, self.b['check_field'], self.b['f32'], self.b['vector']
        ref = dict(kind='effect', id=actual['id'])
        cf('Controlled Technology Impact identity/seed', lambda: b['identity_tuple'](original['identity']),
           lambda: b['identity_tuple'](self.reference(state, ref)), t)
        cf('Controlled Technology Impact actual manager', lambda: original['inActiveManager'], lambda: ref in state['actorScheduler']['actors'], t)
        cf('Controlled Technology Impact actual dead31F', lambda: original['dead'], lambda: actual['dead'], t)
        cf('Controlled Technology Impact active/removed field relation', lambda: not original['active'], lambda: actual['removed'], t)
        for nk, pk in [('position', 'position'), ('savedPosition450', 'savedPosition'),
                       ('renderOffset', 'positionOffset')]:
            cf('Controlled Technology Impact ' + nk, lambda: v(original[nk]), lambda: v(actual[pk]), t)
        for nk, pk in [('animation', 'animation'), ('frame', 'animationClock'), ('playing', 'animationPlaying'), ('rotation', 'rotation')]:
            cf('Controlled Technology Impact Sprite ' + nk, lambda: f(original['sprite'][nk]) if nk in ('frame', 'rotation') else original['sprite'][nk],
               lambda: f(actual[pk]) if nk in ('frame', 'rotation') else actual[pk], t)
        cf('Controlled Technology Impact Sprite scale', lambda: v(original['sprite']['scale']), lambda: v(actual['scale']), t)
        cf('Controlled Technology Impact actual flip2FC', lambda: original['impact']['controlledTechnology']['spriteFlipX'], lambda: actual['spriteFlipX'], t)
        cf('Controlled Technology Impact actual raw814', lambda: f(original['impact']['controlledTechnology']['raw814']), lambda: f(actual['raw814']), t)
        cf('Controlled Technology Impact actual depth7F8', lambda: f(original['impact']['renderSortOffset']), lambda: f(actual['depth']), t)
        for nk, pk in [('permanentColor3a0', 'sourceColor'), ('currentColor2d0', 'currentColor')]:
            cf('Controlled Technology Impact ' + nk, lambda: original[nk], lambda: b['color'](actual[pk]), t)
        try:
            self.relation('Controlled Technology Impact Parent498', original['linkedParent'], actual['parent'], state, t)
        except Exception as error:
            b['missing']('Controlled Technology Impact actual Parent owner', t, error)

    def own_snapshot(self, side, source, state, native_player, t, native_counter):
        """One side's actual fields, never projected from the other game's actors."""
        b, history = self.b, self.own[side]
        cf, f = b['check_field'], b['f32']
        native = side == 'native'
        weapon = source['players'][0] if native else state['attackRuntime']
        counts = native_player['ownedCounts'] if native else state['playerItems']['ownedCounts']
        if len(counts) != 347 or any(type(x) is not int for x in counts):
            raise ValueError('Complete signed inventory required for each own history')
        queued = native_player['queuedId'] if native else state['playerItems']['queuedId']
        if queued is not None:
            history['queued'].add(integer(queued))
        counter = integer(native_counter if native else state['systemClock']['systemCounter'])
        pending = integer(native_player['cacheFlags'] if native else state['playerItems']['unevaluatedCacheFlags'])
        prior = history['priorCounts']
        if prior is not None:
            changed = [i for i in range(347) if counts[i] != prior[i]]
            for item in changed:
                if counts[item] == prior[item] + 1 and item in history['queued'] and queued is None:
                    history['grants'].append(dict(itemId=item, presentation=t,
                        beforeCount=prior[item], afterCount=counts[item],
                        scope='Actual previously queued collectible followed by own count increment; not an atomic Add callback'))
                    history['queued'].discard(item)
        history['priorCounts'] = list(counts)
        for index, stage in enumerate(self.case['requires']['actorStages']):
            expected = [0] * 347
            for entry in stage['ownedCounts']:
                expected[integer(entry['itemId'])] = integer(entry['count'])
            if expected == counts:
                history['stages'].add(index)
                contract = stage['sourceExpectedActorContract']
                family = tuple(integer(contract[k]) for k in ('family', 'variant', 'subtype'))
                stage_rows = history['stageActors'].setdefault(index, dict(observations=0,
                    fullCounterObservations=0, fastCounterObservations=0, identities=set()))
                if pending == 0 and queued is None:
                    if native:
                        slot = weapon['actorSlots']['c18']
                        candidates = [a for a in source['attacks'] if a['inActiveManager']
                            and self.b['identity_tuple'](a['identity'])[:3] == family
                            and (not contract['registeredC18DuringActualLifetime']
                                 or slot is not None and a['identity'] == slot['identity'])]
                        ids = [self.b['identity_tuple'](a['identity']) for a in candidates]
                    else:
                        refs = [ref for ref in state['actorScheduler']['actors']
                            if ref['kind'] in ('laser', 'tear')
                            and (not contract['registeredC18DuringActualLifetime'] or ref == weapon['persistent'])]
                        ids = [self.b['identity_tuple'](self.reference(state, ref)) for ref in refs]
                        ids = [key for key in ids if key[:3] == family]
                    if ids:
                        stage_rows['observations'] += 1
                        stage_rows['fullCounterObservations' if counter % 2 else 'fastCounterObservations'] += 1
                        stage_rows['identities'].update(ids)
        slot = weapon['actorSlots']['c18'] if native else weapon['persistent']
        if native:
            actors = [] if slot is None else [a for a in source['attacks'] if a['identity'] == slot['identity']]
        else:
            actors = [] if slot is None or slot['kind'] != 'laser' else [a for a in state['lasers'] if a['id'] == slot['id']]
        # Other real stages can own an ordinary Laser or controlled Tear. They
        # do not count toward the controlled7/2/1 body or property histories.
        if counts[68] <= 0 or counts[329] <= 0:
            return None
        pending = integer(native_player['cacheFlags'] if native else state['playerItems']['unevaluatedCacheFlags'])
        if pending != 0 or queued is not None:
            return None
        if len(actors) != 1:
            b['missing']('Controlled Technology ' + side + ' own C18 body', t, 'Actual stable7/2/1 C18 is absent or unresolved')
            return None
        actor = actors[0]
        identity = b['identity_tuple'](actor['identity'] if native else self.reference(state, slot))
        if identity[:3] != (7, 2, 1):
            b['missing']('Controlled Technology ' + side + ' own C18 family', t, 'Actual own C18 is not7/2/1')
            return None
        counter = integer(native_counter if native else state['systemClock']['systemCounter'])
        age = integer(source['gameAge'] if native else state['gameUpdateCount'])
        born_age = integer(actor['bornGameAge'] if native else actor['birthGameUpdate'])
        manager = boolean(actor['inActiveManager']) if native else slot in state['actorScheduler']['actors']
        laser = actor['laser'] if native else actor
        cache = (weapon['attackFlagsC98'], tuple(f(x) for x in weapon['cachedColorCCC']), f(native_player['cachedStats']['damage'])) if native else (
            b['flags'](state['weaponCache']['flagsC98']), tuple(b['color'](state['weaponCache']['colorCcc'])), f(state['playerStats']['damage']))
        props = (laser['attackFlags'] if native else b['flags'](laser['attackFlags']),
                 tuple(f(x) for x in actor['permanentColor3a0']) if native else tuple(b['color'](actor['sourceColor'])),
                 f(actor['collisionDamage']))
        line_flags = laser['lineFlags'] if native else b['flags'](laser['lineFlags'])
        primary = integer(weapon['primaryWeaponC08'] if native else state['weaponCache']['primary'])
        label = 'Controlled Technology ' + side + ' '
        for name, expected, actual in (
                ('actual C08', 3, primary), ('manager body', True, manager),
                ('initialized persistent TTL', 0, integer(laser['remainingUpdates'])),
                ('actual selected flags', '0', cache[0]),
                ('current 7E8/818 agree', props[0], line_flags),
                ('current cache flags copied', cache[0], props[0]),
                ('ParentPlayer kind', 1, integer(actor['linkedParent']['kind']) if native else integer(self.reference(state, actor['parent'])['kind']))):
            b['check'](label + name, expected, actual, t)
        cf(label + 'configured Line source writer', lambda: True,
           lambda: boolean(laser['controlledTechnologyLine']['sourceWriterScope']) if native else True, t)
        line = laser['controlledTechnologyLine'] if native else actor['line']
        for name, expected in (('width', 60.0), ('subdivisions', 50), ('mode', 1)):
            cf(label + 'configured Line ' + name, lambda expected=expected: expected,
               lambda name=name: f(line[name]) if name == 'width' else integer(line[name]), t)
        record = dict(presentation=t, counter=counter, identity=identity, ownReference=slot,
            counts=list(counts), cache=cache, properties=props, lineFlags=line_flags,
            manager=manager, active=boolean(actor['active']), dead=boolean(actor['dead']),
            fast=boolean(actor['fast322'] if native else actor['fast']),
            bornGameAge=born_age, gameAge=age, child=laser['child'] if native else actor['child'])
        history['bodies'].append(record)
        if born_age == age:
            history['births'].setdefault(identity, dict(presentation=t, identity=identity,
                scope='Actual own C18 birth-age state; initial Full-before-bind remains a separate callback obligation'))
        last = history['last']
        if (last is not None and last['presentation'] + 1 == t and last['counter'] + 1 == counter
                and last['identity'] == identity and last['manager'] and manager
                and last['active'] and record['active'] and not last['dead'] and not record['dead']
                and counts[4] > 0 and last['counts'] == counts
                and last['cache'] == cache and last['properties'] != props
                and last['properties'][2] != props[2] and props == cache):
            history['transitions'].append(dict(beforePresentation=last['presentation'], presentation=t,
                identity=identity, ownReference=slot, stableCacheOwners=cache,
                beforeProperties=last['properties'], afterProperties=props,
                scope='Dense same-own-C18 property state transition under stable current cache; not observed51D359 callback delivery'))
        history['last'] = record
        child = laser['child'] if native else actor['child']
        if child is None:
            b['missing'](label + 'actual registered Impact child', t, 'Missing canonical variant2 child; no Brim no-Impact alias')
        else:
            key = b['identity_tuple'](child if native else self.reference(state, child))
            if key[:3] != (1000, 50, 0):
                raise ValueError('Actual controlled Technology child must be1000/50/0')
            children = [a for a in source['impacts'] if a['identity'] == child] if native else [actor['impact']] if actor['impact'] is not None and actor['impact']['id'] == child['id'] else []
            if len(children) != 1:
                b['missing'](label + 'actual registered Impact body', t, 'Own registered child unresolved')
            else:
                impact = children[0]
                birth_age = impact['bornGameAge'] if native else impact.get('birthGameUpdate')
                # Missing port birth metadata is unknown. First observation,
                # scheduler registration or child age coincidence cannot fill it.
                entry = history['impacts'].setdefault(key, dict(identity=key, firstObservedPresentation=t,
                    observedBirthAge=False, rows=0, scope='Own canonical registered child state history'))
                entry['rows'] += 1
                if type(birth_age) is int and birth_age == age:
                    entry['observedBirthAge'] = True
                parent = impact['linkedParent'] if native else self.reference(state, impact['parent'])
                cf(label + 'Impact actual ParentLaser', lambda: identity, lambda: b['identity_tuple'](parent), t)
        return actor

    def snapshot(self, source, state, t, native_counter, native_player):
        actors = {}
        for side in ('native', 'port'):
            try:
                actors[side] = self.own_snapshot(side, source, state, native_player, t, native_counter)
            except Exception as error:
                self.b['missing']('Controlled Technology ' + side + ' own history execution', t, error)
        if actors.get('native') is None or actors.get('port') is None:
            return
        original, actual = actors['native'], actors['port']
        # These are the two actual registered C18 roles. Their sequence/seed
        # comparisons remain mandatory; no position/time/ordinal alias is used.
        self.body(original, actual, state, t)
        self.b['coverage']['controlledTechnologyBodies'] += 1
        native_child, port_child = original['laser']['child'], actual['child']
        if native_child is not None and port_child is not None:
            ns = [a for a in source['impacts'] if a['identity'] == native_child]
            ps = [a['impact'] for a in state['lasers'] if a['impact'] is not None and a['impact']['id'] == port_child['id']]
            if len(ns) == len(ps) == 1:
                self.impact(ns[0], ps[0], state, t)
            else:
                self.b['missing']('Controlled Technology exact registered child roles', t, 'Actual own Child4A0 body missing or ambiguous')

    def finish(self):
        result = dict(schema=SCHEMA, sides={}, scope='Each game own C18 and registered child histories are independent. Crossgame C18/child identities, seeds, geometry and every raw field remain strict. State transitions do not prove callbacks, initial order, RNG demand or complete cleanup.')
        b = self.b
        for side, history in self.own.items():
            rows = history['bodies']
            metrics = dict(births=len(history['births']), fullRows=sum(not r['fast'] for r in rows),
                fastRows=sum(r['fast'] for r in rows), impactBirths=sum(x['observedBirthAge'] for x in history['impacts'].values()),
                stableOwnerPropertyTransitions=len(history['transitions']))
            for required, key in (
                    ('minimumActualBirthsPerGame', 'births'),
                    ('minimumActualFullBodyRowsPerGame', 'fullRows'),
                    ('minimumActualFastBodyRowsPerGame', 'fastRows'),
                    ('minimumActualImpactBirthsPerGame', 'impactBirths'),
                    ('minimumSameIdentityPostGrantReapplyTransitionsPerGame', 'stableOwnerPropertyTransitions')):
                b['check']('Controlled Technology ' + side + ' required ' + key, True,
                    metrics[key] >= integer(self.declaration[required]) > 0, None)
            b['check']('Controlled Technology ' + side + ' actual ordered queued grants',
                self.case['requires']['orderedRealPedestalGrants'], [x['itemId'] for x in history['grants']], None)
            b['check']('Controlled Technology ' + side + ' observed declared inventory stages',
                list(range(len(self.case['requires']['actorStages']))), sorted(history['stages']), None)
            for field in ('actual initial Full-before-C18 callback order', 'actual51D359 reapply callback',
                          'actual SharedRaw/bounded2/bounded4 demand sequence', 'actual subtype1 no133 audio dispatch',
                          'actual Remove/BaseUninit completion and complete weak-reference lifecycle'):
                b['missing']('Controlled Technology ' + side + ' required ' + field, None,
                    'Typed state snapshots do not expose this exact event; no final-state or absence inference')
            stages = []
            for index in range(len(self.case['requires']['actorStages'])):
                observed = history['stageActors'].get(index)
                b['check']('Controlled Technology ' + side + ' actual stage' + str(index) + ' manager body observations',
                    True, observed is not None and observed['observations'] > 0, None)
                b['check']('Controlled Technology ' + side + ' actual stage' + str(index) + ' Full/Fast counter observations',
                    True, observed is not None and observed['fullCounterObservations'] > 0 and observed['fastCounterObservations'] > 0, None)
                if observed is not None:
                    stages.append(dict(stage=index, observations=observed['observations'],
                        fullCounterObservations=observed['fullCounterObservations'], fastCounterObservations=observed['fastCounterObservations'],
                        identities=sorted(observed['identities']), scope='Own exact actual count stage and scheduled family/C18 relation; counter observations alone are not fast322 or full body-field parity'))
            result['sides'][side] = dict(metrics=metrics, grants=history['grants'],
                births=list(history['births'].values()), impacts=list(history['impacts'].values()), stages=stages,
                stableOwnerPropertyTransitions=history['transitions'], observedInventoryStages=sorted(history['stages']),
                bodyRows=len(rows), stateHistory=rows)
        return result
