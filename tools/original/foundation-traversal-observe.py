"""Pair floor descriptors with current room contents during public doorway tapes."""
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value

floor = module('traversal_floor', 'foundation-layout-observer.py')
contents = module('traversal_contents', 'foundation-pickup-observe.py')


def observe(pid, context):
    if context.get('publicLogState', {}).get('stage') != 1:
        return {'status': 'skip', 'scope': 'traversal gameplay-only observer'}
    label = context['label']
    routed = dict(context, label='gameplay-start' if label == 'gameplay-start' else 'topology')
    layout = floor.observe(pid, routed)
    actors = contents.observe(pid, context)
    return {'schema': 1, 'status': 'pass' if layout.get('status') == actors.get('status') == 'pass' else 'fail',
            'checkpoint': label, 'memoryWritten': False, 'floor': layout, 'contents': actors}
