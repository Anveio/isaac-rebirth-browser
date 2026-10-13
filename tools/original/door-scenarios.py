#!/usr/bin/env python3
"""Public curse/free-treasure tapes selected before live original execution."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('door_traversal', Path(__file__).with_name('foundation-traversal-scenarios.py'))
traversal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(traversal)


def curse_case(code, slot, selection, parity=0):
    case = traversal.case(code, slot, selection)
    case['doorContract'] = 'curse'
    case['setup'][0]['frames'] += parity
    split = next(index for index, action in enumerate(case['actions']) if action.get('label') == 'neighbor') + 1
    # Public waiting clears entry immunity before testing the exit charge.
    case['actions'][split:split] = [{'type': 'step', 'frames': 120}, {'type': 'capture', 'label': 'cooled'}]
    start = next(index for index, action in enumerate(case['tape']) if action.get('label') == 'gameplay-start') + 1
    case['tape'] = case['tape'][:start] + case['setup'] + case['actions']
    traversal.runner.validate_tape(case['tape'])
    return case


def recorded_treasure_bomb_case():
    # Retain the discovered W0L0LYYM public controls: movement/fire, pause,
    # release and a bomb that pushes Isaac through the free treasure doorway.
    # The source's actor/geometry snapshots never enter the browser.
    case = traversal.case('W0L0LYYM', 1, 'mandatory recorded OS-random bomb-driven treasure admission', 'entry-only')
    actions = [{'type': 'step', 'frames': 57}, {'type': 'capture', 'label': 'run-ready'}]
    def keys(names, down):
        actions.extend({'type': 'key', 'key': key, 'down': down} for key in names)
    for index, (names, held, coast) in enumerate([(['s', 'Down'], 16, 19), (['w', 'Up'], 22, 26), (['w', 'Right'], 22, 27)]):
        keys(names, True)
        actions.extend([{'type': 'step', 'frames': held}, {'type': 'capture', 'label': f'random-held-{index}'}])
        keys(names, False)
        actions.extend([{'type': 'step', 'frames': coast}, {'type': 'capture', 'label': f'random-coasting-{index}'}])
    for label, wait in [('random-paused', 42), ('random-resumed', 27)]:
        keys(['Escape'], True)
        actions.append({'type': 'step', 'frames': 1})
        keys(['Escape'], False)
        actions.extend([{'type': 'step', 'frames': wait}, {'type': 'capture', 'label': label}])
    keys(['e'], True)
    actions.append({'type': 'step', 'frames': 1})
    keys(['e'], False)
    # Preserve every input/time interval while adding dense read-only captures
    # around the blast and its resulting doorway admission.
    for index in range(133):
        actions.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'bomb-door-{index + 1}'}])
    actions.append({'type': 'capture', 'label': 'random-after-bomb'})
    retained = json.loads((traversal.ROOT / 'references/isaac/foundation/runtime-progress/W0L0LYYM.json').read_text())
    start = next(index for index, action in enumerate(retained['tape']) if action.get('label') == 'gameplay-start') + 1
    def public_intervals(tape):
        elapsed, keys = 0, []
        for action in tape:
            if action['type'] == 'step':
                elapsed += action['frames']
            elif action['type'] == 'key':
                keys.append((elapsed, action['key'], action['down']))
        return elapsed, keys
    if public_intervals(actions) != public_intervals(retained['tape'][start:]):
        raise ValueError('Mandatory W0L0LYYM public controls/time differ from the retained source input')
    prefix_end = next(index for index, action in enumerate(case['tape']) if action.get('label') == 'gameplay-start') + 1
    case.update(doorContract='free-treasure', goalRoomType=4, setup=[], actions=actions)
    case['tape'] = case['tape'][:prefix_end] + actions
    traversal.runner.validate_tape(case['tape'])
    return case


def choose(count):
    cases = [curse_case('63B9JKF8', 3, 'contrived opposite-phase curse entry/exit', parity=1), recorded_treasure_bomb_case()]
    catalogs = {stage: traversal.source.decoder.decode_bytes(data, source=name)
                for stage, (name, data) in enumerate(traversal.source.decoder.base_assets()) if stage in (0, 1)}
    identities = {case['runSeed'] for case in cases}
    selected = {10: 0, 4: 0}
    for _ in range(512):
        vector = traversal.source.codec.random_vector()
        if vector['seedValue'] in identities:
            continue
        floor = traversal.generated(vector['seedValue'])
        current = floor['descriptors'][floor['startingDescriptor']]
        for slot, index in enumerate(current['neighbors'][:4]):
            if index is None:
                continue
            target = floor['descriptors'][index]
            template = target['template']
            kind = template['type']
            if kind not in selected or selected[kind] >= count or (template['width'], template['height']) != (13, 7):
                continue
            asset = catalogs[target['assetStage']][target['catalogIndex']]
            allowed = (0, 5) if kind == 10 else (0, 5, 33)
            if any(entry['type'] not in allowed for group in asset['spawns'] for entry in group['entries']):
                continue
            code = traversal.source.codec.encode_seed(vector['seedValue'])
            if kind == 10:
                cases.append(curse_case(code, slot, 'recorded OS-random authored no-hostile curse-room route'))
            else:
                case = traversal.case(code, slot, 'recorded OS-random free-treasure doorway/return')
                case['doorContract'] = 'free-treasure'
                cases.append(case)
            selected[kind] += 1
            identities.add(vector['seedValue'])
            break
        if all(value == count for value in selected.values()):
            return {'schema': 1, 'contract': 'authored-doors', 'randomSourceCount': count * 2, 'cases': cases}
    raise ValueError('OS-random discovery did not find every required authored-door route')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if not 1 <= args.count <= 2:
        parser.error('Random source count must be 1..2')
    print(json.dumps(traversal.validate(args.validate_original) if args.validate_original else choose(args.count)))


if __name__ == '__main__':
    main()
