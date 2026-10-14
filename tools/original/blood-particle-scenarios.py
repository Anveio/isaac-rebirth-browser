#!/usr/bin/env python3
"""Declared public firing tapes; actual NPC deaths and particles are required.

The recorded OS case is selected by the existing authored244 room identity,
before gameplay. Both phase offsets remain mandatory regardless of outcome.
"""
import copy
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('blood_worm', Path(__file__).with_name('roundworm-scenarios.py'))
worms = importlib.util.module_from_spec(spec)
spec.loader.exec_module(worms)


def capture_steps(tape, count, label):
    for index in range(count):
        tape.extend(worms.step(1, f'{label}-{index + 1:03}'))


def cohort():
    existing = worms.cohort()
    cases = []
    for base in (existing[0], existing[2]):
        for offset in (0, 1):
            case = copy.deepcopy(base)
            name = ('contrived-t1' if not base['scenario']['randomized'] else 'recorded-os') + f'-gibs-{offset}'
            case['scenario'] = {'id': name, 'description': 'Natural244 lethal damage, category Blood Particle5 and full/fast callbacks',
                                'systems': ['enemies-bosses', 'damage-pipeline', 'actor-lifecycle', 'rendering'],
                                'randomized': base['scenario']['randomized']}
            case.pop('wormRequired', None)
            case['bloodParticleRequired'] = ['native-death', 'browser-death', 'native-particles', 'browser-particles']
            for field in ('tape', 'pairedTape'):
                tape = case[field]
                if offset:
                    capture_steps(tape, offset, 'declared-offset')
                # Fixed public cardinal sweeps. No observed target, seed stream,
                # HP, clock value or adaptive aim is supplied to either game.
                for turn, key in enumerate(('Right', 'Up', 'Left', 'Down')):
                    tape.append(worms.key(key, True))
                    capture_steps(tape, 72, f'gib-fire-{turn}')
                    tape.append(worms.key(key, False))
                capture_steps(tape, 96, 'gib-tail')
            cases.append(case)
    return {'schema': 1, 'cases': cases,
            'scope': 'Unchanged mandatory T1 and recorded OS244 route plus independent0/1 phase offset and fixed cardinal firing/tail. All outcomes required; no gameplay grading or reroll.'}


if __name__ == '__main__':
    print(json.dumps(cohort()))
