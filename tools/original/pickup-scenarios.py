#!/usr/bin/env python3
"""Public natural coin-room tapes and recorded OS-random fresh-floor discovery."""
import argparse
import importlib.util
import json
from pathlib import Path
import secrets
import subprocess
import sys

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



def run_fixture_case(args):
    """Use the existing authored resource front door and public Challenge tape."""
    bank_path = args.fixture_scenarios.resolve()
    bank = json.loads(bank_path.read_text())
    selected = [row for row in bank['cases'] if row['name'] == args.fixture_case]
    if len(selected) != 1 or args.seed or args.audio_trace is None or args.output is None:
        raise ValueError('One declared fixture case, fresh output and actual audio library required')
    case = selected[0]
    audio_setup = case.get('audioSetup')
    volume_args = []
    if audio_setup is not None:
        if audio_setup != {'schema': 1, 'sfxVolume': 1, 'requirePositivePlayback': True}:
            raise ValueError('Explicit authored audible volume-one setup required')
        volume_args = ['--sfx-volume', '1']
    if (case['fixture']['startingItems'] != [] or case['fixture']['challengeId'] != 1
            or case['pickupsGoldenResource']['required'] != [[30, 2], [20, 1], [20, 2], [20, 3], [20, 4]]):
        raise ValueError('Declared free Golden Key/Coin public fixture required')
    spec = importlib.util.spec_from_file_location('pickup_fixture_front_door', ROOT / 'tools/original/composition-run.py')
    fixture = importlib.util.module_from_spec(spec); spec.loader.exec_module(fixture)
    tape = fixture.public_tape(case)
    capture.validate_tape(tape)
    output = args.output.resolve(); output.mkdir(parents=True, exist_ok=False)
    def resource_path(value):
        path = Path(value)
        return (path if path.is_absolute() else ROOT / path).resolve()
    xml = resource_path(case['fixture']['authoredResourcePath'])
    stb = resource_path(case['fixture']['specialRoomsPath'])
    library = args.audio_trace.resolve()
    inputs = [bank_path, xml, stb, library]
    digest = lambda path: __import__('hashlib').sha256(path.read_bytes()).hexdigest()
    before = {str(path): digest(path) for path in inputs}
    if digest(stb) != case['fixture']['specialRoomsSha256']:
        raise ValueError('Authored room input differs from declaration')
    fixture.stage_view(xml, output / 'resource-view', stb)
    tape_path = output / 'declared-tape.json'; tape_path.write_text(json.dumps(tape, indent=2) + '\n')
    old_root, old_argv, old_observe = capture.ROOT, sys.argv, capture.checked_observation
    metadata = dict(scope='Same declared XML/STB, boot epoch and public Challenge inputs as browser. No Native-derived production inputs.',
        case=case, publicTape=str(tape_path), declaredBefore=before, memoryWritten=False,
        bootstrap='Native intro only precedes the existing first-Title comparison boundary; no simulation-tick equivalence inferred')
    sound_windows = {'title', 'profile', 'main', 'challenge-selected', 'challenges', 'gameplay-start'} | {row['label'] for row in case['actions']}
    def observe(observer, pid, context):
        context['pickupResourceSound'] = True
        context['pickupSoundWindow'] = context['label'] in sound_windows
        return old_observe(observer, pid, context)
    capture.ROOT = output / 'resource-view'; capture.checked_observation = observe
    try:
        sys.argv = [str(ROOT / 'tools/original/foundation-clock-run.py'), '--challenge-run',
            '--boot-epoch-seed', str(case['bootEpochSeed']), '--tape', str(tape_path),
            '--observe-every-step', '--state-observer', str(ROOT / 'tools/original/pickup-resource-sound-observer.py'),
            '--audio-trace', str(library), '--audio-sfx-owners', *volume_args, '--output', str(output)]
        metadata['actualClockExit'] = capture.main()
    finally:
        capture.ROOT, sys.argv, capture.checked_observation = old_root, old_argv, old_observe
        metadata['declaredAfter'] = {str(path): digest(path) for path in inputs}
        metadata['declaredBeforeEqualsEnd'] = metadata['declaredAfter'] == before
        (output / 'pickup-fixture-inputs.json').write_text(json.dumps(metadata, indent=2) + '\n')
        if metadata['declaredBeforeEqualsEnd'] is not True:
            raise RuntimeError('Declared pickup inputs changed')
    if metadata['actualClockExit'] not in (0, None):
        raise RuntimeError('Original pickup fixture failed')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--seed')
    parser.add_argument('--delay', type=int, choices=[0, 1], default=0)
    parser.add_argument('--fixture-scenarios', type=Path)
    parser.add_argument('--fixture-case')
    parser.add_argument('--output', type=Path)
    parser.add_argument('--audio-trace', type=Path)
    args = parser.parse_args()
    if args.fixture_scenarios is not None:
        run_fixture_case(args)
        return
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
