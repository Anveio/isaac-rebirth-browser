"""Actual C18 Bomb4 controller roles and each game's public birth/lifetime states.

Type4 C18 is not filtered by dead/variant/subtype on reuse. New neutral birth is
separate4/0/0. No recall/live-property writer, Brim/Technology body, clock fit,
coordinate alias or callback proof is supplied by these state observations.
"""

SCHEMA = 'controlled-dr-fetus52-bomb-c18-v1'


def integer(value):
    if type(value) is not int:
        raise ValueError('Actual integer owner required')
    return value


def boolean(value):
    if type(value) is not bool:
        raise ValueError('Actual boolean owner required')
    return value


class ControlledDrFetusComparer:
    def __init__(self, shared, case):
        self.b, self.case = shared, case
        self.declaration = case['requires']['controlledDrFetusFunctional']
        if self.declaration['schema'] != SCHEMA:
            raise ValueError('Explicit controlled DrFetus marker required')
        contract = case['requires']['sourceExpectedActorContract']
        if tuple(contract[k] for k in ('primaryC08', 'family', 'controlledBy329')) != (5, 4, True):
            raise ValueError('Exact C08=5+329 Bomb contract required')
        if self.declaration['noSameBombStatGrantWitnessRequired'] is not True:
            raise ValueError('No impossible same-Bomb42-pickup survivor requirement')
        self.own = {side: dict(last=None, queued=set(), grants=[], births={}, bodies=[],
            stages=set(), releasedAlive=[], expiry=[], slotCleanup=[], absent=[], seenManager=set())
            for side in ('native', 'port')}

    def reference(self, state, ref):
        if ref is None or ref['kind'] != 'bomb':
            return self.b['reference'](state, ref)
        actors = [a for a in state['bombs'] if integer(a['id']) == integer(ref['id'])]
        identities = [a for a in state['actorScheduler']['identities'] if a['actor'] == ref]
        if len(actors) != 1 or len(identities) != 1:
            raise ValueError('Actual Bomb reference must resolve one canonical store and scheduler owner')
        actor = actors[0]
        return dict(kind=4, variant=integer(actor['variant']), subtype=integer(actor['shot']['subtype']),
            sequenceId=integer(identities[0]['sequenceId']), spawnSeed=integer(actor['spawnSeed']))

    def public_keys(self, presentation):
        if type(presentation) is not int or presentation <= self.case['initialPhaseOffset']:
            return None
        end = self.case['initialPhaseOffset']
        for action in self.case['actions']:
            end += integer(action['presentations'])
            if presentation <= end:
                return list(action['keys'])
        raise ValueError('Actual own public presentation outside complete declared tape')

    def own_snapshot(self, side, source, state, player, t, native_counter, gameplay_presentation):
        b, h, native = self.b, self.own[side], side == 'native'
        f, label = b['f32'], 'Controlled DrFetus ' + side + ' '
        weapon = source['players'][0] if native else state['attackRuntime']
        counts = player['ownedCounts'] if native else state['playerItems']['ownedCounts']
        if len(counts) != 347 or any(type(x) is not int for x in counts):
            raise ValueError('Actual complete signed inventory required')
        counter = integer(native_counter if native else state['systemClock']['systemCounter'])
        age = integer(source['gameAge'] if native else state['gameUpdateCount'])
        queued = player['queuedId'] if native else state['playerItems']['queuedId']
        pending = integer(player['cacheFlags'] if native else state['playerItems']['unevaluatedCacheFlags'])
        if queued is not None:
            h['queued'].add(integer(queued))
        last = h['last']
        if last is not None:
            for item in self.case['requires']['orderedRealPedestalGrants']:
                if last['counts'][item] == 0 and counts[item] == 1 and item in h['queued'] and queued is None and pending == 0:
                    h['grants'].append(dict(itemId=item, presentation=t,
                        scope='Own actual queued config then count0→1/cache-complete state, not atomic Add'))
                    h['queued'].discard(item)
        for index, stage in enumerate(self.case['requires']['actorStages']):
            expected = [0] * 347
            for entry in stage['ownedCounts']:
                expected[integer(entry['itemId'])] = integer(entry['count'])
            if expected == counts:
                h['stages'].add(index)
        cache = (weapon['attackFlagsC98'], tuple(f(x) for x in weapon['cachedColorCA0']), f(player['cachedStats']['damage'])) if native else (
            b['flags'](state['weaponCache']['flagsC98']), tuple(b['color'](state['weaponCache']['colorCa0'])), f(state['playerStats']['damage']))
        slot = weapon['actorSlots']['c18'] if native else weapon['persistent']
        slot_identity = None if slot is None else b['identity_tuple'](slot['identity'] if native else self.reference(state, slot))
        current = dict(presentation=t, counter=counter, counts=list(counts), queued=queued,
            pending=pending, slotIdentity=slot_identity, body=None, cache=cache)
        if last is not None and last['slotIdentity'] is not None and last['slotIdentity'][0] == 4 and slot is None:
            known = True if native else boolean(state['weaponAdmission']['c18IsKnownEmpty'])
            b['check'](label + 'known actual primary-slot NULL state', True, known, t)
            if known:
                h['slotCleanup'].append(dict(presentation=t, priorIdentity=last['slotIdentity'],
                    scope='Actual previously referenced Bomb C18 now knownNULL; not an Uninit/completion or complete incoming-reference proof'))
        if counts[52] <= 0 or counts[329] <= 0 or pending != 0 or queued is not None or slot_identity is None or slot_identity[0] != 4:
            h['last'] = current
            return None
        # Accept the actual Bomb4 role regardless of dead, variant or subtype.
        # Selected fresh birth checks below do not narrow this reuse predicate.
        actors = [a for a in source['fetusBombs'] if b['identity_tuple'](a['identity']) == slot_identity] if native else [a for a in state['bombs'] if a['id'] == slot['id']]
        if len(actors) != 1:
            raise ValueError('Actual own C18 Bomb4 body absent or ambiguous')
        actor = actors[0]
        shot = actor if native else actor['shot']
        manager = boolean(actor['inActiveManager']) if native else slot in state['actorScheduler']['actors']
        active = boolean(actor['active'] if native else shot['active'])
        dead = boolean(actor['dead'] if native else shot['dead'])
        born = integer(actor['bornGameAge'] if native else actor['birthGameUpdate'])
        fuse = integer(actor['fuse7d4'] if native else actor['motion']['fuseUpdatesRemaining'])
        props = (actor['attackFlags7e0'] if native else b['flags'](shot['attackFlags']),
                 tuple(f(x) for x in actor['sourceColor3a0']) if native else tuple(b['color'](shot['sourceColor'])),
                 f(actor['explosionDamage7d8'] if native else shot['explosionDamage']))
        primary = integer(weapon['primaryWeaponC08'] if native else state['weaponCache']['primary'])
        b['check'](label + 'actual selected C08', 5, primary, t)
        b['check'](label + 'actual neutral typed Child4A0', None,
            actor['controlledChild4a0'] if native else shot['child'] if boolean(shot['childKnown']) else 'unknown', t)
        # Raw phase owner is independent of counter parity. The port has no
        # exposed Bomb322 owner yet; unknown does not earn Full/Fast coverage.
        actual_fast = boolean(actor['fast322']) if native else (
            boolean(actor['fast']) if 'fast' in actor and actor['fast'] is not None else None)
        body = dict(presentation=t, counter=counter, gameAge=age, identity=slot_identity,
            ownReference=slot, manager=manager, active=active, dead=dead, bornGameAge=born, actualFast322=actual_fast,
            fuse=fuse, properties=props, cache=cache, publicGameplayPresentation=gameplay_presentation,
            publicKeys=self.public_keys(gameplay_presentation),
            position=b['vector'](actor['position448'] if native else actor['motion']['presentedPosition']),
            velocity=b['vector'](actor['velocity464'] if native else actor['motion']['velocity']))
        h['bodies'].append(body);current['body'] = body
        if manager:
            h['seenManager'].add(slot_identity)
        if body['publicKeys'] is not None and not any(k in ('Up','Down','Left','Right') for k in body['publicKeys']) and active and not dead and manager:
            h['releasedAlive'].append(dict(presentation=t, identity=slot_identity, fuse=fuse,
                position=body['position'], velocity=body['velocity'],
                scope='Own alive Bomb state during actual own public released-input interval; not an observed no-force/no-recall callback'))
        if born == age and slot_identity not in h['births'] and manager:
            stable = last is not None and last['presentation'] + 1 == t and last['counter'] + 1 == counter and last['counts'] == counts and last['queued'] is None and last['pending'] == 0
            b['check'](label + 'actual fresh selected birth family', (4,0,0), slot_identity[:3], t)
            b['check'](label + 'actual born fuse30', 30, fuse, t)
            b['check'](label + 'birth has dense prior completed inventory', True, stable, t)
            # Current cache matching an earlier birth requires a separately
            # observed stable prior owner; do not infer it from after-Add state.
            prior_cache = last.get('cache') if last is not None else None
            cache_stable = stable and prior_cache == cache
            b['check'](label + 'actual stable prior birth cache', True, cache_stable, t)
            if cache_stable:
                b['check'](label + 'birth kind5 damage/currentC88×3', f(f(3.0)*cache[2]), props[2], t)
                b['check'](label + 'birth kind5 flags/currentC98', cache[0], props[0], t)
                b['check'](label + 'birth kind5 color/currentCA0', cache[1], props[1], t)
            h['births'][slot_identity] = dict(identity=slot_identity,presentation=t,counts=list(counts),
                properties=props,cache=cache,stablePriorBirthCache=cache_stable,
                scope='Own actual birth-age/fuse state; factory/Raw/C18-bind/no-immediateFull order still unobserved')
        old_birth = h['births'].get(slot_identity)
        if old_birth is not None and t > old_birth['presentation']:
            b['check'](label + 'same own Bomb retains born properties without live reapply', old_birth['properties'], props, t)
        if last is not None and last['body'] is not None and last['body']['identity'] == slot_identity and last['body']['fuse'] > 0 and fuse == 0:
            h['expiry'].append(dict(presentation=t,identity=slot_identity,dead=dead,
                scope='Own observed positive→zero fuse state; not detonation callback proof'))
        h['last'] = current
        b['coverage']['nativeFamilyKind4' if native else 'browserFamilyKind4'] += 1
        return actor

    def body(self, original, actual, state, t):
        b, f, v, cf = self.b, self.b['f32'], self.b['vector'], self.b['check_field']
        shot, motion = actual['shot'], actual['motion']
        cf('Controlled DrFetus independent C18 identity/seed', lambda:b['identity_tuple'](original['identity']),
            lambda:b['identity_tuple'](self.reference(state,dict(kind='bomb',id=actual['id']))),t)
        for name,left,right in (
                ('actual active31E',lambda:boolean(original['active']),lambda:boolean(shot['active'])),
                ('actual dead31F',lambda:boolean(original['dead']),lambda:boolean(shot['dead'])),
                ('actual visible31D',lambda:boolean(original['visible']),lambda:boolean(actual['visible'])),
                ('rawfast322 owner',lambda:boolean(original['fast322']),lambda:boolean(actual['fast'])),
                ('position448',lambda:v(original['position448']),lambda:v(motion['presentedPosition'])),
                ('physicalV464',lambda:v(original['velocity464']),lambda:v(motion['velocity'])),
                ('fuse7D4',lambda:integer(original['fuse7d4']),lambda:integer(motion['fuseUpdatesRemaining'])),
                ('bornGameAge',lambda:integer(original['bornGameAge']),lambda:integer(actual['birthGameUpdate'])),
                ('damage7D8',lambda:f(original['explosionDamage7d8']),lambda:f(shot['explosionDamage'])),
                ('countdown7DC',lambda:integer(original['playerContactCountdown7dc']),lambda:integer(shot['playerContactCountdown'])),
                ('flags7E0',lambda:original['attackFlags7e0'],lambda:b['flags'](shot['attackFlags'])),
                ('marker7E8',lambda:integer(original['shotMarker7e8']),lambda:int(boolean(shot['shotMarker']))),
                ('currentFriction470',lambda:f(original['currentFriction470']),lambda:f(actual['friction'])),
                ('gridClass340',lambda:integer(original['gridCollisionClass340']),lambda:integer(shot['gridCollisionClass'])),
                ('gridCollided34C',lambda:boolean(original['gridCollided34c']),lambda:boolean(shot['gridCollided'])),
                ('permanentColor3A0',lambda:[f(x) for x in original['sourceColor3a0']],lambda:b['color'](shot['sourceColor'])),
                ('currentColor2D0',lambda:[f(x) for x in original['currentColor2d0']],lambda:b['color'](shot['currentColor'])),
                ('actorRNG7C0',lambda:original['actorRngWords7c0'],lambda:[integer(shot['actorRng'][k]) for k in ('state','rightA','left','rightB')]),
                ('Parent498 NULL',lambda:original['linkedParent'] is None,lambda:shot['parent'] is None),
                ('Parent498 identity/seed',lambda:b['identity_tuple'](original['linkedParent']),lambda:b['identity_tuple'](self.reference(state,shot['parent']))),
                ('cachedSpawner1C20',lambda:(original['cachedSpawner']['kind'],original['cachedSpawner']['variant']),lambda:(shot['spawnerType'],shot['spawnerVariant'])),
                ('Child4A0 NULL',lambda:original['controlledChild4a0'] is None,lambda:shot['child'] is None if boolean(shot['childKnown']) else 'unknown')):
            cf('Controlled DrFetus '+name,left,right,t)
        if boolean(original['fast322']):
            cf('Controlled DrFetus raw saved450 Fast owner',
                lambda:v(original['savedPosition450']),lambda:v(actual['savedPosition450']),t)

    def snapshot(self,source,state,t,native_counter,native_player,*,native_gameplay_presentation,port_gameplay_presentation):
        actors={}
        for side,gp in (('native',native_gameplay_presentation),('port',port_gameplay_presentation)):
            try:actors[side]=self.own_snapshot(side,source,state,native_player,t,native_counter,gp)
            except Exception as error:self.b['missing']('Controlled DrFetus '+side+' own execution',t,error)
        if actors.get('native') is not None and actors.get('port') is not None:
            self.body(actors['native'],actors['port'],state,t)

    def finish(self):
        result=dict(schema=SCHEMA,sides={},scope='Actual own C18 Bomb roles, public-input intervals and born property/lifetime states. Strict crossgame identities/seeds/geometry remain. No recall/live-reapply/TTL0/Impact or callback/RNG/lifecycle inference.')
        for side,h in self.own.items():
            prefix='Controlled DrFetus '+side+' '
            for name,actual in (('minimumActualControlledBombBirthsPerGame',len(h['births'])),
                    ('minimumActualFullBodyRowsPerGame',sum(r['actualFast322'] is False for r in h['bodies'])),
                    ('minimumActualFastBodyRowsPerGame',sum(r['actualFast322'] is True for r in h['bodies'])),
                    ('minimumReleasedAliveOwnBombRowsPerGame',len(h['releasedAlive'])),
                    ('minimumOwnBornPropertyCohortsPerGame',len({r['properties'] for r in h['births'].values()})),
                    ('minimumActualFuseExpiryStateChainsPerGame',len(h['expiry'])),
                    ('minimumOwnKnownC18CleanupStatesPerGame',len(h['slotCleanup']))):
                self.b['check'](prefix+'required '+name,True,actual>=integer(self.declaration[name])>0,None)
            self.b['check'](prefix+'actual ordered queued grants',self.case['requires']['orderedRealPedestalGrants'],[x['itemId'] for x in h['grants']],None)
            self.b['check'](prefix+'actual all declared inventory stages',list(range(len(self.case['requires']['actorStages']))),sorted(h['stages']),None)
            for field in ('actual type4 dead/variant/subtype reuse callback','actual clearC18-beforeRemove order',
                    'actual held force/currentC7C/friction callback','actual released no-recall/no-reapply callback',
                    'actual kind5-beforeRaw/C18bind/no-initialFull order','actual detonation/applier/Remove/Uninit-completion/full weak-reference lifecycle'):
                self.b['missing'](prefix+'required '+field,None,'Typed public states do not expose this exact event or instruction order')
            result['sides'][side]=dict(grants=h['grants'],births=list(h['births'].values()),
                observedInventoryStages=sorted(h['stages']),bodyRows=len(h['bodies']),bodyHistory=h['bodies'],
                observedBornPropertyCohorts=len({r['properties'] for r in h['births'].values()}),
                bornInventoryCohortDiagnostic=len({tuple(r['counts']) for r in h['births'].values()}),
                actualFullBodyRows=sum(r['actualFast322'] is False for r in h['bodies']),
                actualFastBodyRows=sum(r['actualFast322'] is True for r in h['bodies']),
                unknownRawPhaseRows=sum(r['actualFast322'] is None for r in h['bodies']),
                counterParityDiagnostic=dict(odd=sum(r['counter']%2==1 for r in h['bodies']),even=sum(r['counter']%2==0 for r in h['bodies']),scope='Absolute own system counter parity only; not raw Bomb322 proof'),
                releasedAliveStates=h['releasedAlive'],fuseExpiryStates=h['expiry'],knownC18NullStates=h['slotCleanup'])
        return result
