#!/usr/bin/env python3
"""Run native fresh-profile probes on new OS-random valid Rebirth seeds.

All draws are recorded for replay; production behavior must never depend on
this seed bank, captured tile layouts or per-seed output lookup tables.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import importlib.util
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('foundation_seeds', Path(__file__).with_name('foundation-seeds.py'))
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)


def sample_bank(count):
    vectors=[];seen=set()
    while len(vectors)<count:
        vector=codec.random_vector()
        if vector['seedValue'] not in seen:
            seen.add(vector['seedValue']);vectors.append(vector)
    return {'schema':1,'entropy':'OS secrets','vectors':vectors}


def validate_bank(bank, expected_count):
    if not isinstance(bank, dict) or bank.get('schema') != 1 or not isinstance(bank.get('vectors'), list):
        raise ValueError('Unsupported or malformed random seed bank')
    vectors = bank['vectors']
    if not vectors or len(vectors) != expected_count:
        raise ValueError('Seed bank must be nonempty and match the requested count')
    values, codes = set(), set()
    for item in vectors:
        if not isinstance(item, dict) or codec.decode_seed(item.get('code')) != item.get('seedValue'):
            raise ValueError('Replay bank seed code/value mismatch')
        value = item['seedValue']
        code = codec.encode_seed(value)
        if value in values or code in codes:
            raise ValueError('Seed bank must contain unique codes and values')
        values.add(value);codes.add(code)
        if 'expectedStage1Seed' in item and item['expectedStage1Seed'] != codec.initial_stage_seed(value):
            raise ValueError('Replay bank public initial-stage expectation is inconsistent')
    return vectors


def safe_probe(probe, index_and_vector):
    index, vector = index_and_vector
    try:
        return probe(index_and_vector)
    except Exception as error:
        return {'code': vector['code'], 'seedValue': vector['seedValue'], 'status': 'fail',
                'initialState': {}, 'failures': [f'Native probe {index} failed: {error}']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=4)
    parser.add_argument('--jobs', type=int, default=2)
    parser.add_argument('--replay-bank', type=Path, help='Previously recorded random bank; seed values remain test data')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'random-native-{time.time_ns()}')
    args = parser.parse_args()
    if not 1 <= args.count <= 100 or not 1 <= args.jobs <= 4:
        raise SystemExit('Count must be1..100; jobs1..4')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    try:
        bank = json.loads(args.replay_bank.read_text()) if args.replay_bank else sample_bank(args.count)
        vectors = validate_bank(bank, args.count)
    except (ValueError, TypeError, KeyError, OSError) as error:
        report = {'schema': 1, 'status': 'fail', 'tested': 0, 'skipped': 0,
                  'runs': [], 'failures': [str(error)]}
        (out / 'result.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
        return 1
    (out / 'seeds.json').write_text(json.dumps(bank, indent=2)+'\n')

    def probe(index_and_vector):
        index, vector = index_and_vector
        target = out / f'run-{index}'
        result = subprocess.run(['bash', str(ROOT / 'tools/original/with-display.sh'), 'python3',
                                 str(ROOT / 'tools/original/foundation-capture.py'), '--seed', vector['code'],
                                 '--output', str(target)], capture_output=True, text=True)
        path = target / 'manifest.json'
        report = json.loads(path.read_text()) if path.exists() else {'status': 'fail', 'failures': [result.stderr or result.stdout]}
        state = report.get('initialState', {})
        failures = list(report.get('failures', []))
        if state.get('seedValue') != vector['seedValue'] or not report.get('seedReadbackMatchesCodec'):
            failures.append('Original seed readback must equal independently decoded random seed')
        if state.get('stageSeed') != codec.initial_stage_seed(vector['seedValue']):
            failures.append('Original Stage1 seed diverged from public start-RNG secondNext model')
        if state.get('stage') != 1 or state.get('altStage') != 0 or state.get('playerSubtype') != 0 or not isinstance(state.get('rooms'), int) or state.get('rooms', 0) < 2:
            failures.append('Fresh normal Isaac Stage1/profile invariants failed')
        return {'code': vector['code'], 'seedValue': vector['seedValue'],
                'status': 'pass' if result.returncode == 0 and not failures else 'fail',
                'manifest': str(path), 'initialState': state, 'failures': failures}

    with ThreadPoolExecutor(max_workers=args.jobs) as executor:
        results = list(executor.map(lambda item: safe_probe(probe, item), enumerate(vectors)))
    report = {'schema': 1, 'status': 'pass' if len(results) == args.count > 0 and all(run['status'] == 'pass' for run in results) else 'fail',
              'seedBank': str(out / 'seeds.json'), 'runs': results,
              'tested': len(results), 'skipped': 0,
              'scope': 'general seed codec/native initial stage/log readback; full gameplay RNG, tile/floor layout parity unverified'}
    (out / 'result.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
