"""Witness live item histories from each game's own raw observations.

This augments the body comparer. It neither pairs actors across games nor
replaces its identity, seed, numeric, render or input checks. A missed boundary
is missing evidence, even when later counts or absent objects look plausible.
"""
import math


class Unobservable(ValueError):
    """A required raw field or unique own reference was not observable."""


def _integer(value):
    if type(value) is not int:
        raise Unobservable('Expected an actual integer field')
    return value


def _identity(value):
    if not isinstance(value, dict):
        raise Unobservable('Expected an actual native identity')
    return tuple(_integer(value[key]) for key in
                 ('kind', 'variant', 'subtype', 'sequenceId', 'spawnSeed'))


def _vector(value):
    words = [value['x'], value['y']] if isinstance(value, dict) else value
    if not isinstance(words, list) or len(words) != 2 or any(
            type(word) not in (int, float) or not math.isfinite(word) for word in words):
        raise Unobservable('Expected two actual finite vector components')
    return tuple(words)


def _flags(value):
    if isinstance(value, list) and len(value) == 2:
        if any(type(word) is not int or not 0 <= word <= 0xffffffff for word in value):
            raise Unobservable('Expected two actual u32 flag words')
        return value[0] | value[1] << 32
    if isinstance(value, str) and value.isdecimal():
        value = int(value)
    if type(value) is not int or not 0 <= value <= 0xffffffffffffffff:
        raise Unobservable('Expected an actual u64 flag value')
    return value


def _ref(value):
    if value is None:
        return None
    if not isinstance(value, dict) or value['kind'] not in ('knife', 'laser', 'player'):
        raise Unobservable('Expected an actual weapon or Player handle')
    return value['kind'], _integer(value['id'])


def _boolean(value):
    if type(value) is not bool:
        raise Unobservable('Expected an actual boolean field')
    return value


def _finite_geometry(value):
    if isinstance(value, dict):
        return all(_finite_geometry(word) for word in value.values())
    if isinstance(value, list):
        return all(_finite_geometry(word) for word in value)
    return type(value) in (int, float) and math.isfinite(value)


class _View:
    """Read one raw side; keys remain local to this game throughout a history."""

    def __init__(self, row, side):
        self.row, self.side = row, side
        if side == 'native':
            self.player = row['nativePlayer']
            self.source = row['source']
            if len(self.source['players']) != 1:
                raise Unobservable('Expected the actual single Player')
            self.owner = self.source['players'][0]
            self.counts = self.player['ownedCounts']
            self.primary = _integer(self.owner['primaryWeaponC08'])
            self.counter = _integer(row['nativeCounter'])
            self.pending = _integer(self.player['cacheFlags'])
            self.queued = self.player['queuedId']
        else:
            self.state = row['state']
            self.player = self.state['playerItems']
            self.owner = self.state['attackRuntime']
            self.counts = self.player['ownedCounts']
            self.primary = _integer(self.state['weaponCache']['primary'])
            self.counter = _integer(self.state['systemClock']['systemCounter'])
            self.pending = _integer(self.player['unevaluatedCacheFlags'])
            self.queued = self.player['queuedId']
        if not isinstance(self.counts, list) or len(self.counts) != 347:
            raise Unobservable('Expected the actual complete 347 signed owned counts')
        if any(type(count) is not int for count in self.counts):
            raise Unobservable('Owned counts must be observed integers')
        if self.queued is not None:
            _integer(self.queued)

    def slot(self, index=0):
        if self.side == 'native':
            value = self.owner['actorSlots'][('c18', 'c20', 'c28', 'c30')[index]]
            return None if value is None else _identity(value['identity'])
        value = self.owner['persistent'] if index == 0 else self.owner['secondary'][index - 1]
        return _ref(value)

    def actors(self):
        if self.side == 'native':
            result = {}
            for actor in self.source['attacks']:
                key = _identity(actor['identity'])
                if key in result:
                    raise Unobservable('Duplicate own native actor identity')
                result[key] = actor
            return result
        result = {}
        for kind, name in (('knife', 'knives'), ('laser', 'lasers')):
            for actor in self.state[name]:
                key = (kind, _integer(actor['id']))
                if key in result:
                    raise Unobservable('Duplicate own actor handle')
                result[key] = actor
        return result

    def actor(self, key):
        if key is None:
            return None
        if self.side == 'native':
            matches = [actor for actor in self.source['attacks']
                       if _identity(actor['identity']) == key]
        else:
            kind, identifier = key
            name = {'knife': 'knives', 'laser': 'lasers'}.get(kind)
            if name is None:
                raise Unobservable('Weapon slot references a nonweapon owner')
            matches = [actor for actor in self.state[name] if actor['id'] == identifier]
        if len(matches) > 1:
            raise Unobservable('Own reference resolves more than once')
        return matches[0] if matches else None

    def identity(self, key, actor):
        if self.side == 'native':
            return _identity(actor['identity'])
        ref = {'kind': key[0], 'id': key[1]}
        matches = [entry for entry in self.state['actorScheduler']['identities']
                   if entry['actor'] == ref]
        if len(matches) != 1:
            raise Unobservable('Own actor must have one actual scheduler identity')
        return (_integer(actor['entityType']), _integer(actor['variant']),
                _integer(actor['subtype']), _integer(matches[0]['sequenceId']),
                _integer(actor['spawnSeed']))

    def manager(self, key, actor):
        if self.side == 'native':
            return _boolean(actor['inActiveManager']) if actor is not None else False
        return {'kind': key[0], 'id': key[1]} in self.state['actorScheduler']['actors']

    def parent(self, actor):
        value = actor['linkedParent'] if self.side == 'native' else actor['parent']
        return None if value is None else (_identity(value) if self.side == 'native' else _ref(value))

    def attack_flags(self, actor):
        return _flags(actor['attackFlags7d8'] if self.side == 'native' else actor['attackFlags'])

    def line(self, actor):
        line = actor['knifeLine'] if self.side == 'native' else actor['line']
        # These are actual copied curve owners, not the parent's later curve.
        for name in ('points', 'priorPoints', 'history'):
            for value in line[name]:
                _vector(value)
        copied = {name: line[name] for name in
                  ('controls', 'points', 'priorPoints', 'history', 'origin', 'offset')}
        if not _finite_geometry(copied):
            raise Unobservable('Copied curve contains missing or nonfinite numeric owners')
        return copied

    def weak_references(self, key):
        linked = any(self.parent(actor) == key for actor in self.actors().values())
        slots = any(self.slot(index) == key for index in range(4))
        if self.side == 'native':
            linked = linked or any(impact['linkedParent'] is not None
                                   and _identity(impact['linkedParent']) == key
                                   for impact in self.source['impacts'])
        else:
            ref = {'kind': key[0], 'id': key[1]}
            linked = linked or any(actor['homingTarget'] == ref for actor in self.state['knives'])
            linked = linked or any(laser['impact'] is not None
                                   and laser['impact']['parent'] == ref
                                   for laser in self.state['lasers'])
        return linked or slots


def compare_history(case_definition, paired_rows, *, sides=('native', 'browser')):
    """Compare actual history; native-only reads cannot establish paired parity."""
    if sides not in (('native', 'browser'), ('native',)):
        raise ValueError('Expected the paired contract or explicit native-only readback')
    requires = case_definition.get('requires', {})
    stages = requires.get('actorStages', [])
    checks, witnesses, gaps = [], [], []
    exclusions, frontiers = [], []
    result = {'checks': checks, 'witnesses': witnesses, 'coverageGaps': gaps,
              'sourceExclusions': exclusions, 'untestedFrontiers': frontiers,
              'scope': {'sides': list(sides), 'pairedParityCompared': sides == ('native', 'browser'),
                        'nativeOnlyScope': 'Actual Native history only; no fabricated browser or paired acceptance'
                        if sides == ('native',) else None}}
    if not stages:
        return result

    def check(side, name, expected, actual, presentation=None):
        passed = expected == actual
        checks.append({'field': 'history ' + side + ' ' + name,
                       'presentation': presentation, 'expected': expected,
                       'actual': actual, 'passed': passed})
        return passed

    def missing(side, name, reason, presentation=None):
        check(side, name, 'observed', 'unobservable: ' + reason, presentation)
        gaps.append({'side': side, 'field': name, 'presentation': presentation, 'reason': reason})

    def require(side, name, observations):
        check(side, name, True, bool(observations))
        witnesses.append({'side': side, 'kind': name,
                          'status': 'observed' if observations else 'missing',
                          'observations': observations})
        if not observations:
            gaps.append({'side': side, 'field': name, 'presentation': None,
                         'reason': 'No actual required boundary; final inventory, NULL slots and disappearance cannot substitute.'})

    contract = 'brim-ordinary-controlled-parent-knife-v1'
    queued_lifetime = requires.get('historyWitnessContract') == 'brim-ordinary-controlled-queued-lifetime-v2'
    if queued_lifetime:
        contract = 'brim-ordinary-controlled-queued-lifetime-v2'
        exclusions.append({'field': 'positive prior Laser TTL at deferred Add114',
                           'reason': 'Uninterrupted Pickup42 exceeds ordinary release Laser timeout15; natural expiry is not selector cleanup.'})
        frontiers.append({'field': 'live selector-change Laser Remove and replacement-safe Uninit',
                          'reason': 'Requires a separately admitted public route with a live prior actor at the actual grant.'})
    if not check('declared', 'history witness contract', contract, requires.get('historyWitnessContract')):
        gaps.append({'side': 'declared', 'field': 'history witness contract', 'presentation': None,
                     'reason': 'Unversioned or different history obligations are unsupported. Do not reinterpret an older secondary-slot demand as ParentKnife child continuation.'})
        return result

    expected_counts = []
    for stage in stages:
        counts = [0] * 347
        for entry in stage['ownedCounts']:
            identifier, count = entry['itemId'], entry['count']
            if type(identifier) is not int or not 0 <= identifier < 347 or type(count) is not int:
                raise ValueError('Invalid declared owned-count stage')
            counts[identifier] = count
        if counts in expected_counts:
            raise ValueError('History stages need distinct actual count states')
        expected_counts.append(counts)
    gates, conflicts = {}, set()
    for row in sorted(paired_rows, key=lambda row: row['presentation']):
        gate = row['presentation']
        if gate not in gates:
            gates[gate] = row
            continue
        # Dense prelude and gameplay-start deliberately share one actual gate.
        # Retain each duplicate comparison; only equal actual data can serve
        # as one causal boundary. Provenance labels are not simulated state.
        equal = True
        fields = ('source', 'nativePlayer', 'state', 'nativeCounter') if len(sides) == 2 else (
            'source', 'nativePlayer', 'nativeCounter')
        for field in fields:
            try:
                equal = check('paired', 'same-gate duplicate actual ' + field,
                              gates[gate][field], row[field], gate) and equal
            except KeyError as error:
                missing('paired', 'same-gate duplicate observation', str(error), gate)
                equal = False
        witnesses.append({'side': 'paired', 'kind': 'duplicate actual gate retained',
                          'presentation': gate, 'status': 'observed' if equal else 'conflicting'})
        if not equal:
            conflicts.add(gate)
            gaps.append({'side': 'paired', 'field': 'conflicting duplicate actual gate',
                         'presentation': gate, 'reason': 'Conflicting raw values cannot establish a causal boundary.'})
    ordered = [gates[gate] for gate in sorted(gates) if gate not in conflicts]

    for side in sides:
        observations, seen_stages = [], []
        known_actors = set()
        for row in ordered:
            t = row['presentation']
            try:
                view = _View(row, side)
                stage_index = expected_counts.index(view.counts) if view.counts in expected_counts else None
                if stage_index is None and not seen_stages:
                    continue  # Menu/pregrant observations do not establish a history stage.
                check(side, 'actual counts select a declared stage', True, stage_index is not None, t)
                if stage_index is None:
                    continue
                if not seen_stages or seen_stages[-1] != stage_index:
                    seen_stages.append(stage_index)
                contract = stages[stage_index]['sourceExpectedActorContract']
                check(side, 'stage current primary C08', contract['primaryC08'], view.primary, t)
                if 'actualCachePendingFlags' in requires:
                    check(side, 'actual pending cache flags', requires['actualCachePendingFlags'], view.pending, t)
                if side == 'browser':
                    admission = row['state']['weaponAdmission']
                    check(side, 'canonical unadmitted controller', None, admission['unadmittedController'], t)
                    check(side, 'retained pending primary cleanup', None, admission['pendingPrimaryCleanup'], t)
                slots = [view.slot(index) for index in range(4)]
                actors = view.actors()
                current = view.actor(slots[0])
                if slots[0] is not None:
                    if current is None:
                        raise Unobservable('Registered C18 does not resolve to its own actor')
                    view.identity(slots[0], current)
                for index, key in enumerate(slots[1:], 1):
                    if key is not None and view.actor(key) is None:
                        raise Unobservable('Actual secondary slot does not resolve to its own actor')
                    # Archive NULL as an observed value, never as cleanup coverage.
                    check(side, 'C' + ('20', '28', '30')[index - 1] + ' actual weak-slot observed',
                          True, key is None or view.actor(key) is not None, t)
                births = set(actors) - known_actors
                record = {'view': view, 'stage': stage_index, 'presentation': t,
                          'slot': slots[0], 'secondary': slots[1:], 'actor': current,
                          'actors': actors, 'births': births}
                observations.append(record)
                known_actors.update(actors)
            except (KeyError, TypeError, IndexError, Unobservable) as error:
                missing(side, 'raw history observation', str(error), t)
        check(side, 'all actual count stages observed in declared order', list(range(len(stages))), seen_stages)
        witnesses.append({'side': side, 'kind': 'actual weak-slot values; NULL is not cleanup',
                          'status': 'observed' if observations else 'missing',
                          'observations': [{'presentation': row['presentation'],
                                            'c18': row['slot'], 'secondary': row['secondary']}
                                           for row in observations]})
        for index, stage in enumerate(stages):
            contract = stage['sourceExpectedActorContract']
            actual_bodies = []
            for record in observations:
                if record['stage'] != index or record['actor'] is None:
                    continue
                view, actor, key = record['view'], record['actor'], record['slot']
                try:
                    if (view.identity(key, actor)[:3] == (contract['family'], contract['variant'], contract['subtype'])
                            and view.manager(key, actor) and _boolean(actor['active']) and not _boolean(actor['dead'])):
                        actual_bodies.append(record['presentation'])
                except (KeyError, TypeError, IndexError, Unobservable) as error:
                    missing(side, 'stage own actor body', str(error), record['presentation'])
            require(side, 'stage ' + str(index) + ' actual registered own actor family', actual_bodies)

        # Stage-only declarations can use these checks without claiming the
        # specific three-stage lifecycle below.
        if requires.get('lifecycleExperiment') != 'brim-to-ordinary-to-controlled-triple':
            missing(side, 'declared history implementation', 'No source-specific lifecycle witness implementation for this experiment')
            continue
        grant_rows = {}
        for item in requires['orderedRealPedestalGrants']:
            grants = [(before, after) for before, after in zip(observations, observations[1:])
                      if after['view'].counts[item] - before['view'].counts[item] == 1]
            require(side, 'actual grant count transition ' + str(item),
                    [{'before': before['presentation'], 'after': after['presentation']} for before, after in grants])
            if len(grants) != 1:
                missing(side, 'unique grant boundary ' + str(item), 'Expected exactly one actual count increase')
                continue
            before, after = grants[0]
            grant_rows[item] = (before, after)
            check(side, 'adjacent observed grant boundary ' + str(item),
                  before['presentation'] + 1, after['presentation'], after['presentation'])
            queued = [row['presentation'] for row in observations
                      if row['presentation'] <= after['presentation'] and row['view'].queued == item]
            require(side, 'actual queued item before grant ' + str(item), queued)
        check(side, 'actual ordered pedestal grant history', requires['orderedRealPedestalGrants'],
              [item for item, _ in sorted(grant_rows.items(), key=lambda entry: entry[1][1]['presentation'])])

        def retirement(key, born_identity, after_presentation, name, replacement=None):
            removed, manager_absences, retired, ordered_cleanup = [], [], [], []
            replacement_safe = []
            for record in observations:
                if record['presentation'] < after_presentation:
                    continue
                view, actor = record['view'], record['view'].actor(key)
                try:
                    tombstone = (side == 'browser' and actor is not None and key[0] == 'laser'
                                 and _boolean(actor['uninitialized']))
                    if actor is not None and not tombstone:
                        same_identity = check(side, name + ' own identity retained', born_identity,
                                              view.identity(key, actor), record['presentation'])
                        if same_identity and not _boolean(actor['active']) and _boolean(actor['dead']):
                            removed.append(record['presentation'])
                    elif view.counter % 2 == 1:
                        manager_absent = not view.manager(key, None)
                        absence = {'presentation': record['presentation'], 'counter': view.counter,
                                   'managerAbsent': manager_absent, 'storageAbsent': actor is None,
                                   'explicitPortUninitializedTombstone': tombstone,
                                   'removeFlagsPreviouslyObserved': bool(removed)}
                        if manager_absent:
                            manager_absences.append(absence)
                        absent = manager_absent and not view.weak_references(key)
                        kept = replacement is None or record['slot'] == replacement
                        absence.update(observedReferencesClear=absent, replacementC18Retained=kept)
                        check(side, name + ' full retirement observed slot/Parent/homing references clear', True, absent, record['presentation'])
                        check(side, name + ' replacement C18 retained', True, kept, record['presentation'])
                        if absent:
                            retired.append(record['presentation'])
                            if kept:
                                replacement_safe.append(absence)
                            if removed and record['presentation'] > removed[0]:
                                ordered_cleanup.append({'remove': removed[0], 'laterFull': absence})
                            # A replacement born after the first observed
                            # full cleanup cannot retroactively prove that
                            # cleanup preserved a previously bound replacement.
                            break
                except (KeyError, TypeError, IndexError, Unobservable) as error:
                    missing(side, name + ' retirement observation', str(error), record['presentation'])
            require(side, name + ' actual Remove flags', removed)
            require(side, name + ' actual later full manager absence independently observed', manager_absences)
            require(side, name + ' later full Uninit: own manager absence and observed weak references clear', retired)
            require(side, name + ' observed Remove precedes full cleanup', ordered_cleanup)
            if replacement is not None:
                require(side, name + ' replacement C18 retained at FIRST observed full cleanup', replacement_safe)

        laser_boundary = grant_rows.get(114)
        knife_boundary = grant_rows.get(329)
        if queued_lifetime and laser_boundary:
            before, after = laser_boundary
            contacts = [(left, right) for left, right in zip(observations, observations[1:])
                        if left['view'].queued != 114 and right['view'].queued == 114
                        and right['presentation'] <= after['presentation']]
            require(side, 'actual queued114 contact boundary',
                    [{'before': left['presentation'], 'contact': right['presentation']}
                     for left, right in contacts])
            if len(contacts) != 1:
                missing(side, 'unique queued114 contact', 'Expected one actual transition into queued114')
            else:
                left, contact = contacts[0]
                check(side, 'adjacent observed queued114 contact', left['presentation'] + 1,
                      contact['presentation'], contact['presentation'])
                check(side, 'queued114 contact precedes deferred grant', True,
                      contact['presentation'] < after['presentation'], contact['presentation'])
                check(side, 'queued114 contact preserves pregrant counts', expected_counts[0],
                      contact['view'].counts, contact['presentation'])
                view, key, actor = contact['view'], contact['slot'], contact['actor']
                try:
                    live = bool(actor and view.identity(key, actor)[:3] == (7, 1, 0)
                                and _boolean(actor['active']) and not _boolean(actor['dead'])
                                and view.manager(key, actor))
                    ttl = _integer(actor['laser']['remainingUpdates'] if side == 'native'
                                   else actor['remainingUpdates']) if live else 0
                    require(side, 'live registered positive-TTL Laser at queued114 contact',
                            [contact['presentation']] if live and ttl > 0 else [])
                    if live and ttl > 0:
                        identity, expiry, prior_full = view.identity(key, actor), [], None
                        for row in observations:
                            if not contact['presentation'] <= row['presentation'] < after['presentation']:
                                continue
                            own, actual = row['view'], row['view'].actor(key)
                            if actual is None or own.counter % 2 != 1:
                                continue
                            if not check(side, 'queued Laser own identity retained', identity,
                                         own.identity(key, actual), row['presentation']):
                                continue
                            timer = _integer(actual['laser']['remainingUpdates'] if side == 'native'
                                             else actual['remainingUpdates'])
                            current = {'presentation': row['presentation'], 'counter': own.counter,
                                       'remainingUpdates': timer}
                            if (prior_full and prior_full['remainingUpdates'] == 1 and timer == 0
                                    and current['counter'] == prior_full['counter'] + 2
                                    and current['presentation'] == prior_full['presentation'] + 2):
                                expiry.append({'before': prior_full, 'after': current})
                            prior_full = current
                        require(side, 'same Laser adjacent full timer1 to0 before deferred114 grant', expiry)
                        if expiry:
                            retirement(key, identity, expiry[0]['after']['presentation'],
                                       'naturally expired prior Laser')
                except (KeyError, TypeError, IndexError, Unobservable) as error:
                    missing(side, 'queued Laser lifetime and own retirement', str(error), contact['presentation'])
            witnesses.append({'side': side, 'kind': 'actual deferred114 grant slots; no selector cleanup credit',
                              'status': 'observed', 'before': before['slot'], 'after': after['slot'],
                              'secondaryBefore': before['secondary'], 'secondaryAfter': after['secondary']})
        elif queued_lifetime:
            missing(side, 'queued Laser lifetime and deferred114 grant', 'No actual114 count boundary')
        else:
            if laser_boundary:
                before, after = laser_boundary
                view, key, actor = before['view'], before['slot'], before['actor']
                live = False
                try:
                    live = bool(actor and view.identity(key, actor)[:3] == (7, 1, 0)
                                and _boolean(actor['active']) and not _boolean(actor['dead'])
                                and view.manager(key, actor))
                    if live:
                        lifetime = actor['laser']['remainingUpdates'] if side == 'native' else actor['remainingUpdates']
                        # An adjacent full callback can consume one TTL update.
                        # Exclude an indistinguishable natural expiry at the grant.
                        safe_ttl = _integer(lifetime) > (1 if after['view'].counter % 2 == 1 else 0)
                        check(side, 'prior Laser TTL excludes natural expiry at adjacent grant', True,
                              safe_ttl, before['presentation'])
                        if not safe_ttl:
                            missing(side, 'selection-change Remove causality',
                                    'Prior TTL can expire naturally at this callback', after['presentation'])
                        witnesses.append({'side': side, 'kind': 'own live prior Laser at114 boundary',
                                          'status': 'observed', 'identity': view.identity(key, actor),
                                          'before': before['presentation'], 'grant': after['presentation']})
                        replacements = [row for row in observations if row['stage'] == 1 and row['actor']
                                        and row['view'].identity(row['slot'], row['actor'])[:3] == (8, 0, 0)]
                        replacement = replacements[0]['slot'] if replacements else None
                        require(side, 'actual replacement ordinary Knife C18',
                                [row['presentation'] for row in replacements])
                        if replacement is not None:
                            retirement(key, view.identity(key, actor), after['presentation'], 'prior Laser', replacement)
                        else:
                            missing(side, 'replacement-safe prior Laser retirement', 'No actual replacement Knife reference')
                except (KeyError, TypeError, IndexError, Unobservable) as error:
                    missing(side, 'prior live Laser at114', str(error), before['presentation'])
                require(side, 'live registered prior Laser immediately before114',
                        [before['presentation']] if live else [])
            else:
                missing(side, 'prior live Laser replacement lifecycle', 'No actual114 count boundary')
        if not knife_boundary:
            missing(side, 'same Knife329 reuse and preexisting child continuation', 'No actual329 count boundary')
            continue
        before, after = knife_boundary
        view, key, parent = before['view'], before['slot'], before['actor']
        try:
            parent_ok = bool(parent and view.identity(key, parent)[:3] == (8, 0, 0)
                             and view.manager(key, parent) and _boolean(parent['active'])
                             and not _boolean(parent['dead']))
            require(side, 'live ordinary parent Knife before329', [before['presentation']] if parent_ok else [])
            if not parent_ok:
                continue
            parent_identity = view.identity(key, parent)
            check(side, 'preconversion parent bit33 absent', False, bool(view.attack_flags(parent) & (1 << 33)), before['presentation'])
            controlled = [row for row in observations if row['stage'] == 2 and row['actor']
                          and row['slot'] == key and row['view'].identity(key, row['actor']) == parent_identity
                          and row['view'].attack_flags(row['actor']) & (1 << 33)]
            require(side, 'same actual persistent Knife reused with bit33 after329',
                    [row['presentation'] for row in controlled])
            for row in observations:
                if row['stage'] == 2:
                    check(side, 'same primary Knife C18 after329', key, row['slot'], row['presentation'])
            if not controlled:
                continue
            converted = controlled[0]['presentation']
            for phase, remainder in (('full', 1), ('fast', 0)):
                require(side, 'controlled parent actual ' + phase + ' body',
                        [row['presentation'] for row in controlled if row['view'].counter % 2 == remainder])
            children = []
            for child_key, child in before['actors'].items():
                if (view.identity(child_key, child)[:3] == (8, 0, 1)
                        and view.parent(child) == key and view.manager(child_key, child)
                        and _boolean(child['active']) and not _boolean(child['dead'])):
                    children.append((child_key, child))
            require(side, 'actual preexisting ParentKnife subtype1 child before329',
                    [view.identity(child_key, child) for child_key, child in children])
            for child_key, child in children:
                copied, child_identity = view.line(child), view.identity(child_key, child)
                births = [row['presentation'] for row in observations if row['stage'] == 1
                          and child_key in row['births'] and row['slot'] == key
                          and row['view'].parent(row['actors'][child_key]) == key]
                require(side, 'preexisting child actual birth with own ordinary ParentKnife', births)
                continuation, changed_position, full_pairs = [], [], []
                prior_full = None
                for row in observations:
                    if row['presentation'] < converted:
                        continue
                    own = row['view']
                    actual = own.actor(child_key)
                    if actual is None:
                        continue
                    same_identity = check(side, 'preexisting child own identity', child_identity,
                                          own.identity(child_key, actual), row['presentation'])
                    line_retained = check(side, 'preexisting child retains copied curve after conversion',
                                          copied, own.line(actual), row['presentation'])
                    if (same_identity and line_retained and own.parent(actual) == key and own.manager(child_key, actual)
                            and _boolean(actual['active']) and not _boolean(actual['dead'])):
                        continuation.append(row['presentation'])
                        fast = actual['fast322'] if side == 'native' else actual['fast']
                        if own.counter % 2 == 1 and not _boolean(fast):
                            current_full = {'presentation': row['presentation'], 'counter': own.counter,
                                            'position': _vector(actual['position'])}
                            if (prior_full is not None and own.counter == prior_full['counter'] + 2
                                    and row['presentation'] == prior_full['presentation'] + 2):
                                pair = {'before': prior_full, 'after': current_full,
                                        'physicalPositionChanged': prior_full['position'] != current_full['position'],
                                        'sameOwnIdentityAndParent': True, 'copiedLineRetained': True}
                                full_pairs.append(pair)
                                if pair['physicalPositionChanged']:
                                    changed_position.append(pair)
                            prior_full = current_full
                require(side, 'same child active with ParentKnife after controlled conversion', continuation)
                witnesses.append({'side': side, 'kind': 'same child consecutive FULL physical rows after conversion',
                                  'status': 'observed' if full_pairs else 'missing', 'observations': full_pairs})
                require(side, 'same child actual full curve movement after conversion', changed_position)
                retirement(child_key, child_identity, converted, 'preexisting ParentKnife child')
            # This callback route has no ordinary118 secondary emission. A
            # preexisting child is still observed through its own lifetime.
            emissions = []
            for row in observations:
                if row['presentation'] < converted:
                    continue
                for new_key in row['births']:
                    actor = row['actors'][new_key]
                    family = row['view'].identity(new_key, actor)[:3]
                    if family in ((7, 1, 0), (8, 0, 1)):
                        emissions.append({'presentation': row['presentation'], 'ownIdentity': row['view'].identity(new_key, actor)})
            check(side, 'no new ordinary118 child or Laser after controlled conversion', [], emissions)
            witnesses.append({'side': side, 'kind': 'secondary slots during conversion; absence is not cleanup',
                              'status': 'observed', 'before': before['secondary'], 'after': after['secondary'],
                              'nonnullBefore': sum(value is not None for value in before['secondary'])})
            for secondary in before['secondary']:
                if secondary is not None:
                    actual = view.actor(secondary)
                    if actual is None:
                        missing(side, 'nonnull secondary actual cleanup', 'Prior own referenced actor cannot be observed')
                    else:
                        retirement(secondary, view.identity(secondary, actual), converted,
                                   'actually nonnull secondary', key)
        except (KeyError, TypeError, IndexError, Unobservable) as error:
            missing(side, 'controlled conversion or child lifecycle', str(error), before['presentation'])

    for row in ordered if len(sides) == 2 else ():
        for index, name in enumerate(('C18', 'C20', 'C28', 'C30')):
            try:
                native, browser = _View(row, 'native'), _View(row, 'browser')
                left, right = native.slot(index), browser.slot(index)
                check('paired', name + ' actual NULL value', left is None, right is None,
                      row['presentation'])
                if left is not None and right is not None:
                    original, actual = native.actor(left), browser.actor(right)
                    if original is None or actual is None:
                        raise Unobservable('Actual own nonnull slot unresolved')
                    # Match the declared slot role, retaining the independent
                    # sequence/seed comparisons in the body comparer.
                    check('paired', name + ' own referenced family',
                          native.identity(left, original)[:3], browser.identity(right, actual)[:3],
                          row['presentation'])
            except (KeyError, TypeError, IndexError, Unobservable) as error:
                missing('paired', name + ' actual weak-slot observation', str(error), row['presentation'])

    return result
