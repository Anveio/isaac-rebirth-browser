"""Declare the whole public Challenge41 bank before either actual game.

The original-derived XML stays in the local evidence directory. Only its
transformation recipe belongs in source; no Native observation becomes input.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[2]


def write_json(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())


def declaration(name, phase, epoch, xml, changes, token_index=None):
    case = {
        'name': name,
        'fixture': {
            'kind': 'source-authored-modified-challenge', 'challengeId': 1,
            'startingItems': [], 'startingTrinkets': [41], 'view': 'item-only',
            'authoredResourcePath': str(xml), 'attributeChanges': changes,
        },
        'bootEpochSeed': epoch, 'initialPhaseOffset': phase,
        'entropy': 'fixed declared public input' if token_index is None
            else 'first unfiltered OS token committed before projection',
        'actions': [
            {'label': 'grant-ready', 'keys': [], 'presentations': 8},
            {'label': 'default-shot-after41', 'keys': ['Right'], 'presentations': 36},
            {'label': 'release-and-retain41', 'keys': [], 'presentations': 24},
        ],
        'requires': {
            'actualPublicChallengeEntry': True,
            'bothGamesConsumeIdenticalDeclaredResourceBytes': True,
            'ownedCounts': [], 'orderedRealPedestalGrants': [],
            'compositionSourceFields': ['playerMovementD9c'],
            'sourceExpectedActorContract': {
                'primaryC08': 1, 'family': 2, 'variant': 0, 'subtype': 0,
                'controlledBy329': False, 'registeredC18DuringActualLifetime': False,
            },
            'trinketKind2Startup': {
                'schema': 'source-public-challenge-start41-v1',
                'held': [41, 0], 'collectible41Count': 0, 'poolRemoveId': 41,
                'nativeEvents': ['give-entry', 'give-return', 'remove-entry', 'remove-return'],
                'scope': 'Direct startup Give/Remove and default shots; queue/Get unqualified',
            },
        },
    }
    if token_index is not None:
        case['entropyTokenIndex'] = token_index
    return case


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--count', type=int, default=1)
    args = parser.parse_args()
    if args.count != 1:
        parser.error('This bounded bank declares two fixed phases and exactly the first OS case')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    token = secrets.token_bytes(8)
    # Commit every byte before deriving a case or reading resource outcomes.
    write_json(output / 'entropy.json', {
        'schema': 1, 'producer': 'OS secrets.token_bytes(8)',
        'selection': 'first and only token; no outcome filtering or redraw',
        'tokens': [{'index': 0, 'hex': token.hex()}],
        'projection': 'uint32le bytes0..3 bootEpoch; byte4 low bit phase; all8 bytes retained',
    })
    baseline = ROOT / 'references/isaac/rebirth/resources/challenges.xml'
    raw = baseline.read_bytes()
    tree = ET.fromstring(raw)
    rows = [node for node in tree if node.tag == 'challenge' and node.get('id') == '1']
    if tree.tag != 'challenges' or len(rows) != 1:
        raise ValueError('One actual Base Challenge1 row required')
    before = {node.get('id'): dict(node.attrib) for node in tree if node.tag == 'challenge'}
    changes = []
    for name, value in [('startingitems', ''), ('startingtrinkets', '41'),
                        ('getcurse', '0'), ('cursefilter', '127'), ('roomfilter', None)]:
        changes.append({'attribute': name, 'baselineValue': rows[0].get(name), 'authoredValue': value})
        if value is None:
            rows[0].attrib.pop(name, None)
        else:
            rows[0].set(name, value)
    if any(dict(node.attrib) != before[node.get('id')] for node in tree
           if node.tag == 'challenge' and node.get('id') != '1'):
        raise ValueError('Unrelated Challenge attributes changed')
    resources = output / 'resources'
    resources.mkdir()
    xml = resources / 'challenges.xml'
    ET.ElementTree(tree).write(xml, encoding='utf-8', xml_declaration=True)
    cases = [declaration(f'challenge-start41-fixed-phase{phase}', phase, 1700000000,
                         xml, changes) for phase in (0, 1)]
    cases.append(declaration('challenge-start41-random-0', token[4] & 1,
                             int.from_bytes(token[:4], 'little'), xml, changes, 0))
    write_json(output / 'resource-recipe.json', {
        'schema': 1, 'resource': 'challenges.xml', 'challengeId': 1, 'changes': changes,
        'baselineSha256': hashlib.sha256(raw).hexdigest(),
        'declaredSha256': hashlib.sha256(xml.read_bytes()).hexdigest(),
        'allOtherRowsAndAttributesPreserved': True,
        'privacy': 'Original-derived XML remains local; publish transformation recipe only',
    })
    entropy_path = output / 'entropy.json'
    write_json(output / 'scenarios.json', {
        'schema': 1, 'suite': 'trinket-start', 'cases': cases,
        'entropyJournal': {'path': str(entropy_path),
                           'sha256': hashlib.sha256(entropy_path.read_bytes()).hexdigest()},
        'selection': 'two fixed phases plus first unfiltered OS projection; no outcome selection',
        'outcomePolicy': 'Missing invocation/field/shot witnesses and all mismatches remain failures',
        'scope': 'Declared Challenge1 direct Give41/Remove41; typed queue and randomGet unqualified',
    })


if __name__ == '__main__':
    main()
