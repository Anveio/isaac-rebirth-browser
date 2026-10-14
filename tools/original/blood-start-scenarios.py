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
paired = monstro.paired


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
    result = paired.build('paired-room-combat', 'B9112M6H')
    result['bloodStartSelection'] = {
        'kind': 'contrived deterministic identity-only source fixture', 'seedValue': 978,
        'ruleSha256': '357ef7ca20c7a2974c6060af3ceda3c42a730925f2cc8c969d32e814c0b18f11',
        'selectionSha256': '3788935b0c231b2433946ee83f4fd48764c195cea6e305d42e287e934343090a',
        'route': [6, 8, 9, 11, 0], 'bossSeed': 2321890733,
        'scope': 'First deterministic978 selected before actual game grading; never OS-random or a reachability guarantee.',
    }
    tail = hold('d', 120, 'blood-start-to8') + clear_prefix('blood-room8')
    tail += hold('d', 74, 'blood-room8-align') + hold('s', 120, 'blood-to9') + clear_prefix('blood-room9')
    tail += hold('s', 120, 'blood-to11') + clear_prefix('blood-room11')
    tail += hold('s', 54, 'blood-room11-align') + hold('a', 120, 'blood-to0') + boss_sweep()
    for name in ('tape', 'pairedTape'):
        end = next(i for i, action in enumerate(result[name]) if action.get('label') == 'gameplay-start')
        result[name] = result[name][:end + 1] + tail
    return result


def cohort():
    # Preserve the first recorded OS BLL route and its failed prefix. The
    # declared longer firing tail is fixed here before this gate runs.
    random = monstro.cohort()[1]
    for name in ('tape', 'pairedTape'):
        random[name] += boss_sweep()
    cases = []
    for base, kind in [(contrived(), 'contrived978'), (random, 'recorded-os-bll')]:
        for offset in (0, 1):
            case = json.loads(json.dumps(base))
            case['scenario'] = {
                'id': f'paired-blood-start-{kind}-phase-{offset}',
                'description': 'Actual public Monstro damage/Death and source17/SFX72/42 callbacks',
                'randomized': kind == 'recorded-os-bll',
                'systems': ['enemies-bosses', 'damage-pipeline', 'sprite-effects', 'audio'],
            }
            case['bloodStartInputPhaseOffset'] = offset
            case['bloodStartBossIdentity'] = {
                'descriptorIndex': 0, 'gridIndex': 110 if kind == 'contrived978' else 59,
                'type': 5, 'variant': 1010 if kind == 'contrived978' else 1012, 'name': 'Monstro',
                'spawnSeed': 2321890733 if kind == 'contrived978' else 1050413383,
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
