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
freeze_contract = contract.get('npcFreeze')
if freeze_contract is not None:
    assert freeze_contract['schema'] == 'source-public-npc-freeze-v1'


def artifact_value(value):
    if isinstance(value, dict):
        return {str(key): artifact_value(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [artifact_value(item) for item in value]
    if isinstance(value, set):
        return [artifact_value(item) for item in sorted(value)]
    return value


def check(field, actual, expected=True, gp=None):
    counts[field] += 1
    if type(actual) is not type(expected) or actual != expected:
        failures[field] += 1
        if len(examples) < 120:
            examples.append(dict(field=field, gameplayPresentation=gp, actual=artifact_value(actual), expected=artifact_value(expected)))


def integer(value):
    assert type(value) is int, 'Actual integer owner required'
    return value


def boolean(value):
    assert type(value) is bool, 'Actual boolean owner required'
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
            row = dict(
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
            if freeze_contract is not None:
                row['npcFreeze'] = native_freeze(observation)
            yield integer(context['presentedFrame']) - gameplay_start, row
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
            row = dict(
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
            if freeze_contract is not None:
                row['npcFreeze'] = browser_freeze(state, integer(value['gameplayPresentation']))
            yield integer(value['gameplayPresentation']), row
    assert (records, raw_bytes, raw_hash.hexdigest()) == (metadata['records'], metadata['rawBytes'], metadata['rawSha256'])



def color(value):
    assert type(value) is dict, 'Actual SpriteColor owner required'
    result = [f32(word) for field in ('tint', 'colorize', 'offset') for word in value[field]]
    assert len(result) == 11
    return result


def native_freeze(observation):
    extra, combat = observation['npcFreeze'], observation['combat']
    assert extra['status'] == combat['status'] == 'pass', 'Complete independent Native freeze owners required'
    measured = {(integer(row['sequenceId']), integer(row['spawnSeed'])): row for row in extra['actors']}
    result = dict(fullUpdates=integer(combat['nativePhase']['fullGameUpdates']),
                  systemCounter=integer(combat['nativePhase']['systemIterations']), actors=[])
    for actor in combat['entities']:
        if not 10 <= actor['type'] <= 999:
            continue
        actual = measured.pop((integer(actor['sequenceId']), integer(actor['spawnSeed'])))
        flags = integer(actor['entityFlags'])
        result['actors'].append(dict(
            key=[integer(actor[key]) for key in ('type', 'variant', 'subtype', 'spawnSeed')],
            lifetimeId=integer(actor['sequenceId']), position=vector(actual['physicalPosition']),
            presentedPosition=vector(actor['position']), velocity=vector(actor['velocity']),
            hp=f32(actor['hp']), maxHp=f32(actor['maxHp']), baseFriction=f32(actual['baseFriction']),
            friction=f32(actor['friction']), currentFriction=f32(actor['friction']),
            freezeTimer=integer(actor['statusCountdown']),
            immune=bool(flags & 1), frozen=bool(flags & 0x20), applicationLockout=integer(actual['applicationLockout']),
            state=integer(actor['npcState']), frameAge=integer(actor['npcFrameAge']),
            animation=actor['animation'], animationClock=f32(actor['animationFrame']),
            animationPlaying=actor['animationPlaying'], visible=actor['visible'], dead=actor['removed'],
            raw494=actual['raw494'], boss=actor['boss'], champion=actor['npcChampion'],
            parentNull=actual['parentNull'], childNull=actual['childNull'],
            entityCollisionClass=integer(actor['entityCollisionClass']), eligibleFlag=not bool(flags & 0x10),
            spriteColor=[f32(word) for word in actor['spriteColor']],
            colorStack=[dict(priority=integer(row['priority']), color=[f32(word) for word in row['color']],
                duration=integer(row['duration']), remaining=integer(row['remaining'])) for row in actual['timedColors']],
            ai=[integer(actor['npcStateFrame']), integer(actor['npcFrameAge'])],
            shooter=[integer(actor['npcProjectileCooldown']), integer(actor['npcProjectileDelay'])],
            preview=actual['preview'], statusAvailable=True))
    assert not measured, 'Every independent Native freeze actor must join the actual room list'
    return result


def browser_freeze(state, gp):
    result = dict(fullUpdates=integer(state['gameUpdateCount']),
                  systemCounter=integer(state['systemClock']['systemCounter']), actors=[])
    for actor in state['room']['npcs']:
        status = actor.get('status')
        # Older artifacts have no status owner. Preserve their actual body
        # and base fields; missing status words must fail as unavailable,
        # never become zero/false or stop the entire actual journal read.
        status_values = {}
        for field, decode in (
                ('currentFriction', lambda value: None if value is None else f32(value)),
                ('freezeTimer', integer), ('baseUpdateAdmitted', boolean),
                ('frozen', boolean), ('applicationLockout', integer)):
            try:
                assert type(status) is dict and field in status, 'Missing status owner'
                status_values[field] = decode(status[field])
            except (KeyError, TypeError, ValueError, AssertionError) as error:
                status_values[field] = dict(unavailable=f'Npc.status.{field}', reason=str(error))
        frontier = (status.get('frontier', dict(unavailable='Npc.status.frontier'))
                    if type(status) is dict else dict(unavailable='Npc.status.frontier'))
        status_available = frontier is None and not any(type(value) is dict for value in status_values.values())
        shooter = actor.get('shooter')
        result['actors'].append(dict(
            key=[integer(actor[key]) for key in ('kind', 'variant', 'subtype', 'spawnSeed')],
            lifetimeId=integer(actor['id']), position=vector(actor['position']),
            presentedPosition=vector(actor['presentedPosition']), velocity=vector(actor['velocity']),
            hp=f32(actor['hp']), maxHp=f32(actor['maxHp']), baseFriction=f32(actor['baseFriction']),
            friction=f32(actor['friction']),
            **status_values, statusAvailable=status_available,
            state=integer(actor['state']), frameAge=integer(actor['frameAge']),
            animation=actor['animation'], animationClock=f32(actor['animationClock']),
            animationPlaying=actor['animationPlaying'], visible=actor['visible'], dead=actor['dead'],
            raw494=actor['raw494'], boss=actor['boss'], champion=actor.get('champion') is not None,
            parentNull=actor.get('parent') is None, childNull=actor.get('child') is None,
            entityCollisionClass=integer(actor['entityCollisionClass']),
            spriteColor=color(actor['spriteColor']['current']),
            colorStack=[dict(priority=integer(row['priority']), color=color(row['color']),
                duration=integer(row['duration']), remaining=integer(row['remaining']))
                for row in actor['spriteColor']['stack']],
            ai=[integer(actor['frameAge'])],
            shooter=[integer(shooter['cooldown']), integer(shooter['delay'])] if shooter is not None else None))
        check('Own:freeze actual status owner available', status_available, gp=gp)
        check('Own:freeze status frontier closed', frontier, None, gp)
    return result


def target_map(value):
    actors = {tuple(actor['key'][:3]): actor for actor in value['actors']}
    assert len(actors) == len(value['actors']), 'Actual unique target family identities required'
    return actors


def freeze_witnesses(name, rows):
    """Independent source-law and anti-vacuity checks in each actual game."""
    expected = {(row['type'], row['variant'], row['subtype']) for row in freeze_contract['targets']}
    roster, identities, health, first, last_frozen = None, None, None, None, None
    moved, held, used = set(), False, False
    full_steps, fast_steps, boundary, resumed, post_fast = set(), set(), set(), set(), set()
    release_full_seen = set()
    application_phase, first_full_gp, first_fast_gp = None, None, None
    unavailable_status_presentations = 0
    control = freeze_contract['control']
    previous = None
    prior_gp = None
    preuse_shooter = set()
    space_windows, cursor = [], 0
    for action in case['actions']:
        if 'space' in action['keys']:
            space_windows.append((cursor, cursor + action['presentations'] + 1))
        cursor += action['presentations']
    freeze_color = [f32(.22)] * 3 + [f32(1)] + [f32(0)] * 4 + [f32(f32(40) * f32(1 / 255))] * 3
    for gp in sorted(rows):
        player = rows[gp]
        value = player['npcFreeze']
        actors = target_map(value)
        #The initial room admission may precede its actual spawned actors.
        if set(actors) != expected and roster is None:
            previous, prior_gp = None, gp
            continue
        check(f'{name}:freeze exact target roster', set(actors), expected, gp)
        if set(actors) != expected:
            continue
        now_ids = {key: actor['lifetimeId'] for key, actor in actors.items()}
        now_hp = {key: actor['hp'] for key, actor in actors.items()}
        if roster is None:
            roster, identities, health = set(actors), now_ids, now_hp
        check(f'{name}:freeze stable actor lifetimes', now_ids, identities, gp)
        check(f'{name}:freeze unchanged target HP', now_hp, health, gp)
        held |= player['activeId'] == 39 and player['activeCharge'] == 3
        used |= any(row[0] == 39 for row in player['temporary']) and player['activeCharge'] == 0
        if not all(actor['statusAvailable'] for actor in actors.values()):
            unavailable_status_presentations += 1
            # Movement/shooter observations before the declared first Space
            # remain usable without asserting any missing status value.
            if not space_windows or gp < space_windows[0][0]:
                moved.update(key for key, actor in actors.items() if key[0] == 18 and any(actor['velocity']))
                preuse_shooter.add(tuple(actors[(12, 0, 0)]['shooter']))
            if control:
                check(f'{name}:freeze control no TEMP39 or discharge', not any(row[0] == 39 for row in player['temporary']) and (player['activeId'] != 39 or player['activeCharge'] == 3), gp=gp)
            previous, prior_gp = None, gp
            continue
        active = any(actor['frozen'] for actor in actors.values())
        if control:
            check(f'{name}:freeze control has no positive timer', [actor['freezeTimer'] for actor in actors.values()], [0] * len(actors), gp)
            check(f'{name}:freeze control has no frozen flag', active, False, gp)
            check(f'{name}:freeze control no TEMP39 or discharge', not any(row[0] == 39 for row in player['temporary']) and (player['activeId'] != 39 or player['activeCharge'] == 3), gp=gp)
            if name == 'Own':
                check('Own:freeze control retains ordinary Base owner', [actor['baseUpdateAdmitted'] for actor in actors.values()], [False] * len(actors), gp)
        if first is None and not active and (not space_windows or gp < space_windows[0][0]):
            moved.update(key for key, actor in actors.items() if key[0] == 18 and any(actor['velocity']))
            preuse_shooter.add(tuple(actors[(12, 0, 0)]['shooter']))
        if active and first is None:
            first = gp
            check(f'{name}:freeze application has prior actual targets', previous is not None, gp=gp)
            check(f'{name}:freeze actual application joins physical Space tape', any(start <= gp <= end for start, end in space_windows), gp=gp)
            if previous is not None:
                check(f'{name}:freeze held39 charge3 immediately before application', rows[prior_gp]['activeId'] == 39 and rows[prior_gp]['activeCharge'] == 3, gp=gp)
                for key, actor in previous.items():
                    if name == 'Native':
                        check('Native:freeze actual target passes flags immunity eligibility', actor['eligibleFlag'] and not actor['immune'] and actor['entityCollisionClass'] not in (0, 1), gp=gp)
                    check(f'{name}:freeze target past Appear before use', actor['state'] != 1 and actor['visible'] and not actor['dead'], gp=gp)
                    check(f'{name}:freeze ordinary unlinked living target', not actor['boss'] and not actor['champion'] and actor['parentNull'] and actor['childNull'] and not actor['raw494'] and actor['hp'] > 0, gp=gp)
                full = value['fullUpdates'] != rows[prior_gp]['npcFreeze']['fullUpdates']
                application_phase = 'Full' if full else 'Fast'
                if full:
                    first_full_gp = gp
                else:
                    first_fast_gp = gp
                for key, actor in actors.items():
                    before = previous[key]
                    source_friction = f32(before['currentFriction'])
                    check(f'{name}:freeze application source current470 owner', source_friction, before['baseFriction'], gp)
                    check(f'{name}:freeze application current470 equals base46c', actor['currentFriction'], actor['baseFriction'], gp)
                    check(f'{name}:freeze application transfers current470', actor['friction'], actor['currentFriction'], gp)
                    check(f'{name}:freeze application suppresses AI', [actor['state'], actor['ai'], actor['shooter']], [before['state'], before['ai'], before['shooter']], gp)
                    check(f'{name}:freeze application suppresses sprite', [actor['animation'], actor['animationClock'], actor['animationPlaying']], [before['animation'], before['animationClock'], before['animationPlaying']], gp)
                    if full:
                        reduced = f32(source_friction * f32(.01))
                        velocity = [f32(word * reduced) for word in before['velocity']]
                        check(f'{name}:freeze first Full application velocity', actor['velocity'], velocity, gp)
                        check(f'{name}:freeze first Full application integrates position', actor['position'], [f32(p + v) for p, v in zip(before['position'], velocity)], gp)
                        check(f'{name}:freeze first Full application presents physical position', actor['presentedPosition'], actor['position'], gp)
                    else:
                        check(f'{name}:freeze immediate Fast application holds physical position', actor['position'], before['position'], gp)
                        check(f'{name}:freeze immediate Fast application holds velocity', actor['velocity'], before['velocity'], gp)
                        preview = [f32(p + f32(f32(v * source_friction) / f32(2))) for p, v in zip(actor['position'], actor['velocity'])]
                        check(f'{name}:freeze immediate Fast application uses current470', actor['presentedPosition'], preview, gp)
                check(f'{name}:freeze first timer120/119 follows actual phase',
                      [actor['freezeTimer'] for actor in actors.values()], [119 if full else 120] * len(actors), gp)
        if active:
            last_frozen = gp
            for actor in actors.values():
                check(f'{name}:freeze live current470 remains base46c', actor['currentFriction'], actor['baseFriction'], gp)
                check(f'{name}:freeze live friction follows source current470', actor['friction'], actor['currentFriction'], gp)
                check(f'{name}:freeze actual timed colour entry', any(row['priority'] == 255 and row['color'] == freeze_color and row['remaining'] == actor['freezeTimer'] for row in actor['colorStack']), gp=gp)
        if previous is not None and prior_gp is not None and gp == prior_gp + 1:
            full = value['fullUpdates'] != rows[prior_gp]['npcFreeze']['fullUpdates']
            if first is not None:
                if full and first_full_gp is None:
                    first_full_gp = gp
                elif not full and first_fast_gp is None:
                    first_fast_gp = gp
            check(f'{name}:freeze consecutive system phase clock', value['systemCounter'] - rows[prior_gp]['npcFreeze']['systemCounter'], 1, gp)
            check(f'{name}:freeze full phase clock bound', value['fullUpdates'] - rows[prior_gp]['npcFreeze']['fullUpdates'] in (0, 1), gp=gp)
            for key, actor in actors.items():
                before = previous[key]
                if before['freezeTimer'] > 0:
                    check(f'{name}:freeze countdown follows full callbacks', actor['freezeTimer'], max(0, before['freezeTimer'] - int(full)), gp)
                    check(f'{name}:freeze flag follows timer', actor['frozen'], actor['freezeTimer'] > 0, gp)
                    check(f'{name}:freeze AI held', [actor['state'], actor['ai'], actor['shooter']], [before['state'], before['ai'], before['shooter']], gp)
                    check(f'{name}:freeze sprite held', [actor['animation'], actor['animationClock'], actor['animationPlaying']], [before['animation'], before['animationClock'], before['animationPlaying']], gp)
                    if full:
                        full_steps.add(key)
                        reduced = f32(f32(before['currentFriction']) * f32(.01))
                        velocity = [f32(word * reduced) for word in before['velocity']]
                        check(f'{name}:freeze reduced Full velocity', actor['velocity'], velocity, gp)
                        check(f'{name}:freeze integrated Full position', actor['position'], [f32(p + v) for p, v in zip(before['position'], velocity)], gp)
                        check(f'{name}:freeze current friction reset', actor['friction'], actor['baseFriction'], gp)
                    else:
                        fast_steps.add(key)
                        preview = [f32(p + f32(f32(v * actor['friction']) / f32(2))) for p, v in zip(actor['position'], actor['velocity'])]
                        check(f'{name}:freeze Fast prediction', actor['presentedPosition'], preview, gp)
                        check(f'{name}:freeze Fast physical hold', actor['position'], before['position'], gp)
                    if before['freezeTimer'] == 1 and actor['freezeTimer'] == 0:
                        boundary.add(key)
                        check(f'{name}:freeze expiry removes timed blue entry', not any(row['priority'] == 255 for row in actor['colorStack']), gp=gp)
                elif first is not None and actor['freezeTimer'] == 0:
                    if full:
                        progressed = actor['animationClock'] != before['animationClock'] or actor['shooter'] != before['shooter'] or actor['velocity'] != before['velocity'] or actor['ai'] != before['ai']
                        if key not in release_full_seen:
                            check(f'{name}:freeze first eligible Full resumes AI or sprite', progressed, gp=gp)
                            release_full_seen.add(key)
                            if progressed:
                                resumed.add(key)
                        check(f'{name}:freeze postexpiry Full friction reset', actor['friction'], actor['baseFriction'], gp)
                    else:
                        post_fast.add(key)
                        preview = [f32(p + f32(f32(v * actor['friction']) / f32(2))) for p, v in zip(actor['position'], actor['velocity'])]
                        check(f'{name}:freeze postexpiry Fast prediction', actor['presentedPosition'], preview, gp)
                        check(f'{name}:freeze postexpiry Fast physical hold', actor['position'], before['position'], gp)
        previous, prior_gp = actors, gp
    check(f'{name}:freeze actual held39 full charge3', held)
    if control:
        check(f'{name}:freeze control observed complete target tape', roster is not None and max(rows, default=0) > 500)
        check(f'{name}:freeze control observed moving actor', (18, 0, 0) in moved)
        check(f'{name}:freeze control observed Horf shooter progression', len(preuse_shooter) > 1)
        return dict(control=True, applicationPhase=None, applicationPresentation=None,
                    unavailableStatusPresentations=unavailable_status_presentations)
    check(f'{name}:freeze actual use and discharge', used)
    check(f'{name}:freeze moving target before use', (18, 0, 0) in moved)
    check(f'{name}:freeze Horf shooter progresses before use', len(preuse_shooter) > 1)
    check(f'{name}:freeze actual positive application', first is not None)
    check(f'{name}:freeze actual first application phase retained', application_phase in ('Full', 'Fast'))
    check(f'{name}:freeze first Full boundary immediately witnessed', first is not None and first_full_gp in (first, first + 1))
    check(f'{name}:freeze first Fast boundary immediately witnessed', first is not None and first_fast_gp in (first, first + 1))
    check(f'{name}:freeze Full witnessed for each target', full_steps, expected)
    check(f'{name}:freeze Fast witnessed for each target', fast_steps, expected)
    check(f'{name}:freeze timer1-to0 witnessed for each target', boundary, expected)
    check(f'{name}:freeze next-full resume witnessed for each target', resumed, expected)
    check(f'{name}:freeze postexpiry Fast witnessed for each target', post_fast, expected)
    check(f'{name}:freeze complete expiry margin', first is not None and max(rows, default=0) > first + 260 and last_frozen is not None and max(rows) > last_frozen + 20)
    return dict(control=False, applicationPhase=application_phase, applicationPresentation=first,
                firstFullPresentation=first_full_gp, firstFastPresentation=first_fast_gp,
                unavailableStatusPresentations=unavailable_status_presentations)


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
    if freeze_contract is not None:
        try:
            native_targets, own_targets = target_map(b['npcFreeze']), target_map(a['npcFreeze'])
            check('NPC freeze paired target families', set(own_targets), set(native_targets), gp)
            for key in sorted(set(native_targets) & set(own_targets)):
                for field in ('key', 'position', 'presentedPosition', 'velocity', 'hp', 'maxHp',
                    'baseFriction', 'friction', 'currentFriction', 'freezeTimer', 'frozen', 'applicationLockout', 'state',
                    'frameAge', 'animation', 'animationClock', 'animationPlaying', 'visible', 'dead',
                    'raw494', 'boss', 'champion', 'parentNull', 'childNull', 'entityCollisionClass',
                    'spriteColor', 'colorStack'):
                    check(f'NPC{key}:freeze {field}', own_targets[key][field], native_targets[key][field], gp)
                if key[0] == 12:
                    check('Horf freeze shooter owners', own_targets[key]['shooter'], native_targets[key]['shooter'], gp)
        except (KeyError, TypeError, ValueError, AssertionError, IndexError) as error:
            check('actual paired NPC freeze owners', repr(error), 'complete typed independent owners', gp)
    held.add(b['activeId'])
    for row in b['temporary']:
        uses[row[0]] += 1
    landed += b['entityCollisionOff']
for item in contract['uses']:
    check(f'actual Native active {item} held', item in held)
    check(f'actual Native active {item} use record', uses[item] > 0)
if 282 in contract['uses']:
    check('actual Native How to Jump airborne presentations', landed > 0)
freeze_observations = {}
if freeze_contract is not None:
    for name, rows in (('Native', native), ('Own', own)):
        try:
            freeze_observations[name] = freeze_witnesses(name, rows)
        except (KeyError, TypeError, ValueError, AssertionError, IndexError) as error:
            check(f'actual {name} freeze source witnesses', repr(error), 'complete typed independent witnesses')
after = {str(path): digest(path) for path in inputs}
check('all input archives before equal end', after, before)
result = dict(schema=1, status='fail' if failures else 'pass', case=case['name'],
    checks=sum(counts.values()), failures=sum(failures.values()),
    nativePresentations=len(native), ownPresentations=len(own), pairedPresentations=len(paired),
    checksPerField=dict(counts), failedPerField=dict(failures), examples=examples,
    unavailable=contract['unknown'] + (freeze_contract['unknown'] if freeze_contract else []), inputsBefore=before, inputsAfter=after,
    scope='Per-presentation start-room active slot, charge, temporary records, health, Player body, '
          'collision classes, named main-sprite clips and pedestals. No private order/render/backend claim.'
          + (' Additive independent bounded living NPC freeze state/motion/colour and actual phase-clock witnesses; '
             'the no-Space control retains ordinary NPC behavior. Frozen death, bosses, links and readmission remain open.' if freeze_contract else ''))
if freeze_contract is not None:
    result['npcFreezeWitnesses'] = freeze_observations
raw = (json.dumps(result, indent=2, allow_nan=False) + '\n').encode()
args.output.write_bytes(gzip.compress(raw, mtime=0) if args.output.suffix == '.gz' else raw)
print(json.dumps(dict(status=result['status'], checks=result['checks'], failures=result['failures'])))
raise SystemExit(bool(failures))
