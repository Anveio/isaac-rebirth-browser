#!/usr/bin/env python3
"""Public natural key collection, ordinary library unlock and revisit tapes."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('key_door_secret', Path(__file__).with_name('secret-door-scenarios.py'))
secret = importlib.util.module_from_spec(spec)
spec.loader.exec_module(secret)


def case(code, selection, parity=0, denied=False):
    result = secret.secret_case(code, selection, parity)
    end = next(index for index, action in enumerate(result['actions']) if action.get('label') == 'secret-entry') + 1
    actions = result['actions'][:end]

    def held(key, count, prefix, settle=0):
        actions.append({'type': 'key', 'key': key, 'down': True})
        for index in range(count):
            actions.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'{prefix}-{index + 1}'}])
        actions.append({'type': 'key', 'key': key, 'down': False})
        for index in range(settle):
            actions.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'{prefix}-settle-{index + 1}'}])

    if denied:
        remainder = actions[2:]
        actions = actions[:2]
        held('w', 70, 'no-key-approach', 20)
        held('s', 66, 'no-key-retreat', 30)
        actions += remainder
    for index, (key, count, settle) in enumerate([('a', 13, 20), ('s', 25, 30), ('d', 25, 30), ('a', 13, 20)]):
        held(key, count, f'key-route-{index + 1}', settle)
    held('w', 60, 'secret-return', 40)
    held('w', 90, 'library-approach', 40)
    actions.append({'type': 'capture', 'label': 'library-entered'})
    held('s', 18, 'library-return', 40)
    held('w', 18, 'library-revisit', 40)
    held('s', 18, 'library-return-again', 40)
    actions.append({'type': 'capture', 'label': 'returned'})
    start = next(index for index, action in enumerate(result['tape']) if action.get('label') == 'gameplay-start') + 1
    result.update(doorContract='key', mode='key-return', expectedVisitedRooms=3, setup=[], actions=actions)
    result['tape'] = result['tape'][:start] + actions
    secret.traversal.runner.validate_tape(result['tape'])
    return result


def choose():
    # Recorded independently from OS randomness before running or grading
    # these original/browser routes. W31's second tape stresses the opposite
    # parity and the actual denied approach before any natural acquisition.
    return {'schema': 1, 'contract': 'key', 'randomSourceCount': 1, 'cases': [
        case('W31RQ6TH', 'contrived opposite-phase denied approach, natural keys, unlock and revisit', 1, True),
        case('7XGY2Q9P', 'recorded OS-random natural-key/library configuration, discovery attempt20107'),
    ]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.count != 1:
        parser.error('This scoped gate retains exactly one recorded OS-random library configuration')
    print(json.dumps(secret.traversal.validate(args.validate_original) if args.validate_original else choose()))


if __name__ == '__main__':
    main()
