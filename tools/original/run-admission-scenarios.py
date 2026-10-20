#!/usr/bin/env python3
"""Declared original clock/admission public tapes; no outcome filtering."""
import argparse
import importlib.util
import json
from pathlib import Path
import secrets

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('admission_clock', ROOT / 'tools/original/foundation-clock-run.py')
clock = importlib.util.module_from_spec(spec)
spec.loader.exec_module(clock)

def tape(case):
    actions = clock.default_tape(case['code'])
    # A longer declared intro moves the browser reset boundary later in the
    # original's clock; the harness derives the reset clock from this tape.
    actions[0] = {'type': 'step', 'frames': actions[0]['frames'] + case.get('introWait', 0)}
    actions = actions[:next(i for i, a in enumerate(actions) if a.get('label') == 'character-seeded') + 1]
    if case['beforeRunWait']:
        actions.append({'type': 'step', 'frames': case['beforeRunWait']})
    actions.append({'type': 'capture', 'label': 'before-run-confirmation'})
    actions.append({'type': 'key', 'key': 'Return', 'down': True})
    actions.extend({'type': 'step', 'frames': 1} for _ in range(12))
    actions.append({'type': 'key', 'key': 'Return', 'down': False})
    if case['heldKey']:
        actions.append({'type': 'key', 'key': case['heldKey'], 'down': True})
    actions.extend({'type': 'step', 'frames': 1} for _ in range(300))
    actions.append({'type': 'capture', 'label': 'gameplay-start'})
    if case['heldKey']:
        actions.append({'type': 'key', 'key': case['heldKey'], 'down': False})
    actions.extend({'type': 'step', 'frames': 1} for _ in range(30))
    actions.append({'type': 'capture', 'label': 'settled'})
    clock.validate_tape(actions)
    return actions

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.validate_original:
        spec = importlib.util.spec_from_file_location('admission_validator', ROOT / 'tools/original/foundation-clock-verify.py')
        validator = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(validator)
        manifest = json.loads(args.validate_original.read_text())
        failures = validator.validate_run(manifest)
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures,
            'issuedInputs': len(manifest.get('inputs', [])),
            'acknowledgedPresentations': len(manifest.get('presentations', [])),
            'stepNs': manifest.get('clock', {}).get('stepNs')}))
        raise SystemExit(1 if failures else 0)
    if not 2 <= args.count <= 16:
        raise SystemExit('Requires2..16 unfiltered recorded OS-random cases')
    cases = [
        {'name': 'fixed-idle', 'code': 'T1PK6Y0W', 'bootEpochSeed': 1700000000,
         'beforeRunWait': 0, 'heldKey': None, 'selection': 'contrived dense idle fade'},
        {'name': 'opposite-phase-held-input', 'code': 'QSYVKMSE', 'bootEpochSeed': 0,
         'beforeRunWait': 1, 'heldKey': 'd', 'selection': 'contrived one declared menu presentation and held movement'},
        {'name': 'long-intro', 'code': 'T1PK6Y0W', 'bootEpochSeed': 123456789, 'introWait': 17,
         'beforeRunWait': 0, 'heldKey': None, 'selection': 'contrived 17-presentation longer intro before the Title'},
    ]
    for index in range(args.count):
        vector = clock.codec.random_vector()
        cases.append({'name': f'os-{index}', 'code': vector['code'].replace(' ', ''),
            'bootEpochSeed': secrets.randbits(32), 'beforeRunWait': index % 2,
            'heldKey': 's' if index % 2 else None, 'selection': 'unfiltered OS secrets before observing any game outcome'})
    for case in cases:
        case['runSeed'] = clock.codec.decode_seed(case['code'])
        case['tape'] = tape(case)
    print(json.dumps({'schema': 1, 'clockInput': {'presentationIntervalNanoseconds': 16667000},
        'scope': 'Dense real public Enter/held/release/idle steps across menu request and full-only initial Game fade; all selected outcomes remain required',
        'cases': cases}, indent=2))

if __name__ == '__main__':
    main()
