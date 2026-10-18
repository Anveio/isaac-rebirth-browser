#!/usr/bin/env python3
"""Input-only Map bank: one fixed route plus unfiltered OS run/boot seeds.

Recorded replay and validation do not import entropy modules. Each random case
retains the fixed route, holds, owner declarations and capture partitions.
"""
import argparse
import copy
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
FIXED = ROOT / 'references/isaac/cases/options-map/map-adjust-back-surface-fast31-v2.json'
MODE = 'fixed-plus-unfiltered-map-seed-boot-v1'
ENTROPY = 'OS secrets.randbelow; all logical draws retained; no outcome selection'
INITIAL_SEED = 2184569263
DRAW_BOUNDS = [('runSeed', 0xffffffff), ('bootEpochSeed', 0x100000000)]

# Pure public codec math from tools/original/foundation-seeds.py. Loading that
# module also imports secrets, so replay uses these same pure functions locally.
ALPHABET = 'ABCDEFGHJKLMNPQRSTWXYZ01234V6789'
SEED_XOR = 0x0FEF7FFD


def require(condition, message):
    if not condition:
        raise ValueError(message)


def integer(value, low, high):
    return type(value) is int and low <= value <= high


def same_json(left, right):
    # Python equality otherwise admits True in place of the integer 1.
    return json.dumps(left, sort_keys=True, separators=(',', ':'), allow_nan=False) == \
        json.dumps(right, sort_keys=True, separators=(',', ':'), allow_nan=False)


def checksum(seed):
    value = 0
    while True:
        value = (value + (seed & 255)) & 255
        value = ((value << 1) + (value >> 7)) & 255
        seed >>= 5
        if seed == 0:
            return value


def encode_seed(seed):
    require(integer(seed, 1, 0xffffffff), 'Run seed must be a nonzero u32')
    packed = ((seed ^ SEED_XOR) << 8) | checksum(seed)
    return ''.join(ALPHABET[(packed >> shift) & 31] for shift in range(35, -1, -5))


def decode_seed(code):
    compact = ''.join(code.split()).upper() if isinstance(code, str) else ''
    require(len(compact) == 8 and all(character in ALPHABET for character in compact),
            'Seed must have eight original-alphabet characters')
    packed = 0
    for character in compact:
        packed = (packed << 5) | ALPHABET.index(character)
    seed = (packed >> 8) ^ SEED_XOR
    require(seed != 0 and packed & 255 == checksum(seed), 'Invalid public seed checksum')
    return seed


def read_json(path):
    require(path.stat().st_size <= 4 * 1024 * 1024, 'Input JSON exceeds its 4 MiB bound')
    return json.loads(path.read_text())


def seed_text_start(tape, code):
    captures = [index for index, action in enumerate(tape)
                if action == {'type': 'capture', 'label': 'provenance-seed-entry'}]
    require(len(captures) == 1, 'Expected one public seed-entry capture')
    start = captures[0] - 32
    require(start >= 4 and tape[start-4:start] == [
        {'type': 'key', 'key': 'Tab', 'down': True}, {'type': 'step', 'frames': 12},
        {'type': 'key', 'key': 'Tab', 'down': False}, {'type': 'step', 'frames': 108},
    ], 'Seed entry must follow the fixed public Tab route')
    for index, character in enumerate(''.join(code.split()).lower()):
        require(tape[start+4*index:start+4*index+4] == [
            {'type': 'key', 'key': character, 'down': True}, {'type': 'step', 'frames': 12},
            {'type': 'key', 'key': character, 'down': False}, {'type': 'step', 'frames': 6},
        ], 'Seed entry must retain all eight physical pairs and their fixed holds')
    return start


def fixed_case():
    fixed = read_json(FIXED)
    require(isinstance(fixed, dict) and integer(fixed.get('schema'), 1, 1)
            and fixed.get('declarationMode') == 'map-opacity-fast31-source-v1',
            'Unknown fixed Map declaration')
    require(integer(fixed.get('seedValue'), 1, 0xffffffff)
            and decode_seed(fixed.get('seed')) == fixed['seedValue']
            and integer(fixed.get('bootEpochSeed'), 0, 0xffffffff)
            and integer(fixed.get('initialSeed'), INITIAL_SEED, INITIAL_SEED),
            'Fixed public seed/boot/initial declarations are invalid')
    require(fixed['scenario']['id'] == 'options-map-opacity-adjust-back-surface-fast31-v2'
            and fixed['scenario']['randomized'] is False,
            'The fixed Map route must retain its public identity')
    paired, full = fixed['pairedTape'], fixed['tape']
    labels = lambda tape: [action['label'] for action in tape if action['type'] == 'capture']
    declaration = fixed['captureDeclaration']
    require(full == fixed['bootstrap']['sourceOnlyPrelude'] + paired
            and paired[0] == {'type': 'capture', 'label': fixed['bootstrap']['boundaryLabel']}
            and labels(full) == declaration['nativeLabels']
            and labels(paired) == declaration['pairedLabels']
            and len(labels(full)) == len(set(labels(full))) == 31
            and len(labels(paired)) == len(set(labels(paired))) == 30
            and declaration['sourceOnlyLabels'] == ['intro'],
            'Map native/paired capture partition or full tape changed')
    require(sum(action['type'] == 'key' for action in full) == 62
            and sum(action['type'] == 'key' for action in paired) == 60
            and sum(action['frames'] for action in paired if action['type'] == 'step') == 1368,
            'Map physical edges or presented holds changed')
    seed_text_start(paired, fixed['seed'])
    return fixed


def project_case(fixed, index, draws):
    require(isinstance(draws, list) and len(draws) == 2,
            'Each OS case must retain both independent logical draws')
    samples = {}
    for row, (parameter, bound) in zip(draws, DRAW_BOUNDS):
        require(isinstance(row, dict) and set(row) == {'parameter', 'exclusiveUpperBound', 'sample'}
                and row['parameter'] == parameter and type(row['exclusiveUpperBound']) is int
                and row['exclusiveUpperBound'] == bound and integer(row['sample'], 0, bound-1),
                'Invalid recorded Map entropy draw')
        samples[parameter] = row['sample']
    seed = samples['runSeed'] + 1
    code = encode_seed(seed)
    data = copy.deepcopy(fixed)
    data.update(seed=code[:4] + ' ' + code[4:], seedValue=seed,
                bootEpochSeed=samples['bootEpochSeed'])
    # initialSeed deliberately stays the fixed pre-menu initial state.
    data['scenario'].update(id='options-map-unfiltered-seed-boot-' + str(index), randomized=True)
    data['entropyDraws'] = copy.deepcopy(draws)
    start = seed_text_start(data['pairedTape'], fixed['seed'])
    for index, character in enumerate(code.lower()):
        data['pairedTape'][start+4*index]['key'] = character
        data['pairedTape'][start+4*index+2]['key'] = character
    data['tape'] = copy.deepcopy(data['bootstrap']['sourceOnlyPrelude'] + data['pairedTape'])
    return data


def validate_bank(bank, fixed):
    fields = {'schema', 'suite', 'declarationMode', 'randomCount', 'entropy', 'cases'}
    require(isinstance(bank, dict) and set(bank) in (fields, fields | {'caseFilter'}),
            'Map bank must declare exactly its public protocol fields')
    require(integer(bank['schema'], 1, 1) and bank['suite'] == 'options-map'
            and bank['declarationMode'] == MODE and bank['entropy'] == ENTROPY
            and integer(bank['randomCount'], 1, 16), 'Invalid Map bank mode/count')
    cases = bank['cases']
    filtered = 'caseFilter' in bank
    require(isinstance(cases, list) and 1 <= len(cases) <= bank['randomCount'] + 1,
            'Map bank needs a bounded nonempty case list')
    if not filtered:
        require(len(cases) == bank['randomCount'] + 1 and same_json(cases[0], fixed),
                'Unfiltered Map bank requires the fixed route followed by every OS case')
    identifiers = set()
    for position, data in enumerate(cases):
        require(isinstance(data, dict) and isinstance(data.get('scenario'), dict),
                'Map cases require their public scenario declaration')
        identifier = data['scenario'].get('id')
        require(isinstance(identifier, str) and identifier not in identifiers,
                'Map case IDs must be unique')
        identifiers.add(identifier)
        if identifier == fixed['scenario']['id']:
            require(same_json(data, fixed), 'Fixed Map case differs from the complete public template')
            continue
        match = re.fullmatch(r'options-map-unfiltered-seed-boot-(0|[1-9][0-9]*)', identifier)
        require(match is not None, 'Unknown Map OS case ID')
        index = int(match.group(1))
        require(index < bank['randomCount'] and (filtered or index == position-1),
                'OS case ordinal differs from the declared bank')
        require(same_json(data, project_case(fixed, index, data.get('entropyDraws'))),
                'OS replay must retain its complete declared input/holds/capture/owner projection')
    if filtered:
        selection = bank['caseFilter']
        require(isinstance(selection, str) and 0 < len(selection) <= 4096,
                'Filtered replay requires its nonempty case selection')
        patterns = selection.split(',')
        tests = [re.compile('^' + re.escape(pattern).replace(r'\*', '.*') + '$') for pattern in patterns]
        require(all(pattern and any(test.fullmatch(name) for name in identifiers)
                    for pattern, test in zip(patterns, tests))
                and all(any(test.fullmatch(name) for test in tests) for name in identifiers),
                'Every declared filter and retained case must match')
    return bank


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--scenarios', type=Path)
    mode.add_argument('--validate-bank', type=Path)
    args = parser.parse_args()
    fixed = fixed_case()
    if args.scenarios is not None or args.validate_bank is not None:
        require(args.count is None, 'Recorded replay/validation never draws; omit --count')
        bank = validate_bank(read_json(args.scenarios or args.validate_bank), fixed)
        if args.validate_bank is not None:
            print(json.dumps({'status': 'pass', 'cases': len(bank['cases']), 'randomCount': bank['randomCount']}))
        else:
            print(json.dumps(bank, indent=2))
        return
    count = 1 if args.count is None else args.count
    require(integer(count, 1, 16), 'Fresh Map runs require 1..16 OS cases')
    import secrets
    cases = [copy.deepcopy(fixed)]
    for index in range(count):
        draws = []
        for parameter, bound in DRAW_BOUNDS:
            draw = {'parameter': parameter, 'exclusiveUpperBound': bound, 'sample': secrets.randbelow(bound)}
            # Flush every first logical draw before the next draw or projection.
            print(json.dumps({'kind': 'options-map-os-draw', 'case': index, **draw}), file=sys.stderr, flush=True)
            draws.append(draw)
        cases.append(project_case(fixed, index, draws))
    bank = {'schema': 1, 'suite': 'options-map', 'declarationMode': MODE,
            'randomCount': count, 'entropy': ENTROPY, 'cases': cases}
    print(json.dumps(validate_bank(bank, fixed), indent=2))


if __name__ == '__main__':
    main()
