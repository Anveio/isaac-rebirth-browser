#!/usr/bin/env python3
"""Input-only TNT common-callback bank: fixed and first OS epoch/idle draws.

The run seed and natural room remain fixed. Randomization varies public boot
and input phase, not the room catalogue. No Native observation selects inputs.
"""
import argparse
import copy
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
FIXED = ROOT/'references/isaac/cases/tnt/right-entry-shoot-music1-v2.json'
MODE = 'fixed-plus-unfiltered-epoch-idle-v1'


def author(template, epoch, hold):
    data = copy.deepcopy(template)
    data['id'] = data['scenario']['id'] = 'tnt-unfiltered-epoch-idle-0'
    data['bootEpochSeed'] = epoch
    data['parameters'] = {'beforeFireIdleFrames': hold}
    data['entropyDraws'] = [
        {'name': 'bootEpochSeed', 'source': 'OS secrets.randbits(32)', 'value': epoch},
        {'name': 'beforeFireIdleFrames', 'source': 'OS secrets.randbelow(16)', 'value': hold},
    ]
    tape = data['pairedTape']
    fire = [i for i, row in enumerate(tape) if row == {'type': 'key', 'key': 'Right', 'down': True}]
    if len(fire) != 1:
        raise ValueError('TNT public template needs exactly one declared firing edge')
    if hold:
        tape[fire[0]:fire[0]] = [{'type': 'step', 'frames': hold}]
    data['tape'] = copy.deepcopy(data['bootstrap']['sourceOnlyPrelude'] + tape)
    return data


def same_json(left, right):
    # JSON owner types are strict: true is never an integer1 or numeric field.
    return json.dumps(left, sort_keys=True, separators=(',', ':'), allow_nan=False) == \
        json.dumps(right, sort_keys=True, separators=(',', ':'), allow_nan=False)


def validate(bank, template):
    if not isinstance(bank, dict) or type(bank.get('schema')) is not int or bank['schema'] != 1 \
            or bank.get('suite') != 'tnt' or bank.get('declarationMode') != MODE \
            or type(bank.get('randomCount')) is not int or bank['randomCount'] != 1:
        raise ValueError('TNT requires a declared fixed-plus-one-unfiltered-OS bank')
    if set(bank) != {'schema', 'suite', 'declarationMode', 'randomCount', 'entropy', 'cases'}:
        raise ValueError('Unknown or missing TNT bank declaration')
    cases = bank.get('cases')
    if not isinstance(cases, list) or len(cases) != 2 or not same_json(cases[0], template):
        raise ValueError('TNT bank must retain its complete fixed public case first')
    random = cases[1]
    if not isinstance(random, dict) or not isinstance(random.get('parameters'), dict):
        raise ValueError('Recorded TNT OS case/parameters must be objects')
    epoch = random.get('bootEpochSeed')
    hold = random.get('parameters', {}).get('beforeFireIdleFrames')
    if type(epoch) is not int or not 0 <= epoch <= 0xffffffff or type(hold) is not int or not 0 <= hold <= 15:
        raise ValueError('Recorded TNT OS epoch/hold outside declared domains')
    if not same_json(random, author(template, epoch, hold)):
        raise ValueError('Recorded TNT case differs from its declared public draws/tape')
    if bank['entropy'] != 'First two logical OS draws retained; no resampling or Native-dependent continuation':
        raise ValueError('Missing unfiltered TNT draw provenance')
    return bank


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int)
    parser.add_argument('--scenarios', type=Path)
    args = parser.parse_args()
    template = json.loads(FIXED.read_text())
    if template.get('seedValue') != 26 or template.get('seed') != 'B911933Y' \
            or template.get('declarationMode') != 'source-plan-only-public-tnt-music1-v2':
        raise ValueError('Unadmitted fixed public TNT template')
    if args.scenarios:
        if args.count is not None:
            raise ValueError('Replay never draws entropy; omit --count with --scenarios')
        bank = validate(json.loads(args.scenarios.read_text()), template)
    else:
        if args.count not in (None, 1):
            raise ValueError('This bounded bank requires exactly one first-draw OS case')
        #Validate the public firing seam before any entropy. Persist each first
        #logical draw on stderr so even a later generation failure retains it.
        author(template, template['bootEpochSeed'], 0)
        import secrets
        epoch = secrets.randbits(32)
        print(json.dumps({'kind': 'tnt-os-draw', 'name': 'bootEpochSeed', 'value': epoch}), file=sys.stderr, flush=True)
        hold = secrets.randbelow(16)
        print(json.dumps({'kind': 'tnt-os-draw', 'name': 'beforeFireIdleFrames', 'value': hold}), file=sys.stderr, flush=True)
        bank = validate({'schema': 1, 'suite': 'tnt', 'declarationMode': MODE, 'randomCount': 1,
            'entropy': 'First two logical OS draws retained; no resampling or Native-dependent continuation',
            'cases': [template, author(template, epoch, hold)]}, template)
    print(json.dumps(bank, indent=2))


if __name__ == '__main__':
    main()
