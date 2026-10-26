#!/usr/bin/env python3
"""Compare generated fresh normal Basement floors with independently run originals.

Inputs to the port are only public run/stage seeds and checked authored STBs.
Native layouts, selectors and RNG cursors are acceptance outputs, never inputs.
The report can feed the live browser gate; it keeps bulky capture data in cache.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('floor_source_contracts', ROOT / 'tools/original/foundation-floor-source.py')
source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source)
FIELDS = ('gridIndex', 'roomInitSeed', 'decorationSeed', 'spawnSeed', 'followingSeed', 'doorMask')
TEMPLATE_FIELDS = ('type', 'variant', 'name', 'width', 'height')


def descriptors(values):
    return [{key: room[key] for key in FIELDS}
            | {'assetStage': room.get('assetStage', room['template'].get('assetStage')),
               'template': {key: room['template'][key] for key in TEMPLATE_FIELDS}}
            for room in values]


def capture_selected(output, codes, jobs, hard=False):
    runner = source.load_module('floor_clock_runner', ROOT / 'tools/original/foundation-clock-run.py')
    output.mkdir(parents=True, exist_ok=False)
    vectors = [{'code': code, 'seedValue': source.codec.decode_seed(code)} for code in codes]
    (output / 'requested-seeds.json').write_text(json.dumps(vectors, indent=2) + '\n')

    def capture(pair):
        index, vector = pair
        tape = []
        for action in runner.default_tape(vector['code']):
            if action['type'] != 'capture' or action['label'] == 'gameplay-start':
                tape.append(action)
            if action['type'] == 'capture' and action['label'] == 'gameplay-start':
                break
        if hard:
            # Down on the seeded Character page selects hard before the Return
            # that starts the run.
            start = max(i for i, action in enumerate(tape) if action.get('key') == 'Return' and action['down'])
            tape[start:start] = [{'type': 'key', 'key': 'Down', 'down': True}, {'type': 'step', 'frames': 12},
                                 {'type': 'key', 'key': 'Down', 'down': False}, {'type': 'step', 'frames': 30}]
        tape_path = output / f'case{index}-tape.json'
        tape_path.write_text(json.dumps(tape, indent=2) + '\n')
        directory = output / f'case{index}'
        with (output / f'case{index}.log').open('w') as log:
            subprocess.run([str(ROOT / 'tools/original/with-display.sh'), sys.executable,
                str(ROOT / 'tools/original/foundation-clock-run.py'), '--seed', vector['code'],
                '--tape', str(tape_path), '--state-observer', str(ROOT / 'tools/original/foundation-layout-observer.py'),
                '--output', str(directory)], stdout=log, stderr=subprocess.STDOUT, timeout=180, check=False)
        return directory / 'manifest.json'

    with ThreadPoolExecutor(max_workers=jobs) as executor:
        return list(executor.map(capture, enumerate(vectors, 1)))


def compare(case, generator, catalogs, hard=False):
    source.validate_case(case, catalogs, allow_non_labyrinth_curses=True, hard=hard)
    observed = case['observation']
    public = observed['actualPublicLog']
    request = {'stageSeed': public['stageSeed']} | ({'difficulty': 'hard'} if hard else {})
    process = subprocess.run([str(generator.resolve())], input=json.dumps(request),
        text=True, capture_output=True, timeout=20, check=True)
    predicted = json.loads(process.stdout)
    native_descriptors = descriptors(observed['descriptors'])
    generated_descriptors = descriptors(predicted['descriptors'])
    loops = re.findall(r'^Map Generated in (\d+) Loops$', case['originalLogText'], re.MULTILINE)
    if len(loops) != 1:
        raise ValueError('Initial source floor must retain one actual map-attempt log')
    expected_attempts = int(loops[0])
    differences = []
    for name, expected, actual in [('descriptors', native_descriptors, generated_descriptors),
            ('grid', observed['grid'], predicted['grid']),
            ('startingDescriptor', observed['grid'][84], predicted['startingDescriptor']),
            ('attempts', expected_attempts, predicted['attempts'])]:
        if expected != actual:
            differences.append({'field': name, 'expected': expected, 'actual': actual})
    return {'seed': case['seed'], 'runSeed': public['seedValue'], 'stageSeed': public['stageSeed'], 'sourceCurseMask': observed['curseMask'],
        'bootEpochSeed': case['controller'].get('bootEpochSeed'),
        'status': 'fail' if differences else 'pass', 'differences': differences,
        'expected': {'descriptors': native_descriptors, 'grid': observed['grid'],
                     'startingDescriptor': observed['grid'][84], 'attempts': expected_attempts},
        'sourcePhase': case['provenance']['originalCheckpoint']['context'],
        'nativeBinarySha256': observed['nativeBinarySha256'],
        'sourceOriginalLogSha256': hashlib.sha256(case['originalLogText'].encode()).hexdigest(),
        'sourceManifest': case.get('sourceManifest'),
        'difficulty': 'hard' if hard else 'normal',
        'scope': 'complete ordered floor descriptors/grid/seeds/attempts; fresh Isaac without Labyrinth; other curse effects are separate; pre-RoomInit stream continuation'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifests', type=Path, nargs='*')
    parser.add_argument('--capture', action='store_true')
    parser.add_argument('--count', type=int, default=2, help='Independent OS-random source seeds (0 replays only --seed codes)')
    parser.add_argument('--seed', action='append', default=[], help='Additional public contrived seed code')
    parser.add_argument('--jobs', type=int, default=2)
    parser.add_argument('--hard', action='store_true', help='Start each captured run at hard difficulty (Down on the Character page)')
    parser.add_argument('--generator', type=Path, default=ROOT / 'engine/target/debug/floor')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/original-floor' / str(time.time_ns()))
    parser.add_argument('--report', type=Path, default=ROOT / '.cache/foundation/floor-native.json')
    args = parser.parse_args()
    if not 0 <= args.count <= 16 or not 1 <= args.jobs <= 2:
        parser.error('Random source count must be 0..16 and capture jobs 1..2')
    catalogs, _ = source.authored_catalogs()
    paths = list(args.manifests)
    if args.capture:
        codes = [source.codec.encode_seed(source.codec.decode_seed(code)) for code in args.seed]
        identities = {source.codec.decode_seed(code) for code in codes}
        if len(identities) != len(codes):
            raise ValueError('Contrived source identities must be unique')
        random_codes = []
        while len(random_codes) < args.count:
            vector = source.codec.random_vector()
            if vector['seedValue'] not in identities:
                identities.add(vector['seedValue'])
                random_codes.append(vector['code'])
        paths += capture_selected(args.output, codes + random_codes, args.jobs, args.hard)
    elif args.seed:
        parser.error('--seed requires --capture')
    cases, results = [], []
    if not paths:
        cases = source.portable_cases(source.REFERENCES / 'native-observations.json')
    for path in paths:
        try:
            cases.append(source.captured_case(path, args.hard))
        except (OSError, ValueError, KeyError, TypeError) as error:
            results.append({'status': 'fail', 'sourceManifest': str(path), 'error': str(error)})
    identities = set()
    for case in cases:
        try:
            identity = source.codec.decode_seed(case['seed'])
            if identity in identities:
                raise ValueError('Duplicated independent original identity')
            identities.add(identity)
            results.append(compare(case, args.generator, catalogs, args.hard))
        except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
            results.append({'status': 'fail', 'seed': case.get('seed'), 'error': str(error)})
    report = {'schema': 1, 'status': 'pass' if results and all(case['status'] == 'pass' for case in results) else 'fail',
        'selected': len(cases) + sum('sourceManifest' in case and 'seed' not in case for case in results),
        'skipped': 0, 'randomSourceCount': args.count if args.capture else 0,
        'cases': results, 'contract': 'source-seed-to-finalized-fresh-normal-basement'}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'originals': len(results), 'report': str(args.report)}))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    sys.exit(main())
