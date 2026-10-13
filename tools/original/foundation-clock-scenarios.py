#!/usr/bin/env python3
"""Reusable public input/presentation scenarios; all accept arbitrary valid seeds."""
import argparse
import importlib.util
import json
from pathlib import Path
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('clock_runner', Path(__file__).with_name('foundation-clock-run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

SCENARIOS = [
    {'id': 'menu-movement-fire', 'description': 'Fresh menus and seeded run, movement, shooting and release checkpoints',
     'systems': ['input-determinism', 'menu-navigation', 'locomotion', 'projectile-physics']},
    {'id': 'menu-run-introduction', 'description': 'Preserve the stage banner checkpoint, advance its authored introduction without input, then move and shoot',
     'systems': ['input-determinism', 'menu-navigation', 'run-introduction', 'entity-animation', 'locomotion', 'projectile-physics']},
    {'id': 'motion-trajectory', 'description': 'Every presentation during D30/release30/Right55, with public graphics trajectory measurement',
     'systems': ['locomotion', 'entity-animation', 'projectile-physics', 'projectile-height-range', 'graphics-composition']},
    {'id': 'motion-holdout', 'description': 'Independent direction/length holdout: A40, W20, D30, D+S20 and upward shots',
     'systems': ['locomotion', 'entity-animation', 'projectile-physics', 'projectile-height-range', 'graphics-composition']},
    {'id': 'projectile-inheritance', 'description': 'Compare stationary shots, shots during motion, and shots while coasting after release',
     'systems': ['locomotion', 'projectile-physics', 'projectile-height-range', 'entity-animation']},
]


def key(tape, name, down):
    tape.append({'type': 'key', 'key': name, 'down': down})


def frames(tape, count, individually=False):
    if individually:
        tape.extend({'type': 'step', 'frames': 1} for _ in range(count))
    else:
        tape.append({'type': 'step', 'frames': count})


def capture(tape, label):
    tape.append({'type': 'capture', 'label': label})


def run_prefix(seed):
    tape = runner.default_tape(seed)
    end = next(i for i, action in enumerate(tape) if action.get('label') == 'gameplay-start')
    return [action for action in tape[:end+1] if action['type'] != 'capture' or action['label'] == 'gameplay-start']


def build_scenario(identifier, seed):
    runner.codec.decode_seed(seed)
    if identifier == 'menu-movement-fire':
        return runner.default_tape(seed)
    if identifier == 'menu-run-introduction':
        tape = runner.default_tape(seed)
        title = next(i for i, action in enumerate(tape) if action.get('label') == 'title')
        # Public geometry observes the first Title Idle0 after nine intervals
        # following the intro Return release; preserve all original key phases.
        if tape[title-1] != {'type': 'step', 'frames': 72}:
            raise ValueError('Title bootstrap pacing changed; revalidate the public ready condition')
        tape[title-1:title] = [{'type': 'step', 'frames': 9}, {'type': 'capture', 'label': 'title-ready'},
                             {'type': 'step', 'frames': 63}]
        index = next(i for i, action in enumerate(tape) if action.get('label') == 'gameplay-start')
        tape[index+1:index+1] = [{'type': 'step', 'frames': 57}, {'type': 'capture', 'label': 'gameplay-unpaused'}]
        runner.validate_tape(tape)
        return tape
    tape = run_prefix(seed)
    if identifier == 'motion-trajectory':
        key(tape, 'd', True)
        frames(tape, 30, True)
        key(tape, 'd', False)
        frames(tape, 30, True)
        capture(tape, 'movement')
        key(tape, 'Right', True)
        frames(tape, 55, True)
        key(tape, 'Right', False)
        frames(tape, 1)
        capture(tape, 'shooting-release')
    elif identifier == 'motion-holdout':
        for label, name, count in [('left-40', 'a', 40), ('up-20', 'w', 20), ('right-30', 'd', 30)]:
            key(tape, name, True)
            frames(tape, count, True)
            key(tape, name, False)
            frames(tape, 30, True)
            capture(tape, label)
        key(tape, 'd', True)
        key(tape, 's', True)
        frames(tape, 20, True)
        key(tape, 'd', False)
        key(tape, 's', False)
        frames(tape, 30, True)
        capture(tape, 'diagonal-20')
        key(tape, 'Up', True)
        frames(tape, 50, True)
        key(tape, 'Up', False)
        frames(tape, 30)
        capture(tape, 'up-after-fire')
    elif identifier == 'projectile-inheritance':
        key(tape, 'Right', True)
        frames(tape, 24, True)
        capture(tape, 'stationary-shots')
        key(tape, 'Right', False)
        frames(tape, 90, True)
        key(tape, 'd', True)
        frames(tape, 12, True)
        key(tape, 'Right', True)
        frames(tape, 18, True)
        capture(tape, 'moving-shots')
        key(tape, 'd', False)
        frames(tape, 55, True)
        capture(tape, 'coasting-shots')
        key(tape, 'Right', False)
        frames(tape, 30, True)
        capture(tape, 'after-fire')
    else:
        raise ValueError('Unknown public clock scenario: '+identifier)
    runner.validate_tape(tape)
    return tape


def scenario_metadata():
    # The matrix indexes real executable tapes, rather than a second scenario list.
    seed = runner.codec.random_vector()['code']
    result = []
    for definition in SCENARIOS:
        tape = build_scenario(definition['id'], seed)
        runner.validate_tape(tape)
        result.append({**definition,
            'captureLabels': [action['label'] for action in tape if action['type'] == 'capture'],
            'presentedFrames': sum(action['frames'] for action in tape if action['type'] == 'step'),
            'inputEvents': sum(action['type'] == 'key' for action in tape),
            'source': str(Path(__file__).resolve().relative_to(ROOT))})
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--list', action='store_true', help='Print source scenario metadata for the synchronized test matrix')
    parser.add_argument('--scenario', choices=[s['id'] for s in SCENARIOS], default='menu-movement-fire')
    parser.add_argument('--seed', help='Replay a valid seed; omitted means new OS entropy recorded for replay')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'clock-scenario-{time.time_ns()}')
    args = parser.parse_args()
    if args.list:
        print(json.dumps(scenario_metadata()))
        return 0
    vector = {'code': args.seed, 'seedValue': runner.codec.decode_seed(args.seed)} if args.seed else runner.codec.random_vector()
    tape = build_scenario(args.scenario, vector['code'])
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    (out / 'seed.json').write_text(json.dumps(vector, indent=2)+'\n')
    (out / 'tape.json').write_text(json.dumps(tape, indent=2)+'\n')
    definition = next(s for s in SCENARIOS if s['id'] == args.scenario)
    (out / 'case.json').write_text(json.dumps({'schema': 1, 'scenario': definition, 'seed': vector,
        'captureLabels': [a['label'] for a in tape if a['type'] == 'capture'],
        'presentedFrames': sum(a['frames'] for a in tape if a['type'] == 'step'),
        'simulationTicks': False, 'fullInternalState': False}, indent=2)+'\n')
    print(json.dumps({'scenario': args.scenario, 'seed': vector['code'], 'tape': str(out / 'tape.json'),
        'command': ['bash', str(ROOT / 'tools/original/with-display.sh'), 'python3', str(ROOT / 'tools/original/foundation-clock-run.py'),
                    '--seed', vector['code'], '--tape', str(out / 'tape.json'), '--output', str(out / 'run')]}))
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
