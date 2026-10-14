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


def route(slot):
    tape = [{'type': 'step', 'frames': 60}]
    if slot in (0, 2):
        # Side doorways sit on the room's middle row, above the spawn point.
        tape += [key('w', True), {'type': 'step', 'frames': 27}, key('w', False), {'type': 'step', 'frames': 30}]
    move, frames = DEPART[slot]
    tape.append(key(move, True))
    dense(tape, frames, 'depart')
    tape.append(key(move, False))
    dense(tape, 40, 'entry')
    for turn in range(4):
        aim = AIM[(slot + turn) % 4]
        tape.append(key(aim, True))
        dense(tape, 120 if turn == 0 else 60, f'shoot{turn}', 2)
        tape.append(key(aim, False))
    dense(tape, 90, 'idle', 2)
    return tape


def case(identifier, code, slot, entity, provenance, randomized):
    result = paired.build('paired-bomb-moving-even', code)
    tail = route(slot)
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
    runner.validate_tape(result['tape'])
    result['scenario'] = {'id': identifier, 'description': f'Walk into a room admitting entity {entity[0]}.{entity[1]}',
                          'systems': ['enemies-bosses', 'entity-collision', 'damage-pipeline', 'room-connectivity'],
                          'randomized': randomized}
    result['entity'] = {'type': entity[0], 'variant': entity[1], 'slot': slot, 'selection': provenance}
    return result


def floor(run_seed):
    result = subprocess.run([str(GENERATOR)], input=json.dumps({'stageSeed': codec.initial_stage_seed(run_seed), 'spawns': True}),
                            capture_output=True, text=True, timeout=20, check=True)
    return json.loads(result.stdout)


def admitting_slots(generated, entity):
    start = generated['descriptors'][generated['startingDescriptor']]
    for slot, index in enumerate(start['neighbors'][:4]):
        if index is None:
            continue
        descriptor, spawns = generated['descriptors'][index], generated['spawns'][index]
        template = descriptor['template']
        if template['type'] != 1 or (template['width'], template['height']) != (13, 7) or not spawns:
            continue
        # A dropped co-spawn changes the room's actor order and with it every
        # (counter + sequence) frame gate, so only fully admitted rooms qualify.
        if generated['unsupportedNpcs'][index]:
            continue
        if any(row['category'] == 'entity' and (row['type'], row['variant']) == entity for row in spawns['selectors']):
            yield slot, descriptor


def discover(entity, count, attempts):
    found, tried = [], set()
    for attempt in range(1, attempts + 1):
        vector = codec.random_vector()
        if vector['seedValue'] in tried:
            continue
        tried.add(vector['seedValue'])
        for slot, descriptor in admitting_slots(floor(vector['seedValue']), entity):
            code = codec.encode_seed(vector['seedValue'])
            found.append(case(f'random-{len(found)}', code, slot, entity,
                              {'entropy': 'OS random', 'discoveryAttempts': attempt,
                               'room': descriptor['template']['name'], 'catalogIndex': descriptor['catalogIndex']}, True))
            break
        if len(found) == count:
            return found
    raise RuntimeError(f'Only {len(found)} of {count} rooms admitting {entity} in {attempts} OS-random seeds')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--type', type=int, required=True)
    parser.add_argument('--variant', type=int, default=0)
    parser.add_argument('--contrived', action='append', default=[], metavar='CODE:SLOT')
    parser.add_argument('--count', type=int, default=2, help='OS-random cases to discover')
    parser.add_argument('--attempts', type=int, default=4000)
    args = parser.parse_args()
    entity = (args.type, args.variant)
    cases = []
    for index, item in enumerate(args.contrived):
        code, slot = item.split(':')
        slots = dict(admitting_slots(floor(codec.decode_seed(code)), entity))
        if int(slot) not in slots:
            raise ValueError(f'Contrived {code} slot {slot} does not admit {entity}')
        cases.append(case(f'contrived-{index}', code, int(slot), entity, {'entropy': 'contrived spec seed'}, False))
    cases += discover(entity, args.count, args.attempts) if args.count else []
    print(json.dumps({'schema': 1, 'entity': {'type': args.type, 'variant': args.variant}, 'cases': cases}))


if __name__ == '__main__':
    main()
