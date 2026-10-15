#!/usr/bin/env python3
"""Run a declared, isolated Challenge item fixture through actual public inputs.

The original binary and packed resources stay unchanged. Only the explicitly
authored Challenge XML and optional full Special catalogue view differ. Their bytes are also the browser's input;
native observations never supply the browser's seed, stats, RNG or actors.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess
from pathlib import Path
import sys
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]


def stage_view(authored_xml, output, special_rooms=None):
    data = authored_xml.read_bytes()
    root = ET.fromstring(data)
    ids = [node.get('id') for node in root if node.tag == 'challenge']
    if root.tag != 'challenges' or root.get('version') != '1' or len(ids) != len(set(ids)) or '1' not in ids:
        raise ValueError('Fixture requires base Challenge XML with unique IDs and Challenge 1')
    source = ROOT / '.cache/original-runtime/offline-depot'
    for name in ('isaac.x64', 'lib64', 'resources'):
        if not (source / name).exists():
            raise FileNotFoundError(source / name)
    output.mkdir(parents=True, exist_ok=False)
    depot = output / '.cache/original-runtime/offline-depot'
    depot.mkdir(parents=True)
    for name in ('isaac.x64', 'lib64'):
        (depot / name).symlink_to((source / name).resolve())
    resources = depot / 'resources'
    resources.mkdir()
    for entry in (source / 'resources').iterdir():
        if entry.name != 'challenges.xml' and not (special_rooms and entry.name == 'rooms'):
            (resources / entry.name).symlink_to(entry.resolve())
    (resources / 'challenges.xml').write_bytes(data)
    if special_rooms is not None:
        rooms = resources / 'rooms'
        rooms.mkdir()
        source_rooms = source / 'resources/rooms'
        if source_rooms.is_dir():
            for entry in source_rooms.iterdir():
                if entry.name.lower() != '00.special rooms.stb':
                    (rooms / entry.name).symlink_to(entry.resolve())
        (rooms / '00.special rooms.stb').write_bytes(special_rooms.read_bytes())
    (output / 'declared-challenges.xml').write_bytes(data)
    return data


def public_tape(case):
    if case.get('fixture', {}).get('challengeId') != 1 or type(case.get('initialPhaseOffset')) is not int or case['initialPhaseOffset'] not in (0, 1):
        raise ValueError('The mapped front door admits Challenge 1 with either declared presentation offset')
    # Main5a4580 skips the invalid Continue row in a fresh profile. One Down
    # selects row2 ->5a4790 ->Mode5. Constructor59cef0 selects Challenge ID1;
    # confirm59d080 clears seed text, draws the shared raw seed and594620 queues
    # the real run. A requested Character/manual seed is never substituted.
    tape = [{'type': 'step', 'frames': 180}, {'type': 'capture', 'label': 'intro'}]
    for label in ('title', 'profile', 'main'):
        tape.extend([{'type': 'key', 'key': 'Return', 'down': True}, {'type': 'step', 'frames': 12},
                     {'type': 'key', 'key': 'Return', 'down': False}, {'type': 'step', 'frames': 72},
                     {'type': 'capture', 'label': label}])
    for key, label, settle in [('Down', 'challenge-selected', 72), ('Return', 'challenges', 72),
                               ('Return', 'gameplay-start', 300)]:
        tape.append({'type': 'key', 'key': key, 'down': True})
        tape.extend([{'type': 'step', 'frames': 1}] * 12 if label == 'gameplay-start' else [{'type': 'step', 'frames': 12}])
        tape.append({'type': 'key', 'key': key, 'down': False})
        # Starting NPCs and released controlled attacks can interact during
        # run admission. Retain every real prelude phase, not only its end.
        tape.extend([{'type': 'step', 'frames': 1}] * settle if label == 'gameplay-start' else [{'type': 'step', 'frames': settle}])
        tape.append({'type': 'capture', 'label': label})
    if case['initialPhaseOffset']:
        tape.append({'type': 'step', 'frames': 1})
    held = set()
    for action in case['actions']:
        if set(action) != {'label', 'keys', 'presentations'} or not isinstance(action['keys'], list):
            raise ValueError('Actions must declare a label, public keys and presentation count')
        keys = set(action['keys'])
        if len(keys) != len(action['keys']) or not keys <= {'Right', 'Left', 'Up', 'Down', 'w', 'a', 's', 'd', 'space', 'e', 'q'}:
            raise ValueError('Fixture action uses undeclared or duplicate gameplay keys')
        count = action['presentations']
        if type(count) is not int or not 1 <= count <= 400:
            raise ValueError('Each gameplay interval must be 1..400 declared presentations')
        for key in sorted(held - keys):
            tape.append({'type': 'key', 'key': key, 'down': False})
        for key in sorted(keys - held):
            tape.append({'type': 'key', 'key': key, 'down': True})
        # Dense step gates retain both source phases and all actual callbacks.
        for index in range(1, count + 1):
            tape.append({'type': 'step', 'frames': 1})
            if index in (3, 4):
                tape.append({'type': 'capture', 'label': action['label'] + '-observe-step' + str(index)})
        tape.append({'type': 'capture', 'label': action['label']})
        held = keys
    for key in sorted(held):
        tape.append({'type': 'key', 'key': key, 'down': False})
    return tape


def canonical_original():
    # There is one checked-in binary pin. Resolve and validate it through the
    # existing public module instead of copying a second constant into Python.
    script = ('const {ORIGINAL_SHA,assertOriginalBinary}=await import(process.argv[1]);'
              'await assertOriginalBinary();process.stdout.write(ORIGINAL_SHA);')
    actual = subprocess.check_output(['node', '--input-type=module', '--eval', script,
                                      (ROOT / 'tools/e2e/lib/original.mjs').as_uri()], text=True)
    if len(actual) != 64 or any(character not in '0123456789abcdef' for character in actual):
        raise ValueError('Canonical original identity did not return a SHA256')
    requested = os.environ.get('ISAAC_ORIGINAL_SHA')
    if requested is not None and requested != actual:
        raise ValueError('Requested original identity differs from the canonical input')
    return actual


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--scenarios', type=Path, required=True)
    parser.add_argument('--case', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--prepare-only', action='store_true', help='Write launch inputs without executing either game')
    args, extra = parser.parse_known_args()
    owned_options = {'--seed', '--ordinary-run', '--challenge-run', '--tape', '--output', '--boot-epoch-seed'}
    if any(flag.startswith('--') and any(name.startswith(flag.split('=', 1)[0]) for name in owned_options)
           for flag in extra):
        raise ValueError('The declared case owns mode, public tape, boot input and output')
    if not args.prepare_only and ('--observe-every-step' not in extra or not any(flag.split('=', 1)[0] == '--state-observer' for flag in extra)):
        raise ValueError('Execution requires a read-only source observer at every declared step; preparation is not actor acceptance')
    bank = json.loads(args.scenarios.read_text())
    matches = [case for case in bank['cases'] if case['name'] == args.case]
    if len(matches) != 1:
        raise ValueError('Exactly one declared case must match')
    case = matches[0]
    case['fixture'] = dict(case['fixture'])
    for name in ('authoredResourcePath', 'specialRoomsPath'):
        if case['fixture'].get(name):
            case['fixture'][name] = str((args.scenarios.resolve().parent / case['fixture'][name]).resolve())
    epoch = case.get('bootEpochSeed')
    if type(epoch) is not int or not 0 <= epoch <= 0xffffffff:
        raise ValueError('Case must declare a public unsigned32 boot epoch')
    contract = case.get('requires', {}).get('sourceExpectedActorContract')
    if not isinstance(contract, dict) or type(contract.get('family')) is not int:
        raise ValueError('A source-backed actor-family obligation is required before fixture preparation')
    source_fields = case.get('requires', {}).get('compositionSourceFields', ['playerMovementD9c', 'knifeChildChain4a0'])
    if (not isinstance(source_fields, list) or len(set(source_fields)) != len(source_fields)
            or any(name not in ('playerMovementD9c', 'knifeChildChain4a0', 'npc14And18DamageQueue', 'kind8TearFields', 'ordinaryTechnology', 'ordinaryDrFetusBombs', 'controlledDrFetusBombs') for name in source_fields)):
        raise ValueError('Unknown or duplicate requested composition source field')
    source_fields = list(source_fields)
    if case.get('requires', {}).get('ordinaryTechnologyFunctional') is not None:
        source_fields = [*source_fields, *([] if 'ordinaryTechnology' in source_fields else ['ordinaryTechnology'])]
    if any(name in case['requires'] for name in ('ordinaryBrimContact', 'controlledTearContact')):
        if 'npc14And18DamageQueue' not in source_fields:
            source_fields.append('npc14And18DamageQueue')
    if (contract.get('primaryC08') == 8 and contract['family'] == 2
            and contract.get('controlledBy329') is True and 'kind8TearFields' not in source_fields):
        source_fields.append('kind8TearFields')
    controlled_fetus = case['requires'].get('controlledDrFetusFunctional')
    if controlled_fetus is not None:
        if (not isinstance(controlled_fetus, dict) or controlled_fetus.get('schema') != 'controlled-dr-fetus52-bomb-c18-v1'
                or (contract.get('primaryC08'), contract['family'], contract.get('variant'),
                    contract.get('subtype'), contract.get('controlledBy329')) != (5, 4, 0, 0, True)):
            raise ValueError('Controlled DrFetus Bomb observation requires its exact source declaration')
        for name in ('playerMovementD9c', 'controlledDrFetusBombs'):
            if name not in source_fields:
                source_fields.append(name)
    fetus = case['requires'].get('ordinaryDrFetus')
    if fetus is not None:
        if (not isinstance(fetus, dict) or fetus.get('schema') != 'ordinary-dr-fetus52-bomb-shots-v1'
                or (contract.get('primaryC08'), contract['family'], contract.get('variant'),
                    contract.get('subtype'), contract.get('controlledBy329')) != (5, 4, 0, 0, False)):
            raise ValueError('Typed DrFetus Bomb observation requires its exact source declaration')
        if 'ordinaryDrFetusBombs' not in source_fields:
            source_fields.append('ordinaryDrFetusBombs')
    tape = public_tape(case)
    spec = importlib.util.spec_from_file_location('composition_clock', ROOT / 'tools/original/foundation-clock-run.py')
    clock = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(clock)
    clock.validate_tape(tape)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    authored_xml = Path(case['fixture']['authoredResourcePath']).resolve()
    special_rooms = Path(case['fixture']['specialRoomsPath']).resolve() if case['fixture'].get('specialRoomsPath') else None
    input_hashes = {'challengesXml': hashlib.sha256(authored_xml.read_bytes()).hexdigest()}
    if special_rooms is not None:
        input_hashes['specialRooms'] = hashlib.sha256(special_rooms.read_bytes()).hexdigest()
        if case['fixture'].get('specialRoomsSha256') and input_hashes['specialRooms'] != case['fixture']['specialRoomsSha256']:
            raise ValueError('Declared Special catalogue bytes do not match the input identity')
    data = stage_view(authored_xml, output / 'resource-view', special_rooms)
    target = next(node for node in ET.fromstring(data) if node.tag == 'challenge' and node.get('id') == '1')
    ids = [int(value) for value in target.get('startingitems', '').split(',') if value]
    if ids != case['fixture']['startingItems']:
        raise ValueError('Actual declared XML must preserve the case ordered starting items')
    for change in case['fixture'].get('attributeChanges', []):
        if target.get(change['attribute']) != change['authoredValue']:
            raise ValueError('Actual declared XML must preserve every authored attribute change')
    (output / 'case.json').write_text(json.dumps(case, indent=2) + '\n')
    tape_path = output / 'declared-tape.json'
    tape_path.write_text(json.dumps(tape, indent=2) + '\n')
    metadata = {'schema': 1, 'scope': 'source-authored Challenge fixture, not pristine Challenge parity',
                'declaredXml': data.decode('utf-8'), 'nativeDepotChanged': False,
                'declaredResourceHashes': input_hashes,
                'specialRoomsPath': str(special_rooms) if special_rooms else None,
                'nativeObservationUsedAsInput': False, 'compositionSourceFields': source_fields, 'publicTape': str(tape_path),
                'bootstrapBoundary': 'Existing first-Title boundary: native intro is a source-only prelude; no browser intro or simulation-tick parity claim',
                'browserInputRequirement': 'exact declared XML and boot inputs; independently declared post-Title public commands; no native output import',
                'execution': 'prepared-not-executed', 'requiredActorContract': contract}
    metadata_path = output / 'composition-input.json'
    metadata_path.write_text(json.dumps(metadata, indent=2) + '\n')
    if args.prepare_only:
        print(json.dumps({'status': 'prepared-not-executed', 'output': str(output)}))
        return 0
    # ROOT selects only the immutable binary/resource depot in the shared
    # launcher. Its transport, observers, clock and storage code stay intact.
    expected_original = canonical_original()
    old_original_environment = os.environ.get('ISAAC_ORIGINAL_SHA')
    os.environ['ISAAC_ORIGINAL_SHA'] = expected_original
    metadata['nativeBinarySha256'] = expected_original
    clock.ROOT = output / 'resource-view'
    old_argv, old_observation = sys.argv, clock.checked_observation
    def source_observation(observer, pid, context):
        context['compositionSourceFields'] = source_fields
        return old_observation(observer, pid, context)
    clock.checked_observation = source_observation
    try:
        sys.argv = [str(ROOT / 'tools/original/foundation-clock-run.py'), '--challenge-run',
                    '--boot-epoch-seed', str(epoch), '--tape', str(tape_path), '--output', str(output), *extra]
        metadata['execution'] = 'original-started'
        result = clock.main()
        metadata['execution'] = 'original-closed'
        metadata['originalExitStatus'] = result
        return result
    except BaseException as error:
        metadata['execution'] = 'original-failed'
        metadata['originalError'] = str(error)
        raise
    finally:
        sys.argv, clock.checked_observation = old_argv, old_observation
        if old_original_environment is None:
            os.environ.pop('ISAAC_ORIGINAL_SHA', None)
        else:
            os.environ['ISAAC_ORIGINAL_SHA'] = old_original_environment
        hashes_after = {'challengesXml': hashlib.sha256(authored_xml.read_bytes()).hexdigest()}
        if special_rooms is not None:
            hashes_after['specialRooms'] = hashlib.sha256(special_rooms.read_bytes()).hexdigest()
        metadata['declaredResourceHashesAfter'] = hashes_after
        metadata['declaredResourcesBeforeEqualsEnd'] = hashes_after == input_hashes
        metadata_path.write_text(json.dumps(metadata, indent=2) + '\n')
        if hashes_after != input_hashes:
            raise RuntimeError('Declared resource bytes changed during the original capture')


if __name__ == '__main__':
    raise SystemExit(main())
