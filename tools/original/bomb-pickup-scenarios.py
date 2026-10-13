#!/usr/bin/env python3
"""Public ordinary-rock access routes to naturally authored regular/double bombs."""
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


rock = load('bomb_pickup_public_clock', 'tools/original/rock-blast-scenarios.py')


def route(delay, slot):
    if slot not in (1, 3):
        raise ValueError('This collection slice owns vertical ordinary-room entry')
    entry, inward = ('w', 'w') if slot == 1 else ('s', 's')
    # The opposite entry position is 120 world units from the pickup row.
    # The real starting bomb opens two rock rows without imparting PU velocity.
    return [{'key': key, 'frames': frames} for key, frames in [
        (entry, 74 if slot == 1 else 17), (None, 40 + delay),
        ('e', 1), ('d', 25), (None, 95), ('a', 25), (None, 30),
        (inward, 35), (None, 30), ('a', 13), (None, 30),
        ('d', 25), (None, 30), (None, 20)]]


def ordinary_rock_access(descriptor, groups):
    state = rock.next11(descriptor['spawnSeed'])
    x = state % 13 + 1
    state = rock.next11(state)
    tinted = x + (state % 7 + 1) * 15
    occupied_rocks = {group['x'] + 1 + (group['y'] + 1) * 15
                      for group in groups if any(entry['type'] == 1000 for entry in group['entries'])}
    if tinted in occupied_rocks:
        return False
    for group in groups:
        state = rock.next11(state)
        for entry in group['entries']:
            if entry['type'] == 1000 and rock.next35(state) % 1001 < 16:
                return False
    return True


def random_cases(count):
    decoder = load('bomb_pickup_authored_rooms', 'tools/foundation/decode-rooms.py')
    rooms = decoder.decode(ROOT / 'references/isaac/rebirth/resources/rooms/01.basement.stb')
    candidates = [index for index, room in enumerate(rooms)
                  if room['type'] == 1 and room['variant'] == 35 and
                  (room['width'], room['height']) == (13, 7) and
                  [(group['x'], group['y']) for group in room['spawns']
                   if any(entry['type'] == 5 for entry in group['entries'])] == [(5, 3), (6, 3), (7, 3)] and
                  all((entry['type'], entry['variant'], entry['subtype']) in ((1000, 0, 0), (5, 40, 0))
                      for group in room['spawns'] for entry in group['entries'])]
    if len(candidates) != 1:
        raise ValueError('Bomb pickup route requires the complete authored ordinary Room35 identity')
    found, seen = [], set()
    for attempt in range(1, 20001):
        seed = secrets.randbelow(0xffffffff) + 1
        if seed in seen:
            continue
        seen.add(seed)
        result = subprocess.run([str(ROOT / 'engine/target/debug/floor')],
                                input=json.dumps({'stageSeed': rock.clock.codec.initial_stage_seed(seed)}),
                                capture_output=True, text=True, check=True)
        floor = json.loads(result.stdout)
        for slot in (1, 3):
            index = floor['descriptors'][floor['startingDescriptor']]['neighbors'][slot]
            if index is None:
                continue
            descriptor = floor['descriptors'][index]
            if descriptor['assetStage'] != 1 or descriptor['catalogIndex'] != candidates[0]:
                continue
            # Only grid access is isolated. Every authored PU subtype draw is
            # unfiltered; unsupported conversion remains a blocking mismatch.
            if not ordinary_rock_access(descriptor, rooms[candidates[0]]['spawns']):
                continue
            delay = len(found) % 2
            found.append({'name': f'unfiltered-bomb-pickup-{len(found)}',
                          'code': rock.clock.codec.encode_seed(seed), 'seedValue': seed,
                          'slot': slot, 'delay': delay, 'actions': route(delay, slot),
                          'entropy': 'OS secrets', 'discoveryAttempts': attempt})
            break
        if len(found) == count:
            return found
    raise RuntimeError(f'Only {len(found)} natural regular/double bomb routes found in 20000 OS-random candidates')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--case', type=Path)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.validate_original:
        verifier = load('bomb_pickup_provenance', 'tools/original/foundation-clock-verify.py')
        failures = verifier.validate_run(json.loads(args.validate_original.read_text()))
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures}))
        return 1 if failures else 0
    if args.case:
        print(json.dumps(rock.tape(json.loads(args.case.read_text()))))
        return 0
    if not 2 <= args.count <= 4:
        raise ValueError('Natural bomb pickup coverage requires 2..4 newly recorded OS-random seeds')
    scoped = json.loads((ROOT / 'references/isaac/foundation/bomb-pickups/scoped-scenarios.json').read_text())
    retained = []
    for case in scoped['cases']:
        # Preserve every selected seed and original public interval as an exact
        # prefix. Regular Collect is still playing at the old tape's end; the
        # explicit 20-step public release continuation observes its retirement.
        retained.append({**case, 'name': 'retained-' + case['name'],
                         'originalActions': case['actions'],
                         'actions': case['actions'] + [{'key': None, 'frames': 20}],
                         'continuationScope': 'Original scoped case unchanged as prefix; source-required Collect completion wait'})
    print(json.dumps({'schema': 1, 'bootEpochSeed': 1700000000,
                      'source': 'Public seed and held-key intervals only; authored discovery never enters WASM',
                      'scope': 'Fresh Isaac ordinary-rock access with unfiltered authored bomb pickup subtype draws. All four prior scoped cases remain required with exact original public prefixes plus a declared completion wait. Newly selected unsupported rare/Troll conversions remain blocking failures; no subtype filtering or case replacement. Already-selected rare-rock regressions also remain required in rock-blast.',
                      'cases': retained + random_cases(args.count)}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
