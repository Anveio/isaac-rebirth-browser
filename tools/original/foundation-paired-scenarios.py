#!/usr/bin/env python3
"""Public tapes shared by a fresh original process and the browser game.

The original's introductory cutscene is retained as a source-only prelude.
Pairing starts at its independently observed first Title frame. No observed
game state is injected into the browser. paired-random-run draws its actions
from its OS-random run seed, so `--random-seeds` replays the same tape.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import random

ROOT = Path(__file__).resolve().parents[2]


def load(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


scenarios = load('paired_clock_scenarios', 'foundation-clock-scenarios.py')
runner = scenarios.runner
SCENARIOS = [
    {'id': 'paired-title-menu', 'description': 'Title fade, loop, retained reentry, seeded menu navigation and movement/fire',
     'systems': ['menu-navigation', 'menu-animation', 'locomotion', 'projectile-physics', 'input-determinism']},
    {'id': 'paired-movement-fire', 'description': 'Run entry, held movement, release, shooting and live tears',
     'systems': ['locomotion', 'projectile-physics', 'run-introduction', 'input-determinism']},
    {'id': 'paired-pause-release', 'description': 'Pause while moving, release controls during pause, animate exit and resume both phases',
     'systems': ['menu-navigation', 'menu-animation', 'locomotion', 'input-determinism']},
    {'id': 'paired-bomb-moving-even', 'description': 'Moving bomb birth, held E, coasting blast and damage on one source phase',
     'systems': ['bombs', 'health-inventory', 'locomotion', 'input-determinism']},
    {'id': 'paired-bomb-moving-odd', 'description': 'Same moving bomb interaction shifted by one public presentation',
     'systems': ['bombs', 'health-inventory', 'locomotion', 'input-determinism']},
    {'id': 'paired-room-combat', 'description': 'Enter adjacent room, shoot rocks, blast them and cross destroyed cells',
     'systems': ['room-connectivity', 'grid-entities', 'locomotion', 'health-inventory', 'bombs', 'projectile-physics', 'input-determinism']},
    {'id': 'paired-pause-before-transition', 'description': 'Walk into the upper doorway and pause just before the room transition starts',
     'systems': ['menu-navigation', 'room-connectivity', 'locomotion', 'input-determinism']},
    {'id': 'paired-pause-transition', 'description': 'Escape during the upper room transition is refused; the next Escape pauses',
     'systems': ['menu-navigation', 'room-connectivity', 'locomotion', 'input-determinism']},
    {'id': 'paired-wall-hugger-contact', 'description': 'Recorded #154 run: start-room Wall Huggers crawl into the player before the bomb blast',
     'systems': ['npc-ai', 'entity-collision', 'health-inventory', 'bombs', 'locomotion', 'input-determinism']},
    {'id': 'paired-hard-run', 'description': 'Character-menu Down selects hard difficulty before Enter; hard run HUD, movement, fire and the hard pause screen',
     'systems': ['menu-navigation', 'run-introduction', 'locomotion', 'projectile-physics', 'input-determinism']},
    {'id': 'paired-random-run', 'description': 'Fresh OS-random run seed and recorded randomized movement, firing, coasting, pause and bomb actions',
     'systems': ['locomotion', 'projectile-physics', 'bombs', 'menu-navigation', 'input-determinism'],
     'randomized': True},
]


def key(tape, name, down):
    tape.append({'type': 'key', 'key': name, 'down': down})


def advance(tape, count, label=None):
    tape.append({'type': 'step', 'frames': count})
    if label:
        tape.append({'type': 'capture', 'label': label})


# The #154 run, first drawn OS-randomly before #170 and retained verbatim:
# CQHR 8NEH's Wall Huggers touch the player before its bomb detonates.
WALL_HUGGER_SEED = 'CQHR 8NEH'
WALL_HUGGER_ACTIONS = [('s', 'Right', 18, 12), ('w', 'Left', 26, 16), ('s', 'Right', 31, 31)]


# Contrived room-transition pause cases own a seed whose start room has an
# ordinary room above the spawn point (#168).
TRANSITION_SEED = '0EWL B4FT'
TRANSITION_ESCAPE_FRAMES = {'paired-pause-before-transition': 30, 'paired-pause-transition': 40}


def select_hard(tape):
    """The seeded Character page: Down toggles the difficulty widget to hard
    before the Return that starts the run."""
    seeded = next(i for i, a in enumerate(tape) if a.get('label') == 'character-seeded')
    tape[seeded + 1:seeded + 1] = [{'type': 'key', 'key': 'Down', 'down': True}, {'type': 'step', 'frames': 12},
                                   {'type': 'key', 'key': 'Down', 'down': False}, {'type': 'step', 'frames': 30},
                                   {'type': 'capture', 'label': 'character-hard'}]


def build(identifier, seed, hard=False):
    if identifier in TRANSITION_ESCAPE_FRAMES:
        seed = TRANSITION_SEED
    if identifier == 'paired-wall-hugger-contact':
        seed = WALL_HUGGER_SEED
    if identifier == 'paired-title-menu':
        tape = scenarios.build_scenario('title-animation-fade', seed)
        boundary = 'title-first'
    else:
        tape = scenarios.build_scenario('menu-run-introduction', seed)
        boundary = 'title-ready'
        if identifier == 'paired-room-combat':
            combat = scenarios.build_scenario('combat-rock-typed', seed)
            start = next(i for i, a in enumerate(combat) if a.get('label') == 'gameplay-start')
            tail = combat[start + 1:]
        elif identifier == 'paired-movement-fire':
            tail = tape[next(i for i, a in enumerate(tape) if a.get('label') == 'gameplay-start') + 1:]
        else:
            tail = []
            advance(tail, 57, 'run-ready')
            if identifier == 'paired-pause-release':
                key(tail, 'd', True); advance(tail, 8, 'moving-before-pause')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 27, 'pause-idle'); key(tail, 'd', False)
                advance(tail, 31, 'pause-released-controls')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 1, 'pause-exit-start')
                advance(tail, 22, 'pause-exit-end')
                advance(tail, 1, 'resume-next-phase')
                advance(tail, 1, 'resume-following-phase')
                key(tail, 'Up', True); advance(tail, 24, 'resumed-shooting')
                key(tail, 'Up', False); advance(tail, 30, 'resumed-released')
            elif identifier in ('paired-bomb-moving-even', 'paired-bomb-moving-odd'):
                key(tail, 'd', True); advance(tail, 25, 'bomb-approach')
                if identifier.endswith('odd'):
                    advance(tail, 1)
                key(tail, 'e', True); advance(tail, 1, 'bomb-birth')
                advance(tail, 7, 'bomb-held-input')
                key(tail, 'd', False); key(tail, 'e', False)
                advance(tail, 111, 'bomb-before-blast')
                for label in ('bomb-phase-120', 'bomb-phase-121', 'bomb-phase-122', 'bomb-phase-123'):
                    advance(tail, 1, label)
                advance(tail, 10, 'bomb-after-blast')
            elif identifier in TRANSITION_ESCAPE_FRAMES:
                key(tail, 'w', True); key(tail, 'Down', True); advance(tail, 38, 'transition-approach')
                key(tail, 'w', False); key(tail, 'Down', False); advance(tail, 36, 'transition-coasting')
                key(tail, 'w', True); advance(tail, TRANSITION_ESCAPE_FRAMES[identifier])
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False); key(tail, 'w', False)
                advance(tail, 40, 'transition-escape-1')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 30, 'transition-escape-2')
            elif identifier == 'paired-wall-hugger-contact':
                for index, (movement, shooting, held, coasting) in enumerate(WALL_HUGGER_ACTIONS):
                    key(tail, movement, True); key(tail, shooting, True)
                    advance(tail, held, f'random-held-{index}')
                    key(tail, movement, False); key(tail, shooting, False)
                    advance(tail, coasting, f'random-coasting-{index}')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 43, 'random-paused')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 27, 'random-resumed')
                key(tail, 'e', True); advance(tail, 1); key(tail, 'e', False)
                advance(tail, 133, 'random-after-bomb')
            elif identifier == 'paired-hard-run':
                key(tail, 'd', True); key(tail, 'Right', True); advance(tail, 30, 'hard-moving-fire')
                key(tail, 'd', False); key(tail, 'Right', False); advance(tail, 30, 'hard-coasting')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 40, 'hard-paused')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 30, 'hard-resumed')
            elif identifier == 'paired-random-run':
                draw = random.Random(f'paired-random-run:{seed}')
                for index in range(3):
                    movement = draw.choice(['w', 'a', 's', 'd'])
                    shooting = draw.choice(['Up', 'Left', 'Down', 'Right'])
                    key(tail, movement, True); key(tail, shooting, True)
                    advance(tail, 12 + draw.randrange(37), f'random-held-{index}')
                    key(tail, movement, False); key(tail, shooting, False)
                    advance(tail, 8 + draw.randrange(29), f'random-coasting-{index}')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 28 + draw.randrange(21), 'random-paused')
                key(tail, 'Escape', True); advance(tail, 1); key(tail, 'Escape', False)
                advance(tail, 27, 'random-resumed')
                key(tail, 'e', True); advance(tail, 1); key(tail, 'e', False)
                advance(tail, 133, 'random-after-bomb')
            else:
                raise ValueError('Unknown paired scenario: ' + identifier)
        if hard or identifier == 'paired-hard-run':
            select_hard(tape)
        start = next(i for i, a in enumerate(tape) if a.get('label') == 'gameplay-start')
        tape = tape[:start + 1] + tail
    # X11 input is consumed on the next released phase. Never compare a DOM
    # key mutation with a native image taken before that phase was released.
    stable = []
    for action in tape:
        if action['type'] == 'capture' and stable and stable[-1]['type'] == 'key':
            advance(stable, 1)
        stable.append(action)
    runner.validate_tape(stable)
    definition = next(s for s in SCENARIOS if s['id'] == identifier)
    begin = next(i for i, a in enumerate(stable) if a.get('label') == boundary)
    return {'schema': 1, 'scenario': definition, 'seed': seed, 'seedValue': runner.codec.decode_seed(seed),
            'bootEpochSeed': 1700000000, 'initialSeed': 2184569263,
            'bootstrap': {'boundaryLabel': boundary, 'sourceOnlyPrelude': stable[:begin],
                          'sourceContract': 'First Title frame0/f32 white fade .975; cutscene is not implemented by the browser'},
            'tape': stable, 'pairedTape': stable[begin:],
            'comparisons': {'fullFrameRGBA': 'strict diagnostic at every paired capture; no tolerance',
                            'state': 'source title/fader clocks, player motion, pause/controller, red HP/damage countdown and active tear/bomb counts',
                            'completeOriginalStateParity': False}}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--list', action='store_true')
    parser.add_argument('--scenario', choices=[s['id'] for s in SCENARIOS], default='paired-movement-fire')
    parser.add_argument('--seed')
    parser.add_argument('--output', type=Path)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.list:
        rows = []
        for definition in SCENARIOS:
            case = build(definition['id'], 'T1PK6Y0W')
            rows.append({**definition, 'source': str(Path(__file__).resolve().relative_to(ROOT)),
                         'captureLabels': [a['label'] for a in case['pairedTape'] if a['type'] == 'capture'],
                         'boundaryLabel': case['bootstrap']['boundaryLabel'],
                         'presentationModel': 'recorded OS-random intervals' if definition.get('randomized') else str(sum(a['frames'] for a in case['pairedTape'] if a['type'] == 'step'))})
        print(json.dumps(rows)); return 0
    if args.validate_original:
        verifier = load('paired_native_provenance', 'foundation-clock-verify.py')
        evidence = load('paired_native_evidence', 'native-evidence.py')
        failures = verifier.validate_run(evidence.read_manifest(args.validate_original))
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures}))
        return 1 if failures else 0
    seed = args.seed or runner.codec.random_vector()['code']
    case = build(args.scenario, seed)
    if args.output:
        args.output.mkdir(parents=True, exist_ok=True)
        (args.output / 'case.json').write_text(json.dumps(case, indent=2) + '\n')
        (args.output / 'tape.json').write_text(json.dumps(case['tape'], indent=2) + '\n')
    print(json.dumps(case)); return 0


if __name__ == '__main__':
    raise SystemExit(main())
