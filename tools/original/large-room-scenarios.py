#!/usr/bin/env python3
"""Public tall-room admission, native boundary motion and return tapes."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('large_traversal', Path(__file__).with_name('foundation-traversal-scenarios.py'))
traversal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(traversal)


def case(code, slot, selection, parity=0):
    result = traversal.case(code, slot, selection)
    actions = []
    def held(key, count, prefix, settle=30):
        actions.append({'type': 'key', 'key': key, 'down': True})
        for frame in range(1, count + 1):
            actions.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'{prefix}-{frame}'}])
        actions.append({'type': 'key', 'key': key, 'down': False})
        for frame in range(1, settle + 1):
            actions.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'{prefix}-settle-{frame}'}])
    setup = [{'type': 'step', 'frames': 60 + parity}]
    if slot % 2 == 0:
        setup.extend([{'type': 'key', 'key': 'w', 'down': True}, {'type': 'step', 'frames': 27},
                      {'type': 'key', 'key': 'w', 'down': False}, {'type': 'step', 'frames': 30}])
    held(['a', 'w', 'd', 's'][slot], [72, 74, 72, 18][slot], 'large-enter', 40)
    actions.append({'type': 'capture', 'label': 'large-admitted'})
    # Stay along the source's radius-inset side wall, away from the authored
    # center Rock diamond/chest. No inventory, actors or positions are supplied.
    extended = slot == 2
    if slot == 3:
        held('a', 60, 'left-boundary')
    held('s', 140, 'lower-boundary')
    held('w', 140, 'upper-boundary')
    if slot == 3:
        held('d', 56, 'return-center')
        held('w', 18, 'large-return', 40)
    elif extended:
        # Native generated edge slot4 places Isaac at WorldY560. Return
        # through that lower-left edge, not the ordinary upper-left slot0.
        held('s', 93, 'return-height')
        held('a', 18, 'large-return', 40)
    else:
        # Return to the authored incoming right doorway at WorldY280.
        held('s', 29, 'return-height')
        held('d', 18, 'large-return', 40)
    actions.append({'type': 'capture', 'label': 'returned'})
    start = next(i for i, action in enumerate(result['tape']) if action.get('label') == 'gameplay-start') + 1
    result.update(doorContract='large-geometry', setup=setup, actions=actions, expectedVisitedRooms=2)
    result['tape'] = result['tape'][:start] + setup + actions
    traversal.runner.validate_tape(result['tape'])
    return result


def choose(count):
    cases = [
        case('WF82MYQ2', 3, 'contrived opposite-phase route on recorded OS seed, original discovery attempt15', 1),
        case('MT7N9MF3', 0, 'fresh recorded OS-random clear13x14 neighbor, discovery attempt223'),
    ]
    if count == 2:
        cases.append(case('C06MGLKV', 2, 'fresh recorded OS-random extendedslot4 tall-room admission/return, discovery attempt63'))
    return {'schema': 1, 'contract': 'large-geometry', 'randomSourceCount': count, 'cases': cases}


def validate(path):
    provenance = traversal.validate(path)
    document = json.loads(path.read_text())
    entries = document['debugObservations']
    start = entries[0]['observation']['floor']['currentDescriptorIndex']
    tall = [entry for entry in entries
            if (entry['observation']['floor']['descriptors'][entry['observation']['floor']['currentDescriptorIndex']]['template']['width'],
                entry['observation']['floor']['descriptors'][entry['observation']['floor']['currentDescriptorIndex']]['template']['height']) == (13, 14)]
    if not tall or len({entry['observation']['floor']['currentDescriptorIndex'] for entry in tall}) != 1:
        raise ValueError('Public tape must enter one actual authored13x14 room')
    if entries[-1]['observation']['floor']['currentDescriptorIndex'] != start:
        raise ValueError('Public tall-room tape must return to the starting descriptor')
    if not any(entry['context']['label'].startswith('lower-boundary')
               and abs(entry['observation']['contents']['players'][0]['position'][1] - 690) < .001 for entry in tall):
        raise ValueError('Public tape must exercise the source radius-inset lower plane')
    if not any(entry['context']['label'].startswith('upper-boundary')
               and abs(entry['observation']['contents']['players'][0]['position'][1] - 150) < .001 for entry in tall):
        raise ValueError('Public tape must exercise the source radius-inset upper plane')
    for entry in tall:
        observation = entry['observation']
        floor, contents = observation['floor'], observation['contents']
        template = floor['descriptors'][floor['currentDescriptorIndex']]['template']
        if (template['type'], template['variant']) != (1, 354):
            raise ValueError('Selected source route must enter the declared ordinary Room354')
        room = contents['room']
        if (room['gridWidth'], room['gridHeight'], room['boundsMin'], room['boundsMax']) != (15, 16, [60, 140], [580, 700]):
            raise ValueError('Native room dimensions and bounds must follow authored geometry')
        if any(entity['type'] not in (1, 2, 4, 5, 1000) for entity in contents['entities']):
            raise ValueError('This selected tall-room route must remain free of hostile actors')
        chests = [entity for entity in contents['entities'] if entity['type'] == 5 and entity['variant'] == 50]
        if len(chests) != 1 or chests[0]['position'] != [320, 400]:
            raise ValueError('Concrete native Chest50 admission remains mandatory at its authored location')
    catalogs = {stage: traversal.source.decoder.decode_bytes(data, source=name)
                for stage, (name, data) in enumerate(traversal.source.decoder.base_assets()) if stage in (0, 1)}
    provenance['authoredDoors'] = {}
    slots = {(-1, 3): 0, (6, -1): 1, (13, 3): 2, (26, 3): 2,
             (6, 7): 3, (6, 14): 3, (-1, 10): 4, (19, -1): 5,
             (13, 10): 6, (26, 10): 6, (19, 7): 7, (19, 14): 7}
    for entry in entries:
        floor = entry['observation']['floor']
        index = floor['currentDescriptorIndex']
        if index in provenance['authoredDoors']:
            continue
        template = floor['descriptors'][index]['template']
        matches = [asset for asset in catalogs[template['assetStage']]
                   if all(asset[field] == template[field] for field in ('type', 'variant', 'name', 'width', 'height'))]
        if len(matches) != 1:
            raise ValueError('Source door geometry must join one independent STB configuration')
        provenance['authoredDoors'][index] = [dict(slot=slots[(door['x'], door['y'])],
            index=door['x'] + 1 + (door['y'] + 1) * (template['width'] + 2),
            position=[80 + door['x'] * 40, 160 + door['y'] * 40])
            for door in matches[0]['doors'] if door['enabled']]
    provenance['largeGeometry'] = {'shape': [13, 14], 'sourceGrid': [15, 16],
        'nativeBounds': [[60, 140], [580, 700]], 'lowerAndUpperPlanesExecuted': True,
        'returnedToStartingDescriptor': True, 'mandatoryConcreteActor': 'Pickup5/50'}
    return provenance


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.count not in (1, 2):
        parser.error('This leaf retains one contrived route and one or two recorded OS-random tall rooms')
    print(json.dumps(validate(args.validate_original) if args.validate_original else choose(args.count)))


if __name__ == '__main__':
    main()
