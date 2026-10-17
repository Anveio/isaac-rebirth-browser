"""Public Pill acquisition/use cases: fixed inputs and every fresh OS epoch.

Generate resource fixtures locally from the existing base assets. No game
output supplies a seed, held record, counter, pool assignment or statistic.
"""
import argparse
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUN_SEED, SEED_CODE = 12713, 'B91Y6ZEV'
COLORS = {11: 9, 12: 7}


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--count', type=int, default=1)
    args = parser.parse_args()
    if not 0 <= args.count <= 64:
        parser.error('Fresh unfiltered OS count must be0..64')
    codec = load('pill_use_seed_codec', ROOT / 'tools/original/foundation-seeds.py')
    rooms_codec = load('pill_use_room_codec', ROOT / 'tools/foundation/decode-rooms.py')
    if codec.encode_seed(RUN_SEED) != SEED_CODE or codec.decode_seed(SEED_CODE) != RUN_SEED:
        raise ValueError('The declared public Pill fixture seed is invalid')
    baseline = ROOT / 'references/isaac/rebirth/resources'
    stb_path, xml_path = baseline / 'rooms/00.special rooms.stb', baseline / 'challenges.xml'
    stb_raw = stb_path.read_bytes()
    rooms = rooms_codec.decode_bytes(stb_raw, source=str(stb_path))
    if rooms_codec.encode_bytes(rooms) != stb_raw:
        raise ValueError('Base room codec must preserve every input byte')
    room = rooms[80]
    if (room['type'], room['variant'], room['width'], room['height']) != (1, 2, 13, 7):
        raise ValueError('Public Pill fixture requires the actual base starting-room record')
    old_groups = copy.deepcopy(room['spawns'])
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    resources = output / 'resources'
    resources.mkdir()
    xml = resources / 'challenges.xml'
    xml.write_bytes(xml_path.read_bytes())
    fixtures = {}
    for effect, color in COLORS.items():
        groups = [dict(x=6, y=5, entries=[dict(type=5, variant=70, subtype=color, weight=1.0)])]
        room['spawns'] = copy.deepcopy(groups)
        path = resources / f'pill-range{effect}-special.stb'
        raw = rooms_codec.encode_bytes(rooms)
        path.write_bytes(raw)
        inverse = rooms_codec.decode_bytes(raw, source=str(path))
        inverse[80]['spawns'] = copy.deepcopy(old_groups)
        if rooms_codec.encode_bytes(inverse) != stb_raw:
            raise ValueError('Pill fixture changed a room, door, weight or metadata outside its declared spawn groups')
        fixtures[effect] = dict(path=path, groups=groups,
            sha256=hashlib.sha256(raw).hexdigest())

    actions = [dict(label='released-initial-pill-appear-idle', keys=[], presentations=96),
        dict(label='public-known-empty-pocket-control', keys=['q'], presentations=2),
        dict(label='released-empty-pocket-control', keys=[], presentations=2)]
    for number in range(1, 4):
        actions.extend([dict(label=f'pill-acquire-short-north-{number}', keys=['w'], presentations=4),
            dict(label=f'pill-acquire-released-ceremony-{number}', keys=[], presentations=192)])
    actions.extend([dict(label='public-held-pill-q-use', keys=['q'], presentations=2),
        dict(label='released-after-real-pill-use', keys=[], presentations=96),
        dict(label='public-post-pill-range-shoot-right', keys=['Right'], presentations=40),
        dict(label='released-post-pill-projectile-flight', keys=[], presentations=48)])
    cases = []

    def declare(effect, name, epoch, phase, entropy):
        fixture, color = fixtures[effect], COLORS[effect]
        case = dict(name=name, publicMenuRoute='seeded-normal', publicRunSeed=RUN_SEED,
            publicSeedCode=SEED_CODE, bootEpochSeed=epoch, initialPhaseOffset=phase,
            entropy=entropy, sfxVolume=1,
            fixture=dict(kind='source-authored-start-room-STB-on-public-seeded-normal-run',
                challengeId=1, startingItems=[], view='item-only',
                authoredResourcePath=str(xml), baselineResourcePath=str(xml_path),
                specialRoomsPath=str(fixture['path']), specialRoomsSha256=fixture['sha256'],
                attributeChanges=[], challengeConfigurationUsed=False,
                startRoomSpawnChange=dict(catalogue=0, index=80, type=1, variant=2,
                    operation='replace', originalGroupCount=len(old_groups),
                    groups=copy.deepcopy(fixture['groups']),
                    allOtherRowsMetadataDoorsWeightsOrderPreserved=True)),
            actions=copy.deepcopy(actions),
            pocketCollection=dict(schema='source-public-pocket-collection-v1', originalRoomGroupCount=0,
                replacesOriginalRoomGroups=True, groups=copy.deepcopy(fixture['groups']),
                expectedSlot0History=[dict(id=color, card=False)], minimumCollectedSelectors=1,
                minimumObservedCapacityEjections=0, minimumActualCardRequests=0,
                minimumActualPillRequests=1, minimumActualUseItemPresentations=1,
                minimumObservedPositiveDropAD0=0, minimumObservedDropAD0Refresh=0,
                minimumFullObservations=1, minimumFastObservations=1,
                scope='Public resource/reset/acquisition transport only. This positive-use case requires '
                    'the distinct Pill-use reader; the collection-only comparer does not admit consumption.'),
            pillUse=dict(schema='source-public-pill-range-use-v1', effectId=effect, pillColor=color,
                qUseAction='public-held-pill-q-use', minimumActualUses=1,
                required=['ActualEmptyStartAndPublicMenuResourceReset', 'ActualWorldPillAcquisition',
                    'ActualKnownHeldPillBeforeDeliveredQ', 'ActualIndependentColorToEffectPoolOwner',
                    'ActualNeutralHave75And303BeforeResolver', 'ActualCanonicalCounterIncrementAndConsumedPocket',
                    'ActualCachedRangeAndPublicPostUseShooting', 'ActualContinuedUpdatesAndCompleteJournal'],
                unknown=['BackendPlayedPillAudioAndPCM', 'FullFrameBrowserPixels',
                    'PrivateCallbackEntryAndSynchronousCounterRequestEvaluateSlotOrdering',
                    'OtherPillEffectsAndPlayers', 'ActualPostUseTearBirthParameters'],
                missingWitnessPolicy='FAIL and retain missing actual owners/coverage; no injected counter/color/state, '
                    'no redraw, output-selected seed, empty-Q substitution or endpoint-to-call inference.'))
        reset = dict(challengeXml=xml.read_text(), specialRooms=list(fixture['path'].read_bytes()))
        if len(json.dumps(reset, separators=(',', ':')).encode()) > 512 * 1024:
            raise ValueError('Declared Pill resources exceed the existing public reset bound')
        cases.append(case)

    #These are declarations, not a source-pool prediction used as an oracle.
    #The real-game reader must independently resolve each actual pool/color.
    for effect, phase in ((11, 0), (12, 1)):
        declare(effect, f'pill-range{effect}-real-q-fixed-phase{phase}', 12713, phase,
            'Fixed source-derived manual seed and color, declared before either game')
    with (output / 'os-entropy.jsonl').open('x') as journal:
        for index in range(args.count):
            for effect in (11, 12):
                raw = os.urandom(4)
                epoch = int.from_bytes(raw, 'little')
                choice = dict(rawHex=raw.hex(), bootEpochSeed=epoch, initialPhaseOffset=(epoch >> 31) & 1,
                    rule='One direct unfiltered little-endianu32 public boot epoch, highbitphase; '
                        'the independently declared fixed manual run seed remains literal. No redraw/outcome filter.')
                journal.write(json.dumps(choice) + '\n')
                journal.flush()
                os.fsync(journal.fileno())
                suffix = '' if index == 0 else '-' + str(index)
                declare(effect, f'pill-range{effect}-real-q-fresh-os-epoch{suffix}', epoch,
                    choice['initialPhaseOffset'], choice)
    result = dict(schema=1, cases=cases,
        source='Declared real Pill acquisition and positive publicQ, fixed12713/B91Y6ZEV with colors9→11/7→12. '
            'Every fresh unfiltered OS boot epoch and high-bit phase is journaled before games; no seed search or reroll. '
            'Actual independently observed pool/held/counter/cache owners remain the oracle.')
    (output / 'scenarios.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(status='declared-not-executed', cases=len(cases), output=str(output))))


if __name__ == '__main__':
    main()
