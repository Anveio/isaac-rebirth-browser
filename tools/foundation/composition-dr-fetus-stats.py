"""Each game's real52/4 grants and birth-owned Bomb properties, independently.

The ordinary sole52 comparer remains unchanged and its strict failures remain.
This marker never requires an old Bomb to survive a42-Full pickup with31-Full
fuse. Dense stable prior cache is needed to associate a birth with cache owners;
after-grant end state is not an earlier factory invocation witness.
"""
import math
import struct

SCHEMA = 'ordinary-dr-fetus52-cricket4-birth-properties-v1'


def integer(value):
    if type(value) is not int:
        raise ValueError('Actual integer field required')
    return value


def f32(value):
    if type(value) not in (int, float) or not math.isfinite(value):
        raise ValueError('Actual finite numeric owner required')
    return struct.unpack('<f', struct.pack('<f', value))[0]


class MixedDrFetusComparer:
    def __init__(self, shared, case):
        self.b, self.case = shared, case
        self.declaration = case['requires']['mixedDrFetusCricket4']
        if self.declaration['schema'] != SCHEMA:
            raise ValueError('Explicit mixed DrFetus/Cricket4 marker required')
        if self.declaration['lifetimeBoundary']['requireSameBombAcrossSecondGrant'] is not False:
            raise ValueError('No impossible same-Bomb pickup survivor requirement allowed')
        self.sides = {name: dict(last=None, pending=set(), grants=[], births={},
            firstStageBombBirths=set(), firstStageTearBirths=set(), mixedBirths=set(),
            mixedBodyRows=0, mixedFullRows=0, mixedFastRows=0, mixedFullBodyRows=0, mixedFastBodyRows=0, retainedRows=0,
            sameBombAcrossGrant=[], stageRows={}) for name in ('native', 'port')}

    def port_identity(self, state, actor, kind):
        if kind != 'bomb':
            return self.b['identity_tuple'](self.b['reference'](state, dict(kind=kind, id=actor['id'])))
        ref = dict(kind='bomb', id=integer(actor['id']))
        owners = [x for x in state['actorScheduler']['identities'] if x['actor'] == ref]
        if len(owners) != 1:
            raise ValueError('Actual Bomb requires one canonical scheduler identity')
        return (4, integer(actor['variant']), integer(actor['shot']['subtype']),
            integer(owners[0]['sequenceId']), integer(actor['spawnSeed']))

    def snapshot_side(self, side, source, state, player, t, native_counter):
        native = side == 'native'
        h, b = self.sides[side], self.b
        label = 'Mixed DrFetus/Cricket4 ' + side + ' '
        counts = player['ownedCounts'] if native else state['playerItems']['ownedCounts']
        if len(counts) != 347 or any(type(v) is not int for v in counts):
            raise ValueError('Actual complete signed inventory required')
        queued = player['queuedId'] if native else state['playerItems']['queuedId']
        if queued is not None:
            h['pending'].add(integer(queued))
        counter = integer(native_counter if native else state['systemClock']['systemCounter'])
        age = integer(source['gameAge'] if native else state['gameUpdateCount'])
        pending = integer(player['cacheFlags'] if native else state['playerItems']['unevaluatedCacheFlags'])
        primary = integer(source['players'][0]['primaryWeaponC08'] if native else state['weaponCache']['primary'])
        c88 = f32(player['cachedStats']['damage'] if native else state['playerStats']['damage'])
        weapon = source['players'][0] if native else state['weaponCache']
        cache = (weapon['attackFlagsC98'], tuple(f32(x) for x in weapon['cachedColorCA0']), c88) if native else (
            b['flags'](weapon['flagsC98']), tuple(b['color'](weapon['colorCa0'])), c88)
        old = h['last']
        if old is not None:
            for item in (52, 4):
                if (old['counts'][item] == 0 and counts[item] == 1 and item in h['pending']
                        and queued is None and pending == 0):
                    h['grants'].append(dict(itemId=item, presentation=t,
                        scope='Actual own queued config then count0→1/cache-complete state; not atomic Add ordering'))
                    h['pending'].discard(item)
        stage = None
        for index, declared in enumerate(self.case['requires']['actorStages']):
            expected = [0] * 347
            for entry in declared['ownedCounts']:
                expected[integer(entry['itemId'])] = integer(entry['count'])
            if counts == expected:
                stage = index
                h['stageRows'][index] = h['stageRows'].get(index, 0) + 1
        record = dict(presentation=t, counter=counter, age=age, counts=list(counts),
            queued=queued, pending=pending, primary=primary, cache=cache)
        ready = queued is None and pending == 0 and stage is not None
        if ready and counts[52] > 0:
            for name, expected, actual in (
                    ('actual C08', 5, primary),
                    ('known normalIsaac', 0, integer(source['players'][0]['fetusCaller']['playerTypeBfc']) if native else integer(state['weaponAdmission']['sourcePlayerType'])),
                    ('actual selected flags', '0', cache[0]),
                    ('actual empty TEMP', [], player['temporaryCollectibles'] if native else state['playerItems']['temporaryCollectibles']),
                    ('actual empty trinkets', [0, 0], source['players'][0]['fetusCaller']['trinketSlotsE14'] if native else state['playerItems']['trinketSlots'])):
                b['check'](label + name, expected, actual, t)
        actors = [a for a in source['fetusBombs'] if integer(a['shotMarker7e8']) == 1] if native else [a for a in state['bombs'] if a['shot'] is not None and a['shot']['shotMarker'] is True]
        for actor in actors:
            identity = b['identity_tuple'](actor['identity']) if native else self.port_identity(state, actor, 'bomb')
            if identity[:3] != (4, 0, 0):
                b['missing'](label + 'actual selected Bomb family', t, 'Actual marked Bomb outside4/0/0')
                continue
            damage = f32(actor['explosionDamage7d8'] if native else actor['shot']['explosionDamage'])
            flags = actor['attackFlags7e0'] if native else b['flags'](actor['shot']['attackFlags'])
            color = tuple(f32(x) for x in actor['sourceColor3a0']) if native else tuple(b['color'](actor['shot']['sourceColor']))
            properties = (flags, color, damage)
            born_age = integer(actor['bornGameAge'] if native else actor['birthGameUpdate'])
            manager = actor['inActiveManager'] if native else dict(kind='bomb', id=actor['id']) in state['actorScheduler']['actors']
            if type(manager) is not bool:
                raise ValueError('Actual manager-membership bool required')
            if ready and counts[52] > 0 and counts[4] > 0 and manager:
                h['mixedBodyRows'] += 1
                h['mixedFullRows' if counter % 2 else 'mixedFastRows'] += 1
                actual_fast = actor['fast322'] if native else actor.get('fast')
                if type(actual_fast) is bool:
                    h['mixedFastBodyRows' if actual_fast else 'mixedFullBodyRows'] += 1
                else:
                    b['missing'](label + 'actual rawfast322 owner', t, 'Missing actual port body owner; counter parity is diagnostic only')
            if born_age == age and identity not in h['births'] and manager:
                witness = dict(identity=identity, presentation=t, bornGameAge=born_age,
                    counts=list(counts), properties=properties, currentCache=cache,
                    actualPriorCacheQualified=False,
                    scope='Actual own Bomb birth-age state; snapshots do not prove kind5/SharedRaw/callback order')
                h['births'][identity] = witness
                if ready and stage == 0 and counts[52] > 0:
                    h['firstStageBombBirths'].add(identity)
                if ready and counts[52] > 0 and counts[4] > 0:
                    h['mixedBirths'].add(identity)
                stable = (ready and old is not None and old['presentation'] + 1 == t
                    and old['counter'] + 1 == counter and old['counts'] == counts
                    and old['queued'] is None and old['pending'] == 0
                    and old['primary'] == primary == 5 and old['cache'] == cache)
                b['check'](label + 'birth has actual dense stable prior cache', True, stable, t)
                if stable:
                    witness['actualPriorCacheQualified'] = True
                    witness['priorPresentation'] = old['presentation']
                    for name, expected, actual in (
                            ('born damage uses actual kind5C88×3', f32(f32(3.0) * cache[2]), damage),
                            ('born flags use actual kind5C98', cache[0], flags),
                            ('born color uses actual kind5CA0', cache[1], color)):
                        b['check'](label + name, expected, actual, t)
                b['check_field'](label + 'actual born fuse30', lambda: 30,
                    lambda: integer(actor['fuse7d4'] if native else actor['motion']['fuseUpdatesRemaining']), t)
                if not native:
                    b['check_field'](label + 'actual born fullUpdates0', lambda: 0,
                        lambda: integer(actor['motion']['fullUpdates']), t)
                parent = actor['linkedParent'] if native else b['reference'](state, actor['shot']['parent'])
                b['check_field'](label + 'actual born ParentPlayer', lambda: 1, lambda: integer(parent['kind']), t)
            born = h['births'].get(identity)
            if born is not None and t > born['presentation']:
                b['check'](label + 'same own Bomb retains birth-owned flags/color/damage', born['properties'], properties, t)
                h['retainedRows'] += 1
                if counts != born['counts']:
                    h['sameBombAcrossGrant'].append(dict(identity=identity, presentation=t,
                        birthPresentation=born['presentation'], properties=properties,
                        scope='Optional actually observed same-own-Bomb across current inventory change; never a required survivor'))
        if ready and stage == 0 and counts[4] > 0 and counts[52] == 0:
            tears = [a for a in source['attacks'] if b['identity_tuple'](a['identity'])[:3] == (2, 0, 0)
                     and a['inActiveManager']] if native else [a for a in state['tears']
                     if dict(kind='tear', id=a['id']) in state['actorScheduler']['actors']]
            for actor in tears:
                birth = actor['bornGameAge'] if native else actor.get('birthGameUpdate')
                if type(birth) is not int:
                    b['missing'](label + 'actual first-stage Tear birth owner', t,
                        'Canonical Tear has no exposed birthGameUpdate; age or first observation is not a birth timestamp')
                    continue
                if birth == age:
                    identity = b['identity_tuple'](actor['identity']) if native else self.port_identity(state, actor, 'tear')
                    h['firstStageTearBirths'].add(identity)
        h['last'] = record

    def snapshot(self, source, state, t, native_counter, native_player):
        for side in ('native', 'port'):
            try:
                self.snapshot_side(side, source, state, native_player, t, native_counter)
            except Exception as error:
                self.b['missing']('Mixed DrFetus/Cricket4 ' + side + ' own execution', t, error)

    def finish(self):
        result = dict(schema=SCHEMA, sides={}, scope='Independent actual queued grant and per-own-Bomb birth/lifetime properties. Existing sole52 comparer and strict crossgame failures remain unchanged. No required old Bomb survival across42-Full pickup and no cache-after-Add substituted for prior birth owner.')
        for side, h in self.sides.items():
            prefix = 'Mixed DrFetus/Cricket4 ' + side + ' '
            self.b['check'](prefix + 'actual ordered queued grants', self.declaration['orderedRealPedestalGrants'],
                [x['itemId'] for x in h['grants']], None)
            for field, count in (
                    ('minimumActualGrantsPerSide', len(h['grants'])),
                    ('minimumActualMixedBombBirthsPerSide', len(h['mixedBirths'])),
                    ('minimumActualFirstStageBombBirthsPerSide', len(h['firstStageBombBirths'])),
                    ('minimumActualFirstStageRegularTearBirthsPerSide', len(h['firstStageTearBirths'])),
                    ('minimumActualMixedFullBodyRowsPerSide', h['mixedFullBodyRows']),
                    ('minimumActualMixedFastBodyRowsPerSide', h['mixedFastBodyRows'])):
                minimum = integer(self.declaration[field])
                self.b['check'](prefix + 'required ' + field, True, count >= minimum >= 0, None)
            self.b['check'](prefix + 'actual per-own-Bomb property retention rows', True, h['retainedRows'] > 0, None)
            for field in ('actual kind5-beforeSharedRaw factory event', 'actual C74/C78/BB0 firing callback order',
                          'actual priorDD4 factory argument', 'actual detonation/contact/applier/Remove/Uninit events'):
                self.b['missing'](prefix + 'required ' + field, None,
                    'State snapshots do not expose this exact event or instruction order')
            result['sides'][side] = dict(grants=h['grants'], births=list(h['births'].values()),
                firstStageBombBirths=sorted(h['firstStageBombBirths']), firstStageTearBirths=sorted(h['firstStageTearBirths']),
                mixedBirths=sorted(h['mixedBirths']), mixedBodyRows=h['mixedBodyRows'],
                mixedFullCounterRows=h['mixedFullRows'], mixedFastCounterRows=h['mixedFastRows'],
                mixedActualFullBodyRows=h['mixedFullBodyRows'], mixedActualFastBodyRows=h['mixedFastBodyRows'],
                propertyRetentionRows=h['retainedRows'], optionalSameBombAcrossGrant=h['sameBombAcrossGrant'],
                observedInventoryStageRows=h['stageRows'])
        return result
