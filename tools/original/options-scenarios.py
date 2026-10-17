#!/usr/bin/env python3
"""Declared public Options audio inputs: one fixed case plus unfiltered fuzz.

Fresh generation draws only a valid run seed, public boot epoch and short held
Down interval. Recorded banks replay without entropy or game/observer reads.
"""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import re
import secrets

ROOT = Path(__file__).resolve().parents[2]
FIXED_PATH = ROOT / 'references/isaac/cases/options-audio/audio-arrows-back-v1.json'
MODE = 'public-options-audio-v1'
BANK_MODE = 'fixed-plus-os-public-input-v1'
INITIAL_SEED = 2184569263
OWNER_LABELS = [
    'title-ready', 'profile-ready', 'main-ready', 'main-options-selected',
    'options-initial-row-0', 'options-sfx-before', 'options-sfx-burst-after',
    'options-music-before', 'options-music-decreased', 'options-music-restored',
    'options-music-saved-decreased', 'options-back-main-saved',
]
KEYS = {'Return', 'Escape', 'Tab', 'Up', 'Down', 'Left', 'Right'}
DRAW_BOUNDS = [('runSeed', 0xffffffff), ('bootEpochSeed', 0x100000000),
               ('musicNavigationHoldFrames', 49)]
CASE_FIELDS = {'schema', 'seed', 'seedValue', 'bootEpochSeed', 'initialSeed',
               'bootstrap', 'tape', 'pairedTape', 'declaredAudioOptions',
               'declaredFullscreen', 'scenario', 'publicClockDeclaration', 'id',
               'declarationMode', 'menuOwnerLabels', 'unownedOptionsLabels',
               'parameters', 'entropyDraws'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def integer(value, low, high):
    return type(value) is int and low <= value <= high


def seed_codec():
    # Load the canonical public codec lazily. Importing this generator does not
    # draw inputs, read a case, launch a process or initialize a game.
    path = ROOT / 'tools/original/foundation-seeds.py'
    spec = importlib.util.spec_from_file_location('options_public_seed_codec', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def validate_tape(tape):
    require(isinstance(tape, list) and 0 < len(tape) <= 36000,
            'Options tape must contain 1..36000 public actions')
    held, labels, total = set(), [], 0
    for action in tape:
        require(isinstance(action, dict), 'Options action must be an object')
        kind = action.get('type')
        if kind == 'step':
            require(set(action) == {'type', 'frames'} and integer(action['frames'], 1, 36000),
                    'Options step requires integer frames within1..36000')
            total += action['frames']
        elif kind == 'capture':
            label = action.get('label')
            require(set(action) == {'type', 'label'} and isinstance(label, str)
                    and re.fullmatch(r'[A-Za-z0-9_-]{1,80}', label) and label not in labels,
                    'Options capture labels must be unique public names')
            labels.append(label)
        elif kind == 'key':
            key = action.get('key')
            require(set(action) == {'type', 'key', 'down'} and isinstance(key, str)
                    and (key in KEYS or re.fullmatch(r'[a-z0-9]', key))
                    and type(action['down']) is bool,
                    'Options key requires an admitted X11 name and boolean down')
            require((key not in held) if action['down'] else (key in held),
                    'Options tape has a repeated keydown or an unowned keyup')
            if action['down']:
                held.add(key)
            else:
                held.remove(key)
        else:
            raise ValueError('Unsupported Options public action: ' + str(kind))
    require(total <= 36000 and not held, 'Options tape exceeds its clock or leaves a key held')
    return labels


def mutation_sites(paired):
    """Only the short row1-to-row2 hold and eight seed-text pairs may vary."""
    captures = {action['label']: index for index, action in enumerate(paired)
                if action['type'] == 'capture'}
    music = captures['options-music-before']
    require(paired[music-4:music] == [
        {'type': 'key', 'key': 'Down', 'down': True}, {'type': 'step', 'frames': 2},
        {'type': 'key', 'key': 'Down', 'down': False}, {'type': 'step', 'frames': 2},
    ], 'Fixed Options music navigation must retain its canonical public shape')
    text = captures['provenance-seed-entry'] - 32
    require(paired[text-4:text] == [
        {'type': 'key', 'key': 'Tab', 'down': True}, {'type': 'step', 'frames': 12},
        {'type': 'key', 'key': 'Tab', 'down': False}, {'type': 'step', 'frames': 108},
    ], 'Fixed Options seed suffix must follow canonical Tab entry')
    for index in range(8):
        down, wait, up, release = paired[text+4*index:text+4*index+4]
        require(down['type'] == up['type'] == 'key' and down['down'] is True
                and up['down'] is False and down['key'] == up['key']
                and re.fullmatch(r'[a-z0-9]', down['key'])
                and wait == {'type': 'step', 'frames': 12}
                and release == {'type': 'step', 'frames': 6},
                'Fixed Options seed suffix must retain all eight physical key pairs')
    return music-3, text


def fixed_case(codec):
    case = json.loads(FIXED_PATH.read_text())
    require(set(case) == CASE_FIELDS and integer(case.get('schema'), 1, 1)
            and case.get('id') == 'audio-arrows-back-v1'
            and case.get('declarationMode') == MODE, 'Invalid fixed Options declaration')
    require(case['initialSeed'] == INITIAL_SEED and integer(case['bootEpochSeed'], 0, 0xffffffff)
            and codec.decode_seed(case['seed']) == case['seedValue'],
            'Fixed Options public seeds are invalid')
    require(case['declaredAudioOptions'] == {'sfxVolume': 1, 'musicVolume': 1}
            and case['declaredFullscreen'] is False, 'Options audio inputs must stay independently declared')
    require(case['menuOwnerLabels'] == OWNER_LABELS, 'Fixed Options menu owners differ')
    labels = validate_tape(case['tape'])
    validate_tape(case['pairedTape'])
    require(case['tape'] == case['bootstrap']['sourceOnlyPrelude'] + case['pairedTape'],
            'Full Options tape must exactly retain source-only prelude plus paired tape')
    require(case['pairedTape'][0] == {'type': 'capture', 'label': case['bootstrap']['boundaryLabel']},
            'Options paired boundary differs from its declaration')
    require(case['unownedOptionsLabels'] == [label for label in labels if label not in OWNER_LABELS]
            and all(label in labels for label in OWNER_LABELS), 'Options owner partition is incomplete')
    require(case['parameters'] == {'musicNavigationHoldFrames': 2} and case['entropyDraws'] == []
            and case['scenario']['randomized'] is False, 'Fixed Options case must draw no entropy')
    mutation_sites(case['pairedTape'])
    return case


def project_case(fixed, index, draws, codec):
    require(isinstance(draws, list) and len(draws) == 3, 'Every fuzz case requires all three OS draws')
    samples = {}
    for row, (parameter, bound) in zip(draws, DRAW_BOUNDS):
        require(isinstance(row, dict) and set(row) == {'parameter', 'exclusiveUpperBound', 'sample'}
                and row['parameter'] == parameter and row['exclusiveUpperBound'] == bound
                and integer(row['sample'], 0, bound-1), 'Invalid recorded Options entropy draw')
        samples[parameter] = row['sample']
    run_seed = samples['runSeed'] + 1
    code = codec.encode_seed(run_seed)
    case = copy.deepcopy(fixed)
    case['id'] = 'options-audio-random-' + str(index)
    case['seed'] = code[:4] + ' ' + code[4:]
    case['seedValue'] = run_seed
    case['bootEpochSeed'] = samples['bootEpochSeed']
    case['parameters'] = {'musicNavigationHoldFrames': samples['musicNavigationHoldFrames'] + 1}
    case['entropyDraws'] = copy.deepcopy(draws)
    case['scenario'].update({
        'id': case['id'], 'randomized': True,
        'description': 'Unfiltered public run seed/epoch and short held Options navigation; fixed audio burst and Back route.',
    })
    hold, text = mutation_sites(case['pairedTape'])
    case['pairedTape'][hold]['frames'] = case['parameters']['musicNavigationHoldFrames']
    for offset, character in enumerate(code.lower()):
        case['pairedTape'][text+4*offset]['key'] = character
        case['pairedTape'][text+4*offset+2]['key'] = character
    case['tape'] = copy.deepcopy(case['bootstrap']['sourceOnlyPrelude']) + copy.deepcopy(case['pairedTape'])
    return case


def validate_bank(bank, fixed, codec):
    fields = {'schema', 'suite', 'declarationMode', 'randomCount', 'entropy', 'cases'}
    require(isinstance(bank, dict) and set(bank) in (fields, fields | {'caseFilter'}),
        'Options bank requires exactly its public declaration fields')
    require(integer(bank['schema'], 1, 1) and bank['suite'] == 'options-audio'
            and bank['declarationMode'] == BANK_MODE
            and bank['entropy'] == 'OS secrets.randbelow; all logical draws retained; no outcome selection'
            and integer(bank['randomCount'], 1, 16), 'Invalid Options bank mode/count')
    cases = bank['cases']
    filtered = 'caseFilter' in bank
    if filtered:
        require(isinstance(bank['caseFilter'], str) and 0 < len(bank['caseFilter']) <= 4096
                and isinstance(cases, list) and 0 < len(cases) <= bank['randomCount'] + 1,
                'Filtered Options bank requires its nonempty declared case filter and cases')
    else:
        require(isinstance(cases, list) and len(cases) == bank['randomCount'] + 1
                and cases[0] == fixed, 'Options bank requires its unchanged fixed case plus every OS case')
    identifiers = set()
    for position, case in enumerate(cases):
        require(isinstance(case, dict) and set(case) == CASE_FIELDS
                and isinstance(case.get('id'), str)
                and case['id'] not in identifiers, 'Options case IDs must be unique')
        identifiers.add(case['id'])
        require(integer(case['schema'], 1, 1) and case['declarationMode'] == MODE
                and integer(case['seedValue'], 1, 0xffffffff)
                and integer(case['bootEpochSeed'], 0, 0xffffffff)
                and integer(case['initialSeed'], INITIAL_SEED, INITIAL_SEED)
                and isinstance(case['parameters'], dict)
                and set(case['parameters']) == {'musicNavigationHoldFrames'}
                and integer(case['parameters']['musicNavigationHoldFrames'], 1, 49)
                and isinstance(case['declaredAudioOptions'], dict)
                and set(case['declaredAudioOptions']) == {'sfxVolume', 'musicVolume'}
                and all(type(value) in (int, float) and value == 1
                        for value in case['declaredAudioOptions'].values())
                and case['declaredFullscreen'] is False
                and type(case['scenario']['randomized']) is bool,
                'Options case has invalid public seeds/audio/hold or declaration mode')
        labels = validate_tape(case['tape'])
        validate_tape(case['pairedTape'])
        require(case['menuOwnerLabels'] == OWNER_LABELS
                and case['unownedOptionsLabels'] == [label for label in labels if label not in OWNER_LABELS]
                and case['initialSeed'] == INITIAL_SEED
                and case['tape'] == case['bootstrap']['sourceOnlyPrelude'] + case['pairedTape'],
                'Options case changes its actual input/owner declaration')
        if case['id'] == fixed['id']:
            require(case == fixed, 'Fixed Options input case changed')
        else:
            identifier = re.fullmatch(r'options-audio-random-(0|[1-9][0-9]*)', case['id'])
            require(identifier is not None, 'Invalid Options fuzz case ID')
            index = int(identifier.group(1))
            require(index < bank['randomCount'] and (filtered or index == position-1),
                    'Options fuzz case ID differs from the declared bank ordinal')
            require(case == project_case(fixed, index, case.get('entropyDraws'), codec),
                    'Options fuzz case differs from its recorded public parameter projection')
    if filtered:
        patterns = bank['caseFilter'].split(',')
        matches = []
        for pattern in patterns:
            test = re.compile('^' + re.escape(pattern).replace(r'\*', '.*') + '$')
            require(pattern and any(test.fullmatch(name) for name in identifiers),
                    'Every recorded case filter pattern must match a retained case')
            matches.append(test)
        require(all(any(test.fullmatch(name) for test in matches) for name in identifiers),
                'Every retained case must belong to its recorded filter')
    return bank


def generate_bank(count, fixed, codec):
    require(integer(count, 1, 16), 'Options random count must be1..16')
    cases = [copy.deepcopy(fixed)]
    for index in range(count):
        # Each application-level OS draw is retained before projection. There
        # is no floor, native result, unsupported outcome filter or retry loop.
        draws = [{'parameter': parameter, 'exclusiveUpperBound': bound,
                  'sample': secrets.randbelow(bound)} for parameter, bound in DRAW_BOUNDS]
        cases.append(project_case(fixed, index, draws, codec))
    return validate_bank({'schema': 1, 'suite': 'options-audio', 'declarationMode': BANK_MODE,
                          'randomCount': count, 'entropy': 'OS secrets.randbelow; all logical draws retained; no outcome selection',
                          'cases': cases}, fixed, codec)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, help='Fresh OS case count1..16; default1, plus fixed case')
    parser.add_argument('--scenarios', type=Path, help='Replay an exact recorded bank; draws no entropy')
    args = parser.parse_args()
    require(not (args.scenarios and args.count is not None), 'Replay --scenarios cannot regenerate --count cases')
    codec = seed_codec()
    fixed = fixed_case(codec)
    if args.scenarios:
        bank = validate_bank(json.loads(args.scenarios.read_text()), fixed, codec)
    else:
        bank = generate_bank(1 if args.count is None else args.count, fixed, codec)
    print(json.dumps(bank, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
