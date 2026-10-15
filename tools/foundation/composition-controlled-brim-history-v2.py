"""Stable-owner property transitions; snapshots do not prove reapply callbacks."""
import copy
import importlib.util
from pathlib import Path
import struct

CONTRACT = 'controlled-brim-properties-priority-v2'
PREDECESSOR = 'controlled-brim-properties-priority-v1'


def compare_history(case, paired_rows, *, sides, shared):
    if case['requires']['historyWitnessContract'] != CONTRACT:
        raise ValueError('Explicit controlled-Brim v2 marker required')
    path = Path(__file__).with_name('composition-controlled-brim-history.py')
    spec = importlib.util.spec_from_file_location('controlled_brim_v1_unchanged', path)
    legacy = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(legacy)
    # Only the policy marker is adapted for the unchanged predecessor reader.
    # Actual case inputs/observations and every unrelated body check are intact.
    predecessor_case = copy.deepcopy(case)
    predecessor_case['requires']['historyWitnessContract'] = PREDECESSOR
    result = legacy.compare_history(predecessor_case, paired_rows, sides=sides, shared=shared)
    obsolete = {'controlled Brim history ' + side + ' current C88/CCC/C98 reapplied'
                for side in sides}
    removed = [row for row in result['checks'] if row['field'] in obsolete]
    result['checks'] = [row for row in result['checks'] if row['field'] not in obsolete
                        and row['field'] != 'controlled Brim history declared marker']
    result['scope']['contract'] = CONTRACT
    result['scope']['predecessorContract'] = PREDECESSOR
    result['scope']['publicReplay'] = dict(result['scope']['publicReplay'])
    result['scope']['propertyEvidence'] = (
        'Dense same-own-C18 actor property transition with unchanged cache owners '
        'is a field-state witness only. It is not an observed callback or instruction write.')
    result['historicalPredecessorChecks'] = removed
    result['supersededObligations'] = {
        'fields': sorted(obsolete), 'count': len(removed),
        'reason': 'Player shooting precedes queued Add. End-state guards/new cache '
                  'do not prove the earlier callback consumed those owners.'}

    def check(side, name, expected, actual, t=None):
        passed = type(expected) is type(actual) and expected == actual
        result['checks'].append({'field': 'controlled Brim v2 ' + side + ' ' + name,
            'presentation': t, 'expected': expected, 'actual': actual, 'passed': passed})
        return passed

    def missing(side, name, reason):
        check(side, name, 'observed', 'unobservable: ' + reason)
        result['coverageGaps'].append({'side': side, 'field': name,
                                      'presentation': None, 'reason': reason})

    check('declared', 'marker', CONTRACT, case['requires']['historyWitnessContract'])
    View, flags, integer = shared['_View'], shared['_flags'], shared['_integer']
    boolean = shared['_boolean']
    post_grant_counts = [0] * 347
    for entry in case['requires']['actorStages'][1]['ownedCounts']:
        identifier = integer(entry['itemId'])
        if not 0 <= identifier < 347:
            raise ValueError('Declared item outside actual inventory')
        post_grant_counts[identifier] = integer(entry['count'])

    def color(value):
        words = value if isinstance(value, list) else sum(
            [value[key] for key in ('tint', 'sourceMiddle', 'offsets')], [])
        if len(words) != 11:
            raise ValueError('Actual eleven-component color owner required')
        return tuple(legacy._f32(word) for word in words)

    def owners(view):
        if view.side == 'native':
            return (legacy._f32(view.player['cachedStats']['damage']),
                    flags(view.owner['attackFlagsC98']), color(view.owner['cachedColorCCC']))
        return (legacy._f32(view.state['playerStats']['damage']),
                flags(view.state['weaponCache']['flagsC98']), color(view.state['weaponCache']['colorCcc']))

    def properties(view, actor):
        if view.side == 'native':
            return (legacy._f32(actor['collisionDamage']),
                    flags(actor['laser']['attackFlags']), color(actor['currentColor2d0']))
        return (legacy._f32(actor['collisionDamage']),
                flags(actor['attackFlags']), color(actor['currentColor']))

    for side in sides:
        # Keep only equal duplicates as eligible causal boundaries. Conflicting
        # duplicates already fail in the predecessor and supply no transition.
        grouped = {}
        for row in paired_rows:
            grouped.setdefault(integer(row['presentation']), []).append(row)
        observed, conflicts = {}, set()
        signatures = ('source', 'nativePlayer', 'nativeCounter') if side == 'native' else ('state',)
        for t, rows in grouped.items():
            if any(any(row[key] != rows[0][key] for key in signatures) for row in rows[1:]):
                conflicts.add(t)
            else:
                observed[t] = rows[0]
        transitions = []
        for t in sorted(observed):
            if t - 1 not in observed or t in conflicts or t - 1 in conflicts:
                continue
            try:
                before, after = View(observed[t - 1], side), View(observed[t], side)
                first, second = before.slot(), after.slot()
                if first is None or first != second:
                    continue
                a, b = before.actor(first), after.actor(second)
                if a is None or b is None:
                    continue
                if before.identity(first, a) != after.identity(second, b):
                    continue
                if before.identity(first, a)[:3] != (7, 1, 1):
                    continue
                if not before.manager(first, a) or not after.manager(second, b):
                    continue
                if (before.counts != post_grant_counts or after.counts != post_grant_counts
                        or before.primary != 2 or after.primary != 2
                        or before.pending != 0 or after.pending != 0
                        or not boolean(a['active']) or not boolean(b['active'])
                        or boolean(a['dead']) or boolean(b['dead'])
                        or before.counter + 1 != after.counter):
                    continue
                owned_before, owned_after = owners(before), owners(after)
                old, new = properties(before, a), properties(after, b)
                if owned_before == owned_after and old != new and new == owned_after:
                    transitions.append({'beforePresentation': t - 1, 'presentation': t,
                        'ownReference': first, 'actualIdentity': before.identity(first, a),
                        'stableCacheOwners': owned_after, 'priorProperties': old,
                        'laterProperties': new,
                        'scope': 'Observed property state transition only; no inferred callback delivery.'})
            except (KeyError, TypeError, ValueError, IndexError, OverflowError, struct.error) as error:
                missing(side, 'stable-owner transition input', str(error))
        check(side, 'required dense stable-owner property-state transition', True, bool(transitions))
        result['witnesses'].append({'side': side,
            'kind': 'dense stable-owner property-state transition; not callback delivery',
            'status': 'observed' if transitions else 'missing', 'observations': transitions})
        if not transitions:
            result['coverageGaps'].append({'side': side, 'field': 'stable-owner property-state transition',
                'presentation': None, 'reason': 'No actual dense transition under stable same-actor/cache owners.'})
        missing(side, 'actual admitted reapply callback after new cache available',
                'These snapshots contain no separately bound 51C73C/51C777 callback witness. '
                'End-state guards and matching/changed properties cannot supply it.')
    return result
