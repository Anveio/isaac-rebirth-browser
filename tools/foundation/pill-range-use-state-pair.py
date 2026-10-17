"""Fresh Original/browser pocket laws from each game's actual live records.

The capture readback owns transport, complete journals and identical declared
inputs. This comparer owns observed behavior; unavailable evidence never counts
as equality and never becomes an unconditional permanent failing placeholder.
"""
import argparse
import collections
import gzip
import hashlib
import importlib.util
import json
import math
import struct
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--case', type=Path, required=True)
parser.add_argument('--original', type=Path, required=True)
parser.add_argument('--browser', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--wasm-node', action='store_true')
parser.add_argument('--capture-readback', type=Path, required=True)
args = parser.parse_args()
case = json.loads(args.case.read_text())
kind = 'init' if 'pocketInit' in case else 'collection'
contract = case['pocketInit'] if kind == 'init' else case['pocketCollection']
counts, failures = collections.Counter(), collections.Counter()
examples, unavailable, diagnostics = [], [], []


def check(field, actual, expected=True):
    counts[field] += 1
    if type(actual) is not type(expected) or actual != expected:
        failures[field] += 1
        if len(examples) < 80:
            examples.append(dict(field=field, actual=actual, expected=expected))


def require(field, function):
    try:
        return function()
    except (KeyError, TypeError, ValueError, AssertionError, IndexError, OSError) as error:
        check(field, str(error), 'actual typed and unambiguous field')
        return None


def integer(value):
    assert type(value) is int, 'Actual integer owner required'
    return value


def boolean(value):
    assert type(value) is bool, 'Actual bool owner required'
    return value


def f32(value):
    assert type(value) in (int, float) and math.isfinite(value), 'Actual finite numeric owner required'
    return struct.unpack('<f', struct.pack('<f', value))[0]


def finite(value):
    assert type(value) in (int, float) and math.isfinite(value), 'Actual finite numeric owner required'
    return float(value)


def vector(value):
    if isinstance(value, dict):
        value = [value['x'], value['y']]
    assert isinstance(value, list) and len(value) == 2
    return [f32(word) for word in value]


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(65536), b''):
            value.update(chunk)
    return value.hexdigest()


def evidence_module():
    spec = importlib.util.spec_from_file_location('pocket_native_evidence',
        Path(__file__).resolve().parents[1] / 'original/native-evidence.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def native_rows():
    evidence = evidence_module()
    path = evidence.progress_path(args.original.parent)
    assert path is not None, 'Actual closed Native journal required'
    metadata = json.loads(path.with_name(path.name + '.meta.json').read_text())
    evidence.validate_progress_format(path, metadata)
    assert metadata['status'] == 'closed'
    assert digest(path) == metadata['archiveSha256']
    raw_hash, raw_bytes, records, gameplay_start = hashlib.sha256(), 0, 0, None
    with evidence.open_progress(path) as stream:
        for raw in stream:
            raw_hash.update(raw); raw_bytes += len(raw); records += 1
            entry = json.loads(raw)
            if entry['kind'] != 'debugObservation':
                continue
            value = entry['value']; context = value['context']; observation = value['observation']
            if context.get('label') == 'gameplay-start':
                assert gameplay_start in (None, context['presentedFrame'])
                gameplay_start = context['presentedFrame']
            if gameplay_start is None:
                continue
            check('native observer succeeds', observation['status'], 'pass')
            items, world = observation['items'], observation['pickup']
            if not items.get('gamePresent') or not items.get('players'):
                continue
            assert len(items['players']) == len(world['players']) == 1
            player, physical = items['players'][0], world['players'][0]
            slots = player['heldPocketRecords']
            assert len(slots) == 2
            held = []
            for slot in slots:
                pocket_id, tag = integer(slot['id']), integer(slot['typeTagRaw'])
                assert pocket_id >= 0 and 0 <= tag <= 255
                if pocket_id:
                    assert tag in (0, 1), 'Nonempty held record needs an actual bool tag'
                # Empty tags remain raw in the complete journal; no semantic
                # Card/Pill claim or held transition is made from those bytes.
                held.append(dict(id=pocket_id, card=bool(tag) if pocket_id else None))
            bodies = []
            for actor in world['entities']:
                if actor['type'] != 5 or actor['variant'] not in (70, 300):
                    continue
                bodies.append(dict(key=(integer(actor['sequenceId']), integer(actor['spawnSeed'])),
                    seed=integer(actor['spawnSeed']), sequence=integer(actor['sequenceId']),
                    variant=integer(actor['variant']), subtype=integer(actor['subtype']),
                    position=vector(actor['position']), velocity=vector(actor['velocity']),
                    removed=boolean(actor['removed']), animation=actor['animation'],
                    frame=f32(actor['animationFrame']), playing=boolean(actor['animationPlaying']),
                    radius=f32(actor['collisionRadius']), mass=f32(actor['collisionMass']),
                    grid=integer(actor['gridCollisionClass']), entity=integer(actor['entityCollisionClass']),
                    acc=integer(actor['collectionDelay']), ad0=integer(actor['pickupCooldown'])))
            yield dict(gp=integer(context['presentedFrame']) - gameplay_start,
                counter=integer(items['systemCounter']), held=held, bodies=bodies,
                main=player['mainSprite'], copied=player['pickupSprite'],
                guard=boolean(player['pickupAnimationGuard']), controls=boolean(player['controlsEnabled']),
                position=vector(physical['position']), velocity=vector(physical['velocity']),
                logicalVelocity=None,
                audio=observation.get('audio'), items=items, selectors=None, state=None)
    assert (records, raw_bytes, raw_hash.hexdigest()) == (metadata['records'], metadata['rawBytes'], metadata['rawSha256'])
    assert gameplay_start is not None


def browser_rows():
    path = args.browser.parent / ('actual-states.jsonl.gz' if args.wasm_node else 'browser-progress.jsonl.gz')
    metadata = json.loads(path.with_name(path.name + '.meta.json').read_text())
    assert metadata['status'] == 'closed' and digest(path) == metadata['archiveSha256']
    raw_hash, raw_bytes, records, first_room = hashlib.sha256(), 0, 0, None
    with gzip.open(path, 'rb') as stream:
        for raw in stream:
            raw_hash.update(raw); raw_bytes += len(raw); records += 1
            entry = json.loads(raw)
            if entry['kind'] != 'state':
                continue
            value = entry if args.wasm_node else entry['value']; state = value['state']
            if state['screen'] != 'playing':
                continue
            room = integer(state['currentRoom'])
            if first_room is None:
                first_room = room
            if room != first_room:
                continue  # Continue reading/validating the full journal, retire this room's identities.
            owner = state['playerItems']; held = owner['heldPockets']
            assert 'frontier' in held and held['frontier'] is None
            slots = []
            for slot in held['slots']:
                pocket_id, card = integer(slot['id']), boolean(slot['card'])
                assert pocket_id >= 0
                slots.append(dict(id=pocket_id, card=card if pocket_id else None))
            assert len(slots) == 2
            identities = {row['actor']['id']: row['sequenceId'] for row in state['actorScheduler']['identities']
                          if row['actor']['kind'] == 'pickup'}
            members = {row['id'] for row in state['actorScheduler']['actors'] if row['kind'] == 'pickup'}
            bodies = []
            for actor in state['room']['pickups']:
                if actor['variant'] not in (70, 300):
                    continue
                if not actor['removed']:
                    check('browser live pickup has manager identity', actor['id'] in identities and actor['id'] in members)
                collision = actor['regularCollision']
                bodies.append(dict(key=integer(actor['id']), seed=integer(actor['spawnSeed']),
                    sequence=identities.get(actor['id']), variant=integer(actor['variant']), subtype=integer(actor['subtype']),
                    position=vector(actor['position']), velocity=vector(actor['velocity']), removed=boolean(actor['removed']),
                    animation=actor['animation'], frame=f32(actor['animationFrame']), playing=boolean(actor['animationPlaying']),
                    radius=f32(actor['collisionRadius']), mass=f32(actor['collisionMass']),
                    grid=None if collision is None else integer(collision['gridCollisionClass']),
                    entity=None if collision is None else integer(collision['entityCollisionClass']),
                    acc=integer(actor['collectionDelay']), ad0=integer(actor['pocketContactDelay']), actor=actor['actor']))
            presentation = owner['pickupPresentation']
            copied = None if presentation is None else dict(animation='Sparkle',
                animationPlaying=presentation['animationPlaying'], animationFrame=presentation['animationFrame'],
                overlay=presentation['pocket']['overlay'], overlayFrame=presentation['pocket']['overlayFrame'])
            main = dict(animation=state['playerAnimation']['current'], animationFrame=state['playerAnimation']['frame'],
                        animationPlaying=state['playerAnimation']['playing'])
            player = state['player']
            yield dict(gp=integer(value['gameplayPresentation']), counter=integer(state['systemClock']['systemCounter']),
                held=slots, bodies=bodies, main=main, copied=copied, guard=boolean(state['playerAnimation']['guard']),
                controls=boolean(player['controlsEnabled']),
                position=[f32((player['x'] - 32) / .65), f32((player['y'] + 46) / .65)],
                velocity=[f32(player['vx'] / .65), f32(player['vy'] / .65)],
                logicalVelocity=[finite(player['vx']), finite(player['vy'])],
                audio=state['audio'], items=owner, selectors=(state['room']['spawns'] or {}).get('selectors', []), state=state)
    assert (records, raw_bytes, raw_hash.hexdigest()) == (metadata['records'], metadata['rawBytes'], metadata['rawSha256'])


"""Additive real-use laws over the existing fully validated pocket rows.

The caller retains the existing whole-journal readers literally. This module
does not execute a game, infer a private call from an endpoint, replay an old
case, or run the collection-only law across actual consumption.
"""
import math
import struct


def inspect_pill_use(case, side, rows):
    contract = case['pillUse']
    assert contract['schema'] == 'source-public-pill-range-use-v1'
    assert side in ('native', 'own')
    effect, color = contract['effectId'], contract['pillColor']
    assert effect in (11, 12) and 1 <= color <= 9
    actions = case['actions']
    starts = {}
    at = 0
    for action in actions:
        assert action['label'] not in starts
        starts[action['label']] = at
        at += action['presentations']
    q_start = starts[contract['qUseAction']]
    q_end = q_start + next(a['presentations'] for a in actions if a['label'] == contract['qUseAction'])
    shoot_start = starts['public-post-pill-range-shoot-right']
    assert next(a for a in actions if a['label'] == contract['qUseAction'])['keys'] == ['q']
    checks = []
    def check(field, passed, gp=None, detail=None):
        checks.append(dict(field=field, passed=bool(passed), gameplayPresentation=gp, detail=detail))
    def integer(value):
        assert type(value) is int
        return value
    def f32(value):
        assert type(value) in (int,float) and math.isfinite(value)
        return struct.unpack('<f',struct.pack('<f',value))[0]
    initial = None
    prior = None
    before_q = None
    last_gp = None
    stable_post = []
    post_values, post_words = set(), set()
    known_held_rows = neutral_rows = completed_rows = tears = 0
    actual_uses = []
    response_frames = set()
    world_sources, retired_sources, acquired_sources = set(), set(), set()
    world_idle_rows = acquisition_edges = 0
    previous = None
    all_words = None
    for row in rows:
        gp = integer(row['gp'])
        if last_gp is not None:
            check('complete pocket rows remain ordered',gp >= last_gp,gp)
        last_gp = gp
        try:
            held = row['held']
            assert len(held) == 2
            slots = [(integer(slot['id']),slot['card']) for slot in held]
            present, live = set(), set()
            for body in row['bodies']:
                if body['variant'] != 70 or body['subtype'] != color:
                    continue
                key = body['key']
                present.add(key)
                if not body['removed']:
                    live.add(key)
                    world_sources.add(key)
                    world_idle_rows += gp < q_start and body['animation'] == 'Idle'
                elif gp < q_start and key in world_sources:
                    retired_sources.add(key)
            if gp < q_start:
                retired_sources.update(world_sources - present)
            if gp < q_start and previous is not None and previous['slot'][0] == 0 and slots[0] == (color,False):
                acquisition_edges += 1
                acquired_sources.update(previous['livePills'])
                check('actual source world Pill precedes acquired held record',bool(previous['livePills']),gp)
            if side == 'native':
                items = row['items']
                assert len(items['players']) == 1
                owner = items['players'][0]
                pool = items['pillEffects']
                words = owner['persistentItemInputs']
                pair = (integer(words['source13b4']),integer(words['source13b8']))
                d34 = integer(words['sourceD34'])
                counts = owner['ownedCounts']
                stats = owner['cachedStats']
                cache = integer(owner['cacheFlags'])
                completed = cache & 8 == 0
                output_owned = None
                source_words = None
                update = integer(row['counter'])
                shot_count = integer(owner['shotsFired'])
            else:
                state = row['state']
                owner = state['playerItems']
                pool = state['itemPools']['pillEffects']
                wire = owner['pillEffectHistory']
                assert wire['kind'] == 'current' and type(wire['version']) is int and wire['version'] == 1
                source_words = wire['history']['words']
                assert len(source_words) == 28
                pair = (integer(source_words[11]),integer(source_words[12]))
                d34 = integer(owner['rangeInputs']['sourceD34'])
                counts = owner['ownedCounts']
                stats = state['playerStats']
                cache = integer(owner['unevaluatedCacheFlags'])
                output_owned = integer(owner['queryHistory']['cachedCallbacks']) & 8 == 8
                completed = cache & 8 == 0 and output_owned
                update = integer(state['gameUpdateCount'])
                shot_count = integer(state['player']['shotsFired'])
                assert owner['queryHistory']['permanentCounts'] is True
            assert len(pool) == 10 and all(type(value) is int for value in pool)
            assert len(counts) == 347 and all(type(value) is int for value in counts)
            vector = tuple(f32(stats[field]) for field in ('tearHeight','tearFallingSpeed','tearFallingAcceleration'))
            check('independent actual pool resolves carried color to declared effect',pool[color] == effect,gp)
            neutral = counts[75] == 0 and counts[303] == 0
            check('actual neutral Have75 and303 resolver operands',neutral,gp)
            neutral_rows += neutral
            if initial is None:
                initial = dict(gp=gp,slots=slots,words=pair,d34=d34,range=vector,shotCount=shot_count)
                check('actual known empty initial pocket',all(slot[0] == 0 for slot in slots),gp)
                prior = pair
                all_words = tuple(source_words) if source_words is not None else None
            selected = pair[effect-11]
            other = pair[12-effect]
            if gp < q_start:
                check('empty control and acquisition preserve actual canonical Pill operands',pair == prior,gp)
                if slots[0] == (color,False):
                    known_held_rows += 1
                    before_q = dict(gp=gp,slots=slots,words=pair,d34=d34,range=vector,
                                    main=row['main'],shotCount=shot_count,update=update)
                if gp >= q_start-4:
                    check('actual known carried Pill immediately before declared Q',slots[0] == (color,False),gp)
            if gp >= q_start and previous is not None:
                if previous['slot'] == (color,False) and slots[0][0] == 0:
                    actual_uses.append(dict(gp=gp,beforeGp=previous['gp'],beforeWords=previous['words'],afterWords=pair))
                    check('consumption follows delivered Q window',q_start <= gp <= q_end+4,gp)
                    expected = list(prior)
                    value = expected[effect-11]
                    expected[effect-11] = ((value+1+(1<<31))%(1<<32))-(1<<31)
                    check('actual independent selected Pill counter increments once',pair == tuple(expected),gp)
            if gp >= q_start and row['main']['animation'] == ('Happy' if effect == 12 else 'Sad'):
                response_frames.add(gp)
            if gp >= q_end+4:
                expected = list(prior)
                expected[effect-11] = ((expected[effect-11]+1+(1<<31))%(1<<32))-(1<<31)
                check('post-use canonical counter pair stays actual',pair == tuple(expected),gp)
                check('post-use pocket remains consumed',all(slot[0] == 0 for slot in slots),gp)
                check('real Pill preserves independent D34',d34 == initial['d34'],gp)
                if source_words is not None:
                    check('real Pill preserves every other independent canonical word',
                        all(source_words[index] == all_words[index] for index in range(28) if index != effect),gp)
                if completed:
                    completed_rows += 1
                    stable_post.append(update)
                    post_values.add(vector)
                    post_words.add(pair)
                if gp >= shoot_start and shot_count > initial['shotCount']:
                    tears += 1
            previous = dict(gp=gp,slot=slots[0],words=pair,livePills=live)
        except (KeyError,TypeError,ValueError,AssertionError,IndexError) as error:
            check('required actual canonical Pill use owner/readback',False,gp,str(error))
    check('actual known held Pill acquired before Q',known_held_rows > 0)
    check('actual source world Pill Idle observed',world_idle_rows > 0)
    check('actual empty-to-held acquisition edge observed',acquisition_edges == 1)
    check('actual source Pill retires after held acquisition',bool(acquired_sources) and acquired_sources <= retired_sources)
    check('actual immediate pre-Q held owner exists',before_q is not None)
    if before_q is not None:
        main = before_q['main']
        check('collection UseItem ceremony finished before Q',
              main['animation'] != 'UseItem' or main['animationPlaying'] is False)
    check('actual positive Q consumes exactly one Pill',len(actual_uses) == contract['minimumActualUses'])
    check('actual response main sprite observed',len(response_frames) > 0)
    check('actual completed Range output observed after use',completed_rows >= 8)
    check('actual continued post-use updates',len(stable_post) > 1 and max(stable_post)-min(stable_post) >= 8)
    check('one actual stable cached Range vector after use',len(post_values) == 1)
    check('actual post-use public shooting increments shots',tears > 0)
    check('complete final released tape observed',last_gp is not None and last_gp >= at-2)
    return dict(side=side,case=case['name'],effectId=effect,pillColor=color,
        initial=initial,immediateBeforeQ=before_q,actualUses=actual_uses,
        postRange=[list(value) for value in sorted(post_values)],postWords=[list(value) for value in sorted(post_words)],
        completedRows=completed_rows,responseRows=len(response_frames),postShootingRows=tears,
        worldPillSources=len(world_sources),worldIdleRows=world_idle_rows,
        actualAcquisitionEdges=acquisition_edges,retiredAcquiredSources=len(acquired_sources & retired_sources),
        checks=checks,requiredFailures=sum(not row['passed'] for row in checks),
        unknowns=['Private callback invocation and synchronous ordering',
            'Native independent output-ownership receipt','Pill time/progression/playerBE8',
            'RangeUp color/timed tint','Actual tear birth-parameter snapshots',
            'Full-frame pixels/backend played sound/PCM','Other Pill/Card effects'])


def pair_pill_use(case,native,own):
    checks = []
    def check(field,actual,expected):
        checks.append(dict(field=field,actual=actual,expected=expected,passed=actual == expected))
    check('independent Native real-use laws',native['requiredFailures'],0)
    check('independent Own real-use laws',own['requiredFailures'],0)
    native_initial,own_initial = native.get('initial'),own.get('initial')
    check('actual pre-use canonical Pill owners available',native_initial is not None and own_initial is not None,True)
    check('actual pre-use canonical Pill pair',(native_initial or {}).get('words'),(own_initial or {}).get('words'))
    check('actual post-use canonical Pill pair',native['postWords'],own['postWords'])
    check('actual post-use C8C/C90/C94 binary32 vector',native['postRange'],own['postRange'])
    return dict(case=case['name'],checks=checks,
        requiredFailures=sum(not row['passed'] for row in checks),
        unknowns=['Private callback entry','Per-presentation or actor identity alignment',
                  'Full presentation/progression/time/color/audio parity'],
        scope='Fresh same-declared-case actual counter, consumption and cached Range outputs. '
              'No formula, pending-mask endpoint law, roster whitelist or invocation inference.')

assert case['pillUse']['schema'] == 'source-public-pill-range-use-v1'
readback = json.loads(args.capture_readback.read_text())
check('actual complete capture readback passes', readback['status'], 'pass')
for path, expected in readback['protectedArchivePins'].items():
    check('actual archive still equals capture readback', digest(Path(path)), expected)
native_case_path = args.original.parent / 'case.json'
check('independent Native declaration equals public case', json.loads(native_case_path.read_text()), case)
inputs = [args.case, args.original, args.browser, args.capture_readback, native_case_path]
inputs.extend(Path(path) for path in readback['protectedArchivePins'])
before = {str(path): digest(path) for path in inputs}
native = require('actual Native Pill use owners', lambda: inspect_pill_use(case, 'native', native_rows()))
own = require('actual Own Pill use owners', lambda: inspect_pill_use(case, 'own', browser_rows()))
check('both actual use reports exist', native is not None and own is not None)
paired = None
if native is not None and own is not None:
    paired = pair_pill_use(case, native, own)
    check('actual independent real-use and pair laws pass', paired['requiredFailures'], 0)
after = {str(path): digest(path) for path in inputs}
check('all input archives before equal end', after, before)
unknown = list(case['pillUse']['unknown'])
for report in (native, own, paired):
    if report is not None:
        unknown.extend(report['unknowns'])
result = dict(schema=1, status='fail' if failures else 'pass', case=case['name'],
    checks=sum(counts.values()), failures=sum(failures.values()),
    checksPerField=dict(counts), failedPerField=dict(failures), examples=examples,
    native=native, own=own, pair=paired, unavailable=list(dict.fromkeys(unknown)),
    inputsBefore=before, inputsAfter=after,
    scope='Fresh same-declared-case actual Pill counters, consumption and completed cached Range. '
          'Existing complete public capture/journal readers; no collection law across consumption, '
          'no private invocation/order or tear-birth/render/backend claim.')
if args.wasm_node:
    result['captureBackend'] = 'wasm-node'
    result['unavailable'].extend(['DOM inputs', 'Browser renderer and PNGs', 'Audio backend and PCM'])
raw = (json.dumps(result, indent=2, allow_nan=False) + '\n').encode()
args.output.write_bytes(gzip.compress(raw, mtime=0) if args.output.suffix == '.gz' else raw)
print(json.dumps(dict(status=result['status'], checks=result['checks'], failures=result['failures'])))
raise SystemExit(bool(failures))
