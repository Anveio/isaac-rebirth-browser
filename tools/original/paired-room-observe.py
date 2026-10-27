"""Canonical paired reader plus floor, room contents and owned collectibles.

Every capture keeps the canonical paired observation (menu, gameplay, pause).
Gameplay checkpoints add the traversal reader's floor descriptors and room
contents with the Player+E1C collectible counts; the tapes capture nothing
while the pause or main menu owns the screen. Reads are bounded and write no
memory.
"""
import importlib.util
from pathlib import Path


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


paired = module('paired_room_canonical', 'foundation-paired-observer.py')
traversal = module('paired_room_traversal', 'foundation-traversal-observe.py')


def observe(pid, context):
    result = paired.observe(pid, context)
    rooms = traversal.observe(pid, dict(context, collectibleCounts=True))
    if rooms.get('status') == 'skip':
        return result
    result['floor'], result['contents'] = rooms.get('floor'), rooms.get('contents')
    if rooms.get('status') != 'pass':
        result['status'] = 'fail'
        result['error'] = rooms.get('error', 'Native floor/contents observation failed')
    return result
