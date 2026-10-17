"""Existing bounded observers composed for death Exit Music and SFX8 evidence.

Snapshots establish controller/slot state, not event multiplicity or audibility.
Use existing forwarding audio-trace.c for actual Native backend call evidence.
"""
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'tools/original' / filename)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


death = load('exit_existing_death', 'player-death-observe.py')
music = load('exit_existing_music', 'music-observer.py')


def observe(pid, context):
    result = death.observe(pid, context)
    controller = music.observe(pid, context)
    sounds = death.audio.observe(pid, context)
    counters = [controller.get('systemCounter'), sounds.get('systemCounter')]
    menu_counter = result.get('menu', {}).get('systemCounter')
    if menu_counter is not None:
        counters.append(menu_counter)
    actual = {'status': 'pass' if controller.get('status') == sounds.get('status') == 'pass'
              and len(set(counters)) == 1 and counters[0] is not None else 'fail',
              'music': controller, 'sfx': {key: sounds.get(key) for key in
                  ('status', 'systemCounter', 'cooldownTick', 'globalSfxVolume', 'memoryWritten')},
              'slots': [slot for slot in sounds.get('slots', []) if slot['id'] in (8, 30)],
              'scope': 'Existing bounded read-only Music and SFX8/30 snapshots; no inferred request counts or audible parity'}
    if {slot['id'] for slot in actual['slots']} != {8, 30}:
        actual['status'] = 'fail'
    result['exitAudio'] = actual
    if actual['status'] != 'pass':
        result['status'] = 'fail'
    return result
