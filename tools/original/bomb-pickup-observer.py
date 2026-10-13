"""Read-only pickup/player fields plus the actual native bomb-pickup SFX slot."""
import importlib.util
from pathlib import Path


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


pickups = load('bomb_pickup_fields', 'foundation-pickup-observe.py')
audio = load('bomb_pickup_audio_fields', 'audio-observer.py')


def observe(pid, context):
    result = pickups.observe(pid, context)
    if result['status'] != 'pass':
        return result
    sound = audio.observe(pid, context)
    if sound['status'] != 'pass':
        return dict(result, status='fail', error=sound.get('error', 'Native audio observation failed'))
    result['bombSound'] = next(slot for slot in sound['slots'] if slot['id'] == 201)
    result['audioSystemCounter'] = sound['systemCounter']
    return result
