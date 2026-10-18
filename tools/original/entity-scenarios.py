#!/usr/bin/env python3
"""Public-input tapes that walk into a room admitting one authored entity.

Contrived cases come from the entity spec (seed code + door slot). Fuzz cases
draw OS-random run seeds and keep those whose starting room has a neighbouring
ordinary room where the port's own floor generator and authored spawn
selection admit the entity. Only the seed text and keys reach either game;
discovery metadata never does. Both games then play the same route: walk
through the door, settle, shoot into the room in every direction, and idle.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


paired = load('entity_paired', 'foundation-paired-scenarios.py')
runner = load('entity_clock', 'foundation-clock-run.py')
codec = runner.codec
GENERATOR = ROOT / 'engine/target/debug/floor'
# Door slots 0..3 are left, up, right, down. Hold frames match the traversal
# route that crosses each ordinary 1x1 starting-room doorway.
DEPART = (('a', 72), ('w', 74), ('d', 72), ('s', 18))
AIM = ('Left', 'Up', 'Right', 'Down')


def key(name, down):
    return {'type': 'key', 'key': name, 'down': down}


def dense(tape, frames, prefix, every=1):
    for index in range(every, frames + 1, every):
        tape += [{'type': 'step', 'frames': every}, {'type': 'capture', 'label': f'{prefix}-{index}'}]


def route(slot, sweep=False):
    tape = [{'type': 'step', 'frames': 60}]
    if slot in (0, 2):
        # Side doorways sit on the room's middle row, above the spawn point.
        tape += [key('w', True), {'type': 'step', 'frames': 27}, key('w', False), {'type': 'step', 'frames': 30}]
    move, frames = DEPART[slot]
    tape.append(key(move, True))
    dense(tape, frames, 'depart')
    tape.append(key(move, False))
    dense(tape, 40, 'entry')
    if sweep:
        # Larger shapes: cross the room along each axis so CameraStyle1 scrolls
        # and both games reach the far boundary planes.
        for leg, move in enumerate(('a', 'd', 'd', 'a', 'w', 's', 's', 'w')):
            tape.append(key(move, True))
            dense(tape, 80, f'sweep{leg}', 2)
            tape.append(key(move, False))
    for turn in range(4):
        aim = AIM[(slot + turn) % 4]
        tape.append(key(aim, True))
        dense(tape, 120 if turn == 0 else 60, f'shoot{turn}', 2)
        tape.append(key(aim, False))
    dense(tape, 90, 'idle', 2)
    return tape


def green_creep_contact_tail():
    # Fixed Green champion route: open the Poop, circle the centre rock, then
    # wait beside the trail. Actual captures must still prove the Acid witness.
    tape = [key('d', True)]
    dense(tape, 27, 'contact-align')
    tape.append(key('d', False))
    dense(tape, 30, 'contact-align-settle', 2)
    tape.append(key('Down', True))
    dense(tape, 80, 'contact-open-poop', 2)
    tape.append(key('Down', False))
    for leg, (move, frames) in enumerate((('d', 23), ('s', 12), ('d', 20),
                                        ('w', 22), ('a', 21), ('s', 14), ('a', 20)), 1):
        tape.append(key(move, True))
        dense(tape, frames, f'contact-kite-leg{leg}')
        tape.append(key(move, False))
    dense(tape, 30, 'contact-kite-coast', 2)
    dense(tape, 140, 'contact-kite-observe', 2)
    return tape


def case(identifier, code, slot, entity, provenance, randomized, shape=(13, 7)):
    result = paired.build('paired-bomb-moving-even', code)
    tail = route(slot, sweep=shape != (13, 7))
    if (not randomized and entity == (10, 1) and code.replace(' ', '') == '0F8NB23G'
            and slot == 2 and shape == (13, 7)):
        entry = next(index for index, action in enumerate(tail) if action.get('label') == 'entry-40')
        tail = tail[:entry + 1] + green_creep_contact_tail()
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
    runner.validate_tape(result['tape'])
    subject = f'entity {entity[0]}.{entity[1]}' if entity[0] is not None else f'a {shape[0]}x{shape[1]} room'
    result['scenario'] = {'id': identifier, 'description': f'Walk into a room admitting {subject}',
                          'systems': ['enemies-bosses', 'entity-collision', 'damage-pipeline', 'room-connectivity'],
                          'randomized': randomized}
    result['entity'] = {'type': entity[0], 'variant': entity[1], 'shape': list(shape), 'slot': slot, 'selection': provenance}
    return result


def floor(run_seed):
    result = subprocess.run([str(GENERATOR)], input=json.dumps({'stageSeed': codec.initial_stage_seed(run_seed), 'spawns': True}),
                            capture_output=True, text=True, timeout=20, check=True)
    return json.loads(result.stdout)


def admitting_slots(generated, entity, shape=(13, 7), room_type=None, grid_type=None):
    start = generated['descriptors'][generated['startingDescriptor']]
    for slot, index in enumerate(start['neighbors'][:4]):
        if index is None:
            continue
        descriptor, spawns = generated['descriptors'][index], generated['spawns'][index]
        template = descriptor['template']
        # Ordinary (1) and miniboss (6) 13x7 rooms, or ordinary larger shapes;
        # boss rooms are never adjacent to the start.
        # A room-type spec (e.g. Sacrifice 13) enters only that authored type.
        types = (room_type,) if room_type is not None else (1, 6) if shape == (13, 7) else (1,)
        if template['type'] not in types or (template['width'], template['height']) != shape:
            continue
        if entity[0] is not None and not spawns:
            continue
        # A grid spec (e.g. spider webs 1940) enters only rooms that author it.
        if grid_type is not None and not (spawns and any(row['category'] == 'grid' and row['type'] == grid_type
                                                          for row in spawns['selectors'])):
            continue
        # A dropped co-spawn changes the room's actor order and with it every
        # (counter + sequence) frame gate, so only fully admitted rooms qualify.
        if generated['unsupportedNpcs'][index]:
            continue
        if entity[0] is None or any(row['category'] == 'entity' and (row['type'], row['variant']) == entity
                                    for row in spawns['selectors']):
            yield slot, descriptor


def discover(entity, count, attempts, shape=(13, 7), room_type=None):
    found, tried = [], set()
    for attempt in range(1, attempts + 1):
        vector = codec.random_vector()
        if vector['seedValue'] in tried:
            continue
        tried.add(vector['seedValue'])
        for slot, descriptor in admitting_slots(floor(vector['seedValue']), entity, shape, room_type):
            code = codec.encode_seed(vector['seedValue'])
            found.append(case(f'random-{len(found)}', code, slot, entity,
                              {'entropy': 'OS random', 'discoveryAttempts': attempt,
                               'room': descriptor['template']['name'], 'catalogIndex': descriptor['catalogIndex']}, True, shape))
            break
        if len(found) == count:
            return found
    raise RuntimeError(f'Only {len(found)} of {count} rooms admitting {entity} in {attempts} OS-random seeds')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--type', type=int, help='entities2 type; omit for a room-shape scenario')
    parser.add_argument('--variant', type=int, default=0)
    parser.add_argument('--contrived', action='append', default=[], metavar='CODE:SLOT')
    parser.add_argument('--count', type=int, default=2, help='OS-random cases to discover')
    parser.add_argument('--attempts', type=int, default=4000)
    parser.add_argument('--room-type', type=int, help='authored room type to enter (default ordinary 1, or 1/6 at 13x7)')
    parser.add_argument('--shape', default='13x7', help='authored room WIDTHxHEIGHT (13x7, 13x14, 26x7, 26x14)')
    parser.add_argument('--stop-after-capture', choices=['entry-1', 'entry-40'], help='end both public tapes at first entry or after its existing settle; default keeps the complete route')
    args = parser.parse_args()
    entity = (args.type, args.variant if args.type is not None else None)
    shape = tuple(int(part) for part in args.shape.split('x'))
    cases = []
    for index, item in enumerate(args.contrived):
        code, slot = item.split(':')
        slots = dict(admitting_slots(floor(codec.decode_seed(code)), entity, shape, args.room_type))
        if int(slot) not in slots:
            raise ValueError(f'Contrived {code} slot {slot} does not admit {entity} in a {args.shape} room')
        cases.append(case(f'contrived-{index}', code, int(slot), entity, {'entropy': 'contrived spec seed'}, False, shape))
    cases += discover(entity, args.count, args.attempts, shape, args.room_type) if args.count else []
    if args.stop_after_capture is not None:
        for recorded in cases:
            for name in ('tape', 'pairedTape'):
                matches = [index for index, action in enumerate(recorded[name])
                           if action.get('type') == 'capture' and action.get('label') == args.stop_after_capture]
                if len(matches) != 1:
                    raise ValueError(f'{name} requires one capture {args.stop_after_capture!r}')
                recorded[name] = recorded[name][:matches[0] + 1]
            recorded['entity']['stopAfterCapture'] = args.stop_after_capture
            runner.validate_tape(recorded['tape'])
    print(json.dumps({'schema': 1, 'entity': {'type': args.type, 'variant': entity[1], 'shape': list(shape)}, 'cases': cases}))


if __name__ == '__main__':
    main()
