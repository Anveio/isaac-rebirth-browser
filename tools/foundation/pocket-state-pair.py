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


def observe(side, rows):
    initial, histories, drops, requests, contacts = {}, [], [], [], []
    previous, previous_slot, previous_sound = None, None, {}
    seen_frames, seen_drops, source_keys, seen_events = set(), set(), set(), set()
    retired_sources = set()
    groups = contract['groups']
    last_gp, full, fast, refreshes, empty_edges, sound_overrides = None, 0, 0, 0, 0, 0
    last_drop_ad0 = {}
    for row in rows:
        if row['gp'] in seen_frames:
            continue
        seen_frames.add(row['gp']); last_gp = row['gp']
        full += row['counter'] % 2 == 1; fast += row['counter'] % 2 == 0
        for index, group in enumerate(groups):
            expected = group['entries'][0]
            if index not in initial and row['gp'] <= case['actions'][0]['presentations']:
                if side == 'native':
                    point = [f32(80 + group['x'] * 40), f32(160 + group['y'] * 40)]
                    matches = [body for body in row['bodies'] if body['position'] == point and body['variant'] == expected['variant']]
                else:
                    selected = [selector for selector in row['selectors'] if selector['groupIndex'] == contract['originalRoomGroupCount'] + index]
                    check(side + ' declared selector unique', len(selected) <= 1)
                    matches = [] if not selected else [body for body in row['bodies'] if body['seed'] == selected[0]['spawnSeed']]
                check(side + ' declared source body unambiguous', len(matches) <= 1)
                if len(matches) != 1:
                    continue
                body = matches[0]; initial[index] = dict(body, idle=False); source_keys.add(body['key'])
                check(side + ' declared source variant', body['variant'], expected['variant'])
                if expected['subtype']:
                    check(side + ' explicit source subtype', body['subtype'], expected['subtype'])
                else:
                    check(side + ' resolved random subtype domain', 1 <= body['subtype'] <= (40 if expected['variant'] == 300 else 9))
                check(side + ' source radius', body['radius'], f32(12))
                check(side + ' source mass', body['mass'], f32(3))
                check(side + ' source initial AD0', body['ad0'], 0)
                if side == 'browser' and body['variant'] == 300:
                    card = body['subtype']
                    family = ('tarot card' if card <= 22 else 'rune1' if 28 <= card <= 31
                              else 'rune2' if 32 <= card <= 35 else 'suit card')
                    number = {'tarot card': 301, 'suit card': 302, 'rune1': 303, 'rune2': 304}[family]
                    check('browser actual Card visual family', body['actor'], 'gfx/005.' + str(number) + '_' + family + '.anm2')
            if index not in initial:
                continue
            source = initial[index]
            if source['key'] in retired_sources:
                continue
            matches = [body for body in row['bodies'] if body['key'] == source['key']]
            check(side + ' own source handle unique', len(matches) <= 1)
            if not matches:
                retired_sources.add(source['key'])
            if matches:
                body = matches[0]
                check(side + ' source seed stable', body['seed'], source['seed'])
                if not body['removed']:
                    check(side + ' source sequence stable', body['sequence'], source['sequence'])
                if body['animation'] == 'Idle':
                    source['idle'] = True
                    check(side + ' idle class', body['entity'], 4)
                    check(side + ' idle grid class', body['grid'], 5)
        slot = row['held'][0]
        check(side + ' second slot empty at capacity1', row['held'][1]['id'], 0)
        changed = previous_slot is not None and slot != previous_slot
        new_drops = [body for body in row['bodies'] if body['key'] not in source_keys
                     and body['key'] not in seen_drops and not body['removed']
                     and changed and previous_slot is not None and previous_slot['id'] != 0
                     and body['key'] not in {old['key'] for old in previous['bodies']}]
        for body in new_drops:
            seen_drops.add(body['key']); drops.append(dict(body, gp=row['gp']))
            check(side + ' spill has actual held transition', changed and previous_slot['id'] != 0)
            if previous_slot is not None:
                check(side + ' spill old slot id', body['subtype'], previous_slot['id'])
                check(side + ' spill old slot tag', body['variant'], 300 if previous_slot['card'] else 70)
            check(side + ' spill initial AD0', body['ad0'], 4)
            check(side + ' spill zero velocity', body['velocity'], [f32(0), f32(0)])
        for body in row['bodies']:
            if body['key'] not in seen_drops:
                continue
            if body['ad0'] == 10 and last_drop_ad0.get(body['key']) != 10:
                refreshes += 1
                check(side + ' AD0 refresh occurs on Full', row['counter'] % 2, 1)
            last_drop_ad0[body['key']] = body['ad0']
        if changed:
            copied = row['copied']
            histories.append(dict(slot=slot, gp=row['gp'], counter=row['counter'], copied=copied,
                main=row['main'], guard=row['guard'], position=row['position'], velocity=row['velocity']))
            check(side + ' held transition nonempty', slot['id'] > 0)
            check(side + ' collect main UseItem', row['main']['animation'], 'UseItem')
            check(side + ' collect main forced frame0', f32(row['main']['animationFrame']), f32(0))
            check(side + ' collect main playing', row['main']['animationPlaying'], True)
            check(side + ' collect guard', row['guard'])
            check(side + ' collect retains public controls', row['controls'])
            check(side + ' copied Sprite owner', isinstance(copied, dict))
            if isinstance(copied, dict):
                check(side + ' copied Sparkle', copied['animation'], 'Sparkle')
                check(side + ' copied Sparkle playing', copied['animationPlaying'], True)
                check(side + ' copied Sparkle initial frame0', f32(copied['animationFrame']), f32(0))
                overlay = 'Pills' if not slot['card'] else 'Runes' if 28 <= slot['id'] <= 35 else 'CardFronts'
                requested = slot['id'] - 1 if overlay == 'Pills' or slot['id'] <= 27 else int(slot['id'] > 31) if overlay == 'Runes' else slot['id'] - 8
                check(side + ' copied static overlay', copied['overlay'], overlay)
                check(side + ' copied overlay frame', f32(copied['overlayFrame']), f32(min(requested, 31) if overlay == 'CardFronts' else requested))
            if previous_slot['id']:
                check(side + ' full capacity spills exactly once', len(new_drops), 1)
            sound_id = 8 if slot['card'] else 252
            audio = row['audio']
            raw_override = integer(row['items']['rawGameA1D8']) if side == 'native' else None
            if raw_override == 7:
                sound_overrides += 1
                unavailable.append(dict(field='native collection override sound', gp=row['gp'], rawGameA1D8=raw_override,
                    reason='Actual source override branch reached; the narrow8/252 observer cannot identify its replacement slot or extra draw.'))
            else:
                check(side + ' collection audio owner', isinstance(audio, dict))
            if isinstance(audio, dict) and raw_override != 7:
                slots = audio['slots']
                sound = next((s for s in slots if s['id'] == sound_id), None) if isinstance(slots, list) else slots.get(str(sound_id))
                check(side + ' collection sound slot owner', isinstance(sound, dict))
                if isinstance(sound, dict):
                    tick = integer(sound['lastTick'])
                    check(side + ' fresh collection sound admission', previous_sound.get(sound_id) != tick)
                    check(side + ' collection request volume', f32(sound['lastVolume']), f32(1))
                    check(side + ' collection request delay', (integer(sound['nextAllowed']) - tick) & 0xffffffff, 2)
                    request = dict(id=sound_id, gp=row['gp'], lastTick=tick, volume=sound['lastVolume'], delay=2)
                    if side == 'browser':
                        events = [event for event in audio['events'] if event['id'] == sound_id
                                  and event['halfTick'] == tick and event['sequence'] not in seen_events]
                        check(side + ' unique current collection event', len(events), 1)
                        if len(events) == 1:
                            event = events[0]; seen_events.add(event['sequence'])
                            check(side + ' sound command owner', event['systemCounter'], (row['counter'] - 1) & 0xffffffff)
                            request.update(sample=integer(event['sampleIndex']), gain=f32(event['gain']), pitch=f32(event['pitch']), looping=boolean(event['looping']))
                    else:
                        check('native collection sound has same blocked counter', integer(audio['systemCounter']), row['counter'])
                        signed_counter = row['counter'] if row['counter'] < 1 << 31 else row['counter'] - (1 << 32)
                        check('native current collection sound slot tick', tick, signed_counter >> 1)
                        check(side + ' actual public SFX preference', f32(audio['globalSfxVolume']), f32(case['sfxVolume']))
                        before = previous_sound.get(('samples', sound_id))
                        samples = sound['samples']
                        candidates = [sample for index, sample in enumerate(samples) if before is not None
                                      and sample != before[index]]
                        if len(samples) == 1:
                            candidates = samples
                        if len(candidates) == 1:
                            sample = candidates[0]
                            request.update(sample=integer(sample['index']), gain=f32(sample['gain']), pitch=f32(sample['pitch']), looping=boolean(sample['loop']))
                        else:
                            unavailable.append(dict(field='native selected sample index', gp=row['gp'], sound=sound_id,
                                reason='Existing wrapper snapshots do not uniquely identify this repeated sample draw.'))
                    for field, expected in [('gain', f32(min(case['sfxVolume'], 1))), ('pitch', f32(1)), ('looping', False)]:
                        if field in request:
                            check(side + ' collection sample ' + field, request[field], expected)
                    requests.append(request)
        audio = row['audio']
        if isinstance(audio, dict):
            slots = audio['slots']
            for sid in (8, 252):
                sound = next((s for s in slots if s['id'] == sid), None) if isinstance(slots, list) else slots.get(str(sid))
                if isinstance(sound, dict):
                    previous_sound[sid] = sound['lastTick']
                    previous_sound[('samples', sid)] = sound['samples']
        if previous is not None and row['guard'] and not changed:
            for body in row['bodies']:
                source_group = next((index for index, source in initial.items()
                                     if source['key'] == body['key'] and body['key'] not in retired_sources), None)
                if source_group is None:
                    continue
                old = next((p for p in previous['bodies'] if p['key'] == body['key']), None)
                if old is None:
                    continue
                distance_before = math.dist(previous['position'], old['position'])
                distance_after = math.dist(row['position'], body['position'])
                velocity_started = not any(old['velocity']) and any(body['velocity'])
                #These are end-of-presentation positions, not an invented
                #callback pre-position. A mass split can already separate to
                #the radius. The small bound covers f32 positions projected
                #through the declared f64 logical coordinate store (<1024
                #world units here); it never applies to velocity equality.
                adjacent_after_start = velocity_started and distance_after <= 10 + body['radius'] + 1 / 1024
                if (old['entity'] == 4 and old['ad0'] == old['acc'] == 0
                        and (distance_before < 10 + old['radius'] or adjacent_after_start)):
                    contacts.append(dict(gp=row['gp'], source=body['key'], sourceGroup=source_group,
                        pickupVelocity=body['velocity'], playerVelocity=row['velocity'],
                        playerLogicalVelocity=row['logicalVelocity'], main=row['main']['animation'],
                        velocityStarted=bool(velocity_started), distanceBefore=distance_before, distanceAfter=distance_after,
                        guard=row['guard'], controls=row['controls'], phase='Full' if row['counter'] % 2 else 'Fast'))
        if (side == 'browser' and row['state']['publicActions']['pocketTriggered']
                and all(slot['id'] == 0 for slot in row['held'])):
            empty_edges += 1
            check('browser empty pocket has preceding observation', previous is not None)
            if previous is not None:
                prior_c70 = integer(previous['state']['attackRuntime']['aimRotationC70'])
                current_c70 = integer(row['state']['attackRuntime']['aimRotationC70'])
                prior_move = previous['items']['moveSpeedInputs']; current_move = row['items']['moveSpeedInputs']
                fields = {'sourceD28', 'sourceD44', 'source13bc', 'source13c0'}
                assert isinstance(prior_move, dict) and isinstance(current_move, dict)
                assert set(prior_move) == set(current_move) == fields
                for value in list(prior_move.values()) + list(current_move.values()):
                    integer(value)
                check('browser empty pocket preserves C70', current_c70, prior_c70)
                check('browser empty pocket preserves Move inputs', current_move, prior_move)
        previous, previous_slot = row, slot
    check(side + ' playing observations exist', len(seen_frames) > 0)
    check(side + ' declared initial bodies observed', len(initial), len(groups))
    check(side + ' all declared bodies reach Idle', all(source['idle'] for source in initial.values()))
    check(side + ' Full observations', full >= contract['minimumFullObservations'])
    check(side + ' Fast observations', fast >= contract['minimumFastObservations'])
    if kind == 'collection':
        check(side + ' declared held history', [entry['slot'] for entry in histories], contract['expectedSlot0History'])
        check(side + ' capacity spills', len(drops) >= contract['minimumObservedCapacityEjections'])
        check(side + ' actual nonoverride collection sound requests', len(requests), len(histories) - sound_overrides)
        if 'pocketRefusedContact' in case:
            unavailable.append(dict(field=side + ' same-v1 AD0 refresh coverage',
                reason='Exact retained regression tape never reached this contact; old strict failure is preserved. Actual transitions, when observed, are still checked.'))
        else:
            check(side + ' observed drop AD0 refresh', refreshes >= contract['minimumObservedDropAD0Refresh'])
        if side == 'browser':
            check('browser known empty pocket edge observed', empty_edges > 0)
    else:
        check(side + ' released Init has no collection', histories, [])
    return dict(initial=initial, history=histories, drops=drops, requests=requests,
                refusedContactObservations=contacts, dropAD0Refreshes=refreshes, emptyPocketEdges=empty_edges,
                soundOverridesUnavailable=sound_overrides, Full=full, Fast=fast, lastGameplayPresentation=last_gp)


native_case_path = args.original.parent / 'case.json'
native_case = json.loads(native_case_path.read_text())
same_declaration = native_case == case
check('independent Native declaration equals public case', native_case, case)
inputs = [args.case, args.original, args.browser, native_case_path]
for directory, names in [(args.original.parent, ('native-progress.jsonl.gz', 'native-progress.jsonl.zst')),
                         (args.browser.parent, (('actual-states.jsonl.gz',) if args.wasm_node else ('browser-progress.jsonl.gz',)))]:
    inputs.extend(directory / name for name in names if (directory / name).is_file())
before = {str(path): digest(path) for path in inputs}
native = require('actual Native field ownership', lambda: observe('native', native_rows()))
browser = require('actual browser field ownership', lambda: observe('browser', browser_rows()))
if native is not None and browser is not None:
    for index in range(len(contract['groups'])):
        if index not in native['initial'] or index not in browser['initial']:
            continue
        left, right = native['initial'][index], browser['initial'][index]
        for field in ('variant', 'subtype', 'seed', 'sequence', 'radius', 'mass'):
            check('paired source body ' + field, right[field], left[field])
    check('paired held history', [r['slot'] for r in browser['history']], [r['slot'] for r in native['history']])
    if kind == 'collection':
        for field in ('animation', 'animationFrame', 'animationPlaying'):
            check('paired collection main pose ' + field, [r['main'][field] for r in browser['history']],
                  [r['main'][field] for r in native['history']])
        copied_owned = all(isinstance(row['copied'], dict) for side in (native, browser) for row in side['history'])
        check('paired collection copied pose owners', copied_owned)
        if copied_owned:
            for field in ('animation', 'animationFrame', 'animationPlaying', 'overlay', 'overlayFrame'):
                check('paired collection copied pose ' + field, [r['copied'][field] for r in browser['history']],
                      [r['copied'][field] for r in native['history']])
        for field in ('variant', 'subtype', 'ad0', 'velocity'):
            check('paired spill ' + field, [r[field] for r in browser['drops']], [r[field] for r in native['drops']])
    if kind == 'collection' and native['soundOverridesUnavailable'] == 0:
        for field in ('id', 'volume', 'delay'):
            check('paired collection request ' + field, [r[field] for r in browser['requests']], [r[field] for r in native['requests']])
        for index, (left, right) in enumerate(zip(native['requests'], browser['requests'])):
            for field in ('sample', 'gain', 'pitch', 'looping'):
                if field in left and field in right:
                    check('paired collection sample ' + field, right[field], left[field])
    native_impulses = [row for row in native['refusedContactObservations'] if any(row['pickupVelocity'])]
    browser_impulses = [row for row in browser['refusedContactObservations'] if any(row['pickupVelocity'])]
    if native_impulses:
        check('paired guarded refusal has actual pickup physical response', bool(browser_impulses), True)
    elif kind == 'collection':
        unavailable.append(dict(field='guarded refusal mass response', reason='No qualifying contact observed in this declared tape; exact historical same-v1 regression remains separately retained.'))
    if 'pocketRefusedContact' in case:
        targets = [index for index, group in enumerate(contract['groups'])
                   if group['entries'][0]['variant'] == 70 and group['entries'][0]['subtype'] == 1]
        assert len(targets) == 1, 'The regression declares one initial Pill1 source group'
        selected = {}
        for side, impulses in [('native', native_impulses), ('browser', browser_impulses)]:
            matching = [row for row in impulses if row['sourceGroup'] == targets[0] and row['velocityStarted']]
            check(side + ' mandatory initial Pill guarded response', bool(matching), True)
            if matching:
                selected[side] = matching[0]
                check(side + ' refused-contact UseItem pose', matching[0]['main'], 'UseItem')
                check(side + ' refused-contact controls', matching[0]['controls'], True)
        if len(selected) == 2:
            left, right = selected['native'], selected['browser']
            check('paired first guarded Pill response public presentation', right['gp'], left['gp'])
            check('paired first guarded Pill response phase', right['phase'], left['phase'])
            check('paired first guarded Pill pickup velocity', right['pickupVelocity'], left['pickupVelocity'])
            #The runtime projects the f32 ContactBody into f64 Player storage:
            #f64::from(velocity)*0.65. Compare that declared coordinate mapping,
            #never a prior Own value or an inverted/rounded synthetic world.
            expected = [word * .65 for word in left['playerVelocity']]
            check('paired first guarded Pill Player velocity projection', right['playerLogicalVelocity'], expected)
    diagnostics.append(dict(field='collection presentation and raw position', native=native['history'], browser=browser['history'],
        scope='Actual same-input witnesses retained. Collection main/copied pose and selected guarded response velocity are assertions; raw positions and remaining clocks stay diagnostic while source position integration is unresolved.'))
after = {str(path): digest(path) for path in inputs}
check('capture files before equal end', after, before)
unavailable.extend([dict(field=field, reason='Outside these observed public-input owners; not counted as equality.')
    for field in contract.get('unknown', [])
    if not (field == 'OriginalGameSameInputCounterpart' and same_declaration
            and native is not None and browser is not None)])
unavailable.extend([dict(field='backend played audio and PCM', reason='Request/slot/sample settings do not prove playback; declared mute is recorded separately.'),
    dict(field='private callback invocation and pool call ledger', reason='Observers read actual fields only; no callback inferred from outcomes.')])
result = dict(schema=1, status='fail' if failures else 'pass', case=case['name'], kind=kind,
    checks=sum(counts.values()), failures=sum(failures.values()), checksPerField=dict(counts), failedPerField=dict(failures),
    examples=examples, native=native, browser=browser, diagnostics=diagnostics, unavailable=unavailable,
    audioPreference=dict(sfxVolume=case['sfxVolume'], muted=case['sfxVolume'] == 0, playbackQualified=False),
    inputsBefore=before, inputsAfter=after,
    scope='Fresh declared Original/browser functional comparison. Actual missing required fields fail; unsupported evidence is explicitly unavailable. No native output enters browser inputs.')
if args.wasm_node:
    result['captureBackend'] = 'wasm-node'
    result['unavailable'].append(dict(field='DOM, renderer, PNG and audio backend', reason='Explicit public-WASM ABI backend; these browser observations were not made.'))
    result['scope'] = 'Independent Original versus actual qualified public-WASM ABI functional fields. All selected laws are unchanged; DOM/renderer/backend remain unavailable. No Native output enters WASM inputs.'
raw = (json.dumps(result, indent=2, allow_nan=False) + '\n').encode()
args.output.write_bytes(gzip.compress(raw, mtime=0) if args.output.suffix == '.gz' else raw)
print(json.dumps(dict(status=result['status'], checks=result['checks'], failures=result['failures'], output=str(args.output))))
raise SystemExit(bool(failures))
