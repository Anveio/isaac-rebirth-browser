#!/usr/bin/env python3
"""Natural Cellar Dip217 route; contrived phases and unfiltered OS seeds."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('dip_paired', Path(__file__).with_name('foundation-paired-scenarios.py'))
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)


def key(name, down):
    return {'type': 'key', 'key': name, 'down': down}


def observed(count, prefix, stride=1):
    result = []
    for at in range(0, count, stride):
        frames = min(stride, count - at)
        result += [{'type': 'step', 'frames': frames}, {'type': 'capture', 'label': f'{prefix}-{at + frames}'}]
    return result


def build(identifier, seed, phase, required):
    result = paired.build('paired-room-combat', seed)
    result['scenario'] = {'id': identifier, 'description': 'Natural Dip217 appearance, bursts, contact, ordered damage and poop death callbacks',
        'systems': ['npc-ai', 'entity-collision', 'damage-pipeline', 'room-connectivity', 'graphics-composition'],
        'randomized': not required}
    tail = observed(phase, 'phase') + [key('s', True)] + observed(17, 'depart', 4) + [key('s', False)]
    # Source Floor69 qualifies the retained contrived Cellar Room213 route.
    # Every entry presentation exposes real NPC init/Poof/full/fast staging.
    tail += observed(101, 'entry')
    tail += [key('s', True)] + observed(35, 'black-contact', 5) + [key('s', False)]
    tail += observed(45, 'collection', 5)
    tail += [key('w', True)] + observed(10, 'spike-contact') + [key('w', False)]
    tail += observed(48, 'black-damage')
    # Public firing attempts real death; failure to execute stays required.
    for direction in ['Down', 'Left', 'Right', 'Up']:
        tail += [key(direction, True)] + observed(72, f'shoot-{direction.lower()}', 4) + [key(direction, False)]
    tail += observed(24, 'clear')
    for name in ('tape', 'pairedTape'):
        at = next(n for n, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:at + 1] + tail
        paired.runner.validate_tape(result[name])
    result['dipRequired'] = required
    result['phaseOffset'] = phase
    result['dipScope'] = 'Actual admission, full/fast AI, geometry, queued damage, strict negative death and next-full retirement. Poop58/43 are required where original births exist, despite pending implementation. Blood7 paint/return and complete modifier/color callbacks remain unaccepted.'
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--count', type=int, default=2)
    args = parser.parse_args()
    if not 0 <= args.count <= 16:
        raise ValueError('Random count must be0..16')
    # Draw once before fixture construction or grading. No eligibility reroll.
    vectors = [paired.runner.codec.random_vector() for _ in range(args.count)]
    cases = [build(f'dip-cellar-phase{phase}', 'B9119QA3', phase, True) for phase in (0, 1)]
    cases += [build(f'dip-unfiltered-{n}', vector['code'], n % 2, False) for n, vector in enumerate(vectors)]
    print(json.dumps({'schema': 1, 'entropy': 'OS secrets; unfiltered and recorded before grading', 'randomVectors': vectors, 'cases': cases}))


if __name__ == '__main__':
    main()
