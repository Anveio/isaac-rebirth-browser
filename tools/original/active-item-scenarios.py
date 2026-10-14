#!/usr/bin/env python3
"""Public acquisition/use tapes; retain all recorded and new random outcomes."""
import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('active_items_scenarios', ROOT / 'tools/original/item-scenarios.py')
items = importlib.util.module_from_spec(spec)
spec.loader.exec_module(items)


def use_tail(damaged=False):
    # A nearby own bomb supplies actual damage through the original public
    # callback, not a debug HP command. Full-health use remains a separate
    # required case. Hold/repress action9 while the real charge is exhausted;
    # motion and firing requests occur during the authored UseItem guard.
    return ([{'key': 'e', 'frames': 1}, {'key': None, 'frames': 199}] if damaged else []) + [
        {'key': 'space', 'frames': 12}, {'key': 'w', 'frames': 6},
        {'key': 'Right', 'frames': 6}, {'key': 'space', 'frames': 1},
        {'key': 'q', 'frames': 1}, {'key': None, 'frames': 40},
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    args = parser.parse_args()
    if args.count != 2:
        raise ValueError('The complete twelve-case active gate requires two new unfiltered OS cases')
    # Source-owned port selection predicts this contrived public route. The
    # gate independently requires original acquisition45; prediction is not
    # original evidence. No random discovery selects on an item identity.
    code = 'B911XGSG'
    cases = []
    for damaged in (False, True):
        for delay in (0, 1):
            cases.append({'name': f'yum-{"damaged" if damaged else "full"}-phase-{delay}',
                          'code': code, 'seedValue': items.clock.codec.decode_seed(code),
                          'delay': delay, 'expectedItemId': 45,
                          'actions': items.route(delay, True) + use_tail(damaged),
                          'entropy': 'Contrived public acquisition/use and admission phase offset'})
    # Hold public shooting through the real naturally acquired Pickup guard.
    # Both offsets retain the ordinary contact/movement and complete use tail;
    # these are additional contrived cases, not filtered random discoveries.
    for delay in (0, 1):
        cases.append({'name': f'yum-pickup-aim-phase-{delay}', 'code': code,
                      'seedValue': items.clock.codec.decode_seed(code), 'delay': delay,
                      'expectedItemId': 45, 'requirePickupShotGuard': True,
                      'actions': items.route(delay, True, True) + use_tail(),
                      'entropy': 'Contrived public held shooting during natural Pickup'})
    # Preserve earlier unfiltered active291/passive48 discoveries, including
    # the unsupported active291 use outcome as a real blocking comparison.
    for index, (recorded, delay, attempt) in enumerate([('R74QYMYS', 0, 3), ('VRQPT844', 1, 75)]):
        cases.append({'name': f'recorded-active-use-{index}', 'code': recorded,
                      'seedValue': items.clock.codec.decode_seed(recorded), 'delay': delay,
                      'actions': items.route(delay, True) + use_tail(),
                      'entropy': 'Recorded OS secrets', 'discoveryAttempts': attempt,
                      'recordedEvidence': 'references/isaac/foundation/items/acquisition-initial-live.json'})
    # Preserve the initial fresh OS outcomes as real regression inputs too.
    # Their unfiltered discovery has already happened; new random discoveries
    # below remain independent and are never selected on item identity.
    for index, (recorded, delay, attempt) in enumerate([('DN8RQDXC', 0, 19), ('98C1GNPZ', 1, 139)]):
        cases.append({'name': f'retained-random-pedestal-{index}', 'code': recorded,
                      'seedValue': items.clock.codec.decode_seed(recorded), 'delay': delay,
                      'actions': items.route(delay, True) + use_tail(),
                      'entropy': 'Recorded OS secrets', 'discoveryAttempts': attempt,
                      'recordedEvidence': 'references/isaac/foundation/items/active-use-verification.json'})
    for case in items.random_cases(args.count, True):
        case['actions'] += use_tail()
        cases.append(case)
    print(json.dumps({'schema': 1, 'mode': 'active-use', 'bootEpochSeed': 1700000000,
                      'source': 'Public input/own bomb damage; every OS item outcome retained',
                      'cases': cases}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
