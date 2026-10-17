#!/usr/bin/env python3
"""Public Keyboard Controls tapes: four fixed routes plus unfiltered recorded OS cases."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import re
import secrets

ROOT = Path(__file__).resolve().parents[2]
FIXED = ROOT / 'references/isaac/cases/controls/recorded-four-fixed-and-9xzmzpr0-v1.json'
SOURCE = 'OS secrets; unfiltered nonzero-u32 run seed and independent-u32 boot epoch'
spec = importlib.util.spec_from_file_location('controls_seed_codec', Path(__file__).with_name('foundation-seeds.py'))
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)


def fixed_cases():
    return [data for data in json.loads(FIXED.read_text())['cases'] if data['scenario']['randomized'] is False]


def case(seed_value, boot_epoch, identifier, randomized):
    if type(seed_value) is not int or not 1 <= seed_value <= 0xffffffff:
        raise ValueError('Run seed must be a nonzero u32')
    if type(boot_epoch) is not int or not 0 <= boot_epoch <= 0xffffffff:
        raise ValueError('Boot epoch must be a u32')
    fixed = fixed_cases()[0]
    data = copy.deepcopy(fixed)
    code = codec.encode_seed(seed_value)
    data.update(seed=code, seedValue=seed_value, initialSeed=seed_value, bootEpochSeed=boot_epoch)
    tape = data['pairedTape']
    start = next(i for i, action in enumerate(tape) if action == {'type': 'capture', 'label': 'seed-entry'}) + 1
    end = next(i for i, action in enumerate(tape) if action == {'type': 'capture', 'label': 'character-seed-committed'})
    if end - start != 36:
        raise ValueError('Expected exactly eight public seed characters and Confirm')
    for index, (old, new) in enumerate(zip(fixed['seed'].lower(), code.lower())):
        offset = start + index * 4
        if tape[offset:offset + 4] != [{'type': 'key', 'key': old, 'down': True}, {'type': 'step', 'frames': 12}, {'type': 'key', 'key': old, 'down': False}, {'type': 'step', 'frames': 6}]:
            raise ValueError('Public seed-entry grammar changed')
        tape[offset]['key'] = tape[offset + 2]['key'] = new
    data['tape'] = data['bootstrap']['sourceOnlyPrelude'] + tape
    if randomized:
        data['scenario'] = {'id': identifier, 'randomized': True, 'scope': 'One recorded unfiltered nonzero-u32 run seed and independent-u32 boot epoch, with the same fixed public Keyboard W-to-I route and holds. No room/layout filter or resampling.'}
        data['entropy'] = {'source': SOURCE, 'seedValue': seed_value, 'bootEpochSeed': boot_epoch}
    elif identifier != fixed['scenario']['id']:
        raise ValueError('Unknown fixed Controls route')
    return data


def validate(bank):
    if type(bank.get('schema')) is not int or bank['schema'] != 1:
        raise ValueError('Unknown Controls bank schema')
    cases = bank.get('cases')
    if not isinstance(cases, list) or not 1 <= len(cases) <= 20:
        raise ValueError('Controls bank requires 1..20 cases')
    ids = set()
    fixed_by_id = {data['scenario']['id']: data for data in fixed_cases()}
    fixed_count = random_count = 0
    for data in cases:
        scenario = data.get('scenario', {})
        identifier = scenario.get('id')
        if not isinstance(identifier, str) or identifier in ids:
            raise ValueError('Each declared case needs a unique public ID')
        ids.add(identifier)
        randomized = scenario.get('randomized')
        if type(randomized) is not bool:
            raise ValueError('Every case declares its fixed or OS origin')
        if randomized:
            if not re.fullmatch(r'controls-os-movement-up-w-to-i-v1(?:-[1-9][0-9]*)?', identifier):
                raise ValueError('Unknown recorded OS route')
            random_count += 1
        else:
            fixed_count += 1
            if codec.decode_seed(data.get('seed')) != data.get('seedValue') or type(data.get('bootEpochSeed')) is not int or not 0 <= data['bootEpochSeed'] <= 0xffffffff:
                raise ValueError('Fixed public seed/epoch declaration is invalid')
            if identifier not in fixed_by_id or data != fixed_by_id[identifier]:
                raise ValueError('Every fixed Controls route must retain its complete public input/contract')
            continue
        if codec.decode_seed(data.get('seed')) != data.get('seedValue'):
            raise ValueError('Actual public seed code/value disagree')
        if data != case(data['seedValue'], data['bootEpochSeed'], identifier, randomized):
            raise ValueError('Replay must retain the complete declared public route/holds/owners/startup/bindings')
    filtered = bank.get('caseFilter')
    if filtered is not None and (not isinstance(filtered, str) or not filtered):
        raise ValueError('A filtered replay declares its nonempty selection')
    if not filtered and (fixed_count != 4 or random_count < 1):
        raise ValueError('An unfiltered Controls suite requires all four fixed routes plus recorded OS coverage')
    return {'status': 'pass', 'cases': len(cases), 'fixed': fixed_count, 'recordedOS': random_count}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--validate-bank', type=Path)
    args = parser.parse_args()
    if args.validate_bank:
        if args.validate_bank.stat().st_size > 4 * 1024 * 1024:
            raise ValueError('Controls input bank exceeds its bound')
        print(json.dumps(validate(json.loads(args.validate_bank.read_text()))))
        return
    if not 1 <= args.count <= 16:
        raise ValueError('Fresh Controls runs require 1..16 OS cases')
    cases = fixed_cases()
    for index in range(args.count):
        # No layout predicate, duplicate rejection, search or replacement draw.
        seed_value = secrets.randbelow(0xffffffff) + 1
        boot_epoch = secrets.randbits(32)
        identifier = 'controls-os-movement-up-w-to-i-v1' + (f'-{index}' if index else '')
        cases.append(case(seed_value, boot_epoch, identifier, True))
    bank = {'schema': 1, 'cases': cases}
    validate(bank)
    print(json.dumps(bank))


if __name__ == '__main__':
    main()
