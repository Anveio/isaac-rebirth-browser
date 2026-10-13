#!/usr/bin/env python3
"""Capture original shadow/light targets at controlled public presentation phases.

This uses the existing GL request hook without changing the requested input
intervals: arm a snapshot two presentations before its target boundary. The
hook activates at the next swap and snapshots the following completed frame.
Original gameplay ticks remain unknown. Legacy wall-clock fixtures are untouched.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
POSITIONS = None


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


runner = load_module('stage_mask_clock_runner', Path(__file__).with_name('foundation-clock-run.py'))
graphics = load_module('stage_mask_graphics', Path(__file__).with_name('foundation-gl-observe.py'))


def build_tape(seed, bomb=True, explosion=False):
    initial = runner.default_tape(seed)
    end = next(index for index, action in enumerate(initial) if action.get('label') == 'gameplay-start')
    tape = [action for action in initial[:end] if action['type'] != 'capture']
    if tape[-1] != {'type': 'step', 'frames': 300}:
        raise ValueError('Original startup tape settle changed; review snapshot arming phase')
    tape[-1] = {'type': 'step', 'frames': 298}
    tape += [
        {'type': 'capture', 'label': 'arm-gameplay-start'}, {'type': 'step', 'frames': 2},
        {'type': 'capture', 'label': 'gameplay-start'},
        {'type': 'key', 'key': 'd', 'down': True}, {'type': 'step', 'frames': 27},
        {'type': 'key', 'key': 'd', 'down': False}, {'type': 'step', 'frames': 28},
        {'type': 'capture', 'label': 'arm-movement'}, {'type': 'step', 'frames': 2},
        {'type': 'capture', 'label': 'movement'},
    ]
    if bomb:
        # movement also arms the bomb target. No added idle presentation occurs
        # between the fixed movement endpoint and the planted-bomb input.
        tape += [
            {'type': 'key', 'key': 'e', 'down': True}, {'type': 'step', 'frames': 1},
            {'type': 'key', 'key': 'e', 'down': False}, {'type': 'step', 'frames': 1},
            {'type': 'capture', 'label': 'bomb-planted'},
        ]
        if explosion:
            tape += [
                {'type': 'step', 'frames': 117}, {'type': 'capture', 'label': 'arm-explosion-start'},
                {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'explosion-start'},
                {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'explosion-faded'},
                {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'arm-explosion-middle'},
                {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'explosion-middle'},
                {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'arm-explosion-late'},
                {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'explosion-late'},
                {'type': 'step', 'frames': 2}, {'type': 'capture', 'label': 'explosion-removed'},
            ]
    runner.validate_tape(tape)
    return tape


def observe(pid, context):
    global POSITIONS
    if POSITIONS is None:
        path = Path(os.environ.get('ISAAC_STAGE_MASK_POSITION_OBSERVER',
                                   str(Path(__file__).with_name('position-observer.py'))))
        POSITIONS = load_module('stage_mask_positions', path)
    result = POSITIONS.observe(pid, context)
    if result.get('status') != 'pass':
        return result
    label = context['label']
    target = label[4:] if label.startswith('arm-') else (
        'bomb-planted' if label == 'movement' and os.environ.get('ISAAC_STAGE_MASK_BOMB') == '1'
        else 'explosion-faded' if label == 'explosion-start'
        else 'explosion-removed' if label == 'explosion-late' else None)
    if target:
        request = Path(context['output']) / 'gl-trace/request'
        with request.open('x') as stream:
            stream.write(target + '\n')
        result['graphicsRequest'] = {
            'label': target, 'armedAfterPresentedFrame': context['presentedFrame'],
            'expectedTargetPresentedFrame': context['presentedFrame'] + 2,
            'expectedTargetVirtualElapsedNs': context['virtualElapsedNs'] + 2 * 16667000,
        }
    return result


def dimensions(path):
    data = path.read_bytes()
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError('Requires authored PNG dimensions')
    return struct.unpack_from('>II', data, 16)


def rectangle(quad):
    if not quad['xySubmitted'] or not quad['uvSubmitted']:
        raise ValueError('Required mask primitive has unavailable submitted geometry')
    return [min(xy[0] for xy in quad['xy']), min(xy[1] for xy in quad['xy']),
            max(xy[0] for xy in quad['xy']), max(xy[1] for xy in quad['xy'])]


def position_candidates(observation):
    """Preserve numeric reference evidence without exporting native addresses."""
    result = []
    for candidate in observation.get('candidates', []):
        index = candidate['offset'] // 4
        values = candidate['f32']
        result.append({'worldPairCandidate': values[index:index + 2],
                       'velocityPairCandidate': values[index + 7:index + 9]})
    for candidate in observation.get('matches', []):
        if candidate.get('initial') != [320.0, 400.0] or candidate.get('status') != 'candidate':
            continue
        nearby = {item['relativeBytes']: item['f32'] for item in candidate.get('nearbyF32Candidates', [])}
        result.append({'worldPairCandidate': candidate['f32'],
                       'velocityPairCandidate': [nearby.get(28), nearby.get(32)]})
    return result


def validate_input_phases(native):
    labels = [frame['label'] for frame in native['frames']]
    if len(set(labels)) != len(labels):
        raise ValueError('Controlled checkpoint labels must be unique')
    checkpoints = {frame['label']: frame for frame in native['frames']}
    start = checkpoints['gameplay-start']['presentedFrame']
    if checkpoints['movement']['presentedFrame'] != start + 57:
        raise ValueError('Controlled movement must advance exactly D27/release30 presentations')
    events = [event for phase in native['presentations'] for event in phase.get('consumedX11Events', [])]
    expected = [(100, True, start + 1), (100, False, start + 28)]
    if 'bomb-planted' in checkpoints:
        if checkpoints['bomb-planted']['presentedFrame'] != start + 59:
            raise ValueError('Planted-bomb checkpoint must follow Edown1/Eup1 presentations')
        expected += [(101, True, start + 58), (101, False, start + 59)]
    actual = [(event['keysym'], event['down'], event['presentedFramePhase'])
              for event in events if event['presentedFramePhase'] >= start]
    if actual != expected:
        raise ValueError('Original did not consume exactly the requested X11 inputs at their public phases')
    if any(phase.get('inputOverflow') for phase in native['presentations']):
        raise ValueError('Original input event observations overflowed')
    for label, elapsed in [('explosion-start', 121), ('explosion-faded', 123),
                           ('explosion-middle', 127), ('explosion-late', 131), ('explosion-removed', 133)]:
        if label in checkpoints and checkpoints[label]['presentedFrame'] != start + 57 + elapsed:
            raise ValueError('Explosion checkpoint does not match its requested public bomb-input phase')


def depth_contract(state):
    """The independently observed normal-room mask depth pipeline.

    The native shared renderbuffer changes the rasterizer path even when each
    submitted sprite passes the greater-depth test. Missing attachments cannot
    be treated as an equivalent state merely because the RGBA sizes match.
    """
    expected = {'depthEnabled': True, 'depthFunction': [0x0204], 'depthWriteMask': [1],
                'depthBits': [24], 'depthClearValue': [0], 'depthRange': [0, 1]}
    if not isinstance(state, dict) or any(type(state.get(key)) is not type(value) or state[key] != value
                                         for key, value in expected.items()):
        raise ValueError('Native mask depth contract requires enabled GREATER/write1/depth24/clear0/range0..1')
    if any(isinstance(element, bool) for key, value in expected.items() if isinstance(value, list)
           for element in state[key]):
        raise ValueError('Native mask depth contract requires numeric depth parameters')
    attachment = state.get('depthAttachment', {})
    required = {'available': True, 'objectType': 0x8D41, 'renderbufferParametersAvailable': True,
                'width': 1024, 'height': 1024, 'internalFormat': 0x81A6,
                'depthBits': 24, 'stencilBits': 0, 'samples': 0}
    if not isinstance(attachment, dict) or any(type(attachment.get(key)) is not type(value) or attachment[key] != value
           for key, value in required.items()) \
            or type(attachment.get('framebuffer')) is not int or attachment['framebuffer'] <= 0 \
            or type(attachment.get('object')) is not int or attachment['object'] <= 0:
        raise ValueError('Native mask depth contract requires its actual 1024x1024 DEPTH_COMPONENT24 renderbuffer')
    return {**expected, 'depthAttachment': attachment.copy()}


def extract(manifest, output, require_depth_contract=False):
    native = json.loads(manifest.read_text())
    if native.get('status') != 'pass':
        raise ValueError('Original controlled capture failed')
    validate_input_phases(native)
    observed = graphics.observe(manifest)
    trace = Path(native['graphicsTrace']['path'])
    records = [json.loads(line) for line in trace.read_text().splitlines()]
    ends = {item['frame']: item['monotonicNs'] for item in records if item['op'] == 'frameEnd'}
    start_records = [item for item in records if item['op'] == 'frameStart']
    if len({item['label'] for item in start_records}) != len(start_records):
        raise ValueError('Public GL frame labels must be unique')
    starts = {item['label']: item['frame'] for item in start_records}
    arm_records = [entry['observation']['graphicsRequest'] for entry in native['debugObservations']
                   if 'graphicsRequest' in entry['observation']]
    if len({item['label'] for item in arm_records}) != len(arm_records):
        raise ValueError('GL snapshot arming requests must be unique')
    arms = {item['label']: item for item in arm_records}
    expected = {frame['label'] for frame in native['frames'] if not frame['label'].startswith('arm-')}
    if not {'gameplay-start', 'movement'}.issubset(expected) or set(arms) != expected or set(starts) != expected:
        raise ValueError('Every selected gameplay/movement/mask checkpoint requires an armed complete GL frame')
    numeric = {entry['context']['label']: position_candidates(entry['observation'])
               for entry in native['debugObservations']}
    shadow_size = dimensions(ROOT / 'references/isaac/rebirth/resources/gfx/shadow.png')
    shadow_texture_size = [1 << (value - 1).bit_length() for value in shadow_size]
    cases = []
    output.mkdir(parents=True, exist_ok=True)
    for label, arm in arms.items():
        frames = [frame for frame in observed['frames'] if frame['label'] == label]
        checkpoints = [frame for frame in native['frames'] if frame['label'] == label]
        if len(frames) != 1 or len(checkpoints) != 1:
            raise ValueError(f'{label}: requires one complete GL frame and one blocked checkpoint')
        frame, checkpoint = frames[0], checkpoints[0]
        frame_id = starts[label]
        if checkpoint['presentedFrame'] != arm['expectedTargetPresentedFrame'] \
                or checkpoint['virtualElapsedNs'] != arm['expectedTargetVirtualElapsedNs'] \
                or ends[frame_id] - 1000000000000 != checkpoint['virtualElapsedNs']:
            raise ValueError(f'{label}: GL snapshot does not match the exact requested presentation phase')
        shadow_draws = [draw for draw in frame['draws'] if draw['renderTargetTexture']
                        and draw['textureSize'] == shadow_texture_size]
        light_draws = [draw for draw in frame['draws'] if draw['renderTargetTexture']
                       and any(abs(quad['sourceRect'][0] - 100) < 0.001
                           and abs(quad['sourceRect'][1]) < 0.001
                           and abs(quad['sourceRect'][2] - 400) < 0.001
                           and abs(quad['sourceRect'][3] - 300) < 0.001
                           for quad in draw['quads'] if quad['sourceRect'])]
        targets = {}
        primitives = {}
        depth_states = {}
        for name, draws in [('shadow', shadow_draws), ('lighting', light_draws)]:
            if require_depth_contract:
                depth_states[name] = [depth_contract(draw.get('graphicsState')) for draw in draws]
            ids = {draw['renderTargetTexture'] for draw in draws}
            if len(ids) != 1:
                raise ValueError(f'{label}: cannot identify one original {name} target from authored primitives')
            texture = ids.pop()
            snapshots = [snapshot for snapshot in observed['snapshots']
                         if snapshot.get('frame') == frame_id and snapshot['id'] == texture
                         and snapshot.get('stage') != 'cache-source']
            if len(snapshots) != 1 or (snapshots[0]['width'], snapshots[0]['height']) != (512, 512):
                raise ValueError(f'{label}: missing complete {name} target readback')
            source = Path(snapshots[0]['rawPath'])
            data = source.read_bytes()
            if len(data) != 512 * 512 * 4:
                raise ValueError(f'{label}: incomplete {name} RGBA bytes')
            name_path = f'{label}-{name}.rgba'
            (output / name_path).write_bytes(data)
            targets[name] = {'path': name_path, 'sha256': hashlib.sha256(data).hexdigest(),
                             'texture': texture, 'epoch': snapshots[0]['epoch']}
            primitives[name] = [{'destination': rectangle(quad), 'source': quad['sourceRect'],
                                 'flipX': quad['flipX'], 'flipY': quad['flipY'], 'color': quad['color']}
                                for draw in draws for quad in draw['quads']]
        cases.append({'checkpoint': label, 'presentedFrame': checkpoint['presentedFrame'],
                      'virtualElapsedNs': checkpoint['virtualElapsedNs'], 'graphicsFrame': frame_id,
                      'targets': targets, 'primitives': primitives, 'snapshotRequest': arm,
                      **({'depthContract': depth_states} if require_depth_contract else {}),
                      'numericPositionCandidates': numeric.get(label, [])})
    log = manifest.parent / 'profile/data/binding of isaac rebirth/log.txt'
    shutil.copyfile(log, output / 'original.log')
    report = {'schema': 1, 'status': 'pass', 'seed': native['seed'], 'seedValue': native['seedValue'],
              'initialState': native['initialState'], 'sourceManifest': str(manifest.resolve()),
              'format': 'RGBA8-straight', 'origin': 'top-left-logical', 'width': 512, 'height': 512,
              'cases': cases, 'inputTape': native['tape'], 'inputDelivery': native['inputDelivery'],
              'inputEvents': native['inputs'],
              'presentationAcknowledgements': [{key: value for key, value in phase.items()
                  if key in ['presentedFrame', 'virtualElapsedNs', 'consumedX11Events',
                             'consumedX11FocusEvents', 'inputOverflow']} for phase in native['presentations']],
              'profile': native['profile'], 'clock': native['clock'],
              'simulationTicks': False, 'fullInternalState': False,
              'scope': 'Complete public GL target bytes paired to exact blocked presentation phases and X11 inputs; native gameplay tick delivery remains unknown'}
    (output / 'mask-report.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seed', default='3BMB MFTE')
    parser.add_argument('--output', type=Path, default=ROOT / '.cache/foundation' / f'stage-masks-{time.time_ns()}')
    parser.add_argument('--position-observer', type=Path)
    parser.add_argument('--no-bomb', action='store_true')
    parser.add_argument('--explosion', action='store_true', help='Also capture E+121, E+123, E+127, E+131 and E+133 masks')
    parser.add_argument('--manifest', type=Path, help='Extract an already completed controlled capture')
    parser.add_argument('--require-depth-contract', action='store_true',
                        help='Require the independently observed depth24 renderbuffer/GREATER mask pipeline')
    args = parser.parse_args()
    output = args.output.resolve()
    if args.manifest:
        report = extract(args.manifest.resolve(), output, args.require_depth_contract)
    else:
        runner.codec.decode_seed(args.seed)
        output.mkdir(parents=True, exist_ok=False)
        tape = output / 'tape.json'
        if args.explosion and args.no_bomb:
            raise ValueError('Explosion capture requires bomb placement')
        tape.write_text(json.dumps(build_tape(args.seed, not args.no_bomb, args.explosion), indent=2) + '\n')
        trace = output / 'gl-trace.so'
        subprocess.run(['cc', '-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
            '-Wl,--no-undefined', str(ROOT / 'tools/original/foundation-gl-trace.c'),
            str(ROOT / 'tools/original/foundation-gl-state.c'), '-o', str(trace), '-ldl', '-lm'], check=True)
        environment = os.environ.copy()
        environment['ISAAC_STAGE_MASK_BOMB'] = '0' if args.no_bomb else '1'
        if args.position_observer:
            environment['ISAAC_STAGE_MASK_POSITION_OBSERVER'] = str(args.position_observer.resolve())
        subprocess.run([str(ROOT / 'tools/original/with-display.sh'), os.sys.executable,
            str(ROOT / 'tools/original/foundation-clock-run.py'), '--seed', args.seed, '--tape', str(tape),
            '--state-observer', str(Path(__file__).resolve()), '--gl-trace', str(trace),
            '--output', str(output / 'run')], env=environment, check=True)
        report = extract(output / 'run/manifest.json', output / 'targets', args.require_depth_contract)
    print(json.dumps({'status': report['status'], 'sourceManifest': report['sourceManifest'],
                      'cases': report['cases'], 'simulationTicks': False}))


if __name__ == '__main__':
    main()
