#!/usr/bin/env python3
"""Retained BLL public214 prefix and ordinary Monstro entry/AI; no state injection."""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('boss_paired', Path(__file__).with_name('foundation-paired-scenarios.py'))
paired = importlib.util.module_from_spec(spec)
spec.loader.exec_module(paired)
REFERENCES = ROOT / 'references/isaac/foundation/combat/monstro'


def key(name, down):
    return {'type': 'key', 'key': name, 'down': down}


def step(count, label=None):
    return [{'type': 'step', 'frames': count}] + ([{'type': 'capture', 'label': label}] if label else [])


def observed(count, prefix, stride=2):
    result = []
    for index in range(1, count + 1):
        result += step(1, f'{prefix}-{index}' if index % stride == 0 else None)
    return result


def prefix():
    # Passive Fly13 has no contact damage, but its normal config shuts doors.
    # The first prepared route omitted this real kill; retain that smoke/tape.
    result = step(60) + [key('w', True)] + step(74, 'fly-room-arrival') + [key('w', False)] + step(40, 'fly-room-ready')
    result += [key('Up', True)] + observed(90, 'fly-prefix-shoot', 2) + [key('Up', False)] + observed(24, 'fly-prefix-clear', 2)
    result += [key('w', True)] + step(74, 'level2-room-arrival') + [key('w', False)]
    result += observed(44, 'level2-appearance', 1)
    return result


def case(identifier, tail, required, randomized):
    route = json.loads((REFERENCES / 'public-route-plan.json').read_text())
    bank = json.loads((REFERENCES / 'os-random-candidates-before-grading.json').read_text())
    selected = next(row for row in bank['candidates'] if row['ordinal'] == route['firstSimpleCandidateOrdinal'])
    if selected['code'] != 'BLLV920M' or selected['runSeed'] != 91930655:
        raise ValueError('The retained first candidate must not be replaced after grading')
    result = paired.build('paired-room-combat', selected['code'])
    result['scenario'] = {'id': identifier, 'description': 'Natural lvl2Fly214 public prefix and Monstro20 construction/Scene/AI',
                          'systems': ['enemies-bosses', 'damage-pipeline', 'projectile-physics', 'room-connectivity'], 'randomized': randomized}
    result['monstroRequired'] = required
    result['randomSelection'] = {'recordedAt': bank['recordedAt'], 'source': bank['source'], 'selected': selected,
                                'selectionRule': bank['selectionRule'], 'allCandidatesRetained': route['allCandidatesRetained']}
    result['publicRoute'] = 'Start6 → up7/Room18(passiveFly13) → up9/Room185(two214) → right0/Monstro1012. Inputs only; no expected actor/HP/RNG state is supplied to either game.'
    result['unacceptedRequiredFollowups'] = ['Complete boss blood/gibs/final heart/reward/trapdoor callbacks and actual stage-controller exit',
                                           'Blood background render/return, inherited colors, Fly13 appearance FX and214 flag4 blast acceptance',
                                           'Source run-loader/global shared-stream alignment; no native42 offset', 'Inherited modifiers and source action14 physical skip binding']
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
        paired.runner.validate_tape(result[name])
    return result


def cohort():
    # Contrived held/released inputs isolate actual admission and full/fast feedback.
    admission = case('paired-level2-bll-admission', prefix() + observed(72, 'level2-active', 1),
                     ['level2-admission', 'level2-poof', 'level2-poof-fence', 'level2-poof-retirement', 'level2-appearance', 'level2-active'], False)
    route = prefix()
    # A public directional firing sweep attempts the real prefix clearance.
    # If either game fails to clear/reach the boss, that execution stays red.
    for direction in ['Up', 'Left', 'Right', 'Down']:
        route += [key(direction, True)] + observed(180, f'level2-shoot-{direction.lower()}', 2) + [key(direction, False)]
    route += observed(24, 'level2-clear-tail', 1)
    # From the incoming bottom doorway, ordinary W27/release30 aligns the right slot.
    route += [key('w', True)] + step(27, 'boss-row-approach') + [key('w', False)] + step(30, 'boss-row-released')
    route += [key('d', True)] + observed(120, 'boss-door', 4) + [key('d', False)]
    route += observed(220, 'boss-scene', 2)
    route += [key('Right', True)] + observed(300, 'boss-active', 4) + [key('Right', False)] + step(12, 'boss-route-final')
    random = case('paired-random-bll-monstro-route', route,
                  ['level2-admission', 'level2-poof', 'level2-poof-fence', 'level2-poof-retirement', 'level2-appearance', 'level2-active', 'level2-damage', 'level2-drain', 'level2-death', 'level2-retirement',
                   'prefix-clear-delay', 'prefix-clear-boundary', 'prefix-clear',
                   'boss-admission', 'boss-poof', 'boss-scene', 'boss-cleanup', 'boss-ai', 'boss-projectiles', 'boss-damage'], True)
    return [admission, random]


if __name__ == '__main__':
    print(json.dumps({'schema': 1, 'cases': cohort()}))
