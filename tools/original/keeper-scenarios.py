#!/usr/bin/env python3
"""Public secret-room entry, anchored Keeper idle and ordinary return tapes."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('keeper_secret', Path(__file__).with_name('secret-door-scenarios.py'))
secret = importlib.util.module_from_spec(spec)
spec.loader.exec_module(secret)


def case(code, selection, parity=0):
    result = secret.secret_case(code, selection, parity)
    index = next(i for i, action in enumerate(result['actions']) if action.get('label') == 'secret-entry') + 1
    idle = [action for frame in range(1, 61) for action in
            ({'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'keeper-idle-{frame}'})]
    result['actions'][index:index] = idle
    start = next(i for i, action in enumerate(result['tape']) if action.get('label') == 'gameplay-start') + 1
    result['tape'] = result['tape'][:start] + result['actions']
    result['npcContract'] = 'keeper-idle'
    secret.traversal.runner.validate_tape(result['tape'])
    return result


def choose():
    # WS03MPHH was independently selected from OS randomness before the
    # original secret-door gate. Its animated Guy7 is retained, not resampled.
    return {'schema': 1, 'contract': 'keeper-idle', 'randomSourceCount': 1, 'cases': [
        case('JF2DSF92', 'contrived opposite-phase actual Keeper entry/idle/return', 1),
        case('WS03MPHH', 'retained OS-random secret Keeper, animated Guy7'),
    ]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.count != 1:
        parser.error('Keeper admission gate retains exactly one recorded OS-random case')
    if args.validate_original:
        proof = secret.traversal.validate(args.validate_original)
        original = json.loads(args.validate_original.read_text())
        keepers = [entity for row in original['debugObservations']
                   for entity in row['observation']['contents']['entities'] if entity['type'] == 17]
        if not keepers or any(entity['variant'] != 1 or entity['subtype'] != entity['spawnSeed'] % 9 + 1 for entity in keepers):
            raise ValueError('Public source route must admit the actual seeded secret Keeper')
        print(json.dumps(proof))
    else:
        print(json.dumps(choose()))


if __name__ == '__main__':
    main()
