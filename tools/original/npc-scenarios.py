#!/usr/bin/env python3
"""Public Fly/AttackFly tapes, including a retained OS-random authored room."""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('npc_paired', Path(__file__).with_name('foundation-paired-scenarios.py'))
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)


def key(name, down):
    return {'type': 'key', 'key': name, 'down': down}


def step(count, label=None):
    return [{'type': 'step', 'frames': count}] + ([{'type': 'capture', 'label': label}] if label else [])


def admitted(direction, presentations):
    return [key(direction, True)] + step(presentations, 'npc-room-admitted') + [key(direction, False)] \
        + step(17, 'npc-inbound-frozen') + step(1, 'npc-first-active-full')


def bomb_tail():
    tape = [key('e', True)] + step(1, 'npc-bomb-birth') + [key('e', False)] + step(116, 'npc-before-blast')
    for index in range(117, 128):
        tape += step(1, f'npc-bomb-{index}')
    return tape


def case(identifier, code, tail, required, randomized=False):
    result = paired.build('paired-bomb-moving-even', code)
    result['scenario'] = {'id': identifier, 'description': 'Authored NPC public-input contact, queued HP, death and clear',
                          'systems': ['enemies-bosses', 'entity-collision', 'damage-pipeline', 'doors-minimap'],
                          'randomized': randomized}
    result['npcRequired'] = required
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
    return result


def cohort():
    contact = admitted('s', 17) + [key('s', True)]
    for index in range(1, 21):
        contact += step(1, f'npc-contact-{index}')
    contact += [key('s', False)] + step(1, 'npc-contact-release')
    tear = admitted('s', 17) + [key('Down', True)]
    for index in range(1, 57):
        tear += step(1, f'npc-tear-{index}')
    tear += [key('Down', False)] + step(8, 'npc-tear-retired')
    queue = admitted('w', 85) + step(6, 'npc-held-stationary') + bomb_tail() + step(4, 'npc-after-blast')
    center = admitted('w', 85) + [key('w', True)] + step(50, 'npc-center-approach') + [key('w', False)] \
        + step(30, 'npc-center-coast') + bomb_tail()
    for index in range(1, 33):
        center += step(1, f'npc-clear-{index}')
    random = step(60) + [key('w', True)] + step(27) + [key('w', False)] + step(30) \
        + [key('d', True)] + step(135) + [key('d', False)] + step(30, 'random-npc-room')
    for segment, (aim, count) in enumerate([('Right', 360), ('Up', 120), ('Down', 120), ('Right', 120)]):
        random += [key(aim, True)] + step(1, f'random-clear-{segment}-1')
        for elapsed in range(3, count, 2):
            random += step(2, f'random-clear-{segment}-{elapsed}')
        random += step(1) + [key(aim, False)]
    random += step(40, 'random-npc-cleared') + step(120, 'random-npc-clear-settled')
    selection = json.loads((ROOT / 'references/isaac/foundation/combat/npcs/random-selection.json').read_text())
    random_case = case('paired-random-attackfly-clear', selection['code'], random, ['admission', 'clear', 'clear-delay'], True)
    random_case['randomSelection'] = selection
    regression = case('paired-fly-center-bomb', '3BMBMFTE', center, ['admission', 'clear'])
    regression['knownRegression'] = 'Natural center approach crosses unsupported authored grid kinds3/14; retain the exact original actor/HP and room-clear failure.'
    return [case('paired-attackfly-contact', '3BMBMFTE', contact, ['admission', 'contact']),
            case('paired-fly-bomb-queue', '3BMBMFTE', queue, ['admission', 'bomb']),
            case('paired-attackfly-tear', '3BMBMFTE', tear, ['admission', 'tear']), random_case, regression]


if __name__ == '__main__':
    print(json.dumps({'schema': 1, 'cases': cohort()}))
