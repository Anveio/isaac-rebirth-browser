#!/usr/bin/env python3
"""Public curse-door tapes selected before live original execution."""
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


def choose(count):
    cases = [curse_case('63B9JKF8', 3, 'contrived opposite-phase curse entry/exit', parity=1)]
    catalogs = {stage: traversal.source.decoder.decode_bytes(data, source=name)
                for stage, (name, data) in enumerate(traversal.source.decoder.base_assets()) if stage in (0, 1)}
    identities = {case['runSeed'] for case in cases}
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
            if template['type'] != 10 or (template['width'], template['height']) != (13, 7):
                continue
            asset = catalogs[target['assetStage']][target['catalogIndex']]
            if any(entry['type'] not in (0, 5) for group in asset['spawns'] for entry in group['entries']):
                continue
            cases.append(curse_case(traversal.source.codec.encode_seed(vector['seedValue']), slot,
                                    'recorded OS-random authored no-hostile curse-room route'))
            identities.add(vector['seedValue'])
            break
        if len(cases) == count + 1:
            return {'schema': 1, 'contract': 'curse', 'randomSourceCount': count, 'cases': cases}
    raise ValueError('OS-random discovery did not find every required curse-door route')


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
