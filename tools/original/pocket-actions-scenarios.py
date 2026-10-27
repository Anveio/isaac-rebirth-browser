"""Declare natural Card acquisition and physical Ctrl tap/hold/drop inputs.

Only public boot/resource/menu/key inputs are authored. Every first OS token is
journaled before either game starts, without filtering any observed outcome.
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


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def public_actions():
    actions = [dict(label='released-initial-card-appear', keys=[], presentations=96)]
    for number in range(1, 4):
        actions.extend([dict(label=f'card-acquire-north-{number}', keys=['w'], presentations=4),
            dict(label=f'card-acquire-released-{number}', keys=[], presentations=96)])
    actions.extend([
        dict(label='ctrl-tap', keys=['Control_L'], presentations=1),
        dict(label='ctrl-tap-release', keys=[], presentations=2),
        dict(label='ctrl-hold-120', keys=['Control_L'], presentations=120),
        dict(label='ctrl-hold-121-drop', keys=['Control_L'], presentations=1),
        dict(label='ctrl-drop-release', keys=[], presentations=2),
        #Move away before AD0 expires, leaving the actual dropped body observable.
        dict(label='leave-dropped-card', keys=['s'], presentations=12),
        dict(label='released-away-from-card', keys=[], presentations=24),
        dict(label='ctrl-empty-hold-120', keys=['Control_L'], presentations=120),
        dict(label='ctrl-empty-hold-121', keys=['Control_L'], presentations=1),
        dict(label='ctrl-empty-release', keys=[], presentations=2),
    ])
    return actions


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--count', type=int, default=1)
    args = parser.parse_args()
    if not 1 <= args.count <= 64:
        parser.error('Fresh bank requires1..64 unfiltered OS cases')
    codec = load('pocket_actions_seed', ROOT / 'tools/original/foundation-seeds.py')
    if codec.encode_seed(RUN_SEED) != SEED_CODE:
        raise ValueError('Literal public run seed/code mismatch')
    rooms_codec = load('pocket_actions_rooms', ROOT / 'tools/foundation/decode-rooms.py')
    baseline = ROOT / 'references/isaac/rebirth/resources'
    stb_path, xml_path = baseline / 'rooms/00.special rooms.stb', baseline / 'challenges.xml'
    stb_raw = stb_path.read_bytes()
    rooms = rooms_codec.decode_bytes(stb_raw, source=str(stb_path))
    if rooms_codec.encode_bytes(rooms) != stb_raw:
        raise ValueError('Base catalogue codec must preserve input bytes')
    room = rooms[80]
    if (room['type'], room['variant'], room['width'], room['height']) != (1, 2, 13, 7):
        raise ValueError('Expected source starting-room record')
    old_groups = copy.deepcopy(room['spawns'])
    groups = [dict(x=6, y=5, entries=[dict(type=5, variant=300, subtype=1, weight=1.0)])]
    room['spawns'] = copy.deepcopy(groups)
    authored = rooms_codec.encode_bytes(rooms)
    inverse = rooms_codec.decode_bytes(authored, source='declared Ctrl room')
    inverse[80]['spawns'] = old_groups
    if rooms_codec.encode_bytes(inverse) != stb_raw:
        raise ValueError('Fixture changed undeclared source rows')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    resources = output / 'resources'
    resources.mkdir()
    xml, stb = resources / 'challenges.xml', resources / 'pocket-actions-special.stb'
    xml.write_bytes(xml_path.read_bytes())
    stb.write_bytes(authored)
    cases = []

    def declare(name, epoch, phase, entropy=None):
        cases.append(dict(name=name, publicMenuRoute='seeded-normal', publicRunSeed=RUN_SEED,
            publicSeedCode=SEED_CODE, bootEpochSeed=epoch, initialPhaseOffset=phase,
            entropy=entropy, sfxVolume=1,
            fixture=dict(kind='source-authored-start-room-STB-on-public-seeded-normal-run',
                challengeId=1, startingItems=[], view='item-only',
                authoredResourcePath=str(xml), baselineResourcePath=str(xml_path),
                specialRoomsPath=str(stb), specialRoomsSha256=hashlib.sha256(authored).hexdigest(),
                attributeChanges=[], challengeConfigurationUsed=False,
                startRoomSpawnChange=dict(catalogue=0, index=80, type=1, variant=2,
                    operation='replace', originalGroupCount=len(old_groups), groups=copy.deepcopy(groups),
                    allOtherRowsMetadataDoorsWeightsOrderPreserved=True)),
            actions=public_actions(),
            pocketCollection=dict(schema='source-public-pocket-collection-v1', originalRoomGroupCount=0,
                replacesOriginalRoomGroups=True, groups=copy.deepcopy(groups),
                scope='Transport only; the separate Ctrl reader owns actual drop laws.'),
            pocketActions=dict(schema='source-public-pocket-actions11-v1', cardId=1,
                required=['FreshKeyboardDevice0', 'NaturalCardContactAndRetirement',
                    'PhysicalCtrlTap', 'Actual120Then121Callbacks', 'ActualDropAndAD0',
                    'ReleasedCounterReset', 'ActualEmptyHoldAndContinuedUpdates'],
                unavailable=['PrivateCallbackEntryAndOrdering', 'PositiveTrinketDrop',
                    'Capacity2Swap', 'GnawedLeaf210Consumer', 'BackendAudioPCM', 'FullFramePixels'],
                missingWitnessPolicy='FAIL; preserve all mismatches and missing actual owners.')))

    declare('pocket-actions11-fixed-phase0', 12713, 0)
    declare('pocket-actions11-fixed-phase1', 12713, 1)
    tokens = []
    with (output / 'os-entropy.jsonl').open('x') as journal:
        for index in range(args.count):
            raw = os.urandom(8)
            token = dict(index=index, hex=raw.hex(), bootEpochSeed=int.from_bytes(raw[:4], 'little'),
                initialPhaseOffset=raw[4] & 1,
                rule='First direct8 OS bytes; little-endian u32 boot and byte4 lowbit phase; no redraw.')
            journal.write(json.dumps(token) + '\n')
            journal.flush()
            os.fsync(journal.fileno())
            tokens.append(token)
            declare(f'pocket-actions11-random-{index}', token['bootEpochSeed'], token['initialPhaseOffset'], token)
    bank = dict(schema=1, suite='pocket-actions', cases=cases,
        entropyJournal=dict(path=str(output / 'os-entropy.jsonl'),
            sha256=hashlib.sha256((output / 'os-entropy.jsonl').read_bytes()).hexdigest()),
        scope='Public fixed run, authored natural Card1 contact, physical Ctrl and first unfiltered OS boot/phase inputs.')
    (output / 'scenarios.json').write_text(json.dumps(bank, indent=2) + '\n')
    print(json.dumps(dict(status='declared-not-executed', cases=len(cases), output=str(output))))


if __name__ == '__main__':
    main()
