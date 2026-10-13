#!/usr/bin/env python3
"""Public244 cohorts: retained T1 damage/return and recorded random admission."""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('worm_paired', Path(__file__).with_name('foundation-paired-scenarios.py'))
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)
REFERENCES = ROOT / 'references/isaac/foundation/combat/roundworms'


def key(name, down):
    return {'type': 'key', 'key': name, 'down': down}


def step(count, label=None):
    return [{'type': 'step', 'frames': count}] + ([{'type': 'capture', 'label': label}] if label else [])


def case(identifier, code, tail, required, randomized=False):
    result = paired.build('paired-room-combat', code)
    result['scenario'] = {'id': identifier, 'description': 'Natural Round Worm244 appearance, dig/relocation, hostile projectile and room locks',
                          'systems': ['enemies-bosses', 'projectile-physics', 'damage-pipeline', 'room-connectivity'],
                          'randomized': randomized}
    result['wormRequired'] = required
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
    return result


def cohort():
    original = paired.build('paired-room-combat', 'T1PK6Y0W')
    end = next(index for index, action in enumerate(original['tape']) if action.get('label') == 'gameplay-start')
    tail = []
    for action in original['tape'][end + 1:]:
        # Observe the entire existing fuse interval, without changing inputs,
        # its total elapsed presentations, seed or fitting a contact timestamp.
        if action == {'type': 'step', 'frames': 118}:
            for elapsed in range(1, 119):
                tail += step(1, f'worm-fuse-{elapsed}')
        else:
            tail.append(action)
    damage = case('paired-worm-t1-damage', 'T1PK6Y0W', tail, ['admission', 'appearance', 'hidden', 'relocation', 'shoot', 'projectile-contact'])
    damage['knownRegression'] = 'Mandatory unchanged T1 rock route redHp3, source immunity and live244/type9 fields remain required; shared MT trajectories are separately retained.'
    retained = json.loads((REFERENCES / 'hostile-return-tail.json').read_text())
    returning = case('paired-worm-t1-hostile-return', 'T1PK6Y0W', retained['tail'], ['admission', 'appearance', 'locked', 'hostile-return'])
    returning['retainedTapeSha256'] = retained['originalTapeSha256']
    selection = json.loads((REFERENCES / 'random-selection.json').read_text())
    random = step(60) + [key('w', True)] + step(27) + [key('w', False)] + step(30)
    random += [key('a', True)] + step(72) + [key('a', False)]
    for elapsed in range(1, 31):
        random += step(1, f'random-worm-entry-{elapsed}')
    for elapsed in range(1, 241):
        random += step(1, f'random-worm-active-{elapsed}')
    random_case = case('paired-random-worm-projectile', selection['code'], random,
                       ['admission', 'appearance', 'hidden', 'relocation', 'shoot', 'projectile-contact'], True)
    random_case['randomSelection'] = selection
    return [damage, returning, random_case]


if __name__ == '__main__':
    print(json.dumps({'schema': 1, 'cases': cohort()}))
