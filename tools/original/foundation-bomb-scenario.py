#!/usr/bin/env python3
"""Build a reusable original-game bomb input tape without assuming native ticks.

Captures identify inventory, the authored fuse/explosion presentation and health
changes. The selected seed is recorded, and the tape uses only public X11 keys.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('bomb_clock_runner', Path(__file__).with_name('foundation-clock-run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


def build_tape(seed, variant='stationary', bomb_key='e'):
    runner.codec.decode_seed(seed)
    initial = runner.default_tape(seed)
    end = next(i for i, action in enumerate(initial) if action.get('label') == 'gameplay-start')
    tape = [action for action in initial[:end + 1] if action['type'] != 'capture']
    # The original stage-introduction display still covers gameplay-start in
    # existing evidence. Allow its remaining 57 presentation intervals to end.
    tape.extend([{'type': 'step', 'frames': 60}, {'type': 'capture', 'label': 'before-bomb'},
                 {'type': 'key', 'key': bomb_key, 'down': True}])
    if variant == 'escape':
        tape.append({'type': 'key', 'key': 'w', 'down': True})
    previous = 0
    # Dense sampling around expected fuse times is discovery, not a contract.
    checkpoints = sorted(set([1, 2, 3, 6, 12, 20, 30, 40, 50, 60, 70] + list(range(75, 151)) + [180, 210]))
    for frame in checkpoints:
        tape.append({'type': 'step', 'frames': frame - previous})
        tape.append({'type': 'capture', 'label': f'bomb-{frame:03}'})
        if frame == 3:
            tape.append({'type': 'key', 'key': bomb_key, 'down': False})
        if frame == 50 and variant == 'escape':
            tape.append({'type': 'key', 'key': 'w', 'down': False})
        if frame == 12 and variant == 'pause':
            tape.extend([{'type': 'key', 'key': 'Escape', 'down': True}, {'type': 'step', 'frames': 1},
                         {'type': 'key', 'key': 'Escape', 'down': False}, {'type': 'step', 'frames': 120},
                         {'type': 'capture', 'label': 'bomb-paused'},
                         {'type': 'key', 'key': 'Escape', 'down': True}, {'type': 'step', 'frames': 1},
                         {'type': 'key', 'key': 'Escape', 'down': False}])
        previous = frame
    # Empty-inventory press must not create a second explosive.
    tape.extend([{'type': 'key', 'key': bomb_key, 'down': True}, {'type': 'step', 'frames': 3},
                 {'type': 'key', 'key': bomb_key, 'down': False}, {'type': 'capture', 'label': 'bomb-empty'},
                 {'type': 'step', 'frames': 120}, {'type': 'capture', 'label': 'after-empty'}])
    runner.validate_tape(tape)
    return tape


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed', help='Replay a valid seed; omitted uses recorded OS entropy')
    parser.add_argument('--variant', choices=['stationary', 'escape', 'pause'], default='stationary')
    parser.add_argument('--bomb-key', choices=['e', 'Shift_L', 'Shift_R'], default='e')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'bomb-{time.time_ns()}')
    args = parser.parse_args()
    seed = {'code': args.seed, 'seedValue': runner.codec.decode_seed(args.seed)} if args.seed else runner.codec.random_vector()
    tape = build_tape(seed['code'], args.variant, args.bomb_key)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / 'seed.json').write_text(json.dumps(seed, indent=2) + '\n')
    (output / 'tape.json').write_text(json.dumps(tape, indent=2) + '\n')
    (output / 'case.json').write_text(json.dumps({'schema': 1, 'scenario': 'bomb-' + args.variant,
        'systems': ['bombs', 'health-inventory', 'entity-animation', 'input-determinism'],
        'seed': seed, 'bombKey': args.bomb_key, 'source': str(Path(__file__).resolve().relative_to(ROOT)),
        'captureLabels': [action['label'] for action in tape if action['type'] == 'capture'],
        'simulationTicks': False, 'fullInternalState': False}, indent=2) + '\n')
    print(json.dumps({'seed': seed['code'], 'variant': args.variant, 'tape': str(output / 'tape.json'),
        'command': ['bash', str(ROOT / 'tools/original/with-display.sh'), 'python3',
                    str(ROOT / 'tools/original/foundation-clock-run.py'), '--seed', seed['code'],
                    '--tape', str(output / 'tape.json'), '--output', str(output / 'run')]}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
