#!/usr/bin/env python3
"""Public natural coin-room tapes and recorded OS-random fresh-floor discovery."""
import argparse
import importlib.util
import json
from pathlib import Path
import secrets
import subprocess

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('clock_capture', ROOT / 'tools/original/foundation-clock-run.py')
capture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)


def tape(code, delay):
    actions = capture.default_tape(code)
    end = next(i for i, action in enumerate(actions) if action.get('label') == 'gameplay-start')
    actions = [action for action in actions[:end + 1]
               if action['type'] != 'capture' or action['label'] == 'gameplay-start']
    # A natural down-neighbor coin at authored (6,2) is approached only after
    # Appear finishes. An extra idle phase crosses contact in the other phase.
    for key, frames in [('s', 17), (None, 100 + delay), ('s', 25), (None, 30)]:
        if key:
            actions.append({'type': 'key', 'key': key, 'down': True})
        actions.extend({'type': 'step', 'frames': 1} for _ in range(frames))
        if key:
            actions.append({'type': 'key', 'key': key, 'down': False})
    actions.append({'type': 'capture', 'label': 'pickup-end'})
    return actions


def random_cases(count):
    decoder_spec = importlib.util.spec_from_file_location('room_decoder', ROOT / 'tools/foundation/decode-rooms.py')
    decoder = importlib.util.module_from_spec(decoder_spec)
    decoder_spec.loader.exec_module(decoder)
    # The same complete authored STB parser supplies discovery metadata; only
    # public seed text and keys reach either game, never these selectors.
    assets = decoder.decode(ROOT / 'references/isaac/rebirth/resources/rooms/01.basement.stb')
    found = []
    tried = set()
    for attempt in range(1, 5001):
        run_seed = secrets.randbelow(0xffffffff) + 1
        if run_seed in tried:
            continue
        tried.add(run_seed)
        stage_seed = capture.codec.initial_stage_seed(run_seed)
        result = subprocess.run([str(ROOT / 'engine/target/debug/floor')],
            input=json.dumps({'stageSeed': stage_seed}), capture_output=True, text=True, check=True)
        floor = json.loads(result.stdout)
        index = floor['descriptors'][floor['startingDescriptor']]['neighbors'][3]
        if index is None:
            continue
        descriptor = floor['descriptors'][index]
        template = descriptor['template']
        if descriptor['assetStage'] != 1 or template['type'] != 1 or (template['width'], template['height']) != (13, 7):
            continue
        room = assets[descriptor['catalogIndex']]
        if not any(group['x'] == 6 and group['y'] == 2 and len(group['entries']) == 1
                   and group['entries'][0]['type'] == 5 and group['entries'][0]['variant'] == 20
                   for group in room['spawns']):
            continue
        # This coin gate covers inert normal-rock/decoration rooms. Hazard,
        # fire/NPC placeholder, pit and other pickup selectors require their
        # own runtime gates. 22PFCSFS/Room279's spike HP failure is retained.
        if any(not (entry['type'] in (0, 1000, 1490)
                    or entry['type'] == 5 and entry['variant'] == 20 and 0 <= entry['subtype'] <= 4)
               for group in room['spawns'] for entry in group['entries']):
            continue
        found.append({'name': f'random-{len(found)}', 'code': capture.codec.encode_seed(run_seed),
                      'seedValue': run_seed, 'delay': len(found) % 2,
                      'entropy': 'OS secrets', 'discoveryAttempts': attempt})
        if len(found) == count:
            return found
    raise RuntimeError(f'Only {len(found)} natural coin-room seeds found in 5000 recorded OS-random candidates')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--seed')
    parser.add_argument('--delay', type=int, choices=[0, 1], default=0)
    args = parser.parse_args()
    if args.seed:
        capture.codec.decode_seed(args.seed)
        print(json.dumps(tape(args.seed, args.delay)))
        return
    if not 2 <= args.count <= 8:
        raise ValueError('Random coverage must contain 2..8 recorded seeds')
    code = 'PBW9YNC8'
    cases = [{'name': f'contact-phase-{delay}', 'code': code,
              'seedValue': capture.codec.decode_seed(code), 'delay': delay,
              'entropy': 'Contrived phase offset on a naturally generated coin room'}
             for delay in (0, 1)]
    regressions = json.loads((ROOT / 'references/isaac/foundation/pickups/known-regressions.json').read_text())['cases']
    print(json.dumps({'schema': 1, 'bootEpochSeed': 1700000000,
                      'source': 'Natural authored pickups; public inputs only',
                      'cases': cases + random_cases(args.count) + regressions}))


if __name__ == '__main__':
    main()
