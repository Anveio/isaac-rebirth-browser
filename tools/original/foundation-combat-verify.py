#!/usr/bin/env python3
"""Verify conditional combat kernels against retained or fresh native evidence.

Expected gameplay values come from original read-only checkpoints. This does
not establish room routing, enemy AI, death callbacks or browser integration.
All float comparisons use exact IEEE-754 f32 bits, without a numeric tolerance.
"""
import argparse
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2]
REFERENCES = ROOT / 'references/isaac/foundation/combat'
ORIGINAL_SHA = 'c1b2ca9375d60d4354f9086871913901428af41b82a5892a0d235c5d8ae2f768'
NATIVE_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
REQUIRED = {'rock-typed-blast', 'rock-dense-blast', 'rock-held-movement',
    'rock-dense-movement', 'attackfly-mass-contact', 'fly-aligned-queue',
    'fly-bomb-phase0-queue', 'fly-bomb-phase1-queue'}
spec = importlib.util.spec_from_file_location('combat_seed_codec',
    ROOT / 'tools/original/foundation-seeds.py')
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)


def canonical_sha(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
        allow_nan=False).encode()).hexdigest()


def integer(value, low, high, context):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f'{context}: requires an integer in {low}..{high}')
    return value


def compare(expected, actual, path='result'):
    """Compare selected logical fields; additional diagnostic fields are allowed."""
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise ValueError(f'{path}: missing object')
        for key, value in expected.items():
            if key not in actual:
                raise ValueError(f'{path}.{key}: missing field')
            compare(value, actual[key], f'{path}.{key}')
    elif isinstance(expected, list):
        if not isinstance(actual, list) or len(expected) != len(actual):
            raise ValueError(f'{path}: incomplete sequence')
        for index, (left, right) in enumerate(zip(expected, actual)):
            compare(left, right, f'{path}[{index}]')
    elif type(expected) is float:
        if type(actual) not in (int, float) or not math.isfinite(expected) \
                or not math.isfinite(actual):
            raise ValueError(f'{path}: requires finite f32 values')
        try:
            equal = struct.pack('<f', expected) == struct.pack('<f', actual)
        except (OverflowError, struct.error) as error:
            raise ValueError(f'{path}: f32 value overflowed') from error
        if not equal:
            raise ValueError(f'{path}: native f32 {expected!r} != kernel {actual!r}')
    elif type(expected) is not type(actual) or expected != actual:
        raise ValueError(f'{path}: native {expected!r} != kernel {actual!r}')


def extract_proof(manifest, case, manifest_sha):
    counters = {entry['counter'] for entry in case.get('observations', [])}
    clock = case['request'].get('clock')
    if clock:
        counters.add(clock['systemCounter'])
    labels = set(case['provenance']['captureLabels'])
    captures = [entry for entry in manifest['debugObservations']
        if entry['context']['label'] in labels or
        entry['observation'].get('nativePhase', {}).get('systemIterations') in counters]
    return {'name': case['name'], 'nativeManifestSha256': manifest_sha,
        'status': manifest['status'], 'seed': manifest['seed'], 'seedValue': manifest['seedValue'],
        'profile': manifest['profile'], 'seedReadbackMatchesCodec': manifest['seedReadbackMatchesCodec'],
        'tape': manifest['tape'], 'inputs': manifest['inputs'],
        'inputDelivery': manifest['inputDelivery'], 'captures': captures}


def grid_descriptor(observation):
    width, height = observation['room']['gridWidth'], observation['room']['gridHeight']
    integer(width, 1, 28, 'native grid width')
    integer(height, 1, 16, 'native grid height')
    if width * height > 448:
        raise ValueError('Native grid exceeds source pathfinder bound')
    cells = [{'selector': 0, 'entity': None} for _ in range(width * height)]
    # Fixtures deliberately normalize uncaptured empty-cell navigation costs to
    # zero. Existing grid objects supply the actual collision classes instead.
    for entity in observation['grid']:
        index = integer(entity['index'], 0, len(cells) - 1, 'native grid index')
        if cells[index]['entity'] is not None:
            raise ValueError('Duplicate native grid object')
        cells[index]['entity'] = {'kind': entity['type'], 'variant': entity['variant'],
            'state': entity['state'], 'collisionClass': entity['collisionClass'],
            'spawnSeed': entity['spawnSeed']}
    return {'width': width, 'height': height, 'cells': cells}


def vector(value):
    if not isinstance(value, list) or len(value) != 2:
        raise ValueError('Incomplete native vector')
    return {'x': value[0], 'y': value[1]}


def player(observation):
    if len(observation['players']) != 1:
        raise ValueError('Contract requires one original player')
    result = observation['players'][0]
    if result['type'] != 1:
        raise ValueError('Native player type mismatch')
    return result


def actor(observation, sequence_id):
    matches = [entity for entity in observation['entities']
        if entity['sequenceId'] == sequence_id]
    if len(matches) != 1:
        raise ValueError('Requires one retained native actor sequence')
    return matches[0]


def body(entity):
    return {'id': entity['sequenceId'], 'entityType': entity['type'],
        'position': vector(entity['position']), 'velocity': vector(entity['velocity']),
        'radius': entity['collisionRadius'], 'mass': entity['collisionMass']}


def controls_at(proof, presented_frame):
    held = set()
    previous = 0
    for event in proof['inputs']:
        frame = integer(event['presentedFrame'], previous, 36000, 'native input presentation')
        previous = frame
        if frame >= presented_frame:
            break
        if type(event['down']) is not bool:
            raise ValueError('Native input requires an explicit press/release edge')
        if event['down']:
            held.add(event['key'])
        else:
            held.discard(event['key'])
    return {'x': int('d' in held) - int('a' in held),
        'y': int('s' in held) - int('w' in held)}


def validate_case(case, proof, require_fixture_hash=True):
    provenance, request = case['provenance'], case['request']
    if proof['name'] != case['name'] or proof.get('status') != 'pass' \
            or proof.get('profile') != {'fresh': True, 'isolated': True} \
            or proof.get('seedReadbackMatchesCodec') is not True:
        raise ValueError('Requires a successful fresh isolated original run with seed readback')
    if proof['seed'] != provenance['seed'] or proof['seedValue'] != provenance['seedValue'] \
            or codec.decode_seed(proof['seed']) != proof['seedValue']:
        raise ValueError('Native seed/provenance mismatch')
    if proof.get('inputDelivery') != {'api': 'public X11 key events', 'serverAutoRepeat': False}:
        raise ValueError('Requires explicit public X11 input delivery without auto-repeat')
    if provenance.get('freshProfile') is not True or provenance.get('memoryWritten') is not False \
            or provenance.get('executableMappingsRead') is not False:
        raise ValueError('Requires explicit fresh read-only provenance')
    if require_fixture_hash and (proof['nativeManifestSha256'] != provenance['nativeManifestSha256']
            or canonical_sha(proof) != provenance.get('nativeProofSha256')):
        raise ValueError('Retained original proof integrity mismatch')
    by_counter, by_label, counter_frames = {}, {}, {}
    for checkpoint in proof['captures']:
        context, observation = checkpoint['context'], checkpoint['observation']
        if context.get('gateBlocked') is not True or context.get('seedValue') != proof['seedValue'] \
                or observation.get('status') != 'pass' or observation.get('schema') != 2 \
                or observation.get('memoryWritten') is not False \
                or observation.get('executableMappingsRead') is not False \
                or observation.get('nativeBinarySha256') != NATIVE_SHA:
            raise ValueError('Requires source-hashed bounded read-only blocked checkpoint')
        if context['label'] != observation['checkpoint'] or context['label'] in by_label:
            raise ValueError('Duplicate or mismatched native checkpoint label')
        integer(context['presentedFrame'], 1, 36000, 'native presentation')
        counter = integer(observation['nativePhase']['systemIterations'], 1, 0xffffffff,
            'native system counter')
        if counter in by_counter:
            # Two captures can share one blocked presentation (for example the
            # final coast and the following pre-bomb label). Their data must agree.
            for field in ('nativePhase', 'room', 'grid', 'gridSelectors', 'entities', 'players'):
                compare(by_counter[counter][field], observation[field], f'source.samePhase.{field}')
        by_counter[counter] = observation
        counter_frames[counter] = context['presentedFrame']
        by_label[context['label']] = observation
    if not set(provenance['captureLabels']).issubset(by_label):
        raise ValueError('Required native checkpoint is missing')
    initial = by_label[provenance['captureLabels'][0]]
    if 'grid' in request:
        compare(request['grid'], grid_descriptor(initial), 'request.grid')
    if 'geometry' in request:
        original_player = player(initial)
        compare(request['geometry'], {'radius': original_player['collisionRadius'],
            'scale': vector(original_player['collisionScale']),
            'points': len(original_player['gridCollisionSamples'])}, 'request.geometry')
    operation = request['operation']
    if operation == 'gridBlast':
        bomb = next((entity for entity in initial['entities'] if entity['type'] == 4), None)
        if bomb is None or bomb['bombExplosionDamage'] != 60.0 or request['radius'] != 75.0:
            raise ValueError('Requires observed regular bomb and independently recovered radius tier')
        compare(request['origin'], vector(bomb['position']), 'request.origin')
        after = by_label[provenance['captureLabels'][-1]]
        before_rocks = {entry['index'] for entry in initial['grid']
            if entry['type'] == 2 and entry['state'] == 1 and entry['collisionClass'] == 3}
        after_rocks = {entry['index'] for entry in after['grid']
            if entry['type'] == 2 and entry['state'] == 2 and entry['collisionClass'] == 0}
        compare(case['expectedDestroyedRocks'], sorted(before_rocks & after_rocks), 'source.rocks')
    elif operation == 'gridTrajectory':
        start = by_counter[request['clock']['systemCounter']]
        compare(request['position'], vector(player(start)['position']), 'request.position')
        compare(request['velocity'], vector(player(start)['velocity']), 'request.velocity')
        if len(case['observations']) != len(request['inputs']):
            raise ValueError('Grid trajectory requires every original presentation checkpoint')
        start_frame = counter_frames[request['clock']['systemCounter']]
        compare(request['inputs'], [controls_at(proof, start_frame + index + 1)
            for index in range(len(request['inputs']))], 'request.inputs')
        for expected in case['observations']:
            original_player = player(by_counter[expected['counter']])
            compare(expected, {'counter': expected['counter'],
                'position': vector(original_player['position']), 'velocity': vector(original_player['velocity']),
                'collidedWithGrid': original_player['collidedWithGrid'],
                'normal': vector(original_player['gridCollisionNormal'])}, 'source.trajectory')
    elif operation == 'massContact':
        after_checkpoint = next(entry for entry in proof['captures']
            if entry['context']['label'] == provenance['captureLabels'][-1])
        compare(request['control'], controls_at(proof, after_checkpoint['context']['presentedFrame']),
            'request.control')
        compare(request['player'], body(player(initial)), 'request.player')
        compare(request['npc'], body(actor(initial, request['npc']['id'])), 'request.npc')
        after = by_label[provenance['captureLabels'][-1]]
        compare(case['expected']['player'], body(player(after)), 'source.player')
        compare(case['expected']['npc'], body(actor(after, request['npc']['id'])), 'source.npc')
        if player(after)['redHp'] != player(initial)['redHp'] - 1 \
                or player(after)['damageCountdown'] != 60:
            raise ValueError('Source contact lacks actual one-half-heart damage/countdown')
    elif operation == 'deferredDamage':
        target = provenance['targetSequenceId']
        original_actor = actor(initial, target)
        compare(request['health'], {'hp': original_actor['hp'], 'maxHp': original_actor['maxHp']},
            'request.health')
        for expected in case['observations']:
            entity = actor(by_counter[expected['counter']], target)
            compare(expected, {'counter': expected['counter'], 'hp': entity['hp'],
                'pendingDamage': entity['pendingDamage']}, 'source.damage')
        for admission in request['admissions']:
            observed = by_counter[admission['counter']]
            event = admission['event']
            source = actor(observed, event['source']['id'])
            if source['type'] != event['source']['entityType']:
                raise ValueError('Native damage source type mismatch')
            compare(event['amount'], actor(observed, target)['pendingDamage'], 'source.admission')
    else:
        raise ValueError('Unsupported source combat contract')


def run_kernel(kernel, request):
    process = subprocess.run([str(kernel.resolve())], input=json.dumps(request, allow_nan=False),
        text=True, capture_output=True, check=True, timeout=20)
    result = json.loads(process.stdout)
    if result.get('schema') != 1 or not result.get('scope', '').startswith('conditional source kernels;'):
        raise ValueError('Diagnostic lacks its conditional contract envelope')
    return result['result']


def verify_case(case, proof, kernel, fresh=False):
    validate_case(case, proof, not fresh)
    actual = run_kernel(kernel, case['request'])
    operation = case['request']['operation']
    if operation == 'gridBlast':
        compare(case['expectedDestroyedRocks'], actual['rockCandidates'], 'kernel.rocks')
        checkpoints = 2
    elif operation == 'massContact':
        compare(case['expected'], actual, 'kernel.contact')
        checkpoints = 2
    else:
        frames = actual['frames']
        expected_count = 1 + (len(case['request']['inputs']) if operation == 'gridTrajectory'
            else case['request']['presentations'])
        if len(frames) != expected_count or len({frame['counter'] for frame in frames}) != len(frames):
            raise ValueError('Diagnostic emitted incomplete/duplicate timeline')
        by_counter = {frame['counter']: frame for frame in frames}
        for expected in case['observations']:
            if expected['counter'] not in by_counter:
                raise ValueError('Diagnostic omitted an original checkpoint')
            compare(expected, by_counter[expected['counter']], 'kernel.timeline')
        checkpoints = len(case['observations'])
    return {'name': case['name'], 'status': 'pass', 'operation': operation,
        'originalCheckpoints': checkpoints, 'comparison': 'exact f32 bits', 'freshNative': fresh}


def verify_all(document, proofs, kernel, overrides=None):
    overrides = overrides or {}
    source = document.get('source', {})
    errors = []
    if document.get('schema') != 1 or source.get('originalSha256') != ORIGINAL_SHA \
            or source.get('offlineDerivedSha256') != NATIVE_SHA:
        errors.append('Missing pinned original source identity')
    cases, seen, reports = document.get('cases', []), set(), []
    unknown_overrides = set(overrides) - {case['provenance']['scenario'] for case in cases}
    if unknown_overrides:
        errors.append('Unregistered native override scenarios: ' + ', '.join(sorted(unknown_overrides)))
    missing = REQUIRED - {case.get('name') for case in cases}
    if missing:
        errors.append('Missing required original cases: ' + ', '.join(sorted(missing)))
    for case in cases:
        name = case.get('name')
        try:
            if name in seen or name not in REQUIRED:
                raise ValueError('Duplicate or unregistered original case')
            seen.add(name)
            fresh = case['provenance']['scenario'] in overrides
            if fresh:
                path = overrides[case['provenance']['scenario']]
                raw = path.read_bytes()
                proof = extract_proof(json.loads(raw), case, hashlib.sha256(raw).hexdigest())
            else:
                proof = proofs[name]
            reports.append(verify_case(case, proof, kernel, fresh))
        except (ValueError, KeyError, TypeError, OSError, subprocess.SubprocessError) as error:
            reports.append({'name': name, 'status': 'fail', 'error': str(error)})
    return {'schema': 1, 'status': 'fail' if errors or any(case['status'] != 'pass' for case in reports)
        else 'pass', 'scope': document.get('scope'), 'integrityErrors': errors, 'cases': reports}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixtures', type=Path, default=REFERENCES / 'fixtures.json')
    parser.add_argument('--proofs', type=Path, default=REFERENCES / 'native-observations.json')
    parser.add_argument('--kernel', type=Path, default=ROOT / 'engine/target/debug/combat')
    parser.add_argument('--native-manifest', action='append', default=[], metavar='SCENARIO=PATH')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    overrides = {}
    for value in args.native_manifest:
        name, separator, path = value.partition('=')
        if not separator or name in overrides:
            parser.error('Each native manifest requires a unique SCENARIO=PATH')
        overrides[name] = Path(path)
    document = json.loads(args.fixtures.read_text())
    archive = json.loads(args.proofs.read_text())
    proofs = {entry['name']: entry for entry in archive['cases']}
    report = verify_all(document, proofs, args.kernel, overrides)
    if len(proofs) != len(archive['cases']) or archive.get('schema') != 1 \
            or archive.get('source') != document.get('source'):
        report['status'] = 'fail'
        report['integrityErrors'].append('Duplicate/invalid retained proof archive')
    rendered = json.dumps(report, indent=2) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered)
    print(rendered, end='')
    return int(report['status'] != 'pass')


if __name__ == '__main__':
    raise SystemExit(main())
