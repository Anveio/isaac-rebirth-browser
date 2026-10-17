"""Canonical paired reader plus existing bounded carried-item evidence.

Only the two public checkpoints needed by the declared carried-item contract
request the existing item reader. Its raw status is retained separately;
ordinary control cases retain their canonical paired expectations. A case
that declares carriedActiveItemId must require this reader to pass in the
stage-exit comparator, so missing/failed observations cannot satisfy it.
No new offsets, data writes or input values enter either game.
"""
import importlib.util
from pathlib import Path


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


paired = load('stage_exit_canonical_paired', 'foundation-paired-observer.py')
items = load('stage_exit_canonical_items', 'item-observer.py')


def observe(pid, context):
    result = paired.observe(pid, context)
    if context.get('label') in ('re-anchor', 'next-stage') and result.get('gameplay', {}).get('status') == 'pass':
        #item-observer retains its direct-child/public-gate/binary pin,
        #2MiB writable-data budget, actual Player class and finally-resume.
        #The separate pickup-world attachment is stage1-only. Request the
        #actual core item owners directly on stage1 ANDstage2; never fake
        #publicLogState.stage or waive a failed actual item observation.
        result['carriedItems'] = items.observe(pid, dict(context, includePickupWorld=False))
    return result
