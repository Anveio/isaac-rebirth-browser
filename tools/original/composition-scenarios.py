#!/usr/bin/env python3
"""Declare public item-composition games; this does not execute or grade them."""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import secrets
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]


def write_json(path, value):
    with path.open('x') as handle:
        json.dump(value, handle, indent=2)
        handle.write('\n')
        handle.flush()
        os.fsync(handle.fileno())


def actor_stage(items, primary, family, subtype, controlled, witnesses):
    return {
        'ownedCounts': [{'itemId': item, 'count': 1} for item in sorted(items)],
        'sourceExpectedActorContract': {
            'primaryC08': primary, 'family': family,
            'variant': 1 if family == 7 else 0, 'subtype': subtype,
            'registeredC18DuringActualLifetime': True,
            'managerRoomPersistent': family == 8,
            'controlledBy329': controlled,
            'requiredWhen': 'actual charged release and registered lifetime' if family == 7
                else 'next actual admitted 519FC0 callback after the grant; preserve pickup/charge/control fences',
        },
        'requiredWitnesses': witnesses,
    }


def history_case(name, phase, epoch, side=0, timing=0, token_index=None):
    move, shoot = [('d', 'Right'), ('a', 'Left')][side]
    segments = [
        ('ready', [], 1),
        # A fixed public approach sweep, not a fitted native position or a
        # conditional repair. Actual pickup/grant witnesses remain required.
        ('approach-pedestal-row', ['w'], 28),
        ('settle-before-charge', [], 24),
        ('initial-brim-charge', [shoot], 120 + timing),
        # Same fixed release/movement as the separate Brim→Knife declaration.
        # Its live C18 at queued contact and later natural expiry are required
        # observations. The 42-full Pickup wait exceeds the Laser timeout15.
        ('release-brim-and-move-to-114', [move], 24),
        ('ordinary-knife-charge', [shoot], 120),
        ('release-knife-and-move-to-329', [move], 44),
        ('controlled-triple-dispatch', [shoot], 120),
        ('canceled-held', ['Right', 'Left'], 18),
        ('released', [], 60),
        ('vertical-steering', ['Up'], 80),
        ('final-release', [], 60),
    ]
    stages = [
        actor_stage([118], 2, 7, 0, False, [
            'At actual queued114 contact, observe the own live registered 7/1/0 C18 with positive TTL; then observe its same-identity natural timer expiry and own retirement during Pickup, independently of deferred Add114.',
        ]),
        actor_stage([118, 114], 4, 8, 0, False, [
            'Observe the actual deferred114 count/cache/C08 2→4 transition and actual C18/weak slots. Pickup42 exceeds LaserTTL15; this route does not qualify removal of a live Laser by selector change.',
            'Before 329, observe an actual ordinary Knife C18 and an actually born 8/0/1 child whose Parent498 resolves to that Knife. Missing parent or child fails coverage.',
        ]),
        actor_stage([118, 114, 329], 4, 8, 0, True, [
            'Observe the actual 329 queued grant with C08 still4; next live dispatch reuses the same existing type8 C18 and applies bit33.',
            'Observe the SAME preexisting ParentKnife subtype1 child continuing its copied curve after the parent becomes controlled, then its own Remove and later full Uninit. Conversion does not remove this child; missing continuation or retirement fails coverage.',
            'Observe controlled full/fast bodies and current C7C steering. Its dispatch must not emit a new ordinary 118 child or Laser; compare existing transient actors too.',
        ]),
    ]
    result = {
        'name': name,
        'fixture': {'kind': 'source-authored-modified-challenge', 'challengeId': 1,
                    'startingItems': [118], 'view': 'item-only'},
        'bootEpochSeed': epoch, 'initialPhaseOffset': phase,
        'entropy': 'fixed declared public history' if token_index is None else 'unfiltered OS token recorded before projection',
        'actions': [dict(label=label, keys=keys, presentations=count) for label, keys, count in segments],
        'requires': {
            'actualPublicChallengeEntry': True,
            'bothGamesConsumeIdenticalDeclaredResourceBytes': True,
            'orderedRealPedestalGrants': [114, 329],
            'ownedCounts': [{'itemId': 118, 'count': 1}],
            'sourceExpectedActorContract': stages[0]['sourceExpectedActorContract'],
            'staticActorContractScope': 'Initial stage only; classify each game from its own actual counts against actorStages.',
            'actorStages': stages,
            'lifecycleExperiment': 'brim-to-ordinary-to-controlled-triple',
            'historyWitnessContract': 'brim-ordinary-controlled-queued-lifetime-v2',
            'sourceExcludedObligations': ['Positive prior Laser TTL at deferred Add114 on uninterrupted Pickup42; ordinary release Laser TTL is15.'],
            'untestedHistoryFrontiers': ['Live selector-change Laser Remove and replacement-safe Uninit need a separately admitted public route.'],
            'actualBeforeAndAfterCountsRequired': True,
            'actualQueueGrantCacheAndControllerFencesRequired': True,
            'actualFree5_100SelectorContactAndQueuedGrantRequired': True,
            'canonicalDiagnosticOwner': 'weaponAdmission.unadmittedController must be present and null',
            'actualPendingPrimaryCleanup': 'must be present and null',
            'secondarySlotScope': 'Observe actual C20/C28/C30 values separately. This selected route has neither Have55 nor Have87 secondary producer; null slots are not cleanup evidence. Nonempty slot cleanup needs a separately admitted public producer. Parent498 and Base4A0 are independent owners.',
            'actualCachePendingFlags': 0,
            'preserveAllIdentitySeedRngInitNumericFamilyAndCoverageMismatches': True,
            'unknownDiagnosticOrMissingCoverageIsFailure': True,
            'sourceInferredGrantPath': {'orderedItems': [118],
                'each': '52E1A0 →52CC90 →511F10 →581750',
                'final': '511F10 then3074=true', 'nativeDynamicCallCountObserved': False},
            'inputAndFieldScope': 'Compare actual held/released/canceled input, retained DB4/DA0/C0C, counts, cache, actors and slots at each observed boundary. No fitted identities or source-equivalence across histories.',
        },
        'execution': {'status': 'not-executed', 'blockingPrerequisites': [
            'One immutable public producer with ordinary Brim, ordinary Knife and controlled Knife on canonical stores/C18, actual guards/Remove/Uninit',
            'Whole Special STB input and stage-aware live pickup/actor/child/cleanup observers and comparisons',
            'Owned production build and integration-owner game/storage lease',
        ], 'neverSatisfiedBy': 'Static grants, final inventory, source inference, missing actor/child/cleanup/body, native-state injection or seed rerolls'},
    }
    if token_index is not None:
        result['entropyTokenIndex'] = token_index
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--repository', type=Path, default=ROOT)
    parser.add_argument('--entropy-journal', type=Path,
                        help='Reuse every literal token from an existing declaration; never draws again.')
    args = parser.parse_args()
    if not 0 <= args.count <= 8:
        raise ValueError('count must be 0..8; all declared outcomes are retained')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    entries = []
    if args.entropy_journal:
        entries = [json.loads(line) for line in args.entropy_journal.read_text().splitlines() if line]
        if len(entries) != args.count:
            raise ValueError('Replay every journaled token without truncation or new draws')
    with (output / 'entropy.jsonl').open('x') as journal:
        for index in range(args.count):
            entry = entries[index] if args.entropy_journal else {
                'index': index, 'tokenHex': secrets.token_bytes(16).hex(),
                'source': 'secrets.token_bytes(16)', 'recordedBeforeProjectionAndGames': True}
            if entry['index'] != index or len(bytes.fromhex(entry['tokenHex'])) != 16:
                raise ValueError('Expected ordered literal 16-byte entropy records')
            journal.write(json.dumps(entry) + '\n')
            journal.flush()
            os.fsync(journal.fileno())
            if not args.entropy_journal:
                entries.append(entry)
    cases = [history_case(f'history-118-114-329-phase{phase}', phase, 0x31A9) for phase in (0, 1)]
    for entry in entries:
        token = bytes.fromhex(entry['tokenHex'])
        cases.append(history_case(f'history-random-{entry["index"]}', token[0] % 2,
            int.from_bytes(token[4:8], 'little'), token[1] % 2, token[2] % 13, entry['index']))
    repository = args.repository.resolve()
    resources = repository / 'references/isaac/rebirth/resources'
    xml_path, stb_path = resources / 'challenges.xml', resources / 'rooms/00.special rooms.stb'
    baseline_xml, baseline_stb = xml_path.read_bytes(), stb_path.read_bytes()
    spec = importlib.util.spec_from_file_location('composition_rooms', repository / 'tools/foundation/decode-rooms.py')
    rooms_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(rooms_module)
    rooms = rooms_module.decode_bytes(baseline_stb, source=str(stb_path))
    starts = [index for index, room in enumerate(rooms) if (room['type'], room['variant']) == (1, 2)]
    if len(starts) != 1:
        raise ValueError('Expected the unique Special start room')
    for case in cases:
        root = ET.fromstring(baseline_xml)
        definitions = [node for node in root if node.tag == 'challenge' and node.get('id') == '1']
        if len(definitions) != 1:
            raise ValueError('Expected the unique Challenge1')
        node = definitions[0]
        changes = []
        for name, value in [('getcurse', '0'), ('cursefilter', '127'), ('roomfilter', None)]:
            changes.append(dict(attribute=name, baselineValue=node.get(name), authoredValue=value))
            if value is None:
                node.attrib.pop(name, None)
            else:
                node.set(name, value)
        node.set('startingitems', '118')
        xml = output / case['name'] / 'resources/challenges.xml'
        xml.parent.mkdir(parents=True)
        ET.ElementTree(root).write(xml, encoding='utf-8', xml_declaration=True)
        release = next(action for action in case['actions']
                       if action['label'] == 'release-brim-and-move-to-114')
        side = 0 if release['keys'] == ['d'] else 1
        groups = [dict(x=column, y=3, entries=[dict(type=5, variant=100, subtype=item, weight=1.0)])
                  for column, item in zip(([7, 9], [5, 3])[side], [114, 329])]
        authored = copy.deepcopy(rooms)
        authored[starts[0]]['spawns'].extend(groups)
        stb = output / case['name'] / 'resources/rooms/00.special rooms.stb'
        stb.parent.mkdir(parents=True)
        stb.write_bytes(rooms_module.encode_bytes(authored))
        case['fixture'].update(authoredResourcePath=str(xml), baselineResourcePath=str(xml_path),
            specialRoomsPath=str(stb), specialRoomsSha256=hashlib.sha256(stb.read_bytes()).hexdigest(),
            otherChallengeAttributes='preserve all rows and attributes except explicit neutral changes and startingitems',
            attributeChanges=changes, startRoomSpawnChange=dict(catalogue=0, index=starts[0], type=1,
                variant=2, operation='append', groups=groups, allOtherRowsMetadataDoorsWeightsOrderPreserved=True))
        case['requires'].update(actualSourceCurseSamplerDrawsBeforeMask=True, actualLevelCurseMask=0)
        reset = dict(seed=1, bootEpochSeed=case['bootEpochSeed'], trace=True,
            resources=dict(challengesXml=xml.read_text(), specialRooms=list(stb.read_bytes())))
        size = len(json.dumps(reset, separators=(',', ':')).encode())
        if size > 512 * 1024:
            raise ValueError('Authored public reset exceeds its boot-resource transport')
        case['fixture']['declaredResetJsonBytes'] = size
    write_json(output / 'scenarios.json', dict(schema=1, cases=cases,
        publicRouteRevision='upward-sweep-28-settle24-v1',
        scope='Declared live118→114→329 public-history successor; no execution or acceptance',
        selection='Two fixed phases plus every journaled unfiltered OS projection; no outcome filtering or rerolls',
        outcomePolicy='Missing real stages/grants/actors/children/cleanup/bodies and every mismatch remain failures'))
    print(json.dumps(dict(status='declared-not-executed', cases=len(cases), scenarios=str(output / 'scenarios.json'))))


if __name__ == '__main__':
    main()
