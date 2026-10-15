#!/usr/bin/env python3
"""Require fresh random-seed original repeats to match every rendered pixel.

This checks the public clock/input/presentation adapter, independently of the
browser port. It does not claim original simulation ticks or hidden state.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import importlib.util
import json
from pathlib import Path
import subprocess
import time
import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('foundation_seeds', Path(__file__).with_name('foundation-seeds.py'))
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)
def load_module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result
runner = load_module('clock_runner', 'foundation-clock-run.py')
random_seeds = load_module('random_seeds', 'foundation-random-seeds.py')
scenarios = load_module('clock_scenarios', 'foundation-clock-scenarios.py')


def validate_run(run):
    failures = []
    if not isinstance(run, dict) or run.get('status') != 'pass':
        return ['Original run must complete successfully: ' + str(run.get('failures', []) if isinstance(run, dict) else 'malformed result')]
    try:
        value = codec.decode_seed(run.get('seed'))
        if value != run.get('seedValue'):
            failures.append('Recorded run seed code/value mismatch')
        state = run.get('observedState')
        if not isinstance(state, dict):
            failures.append('Actual public original log state is required')
        else:
            if codec.decode_seed(state.get('seed')) != value or state.get('seedValue') != value:
                failures.append('Actual original seed readback must match the selected seed')
            # The run's first Level::Init; older records only kept the latest.
            first = lambda key: state.get('initial' + key[0].upper() + key[1:], state.get(key))
            if first('stage') != 1 or first('altStage') != 0 or first('stageSeed') != codec.initial_stage_seed(value):
                failures.append('Actual original Stage1 log readback must match the general public stage-seed model')
            if type(state.get('rooms')) is not int or state['rooms'] < 2 or not isinstance(state.get('room'), str) or not state['room'] or type(state.get('spawnSeed')) is not int:
                failures.append('Original room-generation and spawn log readback is required')
        if run.get('seedReadbackMatchesCodec') is not True or run.get('profile') != {'fresh': True, 'isolated': True}:
            failures.append('Fresh isolated profile and verified actual seed readback are required')
        if run.get('simulationTicks') is not False or run.get('fullInternalState') is not False:
            failures.append('Public presentation evidence must not claim hidden state or simulation ticks')
        tape = run.get('tape')
        runner.validate_tape(tape)
        bootstrap = run.get('bootstrapPresentedFrames')
        interval = run.get('clock', {}).get('stepNs')
        if type(bootstrap) is not int or bootstrap < 0 or type(interval) is not int or not 1 <= interval <= 1000000000:
            raise ValueError('Explicit bootstrap presentation count and virtual clock interval are required')
        frame = bootstrap
        expected_phases = [0, bootstrap]
        expected_captures, expected_inputs = [], []
        for action in tape:
            if action['type'] == 'step':
                frame += action['frames']
                expected_phases.append(frame)
            elif action['type'] == 'capture':
                expected_captures.append((action['label'], frame, frame*interval))
            else:
                expected_inputs.append((action['key'], action['down'], frame, frame*interval))
        captures = run.get('frames')
        if not expected_captures or not isinstance(captures, list):
            raise ValueError('Every selected tape capture must have actual rendered evidence')
        actual = [(f.get('label'), f.get('presentedFrame'), f.get('virtualElapsedNs')) for f in captures]
        if actual != expected_captures:
            failures.append('Actual capture labels/presentation phases/time must match every selected tape capture')
        for capture in captures:
            if capture.get('width') != 960 or capture.get('height') != 540 or capture.get('gateBlocked') is not True or capture.get('simulationTick') is not None or not isinstance(capture.get('path'), str):
                failures.append('Actual 960x540 gate-stable framebuffer evidence is required: '+str(capture.get('label')))
        presentations = run.get('presentations')
        if not isinstance(presentations, list) or [a.get('presentedFrame') for a in presentations] != expected_phases:
            raise ValueError('Acknowledged presentation phases must match every selected clock step')
        consumed = []
        for acknowledgement in presentations:
            if acknowledgement.get('virtualElapsedNs') != acknowledgement['presentedFrame']*interval or acknowledgement.get('inputOverflow') is not False or not isinstance(acknowledgement.get('consumedX11Events'), list):
                failures.append('Complete clock/X11 consumption acknowledgement evidence is required')
            consumed.extend(acknowledgement.get('consumedX11Events', []))
        inputs = run.get('inputs')
        if not isinstance(inputs, list) or [(i.get('key'), i.get('down'), i.get('presentedFrame'), i.get('virtualElapsedNs')) for i in inputs] != expected_inputs:
            raise ValueError('Issued public X11 input evidence must match every selected tape input')
        expected_delivery = []
        for event in inputs:
            if type(event.get('keysym')) is not int or event['keysym'] <= 0:
                failures.append('Public X11 keysym readback is required for every issued key')
            if event['presentedFrame'] < frame:
                phase = event['presentedFrame']+1
                expected_delivery.append({'down': event['down'], 'keysym': event.get('keysym'), 'presentedFramePhase': phase, 'virtualElapsedNs': phase*interval})
        if consumed != expected_delivery:
            failures.append('Original must consume exactly the issued X11 events in the next released presentation phase')
    except (ValueError, TypeError, KeyError, AttributeError) as error:
        failures.append('Missing or malformed original evidence: '+str(error))
    return failures


def compare(first, second):
    failures, rows = [], []
    failures.extend('First: '+failure for failure in validate_run(first))
    failures.extend('Repeat: '+failure for failure in validate_run(second))
    if not isinstance(first, dict) or not isinstance(second, dict):
        return {'status': 'fail', 'failures': failures, 'checkpoints': []}
    if first.get('seedValue') != second.get('seedValue') or first.get('tape') != second.get('tape'):
        failures.append('Repeats must use the exact same seed and public input tape')
    actual_log_matches = isinstance(first.get('observedState'), dict) and bool(first['observedState']) and first['observedState'] == second.get('observedState')
    if not actual_log_matches:
        failures.append('Actual public original log state is missing or diverged')
    def rows_in(run, name):
        value = run.get(name)
        return [row for row in value if isinstance(row, dict)] if isinstance(value, list) else []
    def delivery(run):
        return [(a.get('presentedFrame'), a.get('virtualElapsedNs'), a.get('consumedX11Events')) for a in rows_in(run, 'presentations')]
    actual_delivery_matches = bool(first.get('presentations')) and delivery(first) == delivery(second)
    if not actual_delivery_matches:
        failures.append('Public X11 event consumption or virtual presentation clock is missing or diverged')
    right = {f.get('label'): f for f in rows_in(second, 'frames')}
    for frame in rows_in(first, 'frames'):
        label = frame.get('label')
        other = right.get(label)
        if not other:
            failures.append('Missing repeat checkpoint '+str(label))
            continue
        try:
            if Path(frame['path']).resolve() == Path(other['path']).resolve():
                raise ValueError('Independent original repeats must have separate captured files')
            with Image.open(frame['path']) as a_image, Image.open(other['path']) as b_image:
                a = np.asarray(a_image.convert('RGBA')).astype(np.int16)
                b = np.asarray(b_image.convert('RGBA')).astype(np.int16)
            if a.shape != b.shape or a.shape != (540, 960, 4):
                raise ValueError('Unexpected or mismatching rendered dimensions')
            difference = np.abs(a-b)
            changed = int((difference.max(axis=2) != 0).sum())
            rows.append({'label': label, 'presentedFrame': frame.get('presentedFrame'), 'virtualElapsedNs': frame.get('virtualElapsedNs'),
                         'pixels': 960*540, 'changedPixels': changed, 'meanAbsoluteChannelError': float(difference.mean()),
                         'maxChannelError': int(difference.max()), 'status': 'pass' if changed == 0 else 'fail'})
            if changed:
                failures.append('Rendered repeat pixels diverged: '+str(label))
        except (OSError, ValueError, TypeError, KeyError) as error:
            rows.append({'label': label, 'status': 'fail', 'failure': str(error)})
            failures.append('Unreadable actual rendered checkpoint '+str(label)+': '+str(error))
    if not rows:
        failures.append('At least one rendered checkpoint is required')
    return {'status': 'fail' if failures else 'pass', 'checkpoints': rows, 'failures': failures,
            'publicLogStateMatches': bool(actual_log_matches), 'publicInputConsumptionMatches': bool(actual_delivery_matches)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--jobs', type=int, default=2)
    parser.add_argument('--seed-bank', type=Path, help='Replay previously recorded random vectors')
    parser.add_argument('--scenario', choices=[s['id'] for s in scenarios.SCENARIOS], default='menu-movement-fire', help='Execute and repeat every checkpoint in the selected source scenario')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'clock-repeats-{time.time_ns()}')
    args = parser.parse_args()
    if not 1 <= args.count <= 100 or not 1 <= args.jobs <= 4:
        raise SystemExit('count must be 1..100 and jobs 1..4')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    bank = json.loads(args.seed_bank.read_text()) if args.seed_bank else {'schema': 1, 'entropy': 'OS secrets', 'vectors': [codec.random_vector() for _ in range(args.count)]}
    try:
        vectors = random_seeds.validate_bank(bank, args.count)
    except (ValueError, TypeError, KeyError) as error:
        report = {'schema': 1, 'status': 'fail', 'tested': 0, 'skipped': 0, 'runs': [], 'failures': [str(error)]}
        (out / 'result.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
        return 1
    (out / 'seeds.json').write_text(json.dumps(bank, indent=2) + '\n')
    def run(index_vector):
        index, vector = index_vector
        manifests = []
        for repeat in ('first', 'repeat'):
            target = out / f'run-{index}-{repeat}'
            target.mkdir(parents=True, exist_ok=True)
            selected_tape = target / 'selected-tape.json'
            selected_tape.write_text(json.dumps(scenarios.build_scenario(args.scenario, vector['code']), indent=2)+'\n')
            result = subprocess.run(['bash', str(ROOT / 'tools/original/with-display.sh'), 'python3',
                                     str(ROOT / 'tools/original/foundation-clock-run.py'), '--seed', vector['code'],
                                     '--tape', str(selected_tape),
                                     '--output', str(target)], capture_output=True, text=True)
            manifest = target / 'manifest.json'
            candidate = json.loads(manifest.read_text()) if manifest.exists() else {'status': 'fail', 'failures': [result.stderr or result.stdout]}
            if result.returncode:
                candidate['status'] = 'fail'
                candidate.setdefault('failures', []).append('Native runner exited '+str(result.returncode)+': '+(result.stderr or result.stdout)[-2000:])
            manifests.append(candidate)
        comparison = compare(*manifests)
        return {'seedValue': vector['seedValue'], 'code': vector['code'], **comparison,
                'manifests': [str(out / f'run-{index}-{repeat}/manifest.json') for repeat in ('first', 'repeat')],
                'performance': [m.get('performance') for m in manifests]}
    with ThreadPoolExecutor(max_workers=args.jobs) as executor:
        results = list(executor.map(lambda item: random_seeds.safe_probe(run, item), enumerate(vectors)))
    report = {'schema': 1, 'status': 'pass' if all(r['status'] == 'pass' for r in results) else 'fail',
              'scenario': args.scenario,
              'seedBank': str(out / 'seeds.json'), 'tested': len(results), 'skipped': 0, 'runs': results,
              'scope': 'independent fresh original public input/virtual clock/presented framebuffer repeatability',
              'simulationTicks': False, 'fullInternalState': False}
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'tested': len(results), 'skipped': 0, 'report': str(out / 'result.json'),
                      'failedCases': [{'code': r['code'], 'failures': r['failures']} for r in results if r['status'] != 'pass']}))
    return 0 if report['status'] == 'pass' else 1

if __name__ == '__main__':
    raise SystemExit(main())
