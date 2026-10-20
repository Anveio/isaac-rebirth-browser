#!/usr/bin/env python3
"""Public-input tapes that clear a boss room, enter its trapdoor and reach the next floor.

Both games walk the same recorded route into the boss room (the reference
fixture). The fight cannot be one shared tape: no fixed key sequence kills the
boss in both games, because hidden RNG still diverges. Each game therefore
fights with its own public keys over a fixed window. The original replays
the fixture's recorded keys, which a fresh original reproduces exactly. The
port replays keys recorded closed-loop in the production build
(boss-route-record.mjs --fight). Both segments end parked in the bottom-left
corner and are padded to the same frame count. After `exit-start` both games
receive identical keys, recorded in the port with --exit, onto the award
trapdoor and through the stage transition. Only the shared tail is graded.

A descent fixture (`descent`, one entry per floor) carries the run further
before the shared tail: each game clears Basement I's boss room, takes its
trapdoor, walks Basement II's room chain and clears that boss room too, all
with its own keys (the original's are the fixture's fixed `originalFight`).
The shared tail then re-anchors in the last boss room and grades its exit
into the floor after it.

Contrived cases walk straight to the trapdoor. OS-random cases draw an idle
wait and a waypoint for the approach; every recorded case replays exactly.
"""
import argparse
import importlib.util
import json
import os
import sys
from pathlib import Path
import random
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('stage_exit_boss_routes', Path(__file__).with_name('boss-scenarios.py'))
routes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(routes)
FIXTURES = ROOT / 'references/isaac/foundation/stage-exit'
RECORDINGS = ROOT / '.cache/stage-exit-recordings'
# Frames from boss-room-arrival to exit-start; each game's fight must fit.
WINDOW = 9000
# Frames a descent adds per earlier floor for its trapdoor, the stage
# transition and the next floor's route into its boss room.
EXIT_LEG = 3000
PADDING_STEP = 600
# Frames both games stand still after re-entering the boss room.
ANCHOR_IDLE = 120
# Frames the port's walk may spend in each route room of a later floor, whose
# live foes it fights clear before the doors open; a descent floor's
# `roomFrames` sets its own.
ROUTE_ROOM_FRAMES = 2400
# Dense capture labels and floor names after the shared exit, by arrival stage.
ARRIVALS = {2: ('basement-ii', 'Basement II'), 3: ('caves-i', 'Caves I'), 4: ('caves-ii', 'Caves II')}
# Cells the approach waypoint avoids: the trapdoor/pedestal column neighbours
# and the parked corner.
AWARD_CELLS = {22, 36, 37, 38, 52, 82, 96, 97, 98, 112, 106}


def retain(path, content):
    """Exclusive attempt files remain private even when the caller aborts."""
    with path.open('xb') as handle:
        handle.write(content); handle.flush(); os.fsync(handle.fileno())


def record(base, *args, rooms='0', arrival='', production_directory=None):
    RECORDINGS.mkdir(parents=True, exist_ok=True)
    attempt = Path(tempfile.mkdtemp(prefix='attempt-', dir=RECORDINGS))
    parent = os.open(RECORDINGS, os.O_RDONLY | os.O_DIRECTORY)
    try: os.fsync(parent)
    finally: os.close(parent)
    print(f'Stage-exit recorder attempt retained at {attempt}', file=sys.stderr, flush=True)
    input_path = attempt / 'case.json'
    retain(input_path, json.dumps(base).encode())
    production = ['--production-directory', str(production_directory)] if production_directory else []
    argv = ['node', str(routes.RECORDER), '--case', str(input_path), '--rooms', rooms, '--arrival', arrival,
            '--attempt-output', str(attempt), *production, *map(str, args)]
    retain(attempt / 'request.json', (json.dumps({'argv': argv, 'cwd': str(ROOT)}) + '\n').encode())
    completed, failure, returned = None, None, False
    try:
        with (attempt / 'stdout.raw').open('xb') as stdout, (attempt / 'stderr.raw').open('xb') as stderr:
            try:
                completed = subprocess.run(argv, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=3600)
            finally:
                for stream in (stdout, stderr):
                    stream.flush(); os.fsync(stream.fileno())
        lines = (attempt / 'stdout.raw').read_bytes().strip().splitlines()
        if not lines:
            raise RuntimeError(f'Stage-exit recorder produced no result; retained at {attempt}')
        retain(attempt / 'returned-result.raw.json', lines[-1])
        result = json.loads(lines[-1])
        returned = True
        return result
    except BaseException as error:
        failure = {'type': type(error).__name__, 'message': str(error)}
        raise
    finally:
        retain(attempt / 'execution.json', (json.dumps({
            'returnCode': completed.returncode if completed is not None else None,
            'error': failure, 'returnedJsonParsed': returned,
            'artifactObserved': (attempt / 'artifact.json').is_file(),
        }) + '\n').encode())
        directory = os.open(attempt, os.O_RDONLY | os.O_DIRECTORY)
        try: os.fsync(directory)
        finally: os.close(directory)


def port_fight(base, *args, **options):
    """A port fight leg. A fight whose player takes the boss item (the port
    alone; the original's pedestal keeps it) is recorded again keeping off
    the pedestal cells."""
    leg = record(base, *args, **options)
    if leg['status'] != 'pass' and leg.get('reason') == 'The port took the boss item':
        leg = record(base, *args, '--keep-off-pedestal', **options)
    if leg['status'] != 'pass' and str(leg.get('reason', '')).startswith('Room not cleared'):
        # The port's own fight is ungraded closed-loop input. When the default
        # orbit loses, the leading aim and tighter orbit (fightKeys --lead)
        # records a different fight that may clear the room.
        leg = record(base, *args, '--lead', **options)
    if leg['status'] != 'pass' and str(leg.get('reason', '')).startswith("The port's player died"):
        # The same ungraded closed-loop fight, lost to the port's own death
        # rather than to the clock: the leading aim records a different fight.
        leg = record(base, *args, '--lead', **options)
    return leg


def keys_only(actions):
    """A per-game segment carries no captures: only the shared tail is paired."""
    return [action for action in actions if action['type'] != 'capture']


def frames(actions):
    return sum(action['frames'] for action in actions if action['type'] == 'step')


def window(fixture):
    """Frames from boss-room-arrival to exit-start for this fixture's fights. A
    fixture whose later floors take longer than WINDOW each (Caves I's eight-room
    route and Mega Fatty) names its own window."""
    floors = len(fixture.get('descent', [None]))
    return fixture.get('window', WINDOW * floors + EXIT_LEG * (floors - 1))


def final_rooms(fixture):
    """The room chain of the floor whose boss room the shared tail leaves."""
    return fixture['descent'][-1]['rooms'] if 'descent' in fixture else fixture['rooms']


def hurt(health):
    """--hurt when [red, maxRed, soul] ends a fight below full health."""
    red, maximum, soul = health or [0, 0, 0]
    return ['--hurt'] if red + soul < maximum else []


def avoided(fixture):
    """Last boss-room cells the shared walks keep off: the original's own
    solid cells and the cells by the pickups its fight leaves."""
    return sorted(set(fixture.get('originalSolid', [])) | set(near(fixture.get('originalPickups', []))))


def descended(result, fixture, production_directory):
    """The port's own segment for a descent fixture, one closed-loop leg per
    floor, each replaying the legs before it: the leg fights the boss room
    clear and parks (with --hurt as that floor's original fight ends), and on
    every floor but the last also takes the trapdoor; the next leg starts by
    walking that floor's room chain into its boss room."""
    actions, build = [], None
    floors = fixture['descent']
    for index, floor in enumerate(floors):
        marker = f'descent-{index}'
        case = {**result, 'pairedTape': result['pairedTape'] + keys_only(actions) + [{'type': 'capture', 'label': marker}]}
        rooms = floor['rooms'][-1:] if index == 0 else floor['rooms']
        # routeAvoid {room: [cells]}: cells the walk keeps off beyond the
        # routing grid's own solid cells.
        avoid = ';'.join(f"{room}:{','.join(map(str, cells))}" for room, cells in floor.get('routeAvoid', {}).items())
        last = index + 1 == len(floors)
        # A floor's `fight` budget covers a longer boss (Mega Fatty's 600 HP).
        leg = port_fight(case, '--from', marker, '--fight', floor.get('fight', WINDOW), *hurt(floor.get('originalHealth')),
                         *([] if last else ['--exit', EXIT_LEG]),
                         *(['--room-frames', floor.get('roomFrames', ROUTE_ROOM_FRAMES)] if index else []),
                         # A floor's `fightFlags` tune its port leg's closed-loop fight
                         # (boss-route-record.mjs --dodge, --orbit, --boss-radius):
                         # Caves I's fire rooms and Mega Fatty's 30-shot bursts.
                         *map(str, floor.get('fightFlags', [])),
                         *(['--route-avoid', avoid] if avoid else []),
                         *(['--avoid', ','.join(map(str, avoided(fixture)))] if last and avoided(fixture) else []),
                         rooms=','.join(map(str, rooms)), arrival='', production_directory=production_directory)
        if leg['status'] != 'pass':
            raise RuntimeError(f"Port descent leg {index} for {fixture['name']} failed: {leg['reason']}")
        if build not in (None, leg.get('build')):
            raise ValueError('Every descent leg must be recorded on the same production artifact')
        actions, build = actions + leg['actions'], leg.get('build')
    return {'status': 'pass', 'actions': actions, 'build': build}


def fought(fixture, production_directory=None):
    """The shared route plus each game's own fight, recorded once per fixture.

    The port's fight is its own closed-loop keys (boss-route-record.mjs
    --fight). After the clear they also shoot out any poop the port's fight
    dropped in the cell inside an open doorway: the shared re-anchor walk
    needs that doorway clear, and the original's fixture fight left it so.
    Those keys only shape the port's own segment; no engine rule changes.

    Basement II's generation reads the Player's health and coins at the
    transition (a Challenge room needs full health, a stage-2 Arcade more
    than four coins), which each game's own fight decides. A fixture's
    original fight must leave the same full-health and coin thresholds as
    the port's, or the next floor's rooms diverge; a fixture whose original
    ends hurt says so (originalHealth) and the port's segment matches it."""
    result = routes.paired.build('paired-bomb-moving-even', fixture['seed'])
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + fixture['route']
    # originalHealth [red, maxRed, soul]: the original's fight ends below full
    # health, so a port fight at full health takes one bomb blast (--hurt).
    if 'descent' in fixture:
        port = descended(result, fixture, production_directory)
    else:
        port = port_fight(result, '--from', 'boss-room-arrival', '--fight', WINDOW, *hurt(fixture.get('originalHealth')),
                          *(['--avoid', ','.join(map(str, avoided(fixture)))] if avoided(fixture) else []),
                          production_directory=production_directory)
    if port['status'] != 'pass':
        raise RuntimeError(f"Port fight for {fixture['name']} failed: {port['reason']}")
    build = port.get('build')
    if not isinstance(build, str) or len(build) != 64 or any(char not in '0123456789abcdef' for char in build):
        raise ValueError('Port fight must identify its actual production artifact')
    result['portRecordedBuild'] = build
    fights = {'tape': fixture['originalFight'], 'pairedTape': keys_only(port['actions'])}
    for name, fight in fights.items():
        if frames(fight) + 240 > window(fixture):
            raise RuntimeError(f"{fixture['name']} {name} fight takes {frames(fight)} of {window(fixture)} window frames")
    return result, fights


def padding(total):
    """The window's remainder in steps of at most PADDING_STEP frames. The
    original runner bounds each step's host wall time (--native-frame-timeout),
    and several thousand frames in one step can exceed it on a loaded host.
    Only the last step carries the `exit-start` label."""
    steps = [{'type': 'step', 'frames': PADDING_STEP} for _ in range((total - 1) // PADDING_STEP)]
    return steps + [{'type': 'step', 'frames': total - frames(steps), 'label': 'exit-start'}]


def near(points):
    """Boss-room cells whose centre lies within 30 units of a pickup: a
    Player stepping from centre to centre there would touch it."""
    return [row * 15 + col for row in range(1, 8) for col in range(1, 14)
            if any(((40 + col * 40 - x) ** 2 + (120 + row * 40 - y) ** 2) ** 0.5 < 30 for x, y in points)]


def case(fixture, base, fights, identifier, randomized, idle, via, production_directory=None, candidates=None):
    result = json.loads(json.dumps(base))
    for name, fight in fights.items():
        # `exit-start` labels the padding step rather than a capture: where
        # each game's corner press releases depends on its fight, so the first
        # graded checkpoint comes after the trapdoor snaps the player.
        result[name] = result[name] + fight + padding(window(fixture) - frames(fight) + idle)
    # Re-anchor: each game parks after its own fight at a slightly different
    # sub-unit point, which can shift the trapdoor entry by a frame. Leaving
    # the boss room and coming back places both players exactly at the door,
    # and the idle lets that entry velocity decay, so the shared walk starts
    # from one state. The legs' own route captures still carry the fight
    # offset and are dropped; `re-anchor` is the first paired capture.
    # The original's own fight can leave solid cells (a Larry Jr. tail poop)
    # that the port's fight does not; the shared walk routes around them, and
    # stepping from cell centre to cell centre keeps it off their corners.
    rooms = final_rooms(fixture)
    back = f"{rooms[-1]},{rooms[-2]},{rooms[-1]}"
    avoid = avoided(fixture)
    tail = record(result, '--from', 'exit-start', '--idle', ANCHOR_IDLE, '--exit', 1500,
                  *(['--via', f'{via[0]},{via[1]}'] if via else []),
                  *(['--via-candidates', ';'.join(f'{x},{y}' for x, y in candidates)] if candidates else []),
                  *(['--avoid', ','.join(map(str, avoid))] if avoid else []), rooms=back, arrival='re-anchor',
                  production_directory=production_directory)
    if tail['status'] != 'pass':
        return None, f"port-exit: {tail['reason']}"
    # The recorder reports the candidate waypoint it walked through.
    via = tail.get('via') or via
    if tail.get('build') != base.get('portRecordedBuild'):
        raise ValueError('Port fight and exit must be recorded on the same production artifact')
    anchor = next(index for index, action in enumerate(tail['actions']) if action.get('label') == 're-anchor')
    actions = keys_only(tail['actions'][:anchor]) + tail['actions'][anchor:]
    arrived = 1 + len(fixture.get('descent', [None]))
    label, floor = ARRIVALS[arrived]
    routes.dense(actions, 120, label, 4)
    for name in ('tape', 'pairedTape'):
        result[name] = result[name] + actions
    routes.runner.validate_tape(result['tape'])
    result['scenario'] = {'id': identifier,
                          'description': f"Clear boss room {rooms[-1]}, enter its trapdoor and arrive in {floor}",
                          'systems': ['room-connectivity', 'grid-entities', 'run-introduction', 'entity-animation', 'input-determinism'], 'randomized': randomized}
    # Each game fights with its own keys, so red HP after the fight is not a
    # parity observation; the stage-exit comparator grades the exit itself.
    result['pairedIndependent'] = [{'after': 'boss-room-arrival', 'fields': ['player.hp'],
                                    'reason': 'each game fights the boss with its own keys; post-fight red HP depends on that fight'}]
    # The port's fight keys are closed-loop against this production build; a
    # replay on another build feeds them to a different game (state-pair warns).
    result['portRecordedBuild'] = tail.get('build')
    result['stageExit'] = {'fixture': fixture['name'], 'rooms': rooms, 'arrivedStage': arrived, 'window': window(fixture),
                           'idle': idle, 'via': via,
                           'fightFrames': {name: frames(fight) for name, fight in fights.items()},
                           'recordedBuilds': {'fight': base['portRecordedBuild'], 'exit': tail['build']}}
    if 'carriedActiveItemId' in fixture:
        item_id = fixture['carriedActiveItemId']
        if type(item_id) is not int or not 1 <= item_id <= 346:
            raise ValueError('The carried active item must be an authored positive collectible ID')
        result['stageExit']['carriedActiveItemId'] = item_id
    return result, None


def main():
    global RECORDINGS
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--contrived', action='append', default=[], metavar='FIXTURE', help='fixture name under references/.../stage-exit')
    parser.add_argument('--count', type=int, default=1, help='OS-random approach cases per fixture')
    parser.add_argument('--production-directory', help='Existing immutable production artifact for both public-input recordings')
    parser.add_argument('--recordings-output', type=Path, default=RECORDINGS, help='Private recorder attempt archive, retained on every failure')
    args = parser.parse_args()
    RECORDINGS = args.recordings_output.resolve()
    entropy = random.SystemRandom()
    cases, failures = [], []
    for name in args.contrived:
        fixture = {**json.loads((FIXTURES / f'{name}.json').read_text()), 'name': name}
        base, fights = fought(fixture, args.production_directory)
        built, reason = case(fixture, base, fights, f'{name}-contrived', False, 0, None, args.production_directory)
        if built is None:
            raise ValueError(f'Contrived stage exit {name} failed: {reason}')
        cases.append(built)
        for index in range(args.count):
            # The waypoint comes from an OS-shuffled candidate order: the
            # recorder takes the first one clear of the port's own solid
            # cells (authored rocks, its fight's poops) that it can route to,
            # or takes the trapdoor directly when none is. A retry reshuffles.
            for _ in range(2):
                solid = set(fixture.get('originalSolid', []))
                # Turning at the waypoint carries the player about ten units
                # past its centre: beside a cell solid in the original alone
                # (a fight's tail poop), only the original's player touches it
                # and the shared walk splits (#649). Keep one cell clear.
                beside = {cell + step for cell in solid for step in (-15, 15, -1, 1)}
                blocked = AWARD_CELLS | solid | beside | set(near(fixture.get('originalPickups', [])))
                cells = [c + r * 15 for c in range(1, 14) for r in range(1, 8) if c + r * 15 not in blocked]
                entropy.shuffle(cells)
                built, reason = case(fixture, base, fights, f'{name}-random-{index}', True, entropy.randrange(0, 241), None,
                                     args.production_directory, [[40 + cell % 15 * 40, 120 + cell // 15 * 40] for cell in cells])
                if built is not None or not reason.startswith('port-exit'):
                    break
            if built is None:
                failures.append({'fixture': name, 'error': reason})
            else:
                cases.append(built)
    print(json.dumps({'schema': 1, 'failures': failures, 'cases': cases}))


if __name__ == '__main__':
    main()
