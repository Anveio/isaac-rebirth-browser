#!/usr/bin/env python3
"""Public-input checkpoints for authored stage and pause presentation."""
import importlib.util
from pathlib import Path

_spec = importlib.util.spec_from_file_location('presentation_runner', Path(__file__).with_name('foundation-clock-run.py'))
_runner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_runner)

SCENARIO = {
    'id': 'run-pause-presentation',
    'description': 'Stage banner entrance and exit, pause paper animation, run seed, selection and resume',
    'systems': ['run-introduction', 'menu-navigation', 'menu-animation', 'entity-animation', 'input-determinism'],
}


def build_tape(seed):
    tape = _runner.default_tape(seed)
    end = next(index for index, action in enumerate(tape) if action.get('label') == 'gameplay-start')
    tape = tape[:end + 1]
    # Retain the original 300-interval run-entry pacing while exposing its
    # loading/fade/banner phases instead of observing only the settled streak.
    if tape[-2] != {'type': 'step', 'frames': 300}:
        raise ValueError('Run-entry pacing changed; revalidate presentation checkpoints')
    entry = []
    previous = 0
    for frame in [64, 192, 224, 240]:
        entry.extend([{'type': 'step', 'frames': frame - previous},
                      {'type': 'capture', 'label': f'run-entry-{frame}'}])
        previous = frame
    entry.append({'type': 'step', 'frames': 300 - previous})
    tape[-2:-1] = entry

    def step(frames):
        tape.append({'type': 'step', 'frames': frames})

    def capture(label):
        tape.append({'type': 'capture', 'label': label})

    def key(name, down):
        tape.append({'type': 'key', 'key': name, 'down': down})

    previous = 0
    for frame in [1, 8, 24, 40, 56, 57]:
        step(frame - previous)
        capture(f'introduction-{frame}')
        previous = frame
    key('Escape', True)
    step(1)
    key('Escape', False)
    capture('pause-1')
    previous = 1
    for frame in [4, 8, 16, 26, 40]:
        step(frame - previous)
        capture(f'pause-{frame}')
        previous = frame
    key('Down', True)
    step(1)
    key('Down', False)
    step(4)
    capture('pause-selection')
    key('Escape', True)
    step(1)
    key('Escape', False)
    capture('resume-1')
    step(21)
    capture('resume-22')
    _runner.validate_tape(tape)
    return tape
