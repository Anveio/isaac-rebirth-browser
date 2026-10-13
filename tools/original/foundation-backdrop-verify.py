#!/usr/bin/env python3
"""Compare generated Basement backdrops with independent original GL caches.

The room decoration seed is read from the original's bounded writable-data
observer and checked against its actual spawn log. This verifies the backdrop
algorithm given that input; it cannot establish run-to-room seed routing, floor
topology, FX, or complete screenshot parity. Captures never feed production.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]


def load_module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


graphics = load_module('backdrop_graphics', 'foundation-gl-observe.py')
debug = load_module('backdrop_log', 'debug-run.py')
seed_descriptors = load_module('backdrop_seed_descriptors', 'foundation-descriptor-observer.py')


def coordinate_bound(value, scale):
    """One native f32 spacing plus the observer's %.9g serialization error.

    The observed original UVs differ from authored integer source coordinates
    by at most one f32 spacing in the normalized UV domain. They are serialized
    with nine significant decimal digits before conversion back to pixels.
    For source coordinates scale is the observed 512-pixel GL texture extent;
    submitted destination coordinates use scale=1. Zero is represented exactly.
    """
    value = abs(value / scale)
    if value == 0:
        return 0
    f32_spacing = 2 ** (math.floor(math.log2(value)) - 23)
    decimal_rounding = 0.5 * 10 ** (math.floor(math.log10(value)) - 8)
    return scale * (f32_spacing + decimal_rounding)


def close(actual, expected, context, scales=(1, 1, 1, 1)):
    if len(actual) != len(expected):
        raise ValueError(f'{context}: coordinate count differs')
    errors = []
    for value, target, scale in zip(actual, expected, scales):
        if type(value) not in (int, float) or not math.isfinite(value):
            raise ValueError(f'{context}: coordinate is unavailable or nonfinite')
        error = abs(value - target)
        bound = coordinate_bound(target, scale)
        if error > bound:
            raise ValueError(f'{context}: expected {target}, observed {value}; '
                             f'error {error} exceeds f32/serialization bound {bound}')
        errors.append(error)
    return max(errors, default=0)


def compare_quads(observed, expected, patch, columns, context):
    if len(observed) != len(expected):
        raise ValueError(f'{context}: quad count differs')
    maxima = {'sourceCoordinateErrorPixels': 0, 'destinationCoordinateErrorPixels': 0}
    for index, (original, generated) in enumerate(zip(observed, expected)):
        where = f'{context} quad {index}'
        if not original['xySubmitted'] or not original['uvSubmitted']:
            raise ValueError(f'{where}: submitted geometry is unavailable')
        source = generated['sourceRect']
        atlas_x = generated['sourcePatchIndex'] % columns * patch['width'] + source['x']
        atlas_y = generated['sourcePatchIndex'] // columns * patch['height'] + source['y']
        source_error = close(original['sourceRect'], [atlas_x, atlas_y,
              atlas_x + source['width'], atlas_y + source['height']], where + ' source',
              (512, 512, 512, 512))
        maxima['sourceCoordinateErrorPixels'] = max(maxima['sourceCoordinateErrorPixels'], source_error)
        destination = generated['destRect']
        x_values = [xy[0] for xy in original['xy']]
        y_values = [xy[1] for xy in original['xy']]
        destination_error = close([min(x_values), min(y_values), max(x_values), max(y_values)],
              [destination['x'], destination['y'],
               destination['x'] + destination['width'],
               destination['y'] + destination['height']], where + ' destination')
        maxima['destinationCoordinateErrorPixels'] = max(maxima['destinationCoordinateErrorPixels'], destination_error)
        if (original['flipX'], original['flipY']) != (generated['flipX'], generated['flipY']):
            raise ValueError(f'{where}: source mirroring differs')
        if len(original['xy']) != 4 or len(original['sourcePixels']) != 4:
            raise ValueError(f'{where}: requires four submitted vertices')
        source_x = [atlas_x, atlas_x + source['width']]
        source_y = [atlas_y, atlas_y + source['height']]
        if generated['flipX']:
            source_x.reverse()
        if generated['flipY']:
            source_y.reverse()
        # Public native vertex arrays use TL, TR, BL, BR, with submitted
        # triangle indices supplying winding; this is not polygon perimeter order.
        for vertex, (right, bottom) in enumerate([(0, 0), (1, 0), (0, 1), (1, 1)]):
            source_error = close(original['sourcePixels'][vertex],
                [source_x[right], source_y[bottom]], f'{where} source vertex {vertex}', (512, 512))
            destination_error = close(original['xy'][vertex],
                [destination['x'] + right * destination['width'],
                 destination['y'] + bottom * destination['height']], f'{where} destination vertex {vertex}')
            maxima['sourceCoordinateErrorPixels'] = max(maxima['sourceCoordinateErrorPixels'], source_error)
            maxima['destinationCoordinateErrorPixels'] = max(maxima['destinationCoordinateErrorPixels'], destination_error)
    return maxima


def verify(manifest, generator):
    manifest = manifest.resolve()
    native = json.loads(manifest.read_text())
    if native.get('status') != 'pass':
        raise ValueError('Original capture did not pass')
    actual_log = debug.observed_state(
        (manifest.parent / 'profile/data/binding of isaac rebirth/log.txt').read_text())
    if actual_log.get('stage') != 1 or actual_log.get('altStage') != 0:
        raise ValueError('Requires original Basement I stage log')
    if actual_log.get('room') != '1.2(Start Room)':
        raise ValueError('Requires original starting-room template log')
    observations = [entry for entry in native.get('debugObservations', [])
                    if entry['context'].get('label') == 'gameplay-start']
    if len(observations) != 1:
        raise ValueError('Requires exactly one original gameplay-start descriptor observation')
    descriptor = observations[0]['observation']
    if descriptor.get('status') != 'pass' or descriptor.get('memoryWritten') is not False \
            or descriptor.get('executableMappingsRead') is not False:
        raise ValueError('Requires successful read-only descriptor observation')
    candidates = [candidate for candidate in descriptor['seedChainCandidates']
                  if candidate['spawnSeedCandidate'] == actual_log.get('spawnSeed')
                  and candidate['stage35Positions']]
    if len(candidates) != 1:
        raise ValueError('Original logged spawn does not identify one room seed chain')
    decoration_seed = candidates[0]['decorationSeedCandidate']
    room_shifts = seed_descriptors.PUBLIC_SHIFTS[12]
    derived_init = seed_descriptors.inverse(decoration_seed, room_shifts)
    if seed_descriptors.next_state(decoration_seed, room_shifts) != actual_log['spawnSeed'] \
            or seed_descriptors.next_state(actual_log['spawnSeed'], room_shifts) != candidates[0]['followingSeedCandidate'] \
            or derived_init != candidates[0]['derivedRoomInitSeed'] \
            or seed_descriptors.stage_positions(actual_log['stageSeed']).get(derived_init) != candidates[0]['stage35Positions']:
        raise ValueError('Decoration input does not reproduce the original room/stage seed chain')
    process = subprocess.run([str(generator.resolve())],
        input=json.dumps({'decorationSeed': decoration_seed}),
        text=True, capture_output=True, check=True)
    appearance = json.loads(process.stdout)
    observed = graphics.observe(manifest)
    if len(observed['floorAssemblies']) != 1 or len(observed['wallAssemblies']) != 1:
        raise ValueError('Requires exactly one native floor and wall cache assembly')
    maxima = {}
    for name, expected_name in [('floorAssemblies', 'floorQuads'), ('wallAssemblies', 'wallQuads')]:
        if observed[name][0]['textureSize'] != [512, 512]:
            raise ValueError('Requires original padded 512×512 Basement GL texture')
        maxima[name] = compare_quads(observed[name][0]['quads'], appearance[expected_name],
                      appearance['backdropPatchSize'], appearance['backdropPatchColumns'], name)
    return {'manifest': str(manifest), 'status': 'pass', 'seed': actual_log['seed'],
            'seedValue': actual_log['seedValue'], 'decorationSeed': decoration_seed,
            'floorQuadsCompared': 4, 'wallQuadsCompared': 44,
            'maximumCoordinateErrors': maxima,
            'runToRoomSeedRoutingVerified': False, 'fxVerified': False}


def verify_all(manifests, generator):
    cases = []
    manifest_identities, seeds, decoration_seeds = set(), set(), set()
    for path in manifests:
        try:
            identity = hashlib.sha256(path.read_bytes()).hexdigest()
            if identity in manifest_identities:
                raise ValueError('Duplicate original manifest identity')
            manifest_identities.add(identity)
            case = verify(path, generator)
            if case['seedValue'] in seeds or case['decorationSeed'] in decoration_seeds:
                raise ValueError('Each selected original case must have an independent run and decoration seed')
            seeds.add(case['seedValue'])
            decoration_seeds.add(case['decorationSeed'])
            cases.append(case)
        except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
            cases.append({'manifest': str(path.resolve()), 'status': 'fail', 'error': str(error)})
    return {'status': 'fail' if any(case['status'] != 'pass' for case in cases) else 'pass',
            'cases': cases,
            'coordinateBound': 'One f32 ULP in submitted coordinate domain plus %.9g serialization rounding; source UVs scale by original GL texture extent 512',
            'scope': 'Generated backdrop patch selection, crop geometry and mirroring given independently observed room decoration seeds'}


def capture_originals(output, count, jobs):
    """Build observers and capture fresh, independently selected run seeds."""
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    trace = output / 'gl-trace.so'
    subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
        '-Wl,--no-undefined', str(ROOT / 'tools/original/foundation-gl-trace.c'),
        str(ROOT / 'tools/original/foundation-gl-state.c'), '-o', str(trace), '-ldl', '-lm'], check=True)
    runner = load_module('backdrop_clock_runner', 'foundation-clock-run.py')
    vectors, selected = [], set()
    while len(vectors) < count:
        vector = runner.codec.random_vector()
        if vector['seedValue'] not in selected:
            selected.add(vector['seedValue'])
            vectors.append(vector)

    def capture(case):
        index, vector = case
        directory = output / f'case-{index:02d}'
        tape = runner.default_tape(vector['code'])
        checkpoint = next(index for index, action in enumerate(tape)
                          if action.get('label') == 'gameplay-start')
        # The descriptor observer needs the actual spawn log, so startup menu
        # checkpoints are omitted. All public input/presentation steps remain.
        tape = [action for action in tape[:checkpoint + 1]
                if action['type'] != 'capture' or action['label'] == 'gameplay-start']
        tape_path = output / f'case-{index:02d}-tape.json'
        tape_path.write_text(json.dumps(tape) + '\n')
        try:
            command = [str(ROOT / 'tools/original/with-display.sh'), sys.executable,
                str(ROOT / 'tools/original/foundation-clock-run.py'), '--seed', vector['code'],
                '--tape', str(tape_path), '--state-observer',
                str(ROOT / 'tools/original/foundation-descriptor-observer.py'),
                '--gl-trace', str(trace), '--output', str(directory)]
            process = subprocess.Popen(command, text=True, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, start_new_session=True)
            try:
                stdout, stderr = process.communicate(timeout=180)
            except subprocess.TimeoutExpired:
                # The isolated session contains this case's X server,
                # controller and original child. Stop all of them on timeout.
                os.killpg(process.pid, signal.SIGKILL)
                process.communicate()
                raise
            (output / f'case-{index:02d}-capture.log').write_text(stdout + stderr)
        except (OSError, subprocess.SubprocessError) as error:
            # Retain the failed request as failed evidence and allow every
            # independently selected case to finish its own capture attempt.
            directory.mkdir(parents=True, exist_ok=True)
            (directory / 'manifest.json').write_text(json.dumps(
                {'status': 'fail', 'phase': 'original-capture', 'error': str(error)}) + '\n')
        return directory / 'manifest.json'

    with ThreadPoolExecutor(max_workers=jobs) as executor:
        return list(executor.map(capture, enumerate(vectors)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifests', type=Path, nargs='*')
    parser.add_argument('--generator', type=Path, default=ROOT / 'engine/target/debug/backdrop',
                        help='Build with cargo build --manifest-path engine/Cargo.toml --bin backdrop')
    parser.add_argument('--capture', action='store_true', help='Capture fresh independent original seeds first')
    parser.add_argument('--count', type=int, default=2, help='Independent original cases to capture, 2..16')
    parser.add_argument('--jobs', type=int, default=2, help='Parallel original capture processes, 1..4')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'backdrop-native-{time.time_ns()}')
    args = parser.parse_args()
    if not args.manifests and not args.capture:
        parser.error('Provide original manifests or --capture')
    if not 2 <= args.count <= 16 or not 1 <= args.jobs <= 4:
        parser.error('count must be 2..16 and jobs 1..4')
    manifests = list(args.manifests)
    if args.capture:
        try:
            manifests += capture_originals(args.output, args.count, args.jobs)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            print(json.dumps({'status': 'fail', 'error': str(error), 'phase': 'original-capture-setup'}))
            return 1
    report = verify_all(manifests, args.generator)
    if args.capture:
        (args.output / 'verification-report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    sys.exit(main())
