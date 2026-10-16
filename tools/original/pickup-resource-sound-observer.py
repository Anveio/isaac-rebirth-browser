"""Actual pickup fields and selected audio windows at one blocked public gate."""
import importlib.util
from pathlib import Path


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


pickups = module('pickup_fixture_fields', 'foundation-pickup-observe.py')
audio = module('pickup_actual_sfx_table', 'audio-observer.py')


def observe(pid, context):
    pickup = pickups.observe(pid, context)
    sound = audio.observe(pid, context) if context.get('pickupSoundWindow') is True else None
    statuses = [pickup.get('status')] + ([sound.get('status')] if sound is not None else [])
    return {'schema': 1, 'status': 'pass' if all(status in ('pass', 'skip') for status in statuses) else 'fail',
            'scope': 'Independent literal pickup and audio observations at the same blocked public gate; no event inference or production input',
            'memoryWritten': False, 'executableMappingsRead': False,
            'pickup': pickup, 'audio': sound}
