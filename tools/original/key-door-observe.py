"""Read-only room/door fields and the native ordinary unlock SFX slot."""
import importlib.util
from pathlib import Path


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


traversal = load('key_door_rooms', 'foundation-traversal-observe.py')
audio = load('key_door_audio', 'audio-observer.py')


def observe(pid, context):
    result = traversal.observe(pid, context)
    if result['status'] != 'pass':
        return result
    sound = audio.observe(pid, context)
    if sound['status'] != 'pass':
        return dict(result, status='fail', error=sound.get('error', 'Native audio observation failed'))
    result['doorSound'] = next(slot for slot in sound['slots'] if slot['id'] == 156)
    result['audioSystemCounter'] = sound['systemCounter']
    return result
