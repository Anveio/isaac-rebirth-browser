"""Public active-item acquisition/use cases: fixed inputs and fresh OS epochs.

Each case authors free collectible pedestals into the base starting room of
a public seeded normal run, walks onto them and presses Space. Fixtures are
generated locally from the existing base resources; no game output supplies
a seed, held record, charge or position.
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
#Actives whose 522250 leaf is ported (engine/src/active_items.rs USES).
#The dice have their own multi-pedestal cases below.
ACTIVES = {33: 'bible', 34: 'belial', 35: 'necronomicon', 45: 'yum-heart', 58: 'shadows',
           78: 'revelations', 282: 'how-to-jump', 286: 'blank-card', 292: 'satanic-bible',
           85: 'deck-of-cards', 102: 'moms-bottle-of-pills', 133: 'guppys-paw', 146: 'prayer-card',
           296: 'converter', 291: 'flush', 39: 'moms-bra', 40: 'kamikaze',
           41: 'moms-pad', 97: 'book-of-sin', 158: 'crystal-ball'}


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def acquire(prefix='acquire', key='w', direction='north'):
    actions = []
    for number in range(1, 4):
        actions.extend([dict(label=f'{prefix}-short-{direction}-{number}', keys=[key], presentations=4),
            dict(label=f'{prefix}-released-ceremony-{number}', keys=[], presentations=96)])
    return actions


def use(label, keys, after):
    return [dict(label=f'{label}-space', keys=sorted(keys + ['space']), presentations=2),
        dict(label=f'{label}-after', keys=sorted(keys), presentations=after),
        dict(label=f'{label}-released', keys=[], presentations=96)]


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--count', type=int, default=1)
    parser.add_argument('--npc-freeze', action='store_true',
        help='Add target-bearing Mom\'s Bra fixed phases and the same first unfiltered OS draw')
    args = parser.parse_args()
    if not 0 <= args.count <= 64:
        parser.error('Fresh unfiltered OS count must be0..64')
    codec = load('active_items_seed_codec', ROOT / 'tools/original/foundation-seeds.py')
    rooms_codec = load('active_items_room_codec', ROOT / 'tools/foundation/decode-rooms.py')
    if codec.encode_seed(RUN_SEED) != SEED_CODE or codec.decode_seed(SEED_CODE) != RUN_SEED:
        raise ValueError('The declared public active fixture seed is invalid')
    baseline = ROOT / 'references/isaac/rebirth/resources'
    stb_path, xml_path = baseline / 'rooms/00.special rooms.stb', baseline / 'challenges.xml'
    stb_raw = stb_path.read_bytes()
    rooms = rooms_codec.decode_bytes(stb_raw, source=str(stb_path))
    if rooms_codec.encode_bytes(rooms) != stb_raw:
        raise ValueError('Base room codec must preserve every input byte')
    room = rooms[80]
    if (room['type'], room['variant'], room['width'], room['height']) != (1, 2, 13, 7):
        raise ValueError('Active fixture requires the actual base starting-room record')
    old_groups = copy.deepcopy(room['spawns'])
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    resources = output / 'resources'
    resources.mkdir()
    xml = resources / 'challenges.xml'
    xml.write_bytes(xml_path.read_bytes())
    fixtures = {}

    def fixture(key, pedestals, npc_groups=()):
        if key in fixtures:
            return fixtures[key]
        groups = [dict(x=row[-2], y=row[-1], entries=[dict(type=5, variant=100 if len(row) == 3 else row[0],
                  subtype=row[0] if len(row) == 3 else row[1], weight=1.0)]) for row in pedestals]
        groups.extend(copy.deepcopy(npc_groups))
        room['spawns'] = copy.deepcopy(groups)
        path = resources / f'active-{key}-special.stb'
        raw = rooms_codec.encode_bytes(rooms)
        path.write_bytes(raw)
        inverse = rooms_codec.decode_bytes(raw, source=str(path))
        inverse[80]['spawns'] = copy.deepcopy(old_groups)
        if rooms_codec.encode_bytes(inverse) != stb_raw:
            raise ValueError('Active fixture changed a room outside its declared spawn groups')
        fixtures[key] = dict(path=path, groups=groups, sha256=hashlib.sha256(raw).hexdigest())
        return fixtures[key]

    idle = [dict(label='released-initial-appear-idle', keys=[], presentations=96)]
    cases = []

    def declare(name, pedestals, actions, epoch, phase, entropy, uses, npc_freeze=False, freeze_control=False):
        key = '-'.join('.'.join(str(value) for value in row[:-2]) for row in pedestals)
        npc_groups = [dict(x=x, y=y, entries=[dict(type=kind, variant=0, subtype=0, weight=1.0)])
                      for x, y, kind in ((10, 5, 12), (1, 1, 18))] if npc_freeze else []
        if npc_freeze:
            key += '-npc-freeze12-18'
        authored = fixture(key, pedestals, npc_groups)
        cases.append(dict(name=name, publicMenuRoute='seeded-normal', publicRunSeed=RUN_SEED,
            publicSeedCode=SEED_CODE, bootEpochSeed=epoch, initialPhaseOffset=phase,
            entropy=entropy, sfxVolume=1,
            fixture=dict(kind='source-authored-start-room-STB-on-public-seeded-normal-run',
                challengeId=1, startingItems=[], view='item-only',
                authoredResourcePath=str(xml), baselineResourcePath=str(xml_path),
                specialRoomsPath=str(authored['path']), specialRoomsSha256=authored['sha256'],
                attributeChanges=[], challengeConfigurationUsed=False,
                startRoomSpawnChange=dict(catalogue=0, index=80, type=1, variant=2,
                    operation='replace', originalGroupCount=len(old_groups),
                    groups=copy.deepcopy(authored['groups']),
                    allOtherRowsMetadataDoorsWeightsOrderPreserved=True)),
            actions=copy.deepcopy(actions),
            pocketCollection=dict(schema='source-public-pocket-collection-v1', originalRoomGroupCount=0,
                replacesOriginalRoomGroups=True, groups=copy.deepcopy(authored['groups']),
                scope='Public resource/reset transport only; the active reader owns every functional law.'),
            activeItems=dict(schema='source-public-active-items-v1',
                pedestals=[row[0] for row in pedestals if len(row) == 3], uses=uses,
                required=['ActualActiveSlotAndCharge', 'ActualTemporaryRecords', 'ActualHealth',
                    'ActualPlayerPositionAndVelocity', 'ActualMainSpriteClipFrame', 'ActualEntityCollisionClass'],
                unknown=['BackendPlayedAudioAndPCM', 'FullFrameBrowserPixels', 'Costumes',
                    'PrivateCallbackOrdering'],
                missingWitnessPolicy='FAIL; every mismatch is retained, no injected state or redraw.')))
        if npc_freeze:
            cases[-1]['activeItems']['npcFreeze'] = dict(
                schema='source-public-npc-freeze-v1', routeVersion=2,
                itemId=39, durationInput=220, cappedTimer=120,
                control=freeze_control,
                targets=[dict(type=12, variant=0, subtype=0), dict(type=18, variant=0, subtype=0)],
                required=['LiveUnchampionedUnlinkedTargetsPastAppear', 'Held39Charge3AndPhysicalSpace',
                    'MovingTargetBeforeUse', 'HorfShooterProgressBeforeUse', 'UnchangedIdentityCountAndHp',
                    'FullAndFast120To119Boundary', 'ImmediateApplicationCurrent470AndMotion',
                    'FirstFullAndFastBoundary', 'EveryTimerDecrementAndFastHold',
                    'FrozenAiAndSpriteHold', 'ReducedFullMotionAndExactFastPreview',
                    'Timer1To0ColorAndFlagBoundary', 'NextFullResume'],
                unknown=['FrozenDeathAndSfx137', 'BossLockout', 'LinkedPropagation',
                    'NegativeDurationAndReapplication', 'RoomOrContinueReadmission', 'FullFramePixels'])
            if freeze_control:
                cases[-1]['activeItems']['npcFreeze']['required'] = [
                    'SameActualLiveTargetsWithoutSpace', 'Held39Charge3', 'NoTEMP39OrDischarge',
                    'ZeroFreezeTimerAndFlag', 'OrdinaryBaseOwnerRetained',
                    'MovingTargetAndHorfShooterProgress']

    single = lambda item: [(item, 6, 5)]
    fixed = 'Fixed declared public input'
    for index, (item, slug) in enumerate(sorted(ACTIVES.items())):
        if item == 45:
            continue  # active-item-use owns the natural Yum Heart route.
        phase = index & 1
        declare(f'{slug}-{item}-fixed-phase{phase}', single(item), idle + acquire() + use('use', [], 24),
                12713, phase, fixed, [item])
    #Converter needs more than three soul halves: a black and a soul heart
    #beside the Player (the black half forces the Black-heart Necronomicon).
    hearts = [dict(label='black-heart-west', keys=['a'], presentations=6),
              dict(label='black-heart-released', keys=[], presentations=30),
              dict(label='soul-heart-east', keys=['d'], presentations=12),
              dict(label='soul-heart-released', keys=[], presentations=30),
              dict(label='return-centre', keys=['a'], presentations=6),
              dict(label='return-released', keys=[], presentations=30)]
    declare('converter-296-black-soul-phase0', [(296, 6, 5), (10, 6, 5, 6), (10, 3, 7, 6)],
            idle + hearts + acquire() + use('use', [], 24), 12713, 0, fixed, [296])
    #How to Jump while moving lands ahead of the Player.
    declare('how-to-jump-282-moving-phase0', single(282),
            idle + acquire() + use('use-moving', ['d'], 30), 12713, 0, fixed, [282])
    #The D6 rerolls the other live pedestal in place (its own is empty).
    declare('d6-105-reroll-phase0', [(105, 6, 5), (1, 2, 1)],
            idle + acquire() + use('use', [], 24), 12713, 0, fixed, [105])
    #A second active replaces the first and drops it back as a pedestal.
    declare('swap-45-282-phase1', [(45, 6, 5), (282, 5, 6)],
            idle + acquire('first') + acquire('second', 'a', 'west') + use('use', [], 24), 12713, 1, fixed, [282])
    # A w4 pulse settles22.343 world units from the pedestal, outside the
    # strict radius22 contact. Eight public input presentations cross it;
    # the existing released96 then admits the actual Pickup ceremony.
    # Horf(10,5) is inside its strict200 shooting range of this route.
    # Both games must still witness acquisition, shooter progress and survival.
    freeze_actions = idle + [
        dict(label='freeze-acquire-north-8', keys=['w'], presentations=8),
        dict(label='freeze-acquire-released-ceremony', keys=[], presentations=96),
    ] + use('freeze-use', [], 300)
    if args.npc_freeze:
        for phase in (0, 1):
            declare(f'moms-bra-39-npc-freeze-fixed-phase{phase}', single(39), freeze_actions,
                    12713, phase, fixed, [39], npc_freeze=True)
        control_actions = copy.deepcopy(freeze_actions)
        for action in control_actions:
            action['keys'] = [key for key in action['keys'] if key != 'space']
        declare('moms-bra-39-npc-freeze-no-space-control-phase0', single(39), control_actions,
                12713, 0, fixed, [], npc_freeze=True, freeze_control=True)
    with (output / 'os-entropy.jsonl').open('x') as journal:
        for index in range(args.count):
            raw = os.urandom(4)
            epoch = int.from_bytes(raw, 'little')
            choice = dict(rawHex=raw.hex(), bootEpochSeed=epoch, initialPhaseOffset=(epoch >> 31) & 1,
                rule='One direct unfiltered little-endian u32 public boot epoch, high bit phase; no redraw.')
            journal.write(json.dumps(choice) + '\n')
            journal.flush()
            os.fsync(journal.fileno())
            suffix = '' if index == 0 else '-' + str(index)
            declare(f'how-to-jump-282-fresh-os-epoch{suffix}', single(282),
                    idle + acquire() + use('use-moving', ['a'], 30), epoch, choice['initialPhaseOffset'],
                    choice, [282])
            if args.npc_freeze:
                declare(f'moms-bra-39-npc-freeze-fresh-os-epoch{suffix}', single(39), freeze_actions,
                        epoch, choice['initialPhaseOffset'], choice, [39], npc_freeze=True)
    result = dict(schema=1, suite='active-items', cases=cases,
        source='Declared free active pedestals in the base start room of public seeded run 12713/B91Y6ZEV, '
            'walked onto and used with public Space. Fresh OS epochs are journaled before either game runs.')
    (output / 'scenarios.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(status='declared-not-executed', cases=len(cases), output=str(output))))


if __name__ == '__main__':
    main()
