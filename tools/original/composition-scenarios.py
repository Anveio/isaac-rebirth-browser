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
    opposite_shoot = 'Left' if shoot == 'Right' else 'Right'
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
        # Keep charging while approaching; release near public contact and aim
        # across the authored room. Actual maximum charge/contact/survival
        # remain witnesses, never inferred from these durations.
        ('ordinary-knife-charge', [opposite_shoot], 160),
        ('approach-329-while-charged', [move, 's', opposite_shoot], 12),
        ('release-knife-and-move-to-329', [move, 's'], 32),
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


def ordinary_brim_contact_case(name, phase, epoch, receiver=14, side=0,
                               timing=0, token_index=None):
    directions = [('Right', 'Up', 'Left', 'Down'),
                  ('Left', 'Down', 'Right', 'Up')][side]
    if receiver == 18:
        directions = ('Down', 'Right', 'Up', 'Left')
    actions = [dict(label='receiver-ready', keys=[], presentations=1)]
    for index, direction in enumerate(directions):
        actions.extend([
            dict(label=f'charge-{index}-{direction.lower()}', keys=[direction],
                 presentations=120 + timing),
            dict(label=f'release-{index}-and-expire', keys=[], presentations=64),
        ])
    case = {
        'name': name,
        'fixture': {'kind': 'source-authored-modified-challenge', 'challengeId': 1,
                    'startingItems': [118], 'view': 'item-only'},
        'bootEpochSeed': epoch, 'initialPhaseOffset': phase,
        'entropy': 'fixed declared public contact case' if token_index is None
            else 'existing unfiltered OS token reused without a new draw',
        'actions': actions,
        'requires': {
            'actualPublicChallengeEntry': True,
            'bothGamesConsumeIdenticalDeclaredResourceBytes': True,
            'ownedCounts': [{'itemId': 118, 'count': 1}],
            'sourceExpectedActorContract': actor_stage([118], 2, 7, 0, False, [])[
                'sourceExpectedActorContract'],
            'ordinaryBrimContact': {
                'authoredReceiver': {'type': receiver, 'variant': 0, 'subtype': 0},
                'receiverVariantPolicy': 'Preserve actual spawnConfig variants, champions and health; no assumed variant0/HP or identity aliases.',
                'spawnGroup': ({'x': 6, 'y': 5} if receiver == 18 else
                               {'x': 9 if side == 0 else 3, 'y': 3}),
                'publicAimPolicy': 'Stationary four-cardinal charge/release sweep; no observed-position targeting or conditional repair.',
                'requiredActualWitnesses': [
                    'Actual receiver admission, independent identity/seed and own manager membership in each game.',
                    'Actual registered ParentPlayer Laser birth and positive-TTL Full and Fast bodies.',
                    'Actual eligible receiver intersecting an actual Laser Line segment, with raw eligibility/geometry retained.',
                    'At least one accepted FIFO append from this Laser callback: actual ParentPlayer sourceType1, flags8 and amount; raw ordered source payload retained independently.',
                    'The same actual receiver on its next Full callback: ordered pending FIFO drained, applied health change and pending state compared. Nonlethal damage qualifies; enqueue alone does not.',
                    'Current kind3 property outputs compared after the real receiver callback; static source order is not a dynamic call-count witness.',
                    'Same Laser positive TTL to zero/fade and following-Full absence, actual C18 NULL and matching weak-link cleanup; final inventory alone is insufficient.',
                ],
                'minimumAcceptedEnqueuesPerGame': 1,
                'minimumNextFullDamageApplicationsPerGame': 1,
                'minimumNaturalLaserRetirementsPerGame': 1,
                'receiverDeathPolicy': 'When actual damage reaches the source death predicate, retain death/cleanup witnesses; do not require a fitted lethal first hit.',
                'sourcePayloadScope': 'Laser476A60 ParentPlayer DamageInfo, flags8/countdown30. Do not substitute Tear/Dip flags0, a Laser-as-source alias, Tear dying, or generic impulse.',
                'countdownScope': '476A60 passes countdown30 as a callback argument; it is not a stored FIFO field or a claimed dynamic call-argument witness.',
                'requiredToolingStatus': 'pending scoped typed NPC14/18 observer and Laser FIFO comparer extension',
                'missingToolingMustFail': True,
            },
            'actualPendingPrimaryCleanup': 'must be present and null',
            'actualCachePendingFlags': 0,
            'canonicalDiagnosticOwner': 'weaponAdmission.unadmittedController must be present and null',
            'unknownDiagnosticOrMissingCoverageIsFailure': True,
            'preserveAllIdentitySeedRngInitNumericFamilyAndCoverageMismatches': True,
            'nativeObservationIsNeverBrowserInput': True,
            'sourceInferredGrantPath': {'orderedItems': [118],
                'each': '52E1A0 →52CC90 →511F10 →581750',
                'final': '511F10 then3074=true', 'nativeDynamicCallCountObserved': False},
            'unqualifiedCoverage': ['DOM/render/pixel parity', 'explicit native Remove/Uninit callback events',
                'full624 RNG parity', 'full copied receiver-payload history beyond recorded owners'],
        },
        'execution': {'status': 'not-executed', 'blockingPrerequisites': [
            'Immutable owned producer with ordinary Brimstone execution',
            'Typed actual NPC14/18 eligibility, ordered receiver FIFO and next-Full health observation/comparison; existing217-only law is insufficient',
            'Integration-owner serial game/storage lease',
        ], 'neverSatisfiedBy': 'Admission alone, static overlap, hypothetical enqueue, empty scenes, disappearance, unsupported receiver aliases, or outcome filtering'},
    }
    if token_index is not None:
        case['entropyTokenIndex'] = token_index
    return case



def ordinary_technology_case(name, phase, epoch, receiver=14, side=0,
                             timing=0, rotation=0, canceled_extra=0, token_index=None):
    directions = ('Right', 'Up', 'Left', 'Down')
    directions = directions[rotation:] + directions[:rotation]
    actions = [dict(label='technology-ready', keys=[], presentations=1)]
    for index, direction in enumerate(directions):
        actions.extend([
            dict(label=f'held-{index}-{direction.lower()}', keys=[direction],
                 presentations=24 + timing),
            dict(label=f'released-{index}-natural-expiry', keys=[], presentations=16),
        ])
    actions.extend([
        dict(label='canceled-held-prior-db4', keys=['Right', 'Left'],
             presentations=10 + canceled_extra),
        dict(label='final-release-natural-expiry', keys=[], presentations=24),
    ])
    contract = dict(primaryC08=3, family=7, variant=2, subtype=0,
        registeredC18DuringActualLifetime=False, managerRoomPersistent=False,
        controlledBy329=False,
        requiredWhen='actual held ordinary519FC0 with C74<0; manager birth independently of C18',
        actualParentType=1, sourceInitializedLifetime=3,
        sourceInitializationScope='Source factory lifetime3 is followed immediately by actual Full; the first observed remaining lifetime may already be2. Do not demand an impossible post-callback3.',
        sourceC18Scope='Fresh sole68 has no519130/C18 bind; compare actual NULL throughout. NULL is not prior-actor cleanup evidence.')
    case = dict(
        name=name,
        fixture=dict(kind='source-authored-modified-challenge', challengeId=1,
                     startingItems=[68], view='item-only'),
        bootEpochSeed=epoch, initialPhaseOffset=phase,
        entropy='fixed declared ordinary Technology case' if token_index is None
            else 'both literal existing OS records reused without new draws or selection',
        actions=actions,
        requires=dict(
            actualPublicChallengeEntry=True,
            bothGamesConsumeIdenticalDeclaredResourceBytes=True,
            nativeObservationIsNeverBrowserInput=True,
            ownedCounts=[dict(itemId=68, count=1)],
            sourceExpectedActorContract=contract,
            actorStages=[dict(ownedCounts=[dict(itemId=68, count=1)],
                              sourceExpectedActorContract=contract,
                              requiredWitnesses=['Actual ordinary7/2/0 manager births and Full/Fast bodies; no persistent-slot alias.'])],
            ordinaryTechnologyFunctional=dict(
                schema='ordinary-technology-properties-lifetime-v1',
                declaredCountsOnlySelectStage=True,
                role='each game own actual manager7/2/0 with actual ParentPlayer498; never use C18 as Laser identity',
                minimumActualBirthsPerGame=1,
                minimumActualFullBodyRowsPerGame=1,
                minimumActualFastBodyRowsPerGame=1,
                minimumSameIdentityNaturalRetirementsPerGame=1,
                requiredActualFields=[
                    'Independent real identity/spawn seed/birth age/actor RNG; compare every observed eligible actor, not fitted identities.',
                    'Player C08/C74/BB0 and actual held/released/canceled callback aim, retained DB4/DA0/D9C; no C0C charge/release contract is substituted.',
                    'Parent498, relativeParentOffset7D4, physical448/saved450/render458, velocity/friction, actual fast322,31E/31F and both collision classes.',
                    'Born C98/7E8 and separate818 flags, CCC/source/current colors, actual kind2 damage48C; unknown HP/maxHP must remain missing failures.',
                    'Angle/previousAngle, actual Line50/mode0 initialization and width120; compare actual controls/points/.7 resampling/class5 clipping without forcing a post-resample count50.',
                    'Actual positive remaining lifetime through zero/31F, subsequent Remove/31Efalse and following Full manager absence with own Parent/Child weak links. Preserve partial/event gaps separately.',
                ],
                birthAndLivePropertyScope='50E1C0 copies C98/CCC then calls real kind2 AFTER Spawn. Kind3 is current after actual receiver enqueue. Do not broadcast current damage/color to older actors or claim a dynamic callback count from source order.',
                ttlScope=contract['sourceInitializationScope'],
                noBrimstoneFade=True,
                actualC18='must be present and NULL throughout this fresh sole68 route; no C18 bind/reapply/cleanup credit',
                actualSecondarySlots='Compare actual C20/C28/C30 separately; selected no55/no87 NULLs are not cleanup witnesses.',
                impact=dict(entityType=1000, variant=50, subtype=0,
                    minimumActualBirthsPerGame=1,
                    actor='gfx/1000.050_laserimpact.anm2', defaultAnimation='Idle',
                    sourceAuthoredAnimationScope='Actual canonical Idle24 looping; no BrimStart/Loop substitution or inferred pose.',
                    requiredActualFields=['Independent identity/seed/actual manager membership and matching ownParent498/LaserChild4A0.',
                        'Source/current color, actual physical448/saved450/render458, scale and Sprite state.',
                        'Actual Sprite flip2FC and separate raw814; require real fields rather than guessing false/0 or aliasing raw814 to Sprite rotation.',
                        'Actual own-parent death/NULL, dead versus Remove, later Full manager absence and exact matching weak-link cleanup.'],
                    sourceBoundOrder='SharedRaw Effect Spawn, SetChild/SetParent, Bound2 flip, Bound4*90 raw814, v58 color, immediate Full.',
                    nativeDynamicCallCountObserved=False),
                unknownOrMissingFieldsMustFail=True),
            ordinaryTechnologyContact=dict(
                authoredReceiver=dict(type=receiver, variant=0, subtype=0),
                spawnGroup=dict(x=6 if receiver == 18 else (9 if side == 0 else 3),
                                y=5 if receiver == 18 else 3),
                publicAimPolicy='Stationary four-cardinal held/released sweep plus canceled-held; no observed-position targeting, conditional repair or Native state input.',
                receiverVariantPolicy='Preserve real factory variant/subtype/champion/health and independently compare actual manager identity; authored selector is not a fitted actor alias.',
                minimumActualAcceptedEnqueuesPerGame=1,
                minimumActualNextFullDamageApplicationsPerGame=1,
                requiredActualWitnesses=[
                    'Actual eligible overlapping receiver with true class/flags/geometry and actual manager7/2/0 Laser.',
                    'Real ordered FIFO append with amount/flags8/sourceType1 and the actual ParentPlayer copied reference, followed by the same receiver next-Full health application.',
                    'A real bounded source window or event must attribute enqueue to this Laser callback; ParentPlayer plus matching amount or unique current Laser alone is insufficient.',
                ],
                sourceCountdownScope='476A60 callback countdown30 is source-only until an actual call witness; it is not a stored FIFO field.',
                requiredToolingStatus='Pending typed14/18 plus ordinary-Technology manager-role/queue-window continuation; existing217 or ordinary-Brim C18 proof is insufficient.',
                missingContactCoverageRemainsGapAndFail=True),
            compositionSourceFields=['playerMovementD9c', 'knifeChildChain4a0'],
            additionalRequiredObservationFieldsPending=['Manager7/2/0 role and explicit Technology contract consumption',
                'Impact50/0 actual flip2FC/raw814 and independent Sprite/frame ownership',
                'Typed14/18 ordered FIFO with Laser ParentPlayer flags8 and actual source window'],
            actualPendingPrimaryCleanup='must be present and null',
            actualCachePendingFlags=0,
            canonicalDiagnosticOwner='weaponAdmission.unadmittedController must be present and null',
            preserveAllIdentitySeedRngInitNumericFamilyAndCoverageMismatches=True,
            unknownDiagnosticOrMissingCoverageIsFailure=True,
            sourceInferredGrantPath=dict(orderedItems=[68], each='52E1A0→52CC90→511F10→581750',
                final='511F10 then3074=true', nativeDynamicCallCountObserved=False),
            coverageGapsRequiringSeparateActualEvidence=[
                'Explicit Native Remove and Uninit callback events/precedence; GL disappearance alone is not this event witness.',
                'Exact contact enqueue attribution/current kind3 callback timing and complete copied FIFO history.',
                'DOM/render/pixel, Technology wardrobe presentation and Effect raw814 render projection.',
                'Full624 word/index and complete actual actor-init rosters remain separately graded; no seed/cursor proxy.',
            ]),
        execution=dict(status='not-executed', wholeCaseAccepted=False,
            blockingPrerequisites=['Owned qualified ordinary-Technology producer and integration-owner serial game/storage lease',
                'Every declared contract must be consumed explicitly by its grader; missing manager-role/event/contact/render observation stays gap/failure.'],
            neverSatisfiedBy='Static counts/source proof, an invented C18/pose/default, disappearance alone, borrowed WASM, Native-state injection, seed reroll or dropping failed cases'))
    if token_index is not None:
        case['entropyTokenIndex'] = token_index
    return case


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--repository', type=Path, default=ROOT)
    parser.add_argument('--suite', choices=('history', 'ordinary-brim-contact', 'ordinary-technology'), default='history')
    parser.add_argument('--entropy-journal', type=Path,
                        help='Reuse every literal token from an existing declaration; never draws again.')
    args = parser.parse_args()
    if not 0 <= args.count <= 8:
        raise ValueError('count must be 0..8; all declared outcomes are retained')
    contact = args.suite == 'ordinary-brim-contact'
    technology = args.suite == 'ordinary-technology'
    if (contact or technology) and args.entropy_journal is None:
        raise ValueError('Contact/Technology declaration requires an existing entropy journal; no new draws')
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
    cases = ([ordinary_technology_case(f'technology-fixed-phase{phase}', phase, 0x31A9)
              for phase in (0, 1)] if technology else
             ([ordinary_brim_contact_case(f'brim-contact-fixed-phase{phase}', phase, 0x31A9)
               for phase in (0, 1)] if contact else
              [history_case(f'history-118-114-329-phase{phase}', phase, 0x31A9) for phase in (0, 1)]))
    for entry in entries:
        token = bytes.fromhex(entry['tokenHex'])
        if technology:
            cases.append(ordinary_technology_case(f'technology-recorded-{entry["index"]}',
                token[0] % 2, int.from_bytes(token[4:8], 'little'),
                (14, 18)[token[2] % 2], token[1] % 2, token[2] % 13,
                token[8] % 4, token[9] % 3, entry['index']))
        elif contact:
            cases.append(ordinary_brim_contact_case(f'brim-contact-recorded-{entry["index"]}',
                token[0] % 2, int.from_bytes(token[4:8], 'little'),
                (14, 18)[token[2] % 2], token[1] % 2, token[2] % 13, entry['index']))
        else:
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
        node.set('startingitems', ','.join(map(str, case['fixture']['startingItems'])))
        xml = output / case['name'] / 'resources/challenges.xml'
        xml.parent.mkdir(parents=True)
        ET.ElementTree(root).write(xml, encoding='utf-8', xml_declaration=True)
        if contact or technology:
            receiver = case['requires'][
                'ordinaryTechnologyContact' if technology else 'ordinaryBrimContact']
            groups = [dict(**receiver['spawnGroup'], entries=[dict(
                **receiver['authoredReceiver'], weight=1.0)])]
        else:
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
        publicRouteRevision='ordinary-technology-v1' if technology else ('ordinary-brim-contact-v1' if contact else 'maxcharge-near-contact-opposite-aim-v3'),
        scope='Declared ordinary Technology7/2/0 manager bodies/TTL/Impact/contact; no execution or acceptance' if technology else ('Declared ordinary-Brim receiver contact; no execution or acceptance' if contact else 'Declared live118→114→329 public-history successor; no execution or acceptance'),
        selection='Two fixed phases plus every journaled unfiltered OS projection; no outcome filtering or rerolls',
        outcomePolicy='Missing real stages/grants/actors/children/cleanup/bodies and every mismatch remain failures'))
    print(json.dumps(dict(status='declared-not-executed', cases=len(cases), scenarios=str(output / 'scenarios.json'))))


if __name__ == '__main__':
    main()
