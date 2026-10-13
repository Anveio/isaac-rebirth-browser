#!/usr/bin/env python3
"""Public secret-door routes to naturally authored regular keys."""
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


rock = load('key_public_clock', 'tools/original/rock-blast-scenarios.py')


def route(delay):
    return [{'key': key, 'frames': frames} for key, frames in [
        (None, 60 + delay), ('e', 1), ('w', 35), (None, 20),
        (None, 78), ('s', 45), (None, 40), ('a', 13), (None, 20),
        ('s', 25), (None, 30), ('d', 25), (None, 30)]]


def random_cases(count):
    decoder = load('key_authored_rooms', 'tools/foundation/decode-rooms.py')
    rooms = decoder.decode(ROOT / 'references/isaac/rebirth/resources/rooms/00.special rooms.stb')
    candidates = [index for index, room in enumerate(rooms)
                  if room['type'] == 7 and room['variant'] == 11 and
                  (room['width'], room['height']) == (13, 7) and
                  [(group['x'], group['y'], [(entry['type'], entry['variant'], entry['subtype'])
                                            for entry in group['entries']])
                   for group in room['spawns']] == [
                       (4, 0, [(17, 1, 0)]), (5, 3, [(5, 30, 0)]), (7, 3, [(5, 30, 0)])]]
    if len(candidates) != 1:
        raise ValueError('Key route requires the complete authored two-key secret room identity')
    found, seen = [], set()
    for attempt in range(1, 10001):
        seed = secrets.randbelow(0xffffffff) + 1
        if seed in seen:
            continue
        seen.add(seed)
        result = subprocess.run([str(ROOT / 'engine/target/debug/floor')],
                                input=json.dumps({'stageSeed': rock.clock.codec.initial_stage_seed(seed)}),
                                capture_output=True, text=True, check=True)
        floor = json.loads(result.stdout)
        index = floor['descriptors'][floor['startingDescriptor']]['neighbors'][3]
        if index is None:
            continue
        descriptor = floor['descriptors'][index]
        if descriptor['assetStage'] != 0 or descriptor['catalogIndex'] != candidates[0]:
            continue
        # Do not filter golden subtype draws: a selected unsupported key must
        # remain a functional failure in the recorded cohort.
        delay = len(found) % 2
        found.append({'name': f'random-key-{len(found)}', 'code': rock.clock.codec.encode_seed(seed),
                      'seedValue': seed, 'delay': delay, 'actions': route(delay),
                      'entropy': 'OS secrets', 'discoveryAttempts': attempt})
        if len(found) == count:
            return found
    raise RuntimeError(f'Only {len(found)} natural key routes found in 10000 OS-random candidates')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--case', type=Path)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.validate_original:
        verifier = load('key_provenance', 'tools/original/foundation-clock-verify.py')
        failures = verifier.validate_run(json.loads(args.validate_original.read_text()))
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures}))
        return 1 if failures else 0
    if args.case:
        print(json.dumps(rock.tape(json.loads(args.case.read_text()))))
        return 0
    if not 2 <= args.count <= 4:
        raise ValueError('Natural key coverage requires 2..4 newly recorded OS-random seeds')
    code = 'JF2DSF92'
    contrived = [{'name': f'key-phase-{delay}', 'code': code,
                  'seedValue': rock.clock.codec.decode_seed(code), 'delay': delay,
                  'actions': route(delay), 'entropy': 'Contrived public collection phase offset'}
                 for delay in (0, 1)]
    print(json.dumps({'schema': 1, 'bootEpochSeed': 1700000000,
                      'source': 'Public seed and keys only; authored discovery never enters WASM',
                      'cases': contrived + random_cases(args.count)}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
