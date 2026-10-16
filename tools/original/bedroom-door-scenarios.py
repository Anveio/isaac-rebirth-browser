#!/usr/bin/env python3
"""Public-input tapes that bomb a cracked bedroom door from the starting room.

Door::SetRoomTypes 565b00 gives a doorway into a bedroom (Isaac's room 18 or
the barren room 19) the cracked variant3. Fresh Isaac carries one bomb: its
blast damages the door (TryBlowOpen 564870: state4) and scatters Random(5)+6
Wood Particles 1000.27, but the door stays shut. Both games receive identical
keys: walk to the doorway, drop the bomb, retreat, watch the blast and the
particles land, then push against the damaged door.

The contrived case pins an Isaac's room above the start. OS-random cases draw
run seeds until the port's floor generator puts a bedroom next to the start
room, and an idle wait before the walk.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import random
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


paired = load('bedroom_paired', 'foundation-paired-scenarios.py')
runner = paired.runner
GENERATOR = ROOT / 'engine/target/debug/floor'
CONTRIVED = [('GY0CAQWE', 'Isaac\'s room above the starting room')]
# Per door slot (left, up, right, down): the setup that lines Isaac up with the
# doorway from his spawn (320, 400), the key held into the doorway and its
# frames, and the retreat key. The retreat leaves the blast radius.
ROUTES = {0: ([('w', 27)], 'a', 72, 'd'), 1: ([], 'w', 70, 's'),
          2: ([('w', 27)], 'd', 72, 'a'), 3: ([], 's', 6, 'w')}


def actions(slot, idle):
    tape = []
    def key(name, down): tape.append({'type': 'key', 'key': name, 'down': down})
    def step(count): tape.append({'type': 'step', 'frames': count})
    def capture(label): tape.append({'type': 'capture', 'label': label})
    def dense(count, prefix):
        for index in range(count):
            step(1); capture(f'{prefix}-{index + 1}')
    setup, towards, frames, retreat = ROUTES[slot]
    step(60 + idle); capture('run-ready')
    for name, count in setup:
        key(name, True); step(count); key(name, False); step(30)
    key(towards, True); step(frames); key(towards, False); step(10); capture('at-door')
    key('e', True); step(1); key('e', False); step(1); capture('bomb-birth')
    key(retreat, True); step(35); key(retreat, False); step(10); capture('retreated')
    # The bomb fuse ends about 120 frames after its birth; the particles land
    # within a further 45.
    dense(110, 'blast')
    key(towards, True); dense(60, 'push'); key(towards, False)
    step(10); capture('pushed')
    return tape


def bedroom_slots(run_seed):
    result = subprocess.run([str(GENERATOR)], input=json.dumps({'stageSeed': runner.codec.initial_stage_seed(run_seed)}),
                            capture_output=True, text=True, timeout=20, check=True)
    floor = json.loads(result.stdout)
    start = floor['descriptors'][floor['startingDescriptor']]
    return [(slot, floor['descriptors'][index]['template']['type'])
            for slot, index in enumerate(start['neighbors'][:4])
            if index is not None and floor['descriptors'][index]['template']['type'] in (18, 19)]


def case(code, slot, room_type, idle, randomized, provenance):
    result = paired.build('paired-bomb-moving-even', code)
    route = actions(slot, idle)
    for name in ('tape', 'pairedTape'):
        start = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:start + 1] + route
    runner.validate_tape(result['tape'])
    result['scenario'] = {'id': f'bedroom-door-{code}-{slot}', 'description': f'Bomb the cracked door into bedroom type {room_type} ({provenance})',
                          'systems': ['grid-entities', 'room-connectivity', 'entity-animation', 'input-determinism'], 'randomized': randomized}
    result['bedroomDoor'] = {'slot': slot, 'roomType': room_type, 'idle': idle, 'provenance': provenance}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1, help='OS-random seeds with a bedroom next to the start room')
    parser.add_argument('--attempts', type=int, default=20000)
    args = parser.parse_args()
    cases = []
    for code, provenance in CONTRIVED:
        slots = bedroom_slots(runner.codec.decode_seed(code))
        if not slots:
            raise ValueError(f'Contrived seed {code} has no bedroom next to its starting room')
        cases.append(case(code, *slots[0], 0, False, provenance))
    entropy = random.SystemRandom()
    tried = set()
    for _ in range(args.attempts):
        if len(cases) == len(CONTRIVED) + args.count:
            break
        vector = runner.codec.random_vector()
        if vector['seedValue'] in tried:
            continue
        tried.add(vector['seedValue'])
        slots = bedroom_slots(vector['seedValue'])
        if slots:
            slot, room_type = entropy.choice(slots)
            cases.append(case(runner.codec.encode_seed(vector['seedValue']), slot, room_type, entropy.randrange(0, 61), True,
                              'recorded OS-random run seed'))
    if len(cases) < len(CONTRIVED) + args.count:
        raise RuntimeError(f'Only {len(cases) - len(CONTRIVED)} of {args.count} OS-random bedroom doors in {args.attempts} seeds')
    print(json.dumps({'schema': 1, 'failures': [], 'cases': cases}))


if __name__ == '__main__':
    main()
