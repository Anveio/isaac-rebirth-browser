#!/usr/bin/env python3
"""Declare controlled-ring contact games from the retained ordinary contact bank."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

CONTRACT = 'controlled-brim-ring-contact-v1'
SOURCE_BANK = '043fa228676b91285579049c8783b3d7d8cd50e8ffd0e1d0d3af4d2d5ac0cd03'
ENTROPY = '445754e51519b50f8e5d29d63ce4a699ac3ff63ab39fa34f683c4a5dc196738b'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def project_case(prior):
    case = copy.deepcopy(prior)
    case['name'] = prior['name'].replace('brim-contact-', 'controlled-brim-ring-contact-', 1)
    case['fixture']['startingItems'] = [118, 329]
    requirement = case['requires']
    ordinary = requirement.pop('ordinaryBrimContact')
    receiver = ordinary['authoredReceiver']['type']
    if receiver not in (14, 18):
        raise ValueError('Only the retained actual NPC14/18 declarations are supported')
    # A fixed quarter-turn points the first sweep toward each authored receiver
    # region. This is declaration-time geometry, never observed-position repair.
    rotation = ({'Right': 'Up', 'Up': 'Left', 'Left': 'Down', 'Down': 'Right'}
                if receiver == 14 else
                {'Down': 'Left', 'Left': 'Up', 'Up': 'Right', 'Right': 'Down'})
    for index, action in enumerate(case['actions']):
        if action['keys']:
            if len(action['keys']) != 1 or action['keys'][0] not in rotation:
                raise ValueError('Expected the retained single-cardinal public sweep')
            action['keys'] = [rotation[action['keys'][0]]]
            action['label'] = f'controlled-steer-{index}-{action["keys"][0].lower()}'
        else:
            action['label'] = 'receiver-ready' if index == 0 else f'controlled-release-{index}'
    requirement['ownedCounts'] = [{'itemId': item, 'count': 1} for item in (118, 329)]
    requirement['sourceExpectedActorContract'].update(
        subtype=1, controlledBy329=True,
        requiredWhen='actual admitted519FC0 controlled callback, including released input')
    requirement['compositionSourceFields'] = [
        'playerMovementD9c', 'knifeChildChain4a0', 'npc14And18DamageQueue']
    requirement['controlledBrimContact'] = {
        'schema': CONTRACT,
        'authoredReceiver': ordinary['authoredReceiver'],
        'receiverVariantPolicy': ordinary['receiverVariantPolicy'],
        'spawnGroup': ordinary['spawnGroup'],
        'sourceEntityType': 1, 'sourceVariant': 0, 'damageFlags': 8,
        'minimumAcceptedEnqueuesPerGame': 1,
        'minimumNextFullDamageApplicationsPerGame': 1,
        'publicAimPolicy': 'Stationary four-cardinal controlled steering/release; fixed quarter-turn of314 schedule toward authored receiver region. No outcome-based targeting or repair.',
        'geometryScope': 'Retained authored14 cell9/3 (or mirrored3/3),18 cell6/5; actual controlled ring radius80 and own physical center. Declared geometry does not prove actual overlap/contact.',
        'requiredActualWitnesses': [
            'Actual own C18 type7/variant1/subtype1, explicit ParentPlayer, independent identity/seed/manager and both Full/Fast physical bodies.',
            'Actual typed14/18 admission/eligibility and collision against the actual controlled ring Line segments; no Entityradius80 alias.',
            'Real empty receiver FIFO then an accepted append: exact ordered payload, sourceType1/sourceVariant0, flags8 and actual amount.',
            'Exact Laser attribution requires a bounded unique-source window with the one actual eligible C18 Laser and no competing source callbacks. ParentPlayer and matching amount alone cannot identify a Laser.',
            'The same actual receiver next Full: real ordered drain and HP change; enqueue alone is insufficient and nonlethal damage qualifies.',
            'Missing actual callback delivery, unknown source windows, payload/identity/property owners and native Remove/Uninit events remain explicit failures.',
        ],
        'countdownScope': 'Source476A60 countdown30 is a callback argument, not a stored FIFO field or observed event.',
        'uniqueSourceWindow': {
            'required': True, 'minimumEligibleControlledLasers': 1,
            'maximumEligibleControlledLasers': 1, 'maximumCompetingDamageSourceCallbacks': 0,
            'missingAttributionIsFailure': True,
        },
        'ordinaryLifetimeScope': 'No ordinaryTTL15, fade, Impact or syntheticbit33 obligation belongs to controlled7/1/1.',
        'missingToolingMustFail': True,
    }
    requirement['sourceInferredGrantPath']['orderedItems'] = [118, 329]
    case['execution']['blockingPrerequisites'] = [
        'Immutable owned producer implementing controlled7/1/1 and typed14/18 receiver paths',
        'Distinct controlled-brim-ring-contact-v1 consumer with actual FIFO, unique-source attribution and next-Full HP evidence',
        'Integration-owner serial game/storage lease',
    ]
    return case


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-bank', type=Path, required=True)
    parser.add_argument('--entropy-journal', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    bank_bytes, entropy_bytes = args.source_bank.read_bytes(), args.entropy_journal.read_bytes()
    if sha(bank_bytes) != SOURCE_BANK or sha(entropy_bytes) != ENTROPY:
        raise ValueError('This version requires the exact retained314 bank and all existing entropy')
    original = json.loads(bank_bytes)
    if len(original['cases']) != 4:
        raise ValueError('Preserve both fixed phases and both unfiltered recorded cases')
    resource_bytes = sum(Path(case['fixture'][key]).stat().st_size for case in original['cases']
                         for key in ('authoredResourcePath', 'specialRoomsPath'))
    if resource_bytes + len(bank_bytes) + 65536 > 32 * 1024 * 1024:
        raise ValueError('Explicit four-case resources exceed the source budget')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    cases, provenance = [], []
    for prior in original['cases']:
        case = project_case(prior)
        fixture, old = case['fixture'], prior['fixture']
        xml_bytes = Path(old['authoredResourcePath']).read_bytes()
        root = ET.fromstring(xml_bytes)
        nodes = [n for n in root if n.tag == 'challenge' and n.get('id') == '1']
        if len(nodes) != 1 or nodes[0].get('startingitems') != '118':
            raise ValueError('Expected the exact ordinary118-only resource owner')
        nodes[0].set('startingitems', '118,329')
        xml = output / case['name'] / 'resources/challenges.xml'
        stb = output / case['name'] / 'resources/rooms/00.special rooms.stb'
        xml.parent.mkdir(parents=True)
        stb.parent.mkdir(parents=True)
        ET.ElementTree(root).write(xml, encoding='utf-8', xml_declaration=True)
        stb_bytes = Path(old['specialRoomsPath']).read_bytes()
        if sha(stb_bytes) != old['specialRoomsSha256']:
            raise ValueError('Retained authored receiver resource changed')
        with stb.open('xb') as stream:
            stream.write(stb_bytes)
        fixture.update(authoredResourcePath=str(xml), specialRoomsPath=str(stb))
        fixture['attributeChanges'].append(dict(attribute='startingitems', baselineValue=None, sourceParentValue='118', authoredValue='118,329'))
        fixture['declaredResetJsonBytes'] = len(json.dumps(dict(seed=1, bootEpochSeed=case['bootEpochSeed'], trace=True,
            resources=dict(challengesXml=xml.read_text(), specialRooms=list(stb_bytes))), separators=(',', ':')).encode())
        if fixture['declaredResetJsonBytes'] > 512 * 1024:
            raise ValueError('Authored public reset exceeds its transport')
        write_json(output / case['name'] / 'case.json', case)
        cases.append(case)
        provenance.append(dict(sourceCase=prior['name'], case=case['name'],
            phaseAndEpochExact=True, everyActionDurationExact=True, receiverStbByteExact=True,
            xmlSha256=sha(xml.read_bytes()), stbSha256=sha(stb_bytes)))
    with (output / 'entropy.jsonl').open('xb') as stream:
        stream.write(entropy_bytes)
    bank = dict(schema=1, cases=cases, publicRouteRevision=CONTRACT,
        scope='Source-only controlled-ring14/18 contact declaration; no gameplay/transport/parity acceptance.',
        selection='Both fixed phases and every existing314 OS token, no new draws/rerolls/filters.',
        sourceParent=dict(path=str(args.source_bank.resolve()), sha256=SOURCE_BANK),
        entropyJournal=dict(path=str(args.entropy_journal.resolve()), sha256=ENTROPY),
        caseCorrespondence=provenance,
        outcomePolicy='Every missing contact, callback, unique attribution, lifecycle, field and mismatch remains failure.')
    write_json(output / 'scenarios.json', bank)
    print(json.dumps(dict(status='declared-not-executed', cases=4, scenarios=str(output / 'scenarios.json'),
                          sha256=sha((output / 'scenarios.json').read_bytes()), rawCopies=0)))


if __name__ == '__main__':
    main()
