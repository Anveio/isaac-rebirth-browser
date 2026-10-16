#!/usr/bin/env python3
"""Public ordinary-door tapes and full source provenance for paired browser gates."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


source = load('traversal_source', 'foundation-floor-source.py')
runner = load('traversal_clock', 'foundation-clock-run.py')
clock = load('traversal_provenance', 'foundation-clock-verify.py')
generator = ROOT / 'engine/target/debug/floor'


def generated(run_seed):
    result = subprocess.run([str(generator)], input=json.dumps({'stageSeed': source.codec.initial_stage_seed(run_seed)}),
                            text=True, capture_output=True, timeout=20, check=True)
    return json.loads(result.stdout)


def actions(slot, return_room=True, cross=0):
    tape = []
    def held(key, count, prefix):
        tape.append({'type': 'key', 'key': key, 'down': True})
        for index in range(count):
            tape.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'{prefix}-{index + 1}'}])
        tape.append({'type': 'key', 'key': key, 'down': False})
    def settle(prefix):
        for index in range(40):
            tape.extend([{'type': 'step', 'frames': 1}, {'type': 'capture', 'label': f'{prefix}-{index + 1}'}])
    held(['a', 'w', 'd', 's'][slot], [72, 74, 72, 18][slot] + cross, 'depart')
    settle('entry')
    tape.append({'type': 'capture', 'label': 'neighbor'})
    if not return_room:
        return tape
    held(['a', 'w', 'd', 's'][(slot + 2) % 4], 18, 'return-depart')
    settle('return-entry')
    tape.append({'type': 'capture', 'label': 'returned'})
    return tape


def case(code, slot, provenance, mode="return", cross=0):
    value = source.codec.decode_seed(code)
    prefix = runner.default_tape(code)
    end = next(index for index, action in enumerate(prefix) if action.get('label') == 'gameplay-start')
    tape = [action for action in prefix[:end + 1] if action['type'] != 'capture' or action['label'] == 'gameplay-start']
    setup = [{'type': 'step', 'frames': 60}]
    if slot in (0, 2):
        setup += [{'type': 'key', 'key': 'w', 'down': True}, {'type': 'step', 'frames': 27},
                  {'type': 'key', 'key': 'w', 'down': False}, {'type': 'step', 'frames': 30}]
    route = actions(slot, mode != "entry-only", cross)
    tape += setup + route
    runner.validate_tape(tape)
    return {'code': code, 'runSeed': value, 'slot': slot, 'selection': provenance, 'mode': mode,
            'setup': setup, 'actions': route, 'tape': tape}


# Extra held frames that carry an entering player across the neighbor's
# doorway line onto its first web (GridEntity_Web slowing, spider_webs.rs).
WEB_CROSS = 60
# Recorded OS-random draw whose right neighbor is the web room (Basement 223).
WEB_SEED, WEB_SLOT = 'ZJQKAXCG', 2


def web_case(code, slot, provenance):
    return dict(case(code, slot, provenance, 'entry-only', WEB_CROSS), goalRoomType=1)


def web_line(asset, slot):
    """Whether entering `asset` from `slot` walks onto an authored web first.

    The room must hold only rocks, blocks and webs (no actor choices), its entry
    door must be enabled, and the first nonempty cell on the doorway line must
    be a web in every authored choice."""
    if asset['type'] != 1 or (asset['width'], asset['height']) != (13, 7):
        return False
    if any(entry['type'] not in (1000, 1900, 1940) for group in asset['spawns'] for entry in group['entries']):
        return False
    entry_door = [(13, 3), (6, 7), (-1, 3), (6, -1)][slot]
    if not any((door['x'], door['y']) == entry_door and door['enabled'] for door in asset['doors']):
        return False
    cells = {(group['x'], group['y']): group['entries'] for group in asset['spawns'] if group['entries']}
    line = [[(x, 3) for x in range(12, -1, -1)], [(6, y) for y in range(6, -1, -1)],
            [(x, 3) for x in range(13)], [(6, y) for y in range(7)]][slot]
    for cell in line:
        if cell in cells:
            return all(entry['type'] == 1940 for entry in cells[cell])
    return False


def choose(count):
    # Contrived route is a source input, never a production seed exception.
    result = [case('CSTGNPJ8', 2, 'contrived observed ordinary doorway/return'),
              case('T1PK6Y0W', 2, 'mandatory hostile entrance/return-lock regression', 'blocked-return'),
              web_case(WEB_SEED, WEB_SLOT, 'contrived spider-web crossing')]
    catalog = source.decoder.decode_bytes(source.decoder.base_assets()[1][1], source='authored Basement')
    identities = {entry['runSeed'] for entry in result}
    web = None
    for _ in range(8192):
        vector = source.codec.random_vector()
        if vector['seedValue'] in identities:
            continue
        floor = generated(vector['seedValue'])
        neighbors = floor['descriptors'][floor['startingDescriptor']]['neighbors'][:4]
        slot = next((slot for slot, index in enumerate(neighbors) if index is not None
                     and floor['descriptors'][index]['assetStage'] == 1
                     and web_line(catalog[floor['descriptors'][index]['catalogIndex']], slot)), None)
        if slot is not None:
            web = web_case(source.codec.encode_seed(vector['seedValue']), slot,
                           'recorded OS-random spider-web crossing')
            identities.add(vector['seedValue'])
            break
    if web is None:
        raise ValueError('OS-random discovery did not find a spider-web crossing route')
    result.append(web)
    for _ in range(2048):
        vector = source.codec.random_vector()
        if vector['seedValue'] in identities:
            continue
        floor = generated(vector['seedValue'])
        for slot, index in enumerate(floor['descriptors'][floor['startingDescriptor']]['neighbors'][:4]):
            if index is None:
                continue
            descriptor = floor['descriptors'][index]
            if descriptor['assetStage'] != 1 or descriptor['template']['type'] != 1 \
                    or (descriptor['template']['width'], descriptor['template']['height']) != (13, 7):
                continue
            asset = catalog[descriptor['catalogIndex']]
            # Scope the return tape to configurations without hostile authored
            # actor choices. All selected originals still execute and must pass.
            if any(entry['type'] not in (0, 5) for group in asset['spawns'] for entry in group['entries']):
                continue
            result.append(case(source.codec.encode_seed(vector['seedValue']), slot, 'recorded OS-random ordinary no-hostile authored configuration'))
            identities.add(vector['seedValue'])
            break
        if len(result) == count + 4:
            return {'schema': 1, 'randomSourceCount': count, 'cases': result}
    raise ValueError('OS-random discovery did not find every required ordinary return route')


def validate(path):
    document = json.loads(path.read_text())
    failures = clock.validate_run(document)
    if failures:
        raise ValueError('Actual full X11/clock tape provenance failed: ' + '; '.join(failures))
    entries = document['debugObservations']
    if [entry['context']['label'] for entry in entries] != [frame['label'] for frame in document['frames']]:
        raise ValueError('Every selected framebuffer requires its paired source observation')
    for entry, frame in zip(entries, document['frames']):
        context, observation = entry['context'], entry['observation']
        if observation['status'] != 'pass' or observation['memoryWritten'] is not False \
                or any(context[key] != frame[key] for key in ('label', 'presentedFrame', 'virtualElapsedNs')):
            raise ValueError('Source traversal observation does not pair with the selected phase')
        for nested in (observation['floor'], observation['contents']):
            if nested['status'] != 'pass' or nested['memoryWritten'] is not False \
                    or nested['nativeBinarySha256'] != source.observer.NATIVE_SHA:
                raise ValueError('Every phase requires source-hashed read-only floor and actor evidence')
        if observation['floor']['actualPublicLog'] != context['publicLogState']:
            raise ValueError('Current descriptor must pair with the actual public room/SpawnRNG log')
        if not (path.parent / frame['path']).is_file():
            raise ValueError('Selected original framebuffer is missing')
    start = entries[0]
    if start['context']['label'] != 'gameplay-start':
        raise ValueError('Traversal begins at the independent original gameplay boundary')
    # Project the retained composite observer's actual floor data for the
    # reusable descriptor guards; do not replace any observed values.
    controller = dict(document, debugObservations=[dict(entry, observation=entry['observation']['floor']) for entry in entries])
    text = path.with_name('original.log').read_text()
    actual = source.starting_log_state(text)
    floor_case = {'seed': document['seed'], 'actualLog': actual, 'observation': start['observation']['floor'],
        'controller': controller, 'growthInput': source.supplied_growth_input(actual['stageSeed'], actual['rooms']),
        'provenance': {'memoryWritten': False, 'nativeBinarySha256': source.observer.NATIVE_SHA,
                       'originalCheckpoint': {'context': start['context']}}}
    source.validate_case(floor_case, source.authored_catalogs()[0], allow_non_labyrinth_curses=True)
    # Join independently observed template identity to the original STB, not
    # the browser's catalogue index or its selected group list. Retain every
    # nonempty authored group, including unsupported/empty-entity choices.
    catalogs = {stage: source.decoder.decode_bytes(data, source=name)
                for stage, (name, data) in enumerate(source.decoder.base_assets()) if stage in (0, 1)}
    authored_groups = {}
    for entry in entries:
        floor = entry['observation']['floor']
        index = floor['currentDescriptorIndex']
        if index in authored_groups:
            continue
        template = floor['descriptors'][index]['template']
        matches = [asset for asset in catalogs[template['assetStage']]
                   if all(asset[field] == template[field] for field in ('type', 'variant', 'name', 'width', 'height'))]
        if len(matches) != 1:
            raise ValueError('Current native template requires one independent authored STB identity')
        authored_groups[index] = [dict(groupIndex=group_index, **group)
                                 for group_index, group in enumerate(matches[0]['spawns']) if group['entries']]
    return {'status': 'pass', 'nativeBinarySha256': source.observer.NATIVE_SHA,
            'manifestSha256': hashlib.sha256(path.read_bytes()).hexdigest(),
            'originalLogSha256': hashlib.sha256(text.encode()).hexdigest(), 'observations': len(entries),
            'authoredGroups': authored_groups}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if not 1 <= args.count <= 2:
        parser.error('Random source count must be 1..2')
    print(json.dumps(validate(args.validate_original) if args.validate_original else choose(args.count)))


if __name__ == '__main__':
    main()
