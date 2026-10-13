#!/usr/bin/env python3
"""Public routes to naturally selected treasure pedestals; record OS entropy."""
import argparse
import importlib.util
import json
from pathlib import Path
import secrets
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, ROOT / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


clock = load('item_clock', 'tools/original/foundation-clock-run.py')


def route(delay):
    # An upper treasure room is reached through ordinary public movement.
    # This leaf stops before collecting: selection and acquisition have
    # different manager/animation boundaries and independent acceptance.
    return [{'key': 'w', 'frames': 74}, {'key': None, 'frames': 60 + delay}]


def tape(case):
    actions = clock.default_tape(case['code'])
    end = next(index for index, action in enumerate(actions) if action.get('label') == 'gameplay-start')
    actions = [action for action in actions[:end + 1]
               if action['type'] != 'capture' or action['label'] == 'gameplay-start']
    for action in case['actions']:
        if set(action) != {'key', 'frames'} or action['key'] not in (None, 'w', 'a', 's', 'd') \
                or type(action['frames']) is not int or not 1 <= action['frames'] <= 200:
            raise ValueError('Pedestal tape requires declared bounded public inputs')
        if action['key']:
            actions.append({'type': 'key', 'key': action['key'], 'down': True})
        actions.extend({'type': 'step', 'frames': 1} for _ in range(action['frames']))
        if action['key']:
            actions.append({'type': 'key', 'key': action['key'], 'down': False})
    actions.append({'type': 'capture', 'label': 'pedestal-end'})
    clock.validate_tape(actions)
    return actions


def random_cases(count):
    decoder = load('item_rooms', 'tools/foundation/decode-rooms.py')
    rooms = decoder.decode(ROOT / 'references/isaac/rebirth/resources/rooms/00.special rooms.stb')
    candidates = {index for index, room in enumerate(rooms)
                  if room['type'] == 4 and room['variant'] == 8 and
                  (room['width'], room['height']) == (13, 7)}
    if len(candidates) != 1:
        raise ValueError('Expected one complete authored treasure room identity')
    found, seen = [], set()
    for attempt in range(1, 10001):
        seed = secrets.randbelow(0xffffffff) + 1
        if seed in seen:
            continue
        seen.add(seed)
        result = subprocess.run([str(ROOT / 'engine/target/debug/floor')],
                                input=json.dumps({'stageSeed': clock.codec.initial_stage_seed(seed)}),
                                capture_output=True, text=True, check=True)
        floor = json.loads(result.stdout)
        index = floor['descriptors'][floor['startingDescriptor']]['neighbors'][1]
        if index is None:
            continue
        descriptor = floor['descriptors'][index]
        if descriptor['assetStage'] != 0 or descriptor['catalogIndex'] not in candidates:
            continue
        # Item identities, unlock/redraw results and weights are never a
        # discovery filter. Any failing natural selection remains required.
        delay = len(found) % 2
        found.append({'name': f'random-pedestal-{len(found)}', 'code': clock.codec.encode_seed(seed),
                      'seedValue': seed, 'delay': delay, 'actions': route(delay),
                      'entropy': 'OS secrets', 'discoveryAttempts': attempt})
        if len(found) == count:
            return found
    raise RuntimeError(f'Found only {len(found)} required treasure routes in 10000 OS-random floors')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--case', type=Path)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.validate_original:
        verifier = load('item_transport', 'tools/original/foundation-clock-verify.py')
        failures = verifier.validate_run(json.loads(args.validate_original.read_text()))
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures}))
        return 1 if failures else 0
    if args.case:
        print(json.dumps(tape(json.loads(args.case.read_text()))))
        return 0
    if not 2 <= args.count <= 4:
        raise ValueError('Pedestal gate requires 2..4 newly recorded OS-random cases')
    code = 'W0L0LYYM'
    contrived = [{'name': f'pedestal-phase-{delay}', 'code': code,
                  'seedValue': clock.codec.decode_seed(code), 'delay': delay,
                  'actions': route(delay), 'entropy': 'Contrived public admission phase offset'}
                 for delay in (0, 1)]
    print(json.dumps({'schema': 1, 'bootEpochSeed': 1700000000,
                      'source': 'Public seeded menus/movement; full authored room discovery, no selected-item filtering',
                      'cases': contrived + random_cases(args.count)}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
