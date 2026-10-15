"""Actual controlled-Brim property/priority history, separate from older markers."""
import math
import struct

CONTRACT = 'controlled-brim-properties-priority-v1'


def _f32(value):
    if type(value) not in (int, float) or not math.isfinite(value):
        raise ValueError('Actual finite f32 owner required')
    result = struct.unpack('<f', struct.pack('<f', value))[0]
    if not math.isfinite(result):
        raise ValueError('Actual f32 overflow')
    return result


def compare_history(case, paired_rows, *, sides, shared):
    """Use canonical own-reference readers; no cross-game identity/time fitting."""
    View, integer = shared['_View'], shared['_integer']
    boolean, flags, vector = shared['_boolean'], shared['_flags'], shared['_vector']
    checks, witnesses, gaps = [], [], []
    result = {'checks': checks, 'witnesses': witnesses, 'coverageGaps': gaps,
              'sourceExclusions': [], 'untestedFrontiers': [],
              'scope': {'contract': CONTRACT, 'sides': list(sides),
                        'pairedParityCompared': sides == ('native', 'browser'),
                        'publicReplay': {'case': case['name'], 'epoch': case['bootEpochSeed'],
                                         'phase': case['initialPhaseOffset'],
                                         'actions': case['actions'], 'fixture': case['fixture']},
                        'provenanceScope': 'Literal case declaration only; unchanged actual public reset/tape/artifact binding remains the capture/readback prerequisite. Native observations are comparison inputs only.'}}

    def check(side, name, expected, actual, t=None):
        passed = type(expected) is type(actual) and expected == actual
        checks.append({'field': 'controlled Brim history ' + side + ' ' + name,
                       'presentation': t, 'expected': expected, 'actual': actual, 'passed': passed})
        return passed

    def missing(side, name, error, t=None):
        check(side, name, 'observed', 'unobservable: ' + str(error), t)
        gaps.append({'side': side, 'field': name, 'presentation': t, 'reason': str(error)})

    def field(side, name, expected, actual, t=None):
        try:
            return check(side, name, expected(), actual(), t)
        except (KeyError, TypeError, ValueError, IndexError, OverflowError, struct.error) as error:
            missing(side, name, error, t)
            return False

    def require(side, name, rows):
        check(side, name, True, bool(rows))
        witnesses.append({'side': side, 'kind': name,
                          'status': 'observed' if rows else 'missing', 'observations': rows})
        if not rows:
            gaps.append({'side': side, 'field': name, 'presentation': None,
                         'reason': 'No actual required witness; final counts, NULL slots and absence do not substitute.'})

    def vec(value):
        return tuple(_f32(v) for v in vector(value))

    def color(value):
        words = value if isinstance(value, list) else sum(
            [value[key] for key in ('tint', 'sourceMiddle', 'offsets')], [])
        if len(words) != 11:
            raise ValueError('Actual complete eleven-component color required')
        return tuple(_f32(v) for v in words)

    stages = case['requires']['actorStages']
    if len(stages) != 3:
        raise ValueError('This version requires the declared three property/priority stages')
    counts = []
    for stage in stages:
        value = [0] * 347
        for entry in stage['ownedCounts']:
            identifier, count = integer(entry['itemId']), integer(entry['count'])
            if not 0 <= identifier < 347:
                raise ValueError('Declared item outside source inventory')
            value[identifier] = count
        if value in counts:
            raise ValueError('Actual count stages must be distinct')
        counts.append(value)
    check('declared', 'marker', CONTRACT, case['requires']['historyWitnessContract'])
    check('declared', 'real public grants', [4, 114], case['requires']['orderedRealPedestalGrants'])

    # Preserve duplicate multiplicity, then use only equal actual gates causally.
    gates, conflicting = {}, set()
    signatures = ('source', 'nativePlayer', 'nativeCounter', 'state') if len(sides) == 2 else (
        'source', 'nativePlayer', 'nativeCounter')
    for row in sorted(paired_rows, key=lambda r: r['presentation']):
        t = integer(row['presentation'])
        if t not in gates:
            gates[t] = row
        else:
            for key in signatures:
                if not field('paired', 'duplicate actual ' + key,
                             lambda key=key: gates[t][key], lambda key=key: row[key], t):
                    conflicting.add(t)
    ordered = [gates[t] for t in sorted(gates) if t not in conflicting]
    records = {side: [] for side in sides}
    by_gate = {side: {} for side in sides}

    def actor_field(record, source_key, port_key):
        return record['actor'][source_key if record['view'].side == 'native' else port_key]

    def laser_field(record, key):
        a = record['actor']
        return a['laser'][key] if record['view'].side == 'native' else a[key]

    def line(record):
        a = record['actor']
        return a['laser']['line'] if record['view'].side == 'native' else a['line']

    def width(record):
        value = line(record)
        if record['view'].side != 'native':
            return _f32(value['width'])
        actor = record['actor']
        if (record['view'].identity(record['slot'], actor)[:3] != (7, 1, 1)
                or actor['linkedParent'] is None or actor['linkedParent']['kind'] != 1
                or boolean(actor['laser']['firstUpdate'])
                or not boolean(value['sourceControlledLineWriterScope']) or len(value['points']) < 2):
            raise ValueError('Actual controlled Line writer scope is missing')
        # 4799DF owns810 after577120(50,1). Decode actual bits, never insert60.
        bits = integer(value['widthRawBits'])
        if not 0 <= bits <= 0xffffffff:
            raise ValueError('Actual u32 width bits required')
        return _f32(struct.unpack('<f', struct.pack('<I', bits))[0])

    def parent_identity(record):
        view, actor = record['view'], record['actor']
        if view.side == 'native':
            return shared['_identity'](actor['linkedParent'])
        ref = actor['parent']
        if shared['_ref'](ref) != ('player', 0):
            raise ValueError('Controlled Laser actual Parent must resolve to first Player')
        rows = [r for r in view.state['actorScheduler']['identities'] if r['actor'] == ref]
        if len(rows) != 1:
            raise ValueError('Actual Parent scheduler identity must be unique')
        return (1, 0, 0, integer(rows[0]['sequenceId']), integer(view.state['playerItems']['entitySpawnSeed']))

    def rng(record):
        actor = record['actor']
        words = actor['actorRngWords7c0'] if record['view'].side == 'native' else [
            actor['actorRng'][key] for key in ('state', 'rightA', 'left', 'rightB')]
        if len(words) != 4 or any(not 0 <= integer(word) <= 0xffffffff for word in words):
            raise ValueError('Actual four u32 RNG owners required')
        return tuple(words)

    def owner_properties(record):
        view = record['view']
        if view.side == 'native':
            return (_f32(view.player['cachedStats']['damage']),
                    flags(view.owner['attackFlagsC98']), color(view.owner['cachedColorCCC']))
        return (_f32(view.state['playerStats']['damage']),
                flags(view.state['weaponCache']['flagsC98']), color(view.state['weaponCache']['colorCcc']))

    def current_properties(record):
        return (_f32(record['actor']['collisionDamage']), flags(laser_field(record, 'attackFlags')),
                color(actor_field(record, 'currentColor2d0', 'currentColor')))

    def property_fence(record):
        view = record['view']
        if view.side == 'native':
            return (boolean(view.player['controlsEnabled']) and integer(view.player['inputCooldown']) <= 0
                    and not boolean(view.player['pickupAnimationGuard'])
                    and integer(view.player['activeStateD80']) == 0 and boolean(view.owner['canShoot3075']))
        return (boolean(view.state['player']['controlsEnabled']) and integer(view.state['player']['inputCooldown']) <= 0
                and not boolean(view.player['pickupAnimationGuard'])
                and integer(view.player['activeStateD80']) == 0 and boolean(view.state['player']['canShoot']))

    for side in sides:
        seen = []
        for row in ordered:
            t = row['presentation']
            try:
                view = View(row, side)
                stage = counts.index(view.counts) if view.counts in counts else None
                if stage is None and not seen:
                    continue
                check(side, 'actual counts select declared stage', True, stage is not None, t)
                if stage is None:
                    continue
                if not seen or seen[-1] != stage:
                    seen.append(stage)
                check(side, 'current primary', stages[stage]['sourceExpectedActorContract']['primaryC08'], view.primary, t)
                check(side, 'pending cache', 0, view.pending, t)
                if side == 'browser':
                    check(side, 'unadmitted controller', None, view.state['weaponAdmission']['unadmittedController'], t)
                    check(side, 'pending primary cleanup', None, view.state['weaponAdmission']['pendingPrimaryCleanup'], t)
                slot = view.slot()
                actor = view.actor(slot)
                record = {'view': view, 'stage': stage, 'presentation': t, 'slot': slot, 'actor': actor}
                records[side].append(record)
                by_gate[side][t] = record
                secondary = [view.slot(i) for i in (1, 2, 3)]
                witnesses.append({'side': side, 'kind': 'actual weak slots; NULL is not cleanup',
                                  'presentation': t, 'c18': slot, 'secondary': secondary})
                for key in secondary:
                    if key is not None:
                        check(side, 'nonnull secondary resolves to own actor', True, view.actor(key) is not None, t)
                if slot is not None:
                    # Counts can change while Pickup still fences the next caller.
                    # Keep the actual ref now; require the new family as a witness below.
                    field(side, 'C18 actual identity observable', lambda: True,
                          lambda: len(view.identity(slot, actor)) == 5, t)
                    field(side, 'C18 own manager membership', lambda: True, lambda: view.manager(slot, actor), t)
                if actor is not None and stage < 2:
                    for key, expected in [('subdivisions', 50), ('mode', 1)]:
                        field(side, 'controlled Line ' + key, lambda expected=expected: expected,
                              lambda key=key: integer(line(record)[key]), t)
                    field(side, 'controlled Line width', lambda: 60.0, lambda: width(record), t)
                    field(side, 'actual ParentPlayer owner', lambda: (1, 0, 0), lambda: parent_identity(record)[:3], t)
                    field(side, 'no fabricated Impact child', lambda: None, lambda: laser_field(record, 'child'), t)
                    field(side, 'actual Line flags mirror current C98', lambda: owner_properties(record)[1],
                          lambda: flags(laser_field(record, 'lineFlags')), t)
                    # This records observed property ownership, not an inferred callback trace.
                    try:
                        ready = property_fence(record)
                    except (KeyError, TypeError, ValueError) as error:
                        missing(side, 'actual property callback guards', error, t)
                        ready = False
                    if ready:
                        field(side, 'current C88/CCC/C98 reapplied', lambda: owner_properties(record),
                              lambda: current_properties(record), t)
                        field(side, 'permanent CCC owner', lambda: owner_properties(record)[2],
                              lambda: color(actor_field(record, 'permanentColor3a0', 'sourceColor')), t)
                    else:
                        witnesses.append({'side': side, 'kind': 'property callback blocked or unobservable',
                                          'presentation': t, 'scope': 'Body and cross-game fields remain compared; not a reapply witness.'})
                elif actor is not None and view.identity(slot, actor)[:3] == (8, 0, 0):
                    field(side, 'actual controlled Knife bit33', lambda: True,
                          lambda: bool(view.attack_flags(actor) & (1 << 33)), t)
            except (KeyError, TypeError, ValueError, IndexError) as error:
                missing(side, 'actual own-reference observation', error, t)
        check(side, 'all actual stages in order', [0, 1, 2], seen)
        for stage in range(3):
            actual = []
            expected = (7, 1, 1) if stage < 2 else (8, 0, 0)
            for r in records[side]:
                if r['stage'] != stage or r['actor'] is None:
                    continue
                try:
                    if (r['view'].identity(r['slot'], r['actor'])[:3] == expected
                            and r['view'].manager(r['slot'], r['actor'])
                            and boolean(r['actor']['active']) and not boolean(r['actor']['dead'])):
                        actual.append(r)
                except (KeyError, TypeError, ValueError) as error:
                    missing(side, 'actual stage actor family', error, r['presentation'])
            require(side, 'stage ' + str(stage) + ' actual C18 body', [r['presentation'] for r in actual])
            for fast in (False, True):
                found = []
                for r in actual:
                    try:
                        if boolean(actor_field(r, 'fast322', 'fast')) == fast:
                            found.append({'presentation': r['presentation'], 'actualCounter': r['view'].counter})
                    except (KeyError, TypeError, ValueError) as error:
                        missing(side, 'actual Full/Fast body flag', error, r['presentation'])
                require(side, 'stage ' + str(stage) + (' Fast' if fast else ' Full') + ' body', found)

        for target, item in ((1, 4), (2, 114)):
            after = next((r for r in records[side] if r['stage'] == target), None)
            before = [r for r in records[side] if after and r['presentation'] < after['presentation']]
            require(side, 'actual queued' + str(item), [r['presentation'] for r in before if r['view'].queued == item])
            if not after or not before:
                missing(side, 'actual Add' + str(item) + ' boundary', 'Missing observed before/after count stages')
                continue
            prior = before[-1]
            check(side, 'Add' + str(item) + ' dense count boundary', prior['presentation'] + 1, after['presentation'], after['presentation'])
            if target == 1:
                field(side, 'Add4 current C88 changes', lambda: True,
                      lambda: owner_properties(prior)[0] != owner_properties(after)[0], after['presentation'])
                field(side, 'Add4 retains same own C18', lambda: prior['slot'], lambda: after['slot'], after['presentation'])
                reapply = []
                for r in records[side]:
                    if r['stage'] == 1 and r['slot'] == prior['slot'] and r['actor'] is not None:
                        try:
                            if property_fence(r) and current_properties(r) == owner_properties(r):
                                reapply.append(r['presentation'])
                        except (KeyError, TypeError, ValueError, OverflowError) as error:
                            missing(side, 'actual Add4 live reapply owner', error, r['presentation'])
                require(side, 'actual Add4 live properties on same actor', reapply)
            else:
                old = prior['slot']
                field(side, 'prior controlled Laser actually live before114', lambda: (True, False, True),
                      lambda: (boolean(prior['actor']['active']), boolean(prior['actor']['dead']),
                               prior['view'].manager(old, prior['actor'])), prior['presentation'])
                removal, absent, uninit = [], [], []
                for r in records[side]:
                    if r['presentation'] < after['presentation']:
                        continue
                    try:
                        actor = r['view'].actor(old)
                        if actor is not None and not boolean(actor['active']) and boolean(actor['dead']):
                            removal.append(r['presentation'])
                        if actor is None or not r['view'].manager(old, actor):
                            absent.append(r['presentation'])
                            field(side, 'retired Laser observed weak references clear', lambda: False,
                                  lambda: r['view'].weak_references(old), r['presentation'])
                        if side == 'browser' and actor is not None and boolean(actor['uninitialized']):
                            uninit.append(r['presentation'])
                    except (KeyError, TypeError, ValueError) as error:
                        missing(side, 'actual retired Laser owners', error, r['presentation'])
                require(side, 'actual prior Laser Remove flags', removal)
                require(side, 'independent later manager absence', absent)
                if side == 'browser':
                    require(side, 'actual serialized Uninit observation', uninit)
                replacements = []
                for r in records[side]:
                    if r['stage'] != 2 or r['actor'] is None:
                        continue
                    try:
                        if (r['view'].identity(r['slot'], r['actor'])[:3] == (8, 0, 0)
                                and r['view'].manager(r['slot'], r['actor'])
                                and boolean(r['actor']['active']) and not boolean(r['actor']['dead'])):
                            replacements.append(r)
                    except (KeyError, TypeError, ValueError) as error:
                        missing(side, 'actual post114 replacement body', error, r['presentation'])
                require(side, '114 registered controlled Knife replacement', [r['presentation'] for r in replacements])
                if replacements:
                    replacement = replacements[0]
                    field(side, '114 creates distinct own C18', lambda: False,
                          lambda: old == replacement['slot'], replacement['presentation'])

    if sides == ('native', 'browser'):
        common = sorted(set(by_gate['native']) | set(by_gate['browser']))
        for t in common:
            n, p = by_gate['native'].get(t), by_gate['browser'].get(t)
            if not n or not p:
                missing('paired', 'actual same-gate stage/body', 'Missing actual side', t)
                continue
            field('paired', 'stage', lambda: n['stage'], lambda: p['stage'], t)
            field('paired', 'C18 nonnull', lambda: n['slot'] is not None, lambda: p['slot'] is not None, t)
            if n['actor'] is None and p['actor'] is None:
                continue
            if n['actor'] is None or p['actor'] is None:
                missing('paired', 'actual registered body', 'Missing own C18 actor', t)
                continue
            field('paired', 'actual identity including seed', lambda: n['view'].identity(n['slot'], n['actor']),
                  lambda: p['view'].identity(p['slot'], p['actor']), t)
            if n['stage'] >= 2 or p['stage'] >= 2:
                continue  # The canonical body comparer retains controlled-Knife fields.
            a, b = n['actor'], p['actor']
            for nk, pk in [('position', 'position'), ('savedPosition450', 'savedPosition'),
                           ('velocity', 'velocity'), ('renderOffset', 'renderOffset')]:
                field('paired', nk, lambda nk=nk: vec(a[nk]), lambda pk=pk: vec(b[pk]), t)
            for key in ('active', 'dead', 'visible', 'raw494', 'gridCollisionClass', 'entityCollisionClass'):
                field('paired', key, lambda key=key: a[key], lambda key=key: b[key], t)
            for nk, pk in [('hp', 'hp'), ('maxHp', 'maxHp'), ('collisionDamage', 'collisionDamage'),
                           ('collisionRadius', 'collisionRadius'), ('collisionMass', 'collisionMass'),
                           ('currentFriction470', 'currentFriction'), ('configuredFriction46c', 'configuredFriction')]:
                field('paired', nk, lambda nk=nk: _f32(a[nk]), lambda pk=pk: _f32(b[pk]), t)
            field('paired', 'Fast actual flag', lambda: boolean(a['fast322']), lambda: boolean(b['fast']), t)
            field('paired', 'actual birth age', lambda: integer(a['bornGameAge']), lambda: integer(b['birthGameUpdate']), t)
            field('paired', 'RNG actual words', lambda: rng(n), lambda: rng(p), t)
            field('paired', 'actual Parent identity including seed', lambda: parent_identity(n), lambda: parent_identity(p), t)
            field('paired', 'generic flags318', lambda: integer(a['genericFlags318']), lambda: integer(b['entityFlags']), t)
            for nk, pk in [('permanentColor3a0', 'sourceColor'), ('currentColor2d0', 'currentColor')]:
                field('paired', nk, lambda nk=nk: color(a[nk]), lambda pk=pk: color(b[pk]), t)
            for key in ('remainingUpdates', 'firstUpdate', 'lineEnabled', 'contactDisabled', 'lineController'):
                field('paired', 'actual ' + key, lambda key=key: laser_field(n, key), lambda key=key: laser_field(p, key), t)
            for key in ('attackFlags', 'lineFlags'):
                field('paired', key, lambda key=key: flags(laser_field(n, key)), lambda key=key: flags(laser_field(p, key)), t)
            for key in ('origin', 'offset'):
                field('paired', 'Line ' + key, lambda key=key: vec(line(n)[key]), lambda key=key: vec(line(p)[key]), t)
            for key in ('points', 'priorPoints', 'history'):
                field('paired', 'complete Line ' + key, lambda key=key: [vec(v) for v in line(n)[key]],
                      lambda key=key: [vec(v) for v in line(p)[key]], t)
            field('paired', 'Line width', lambda: width(n), lambda: width(p), t)
            field('paired', 'Line length', lambda: _f32(line(n)['length']), lambda: _f32(line(p)['length']), t)
            field('paired', 'complete Line controls',
                  lambda: [tuple(_f32(v) for v in values) for values in line(n)['controls']],
                  lambda: [vec(v['position']) + vec(v['incoming']) + vec(v['outgoing']) for v in line(p)['controls']], t)
            for key in ('animation', 'frame', 'playing', 'rotation', 'scale', 'offset'):
                normal = vec if key in ('scale', 'offset') else _f32 if key in ('frame', 'rotation') else lambda v: v
                field('paired', 'Sprite ' + key, lambda key=key: normal(a['sprite'][key]),
                      lambda key=key: normal(b['sprite'][key]), t)
    for event in ('Remove', 'Uninit'):
        missing('native', 'actual Laser ' + event + ' source event',
                'Paired rows carry no separately validated Laser lifecycle journal; flags and manager absence are independent observations')
    missing('native', 'initial Full-before-C18 call-order trace',
            'Snapshots establish bodies, not call order; a separately bound source-event observer is required')
    return result
