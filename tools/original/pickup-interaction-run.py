#!/usr/bin/env python3
"""Run one declared pickup-interaction case in the original game.

Stages the case's Challenge XML and Start Room catalogue over the unchanged
binary/resources (composition-run.py's resource view), replays the shared
public Challenge-menu tape and observes entities/Player words at every label.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--scenarios', type=Path, required=True)
    parser.add_argument('--case', required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--frame-timeout', default='60')
    parser.add_argument('--tape', type=Path, help='Replay this tape instead of the case tape (side-by-side footage adds captures)')
    parser.add_argument('--print-tape', action='store_true', help='Print the case public tape as JSON and exit')
    args = parser.parse_args()
    bank = json.loads(args.scenarios.read_text())
    matches = [case for case in bank['cases'] if case['name'] == args.case]
    if len(matches) != 1:
        raise SystemExit('Exactly one declared case must match')
    case = matches[0]
    base = args.scenarios.resolve().parent
    xml = (base / case['fixture']['authoredResourcePath']).resolve()
    stb = (base / case['fixture']['specialRoomsPath']).resolve()
    composition = load('interaction_composition_run', ROOT / 'tools/original/composition-run.py')
    clock = load('interaction_clock', ROOT / 'tools/original/foundation-clock-run.py')
    tape = composition.public_tape(case)
    if args.print_tape:
        print(json.dumps(tape))
        return 0
    if args.tape:
        tape = json.loads(args.tape.read_text())
    clock.validate_tape(tape)
    if args.output is None:
        raise SystemExit('--output is required to run the case')
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    composition.stage_view(xml, output / 'resource-view', stb)
    tape_path = output / 'declared-tape.json'
    tape_path.write_text(json.dumps(tape) + '\n')
    clock.ROOT = output / 'resource-view'
    sys.argv = [str(ROOT / 'tools/original/foundation-clock-run.py'), '--challenge-run',
                '--boot-epoch-seed', str(case['bootEpochSeed']), '--tape', str(tape_path),
                '--output', str(output / 'run'), '--frame-timeout', args.frame_timeout,
                '--state-observer', str(ROOT / 'tools/original/pickup-interaction-observe.py')]
    return clock.main()


if __name__ == '__main__':
    raise SystemExit(main())
