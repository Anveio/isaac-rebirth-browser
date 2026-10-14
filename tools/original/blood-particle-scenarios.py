#!/usr/bin/env python3
"""Declared public movement/firing; actual deaths and particles are required.

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


def initial_firing_cohort():
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


def cohort():
    bank = initial_firing_cohort()
    for case in bank['cases']:
        original_id = case['scenario']['id']
        case['scenario']['id'] = original_id.replace('-gibs-', '-moving-gibs-')
        case['scenario']['description'] = 'Natural244 death/5 after declared boundary movement and cardinal firing'
        randomized = case['scenario']['randomized']
        # Mirror only the previously declared public entry direction. No
        # observed actor position, health, target, RNG or clock is read here.
        pattern = [('w', 'Left', 160), ('a', 'Down', 240), ('s', 'Right', 160), ('d', 'Up', 240)] if randomized else [
            ('w', 'Right', 160), ('d', 'Down', 240), ('s', 'Left', 160), ('a', 'Up', 240)]
        for field in ('tape', 'pairedTape'):
            tape = case[field]
            elapsed = 0
            for lap in range(2):
                for movement, shot, duration in pattern:
                    tape.extend([worms.key(movement, True), worms.key(shot, True)])
                    remaining = duration
                    while remaining:
                        count = min(remaining, 25 - elapsed % 25)
                        tape.extend(worms.step(count))
                        elapsed += count
                        remaining -= count
                        if elapsed % 25 == 0:
                            tape.append({'type': 'capture', 'label': f'moving-{elapsed:04}'})
                    tape.extend([worms.key(movement, False), worms.key(shot, False)])
            capture_steps(tape, 32, 'moving-tail')
        case['movingDeclaration'] = {
            'initialCaseId': original_id,
            'initialSource': '1e9d02f6954e4296a9864ee5f0b9b61d1225c100',
            'initialReportSha256': 'd7814b5a8195e654d8116a1967e474f6c9caf4f9d2606ba360e2baf96975f2ae',
            'scope': 'Every prior public action/capture remains an exact prefix; two fixed movement/firing laps are appended. Both original phase offsets and recorded OS identities remain required.',
            'pattern': pattern, 'laps': 2,
            'sampling': 'Odd25-presentation samples alternate full/fast during movement; every presentation of32-tail is captured. Each admitted snapshot is complete.',
        }
    bank['scope'] = 'All four original firing tapes retained as exact prefixes, followed by declared movement/cardinal firing. Same unfiltered contrived/recorded identities and phase offsets; no original state/target/clock input, gameplay grading or reroll.'
    return bank


if __name__ == '__main__':
    print(json.dumps(cohort()))
