#!/usr/bin/env python3
"""Public-input tapes that walk Isaac into a doorway off-centre and back.

Each case starts a seed whose free treasure room borders the starting room at
one door slot, so neither room shuts its doors. Isaac lines up away from the
doorway's centre (or not at all), then walks into it straight, diagonally or
by sliding along the wall. A capture every frame of the approach pins the
player's position and velocity and the frame the room changes. In the
treasure room he steps aside from the entry door and walks back out the same
way, again captured every frame.

Every variant is a fixed public input; the OS-random case draws a seed with a
treasure room at a random slot, a random variant and a random extra offset.
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


paired = load('door_approach_paired', 'foundation-paired-scenarios.py')
runner = paired.runner
GENERATOR = ROOT / 'engine/target/debug/floor'
# Seeds whose starting room has the free treasure room at slot 0..3.
SEEDS = {0: 'NXTC03X1', 1: 'ZFGP1MQ0', 2: 'MYE61TQ0', 3: 'PM8FT68Z'}
# Per slot: the key that walks into the door, the keys across it (towards the
# lower/greater coordinate first) and the key that walks back out.
AXES = {0: ('a', ('w', 's'), 'd'), 1: ('w', ('a', 'd'), 's'), 2: ('d', ('w', 's'), 'a'), 3: ('s', ('a', 'd'), 'w')}
# Isaac spawns at (320, 400): a side door needs 27 frames of 'w' (and
# settling) to line up with its centre at y=280; the top and bottom doors are
# already lined up.
LIFT = {0: [('w', 27)], 1: [], 2: [('w', 27)], 3: []}
# Diagonal: a setup that puts the doorway on a 45-degree line from Isaac, then
# the door key and one across key together. Slide: a setup that puts Isaac
# against the door's wall away from the doorway, then the door key and the
# across key that slides him along the wall into it.
DIAGONAL = {0: ([('a', 24)], 'w'), 1: ([('d', 60)], 'a'), 2: ([('d', 24)], 'w'), 3: ([('d', 60), ('w', 20)], 'a')}
SLIDE = {0: ([('a', 60)], 'w'), 1: ([('d', 60), ('w', 60)], 'a'), 2: ([('d', 60)], 'w'), 3: ([('d', 60)], 'a')}


def variant_plan(slot, variant):
    """(setup holds, keys held into the door, the across hold before the walk back)."""
    towards, across, _ = AXES[slot]
    if variant == 'diagonal':
        return DIAGONAL[slot][0], [towards, DIAGONAL[slot][1]], None
    if variant == 'slide':
        return SLIDE[slot][0], [towards, SLIDE[slot][1]], None
    side, frames = OFFSETS[variant]
    hold = [(across[side], frames)] if frames else []
    return LIFT[slot] + hold, [towards], (hold or [None])[0]


# Off-centre line-ups: the across key (0 low, 1 high) held for this many frames
# before the walk, and again in the treasure room before the walk back.
# Four frames shift Isaac about 17 units, six about 26: either side of where
# the port's doorway stops letting him through.
OFFSETS = {'centre': (0, 0), 'offset-low-4': (0, 4), 'offset-low-6': (0, 6),
           'offset-high-4': (1, 4), 'offset-high-6': (1, 6)}
VARIANTS = [*OFFSETS, 'diagonal', 'slide']
APPROACH = 100
BACK = 70


def actions(slot, variant):
    towards, across, back = AXES[slot]
    setup, keys, back_shift = variant_plan(slot, variant)
    tape = []
    def key(name, down): tape.append({'type': 'key', 'key': name, 'down': down})
    def step(count): tape.append({'type': 'step', 'frames': count})
    def capture(label): tape.append({'type': 'capture', 'label': label})
    def hold(name, frames): key(name, True); step(frames); key(name, False); step(30)
    def dense(count, prefix):
        for index in range(count):
            step(1); capture(f'{prefix}-{index + 1:03d}')
    step(60); capture('run-ready')
    for name, frames in setup:
        hold(name, frames)
    capture('lined-up')
    for name in keys: key(name, True)
    dense(APPROACH, 'in')
    for name in keys: key(name, False)
    step(40); capture('entered')
    # Step aside from the entry door in the treasure room, then walk back out.
    if back_shift:
        hold(*back_shift)
    key(back, True); dense(BACK, 'out'); key(back, False)
    step(40); capture('returned')
    return tape


def treasure_slots(run_seed):
    result = subprocess.run([str(GENERATOR)], input=json.dumps({'stageSeed': runner.codec.initial_stage_seed(run_seed)}),
                            capture_output=True, text=True, timeout=20, check=True)
    floor = json.loads(result.stdout)
    start = floor['descriptors'][floor['startingDescriptor']]
    return [slot for slot, index in enumerate(start['neighbors'][:4])
            if index is not None and floor['descriptors'][index]['template']['type'] == 4]


def case(code, slot, variant, randomized, provenance):
    if slot not in treasure_slots(runner.codec.decode_seed(code)):
        raise ValueError(f'{code} has no treasure room at slot {slot}')
    result = paired.build('paired-bomb-moving-even', code)
    route = actions(slot, variant)
    for name in ('tape', 'pairedTape'):
        start = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:start + 1] + route
    runner.validate_tape(result['tape'])
    result['scenario'] = {'id': f'door-approach-{code}-{slot}-{variant}',
                          'description': f'Walk into door slot {slot} {variant} and back ({provenance})',
                          'systems': ['player-movement', 'grid-collision', 'room-connectivity', 'input-determinism'],
                          'randomized': randomized}
    result['doorApproach'] = {'slot': slot, 'variant': variant, 'provenance': provenance}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1, help='OS-random seeds')
    parser.add_argument('--only', action='append', default=[], help='SLOT:VARIANT fixed cases to keep')
    parser.add_argument('--attempts', type=int, default=5000)
    args = parser.parse_args()
    fixed = [(slot, variant) for slot in SEEDS for variant in VARIANTS]
    if args.only:
        fixed = [pair for pair in fixed if f'{pair[0]}:{pair[1]}' in args.only]
    cases = [case(SEEDS[slot], slot, variant, False, 'fixed seed and variant') for slot, variant in fixed]
    entropy = random.SystemRandom()
    for _ in range(args.attempts):
        if len(cases) == len(fixed) + args.count:
            break
        vector = runner.codec.random_vector()
        slots = treasure_slots(vector['seedValue'])
        if slots:
            cases.append(case(runner.codec.encode_seed(vector['seedValue']), entropy.choice(slots),
                              entropy.choice(VARIANTS), True, 'recorded OS-random run seed and variant'))
    if len(cases) < len(fixed) + args.count:
        raise RuntimeError('Not enough OS-random seeds with a treasure room next to the start')
    print(json.dumps({'schema': 1, 'failures': [], 'cases': cases}))


if __name__ == '__main__':
    main()
