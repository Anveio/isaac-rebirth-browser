"""Ordinary7/2/0 manager roles. No C18 identity, fitted gate or pose defaults."""


class TechnologyComparer:
    def __init__(self, base, case):
        self.b, self.case = base, case
        contract = case['requires']['sourceExpectedActorContract']
        assert (contract['family'], contract['variant'], contract['subtype']) == (7, 2, 0)
        assert contract['registeredC18DuringActualLifetime'] is False
        assert case['requires']['ordinaryTechnologyFunctional']['schema'] == 'ordinary-technology-properties-lifetime-v1'
        self.native_seen, self.port_seen, self.pairs = set(), set(), {}
        self.lives, self.impacts, self.impact_lives = {}, {}, {}

    def retained(self, rows):
        assert len(rows) == 1, 'Exactly one retained actual owner required; absence is not a NULL field'
        return rows[0]

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
        cf('Technology independent identity/seed', lambda: b['identity_tuple'](original['identity']),
           lambda: b['identity_tuple'](self.reference(state, ref)), t)
        cf('Technology actual manager membership', lambda: original['inActiveManager'],
           lambda: ref in state['actorScheduler']['actors'], t)
        cf('Technology actual ParentPlayer', lambda: (1, 1),
           lambda: (original['linkedParent']['kind'], self.reference(state, actual['parent'])['kind']), t)
        for key in ('active', 'dead', 'visible', 'gridCollisionClass', 'entityCollisionClass', 'raw494'):
            cf('Technology ' + key, lambda: original[key], lambda: actual[key], t)
        for key in ('hp', 'maxHp', 'collisionDamage', 'collisionRadius', 'collisionMass'):
            cf('Technology ' + key + ' numeric Native owner', lambda: True,
               lambda: b['math'].isfinite(f(original[key])), t)
            cf('Technology ' + key + ' numeric port owner', lambda: True,
               lambda: b['math'].isfinite(f(actual[key])), t)
            cf('Technology ' + key, lambda: f(original[key]), lambda: f(actual[key]), t)
        for nk, pk in [('position', 'position'), ('savedPosition450', 'savedPosition'),
                       ('velocity', 'velocity'), ('renderOffset', 'renderOffset')]:
            cf('Technology ' + nk, lambda: v(original[nk]), lambda: v(actual[pk]), t)
        for nk, pk in [('bornGameAge', 'birthGameUpdate'), ('fast322', 'fast'),
                       ('genericFlags318', 'entityFlags')]:
            cf('Technology ' + nk, lambda: original[nk], lambda: actual[pk], t)
        for nk, pk in [('currentFriction470', 'currentFriction'), ('configuredFriction46c', 'configuredFriction')]:
            cf('Technology ' + nk, lambda: f(original[nk]), lambda: f(actual[pk]), t)
        cf('Technology actual RNG', lambda: original['actorRngWords7c0'],
           lambda: [actual['actorRng'][k] for k in ('state', 'rightA', 'left', 'rightB')], t)
        cf('Technology actual cached spawner', lambda: (original['cachedSpawner']['kind'], original['cachedSpawner']['variant']),
           lambda: (actual['spawnerType'], actual['spawnerVariant']), t)
        for nk, pk in [('permanentColor3a0', 'sourceColor'), ('currentColor2d0', 'currentColor')]:
            cf('Technology ' + nk, lambda: original[nk], lambda: b['color'](actual[pk]), t)
        for key in ('parentOffset', 'endpoint'):
            cf('Technology ' + key, lambda: v(original['laser'][key]), lambda: v(actual[key]), t)
        for key in ('angle', 'previousAngle', 'renderSortOffset', 'beamLength', 'maxDistance'):
            cf('Technology ' + key, lambda: f(original['laser'][key]), lambda: f(actual[key]), t)
        for key in ('remainingUpdates', 'firstUpdate', 'lineEnabled', 'contactDisabled', 'rotationEnabled', 'lineController'):
            cf('Technology ' + key, lambda: original['laser'][key], lambda: actual[key], t)
        for key in ('attackFlags', 'lineFlags'):
            cf('Technology ' + key, lambda: original['laser'][key], lambda: b['flags'](actual[key]), t)
        for key in ('animation', 'frame', 'playing', 'rotation'):
            cf('Technology Sprite ' + key, lambda: f(original['sprite'][key]) if key in ('frame', 'rotation') else original['sprite'][key],
               lambda: f(actual['sprite'][key]) if key in ('frame', 'rotation') else actual['sprite'][key], t)
        for key in ('scale', 'offset'):
            cf('Technology Sprite ' + key, lambda: v(original['sprite'][key]), lambda: v(actual['sprite'][key]), t)
        cf('Technology actual typed Line writer', lambda: True,
           lambda: original['laser']['technologyLine']['sourceWriterScope'], t)
        for key in ('subdivisions', 'mode'):
            cf('Technology Line ' + key + ' Native integer owner', lambda: True,
               lambda: type(original['laser']['technologyLine'][key]) is int and original['laser']['technologyLine'][key] >= 0, t)
            cf('Technology Line ' + key + ' port integer owner', lambda: True,
               lambda: type(actual['line'][key]) is int and actual['line'][key] >= 0, t)
        for key in ('width', 'length', 'subdivisions', 'mode'):
            cf('Technology initialized Line ' + key,
               lambda: f(original['laser']['technologyLine'][key]) if key in ('width', 'length') else original['laser']['technologyLine'][key],
               lambda: f(actual['line'][key]) if key in ('width', 'length') else actual['line'][key], t)
        for key in ('origin', 'offset'):
            cf('Technology Line ' + key, lambda: v(original['laser']['line'][key]), lambda: v(actual['line'][key]), t)
        for key in ('points', 'priorPoints', 'history'):
            cf('Technology complete Line ' + key, lambda: [v(a) for a in original['laser']['line'][key]],
               lambda: [v(a) for a in actual['line'][key]], t)
        cf('Technology complete Line controls', lambda: [[f(x) for x in a] for a in original['laser']['line']['controls']],
           lambda: [v(a['position']) + v(a['incoming']) + v(a['outgoing']) for a in actual['line']['controls']], t)
        for label, native_read, port_read in [
                ('Parent498', lambda: original['linkedParent'], lambda: actual['parent']),
                ('Child4A0', lambda: original['laser']['child'], lambda: actual['child'])]:
            try:
                self.relation('Technology actual ' + label, native_read(), port_read(), state, t)
            except Exception as error:
                b['missing']('Technology actual ' + label + ' owner', t, error)
        try:
            child, port_child = original['laser']['child'], actual['child']
            if child is not None and port_child is not None:
                assert (child['kind'], child['variant'], child['subtype']) == (1000, 50, 0)
                assert port_child['kind'] == 'effect'
                key = b['identity_tuple'](child)
                target = dict(nativeIdentity=child, portRef=port_child,
                              nativeHostIdentity=original['identity'], portHostRef=ref)
                cf('Technology Impact relation stable', lambda: self.impacts.get(key, target), lambda: target, t)
                self.impacts.setdefault(key, target)
        except Exception as error:
            b['missing']('Technology parent/child actual ownership', t, error)

    def impact(self, original, actual, state, t):
        b, cf, f, v = self.b, self.b['check_field'], self.b['f32'], self.b['vector']
        ref = dict(kind='effect', id=actual['id'])
        cf('Technology Impact identity/seed', lambda: b['identity_tuple'](original['identity']),
           lambda: b['identity_tuple'](self.reference(state, ref)), t)
        cf('Technology Impact actual manager', lambda: original['inActiveManager'], lambda: ref in state['actorScheduler']['actors'], t)
        cf('Technology Impact actual dead31F', lambda: original['dead'], lambda: actual['dead'], t)
        cf('Technology Impact actual Remove state', lambda: not original['active'], lambda: actual['removed'], t)
        for nk, pk in [('position', 'position'), ('savedPosition450', 'savedPosition'),
                       ('renderOffset', 'positionOffset')]:
            cf('Technology Impact ' + nk, lambda: v(original[nk]), lambda: v(actual[pk]), t)
        for nk, pk in [('animation', 'animation'), ('frame', 'animationClock'), ('playing', 'animationPlaying'), ('rotation', 'rotation')]:
            cf('Technology Impact Sprite ' + nk, lambda: f(original['sprite'][nk]) if nk in ('frame', 'rotation') else original['sprite'][nk],
               lambda: f(actual[pk]) if nk in ('frame', 'rotation') else actual[pk], t)
        cf('Technology Impact Sprite scale', lambda: v(original['sprite']['scale']), lambda: v(actual['scale']), t)
        cf('Technology Impact actual flip2FC', lambda: original['impact']['technology']['spriteFlipX'], lambda: actual['spriteFlipX'], t)
        cf('Technology Impact actual raw814', lambda: f(original['impact']['technology']['raw814']), lambda: f(actual['raw814']), t)
        cf('Technology Impact actual depth7F8', lambda: f(original['impact']['renderSortOffset']), lambda: f(actual['depth']), t)
        for nk, pk in [('permanentColor3a0', 'sourceColor'), ('currentColor2d0', 'currentColor')]:
            cf('Technology Impact ' + nk, lambda: original[nk], lambda: b['color'](actual[pk]), t)
        try:
            self.relation('Technology Impact Parent498', original['linkedParent'], actual['parent'], state, t)
        except Exception as error:
            b['missing']('Technology Impact actual Parent owner', t, error)

    def snapshot(self, source, state, t, native_counter):
        b, cf, ident, cov = self.b, self.b['check_field'], self.b['identity_tuple'], self.b['coverage']
        for name, read in [('native', lambda: source['players'][0]['actorSlots']['c18']),
                           ('port', lambda: state['attackRuntime']['persistent'])]:
            cf('Technology ' + name + ' actual C18 remains NULL', lambda: None, read, t)
        cf('Technology port C18 known-empty owner', lambda: True, lambda: state['weaponAdmission']['c18IsKnownEmpty'], t)
        if source['players'][0]['initialized3074'] is True or state['player']['initialized3074'] is True:
            cf('Technology actual initialized C08', lambda: (3, 3),
               lambda: (source['players'][0]['primaryWeaponC08'], state['weaponCache']['primary']), t)
            expected_counts = [0] * 347; expected_counts[68] = 1
            cf('Technology declared sole68 port counts', lambda: expected_counts, lambda: state['playerItems']['ownedCounts'], t)
        ns = sorted([a for a in source['attacks'] if (a['identity']['kind'], a['identity']['variant'], a['identity']['subtype']) == (7, 2, 0)], key=lambda a: a['identity']['sequenceId'])
        ps = sorted([a for a in state['lasers'] if (a['entityType'], a['variant'], a['subtype']) == (7, 2, 0)
                     and dict(kind='laser', id=a['id']) in state['actorScheduler']['actors']],
                    key=lambda a: self.reference(state, dict(kind='laser', id=a['id']))['sequenceId'])
        nb, pb = [a for a in ns if ident(a['identity']) not in self.native_seen], [a for a in ps if a['id'] not in self.port_seen]
        self.native_seen.update(ident(a['identity']) for a in nb); self.port_seen.update(a['id'] for a in pb)
        cov['technologyNativeBirths'] += len(nb); cov['technologyPortBirths'] += len(pb)
        b['check']('Technology same-gate manager births', len(nb), len(pb), t)
        if len(nb) == len(pb):
            for n, p in zip(nb, pb):
                key = ident(n['identity']); self.pairs[key] = p['id']
                self.lives[key] = dict(nativeIdentity=n['identity'], portId=p['id'], birthPresentation=t,
                    joinBasis='same actual gate and own manager birth order; sequence/seed remain independent required comparisons', rows=[], observedRetirement=None)
        for n in ns:
            key = ident(n['identity']); p = next((a for a in state['lasers'] if a['id'] == self.pairs.get(key)), None)
            if p is None:
                b['missing']('Technology actual same-gate manager correspondence', t, 'No fitted identity/seed/time fallback'); continue
            self.body(n, p, state, t)
            cov['technologyBodies'] += 1
            if n['fast322'] is False and p['fast'] is False: cov['technologyFullBodies'] += 1
            if n['fast322'] is True and p['fast'] is True: cov['technologyFastBodies'] += 1
            self.lives[key]['rows'].append(dict(presentation=t, nativeCounter=native_counter,
                nativeTTL=n['laser']['remainingUpdates'], portTTL=p['remainingUpdates'],
                nativeDead=n['dead'], portDead=p['dead'], nativeActive=n['active'], portActive=p['active']))
        for key, life in self.lives.items():
            n = next((a for a in ns if ident(a['identity']) == key), None)
            p = next((a for a in state['lasers'] if a['id'] == life['portId']), None)
            ref = dict(kind='laser', id=life['portId'])
            if (life['observedRetirement'] is None and t > life['birthPresentation'] and n is None
                    and ref not in state['actorScheduler']['actors']
                    and native_counter % 2 == state['systemClock']['systemCounter'] % 2 == 1):
                cf('Technology retained port Uninit owner', lambda: True, lambda: p['uninitialized'] is True, t)
                port_uninit = p is not None and p.get('uninitialized') is True
                positive = any(r['nativeTTL'] > 0 and r['portTTL'] > 0 for r in life['rows'])
                zero_dead = any(r['nativeTTL'] == r['portTTL'] == 0 and r['nativeDead'] is True and r['portDead'] is True for r in life['rows'])
                native_refs = [a['linkedParent'] for a in [*source['attacks'], *source['impacts']]] + [a['laser']['child'] for a in source['attacks'] if a['identity']['kind'] == 7]
                native_refs += [slot['identity'] if slot is not None else None
                    for player in source['players'] for slot in (player['actorSlots'][name] for name in ('c18', 'c20', 'c28', 'c30'))]
                port_refs = [value for a in state['lasers'] for value in (a['parent'], a['child'])] + [a['impact']['parent'] for a in state['lasers'] if a['impact'] is not None]
                port_refs += [state['attackRuntime']['persistent'], *state['attackRuntime']['secondary']]
                clear = not any(value == life['nativeIdentity'] for value in native_refs) and ref not in port_refs
                life['observedRetirement'] = dict(presentation=t, positiveTTLObserved=positive,
                    zeroDeadObserved=zero_dead, portUninitObserved=port_uninit, incomingKnownReferencesClear=clear,
                    passed=positive and zero_dead and clear and port_uninit,
                    scope='Following actual Full manager absence with required retained port Uninit and observed known references. C18NULL is not cleanup credit; no absent Native object Uninit, complete weak-list or Remove event is invented.')
        for key, pair in self.impacts.items():
            ns_impact = [a for a in source['impacts'] if ident(a['identity']) == key]
            ps_impact = [a['impact'] for a in state['lasers'] if a['impact'] is not None and a['impact']['id'] == pair['portRef']['id']]
            cf('Technology Impact actual manager presence', lambda: any(a['inActiveManager'] for a in ns_impact), lambda: pair['portRef'] in state['actorScheduler']['actors'], t)
            if ns_impact:
                if len(ns_impact) != 1 or len(ps_impact) != 1:
                    b['missing']('Technology Impact actual body resolution', t, 'Missing/ambiguous own child'); continue
                self.impact(ns_impact[0], ps_impact[0], state, t); cov['technologyImpactBodies'] += 1
            life = self.impact_lives.setdefault(key, dict(nativeIdentity=pair['nativeIdentity'],
                portRef=pair['portRef'], firstPresentation=t, observedCleanup=None))
            if (life['observedCleanup'] is None and t > life['firstPresentation']
                    and not any(a['inActiveManager'] for a in ns_impact)
                    and pair['portRef'] not in state['actorScheduler']['actors']
                    and native_counter % 2 == state['systemClock']['systemCounter'] % 2 == 1):
                native_hosts = [a for a in source['attacks'] if ident(a['identity']) == ident(pair['nativeHostIdentity'])]
                port_hosts = [a for a in state['lasers'] if dict(kind='laser', id=a['id']) == pair['portHostRef']]
                # Missing retained owners cannot supply synthetic NULL or Uninit.
                cf('Technology Impact following-Full retained Native own Parent498 clear', lambda: None,
                   lambda: self.retained(ns_impact)['linkedParent'], t)
                cf('Technology Impact following-Full retained port own Parent498 clear', lambda: None,
                   lambda: self.retained(ps_impact)['parent'], t)
                cf('Technology Impact following-Full retained Native host Child4A0 clears same target', lambda: False,
                   lambda: self.retained(native_hosts)['laser']['child'] == pair['nativeIdentity'], t)
                cf('Technology Impact following-Full retained port host Child4A0 clears same target', lambda: False,
                   lambda: self.retained(port_hosts)['child'] == pair['portRef'], t)
                life['observedCleanup'] = dict(presentation=t,
                    nativeOwnParentObserved=len(ns_impact) == 1,
                    portOwnParentObserved=len(ps_impact) == 1,
                    nativeHostChildObserved=len(native_hosts) == 1,
                    portHostChildObserved=len(port_hosts) == 1,
                    scope='Following actual Full manager absence with independent retained own-Parent and exact-target host-Child checks; absence supplies no NULL or Native Uninit event.')

    def finish(self):
        b, cov = self.b, self.b['coverage']
        req = self.case['requires']['ordinaryTechnologyFunctional']
        cov['technologyImpactPairedRelations'] = len(self.impacts)
        for key, minimum in [('technologyNativeBirths', req['minimumActualBirthsPerGame']),
                             ('technologyPortBirths', req['minimumActualBirthsPerGame']),
                             ('technologyFullBodies', req['minimumActualFullBodyRowsPerGame']),
                             ('technologyFastBodies', req['minimumActualFastBodyRowsPerGame']),
                             ('technologyImpactPairedRelations', req['impact']['minimumActualBirthsPerGame'])]:
            b['check']('required actual ' + key, True, cov[key] >= minimum, None)
        retired = sum(bool(life['observedRetirement'] and life['observedRetirement']['passed']) for life in self.lives.values())
        b['check']('required Technology observed natural retirement', True,
                   retired >= req['minimumSameIdentityNaturalRetirementsPerGame'], None)
        b['check']('required Technology Impact following-Full cleanup observations', True,
                   any(cleanup is not None and all(cleanup[key] is True for key in (
                       'nativeOwnParentObserved', 'portOwnParentObserved', 'nativeHostChildObserved', 'portHostChildObserved'))
                       for cleanup in (life['observedCleanup'] for life in self.impact_lives.values())), None)
        for label in ('explicit Native Remove/Uninit events', 'actual contact FIFO/source-window and next-Full damage',
                      'DOM/render/eye-pose/raw814-render evidence'):
            b['missing']('Technology separately required ' + label, None,
                         'This version does not supply that actual evidence; body/absence is not a substitute')
        return dict(schema=1, status='incomplete', lifetimes=list(self.lives.values()),
                    impactRelations=list(self.impacts.values()), impactLifetimes=list(self.impact_lives.values()), sourceScope='ordinary-technology-properties-lifetime-v1')
