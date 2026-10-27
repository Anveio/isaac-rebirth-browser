"""Actual public Ctrl11 game-level laws over independent complete journals.

Canonical readback owns physical inputs, clocks and resources. This reader
requires actual counter/record/body owners and all declared branch witnesses;
final values never certify entry into a private helper or its call ordering.
"""
import argparse
import collections
import gzip
import hashlib
import importlib.util
import json
import struct
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--case', type=Path, required=True)
p.add_argument('--original', type=Path, required=True)
p.add_argument('--browser', type=Path, required=True)
p.add_argument('--capture-readback', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--wasm-node', action='store_true')
a = p.parse_args()
case = json.loads(a.case.read_text())
checks, errors = [], []
unowned_observations = []


def check(field, actual, expected=True, gp=None):
    checks.append(dict(field=field, actual=actual, expected=expected, gp=gp,
        passed=type(actual) is type(expected) and actual == expected))


def integer(value):
    assert type(value) is int
    return value


def action_owner(player):
    """Retain missing/unknown fields as observations, never manufactured zeros."""
    owner = player.get('pocketActions')
    available = type(owner) is dict
    fields = owner if available else {}
    return available, fields


def unowned_initial_counter(side, row):
    #The fresh constructor has no proven3054 initializer. Only an explicit
    #owned null at GP0 may precede52F220's first released callback. A missing
    #owner/key, or any null after GP0, remains a required failure.
    return (side == 'own' and row['gp'] == 0 and row['ownerAvailable']
        and row['heldFieldPresent'] and row['held'] is None)


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for raw in iter(lambda: stream.read(65536), b''):
            h.update(raw)
    return h.hexdigest()


def f32(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


def journal(path, opener):
    meta = json.loads(path.with_name(path.name + '.meta.json').read_text())
    assert meta['status'] == 'closed', 'Actual complete journal required'
    assert digest(path) == meta['archiveSha256']
    h, size, count = hashlib.sha256(), 0, 0
    with opener(path) as stream:
        for raw in stream:
            h.update(raw); size += len(raw); count += 1
            yield json.loads(raw)
    assert (count, size, h.hexdigest()) == (meta['records'], meta['rawBytes'], meta['rawSha256'])


def native_rows():
    reader = module('ctrl_native_journal', Path(__file__).resolve().parents[1] / 'original/native-evidence.py')
    path = reader.progress_path(a.original.parent)
    assert path is not None
    reader.validate_progress_format(path, json.loads(path.with_name(path.name + '.meta.json').read_text()))
    start, rows = None, {}
    for entry in journal(path, reader.open_progress):
        if entry['kind'] != 'debugObservation':
            continue
        value = entry['value']; context = value['context']; observation = value['observation']
        if context.get('label') == 'gameplay-start':
            assert start in (None, context['presentedFrame'])
            start = context['presentedFrame']
        if start is None:
            continue
        assert observation['status'] == 'pass'
        items, world = observation['items'], observation['pickup']
        assert items['gamePresent'] and len(items['players']) == 1
        player = items['players'][0]
        gp = integer(context['presentedFrame']) - start
        owner_available, owner = action_owner(player)
        bodies = []
        for actor in world['entities']:
            if actor['type'] == 5 and actor['variant'] in (300, 70):
                bodies.append(dict(key=(actor['sequenceId'], actor['spawnSeed']), seed=actor['spawnSeed'],
                    variant=actor['variant'], subtype=actor['subtype'], removed=actor['removed'],
                    ad0=actor['pickupCooldown'], position=[f32(x) for x in actor['position']],
                    source=actor.get('pocketSpawnSource')))
        row = dict(gp=gp, ownerAvailable=owner_available,
            ownerFieldsPresent=all(field in owner for field in ('sourceD98','heldCallbacks3054','source30d0','trinketSlots','queuedConfigKind')),
            heldFieldPresent='heldCallbacks3054' in owner,
            device=owner.get('sourceD98'), held=owner.get('heldCallbacks3054'), marker=owner.get('source30d0'),
            trinkets=owner.get('trinketSlots'), queueKind=owner.get('queuedConfigKind'),
            queuedId=player.get('queuedId'), cacheFlags=player.get('cacheFlags'),
            slots=[(integer(x['id']), integer(x['typeTagRaw'])) for x in player['heldPocketRecords']],
            bodies=bodies, controls=player['controlsEnabled'], inputCooldown=player['inputCooldown'])
        if gp in rows:
            assert rows[gp] == row, 'Duplicate capture at one public presentation changed'
        rows[gp] = row
    assert start is not None
    return rows


def own_rows():
    path = a.browser.parent / ('actual-states.jsonl.gz' if a.wasm_node else 'browser-progress.jsonl.gz')
    rows = {}
    for entry in journal(path, lambda path: gzip.open(path, 'rb')):
        if entry['kind'] != 'state':
            continue
        value = entry if a.wasm_node else entry['value']; state = value['state']
        if state['screen'] != 'playing':
            continue
        gp = integer(value['gameplayPresentation'])
        player, items = state['player'], state['playerItems']
        owner_available, owner = action_owner(player)
        bodies = [dict(key=actor['id'], seed=actor['spawnSeed'], variant=actor['variant'], subtype=actor['subtype'],
            removed=actor['removed'], ad0=actor['pocketContactDelay'], position=[f32(actor['position'][axis]) for axis in ('x','y')],
            source=actor.get('sourceSpawner')) for actor in state['room']['pickups'] if actor['variant'] in (70, 300)]
        row = dict(gp=gp, ownerAvailable=owner_available,
            ownerFieldsPresent=all(field in owner for field in ('sourceD98','heldCallbacks3054','source30d0','frontier')),
            heldFieldPresent='heldCallbacks3054' in owner,
            device=owner.get('sourceD98'), held=owner.get('heldCallbacks3054'), marker=owner.get('source30d0'),
            ownerFrontier=owner.get('frontier'), trinkets=items.get('trinketSlots'),
            queuedId=items.get('queuedId'), cacheFlags=items.get('unevaluatedCacheFlags'),
            slots=[(integer(x['id']), int(x['card'])) for x in items['heldPockets']['slots']],
            bodies=bodies, controls=player['controlsEnabled'], inputCooldown=player['inputCooldown'])
        assert gp not in rows
        rows[gp] = row
    return rows


def inspect(side, rows):
    at, windows = case['initialPhaseOffset'], {}
    for action in case['actions']:
        assert action['label'] not in windows
        windows[action['label']] = (at+1, at+action['presentations'])
        at += action['presentations']
    check(side+' complete public tape observed', max(rows, default=-1), at)
    check(side+' every public presentation observed', all(gp in rows for gp in range(1, at+1)))
    for gp, row in rows.items():
        check(side+' actual pocketActions owner available', row['ownerAvailable'], True, gp)
        check(side+' actual pocketActions fields present', row['ownerFieldsPresent'], True, gp)
        if not unowned_initial_counter(side, row):
            check(side+' actual heldCallbacks3054 uint32 available',
                type(row['held']) is int and 0 <= row['held'] <= 0xffffffff, True, gp)
        check(side+' actual Keyboard sourceD98', row['device'], 0, gp)
        check(side+' actual source30d0 remains constructor/reset zero', row['marker'], 0, gp)
        check(side+' actual empty held trinkets', row['trinkets'], [0, 0], gp)
        check(side+' actual null queued item', row['queuedId'], None, gp)
        check(side+' actual pending cache zero', row['cacheFlags'], 0, gp)
        if side == 'native':
            check(side+' actual null queued kind', row['queueKind'], None, gp)
        else:
            check(side+' actual pocketActions frontier absent', row['ownerFrontier'], None, gp)
        check(side+' actual live input gate', row['controls'], True, gp)
        check(side+' actual live input cooldown', row['inputCooldown'] <= 0, True, gp)
    def row_at(label, end=True):
        return rows[windows[label][int(end)]]
    initial = row_at('released-initial-card-appear')
    check(side+' initial released actual counter', initial['held'], 0)
    check(side+' actual initial pockets empty', [slot[0] for slot in initial['slots']], [0, 0])
    before = rows[windows['ctrl-tap'][0]-1]
    check(side+' natural Card1 acquired before Ctrl', before['slots'], [(1,1),(0,1)])
    sources = {body['key'] for body in initial['bodies'] if body['variant'] == 300 and body['subtype'] == 1 and not body['removed']}
    retired = {body['key'] for row in rows.values() for body in row['bodies'] if body['removed']}
    retired |= sources - {body['key'] for body in before['bodies'] if not body['removed']}
    check(side+' declared natural Card1 body actually observed', len(sources), 1)
    check(side+' actual source Card1 retirement observed', bool(sources) and sources <= retired)
    for label, expected in [('ctrl-tap',1), ('ctrl-tap-release',0), ('ctrl-hold-120',120),
            ('ctrl-hold-121-drop',0), ('ctrl-drop-release',0), ('ctrl-empty-hold-120',120),
            ('ctrl-empty-hold-121',0), ('ctrl-empty-release',0)]:
        check(side+' actual counter '+label, row_at(label)['held'], expected)
    for label in ('ctrl-tap','ctrl-tap-release','ctrl-hold-120'):
        check(side+' Card retained '+label, row_at(label)['slots'], before['slots'])
    hold_start, hold_end = windows['ctrl-hold-120']
    for gp in range(hold_start,hold_end+1):
        check(side+' every actual held callback increments', rows[gp]['held'], gp-hold_start+1, gp)
        check(side+' every pre121 callback retains Card', rows[gp]['slots'], before['slots'], gp)
    dropped = row_at('ctrl-hold-121-drop')
    check(side+' actual121 clears held IDs only', dropped['slots'], [(0,1),(0,1)])
    new = [body for body in dropped['bodies'] if body['key'] not in sources and not body['removed']]
    check(side+' actual121 new Card body', len(new), 1)
    if len(new) == 1:
        body = new[0]
        check(side+' actual dropped Card family/subtype', [body['variant'],body['subtype']], [300,1])
        check(side+' actual positive drop AD0', body['ad0'] > 0)
        if side == 'native':
            check(side+' actual null drop source', body['source'], dict(entityType=0,variant=0,parentPointerIsNull=True))
        else:
            check(side+' actual null drop source', body['source'], dict(entityType=0,variant=0))
    for label in ('ctrl-drop-release','released-away-from-card','ctrl-empty-hold-120','ctrl-empty-hold-121','ctrl-empty-release'):
        check(side+' actual empty held '+label, [slot[0] for slot in row_at(label)['slots']], [0,0])
    empty_start, empty_end = windows['ctrl-empty-hold-121']
    prior_live = {body['key'] for body in rows[empty_start-1]['bodies'] if not body['removed']}
    after_live = {body['key'] for body in rows[empty_end]['bodies'] if not body['removed']}
    check(side+' actual empty121 creates no new body', sorted(after_live), sorted(prior_live))
    return dict(side=side, presentations=len(rows), naturalSources=len(sources), dropBodies=len(new),
        drop=None if len(new)!=1 else {key:new[0][key] for key in ('seed','variant','subtype','position')})


summaries = []
try:
    assert case['pocketActions']['schema'] == 'source-public-pocket-actions11-v1'
    check('canonical independent input/clock/resource readback', json.loads(a.capture_readback.read_text())['status'], 'pass')
    native, own = native_rows(), own_rows()
    summaries = [inspect('native',native),inspect('own',own)]
    check('paired actual new drop seed/family/subtype/position', summaries[1]['drop'], summaries[0]['drop'])
    for gp in sorted(set(native) | set(own)):
        if gp not in native or gp not in own:
            check('paired actual presentation available', False, True, gp)
            continue
        for field in ('device','held','marker','slots'):
            if field == 'held' and unowned_initial_counter('own', own[gp]):
                unowned_observations.append(dict(gp=gp, field='Player+3054', native=native[gp][field], own=own[gp][field],
                    reason='Fresh own constructor has no proven3054 initializer; GP0 precedes the first released Player callback. No equality or zero is inferred.'))
                continue
            check('paired actual '+field, own[gp][field], native[gp][field], gp)
        #Source seeds and exact initial drop position are observable separately
        #from later per-actor integration and identities.
        for side, rows in (('native',native),('own',own)):
            assert rows[gp]['gp'] == gp
except Exception as error:
    errors.append(repr(error))
    check('required actual Ctrl owners and complete evidence', False)
failed = [row for row in checks if not row['passed']]
result = dict(schema=1,status='pass' if not failed else 'fail',case=case['name'],
    checks=checks,requiredFailures=len(failed),errors=errors,sides=summaries,
    unownedObservations=unowned_observations,
    counts=dict(collections.Counter(row['field'] for row in failed)),
    unavailable=case.get('pocketActions',{}).get('unavailable',[]),
    scope='Public freshKeyboard emptytrinket naturalCard1 Ctrl tap/120/121/drop/release/emptyhold actual owners. '
        'Private helper entry, two radius0 calls, queue/removal invocation ordering, whole sharedMT, '
        'positive trinkets, capacity2 swap, GnawedLeaf, pixels and backendPCM are not certified by endpoint values.')
a.output.parent.mkdir(parents=True,exist_ok=True)
with gzip.open(a.output,'wt') as stream:
    json.dump(result,stream,separators=(',',':'))
print(json.dumps(dict(status=result['status'],checks=len(checks),requiredFailures=len(failed),errors=errors)))
raise SystemExit(0 if result['status']=='pass' else 1)
