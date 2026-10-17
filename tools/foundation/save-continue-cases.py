#!/usr/bin/env python3
"""Public Save tapes mapped from declared seeds; no entropy or Native data."""
import argparse
import copy
import importlib.util
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CASE_ROOT = ROOT / 'references/isaac/cases/save-continue'
spec = importlib.util.spec_from_file_location('save_public_seed_codec', ROOT / 'tools/original/foundation-seeds.py')
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)


def template(name):
    if name not in ('initial-room-footprints', 'initial-room-stationary'):
        raise ValueError('Unknown public Save template')
    return json.loads((CASE_ROOT / (name + '.json')).read_text())


def case_for_seed(seed_value, ordinal):
    code = codec.encode_seed(seed_value)
    if type(ordinal) is not int or ordinal < 0:
        raise ValueError('OS ordinal must be nonnegative')
    result = copy.deepcopy(template('initial-room-footprints'))
    for name in ('tape', 'pairedTape'):
        tape = result[name]
        start = next(i for i, row in enumerate(tape) if row.get('type') == 'key'
                     and row.get('key') == 'Tab' and row.get('down') is False) + 1
        end = next(i for i, row in enumerate(tape) if row.get('label') == 'seed-entry')
        entries = [row for row in tape[start:end] if row.get('type') == 'key']
        if len(entries) != 16:
            raise ValueError('Seed dialog must declare eight complete physical pairs')
        for i, char in enumerate(code.lower()):
            if entries[2*i].get('down') is not True or entries[2*i+1].get('down') is not False:
                raise ValueError('Seed dialog pair is incomplete')
            entries[2*i]['key'] = char
            entries[2*i+1]['key'] = char
    result.update(seed=code, seedValue=seed_value, initialSeed=seed_value)
    result['scenario'].update(id=f'save-os-initial-footprints-{ordinal:04}', randomized=True,
        description='One recorded unfiltered OS public run seed; unchanged moving-before-Save tape, own IO, Continue and natural reload. No observed floor or door filtering.')
    result['saveContinueContract']['osSeed'] = {'ordinal': ordinal, 'u32': seed_value,
        'scope': 'Recorded before mapping; no resampling by route, support, Original or Browser outcome'}
    return result


def same_public_json(left, right):
    # JSON type-sensitive: bools cannot impersonate integer action/contract owners.
    encode = lambda value: json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False)
    return encode(left) == encode(right)


def validate_bank(bank):
    if type(bank) is not dict or type(bank.get('schema')) is not int or bank['schema'] != 1:
        raise ValueError('Save bank must declare schema1')
    count = bank.get('randomCount')
    if type(count) is not int or not 0 <= count <= 16:
        raise ValueError('Save bank must declare a bounded integer OS count')
    draws = bank.get('osDraws')
    if type(draws) is not list or len(draws) != count:
        raise ValueError('Every original OS draw must remain in the bank')
    for ordinal, draw in enumerate(draws):
        if type(draw) is not dict or draw.get('ordinal') != ordinal or type(draw.get('ordinal')) is not int:
            raise ValueError('OS draw ordinals must be complete, unique and ordered')
        if type(draw.get('u32')) is not int or not 0 <= draw['u32'] <= 0xffffffff:
            raise ValueError('OS receipt must retain its actual u32')
        if draw.get('source') != 'OS randomBytes(4) before mapping' or type(draw.get('bytesHex')) is not str:
            raise ValueError('OS receipt must retain the pre-mapping byte owner')
        raw = bytes.fromhex(draw['bytesHex'])
        if len(raw) != 4 or int.from_bytes(raw, 'little') != draw['u32']:
            raise ValueError('OS draw byte/u32 join failed')
    cases = bank.get('cases')
    if type(cases) is not list or not cases:
        raise ValueError('Save bank must contain actual public cases')
    known = {template(name)['scenario']['id']: template(name)
             for name in ('initial-room-footprints', 'initial-room-stationary')}
    ids = set()
    for case in cases:
        if type(case['scenario']['randomized']) is not bool:
            raise ValueError('Each public case must declare a boolean randomized owner')
        identity = case['scenario']['id']
        if identity in ids:
            raise ValueError('Duplicate Save public case')
        ids.add(identity)
        if case['scenario']['randomized'] is False:
            if identity not in known or not same_public_json(case, known[identity]):
                raise ValueError('Fixed case differs from its declared public tape')
        else:
            draw = case['saveContinueContract']['osSeed']
            if not same_public_json(case, case_for_seed(draw['u32'], draw['ordinal'])):
                raise ValueError('OS case differs from its declared public mapping')
            receipt = next((row for row in bank.get('osDraws', []) if row.get('ordinal') == draw['ordinal']), None)
            if receipt is None or receipt.get('u32') != draw['u32'] or receipt.get('source') != 'OS randomBytes(4) before mapping':
                raise ValueError('OS case lacks its pre-mapping public draw receipt')
            raw = bytes.fromhex(receipt['bytesHex'])
            if len(raw) != 4 or int.from_bytes(raw, 'little') != draw['u32']:
                raise ValueError('OS draw byte/u32 join failed')
    if 'caseFilter' in bank:
        marker = bank['caseFilter']
        if type(marker) is not str or not marker:
            raise ValueError('Partial bank requires a nonempty explicit caseFilter string')
        patterns = marker.split(',')
        tests = []
        for pattern in patterns:
            if not pattern:
                raise ValueError('Partial-bank patterns cannot be empty')
            test = re.compile('^' + re.escape(pattern).replace(r'\*', '.*') + '$')
            if not any(test.fullmatch(identity) for identity in ids):
                raise ValueError('Persisted caseFilter pattern matches no recorded ID')
            tests.append(test)
        if not all(any(test.fullmatch(identity) for test in tests) for identity in ids):
            raise ValueError('Partial bank contains an ID outside its persisted filter')
    else:
        if not any(case['scenario']['randomized'] is False for case in cases):
            raise ValueError('Default bank requires a fixed public control')
        if sum(case['scenario']['randomized'] is True for case in cases) != bank.get('randomCount'):
            raise ValueError('Every declared OS draw must remain selected')
    return bank


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed-value', type=int)
    parser.add_argument('--ordinal', type=int)
    parser.add_argument('--validate-bank', type=Path)
    parser.add_argument('--fixed', choices=['initial-room-footprints', 'initial-room-stationary'])
    args = parser.parse_args()
    if args.validate_bank:
        value = validate_bank(json.loads(args.validate_bank.read_text()))
    elif args.fixed:
        value = template(args.fixed)
    else:
        value = case_for_seed(args.seed_value, args.ordinal)
    print(json.dumps(value, indent=2))


if __name__ == '__main__':
    main()
