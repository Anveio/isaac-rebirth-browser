#!/usr/bin/env python3
"""Gamma public inputs on the existing Options/Map pure seed mapper."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('gamma_public_mapper', ROOT/'tools/original/options-map-scenarios.py')
shared = importlib.util.module_from_spec(spec)
spec.loader.exec_module(shared)
FIXED = ROOT/'references/isaac/cases/options-gamma/gamma-clamps-back-surface-v1.json'
MODE = 'fixed-plus-unfiltered-gamma-seed-boot-v1'


def project(fixed, index, draws):
    data = shared.project_case(fixed, index, draws)
    data['scenario']['id'] = 'options-gamma-unfiltered-seed-boot-' + str(index)
    return data


def validate(bank, fixed):
    require = shared.require
    fields = {'schema', 'suite', 'declarationMode', 'randomCount', 'entropy', 'cases'}
    require(isinstance(bank, dict) and set(bank) in (fields, fields|{'caseFilter'}), 'Invalid Gamma bank fields')
    require(shared.integer(bank['schema'], 1, 1) and bank['suite'] == 'options-gamma'
            and bank['declarationMode'] == MODE and bank['entropy'] == shared.ENTROPY
            and shared.integer(bank['randomCount'], 1, 16), 'Invalid Gamma declaration')
    cases, filtered = bank['cases'], 'caseFilter' in bank
    require(isinstance(cases, list) and 1 <= len(cases) <= bank['randomCount']+1, 'Gamma bank is empty or unbounded')
    if not filtered:
        require(len(cases) == bank['randomCount']+1 and shared.same_json(cases[0], fixed), 'Retain fixed plus every OS case')
    ids = set()
    for position, case in enumerate(cases):
        require(isinstance(case, dict) and isinstance(case.get('scenario'), dict), 'Missing Gamma scenario')
        name = case['scenario'].get('id')
        require(isinstance(name, str) and name not in ids, 'Duplicate or invalid Gamma ID')
        ids.add(name)
        if name == fixed['scenario']['id']:
            require(shared.same_json(case, fixed), 'Fixed Gamma public inputs changed')
            continue
        match = re.fullmatch(r'options-gamma-unfiltered-seed-boot-(0|[1-9][0-9]*)', name)
        require(match is not None, 'Unknown Gamma OS ID')
        index = int(match.group(1))
        require(index < bank['randomCount'] and (filtered or index == position-1), 'Gamma OS draw order changed')
        require(shared.same_json(case, project(fixed, index, case.get('entropyDraws'))), 'Gamma OS public projection changed')
    if filtered:
        require(isinstance(bank['caseFilter'], str) and 0 < len(bank['caseFilter']) <= 4096, 'Missing Gamma selection')
        patterns = bank['caseFilter'].split(',')
        tests = [re.compile('^'+re.escape(p).replace(r'\*', '.*')+'$') for p in patterns]
        require(all(p and any(t.fullmatch(n) for n in ids) for p,t in zip(patterns,tests))
                and all(any(t.fullmatch(n) for t in tests) for n in ids), 'Gamma filter membership changed')
    return bank


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--scenarios', type=Path)
    mode.add_argument('--validate-bank', type=Path)
    args = parser.parse_args()
    fixed = shared.read_json(FIXED)
    shared.require(fixed.get('declarationMode') == 'gamma-main-options-source-v1'
                   and fixed['scenario']['randomized'] is False, 'Invalid fixed Gamma declaration')
    shared.require(shared.integer(fixed.get('schema'), 1, 1)
                   and shared.integer(fixed.get('seedValue'), 1, 0xffffffff)
                   and shared.decode_seed(fixed.get('seed')) == fixed['seedValue']
                   and shared.integer(fixed.get('bootEpochSeed'), 0, 0xffffffff)
                   and shared.integer(fixed.get('initialSeed'), shared.INITIAL_SEED, shared.INITIAL_SEED),
                   'Fixed Gamma seed/boot/initial declarations are invalid')
    native = [a['label'] for a in fixed['tape'] if a['type'] == 'capture']
    paired = [a['label'] for a in fixed['pairedTape'] if a['type'] == 'capture']
    shared.require(fixed['tape'] == fixed['bootstrap']['sourceOnlyPrelude'] + fixed['pairedTape']
                   and len(native) == len(set(native)) == 21 and len(paired) == len(set(paired)) == 20
                   and native == fixed['captureDeclaration']['nativeLabels']
                   and paired == fixed['captureDeclaration']['pairedLabels']
                   and fixed['captureDeclaration']['sourceOnlyLabels'] == ['intro']
                   and sum(a.get('frames', 0) for a in fixed['pairedTape']) == 1550,
                   'Gamma literal capture/hold partition changed')
    shared.seed_text_start(fixed['pairedTape'], fixed['seed'])
    if args.scenarios is not None or args.validate_bank is not None:
        shared.require(args.count is None, 'Recorded Gamma replay never draws')
        bank = validate(shared.read_json(args.scenarios or args.validate_bank), fixed)
        print(json.dumps({'status':'pass', 'cases':len(bank['cases'])} if args.validate_bank else bank, indent=2))
        return
    count = 1 if args.count is None else args.count
    shared.require(shared.integer(count, 1, 16), 'Gamma fresh count must be1..16')
    import secrets
    cases = [copy.deepcopy(fixed)]
    for index in range(count):
        draws = []
        for parameter,bound in shared.DRAW_BOUNDS:
            row = dict(parameter=parameter, exclusiveUpperBound=bound, sample=secrets.randbelow(bound))
            print(json.dumps(dict(kind='options-gamma-os-draw', case=index, **row)), file=sys.stderr, flush=True)
            draws.append(row)
        cases.append(project(fixed, index, draws))
    bank = dict(schema=1, suite='options-gamma', declarationMode=MODE, randomCount=count, entropy=shared.ENTROPY, cases=cases)
    print(json.dumps(validate(bank, fixed), indent=2))


if __name__ == '__main__':
    main()
