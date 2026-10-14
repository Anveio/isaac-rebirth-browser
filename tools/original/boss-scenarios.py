#!/usr/bin/env python3
"""Public-input tapes that walk from the starting room into a boss room.

Discovery draws OS-random run seeds and keeps floors whose boss room holds the
spec's boss and is reachable from the start through ordinary rooms with no
hostile spawns. The production build records the route closed-loop
(tools/e2e/boss-route-record.mjs); both games then replay the same public
keys. Every recorded route is validated against a fresh original process,
which must stand in the boss room at `boss-room-arrival`; seeds it misses are
dropped and listed with counts in the output's `discovery` block. The fight
segment after that checkpoint is shared with later progression tapes (boss
kill, trapdoor), which start from the same arrival checkpoint.
"""
import argparse
from collections import Counter
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


paired = load('boss_paired', 'foundation-paired-scenarios.py')
runner = load('boss_clock', 'foundation-clock-run.py')
codec = runner.codec
GENERATOR = ROOT / 'engine/target/debug/floor'
RECORDER = ROOT / 'tools/e2e/boss-route-record.mjs'
# Pickups, slot machines, shopkeepers and fireplaces never shut doors.
PASSIVE = {17, 33}
AIM = ('Left', 'Up', 'Right', 'Down')


def key(name, down):
    return {'type': 'key', 'key': name, 'down': down}


def dense(tape, frames, prefix, every=1):
    for index in range(every, frames + 1, every):
        tape += [{'type': 'step', 'frames': every}, {'type': 'capture', 'label': f'{prefix}-{index}'}]


def floor(run_seed):
    result = subprocess.run([str(GENERATOR)], input=json.dumps({'stageSeed': codec.initial_stage_seed(run_seed), 'spawns': True}),
                            capture_output=True, text=True, timeout=20, check=True)
    return json.loads(result.stdout)


def quiet(spawns):
    return not spawns or all(row['category'] != 'entity' or row['type'] < 10 or row['type'] in PASSIVE
                             for row in spawns['selectors'])


def plan(generated, boss):
    """Shortest room chain start → boss room through quiet ordinary rooms, or a rejection reason."""
    descriptors, spawns = generated['descriptors'], generated['spawns']
    room = next(index for index, descriptor in enumerate(descriptors) if descriptor['template']['type'] == 5)
    if not any(row['category'] == 'entity' and (row['type'], row['variant']) == boss for row in spawns[room]['selectors']):
        return None, 'other-boss'
    start = generated['startingDescriptor']
    previous, queue = {start: None}, [start]
    while queue and room not in previous:
        current = queue.pop(0)
        for slot, neighbor in enumerate(descriptors[current]['neighbors']):
            if neighbor is None or neighbor in previous:
                continue
            template = descriptors[neighbor]['template']
            if neighbor != room and (template['type'] != 1 or not quiet(spawns[neighbor]) or generated['unsupportedNpcs'][neighbor]):
                continue
            previous[neighbor] = current
            queue.append(neighbor)
    if room not in previous:
        return None, 'no-quiet-path'
    rooms = [room]
    while previous[rooms[0]] is not None:
        rooms.insert(0, previous[rooms[0]])
    entry = descriptors[rooms[-2]]['neighbors'].index(room) % 4
    return {'rooms': rooms, 'entrySlot': entry, 'boss': descriptors[room]['template']['name']}, None


def fight(entry):
    """The boss segment after `boss-room-arrival`: Scene, then shoot along every axis."""
    tape = []
    dense(tape, 220, 'boss-scene', 2)
    for turn in range(4):
        aim = AIM[(entry + turn) % 4]
        tape.append(key(aim, True))
        dense(tape, 180 if turn == 0 else 120, f'boss-shoot{turn}', 2)
        tape.append(key(aim, False))
    dense(tape, 60, 'boss-idle', 2)
    return tape


def record(base, rooms):
    with tempfile.NamedTemporaryFile('w', suffix='.json') as handle:
        json.dump(base, handle); handle.flush()
        result = subprocess.run(['node', str(RECORDER), '--case', handle.name, '--rooms', ','.join(map(str, rooms))],
                                cwd=ROOT, capture_output=True, text=True, timeout=600)
    lines = result.stdout.strip().splitlines()
    if not lines:
        raise RuntimeError(f'Route recorder failed: {result.stderr.strip()[-2000:]}')
    return json.loads(lines[-1])


def arrival_tape(result):
    end = next(index for index, action in enumerate(result['tape']) if action.get('label') == 'boss-room-arrival')
    return result['tape'][:end + 1]


def original_route(result, directory):
    """The fresh original's room and player position at every route checkpoint."""
    directory.mkdir(parents=True, exist_ok=True)
    tape = directory / 'tape.json'
    tape.write_text(json.dumps(arrival_tape(result)))
    subprocess.run(['bash', 'tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
                    '--seed', result['seed'], '--tape', str(tape), '--state-observer', 'tools/original/foundation-combat-observe.py',
                    '--frame-timeout', '120', '--output', str(directory / 'original')],
                   cwd=ROOT, capture_output=True, text=True, timeout=1800)
    manifest = json.loads((directory / 'original/manifest.json').read_text())
    route = {}
    for row in manifest['debugObservations']:
        observed = row['observation']
        if observed.get('status') == 'pass' and observed.get('players'):
            route[row['context']['label']] = {'room': observed['room'], 'player': observed['players'][0]['position']}
    return route


def case(identifier, code, route, boss, provenance, randomized):
    result = paired.build('paired-bomb-moving-even', code)
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1]
    recorded = record(result, route['rooms'])
    if recorded['status'] != 'pass':
        return None, f"port-route: {recorded['reason']}", None
    tail = recorded['actions'] + fight(route['entrySlot'])
    for name in ('tape', 'pairedTape'):
        result[name] = result[name] + tail
    runner.validate_tape(result['tape'])
    result['scenario'] = {'id': identifier, 'description': f"Walk {len(route['rooms']) - 1} rooms into {route['boss']} and fight",
                          'systems': ['enemies-bosses', 'damage-pipeline', 'room-connectivity', 'projectile-physics'],
                          'randomized': randomized}
    result['entity'] = {'type': boss[0], 'variant': boss[1], 'shape': [13, 7], 'slot': route['entrySlot'], 'boss': True,
                        'rooms': route['rooms'], 'routeFrames': recorded['frames'], 'selection': provenance}
    return result, None, recorded['checkpoints']


def validate(result, checkpoints, scratch):
    """Reject a route the fresh original does not follow into the boss room,
    naming the first checkpoint where it left the port's path."""
    original = original_route(result, scratch)
    for label, port in checkpoints.items():
        source = original.get(label)
        if not source:
            return f'original-missing-{label}'
        room = source['room']['descriptorIndex']
        drift = max(abs(a - b) for a, b in zip(source['player'], port['player']))
        if room != port['room'] or drift > 40:
            return f"original-left-route at {label}: room {room} vs {port['room']}, drift {drift:.1f}"
    if original['boss-room-arrival']['room']['type'] != 5:
        return f"original-room-type-{original['boss-room-arrival']['room']['type']}"
    return None


def build(identifier, code, boss, provenance, randomized, checked, scratch):
    route, reason = plan(floor(codec.decode_seed(code)), boss)
    if route is None:
        return None, reason
    result, reason, checkpoints = case(identifier, code, route, boss, provenance, randomized)
    if result is None or not checked:
        return result, reason
    reason = validate(result, checkpoints, scratch / code.replace(' ', ''))
    return (None, reason) if reason else (result, None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--type', type=int, required=True, help='entities2 boss type')
    parser.add_argument('--variant', type=int, default=0)
    parser.add_argument('--contrived', action='append', default=[], metavar='CODE')
    parser.add_argument('--count', type=int, default=1, help='OS-random cases to discover (at least one when discovering)')
    parser.add_argument('--attempts', type=int, default=6000)
    parser.add_argument('--no-validate', action='store_true', help='skip the fresh-original arrival check (research only)')
    args = parser.parse_args()
    boss = (args.type, args.variant)
    checked = not args.no_validate
    counts, rejected, cases = Counter(), [], []
    # Rejected routes keep their original evidence for diagnosis.
    (ROOT / '.cache/boss-route').mkdir(parents=True, exist_ok=True)
    scratch = Path(tempfile.mkdtemp(prefix='validate-', dir=ROOT / '.cache/boss-route'))
    for index, code in enumerate(args.contrived):
        result, reason = build(f'contrived-{index}', code, boss, {'entropy': 'contrived spec seed'}, False, checked, scratch)
        if result is None:
            raise ValueError(f'Contrived {code} does not reach boss {boss}: {reason}')
        cases.append(result)
    found, tried = 0, set()
    for attempt in range(1, args.attempts + 1):
        if found == args.count:
            break
        vector = codec.random_vector()
        if vector['seedValue'] in tried:
            continue
        tried.add(vector['seedValue'])
        code = codec.encode_seed(vector['seedValue'])
        result, reason = build(f'random-{found}', code, boss, {'entropy': 'OS random', 'discoveryAttempts': attempt}, True, checked, scratch)
        if result is None:
            counts[reason.split(':')[0].split(' ')[0]] += 1
            if reason not in ('other-boss', 'no-quiet-path'):
                rejected.append({'code': code, 'reason': reason})
            continue
        cases.append(result)
        found += 1
    if found < args.count:
        raise RuntimeError(f'Only {found} of {args.count} routes into boss {boss} in {args.attempts} OS-random seeds: {dict(counts)} {rejected}')
    print(json.dumps({'schema': 1, 'entity': {'type': boss[0], 'variant': boss[1], 'shape': [13, 7], 'boss': True},
                      'discovery': {'seedsTried': len(tried), 'rejectedCounts': dict(counts), 'rejectedAfterPlanning': rejected,
                                    'originalValidated': checked},
                      'cases': cases}))


if __name__ == '__main__':
    main()
