#!/usr/bin/env python3
"""Controls front door followed by declared normal-Isaac seeded Restart holds."""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import secrets

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('restart_controls_recipe', ROOT / 'tools/original/controls-scenarios.py')
controls = importlib.util.module_from_spec(spec)
spec.loader.exec_module(controls)
SOURCE = 'OS secrets; first eight unfiltered bytes, little-endian u32 run max1 and independent-u32 boot epoch'


def build(seed, boot, phase, paused, randomized, draw_hex=None):
    data = controls.case(seed, boot, 'controls-os-movement-up-w-to-i-v1', True)
    data = copy.deepcopy(data)
    identifier = f'living-seeded-r61-phase{phase}' + ('-paused' if paused else '') + ('-os' if randomized else '')
    data['scenario'] = dict(id=identifier, randomized=randomized,
                            scope='Public Controls/Back/typed-seed normal Isaac, menu exclusion, R60/release/R61 and deferred Restart. No outcome filter or resampling.')
    if randomized:
        raw = bytes.fromhex(draw_hex)
        if len(raw) != 8 or max(1, int.from_bytes(raw[:4], 'little')) != seed or int.from_bytes(raw[4:], 'little') != boot:
            raise ValueError('Recorded raw OS bytes differ from their declared projection')
        data['entropy'] = dict(source=SOURCE, drawBytesHex=draw_hex, seedValue=seed, bootEpochSeed=boot)
    else:
        data.pop('entropy', None)
    tape = data['pairedTape']
    end = next(i for i, action in enumerate(tape) if action.get('label') == 'remap-gameplay-start')
    tape = tape[:end + 1]
    main = next(i for i, action in enumerate(tape) if action.get('label') == 'main-ready')
    tape[main + 1:main + 1] = [dict(type='key', key='r', down=True), dict(type='step', frames=61),
                              dict(type='capture', label='living-menu-r61'), dict(type='key', key='r', down=False),
                              dict(type='step', frames=1)]
    contract = {'living-menu-r61': dict(menu=True)}

    def capture(label, count, pending=False, dispatch=False):
        tape.append(dict(type='capture', label=label))
        contract[label] = dict(menu=False, heldPolls=count, pending=pending, dispatch=dispatch)

    if phase:
        tape.append(dict(type='step', frames=phase))
    if paused:
        tape.extend([dict(type='key', key='Escape', down=True), dict(type='step', frames=1),
                     dict(type='key', key='Escape', down=False), dict(type='step', frames=20)])
    capture('living-before-r', 0)
    tape.append(dict(type='key', key='r', down=True))
    for n in range(1, 61):
        tape.append(dict(type='step', frames=1))
        capture(f'living-first-hold-{n}', n)
    tape.extend([dict(type='key', key='r', down=False), dict(type='step', frames=1)])
    capture('living-release-resets', 0)
    tape.append(dict(type='key', key='r', down=True))
    for n in range(1, 62):
        tape.append(dict(type='step', frames=1))
        capture(f'living-second-hold-{n}', n if n < 61 else 0, pending=n == 61)
    tape.append(dict(type='step', frames=1))
    capture('living-dispatched-held', 1, dispatch=True)
    tape.extend([dict(type='key', key='r', down=False), dict(type='step', frames=1)])
    capture('living-new-release', 0, dispatch=True)
    data['pairedTape'] = tape
    data['tape'] = data['bootstrap']['sourceOnlyPrelude'] + tape
    data['livingRestart'] = dict(schema=1, phase=phase, paused=paused, contract=contract,
                                expectedSeed=controls.codec.encode_seed(seed), scope='Normal seeded Isaac only; general/dead/Challenge and unseeded Shared request remain separate.')
    return data


def validate(bank):
    if type(bank.get('schema')) is not int or bank['schema'] != 1 or not isinstance(bank.get('cases'), list) or not bank['cases']:
        raise ValueError('Restart requires a nonempty declared case bank')
    ids = set()
    fixed = random = 0
    for data in bank['cases']:
        mode = data.get('livingRestart', {})
        randomized = data.get('scenario', {}).get('randomized')
        if type(randomized) is not bool or mode.get('phase') not in (0, 1) or type(mode.get('paused')) is not bool:
            raise ValueError('Unknown Restart route declaration')
        if randomized and (mode['phase'] != 0 or mode['paused']):
            raise ValueError('The OS case retains the declared phase0 unpaused route')
        if not randomized and (data['seedValue'] != controls.codec.decode_seed('T1PK6Y0W')
                               or data['bootEpochSeed'] != 0
                               or (mode['phase'], mode['paused']) not in [(0, False), (1, False), (0, True)]):
            raise ValueError('The fixed Restart controls retain their declared seed/epoch/phase/pause inputs')
        expected = build(data['seedValue'], data['bootEpochSeed'], mode['phase'], mode['paused'], randomized,
                         data.get('entropy', {}).get('drawBytesHex'))
        if data != expected or data['scenario']['id'] in ids:
            raise ValueError('Replay must preserve the complete declared input route and independent obligations')
        ids.add(data['scenario']['id'])
        random += int(randomized)
        fixed += int(not randomized)
    filtered = bank.get('caseFilter')
    if filtered is not None and (not isinstance(filtered, str) or not filtered):
        raise ValueError('A filtered Restart replay declares its nonempty selection')
    if not filtered and (fixed != 3 or random != 1):
        raise ValueError('Restart requires three fixed controls and the first unfiltered OS case')
    return dict(status='pass', cases=len(ids), fixed=fixed, recordedOS=random)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--validate-bank', type=Path)
    args = parser.parse_args()
    if args.validate_bank:
        if args.validate_bank.stat().st_size > 4 * 1024 * 1024:
            raise ValueError('Restart bank exceeds4MiB')
        print(json.dumps(validate(json.loads(args.validate_bank.read_text()))))
        return
    # All fixed controls are constructed before the one unfiltered OS draw.
    cases = [build(controls.codec.decode_seed('T1PK6Y0W'), 0, phase, paused, False)
             for phase, paused in [(0, False), (1, False), (0, True)]]
    raw = secrets.token_bytes(8)
    seed = max(1, int.from_bytes(raw[:4], 'little'))
    boot = int.from_bytes(raw[4:], 'little')
    cases.append(build(seed, boot, 0, False, True, raw.hex()))
    bank = dict(schema=1, cases=cases)
    validate(bank)
    print(json.dumps(bank))


if __name__ == '__main__':
    main()
