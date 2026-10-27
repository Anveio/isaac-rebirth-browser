"""Fresh Original/browser active-item laws from each game's actual live records.

The capture readback owns transport, complete journals and identical declared
inputs. This comparer pairs both games' gameplay presentations of the start
room and compares the held active, charge, temporary records, health, Player
body, collision classes, the main sprite and the collectible pedestals.
Unavailable evidence never counts as equality.
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
parser.add_argument('--capture-readback', type=Path, required=True)
args = parser.parse_args()
case = json.loads(args.case.read_text())
contract = case['activeItems']
assert contract['schema'] == 'source-public-active-items-v1'
counts, failures = collections.Counter(), collections.Counter()
examples = []
#Clips the port names; ordinary walk/head clips are outside this reader.
CLIPS = {'UseItem', 'Pickup', 'Jump', 'Hit', 'Sad', 'Happy'}


def check(field, actual, expected=True, gp=None):
    counts[field] += 1
    if type(actual) is not type(expected) or actual != expected:
        failures[field] += 1
        if len(examples) < 120:
            examples.append(dict(field=field, gameplayPresentation=gp, actual=actual, expected=expected))


def integer(value):
    assert type(value) is int, 'Actual integer owner required'
    return value


def f32(value):
    assert type(value) in (int, float) and math.isfinite(value), 'Actual finite numeric owner required'
    return struct.unpack('<f', struct.pack('<f', value))[0]


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
    spec = importlib.util.spec_from_file_location('active_native_evidence',
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
            pedestals = sorted(integer(actor['subtype']) for actor in world['entities']
                               if actor['type'] == 5 and actor['variant'] == 100 and not actor['removed'])
            yield integer(context['presentedFrame']) - gameplay_start, dict(
                activeId=integer(player['activeId']), activeCharge=integer(player['activeCharge']),
                temporary=[[integer(row['itemId']), integer(row['count']), integer(row['cooldown'])]
                           for row in player['temporaryCollectibles']],
                health=[integer(value) for value in player['health']],
                position=vector(physical['position']), velocity=vector(physical['velocity']),
                entityCollisionOff=integer(physical['entityCollisionClass']) == 0,
                eternal=integer(physical['eternalHearts']),
                pockets=[[integer(slot['id']), bool(integer(slot['typeTagRaw'])) if slot['id'] else None]
                         for slot in player['heldPocketRecords']],
                clip=player['mainSprite']['animation'], frame=f32(player['mainSprite']['animationFrame']),
                controls=player['controlsEnabled'] is True, pedestals=pedestals)
    assert (records, raw_bytes, raw_hash.hexdigest()) == (metadata['records'], metadata['rawBytes'], metadata['rawSha256'])
    assert gameplay_start is not None


def browser_rows():
    path = args.browser.parent / 'browser-progress.jsonl.gz'
    metadata = json.loads(path.with_name(path.name + '.meta.json').read_text())
    assert metadata['status'] == 'closed' and digest(path) == metadata['archiveSha256']
    raw_hash, raw_bytes, records, first_room = hashlib.sha256(), 0, 0, None
    with gzip.open(path, 'rb') as stream:
        for raw in stream:
            raw_hash.update(raw); raw_bytes += len(raw); records += 1
            entry = json.loads(raw)
            if entry['kind'] != 'state':
                continue
            value = entry['value']; state = value['state']
            if state['screen'] != 'playing':
                continue
            room = integer(state['currentRoom'])
            if first_room is None:
                first_room = room
            if room != first_room:
                continue
            owner = state['playerItems']; player = state['player']
            clip, frame = state['playerAnimation']['current'], state['playerAnimation']['frame']
            if clip is None and owner['pickupAnimationGuard'] is True:
                #The collection Pickup clip has its own legacy owner.
                clip, frame = 'Pickup', owner['pickupAnimationFrame']
            pedestals = sorted(integer(actor['subtype']) for actor in state['room']['pickups']
                               if actor['variant'] == 100 and not actor['removed'])
            yield integer(value['gameplayPresentation']), dict(
                activeId=integer(owner['activeId']), activeCharge=integer(owner['activeCharge']),
                temporary=[[integer(row['itemId']), integer(row['count']), integer(row['cooldown'])]
                           for row in owner['temporaryCollectibles']],
                health=[integer(player['maxHp']), integer(player['hp']), integer(player['soulHp']),
                        integer(player['blackHeartMask'])],
                position=[f32((player['x'] - 32) / .65), f32((player['y'] + 46) / .65)],
                velocity=[f32(player['vx'] / .65), f32(player['vy'] / .65)],
                entityCollisionOff=player.get('entityCollisionOff', False) is True,
                eternal=integer(player['eternalHearts']),
                pockets=[[integer(slot['id']), (slot['card'] is True) if slot['id'] else None]
                         for slot in owner['heldPockets']['slots']],
                clip=clip, frame=frame,
                controls=player['controlsEnabled'] is True, pedestals=pedestals)
    assert (records, raw_bytes, raw_hash.hexdigest()) == (metadata['records'], metadata['rawBytes'], metadata['rawSha256'])


readback = json.loads(args.capture_readback.read_text())
check('actual complete capture readback passes', readback['status'], 'pass')
native_case_path = args.original.parent / 'case.json'
check('independent Native declaration equals public case', json.loads(native_case_path.read_text()), case)
inputs = [args.case, args.original, args.browser, args.capture_readback, native_case_path]
before = {str(path): digest(path) for path in inputs}
native, own = {}, {}
for name, reader, rows in (('Native', native_rows, native), ('Own', browser_rows, own)):
    try:
        for gp, row in reader():
            rows.setdefault(gp, row)  # The first record of each presentation.
    except (KeyError, TypeError, ValueError, AssertionError, IndexError, OSError) as error:
        check(f'actual {name} journal readback', repr(error), 'complete typed journal')
paired = sorted(set(native) & set(own))
total = sum(action['presentations'] for action in case['actions'])
check('paired presentations cover the declared tape', bool(paired) and paired[-1] >= total - 4)
uses, held, landed = collections.Counter(), set(), 0
for gp in paired:
    a, b = own[gp], native[gp]
    for field in ('activeId', 'activeCharge', 'temporary', 'health', 'position', 'velocity',
                  'entityCollisionOff', 'controls', 'pedestals', 'eternal', 'pockets'):
        check(f'Player:{field}', a[field], b[field], gp)
    if b['clip'] in CLIPS or a['clip'] is not None:
        check('main sprite clip', a['clip'], b['clip'], gp)
        check('main sprite frame', f32(a['frame']) if a['frame'] is not None else None, b['frame'], gp)
    held.add(b['activeId'])
    for row in b['temporary']:
        uses[row[0]] += 1
    landed += b['entityCollisionOff']
for item in contract['uses']:
    check(f'actual Native active {item} held', item in held)
    check(f'actual Native active {item} use record', uses[item] > 0)
if 282 in contract['uses']:
    check('actual Native How to Jump airborne presentations', landed > 0)
after = {str(path): digest(path) for path in inputs}
check('all input archives before equal end', after, before)
result = dict(schema=1, status='fail' if failures else 'pass', case=case['name'],
    checks=sum(counts.values()), failures=sum(failures.values()),
    nativePresentations=len(native), ownPresentations=len(own), pairedPresentations=len(paired),
    checksPerField=dict(counts), failedPerField=dict(failures), examples=examples,
    unavailable=contract['unknown'], inputsBefore=before, inputsAfter=after,
    scope='Per-presentation start-room active slot, charge, temporary records, health, Player body, '
          'collision classes, named main-sprite clips and pedestals. No private order/render/backend claim.')
raw = (json.dumps(result, indent=2, allow_nan=False) + '\n').encode()
args.output.write_bytes(gzip.compress(raw, mtime=0) if args.output.suffix == '.gz' else raw)
print(json.dumps(dict(status=result['status'], checks=result['checks'], failures=result['failures'])))
raise SystemExit(bool(failures))
