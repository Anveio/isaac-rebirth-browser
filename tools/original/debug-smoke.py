#!/usr/bin/env python3
"""Executable native debug checks; --require-parity makes unavailable guarantees red.

Read result.json for native repeated-seed state, actual pixels, process metrics,
input transport/clock, and explicit missing capabilities. Nothing is skipped.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def checkpoint_contract(first, second, case='movement-fire'):
    """Missing/duplicate checkpoints can never establish repeated pixel equality."""
    labels = [[frame.get('label') for frame in frames] for frames in (first, second)]
    errors = []
    for index, run_labels in enumerate(labels):
        if not run_labels or any(not isinstance(label, str) or not label for label in run_labels):
            errors.append(f'Run {index} must have nonempty named checkpoints')
        elif len(set(run_labels)) != len(run_labels):
            errors.append(f'Run {index} has duplicate checkpoint labels')
    if labels[0] != labels[1]:
        errors.append('Repeated runs must capture the same checkpoint labels in the same order')
    required = {'gameplay-start', 'before-input', 'idle-control', 'seed-entry', 'pause-seed'}
    config = json.loads(Path(__file__).with_name('debug-config.json').read_text())
    if case not in config['cases']:
        errors.append(f'Unknown native capture case: {case}')
    else:
        required.update(item['capture'] for item in config['cases'][case])
    for index, run_labels in enumerate(labels):
        missing = sorted(required.difference(run_labels))
        if missing:
            errors.append(f'Run {index} missing required checkpoints: {missing}')
    return not errors, errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/original-runtime' / f'debug-smoke-{time.time_ns()}')
    parser.add_argument('--case', default='movement-fire')
    parser.add_argument('--seed', help='Known base Rebirth seed; default comes from debug-config.json')
    parser.add_argument('--require-parity', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    runs = []
    tests = []
    for index in range(2):
        target = output / f'original-{index}'
        command = ['bash', str(ROOT / 'tools/original/with-display.sh'), 'python3',
                   str(ROOT / 'tools/original/debug-run.py'), '--case', args.case, '--output', str(target)]
        if args.seed:
            command.extend(['--seed', args.seed])
        result = subprocess.run(command, capture_output=True, text=True)
        report_file = target / 'result.json'
        report = json.loads(report_file.read_text()) if report_file.exists() else {'status': 'fail', 'failures': [result.stderr or result.stdout]}
        runs.append(report)
        tests.append({'id': f'native.seeded-launch-{index}', 'status': 'pass' if result.returncode == 0 and report['status'] == 'pass' else 'fail',
                      'detail': report.get('failures', []), 'evidence': str(report_file)})
    usable = all(item['status'] == 'pass' for item in runs)
    profiles_isolated = usable and runs[0]['profile']['path'] != runs[1]['profile']['path'] and all(item['profile']['fresh'] and item['profile']['isolated'] for item in runs)
    tests.append({'id': 'native.fresh-profile-isolation', 'status': 'pass' if profiles_isolated else 'fail'})
    states_equal = usable and runs[0]['observations']['initialState'] == runs[1]['observations']['initialState']
    tests.append({'id': 'native.known-seed-log-state-repeatability', 'status': 'pass' if states_equal else 'fail',
                  'scope': 'exposed initial seed/player/stage/map/room/spawn fields only'})
    if args.case == 'movement-fire':
        causal_input = usable and all(
            item['observations'].get('actionResponse', {}).get('moveRightDisplacementPixels', 0) > 20
            and item['observations'].get('actionResponse', {}).get('shootRightBluePixelGain', 0) > 20
            and item['observations'].get('actionResponse', {}).get('idleDisplacementPixels', 999) < 8
            for item in runs)
        tests.append({'id': 'native.movement-projectile-pixel-response',
                      'status': 'pass' if causal_input else 'fail',
                      'scope': 'known tutorial-room sprite displacement/projectile pixels versus idle control'})
    first_frames, second_frames = (run.get('frames', []) for run in runs)
    checkpoints_complete, checkpoint_errors = checkpoint_contract(first_frames, second_frames, args.case)
    tests.append({'id': 'native.capture-checkpoints',
                  'status': 'pass' if usable and checkpoints_complete else 'fail',
                  'detail': checkpoint_errors})
    observations = {}
    if usable:
        spec = importlib.util.spec_from_file_location('native_debug_run', ROOT / 'tools/original/debug-run.py')
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
        pixel_repeats = []
        second_by_label = {frame['label']: frame for frame in second_frames}
        pixel_errors = []
        for frame in first_frames:
            if frame['label'] in second_by_label:
                try:
                    pixel_repeats.append({'label': frame['label'], **runner.pixel_diff(
                        frame['path'], second_by_label[frame['label']]['path'])})
                except Exception as error:
                    pixel_errors.append({'label': frame['label'], 'error': str(error)})
        if pixel_errors:
            tests.append({'id': 'native.capture-readable', 'status': 'fail', 'detail': pixel_errors})
        observations['checkpointErrors'] = checkpoint_errors
        observations['pixelErrors'] = pixel_errors
        observations['repeatedPixels'] = pixel_repeats
        observations['performance'] = [run['performance'] for run in runs]
        observations['initialState'] = runs[0]['observations']['initialState']
        observations['pixelEquality'] = (checkpoints_complete and not pixel_errors
                                         and len(pixel_repeats) == len(first_frames) > 0
                                         and all(item['equal'] for item in pixel_repeats))
        observations['pixelEqualityScope'] = 'wall-clock checkpoints; render animation/timing may differ'
    # These checks execute as failures in strict mode. A visual run cannot fulfill them.
    if args.require_parity:
        for requirement in ('simulationTicks', 'fullInternalState', 'pauseAtTick'):
            achieved = usable and all(item['capabilities'].get(requirement) is True for item in runs)
            tests.append({'id': 'native.parity.' + requirement, 'status': 'pass' if achieved else 'fail',
                          'detail': f'Original adapter capability {requirement} is required for deterministic state parity'})
        tests.append({'id': 'native.parity.repeated-pixels',
                      'status': 'pass' if observations.get('pixelEquality') else 'fail',
                      'detail': 'Exact rendered pixels at corresponding checkpoints are required'})
    report = {'schema': 1, 'status': 'pass' if all(test['status'] == 'pass' for test in tests) else 'fail',
              'scope': 'native seeded debug readiness' if not args.require_parity else 'native strict parity prerequisites',
              'tests': tests, 'observations': observations,
              'runs': [str(output / f'original-{index}' / 'result.json') for index in range(2)],
              'capabilities': runs[0].get('capabilities', {}),
              'unavailableGuarantees': runs[0].get('unavailableGuarantees', [])}
    (output / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'scope': report['scope'], 'tests': tests, 'output': str(output)}))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
