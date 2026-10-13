#!/usr/bin/env python3
"""Compare explicit-source Rust physics kernels with portable native evidence.

Component requests supply admitted source inputs. They do not seed the browser
with native memory, captured cursors, or per-run correction values.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / 'references/isaac/foundation/physics'
ORIGINAL_BINARY_SHA256 = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
GRID_BOUNDARY_COORDINATES = {(room, cell) for room in (0, 1, 134) for cell in (0, 1, 447)}


def f32_bits(value):
    return struct.unpack('<I', struct.pack('<f', value))[0]


def read_fixture(name):
    path = FIXTURES / name
    raw = path.read_bytes()
    fixture = json.loads(raw)
    if fixture.get('schema') != 1:
        raise ValueError('Unknown native physics fixture schema')
    return fixture, hashlib.sha256(raw).hexdigest()


def validate_origin(origin):
    for key in ('originalBinarySha256', 'manifestSha256'):
        digest = origin.get(key)
        if not isinstance(digest, str) or len(digest) != 64 or any(c not in '0123456789abcdef' for c in digest):
            raise ValueError('Native physics evidence requires binary and capture hashes')
    if origin['originalBinarySha256'] != ORIGINAL_BINARY_SHA256:
        raise ValueError('Native physics evidence uses a different source binary')
    if origin.get('bootEpochSeed') != 1700000000 or not origin.get('publicTape') or not origin.get('clock'):
        raise ValueError('Native physics evidence lacks the explicit controlled boot/input contract')


def load_source(origin, directory=FIXTURES):
    validate_origin(origin)
    reference = origin.get('sourceCapture', {})
    if not isinstance(reference.get('path'), str) or not reference.get('sha256'):
        raise ValueError('Native evidence requires a retained source capture')
    path = (directory / reference['path']).resolve()
    try:
        path.relative_to(directory.resolve())
    except ValueError as error:
        raise ValueError('Native source path leaves its evidence directory') from error
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != reference['sha256']:
        raise ValueError('Native source capture digest differs')
    source = json.loads(raw)
    if source.get('schema') != 1:
        raise ValueError('Unknown retained source schema')
    metadata = {key: value for key, value in origin.items() if key != 'sourceCapture'}
    if source.get('original') != metadata:
        raise ValueError('Native source capture and derived origin are not paired')
    return source


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def height_offset(height, acceleration):
    if acceleration > f32(.001):
        return height
    bend = max(f32(height + 8.5), 0.)
    return f32(f32(f32(height * .5) - 15.) + f32(bend * bend))


def native_birth_uniforms(state):
    # This reference-only parser validates an admitted component transcript.
    # The diagnostic engine receives no captured MT state or cursor.
    words, index = list(state['words']), state['index']
    if len(words) != 624 or type(index) is not int or not 0 <= index <= 624 or any(type(word) is not int or not 0 <= word <= 0xffffffff for word in words):
        raise ValueError('Malformed native MT observation')
    output = []
    for _consumer in range(5):
        if index == 624:
            for i in range(624):
                y = (words[i] & 0x80000000) | (words[(i + 1) % 624] & 0x7fffffff)
                words[i] = words[(i + 397) % 624] ^ (y >> 1) ^ ((y & 1) * 0x9908b0df)
            index = 0
        value = words[index]
        index += 1
        value ^= value >> 11
        value ^= (value << 7) & 0x9d2c5680
        value ^= (value << 15) & 0xefc60000
        value ^= value >> 18
        output.append(f32(f32(value) / 4294967296.))
    return output


def validate_projectile_case(case, source):
    if source['captureId'] != case['name']:
        raise ValueError('Projectile capture identity differs from its case')
    samples, original = case['samples'], source['samples']
    if len(samples) != len(original) or not samples or source['birth']['presentedFrame'] != samples[0]['presentation']:
        raise ValueError('Projectile sample coverage or birth pairing differs')
    inputs = case['input']
    uniforms = native_birth_uniforms(source['birth']['beforeMt'])
    if f32_bits(inputs['eyeUniform']) != f32_bits(uniforms[0]) or f32_bits(inputs['finalFallingUniform']) != f32_bits(uniforms[4]):
        raise ValueError('Projectile admitted random values differ from the native birth consumers')
    if source['birth']['players']:
        player = source['birth']['players'][0]
        if [inputs['playerWorld'][axis] for axis in ('x', 'y')] != player['position'] or inputs['parameters'] != {key: player[field] for key, field in [('height', 'tearHeight'), ('fallingSpeed', 'fallingSpeed'), ('fallingAcceleration', 'fallingAcceleration')]}:
            raise ValueError('Projectile birth parameters differ from the observed player')
        if inputs['eye'] != ('negative' if player['eye'] == -1 else 'positive'):
            raise ValueError('Projectile eye does not match the native birth')
    birth_tear = original[0]['tear']
    identity = (birth_tear['referenceAddress'], birth_tear['opaqueBaseBytes'][16:24])
    for index, (sample, captured) in enumerate(zip(samples, original)):
        if sample['presentation'] != captured['presentedFrame'] or sample['presentation'] != samples[0]['presentation'] + index:
            raise ValueError('Projectile original/derived presentation pairing differs')
        tear = captured['tear']
        if (tear['referenceAddress'], tear['opaqueBaseBytes'][16:24]) != identity or tear['type'] != 2 or tear['variant'] != 0:
            raise ValueError('Projectile samples change the observed entity identity')
        if tear.get('active', True) is not True or tear.get('dead', False) is not False:
            raise ValueError('Live projectile evidence includes an inactive or dying entity')
        if 'systemPhaseCounter' in captured:
            phase = 'update' if captured['systemPhaseCounter'] % 2 else 'interpolate'
        else:
            # Earlier captures lack the typed system counter. The actual
            # source height transform uniquely distinguishes these live phases.
            actual = height_offset(tear['height'], tear['fallingAcceleration'])
            predicted = height_offset(f32(tear['height'] + f32(tear['fallingSpeed'] * .5)), tear['fallingAcceleration'])
            matches = [name for name, offset in [('update', actual), ('interpolate', predicted)] if f32_bits(offset) == f32_bits(tear['renderOffset'][1])]
            if len(matches) != 1:
                raise ValueError('Native projectile phase is not uniquely observed')
            phase = matches[0]
        if sample['phase'] != phase or (index == 0 and inputs['birthPhase'] != phase):
            raise ValueError('Projectile phase label differs from native data')
        position = tear['position'] if phase == 'update' else tear['previousPosition']
        for field, observed in [('position', position), ('presentedPosition', tear['position'])]:
            if [f32_bits(sample[field][axis]) for axis in ('x', 'y')] != [f32_bits(value) for value in observed]:
                raise ValueError('Projectile coordinate samples are not paired to the original')
        if [f32_bits(inputs['storedVelocity'][axis]) for axis in ('x', 'y')] != [f32_bits(value) for value in tear['velocity']]:
            raise ValueError('Projectile stored velocity differs from the original')
        for field in ('height', 'fallingSpeed', 'fallingAcceleration'):
            if f32_bits(sample[field]) != f32_bits(tear[field]):
                raise ValueError('Projectile height samples are not paired to the original')
        if f32_bits(sample['renderOffsetY']) != f32_bits(tear['renderOffset'][1]):
            raise ValueError('Projectile render sample differs from the original')


def validate_projectile_fixture(fixture, directory=FIXTURES):
    origins, identities = set(), set()
    compared = 0
    for case in fixture['cases']:
        source = load_source(case['original'], directory)
        origin = case['original']['sourceCapture']['sha256']
        identity = source['captureId']
        if origin in origins or identity in identities:
            raise ValueError('Independent projectile cases repeat a source origin')
        origins.add(origin)
        identities.add(identity)
        validate_projectile_case(case, source)
        compared += len(case['samples'])
    if compared != 215 or len(fixture['cases']) != 5:
        raise ValueError('The accepted independent projectile evidence was narrowed')


def validate_mt_evidence(fixture, source):
    if source['captureId'] != 'startup-title-mt' or len(source['checkpoints']) != 2 or len(fixture['checkpoints']) != 2:
        raise ValueError('MT checkpoint coverage differs from the original')
    for checkpoint, original in zip(fixture['checkpoints'], source['checkpoints']):
        context, observed = original['context'], original['observation']
        if checkpoint['sourceLabel'] != context['label'] or checkpoint['presentation'] != context['presentedFrame'] or observed.get('status') != 'pass' or observed.get('memoryWritten') is not False:
            raise ValueError('MT checkpoint and native observation are not paired')
        if checkpoint['mt'] != {'words': observed['mtState'], 'index': observed['mtIndex']} or checkpoint['playerRngSeed'] != observed['stateSeed9688'] or checkpoint['gridSeeds'] != observed['gridSeedSamples']:
            raise ValueError('MT state/seed samples differ from the retained original')
        coordinates = [(cell['roomRecord'], cell['gridIndex']) for cell in checkpoint['gridSeeds']]
        if len(coordinates) != 9 or set(coordinates) != GRID_BOUNDARY_COORDINATES:
            raise ValueError('MT descriptor boundary sample coverage was narrowed')


def invoke(binary, request):
    result = subprocess.run([str(binary)], input=json.dumps(request), text=True,
                            capture_output=True, check=True, timeout=30)
    response = json.loads(result.stdout)
    if response.get('schema') != 1 or response.get('kernel') != request['kernel']:
        raise AssertionError('Physics diagnostic response has the wrong contract')
    return response


def compare_projectiles(binary):
    fixture, digest = read_fixture('native-projectiles.json')
    validate_projectile_fixture(fixture)
    cases, compared = [], 0
    for case in fixture['cases']:
        samples = case['samples']
        request = {'kernel': 'projectile', **case['input'],
                   'steps': [{'phase': sample['phase']} for sample in samples[1:]]}
        response = invoke(binary, request)
        if len(response['states']) != len(samples):
            raise AssertionError('Projectile diagnostic omitted a source phase')
        for actual, expected in zip(response['states'], samples):
            for field in ('position', 'presentedPosition'):
                for axis in ('x', 'y'):
                    if f32_bits(actual[field][axis]) != f32_bits(expected[field][axis]):
                        raise AssertionError(f"{case['name']} phase{expected['presentation']} {field}.{axis}")
            for field in ('height', 'fallingSpeed', 'fallingAcceleration', 'renderOffsetY'):
                if f32_bits(actual[field]) != f32_bits(expected[field]):
                    raise AssertionError(f"{case['name']} phase{expected['presentation']} {field}")
            compared += 1
        cases.append({'name': case['name'], 'seed': case['original']['seed'], 'phases': len(samples), 'status': 'pass'})
    if compared != 215 or len(cases) != 5:
        raise AssertionError('The accepted independent projectile evidence was narrowed')
    return {'status': 'pass', 'fixtureSha256': digest, 'cases': cases, 'exactF32Phases': compared}


def compare_mt(binary):
    fixture, digest = read_fixture('native-mt.json')
    source = load_source(fixture['original'])
    validate_mt_evidence(fixture, source)
    first, title = fixture['checkpoints']
    queries = [{'roomRecord': cell['roomRecord'], 'gridIndex': cell['gridIndex']} for cell in first['gridSeeds']]
    startup = [{'type': 'constructSavedGame', 'samples': queries},
               {'type': 'initializeDescriptorGrid'}, {'type': 'uint32'}]
    cases = []
    for checkpoint, consumers, output_index in [
        (first, startup, 0),
        (title, startup + [{'type': 'clearSavedGame', 'samples': queries} for _ in range(4)]
         + [{'type': 'uint32'}] + [{'type': 'bounded', 'bound': 3} for _ in range(179)], 6),
    ]:
        # Each bulk operation runs the concrete source constructor/Clear loops.
        # No operation advances to a captured cursor or imports a native state.
        response = invoke(binary, {'kernel': 'mt', 'bootEpochSeed': fixture['original']['bootEpochSeed'], 'consumers': consumers})
        if response['state'] != checkpoint['mt']:
            raise AssertionError(f"Complete native MT state differs at {checkpoint['label']}")
        saved = response['outputs'][output_index]
        if saved['playerRngSeed'] != checkpoint['playerRngSeed']:
            raise AssertionError('Saved player RNG seed differs from the original field')
        for actual, expected in zip(saved['gridSeeds'], checkpoint['gridSeeds']):
            if [actual['roomRecord'], actual['gridIndex'], *actual['seeds']] != [expected['roomRecord'], expected['gridIndex'], expected['seed10'], expected['seed18']]:
                raise AssertionError('Source descriptor initialization order differs')
        if len(saved['gridSeeds']) != 9:
            raise AssertionError('Descriptor boundary sample coverage was narrowed')
        cases.append({'label': checkpoint['label'], 'gridPairs': 9, 'completeStateWords': 624, 'status': 'pass'})
    return {'status': 'pass', 'fixtureSha256': digest, 'cases': cases}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=ROOT / 'engine/target/debug/physics')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    report = {'schema': 1, 'status': 'pass', 'scope': 'Exact Rust component kernels from explicit source inputs; browser shared consumer routing is not asserted',
              'projectiles': compare_projectiles(args.binary.resolve()), 'mt': compare_mt(args.binary.resolve())}
    encoded = json.dumps(report, indent=2) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded)
    print(encoded, end='')


if __name__ == '__main__':
    main()
