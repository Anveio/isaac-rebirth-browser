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



def boss_clear_case(phase):
    # This source-selected contrived fixture is independent of the retained BLL
    # bank. The fixed public plan was declared before any native/browser grade.
    plan = json.loads((REFERENCES / 'public-clear-route-plan.json').read_text())
    if (plan['seedValue'], plan['seed'], plan['randomized'], plan['route'], plan['doorSlots'], plan['inputPhases']) != (978, 'B9112M6H', False, [6, 8, 9, 11, 0], [2, 3, 3, 0], [0, 1]):
        raise ValueError('The first source-selected contrived boss route must remain fixed')
    result = paired.build('paired-room-combat', plan['seed'])
    result['scenario'] = {'id': f'paired-contrived-978-boss-clear-phase{phase}',
        'description': 'Fixed public 3Worm244/214/3AttackFly18 prefix and actual Monstro death/final Heart callback',
        'systems': ['enemies-bosses', 'damage-pipeline', 'projectile-physics', 'room-connectivity', 'health-inventory', 'pickup-physics'], 'randomized': False}
    result['monstroRequired'] = ['contrived-prefix-8-admission', 'contrived-prefix-8-retirement', 'contrived-prefix-8-clear',
        'contrived-prefix-9-admission', 'contrived-prefix-9-retirement', 'contrived-prefix-9-clear',
        'contrived-prefix-11-admission', 'contrived-prefix-11-retirement', 'contrived-prefix-11-clear',
        'boss-admission', 'boss-poof', 'boss-scene', 'boss-cleanup', 'boss-ai', 'boss-projectiles', 'boss-damage',
        'boss-count-admitted', 'boss-death', 'boss-death-animation', 'boss-retirement', 'boss-count-zero', 'boss-room-clear',
        'boss-final-heart']
    result['bossClear'] = {'plan': 'references/isaac/foundation/combat/monstro/public-clear-route-plan.json',
        'selection': plan['sources'], 'phaseOffset': phase, 'bossSeed': 2321890733,
        'prefixActors': {'8': [[244, 12804082], [244, 1321941351], [244, 3236979734]],
                         '9': [[214, 2947905018]],
                         '11': [[18, 2486173107], [18, 999304435], [18, 635342908]]},
        'firingBudgetFixedBeforeGrading': True, 'randomized': False}
    result['publicRoute'] = 'Start6 → right8/Room238(three244) → down9/Room212(one214) → down11/Room241(three18) → left0/Monstro1010. Public inputs only; source-selected contrived, not OS-random.'
    result['unacceptedRequiredFollowups'] = ['Actual boss blood/gibs/effects2/42/77 factories and immediate-update ordering',
        'Actual RoomClear reward RNG, pedestal, Grid17 admission and Stage2 request/Nightmare/new-floor chain',
        'Source run-loader/global shared-stream alignment; all current trajectory mismatches remain required',
        'Final-heart Trinket22 modifier and colored/special-boss branches outside this fresh ordinary scope']
    result['bossClear']['sampling'] = plan['compactCaptureSchedule']
    def clear_observed(count, prefix, _stride):
        # Capture-only resource correction. Every original step1/key action
        # stays unchanged; no live observations choose this fixed schedule.
        stride = plan['prefixSweep']['captureStride'] if '-sweep' in prefix and prefix.startswith('contrived-room') else plan['bossSweep']['captureStride']
        rows = []
        for index in range(1, count + 1):
            label = f'{prefix}-{index}' if index == count or index % stride in (0, 1) else None
            rows += step(1, label)
        return rows
    tail = step(60 + phase, 'contrived-ready')
    movements = iter(plan['fixedMovement'])
    def move():
        row = next(movements)
        return [key(row['key'], True)] + clear_observed(row['frames'], 'contrived-' + row['stage'].replace(' ', '-'), 16) + [key(row['key'], False)] + step(1)
    tail += move()
    for room in (8, 9, 11):
        tail += clear_observed(plan['prefixSweep']['settleFrames'], f'contrived-room{room}-settle', 16)
        for sweep in range(plan['prefixSweep']['rounds']):
            for direction in plan['prefixSweep']['directions']:
                tail += [key(direction, True)] + clear_observed(plan['prefixSweep']['heldFramesEach'], f'contrived-room{room}-sweep{sweep}-{direction.lower()}', plan['prefixSweep']['captureStride']) + [key(direction, False)]
        tail += clear_observed(plan['prefixSweep']['clearTailFrames'], f'contrived-room{room}-clear-tail', 16)
        if room == 8:
            tail += move()
        if room == 11:
            tail += move()
        tail += move()
    tail += clear_observed(plan['bossSweep']['settleFrames'], 'contrived-boss-scene', 1)
    for index, row in enumerate(plan['bossSweep']['directions']):
        tail += [key(row['key'], True)] + clear_observed(row['frames'], f'contrived-boss-sweep{index}-{row["key"].lower()}', plan['bossSweep']['captureStride']) + [key(row['key'], False)]
    tail += clear_observed(plan['bossSweep']['clearTailFrames'], 'contrived-boss-final-callback', 1)
    tail += [{'type': 'capture', 'label': 'contrived-boss-route-final'}]
    for name in ('tape', 'pairedTape'):
        end = next(index for index, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
        paired.runner.validate_tape(result[name])
        if sum(a['type'] == 'capture' for a in result[name]) > plan['compactCaptureSchedule']['maxCapturesPerCase']:
            raise ValueError('Compact boss route exceeds its preregistered capture budget')
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
    return [admission, random, boss_clear_case(0), boss_clear_case(1)]


if __name__ == '__main__':
    print(json.dumps({'schema': 1, 'cases': cohort()}))
