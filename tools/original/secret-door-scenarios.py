#!/usr/bin/env python3
"""Public bomb revelation and return tapes for actual Basement secret doors."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('secret_traversal', Path(__file__).with_name('foundation-traversal-scenarios.py'))
traversal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(traversal)


def secret_case(code, selection, parity=0):
    case = traversal.case(code, 3, selection)
    actions = []
    def key(name, down):
        actions.append({'type': 'key', 'key': name, 'down': down})
    def step(count):
        actions.append({'type': 'step', 'frames': count})
    def capture(label):
        actions.append({'type': 'capture', 'label': label})
    def dense(count, prefix):
        for index in range(count):
            step(1)
            capture(f'{prefix}-{index + 1}')
    step(60 + parity)
    capture('run-ready')
    key('e', True)
    step(1)
    key('e', False)
    capture('bomb-birth')
    key('w', True)
    step(35)
    key('w', False)
    step(20)
    capture('retreated')
    for index in range(56, 134):
        step(1)
        capture(f'blast-{index}')
    key('s', True)
    dense(45, 'enter')
    key('s', False)
    dense(40, 'entry-settle')
    capture('secret-entry')
    key('w', True)
    dense(60, 'return')
    key('w', False)
    dense(40, 'return-settle')
    capture('returned')
    start = next(index for index, action in enumerate(case['tape']) if action.get('label') == 'gameplay-start') + 1
    case.update(doorContract='secret', setup=[], actions=actions)
    case['tape'] = case['tape'][:start] + actions
    traversal.runner.validate_tape(case['tape'])
    return case


def choose(count):
    cases = [secret_case('JF2DSF92', 'contrived opposite-phase secret blast/entry/return', parity=1)]
    catalogs = {stage: traversal.source.decoder.decode_bytes(data, source=name)
                for stage, (name, data) in enumerate(traversal.source.decoder.base_assets()) if stage in (0, 1)}
    identities = {case['runSeed'] for case in cases}
    for _ in range(2048):
        vector = traversal.source.codec.random_vector()
        if vector['seedValue'] in identities:
            continue
        floor = traversal.generated(vector['seedValue'])
        index = floor['descriptors'][floor['startingDescriptor']]['neighbors'][3]
        if index is None:
            continue
        target = floor['descriptors'][index]
        template = target['template']
        if template['type'] not in (7, 8) or (template['width'], template['height']) != (13, 7):
            continue
        asset = catalogs[target['assetStage']][target['catalogIndex']]
        # Shopkeepers explicitly have shutdoors=false in authored entities2;
        # actor behavior/loot stays outside this doorway route.
        if any(entry['type'] not in (0, 5, 17) for group in asset['spawns'] for entry in group['entries']):
            continue
        cases.append(secret_case(traversal.source.codec.encode_seed(vector['seedValue']),
                                 'recorded OS-random no-hostile secret revelation/entry/return'))
        identities.add(vector['seedValue'])
        if len(cases) == count + 1:
            return {'schema': 1, 'contract': 'secret', 'randomSourceCount': count, 'cases': cases}
    raise ValueError('OS-random discovery did not find every required secret route')


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
