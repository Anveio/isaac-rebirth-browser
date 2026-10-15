"""Optional actual Bomb4 owners for the ordinary Dr. Fetus public contract."""

SCHEMA = 'ordinary-dr-fetus52-bomb-shots-v1'


def integer(value):
    if type(value) is not int:
        raise ValueError('Actual integer Bomb owner required')
    return value


def boolean(value):
    if type(value) is not bool:
        raise ValueError('Actual boolean Bomb owner required')
    return value


def begin(case):
    declaration = case['requires'].get('ordinaryDrFetus')
    if declaration is None:
        return None
    contract = case['requires']['sourceExpectedActorContract']
    if (declaration['schema'] != SCHEMA or
            tuple(contract[key] for key in ('primaryC08', 'family', 'variant', 'subtype', 'controlledBy329')) != (5, 4, 0, 0, False)):
        raise ValueError('Exact ordinary Dr. Fetus Bomb declaration required')
    return dict(declaration=declaration, bodies=0, full=set(), fast=set(), births=set(),
                seen={}, retirement=[], contacts={}, queued=0, hp_changes=0, grants=[], prior_counts=None, queued52=False)


def bomb_identity(state, actor):
    """Actual Bomb handle, stored subtype/seed and its own scheduler identity."""
    ref = {'kind': 'bomb', 'id': integer(actor['id'])}
    matches = [entry for entry in state['actorScheduler']['identities'] if entry['actor'] == ref]
    if len(matches) != 1:
        raise ValueError('Bomb requires one actual scheduler identity')
    return (4, integer(actor['variant']), integer(actor['shot']['subtype']),
            integer(matches[0]['sequenceId']), integer(actor['spawnSeed']))


def snapshot(context, source, state, t, native_counter, shared, native_player):
    """Join exact own identities; no position, age or ordinal correspondence fit."""
    check, field, missing = shared['check'], shared['check_field'], shared['missing']
    f32, vector, color, flags = (shared[name] for name in ('f32', 'vector', 'color', 'flags'))
    identity, reference = shared['identity_tuple'], shared['reference']
    native_counts = native_player['ownedCounts']
    port_counts = state['playerItems']['ownedCounts']
    if (len(native_counts) != 347 or len(port_counts) != 347
            or any(type(x) is not int for x in [*native_counts, *port_counts])):
        raise ValueError('Both actual complete signed inventories required')
    check('DrFetus actual signed owned counts', native_counts, port_counts, t)
    expected = [0] * 347
    if native_counts[52] != 0 or port_counts[52] != 0:
        expected[52] = 1
    check('DrFetus declared actual pre/post-grant inventories', (expected, expected), (native_counts, port_counts), t)
    if native_player['queuedId'] == state['playerItems']['queuedId'] == 52:
        context['queued52'] = True
    prior = context['prior_counts']
    if (prior is not None and prior[0] == prior[1] == [0] * 347 and native_counts == port_counts
            and native_counts[52] == 1 and all(count == 0 for i, count in enumerate(native_counts) if i != 52)
            and context['queued52'] and native_player['queuedId'] is None
            and state['playerItems']['queuedId'] is None):
        context['grants'].append(t)
    context['prior_counts'] = (list(native_counts), list(port_counts))
    if native_counts[52] > 0 or port_counts[52] > 0:
        expected = [0] * 347
        expected[52] = 1
        check('DrFetus actual sole52 inventory domain', (expected, expected), (native_counts, port_counts), t)
        field('DrFetus actual normal-Isaac type', lambda: (0, 0),
              lambda: (integer(source['players'][0]['fetusCaller']['playerTypeBfc']),
                       integer(state['weaponAdmission']['sourcePlayerType'])), t)
        field('DrFetus actual cache completion', lambda: (0, 0),
              lambda: (integer(native_player['cacheFlags']),
                       integer(state['playerItems']['unevaluatedCacheFlags'])), t)
        field('DrFetus actual empty TEMP', lambda: ([], []),
              lambda: (native_player['temporaryCollectibles'], state['playerItems']['temporaryCollectibles']), t)
        field('DrFetus actual empty trinkets', lambda: ([0, 0], [0, 0]),
              lambda: (source['players'][0]['fetusCaller']['trinketSlotsE14'], state['playerItems']['trinketSlots']), t)
    port_counter = integer(state['systemClock']['systemCounter'])
    field('DrFetus actual absolute system counter', lambda: integer(native_counter), lambda: port_counter, t)
    field('DrFetus actual Full/Fast counter parity', lambda: integer(native_counter) % 2, lambda: port_counter % 2, t)
    originals = [actor for actor in source['fetusBombs'] if integer(actor['shotMarker7e8']) == 1]
    shared['coverage']['nativeFamilyKind4'] += len(originals)
    actuals = [actor for actor in state['bombs'] if actor.get('shot') is not None and boolean(actor['shot']['shotMarker'])]
    shared['coverage']['browserFamilyKind4'] += len(actuals)
    field('DrFetus complete shot Bomb identities',
          lambda: sorted(identity(actor['identity']) for actor in originals),
          lambda: sorted(bomb_identity(state, actor) for actor in actuals), t)
    for original in originals:
        key = identity(original['identity'])
        if key[:3] != (4, 0, 0):
            missing('DrFetus source Bomb role', t, 'Marked Bomb is outside actual4/0/0')
            continue
        matches = [actor for actor in actuals if bomb_identity(state, actor) == key]
        if len(matches) != 1:
            missing('DrFetus exact Bomb body correspondence', t, 'No unique actual type/variant/subtype/sequence/seed match')
            continue
        actual = matches[0]
        shot, motion = actual['shot'], actual['motion']
        ref = {'kind': 'bomb', 'id': actual['id']}
        for name, expected, observed in (
                ('manager membership', lambda: boolean(original['inActiveManager']), lambda: ref in state['actorScheduler']['actors']),
                ('birth game age', lambda: integer(original['bornGameAge']), lambda: integer(actual['birthGameUpdate'])),
                ('raw448 / explicit presented position', lambda: vector(original['position448']), lambda: vector(motion['presentedPosition'])),
                ('physical velocity464', lambda: vector(original['velocity464']), lambda: vector(motion['velocity'])),
                ('fuse7D4', lambda: integer(original['fuse7d4']), lambda: integer(motion['fuseUpdatesRemaining'])),
                ('born fuse30 configuration', lambda: 30, lambda: integer(motion['initialFuseUpdates'])),
                ('explosion damage7D8', lambda: f32(original['explosionDamage7d8']), lambda: f32(shot['explosionDamage'])),
                ('player contact countdown7DC', lambda: integer(original['playerContactCountdown7dc']), lambda: integer(shot['playerContactCountdown'])),
                ('attack flags7E0', lambda: original['attackFlags7e0'], lambda: flags(shot['attackFlags'])),
                ('shot marker7E8', lambda: integer(original['shotMarker7e8']), lambda: int(boolean(shot['shotMarker']))),
                ('current friction470', lambda: f32(original['currentFriction470']), lambda: f32(actual['friction'])),
                ('grid class340', lambda: integer(original['gridCollisionClass340']), lambda: integer(shot['gridCollisionClass'])),
                ('actual grid collided34C', lambda: boolean(original['gridCollided34c']), lambda: boolean(shot['gridCollided'])),
                ('source color3A0', lambda: [f32(x) for x in original['sourceColor3a0']], lambda: color(shot['sourceColor'])),
                ('current color2D0', lambda: [f32(x) for x in original['currentColor2d0']], lambda: color(shot['currentColor'])),
                ('cached spawner1C/20', lambda: (integer(original['cachedSpawner']['kind']), integer(original['cachedSpawner']['variant'])),
                 lambda: (integer(shot['spawnerType']), integer(shot['spawnerVariant']))),
                ('actual Parent498', lambda: identity(original['linkedParent']), lambda: identity(reference(state, shot['parent']))),
                ('persistent entity RNG7C0', lambda: [integer(x) for x in original['actorRngWords7c0']],
                 lambda: [integer(shot['actorRng'][name]) for name in ('state', 'rightA', 'left', 'rightB')])):
            field('DrFetus ' + name, expected, observed, t)
        # These physical owners have no current Bomb DTO counterpart. Missing
        # stays missing; lifetime, counter parity and presented P are no aliases.
        for name in ('visible', 'active', 'dead', 'fast322'):
            field('DrFetus actual ' + name + ' owner', lambda: boolean(original[name]),
                  lambda: boolean(actual[name]), t)
        if boolean(original['fast322']):
            field('DrFetus raw saved450 owner', lambda: vector(original['savedPosition450']),
                  lambda: vector(actual['savedPosition450']), t)
        context['bodies'] += 1
        context['full' if native_counter % 2 else 'fast'].add((t, key))
        if (integer(original['bornGameAge']) == integer(actual['birthGameUpdate']) == integer(source['gameAge'])
                and integer(original['fuse7d4']) == integer(motion['fuseUpdatesRemaining']) == 30
                and integer(motion['fullUpdates']) == 0):
            context['births'].add(key)
        context['seen'][key] = dict(lastPresentation=t, nativeCounter=native_counter, portCounter=port_counter,
                                    ref=ref, nativeManager=original['inActiveManager'])
    # Preserve a manager-absence state witness separately from any callback.
    native_keys = {identity(actor['identity']) for actor in originals}
    for key, prior in context['seen'].items():
        if (key not in native_keys and prior['nativeManager'] is True and t > prior['lastPresentation']
                and native_counter % 2 == port_counter % 2 == 1
                and prior['ref'] not in state['actorScheduler']['actors']
                and not any(row['identity'] == key for row in context['retirement'])):
            context['retirement'].append(dict(identity=key, presentation=t,
                scope='Previously observed own Bomb absent/unscheduled at Full; not Remove or Uninit evidence'))
    # The actual FIFO carries Bomb4 as its source. ParentPlayer is not substituted.
    for original in source['contactNpcs']:
        key = identity(original['identity'])
        matches = [actor for actor in state['room']['npcs'] if actor['kind'] == key[0]
                   and identity(reference(state, {'kind': 'npc', 'id': actor['id']})) == key]
        if len(matches) != 1:
            missing('DrFetus actual receiver identity', t, 'No unique retained receiver sequence/seed match')
            continue
        actual = matches[0]
        for name in ('hp', 'maxHp'):
            field('DrFetus receiver ' + name, lambda: f32(original[name]), lambda: f32(actual[name]), t)
        records, events = original['pendingDamageRecords'], actual['damage']['events']
        check('DrFetus complete receiver FIFO length', len(records), len(events), t)
        for record, event in zip(records, events):
            field('DrFetus FIFO amount', lambda: f32(record['amount']), lambda: f32(event['amount']), t)
            field('DrFetus FIFO flags', lambda: integer(record['flags']),
                  lambda: {'bomb': 4, 'tear': 0, 'knife': 0, 'laser': 8, 'fireContact': 2, 'roomEffect': 0}[event['kind']], t)
            field('DrFetus FIFO source type', lambda: integer(record['sourceType']), lambda: integer(event['source']['entityType']), t)
            def attacker():
                if event['kind'] != 'bomb' or integer(event['source']['entityType']) != 4:
                    raise ValueError('An actual Bomb4 event owner is required')
                actors = [actor for actor in state['bombs'] if actor['id'] == event['source']['id']]
                if len(actors) != 1:
                    raise ValueError('Actual FIFO Bomb handle unresolved')
                return bomb_identity(state, actors[0])
            if integer(record['sourceType']) == 4 or event['kind'] == 'bomb':
                field('DrFetus FIFO source variant', lambda: integer(record['sourceVariant']), lambda: attacker()[1], t)
                field('DrFetus FIFO actual Bomb identity', lambda: identity(record['actualAttacker']), attacker, t)
        try:
            native_hp, port_hp = f32(original['hp']), f32(actual['hp'])
        except Exception:
            # Independent numeric HP checks already failed. Keep known FIFO
            # fields and other receivers; None here is an unknown, not equality.
            native_hp, port_hp = None, None
        row = dict(presentation=t, nativeCounter=native_counter, portCounter=port_counter,
                   nativeHp=native_hp, portHp=port_hp,
                   nativeGameAge=integer(source['gameAge']), portGameAge=integer(state['gameUpdateCount']),
                   nativeBombQueue=any(integer(r['sourceType']) == 4 and integer(r['flags']) == 4
                       and r['actualAttacker'] is not None and identity(r['actualAttacker']) in context['seen'] for r in records),
                   portBombQueue=any(e['kind'] == 'bomb' and integer(e['source']['entityType']) == 4
                       and any(old['ref'] == {'kind': 'bomb', 'id': e['source']['id']}
                               for old in context['seen'].values()) for e in events))
        chain = context['contacts'].setdefault(key, [])
        if chain and chain[-1]['presentation'] == t:
            continue
        if row['nativeBombQueue'] and row['portBombQueue']:
            context['queued'] += 1
        if (chain and native_counter % 2 == port_counter % 2 == 1
                and row['nativeGameAge'] == chain[-1]['nativeGameAge'] + 1
                and row['portGameAge'] == chain[-1]['portGameAge'] + 1
                and chain[-1]['nativeBombQueue'] and chain[-1]['portBombQueue']
                and all(type(value) is float for value in (row['nativeHp'], row['portHp'],
                        chain[-1]['nativeHp'], chain[-1]['portHp']))
                and row['nativeHp'] < chain[-1]['nativeHp'] and row['portHp'] < chain[-1]['portHp']):
            context['hp_changes'] += 1
            row['scope'] = 'Actual same-target queued Bomb source followed by next-Full HP decrease; not atomic attribution or an unproved damage-applier formula'
        chain.append(row)


def finish(context, shared):
    check, missing = shared['check'], shared['missing']
    for key, actual in (
            ('minimumOrdinaryBombBirthsPerGame', len(context['births'])),
            ('minimumFullBodyObservationsPerGame', len(context['full'])),
            ('minimumFastBodyObservationsPerGame', len(context['fast'])),
            ('minimumLaterFullRetirementsPerGame', len(context['retirement']))):
        minimum = integer(context['declaration'][key])
        check('required DrFetus ' + key, True, actual >= minimum > 0, None)
    check('required DrFetus actual real queued52 grant', True,
          len(context['grants']) >= integer(context['declaration']['minimumActual52GrantsPerGame']) > 0, None)
    check('required DrFetus actual receiver Bomb FIFO', True, context['queued'] > 0, None)
    # State snapshots cannot establish callback atomicity, a birth's prior DD4,
    # or explosion/Remove/Uninit execution from a fuse/absence coincidence.
    for field in ('actual nonzero prior-DD4 birth attribution', 'actual detonation callback',
                  'actual next-Full damage-applier coverage',
                  'actual ordered NPC damping/paired callback veto', 'actual Remove/BaseUninit event boundaries'):
        missing('required DrFetus ' + field, None, 'Not exposed by these typed snapshots; no event or default inferred')
    return dict(schema=SCHEMA, actualQueued52GrantPresentations=context['grants'], bodyRows=context['bodies'], observedBornZeroUpdateIdentities=sorted(context['births']),
                fullRows=len(context['full']), fastRows=len(context['fast']), managerRetirementStates=context['retirement'],
                receiverBombQueueRows=context['queued'], nextFullHpDecreaseRows=context['hp_changes'],
                contacts=[dict(identity=key, rows=rows) for key, rows in context['contacts'].items()],
                unqualifiedOwners=['Global7EC initializer/history (unused for flags0); no raw padding read',
                    'Port raw saved450/fast322/common visible/active/dead owners when absent',
                    'Atomic prior-DD4 birth, detonation, contact damping/veto and Remove/Uninit event ordering'],
                scope='Actual independently owned Bomb4 identities/body/FIFO fields only. Unknown fields remain failures; observed manager absence and HP changes are distinct from atomic callbacks. No whole Bomb/controller/render acceptance.')
