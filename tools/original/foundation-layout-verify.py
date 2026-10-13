#!/usr/bin/env python3
"""Verify supplied-seed layout kernels against independent original floors.

The template contract is independently supplied selector seed + authored
shape/doors to a chosen STB record. Growth compares generated rectangles with
the actual finalized floor, so unresolved special resizing remains a failure.
Neither contract establishes complete run-to-room routing or room traversal.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
REFERENCES = ROOT / 'references/isaac/foundation/room-layout'


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


decoder = load_module('layout_room_assets', ROOT / 'tools/foundation/decode-rooms.py')
rng = load_module('layout_source_rng', ROOT / 'tools/original/foundation-descriptor-observer.py')
codec = load_module('layout_source_seed', ROOT / 'tools/original/foundation-seeds.py')
debug = load_module('layout_source_log', ROOT / 'tools/original/debug-run.py')
observer = load_module('layout_source_observer', ROOT / 'tools/original/foundation-layout-observer.py')


def integer(value, low, high, context):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f'{context}: requires an integer in {low}..{high}')
    return value


def shape(room):
    return {(13, 7): 0, (13, 14): 1, (26, 7): 2, (26, 14): 3}.get(
        (room['width'], room['height']), 4)


def authored_door_mask(room):
    # Native STB loader5d4dec..5d514c; preserve the exact eight-slot mapping.
    coordinates = {(-1, 3): 0, (6, -1): 1, (13, 3): 2, (26, 3): 2,
                   (6, 7): 3, (6, 14): 3, (-1, 10): 4, (19, -1): 5,
                   (13, 10): 6, (26, 10): 6, (19, 7): 7, (19, 14): 7}
    result = 0
    for door in room['doors']:
        if door['enabled']:
            point = (door['x'], door['y'])
            if point not in coordinates:
                raise ValueError(f'Unsupported enabled authored door {point}')
            result |= 1 << coordinates[point]
    return result


def authored_catalogs():
    assets = decoder.base_assets()
    catalogs, source = {}, None
    for stage, (name, data) in enumerate(assets):
        if stage not in (0, 1):
            continue
        rooms = decoder.decode_bytes(data, source=name)
        catalogs[stage] = [{key: room[key] for key in
            ('type', 'variant', 'difficulty', 'name', 'weight', 'width', 'height')}
            | {'doorMask': authored_door_mask(room)} for room in rooms]
        if stage == 1:
            source = {'schema': 1, 'source': name, 'sourceBytes': len(data),
                      'sourceSha256': hashlib.sha256(data).hexdigest(),
                      'catalog': catalogs[stage]}
    if set(catalogs) != {0, 1} or not source['source'].endswith('/01.basement.stb'):
        raise ValueError('Pinned authored Special/Basement catalogues are unavailable')
    return catalogs, source


def supplied_growth_input(stage_seed, additional):
    # Recovered caller contract, not fitted offsets: reset's discarded draw,
    # backdrop Random(1), room-count Random(2), generator-constructor Next.
    state, words = stage_seed, []
    for _ in range(4):
        state = rng.next_state(state, rng.PUBLIC_SHIFTS[35])
        words.append(state)
    if additional != 8 + words[2] % 2:
        raise ValueError('Actual normal Basement room count violates recovered caller prefix')
    return {'generatorSeed': words[3], 'additionalRooms': additional, 'shapeMask': 15}


def starting_log_state(text):
    """Pair the initial room phase even when a retained run later traversed."""
    prefix, started = [], False
    for line in text.splitlines(keepends=True):
        prefix.append(line)
        if 'RNG Start Seed:' in line:
            started = True
        if started and line.startswith('SpawnRNG seed:'):
            return debug.observed_state(''.join(prefix))
    raise ValueError('Original log lacks its initial seeded room/SpawnRNG phase')


def validate_controller(case):
    controller = case['controller']
    context = case['provenance']['originalCheckpoint']['context']
    if controller.get('status') != 'pass' or controller.get('seedReadbackMatchesCodec') is not True \
            or controller.get('profile') != {'fresh': True, 'isolated': True} \
            or controller.get('inputDelivery') != {'api': 'public X11 key events', 'serverAutoRepeat': False}:
        raise ValueError('Requires successful fresh-profile public-input source controller')
    value = codec.decode_seed(case['seed'])
    if codec.decode_seed(context.get('seed')) != value \
            or codec.decode_seed(controller.get('seed')) != value \
            or controller.get('seedValue') != value:
        raise ValueError('Actual controller/context seed text does not match original readback')
    step = integer(controller['clock']['stepNs'], 16_667_000, 16_667_000, 'source clock step')
    frame = integer(controller['bootstrapPresentedFrames'], 30, 30, 'actual source bootstrap')
    if sum(action.get('type') == 'capture' and action.get('label') == 'gameplay-start'
           for action in controller['tape']) != 1:
        raise ValueError('Source tape requires one unique gameplay-start identity')
    if len(controller['presentations']) < 2 \
            or [ack.get('presentedFrame') for ack in controller['presentations'][:2]] != [0, 30]:
        raise ValueError('Actual source gate must retain initial zero and bootstrap acknowledgments')
    expected_inputs, captures = [], []
    for action in controller['tape']:
        if action['type'] == 'step':
            frame += integer(action['frames'], 1, 36_000, 'actual source step')
        elif action['type'] == 'key':
            if not isinstance(action.get('key'), str) or type(action.get('down')) is not bool:
                raise ValueError('Malformed actual public key request')
            expected_inputs.append((action['key'], action['down'], frame))
        elif action['type'] == 'capture':
            captures.append(action['label'])
            if action['label'] == 'gameplay-start':
                break
        else:
            raise ValueError('Unknown source controller action')
    if captures.count('gameplay-start') != 1:
        raise ValueError('Actual source tape does not reach one gameplay-start phase')
    checkpoints = [checkpoint for checkpoint in controller['frames'] if checkpoint['label'] == 'gameplay-start']
    observations = [entry for entry in controller['debugObservations']
                    if entry['context']['label'] == 'gameplay-start']
    if len(checkpoints) != 1 or len(observations) != 1:
        raise ValueError('Actual controller checkpoint/observation identity is duplicated or absent')
    checkpoint = checkpoints[0]
    for key, expected in [('presentedFrame', frame), ('virtualElapsedNs', frame * step)]:
        for source in (checkpoint, context, observations[0]['context']):
            if integer(source.get(key), 1, 1_000_000_000_000, key) != expected:
                raise ValueError('Source checkpoint frame/time does not pair with its actual consumed tape')
    if checkpoint.get('gateBlocked') is not True or context.get('gateBlocked') is not True \
            or checkpoint['x11Focus']['window'] != checkpoint['targetWindow']:
        raise ValueError('Actual source checkpoint was not gated with target X11 focus')
    paired_context = observations[0]['context']
    if any(paired_context.get(key) != value for key, value in context.items()):
        raise ValueError('Retained observation context differs from actual controller observation')
    paired_observation = observations[0]['observation']
    if any(paired_observation.get(key) != value for key, value in case['observation'].items()):
        raise ValueError('Retained descriptor data differs from actual controller observation')
    inputs = controller['inputs'][:len(expected_inputs)]
    if len(inputs) != len(expected_inputs) or [(event['key'], event['down'], event['presentedFrame'])
            for event in inputs] != expected_inputs:
        raise ValueError('Recorded source input sequence does not pair with actual controller tape')
    expected_consumed = [(event['keysym'], event['down'], event['presentedFrame'] + 1,
                          (event['presentedFrame'] + 1) * step) for event in inputs]
    consumed, previous, matched = [], None, 0
    for ack in controller['presentations']:
        at = integer(ack['presentedFrame'], 0, 36_000, 'actual gate ack frame')
        if previous is not None:
            # The controller's initial bootstrap release predates tape steps
            # and records its span in bootstrapPresentedFrames instead.
            span = controller['bootstrapPresentedFrames'] if previous == 0 and at == 30 \
                and 'requestedPresentationFrames' not in ack else ack.get('requestedPresentationFrames')
            if at <= previous or at - previous != integer(span, 1, 36_000, 'gate ack span'):
                raise ValueError('Actual gate acknowledgment sequence is incomplete or reused')
        if integer(ack['virtualElapsedNs'], 0, 1_000_000_000_000, 'gate ack clock') != at * step \
                or ack.get('inputOverflow') is not False:
            raise ValueError('Actual gate ack clock/input-overflow contract fails')
        consumed.extend((event['keysym'], event['down'], event['presentedFramePhase'], event['virtualElapsedNs'])
                        for event in ack['consumedX11Events'])
        previous = at
        if at == frame:
            matched += 1
            break
        if at > frame:
            raise ValueError('Actual gate ack skips the selected source phase')
    if matched != 1 or consumed != expected_consumed:
        raise ValueError('Actual X11 events were not consumed at the paired source phase')


def validate_case(case, catalogs, *, allow_non_labyrinth_curses=False):
    validate_controller(case)
    observation = case['observation']
    provenance = case['provenance']
    context = provenance['originalCheckpoint']['context']
    actual = case['actualLog']
    if observation.get('status') != 'pass' or observation.get('schema') != 2 \
            or observation.get('memoryWritten') is not False \
            or provenance.get('memoryWritten') is not False \
            or provenance.get('nativeBinarySha256') != observer.NATIVE_SHA \
            or observation.get('nativeBinarySha256') != observer.NATIVE_SHA:
        raise ValueError('Requires successful source-hashed read-only descriptor observation')
    if context.get('label') != 'gameplay-start' or context.get('gateBlocked') is not True \
            or observation.get('checkpoint') != 'gameplay-start':
        raise ValueError('Requires actual blocked gameplay-start checkpoint')
    integer(context.get('presentedFrame'), 1, 36_000, 'source presented frame')
    integer(context.get('virtualElapsedNs'), 1, 1_000_000_000_000, 'source elapsed clock')
    if actual != observation.get('actualPublicLog') or actual != context.get('publicLogState'):
        raise ValueError('Retained original log does not pair with the observed descriptor phase')
    if codec.decode_seed(case['seed']) != actual.get('seedValue') \
            or context.get('seedValue') != actual.get('seedValue'):
        raise ValueError('Public seed/readback mismatch')
    if actual.get('stage') != 1 or actual.get('altStage') != 0 \
            or actual.get('room') != '1.2(Start Room)' \
            or observation.get('stage') != 1 or observation.get('altStage') != 0 \
            or (not allow_non_labyrinth_curses and observation.get('curseMask') != 0):
        raise ValueError('Contract requires ordinary Basement I / Start Room / no curse')
    for value in (actual['stage'], observation['stage']):
        integer(value, 1, 1, 'normal stage')
    for value in (actual['altStage'], observation['altStage']):
        integer(value, 0, 0, 'normal alt-stage')
    curse = integer(observation['curseMask'], 0, 127 if allow_non_labyrinth_curses else 0, 'source curse mask')
    if curse & 2:
        raise ValueError('Finalized normal floor contract excludes Labyrinth geometry')
    stage_seed = integer(actual.get('stageSeed'), 1, 0xffffffff, 'actual stage seed')
    if stage_seed != codec.initial_stage_seed(actual['seedValue']):
        raise ValueError('Actual stage seed violates independently recovered startup RNG27')
    if case['growthInput'] != supplied_growth_input(stage_seed, actual.get('rooms')):
        raise ValueError('Supplied generator contract disagrees with source caller prefix')
    grid = observation['grid']
    count = integer(observation['descriptorCount'], 1, 128, 'descriptor count')
    descriptors = observation['descriptors']
    if len(grid) != 169 or len(descriptors) != count:
        raise ValueError('Incomplete original grid/descriptor table')
    for value in grid:
        integer(value, -1, count - 1, 'original grid cell')
    positions = rng.stage_positions(stage_seed)
    for index, descriptor in enumerate(descriptors):
        if integer(descriptor['index'], 0, count - 1, 'descriptor index') != index:
            raise ValueError('Original descriptor table order is inconsistent')
        origin = integer(descriptor['gridIndex'], 0, 168, 'descriptor origin')
        integer(descriptor['doorMask'], 0, 255, 'required door mask')
        if len(descriptor['doorGridIndices']) != 8:
            raise ValueError('Incomplete eight-slot native door destinations')
        for target in descriptor['doorGridIndices']:
            integer(target, -1, 168, 'door destination')
        template = descriptor['template']
        for key, low, high in [('assetStage', 0, 1), ('type', 0, 0xffffffff),
                               ('variant', 0, 0xffffffff), ('width', 1, 26),
                               ('height', 1, 14), ('shapeIndex', 0, 3)]:
            integer(template[key], low, high, f'authored {key}')
        matching = [room for room in catalogs.get(template['assetStage'], [])
                    if (room['type'], room['variant'], room['name'], room['width'], room['height'], shape(room))
                    == (template['type'], template['variant'], template['name'],
                        template['width'], template['height'], template['shapeIndex'])]
        if len(matching) != 1:
            raise ValueError('Native descriptor does not identify one complete authored template')
        cells = [cell for cell, value in enumerate(grid) if value == index]
        x, y = origin % 13, origin // 13
        width, height = template['width'] // 13, template['height'] // 7
        expected_cells = [x + dx + (y + dy) * 13 for dy in range(height) for dx in range(width)]
        if not width or not height or x + width > 13 or y + height > 13 \
                or cells != descriptor['cells'] or cells != expected_cells:
            raise ValueError('Native rectangle cells are incomplete/overlapping/outside grid')
        destinations = [(x - 1, y), (x, y - 1), (x + width, y), (x, y + height),
                        (x - 1, y + 1), (x + 1, y - 1),
                        (x + width, y + 1), (x + 1, y + height)]
        expected_destinations = [dx + dy * 13 if 0 <= dx < 13 and 0 <= dy < 13 else -1
                                 for dx, dy in destinations]
        if descriptor['doorGridIndices'] != expected_destinations:
            raise ValueError('Native door destinations disagree with authored rectangle geometry')
        for key in ('roomInitSeed', 'decorationSeed', 'spawnSeed', 'followingSeed'):
            integer(descriptor[key], 1, 0xffffffff, key)
        state = descriptor['roomInitSeed']
        for key in ('decorationSeed', 'spawnSeed', 'followingSeed'):
            state = rng.next_state(state, rng.PUBLIC_SHIFTS[12])
            if state != descriptor[key]:
                raise ValueError('Native descriptor violates the source RNG12 triple')
        if descriptor['stage35DrawPositions'] != positions.get(descriptor['roomInitSeed'], []):
            raise ValueError('Room init seed is not paired with the actual stage primary stream')
    current = integer(observation['currentGridIndex'], 0, 168, 'current room')
    current_index = grid[current]
    if current != 84 or observation['startingGridIndex'] != 84 or current_index < 0 \
            or observation['currentDescriptorIndex'] != current_index \
            or descriptors[current_index]['spawnSeed'] != actual.get('spawnSeed'):
        raise ValueError('Actual public Start Room/SpawnRNG does not pair with current descriptor')


def run_kernel(generator, request):
    process = subprocess.run([str(generator.resolve())], input=json.dumps(request),
        text=True, capture_output=True, timeout=20, check=True)
    return json.loads(process.stdout)


def verify_case(case, generator, catalogs, contract):
    validate_case(case, catalogs)
    report = {'seed': case['seed'], 'status': 'pass', 'contracts': {}}
    if case.get('sourceManifest'):
        report['sourceManifest'] = case['sourceManifest']
    descriptors = case['observation']['descriptors']
    if contract in ('templates', 'both'):
        ordinary = [descriptor for descriptor in descriptors
                    if descriptor['template']['assetStage'] == 1 and descriptor['template']['type'] == 1]
        if not ordinary:
            raise ValueError('Selected original floor has no ordinary template observations')
        requests = [{'seed': descriptor['roomInitSeed'], 'type': 1,
            'shape': descriptor['template']['shapeIndex'], 'requiredDoors': descriptor['doorMask'],
            'minVariant': 0, 'maxVariant': 0xffffffff, 'minDifficulty': 1, 'maxDifficulty': 5}
            for descriptor in ordinary]
        selected = run_kernel(generator, {'operation': 'templates', 'catalog': catalogs[1], 'selections': requests})
        if len(selected) != len(ordinary):
            raise ValueError('Diagnostic returned an incomplete template sequence')
        failures = []
        for native, chosen in zip(ordinary, selected):
            template = native['template']
            index = integer(chosen['catalogIndex'], 0, len(catalogs[1]) - 1, 'chosen authored index')
            authored = catalogs[1][index]
            expected = (template['type'], template['variant'], template['name'], template['shapeIndex'])
            actual = (chosen['type'], chosen['variant'], chosen['name'], chosen['shape'])
            if actual != expected or (chosen['type'], chosen['variant'], chosen['name'], chosen['doorMask']) \
                    != (authored['type'], authored['variant'], authored['name'], authored['doorMask']):
                failures.append({'gridIndex': native['gridIndex'], 'expected': expected, 'actual': actual})
        report['contracts']['templates'] = {'status': 'fail' if failures else 'pass',
            'observedTemplates': len(ordinary), 'failures': failures,
            'scope': 'independently supplied selector seed/shape/doors; complete authored record identity'}
    if contract in ('growth', 'both'):
        grown = run_kernel(generator, {'operation': 'growth'} | case['growthInput'])
        expected = sorted((descriptor['gridIndex'], descriptor['template']['width'] // 13,
            descriptor['template']['height'] // 7) for descriptor in descriptors if descriptor['template']['type'] != 7)
        actual = sorted((room['origin']['x'] + room['origin']['y'] * 13, room['width'], room['height'])
                        for room in grown['rooms'])
        expected_expansions = case['originalLogText'].count('not enough dead ends')
        matched = actual == expected and grown['expansionAttempts'] == expected_expansions
        report['contracts']['growth'] = {'status': 'pass' if matched else 'fail',
            'expectedRectangleSet': expected, 'actualRectangleSet': actual,
            'expectedExpansionAttempts': expected_expansions,
            'actualExpansionAttempts': grown['expansionAttempts'],
            'knownFinalizationGap': case.get('knownFinalizationGap'),
            'scope': 'strict finalized-floor comparison; special resizing/retries are unresolved'}
    if any(value['status'] != 'pass' for value in report['contracts'].values()):
        report['status'] = 'fail'
    return report


def portable_cases(path):
    fixture = json.loads(path.read_text())
    if fixture.get('schema') != 1 or not 1 <= len(fixture.get('cases', [])) <= 128:
        raise ValueError('Unsupported or empty portable native observation fixture')
    cases = []
    for source in fixture['cases']:
        case = dict(source)
        name = case['provenance']['sourceOriginalLog']
        if Path(name).name != name or not name.endswith('-original.log'):
            raise ValueError('Original log must be a local retained reference file')
        data = (path.parent / name).read_bytes()
        if hashlib.sha256(data).hexdigest() != case['provenance']['sourceOriginalLogSha256']:
            raise ValueError('Retained original log SHA mismatch')
        case['originalLogText'] = data.decode('utf-8')
        case['actualLog'] = starting_log_state(case['originalLogText'])
        controller_name = case['provenance']['sourceController']
        if Path(controller_name).name != controller_name or not controller_name.endswith('-controller.json'):
            raise ValueError('Source controller must be a retained local reference file')
        controller_data = (path.parent / controller_name).read_bytes()
        if hashlib.sha256(controller_data).hexdigest() != case['provenance']['sourceControllerSha256']:
            raise ValueError('Retained actual source controller SHA mismatch')
        case['controller'] = json.loads(controller_data)
        cases.append(case)
    return cases


def captured_case(path):
    manifest = json.loads(path.read_text())
    if manifest.get('status') != 'pass':
        raise ValueError('Independent original capture failed')
    observations = [entry for entry in manifest.get('debugObservations', [])
                    if entry.get('context', {}).get('label') == 'gameplay-start']
    if len(observations) != 1:
        raise ValueError('Requires exactly one actual gameplay-start observation')
    entry = observations[0]
    log = path.with_name('original.log').read_text()
    actual = starting_log_state(log)
    return {'seed': manifest['seed'], 'sourceManifest': str(path),
        'provenance': {'nativeBinarySha256': entry['observation']['nativeBinarySha256'],
            'memoryWritten': entry['observation']['memoryWritten'],
            'originalCheckpoint': {'context': entry['context']}},
        'observation': entry['observation'], 'actualLog': actual, 'controller': manifest,
        'growthInput': supplied_growth_input(actual['stageSeed'], actual['rooms']),
        'originalLogText': log}


def verify_all(cases, generator, catalogs, contract):
    reports, seeds = [], set()
    for case in cases:
        try:
            seed = codec.decode_seed(case['seed'])
            if seed in seeds:
                raise ValueError('Duplicate independent original seed identity')
            seeds.add(seed)
            reports.append(verify_case(case, generator, catalogs, contract))
        except (KeyError, TypeError, ValueError, OSError, subprocess.SubprocessError) as error:
            reports.append({'seed': case.get('seed'), 'status': 'fail', 'error': str(error)})
    return {'status': 'pass' if reports and all(case['status'] == 'pass' for case in reports) else 'fail',
            'contract': contract, 'independentOriginals': len(reports), 'cases': reports}


def capture_originals(output, count, jobs):
    runner = load_module('layout_clock_runner', ROOT / 'tools/original/foundation-clock-run.py')
    vectors, identities = [], set()
    while len(vectors) < count:
        vector = runner.codec.random_vector()
        if vector['seedValue'] not in identities:
            identities.add(vector['seedValue'])
            vectors.append(vector)
    output.mkdir(parents=True, exist_ok=False)
    (output / 'requested-seeds.json').write_text(json.dumps(vectors, indent=2) + '\n')

    def capture(pair):
        index, vector = pair
        tape = []
        for action in runner.default_tape(vector['code']):
            if action['type'] != 'capture' or action['label'] == 'gameplay-start':
                tape.append(action)
            if action['type'] == 'capture' and action['label'] == 'gameplay-start':
                break
        tape_path = output / f'case{index}-tape.json'
        tape_path.write_text(json.dumps(tape, indent=2) + '\n')
        directory = output / f'case{index}'
        with (output / f'case{index}.log').open('w') as log:
            process = subprocess.Popen([str(ROOT / 'tools/original/with-display.sh'), sys.executable,
                str(ROOT / 'tools/original/foundation-clock-run.py'), '--seed', vector['code'],
                '--tape', str(tape_path), '--state-observer', str(ROOT / 'tools/original/foundation-layout-observer.py'),
                '--output', str(directory)], stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                process.wait(timeout=180)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
        if not (directory / 'manifest.json').exists():
            directory.mkdir(parents=True, exist_ok=True)
            (directory / 'manifest.json').write_text(json.dumps({'status': 'fail',
                'seed': vector['code'], 'reason': 'Original did not complete its controlled capture'}) + '\n')
        return directory / 'manifest.json'

    with ThreadPoolExecutor(max_workers=jobs) as executor:
        return list(executor.map(capture, enumerate(vectors, 1)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifests', type=Path, nargs='*')
    parser.add_argument('--fixture', type=Path)
    parser.add_argument('--generator', type=Path, default=ROOT / 'engine/target/debug/layout')
    parser.add_argument('--contract', choices=('templates', 'growth', 'both'), default='templates')
    parser.add_argument('--capture', action='store_true')
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--jobs', type=int, default=2)
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/original-layout' / str(time.time_ns()))
    parser.add_argument('--report', type=Path, default=ROOT / '.cache/original-layout-report.json')
    args = parser.parse_args()
    if not 1 <= args.count <= 16 or not 1 <= args.jobs <= 2:
        parser.error('Capture count must be1..16 and jobs1..2')
    catalogs, source = authored_catalogs()
    retained = json.loads((REFERENCES / 'basement-templates.json').read_text())
    if source != retained:
        raise ValueError('Retained template catalogue differs from complete pinned authored STB')
    paths = list(args.manifests)
    if args.capture:
        paths += capture_originals(args.output, args.count, args.jobs)
    cases, failures = [], []
    if args.fixture or not paths:
        cases += portable_cases(args.fixture or REFERENCES / 'native-observations.json')
    for path in paths:
        try:
            cases.append(captured_case(path))
        except (KeyError, TypeError, ValueError, OSError) as error:
            failures.append({'sourceManifest': str(path), 'status': 'fail', 'error': str(error)})
    report = verify_all(cases, args.generator, catalogs, args.contract)
    report['cases'] += failures
    if failures:
        report['status'] = 'fail'
    report['authoredCatalogue'] = {key: value for key, value in source.items() if key != 'catalog'}
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'status': report['status'], 'contract': args.contract,
        'independentOriginals': report['independentOriginals'], 'report': str(args.report)}))
    return 0 if report['status'] == 'pass' else 1


if __name__ == '__main__':
    sys.exit(main())
