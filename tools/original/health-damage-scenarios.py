#!/usr/bin/env python3
"""Predeclared public acquisition/damage routes and unfiltered OS seeds."""
import argparse
import importlib.util
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('health_paired_scenarios', ROOT / 'tools/original/foundation-paired-scenarios.py')
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)


def key(name, down): return {'type': 'key', 'key': name, 'down': down}


def advance(count, prefix, stride=1):
    result = []
    for at in range(0, count, stride):
        step = min(stride, count - at)
        result += [{'type': 'step', 'frames': step}, {'type': 'capture', 'label': f'{prefix}-{at + step}'}]
    return result


def route(family, phase):
    result = advance(phase, 'phase')
    if family == 'soul':
        # Retained source-qualified item72 fixture; the original must actually
        # select/acquire it again. Public movement, then the initial own bomb.
        result += [key('w', True)] + advance(74, 'treasure-approach', 8) + [key('w', False)]
        result += advance(60, 'treasure-entry', 10)
        result += [key('w', True)] + advance(32, 'treasure-contact', 4) + [key('w', False)]
        result += advance(100, 'acquisition', 10) + [{'type': 'capture', 'label': 'health-acquired'}]
        result += [key('e', True)] + advance(1, 'bomb-birth') + [key('e', False)]
        result += advance(105, 'bomb-fuse', 7) + advance(48, 'bomb-damage')
    else:
        # Source-only Floor69 predicts authored Heart6 in Cellar Room213.
        # All unsupported room actors stay in the required paired comparison.
        result += [key('s', True)] + advance(17, 'depart', 4) + [key('s', False)]
        result += advance(101, 'entry', 10)
        result += [key('s', True)] + advance(35, 'black-contact') + [key('s', False)]
        result += advance(45, 'collection', 5) + [{'type': 'capture', 'label': 'health-acquired'}]
        result += [key('w', True)] + advance(10, 'spike-contact') + [key('w', False)]
        result += advance(180, 'black-damage')
    return result + [{'type': 'capture', 'label': 'health-settled'}]


def build(identifier, seed, family, phase, required):
    case = paired.build('paired-room-combat', seed)
    case['scenario'] = {'id': identifier, 'description': f'Public {family} acquisition route followed by ordinary admitted damage',
        'systems': ['health-inventory', 'damage-pipeline', 'player-death', 'information-animation', 'audio', 'input-clock'],
        'randomized': not required}
    for name in ('tape', 'pairedTape'):
        at = next(i for i, action in enumerate(case[name]) if action.get('label') == 'gameplay-start')
        case[name] = case[name][:at + 1] + route(family, phase)
    case['healthRequired'] = family if required else None
    case['healthRoute'] = family
    case['phaseOffset'] = phase
    case['healthScope'] = 'Both contrived phase offsets require actual acquisition and damage in original and browser. Fresh OS seeds are recorded before either game, without filtering or reroll. All checkpoints, failures and public intervals retained; no native state supplies the browser.'
    case['capturePolicy'] = 'Sparse declared approach/fuse captures; every presentation in actual damage/Hit/Book windows. Policy fixed before grading; no failed snapshots removed.'
    paired.runner.validate_tape(case['tape'])
    return case


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--count', type=int, default=2)
    args = parser.parse_args()
    if not 2 <= args.count <= 4: raise ValueError('Required OS count must be2..4')
    # Draw the entire unfiltered cohort once, before fixture construction or
    # any original/port grading. Its cardinality and values remain explicit.
    vectors = [paired.runner.codec.random_vector() for _ in range(args.count)]
    cases = [build(f'health-{family}-phase{phase}', seed, family, phase, True)
             for family, seed in [('soul', 'B91XHWCZ'), ('black', 'B9119QA3')] for phase in (0, 1)]
    cases += [build(f'health-unfiltered-{n}', vector['code'], 'soul' if n % 2 == 0 else 'black', n % 2, False)
              for n, vector in enumerate(vectors)]
    print(json.dumps({'schema': 1, 'entropy': 'OS secrets; unfiltered recorded once before grading', 'randomVectors': vectors, 'cases': cases}))


if __name__ == '__main__': main()
