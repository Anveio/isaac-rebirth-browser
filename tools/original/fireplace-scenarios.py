#!/usr/bin/env python3
"""Natural ordinary Fire33: retained random admission and public tear/blast routes."""
import importlib.util
import json
from pathlib import Path


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


paired = module('fire_paired', 'foundation-paired-scenarios.py')
doors = module('fire_doors', 'door-scenarios.py')


def key(name, down):
    return {'type': 'key', 'key': name, 'down': down}


def step(count, label=None):
    return [{'type': 'step', 'frames': count}] + ([{'type': 'capture', 'label': label}] if label else [])


def dense(count, prefix):
    return [action for index in range(1, count + 1) for action in step(1, f'{prefix}-{index}')]


def case(identifier, tail, required, randomized=False):
    result = paired.build('paired-room-combat', 'W0L0LYYM')
    result['scenario'] = {
        'id': identifier,
        'description': 'Authored ordinary Fireplace33 admission and subclass extinguishing',
        'systems': ['enemies-bosses', 'damage-pipeline', 'projectile-physics', 'room-connectivity'],
        'randomized': randomized,
    }
    result['fireRequired'] = required
    result['recordedSeed'] = {
        'code': 'W0L0LYYM', 'runSeed': 2591693487,
        'provenance': 'Previously recorded OS-random first-floor treasure holdout; no source state is supplied to either game',
    }
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
    paired.runner.validate_tape(result['tape'])
    return result


def cohort():
    admission = case('paired-fire-w0-admission', doors.recorded_treasure_bomb_case()['actions'] + dense(72, 'fire-idle'),
                     ['admission', 'ember-birth', 'ember-first-full', 'ember-retirement'], True)
    admission['knownRegression'] = 'Mandatory unchanged W0 treasure route, followed by public idle observations; raw1400 Fire admission and all prior player/door failures remain required.'
    entry = step(60) + [key('w', True)] + dense(74, 'fire-depart') + [key('w', False)]
    entry += dense(40, 'fire-entry') + step(120, 'fire-cooled')
    tears = entry + [key('Up', True)] + dense(220, 'fire-tear') + [key('Up', False)] + dense(20, 'fire-empty')
    tear_case = case('paired-fire-w0-tears', tears, ['admission', 'tear-queue', 'tear-drain', 'extinguish', 'empty-persistence'])
    blast = entry + [key('d', True)] + step(20) + [key('d', False)] + step(30, 'fire-side')
    blast += [key('w', True)] + step(25) + [key('w', False)] + step(30, 'fire-blast-approach')
    blast += [key('e', True)] + step(1, 'fire-bomb-birth') + [key('e', False)] + dense(140, 'fire-blast')
    blast_case = case('paired-fire-w0-blast', blast, ['admission', 'bomb-queue', 'extinguish', 'empty-persistence'])
    return [admission, tear_case, blast_case]


if __name__ == '__main__':
    print(json.dumps({'schema': 1, 'cases': cohort()}))
