#!/usr/bin/env python3
"""Public-input tapes that clear a boss room, enter its trapdoor and reach Basement II.

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

Contrived cases walk straight to the trapdoor. OS-random cases draw an idle
wait and a waypoint for the approach; every recorded case replays exactly.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import random
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('stage_exit_boss_routes', Path(__file__).with_name('boss-scenarios.py'))
routes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(routes)
FIXTURES = ROOT / 'references/isaac/foundation/stage-exit'
# Frames from boss-room-arrival to exit-start; each game's fight must fit.
WINDOW = 9000
# Cells the approach waypoint avoids: the trapdoor/pedestal column neighbours
# and the parked corner.
AWARD_CELLS = {22, 36, 37, 38, 52, 82, 96, 97, 98, 112, 106}


def record(base, *args):
    with tempfile.NamedTemporaryFile('w', suffix='.json') as handle:
        json.dump(base, handle); handle.flush()
        result = subprocess.run(['node', str(routes.RECORDER), '--case', handle.name, '--rooms', '0', '--arrival', '', *map(str, args)],
                                cwd=ROOT, capture_output=True, text=True, timeout=3600)
    lines = result.stdout.strip().splitlines()
    if not lines:
        raise RuntimeError(f'Stage-exit recorder failed: {result.stderr.strip()[-2000:]}')
    return json.loads(lines[-1])


def keys_only(actions):
    """A per-game segment carries no captures: only the shared tail is paired."""
    return [action for action in actions if action['type'] != 'capture']


def frames(actions):
    return sum(action['frames'] for action in actions if action['type'] == 'step')


def fought(fixture):
    """The shared route plus each game's own fight, recorded once per fixture."""
    result = routes.paired.build('paired-bomb-moving-even', fixture['seed'])
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + fixture['route']
    port = record(result, '--from', 'boss-room-arrival', '--fight', WINDOW)
    if port['status'] != 'pass':
        raise RuntimeError(f"Port fight for {fixture['name']} failed: {port['reason']}")
    fights = {'tape': fixture['originalFight'], 'pairedTape': keys_only(port['actions'])}
    for name, fight in fights.items():
        if frames(fight) + 240 > WINDOW:
            raise RuntimeError(f"{fixture['name']} {name} fight takes {frames(fight)} of {WINDOW} window frames")
    return result, fights


def case(fixture, base, fights, identifier, randomized, idle, via):
    result = json.loads(json.dumps(base))
    for name, fight in fights.items():
        # `exit-start` labels the padding step rather than a capture: where
        # each game's corner press releases depends on its fight, so the first
        # graded checkpoint comes after the trapdoor snaps the player.
        result[name] = result[name] + fight + [{'type': 'step', 'frames': WINDOW - frames(fight) + idle, 'label': 'exit-start'}]
    tail = record(result, '--from', 'exit-start', '--exit', 1500, *(['--via', f'{via[0]},{via[1]}'] if via else []))
    if tail['status'] != 'pass':
        return None, f"port-exit: {tail['reason']}"
    actions = list(tail['actions'])
    routes.dense(actions, 120, 'basement-ii', 4)
    for name in ('tape', 'pairedTape'):
        result[name] = result[name] + actions
    routes.runner.validate_tape(result['tape'])
    result['scenario'] = {'id': identifier,
                          'description': f"Clear boss room {fixture['rooms'][-1]}, enter its trapdoor and arrive in Basement II",
                          'systems': ['room-connectivity', 'grid-entities', 'run-introduction', 'entity-animation', 'input-determinism'], 'randomized': randomized}
    # Each game fights with its own keys, so red HP after the fight is not a
    # parity observation; the stage-exit comparator grades the exit itself.
    result['pairedIndependent'] = [{'after': 'boss-room-arrival', 'fields': ['player.hp'],
                                    'reason': 'each game fights the boss with its own keys; post-fight red HP depends on that fight'}]
    result['stageExit'] = {'fixture': fixture['name'], 'rooms': fixture['rooms'], 'window': WINDOW, 'idle': idle, 'via': via,
                           'fightFrames': {name: frames(fight) for name, fight in fights.items()}}
    return result, None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--contrived', action='append', default=[], metavar='FIXTURE', help='fixture name under references/.../stage-exit')
    parser.add_argument('--count', type=int, default=1, help='OS-random approach cases per fixture')
    args = parser.parse_args()
    entropy = random.SystemRandom()
    cases, failures = [], []
    for name in args.contrived:
        fixture = {**json.loads((FIXTURES / f'{name}.json').read_text()), 'name': name}
        base, fights = fought(fixture)
        built, reason = case(fixture, base, fights, f'{name}-contrived', False, 0, None)
        if built is None:
            raise ValueError(f'Contrived stage exit {name} failed: {reason}')
        cases.append(built)
        for index in range(args.count):
            # A few draws, since a drawn waypoint can sit on an authored rock.
            for _ in range(3):
                cell = entropy.choice([c + r * 15 for c in range(1, 14) for r in range(1, 8) if c + r * 15 not in AWARD_CELLS])
                built, reason = case(fixture, base, fights, f'{name}-random-{index}', True, entropy.randrange(0, 241),
                                     [40 + cell % 15 * 40, 120 + cell // 15 * 40])
                if built is not None or not reason.startswith('port-exit'):
                    break
            if built is None:
                failures.append({'fixture': name, 'error': reason})
            else:
                cases.append(built)
    print(json.dumps({'schema': 1, 'failures': failures, 'cases': cases}))


if __name__ == '__main__':
    main()
