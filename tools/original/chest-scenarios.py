"""Public ordinary Chest contact after a naturally planted access bomb."""
import argparse
import importlib.util
import json
from pathlib import Path

spec = importlib.util.spec_from_file_location('chest_large_routes', Path(__file__).with_name('large-room-scenarios.py'))
large = importlib.util.module_from_spec(spec)
spec.loader.exec_module(large)


def case(code, slot, selection, parity=0):
    result = large.case(code, slot, selection, parity)
    actions = []

    def wait(count, label):
        for frame in range(1, count + 1):
            actions.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'{label}-{frame}'}])

    def held(key, count, label, settle=30):
        actions.append({'type': 'key', 'key': key, 'down': True})
        wait(count, label)
        actions.append({'type': 'key', 'key': key, 'down': False})
        wait(settle, label + '-settle')

    held(['a', 'w', 'd', 's'][slot], [72, 74, 72, 18][slot], 'chest-enter', 40)
    # These are public movements around the authored outer Rock diagonals.
    # No positions, inventory, actor state or scatter vectors are supplied.
    if slot == 0:
        held('w', 50, 'chest-upper-route')
        held('a', 56, 'chest-center-route')
    elif slot == 2:
        held('w', 110, 'chest-upper-route')
        held('d', 56, 'chest-center-route')
    actions.append({'type': 'capture', 'label': 'chest-closed'})
    held('s', 60, 'chest-access-approach')
    held('e', 2, 'chest-access-bomb', 0)
    held('w', 35, 'chest-access-retreat')
    wait(80, 'chest-access-blast')
    held('s', 80, 'chest-contact')
    wait(30, 'chest-open-continuation')
    actions.append({'type': 'capture', 'label': 'chest-open-end'})
    boundary = next(i for i, action in enumerate(result['tape']) if action.get('label') == 'gameplay-start') + 1
    result.update(doorContract='chest-opening', mode='entry-only', goalRoomType=1,
                  actions=actions, expectedVisitedRooms=2)
    result['tape'] = result['tape'][:boundary] + result['setup'] + actions
    large.traversal.runner.validate_tape(result['tape'])
    return result


def choose(count):
    cases = [case('WF82MYQ2', 3, 'contrived opposite-phase Chest contact on recorded OS seed; original discovery attempt15', 1),
             case('MT7N9MF3', 0, 'retained unfiltered OS-random ordinary Room354 neighbor; discovery attempt223')]
    if count == 2:
        cases.append(case('C06MGLKV', 2, 'retained unfiltered OS-random extendedslot4 Room354 neighbor; discovery attempt63'))
    return {'schema': 1, 'contract': 'chest-opening', 'randomSourceCount': count, 'cases': cases}


def validate(path):
    result = large.traversal.validate(path)
    native = json.loads(path.read_text())
    if any(entry['observation']['contents']['players'][0]['redHp'] != 6 for entry in native['debugObservations']):
        raise ValueError('Public access retreat must keep fresh Isaac outside the actual blast')
    first = native['debugObservations'][0]['observation']['contents']['players'][0]
    if first['bombInventory'] != 1:
        raise ValueError('Public Chest access begins with the natural single starting bomb')
    snapshots = [entry for entry in native['debugObservations']
                 if entry['observation']['floor']['descriptors'][entry['observation']['floor']['currentDescriptorIndex']]['template']['variant'] == 354]
    for entry in snapshots:
        chests = [entity for entity in entry['observation']['contents']['entities'] if entity['type'] == 5 and entity['variant'] == 50]
        if not chests or any('localRngWords' not in chest for chest in chests):
            raise ValueError('Every reached normal Chest needs actual persistent RNG30 readback')
    if not any(entity['subtype'] == 0 and entity['animation'] == 'Open'
               for entry in snapshots for entity in entry['observation']['contents']['entities']
               if entity['type'] == 5 and entity['variant'] == 50):
        raise ValueError('The public route must actually contact and open its natural normal Chest')
    result['chestOpening'] = {'sourcePersistentRngReadback': True, 'naturalStartingBombAccess': True,
                              'actualOpenExecuted': True, 'scope': 'fresh free normal50, no modifier/rare-family/save-load claim'}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.count not in (1, 2):
        parser.error('Retain one contrived route and one or two independently recorded OS seeds')
    print(json.dumps(validate(args.validate_original) if args.validate_original else choose(args.count)))


if __name__ == '__main__':
    main()
