"""Explicitly scoped Options/audio readers plus the unchanged standard observer.

No Original observation can supply Browser input. The required normal-run
suffix never invokes the Main-family Options reader after menu destruction.
"""
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]

def declaration(context):
    case_path = Path(context['output']).resolve().parent / 'case.json'
    raw = case_path.read_bytes()
    if len(raw) > 256 * 1024:
        raise ValueError('Options declaration exceeds256KiB')
    case = json.loads(raw)
    if case.get('declarationMode') != 'public-options-audio-v1':
        raise ValueError('Missing explicit Options public-input declaration')
    owned = frozenset(case['menuOwnerLabels'])
    expected = frozenset(('title-ready', 'profile-ready', 'main-ready', 'main-options-selected',
        'options-initial-row-0', 'options-sfx-before', 'options-sfx-burst-after', 'options-music-before',
        'options-music-decreased', 'options-music-restored', 'options-music-saved-decreased', 'options-back-main-saved'))
    if owned != expected:
        raise ValueError('Only the twelve declared live menu labels admit Options owner reads')
    unowned = frozenset(case['unownedOptionsLabels'])
    labels = {a['label'] for a in case['tape'] if a['type'] == 'capture'}
    if owned & unowned or owned | unowned != labels:
        raise ValueError('Every capture requires disjoint explicit owner declarations')
    return owned, unowned


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


standard = load('options_standard_paired', ROOT / 'tools/original/foundation-paired-observer.py')
audio = load('options_existing_audio', ROOT / 'tools/original/audio-observer.py')
music = load('options_existing_music', ROOT / 'tools/original/music-observer.py')
options = load('options_menu_owner_v2', HERE / 'options-owner-reader.py')


def observe(pid, context):
    label = context.get('label')
    MENU_LABELS, UNOWNED_LABELS = declaration(context)
    if label not in MENU_LABELS | UNOWNED_LABELS:
        return {'schema': 1, 'status': 'fail', 'error': 'Undeclared capture label',
                'memoryWritten': False, 'executableMappingsRead': False}
    # This standard reader and the clock runner's real final seed/log guard
    # remain mandatory for the provenance suffix, including gameplay-start.
    actual = standard.observe(pid, context)
    result = {'schema': 1, 'status': actual['status'], 'standard': actual,
              'memoryWritten': False, 'executableMappingsRead': False,
              'scope': 'Declared Options/audio fields plus unchanged standard menu/gameplay observer. Run seed is validated by the canonical launcher; no observed state becomes Browser input.'}
    if label in UNOWNED_LABELS:
        result['optionsAudio'] = {
            'status': 'skip', 'label': label,
            'reason': 'Explicit source-only intro' if label == 'intro' else 'Explicit normal-run provenance suffix outside this menu-only Options/audio scope',
            'optionsReaderCalled': False, 'audioReadersCalled': False,
        }
        if label == 'gameplay-start' and actual.get('gameplay', {}).get('status') != 'pass':
            result['status'] = 'fail'
        return result
    menu = options.observe(pid, context)
    sounds = audio.observe(pid, context)
    controller = music.observe(pid, context)
    counters = [menu.get('systemCounter'), sounds.get('systemCounter'),
                controller.get('systemCounter'), actual.get('menu', {}).get('systemCounter')]
    selected = {'status': 'pass' if menu.get('status') == sounds.get('status') == controller.get('status') == 'pass'
                and counters[0] is not None and len(set(counters)) == 1 else 'fail',
                'optionsReaderCalled': True, 'audioReadersCalled': True,
                'menu': menu, 'sfx': sounds, 'music': controller,
                'scope': 'Typed read-only selected menu owners; actual request/IsPlaying/gain/start event evidence is separately required from forwarding OpenAL calls'}
    selected['slot237'] = next((slot for slot in sounds.get('slots', []) if slot['id'] == 237), None)
    if selected['slot237'] is None:
        selected['status'] = 'fail'
    result['optionsAudio'] = selected
    if selected['status'] != 'pass':
        result['status'] = 'fail'
    return result
