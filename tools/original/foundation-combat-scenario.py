#!/usr/bin/env python3
"""Replay public combat/grid holdouts in their independently observed rooms.

The route is a public input tape paired with its original seed, not an assumed
floor topology. A seed override is discovery and may enter a different room.
"""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import time

ROOT = Path(__file__).resolve().parents[2]
TAPES = ROOT / 'references/isaac/foundation/combat/tapes.json'
spec = importlib.util.spec_from_file_location('combat_clock_runner',
    Path(__file__).with_name('foundation-clock-run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
SCENARIOS = [
    {'id': 'combat-rock-typed', 'case': 'rock-typed',
     'description': 'Traverse right, test tears against rocks, plant a bomb and cross destroyed grid cells',
     'systems': ['grid-entities', 'bombs', 'projectile-physics', 'locomotion']},
    {'id': 'combat-rock-dense', 'case': 'rock-dense',
     'description': 'Independent horizontal contact/coast and dense bomb destruction checkpoints',
     'systems': ['grid-entities', 'bombs', 'locomotion']},
    {'id': 'combat-rock-held', 'case': 'rock-held',
     'description': 'Every held movement presentation into a source rock collider',
     'systems': ['grid-entities', 'locomotion']},
    {'id': 'combat-fly-aligned', 'case': 'fly-aligned',
     'description': 'Observe an aligned tear collision and deferred enemy HP drain',
     'systems': ['entity-collision', 'damage-pipeline', 'projectile-physics']},
    {'id': 'combat-fly-bomb-phase0', 'case': 'fly-bomb-phase0',
     'description': 'Bomb born in intermediate phase; first blast after 120 presentations and delayed enemy HP drain',
     'systems': ['bombs', 'damage-pipeline', 'entity-collision']},
    {'id': 'combat-fly-bomb-phase1', 'case': 'fly-bomb-phase1',
     'description': 'Bomb born in full phase; first blast after 121 presentations and delayed enemy HP drain',
     'systems': ['bombs', 'damage-pipeline', 'entity-collision']},
    {'id': 'combat-attackfly-entry', 'case': 'attackfly-entry',
     'description': 'Dense original enemy contact, mass push and actual player damage countdown',
     'systems': ['entity-collision', 'health-inventory', 'locomotion']},
    {'id': 'combat-gaper-entry', 'case': 'gaper-entry',
     'description': 'Source appearance/chase and obstacle routing discovery; NPC AI remains unimplemented',
     'systems': ['npc-ai', 'grid-entities', 'entity-collision']},
]


def cases():
    document = json.loads(TAPES.read_text())
    if document.get('schema') != 1 or not document.get('cases'):
        raise ValueError('Missing combat source tapes')
    result = {}
    for case in document['cases']:
        if case['name'] in result:
            raise ValueError('Duplicate combat source tape')
        runner.validate_tape(case['tape'])
        if runner.codec.decode_seed(case['seed']) != case['seedValue']:
            raise ValueError('Combat source tape seed mismatch')
        result[case['name']] = case
    return result


def build_tape(seed, identifier):
    catalog = cases()
    case_name = next((definition['case'] for definition in SCENARIOS
        if definition['id'] == identifier), identifier)
    template = catalog[case_name]
    runner.codec.decode_seed(seed)
    if seed == template['seed']:
        return copy.deepcopy(template['tape'])
    # Matrix discovery can supply any valid seed. Only the known seed establishes
    # the captured neighbor; replacing menu entry does not assert floor parity.
    prefix = runner.default_tape(seed)
    prefix_end = next(index for index, action in enumerate(prefix)
        if action.get('label') == 'gameplay-start')
    template_end = next(index for index, action in enumerate(template['tape'])
        if action.get('label') == 'gameplay-start')
    tape = [action for action in prefix[:prefix_end + 1]
        if action['type'] != 'capture' or action['label'] == 'gameplay-start']
    tape.extend(copy.deepcopy(template['tape'][template_end + 1:]))
    runner.validate_tape(tape)
    return tape


def main():
    catalog = cases()
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--case', choices=list(catalog), required=True)
    parser.add_argument('--output', type=Path,
        default=ROOT / '.cache/foundation' / f'combat-{time.time_ns()}')
    args = parser.parse_args()
    case = catalog[args.case]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / 'tape.json').write_text(json.dumps(case['tape'], indent=2) + '\n')
    (output / 'case.json').write_text(json.dumps({'schema': 1, 'scenario': case['name'],
        'seed': case['seed'], 'seedValue': case['seedValue'],
        'systems': ['grid-entities', 'entity-collision', 'npc-ai', 'damage-pipeline'],
        'captureLabels': [action['label'] for action in case['tape'] if action['type'] == 'capture'],
        'scope': 'source discovery/conditional kernel holdout; browser parity not established'}, indent=2) + '\n')
    print(json.dumps({'case': case['name'], 'seed': case['seed'], 'command': [
        'bash', str(ROOT / 'tools/original/with-display.sh'), 'python3',
        str(ROOT / 'tools/original/foundation-clock-run.py'), '--seed', case['seed'],
        '--tape', str(output / 'tape.json'), '--output', str(output / 'run'),
        '--state-observer', str(ROOT / 'tools/original/foundation-combat-observe.py')]}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
