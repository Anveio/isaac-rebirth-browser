#!/usr/bin/env python3
"""Public title checkpoints across fade, animation and the first authored loop."""
import importlib.util
from pathlib import Path

_spec = importlib.util.spec_from_file_location('title_runner', Path(__file__).with_name('foundation-clock-run.py'))
_runner = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_runner)

SCENARIO = {
    'id': 'title-animation-fade',
    'description': 'Observe first title, alternate actor updates, global white fade, loop and retained animation after menu reentry',
    'systems': ['menu-navigation', 'menu-animation', 'entity-animation', 'graphics-composition', 'input-determinism'],
}


def build_tape(seed):
    tape = _runner.default_tape(seed)
    title = next(index for index, action in enumerate(tape) if action.get('label') == 'title')
    if tape[title - 1] != {'type': 'step', 'frames': 72}:
        raise ValueError('Title bootstrap changed; revalidate the original ready condition')
    # Controlled source observations locate actor frame0 nine presentations
    # after intro Return release. Later phases follow public intervals only.
    steps = []
    previous = 0
    for interval, label in [(9, 'title-first'), (10, 'title-first-update'),
                            (32, 'title-fading'), (48, 'title-fade-last'),
                            (72, 'title-settled'), (90, 'title-loop')]:
        steps.extend([{'type': 'step', 'frames': interval - previous},
                      {'type': 'capture', 'label': label}])
        previous = interval
    # Title Reset is a no-op. The public return path independently observes
    # its offscreen clock freezing and then continuing through another loop.
    steps.extend([
        {'type': 'key', 'key': 'Return', 'down': True}, {'type': 'step', 'frames': 1},
        {'type': 'capture', 'label': 'title-leave-1'}, {'type': 'step', 'frames': 11},
        {'type': 'key', 'key': 'Return', 'down': False}, {'type': 'step', 'frames': 72},
        {'type': 'capture', 'label': 'title-profile'},
        {'type': 'key', 'key': 'Escape', 'down': True}, {'type': 'step', 'frames': 1},
        {'type': 'capture', 'label': 'title-return-1'}, {'type': 'step', 'frames': 11},
        {'type': 'key', 'key': 'Escape', 'down': False}, {'type': 'step', 'frames': 72},
        {'type': 'capture', 'label': 'title-return-settled'},
    ])
    tape[title - 1:title + 1] = steps
    _runner.validate_tape(tape)
    return tape
