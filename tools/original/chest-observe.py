"""Read-only authored room/Chest fields and its real DropSound/open SFX slots."""
import importlib.util
from pathlib import Path


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


rooms = load('chest_room_fields', 'foundation-traversal-observe.py')
audio = load('chest_audio_fields', 'audio-observer.py')


def observe(pid, context):
    result = rooms.observe(pid, context)
    if result['status'] != 'pass':
        return result
    sound = audio.observe(pid, context)
    if sound['status'] != 'pass':
        return dict(result, status='fail', error=sound.get('error', 'Native audio observation failed'))
    result['chestSounds'] = [slot for slot in sound['slots'] if slot['id'] in (21, 22)]
    result['audioSystemCounter'] = sound['systemCounter']
    result['globalSfxVolume'] = sound['globalSfxVolume']
    result['sharedMt'] = {'index': sound['mtIndex'], 'words': sound['mtState']}
    return result
