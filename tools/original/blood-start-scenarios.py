#!/usr/bin/env python3
"""Fixed public boss routes; neither capture nor grade changes the input plan."""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


monstro = load('blood_start_monstro', Path(__file__).with_name('monstro-scenarios.py'))
boss = load('blood_start_boss', Path(__file__).with_name('boss-scenarios.py'))
paired = monstro.paired
# Validated with `boss-scenarios.py --type 20 --contrived EJ32YEEL --count 0`.
CONTRIVED = 'EJ32YEEL'


def observed(count, label, stride=16):
    result = []
    for index in range(1, count + 1):
        capture = index == 1 or index % stride in (0, 1) or index == count
        result += monstro.step(1, f'{label}-{index}' if capture else None)
    return result


def hold(key, count, label, stride=16):
    return [monstro.key(key, True)] + observed(count, label, stride) + [monstro.key(key, False)]


def clear_prefix(label):
    result = observed(80, label + '-settle')
    for round_ in range(2):
        for direction in ['Up', 'Left', 'Right', 'Down']:
            result += hold(direction, 300, f'{label}-sweep-{round_}-{direction.lower()}', 64)
    return result + observed(32, label + '-tail')


def boss_sweep():
    result = observed(240, 'blood-boss-scene')
    for index, direction in enumerate(['Left', 'Up', 'Right', 'Down', 'Left']):
        result += hold(direction, 3000 if index == 0 else 300, f'blood-boss-shoot-{index}')
    return result + observed(240, 'blood-boss-tail')


def contrived():
    # A port-recorded route that a fresh original was validated to follow into
    # Monstro's room (boss-scenarios.py); the boss sweep starts on arrival.
    route, reason = boss.plan(boss.floor(paired.runner.codec.decode_seed(CONTRIVED)), (20, 0))
    if route is None:
        raise ValueError(f'{CONTRIVED} no longer plans a quiet route to Monstro: {reason}')
    result, recorded = boss.arrival(CONTRIVED, route)
    if result is None:
        raise ValueError(f'{CONTRIVED} route was not recorded: {recorded}')
    result['bloodStartSelection'] = {
        'kind': 'contrived boss-route fixture, original-validated by boss-scenarios.py', 'code': CONTRIVED,
        'route': route['rooms'], 'bossSeed': route['selector']['spawnSeed'],
    }
    for name in ('tape', 'pairedTape'):
        result[name] += boss_sweep()
    return result, route


def cohort():
    # Preserve the first recorded OS BLL route and its failed prefix. The
    # declared longer firing tail is fixed here before this gate runs.
    random = monstro.cohort()[1]
    for name in ('tape', 'pairedTape'):
        random[name] += boss_sweep()
    cases = []
    fixture, route = contrived()
    for base, kind in [(fixture, 'contrived-route'), (random, 'recorded-os-bll')]:
        for offset in (0, 1):
            case = json.loads(json.dumps(base))
            case['scenario'] = {
                'id': f'paired-blood-start-{kind}-phase-{offset}',
                'description': 'Actual public Monstro damage/Death and source17/SFX72/42 callbacks',
                'randomized': kind == 'recorded-os-bll',
                'systems': ['enemies-bosses', 'damage-pipeline', 'sprite-effects', 'audio'],
            }
            case['bloodStartInputPhaseOffset'] = offset
            routed = kind == 'contrived-route'
            case['bloodStartBossIdentity'] = {
                'descriptorIndex': route['rooms'][-1] if routed else 0,
                'gridIndex': route['selector']['gridIndex'] if routed else 59,
                'type': 5, 'variant': route['template']['variant'] if routed else 1012, 'name': 'Monstro',
                'spawnSeed': route['selector']['spawnSeed'] if routed else 1050413383,
                'scope': 'Independent source-selected fixture identity; gate-only expectation, never a runtime input.',
            }
            case['bloodStartRequired'] = ['bossAdmission', 'bossDamage', 'bossDeath', 'poof04', 'bloodGush',
                                          'sound72', 'matchedPoof04', 'matchedBloodGush', 'full', 'fast', 'gushCleanup']
            for name in ('tape', 'pairedTape'):
                if offset:
                    end = next(i for i, action in enumerate(case[name]) if action.get('label') == 'gameplay-start')
                    case[name][end + 1:end + 1] = monstro.step(offset)
                paired.runner.validate_tape(case[name])
            case['unacceptedRequiredFollowups'] = ['Explosion2/77/category5/final7/final hearts/Grid17',
                'Complete native render submission/background return/RGBA', 'Unobserved exact birth fence remains pending']
            cases.append(case)
    return cases


if __name__ == '__main__':
    print(json.dumps({'schema': 1, 'scope': 'Fixed complete four-case public bank; all outcomes are required and retained', 'cases': cohort()}))
