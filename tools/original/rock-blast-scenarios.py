#!/usr/bin/env python3
"""Public bomb/rock routes, with recorded OS-random ordinary-room discovery."""
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


clock = load('rock_clock', 'tools/original/foundation-clock-run.py')


def route(delay, slot=3):
    setup = []
    if slot in (0, 2):
        # Source-validated ordinary side-door lineup from floor traversal.
        setup = [(None, 60), ('w', 27), (None, 30)]
    entry = [('a', 72), ('w', 74), ('d', 72), ('s', 17)][slot]
    approach = 'w' if slot in (0, 2) else 'a'
    retreat = ['a', 'w', 'd', 's'][slot]
    return_key = ['d', 's', 'a', 'w'][slot]
    return [{'key': key, 'frames': frames} for key, frames in
            setup + [entry, (None, (20 if slot == 3 else 40) + delay),
                     (approach, 25), (None, 30), ('e', 1), (retreat, 25),
                     (None, 95), (return_key, 25), (approach, 30), (None, 10)]]


def tape(case):
    actions = clock.default_tape(case['code'])
    end = next(i for i, action in enumerate(actions) if action.get('label') == 'gameplay-start')
    actions = [action for action in actions[:end + 1]
               if action['type'] != 'capture' or action['label'] == 'gameplay-start']
    for action in case['actions']:
        if set(action) != {'key', 'frames'} or action['key'] not in (None, 'w', 'a', 's', 'd', 'e') \
                or not isinstance(action['frames'], int) or not 1 <= action['frames'] <= 200:
            raise ValueError('Rock route accepts only bounded public held-key intervals')
        key, frames = action['key'], action['frames']
        if key:
            actions.append({'type': 'key', 'key': key, 'down': True})
        actions.extend({'type': 'step', 'frames': 1} for _ in range(frames))
        if key:
            actions.append({'type': 'key', 'key': key, 'down': False})
    actions.append({'type': 'capture', 'label': 'rock-end'})
    clock.validate_tape(actions)
    return actions


def next11(state):
    state ^= state >> 2
    state ^= (state << 7) & 0xffffffff
    state ^= state >> 7
    return state & 0xffffffff


def next35(state):
    state ^= state >> 5
    state ^= (state << 9) & 0xffffffff
    state ^= state >> 7
    return state & 0xffffffff


def random_cases(count):
    decoder = load('rock_rooms', 'tools/foundation/decode-rooms.py')
    rooms = decoder.decode(ROOT / 'references/isaac/rebirth/resources/rooms/01.basement.stb')
    # The recorded route approaches the left block of the authored Room32
    # four-block layout. A separate retained case exercises a tinted rock.
    candidates = {i for i, room in enumerate(rooms)
                  if room['type'] == 1 and room['variant'] == 32 and
                  (room['width'], room['height']) == (13, 7) and room['spawns'] and
                  all(entry['type'] == 1000 and entry['variant'] == 0 and entry['subtype'] == 0
                      for group in room['spawns'] for entry in group['entries'])}
    if len(candidates) != 1:
        raise ValueError('Ordinary rock route requires one complete authored Room32 identity')
    catalog_index = candidates.pop()
    occupied = {group['x'] + 1 + (group['y'] + 1) * 15
                for group in rooms[catalog_index]['spawns'] if group['entries']}
    found = []
    seen = set()
    for attempt in range(1, 10001):
        run_seed = secrets.randbelow(0xffffffff) + 1
        if run_seed in seen:
            continue
        seen.add(run_seed)
        result = subprocess.run([str(ROOT / 'engine/target/debug/floor')],
            input=json.dumps({'stageSeed': clock.codec.initial_stage_seed(run_seed)}),
            capture_output=True, text=True, check=True)
        floor = json.loads(result.stdout)
        neighbors = floor['descriptors'][floor['startingDescriptor']]['neighbors'][:4]
        for slot in (3, 1, 2, 0):
            index = neighbors[slot]
            if index is None:
                continue
            descriptor = floor['descriptors'][index]
            if descriptor['assetStage'] != 1 or descriptor['catalogIndex'] != catalog_index:
                continue
            first = next11(descriptor['spawnSeed'])
            second = next11(first)
            tinted = first % 13 + 1 + (second % 7 + 1) * 15
            if tinted in occupied:
                continue
            # Source5ceba5 chooses kind5/6 from each group's own RNG35.
            # Isolate ordinary callbacks using source authored admission,
            # while already-selected explosive tapes remain mandatory below.
            group_seed = second
            ordinary = True
            for group in rooms[catalog_index]['spawns']:
                group_seed = next11(group_seed)
                if group['entries'] and next35(group_seed) % 1001 < 16:
                    ordinary = False
                    break
            if not ordinary:
                continue
            delay = len(found) % 2
            found.append({'name': f'random-rock-{len(found)}', 'code': clock.codec.encode_seed(run_seed),
                          'seedValue': run_seed, 'delay': delay, 'slot': slot,
                          'actions': route(delay, slot), 'entropy': 'OS secrets',
                          'discoveryAttempts': attempt, 'cohort': 'ordinary-rock-only'})
            break
        if len(found) == count:
            return found
    raise RuntimeError(f'Only {len(found)} isolated ordinary rock rooms found in 10000 OS-random candidates')


# Room283 (variant282) seen through the starting room's top door. The bomb
# sits in the bottom notch beside Block(5,4); Isaac walks round to the notch
# above it, 77 native units away and inside the 85-unit blast circle. The
# original rejects both knockback and damage through Room::CheckLine.
BLOCK_ROOM = 283
BLOCK_ROUTE = [('w', 74), (None, 40), ('w', 12), (None, 20), ('a', 15), (None, 20), ('e', 1),
               ('d', 12), (None, 15), ('w', 18), (None, 15), ('a', 15), (None, 120)]


def block_case(name, code, entropy, **extra):
    return {'name': name, 'code': code, 'seedValue': clock.codec.decode_seed(code), 'delay': 0, 'slot': 1,
            'actions': [{'key': key, 'frames': frames} for key, frames in BLOCK_ROUTE],
            'entropy': entropy, 'cohort': 'block-line', **extra}


def random_block_case():
    decoder = load('rock_rooms', 'tools/foundation/decode-rooms.py')
    rooms = decoder.decode(ROOT / 'references/isaac/rebirth/resources/rooms/01.basement.stb')
    occupied = {group['x'] + 1 + (group['y'] + 1) * 15
                for group in rooms[BLOCK_ROOM]['spawns'] if group['entries']}
    for attempt in range(1, 20001):
        run_seed = secrets.randbelow(0xffffffff) + 1
        result = subprocess.run([str(ROOT / 'engine/target/debug/floor')],
            input=json.dumps({'stageSeed': clock.codec.initial_stage_seed(run_seed)}),
            capture_output=True, text=True, check=True)
        floor = json.loads(result.stdout)
        index = floor['descriptors'][floor['startingDescriptor']]['neighbors'][1]
        if index is None:
            continue
        descriptor = floor['descriptors'][index]
        if descriptor['assetStage'] != 1 or descriptor['catalogIndex'] != BLOCK_ROOM:
            continue
        first = next11(descriptor['spawnSeed'])
        second = next11(first)
        if first % 13 + 1 + (second % 7 + 1) * 15 in occupied:
            continue
        # Exclude natural kind5/6 rolls exactly as the ordinary cohort does.
        group_seed, ordinary = second, True
        for group in rooms[BLOCK_ROOM]['spawns']:
            group_seed = next11(group_seed)
            if group['entries'] and next35(group_seed) % 1001 < 16:
                ordinary = False
                break
        if not ordinary:
            continue
        return block_case('random-block-line', clock.codec.encode_seed(run_seed), 'OS secrets',
                          discoveryAttempts=attempt)
    raise RuntimeError('No ordinary Room283 above the start room in 20000 OS-random candidates')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--case', type=Path)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.validate_original:
        verifier = load('rock_provenance', 'tools/original/foundation-clock-verify.py')
        failures = verifier.validate_run(json.loads(args.validate_original.read_text()))
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures}))
        return 1 if failures else 0
    if args.case:
        print(json.dumps(tape(json.loads(args.case.read_text()))))
        return 0
    if not 2 <= args.count <= 4:
        raise ValueError('Rock random coverage requires 2..4 newly recorded OS-random seeds')
    code = 'YQ16XWHR'
    contrived = [{'name': f'rock-phase-{delay}', 'code': code,
                  'seedValue': clock.codec.decode_seed(code), 'delay': delay, 'slot': 0,
                  'actions': route(delay, 0),
                  'entropy': 'Contrived public bomb phase offset on a recorded natural rock room',
                  'cohort': 'ordinary-rock-only'} for delay in (0, 1)]
    regressions = json.loads((ROOT / 'references/isaac/foundation/rock-blast/known-regressions.json').read_text())['cases']
    print(json.dumps({'schema': 1, 'bootEpochSeed': 1700000000,
                      'source': 'Public seed and key events only; authored discovery never enters WASM',
                      'cases': contrived + [block_case('block-line', '286TWPAC',
                                                       'Contrived public route behind a natural Block')]
                      + random_cases(args.count) + [random_block_case()] + regressions}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
