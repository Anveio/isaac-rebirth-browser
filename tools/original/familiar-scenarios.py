#!/usr/bin/env python3
"""Declared public grant/appearance routes; never filter selected item IDs."""
import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('familiar_item_routes', ROOT / 'tools/original/item-scenarios.py')
routes = importlib.util.module_from_spec(spec)
spec.loader.exec_module(routes)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--case', type=Path)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.case:
        print(json.dumps(routes.tape(json.loads(args.case.read_text()))))
        return 0
    if args.validate_original:
        verifier = routes.load('familiar_transport', 'tools/original/foundation-clock-verify.py')
        failures = verifier.validate_run(json.loads(args.validate_original.read_text()))
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures}))
        return bool(failures)
    if not 2 <= args.count <= 4:
        raise ValueError('Familiar gate requires2..4 unfiltered OS-random routes')
    code = 'W0L0LYYM'
    cases = [{'name': f'familiar-phase-{delay}', 'code': code,
              'seedValue': routes.clock.codec.decode_seed(code), 'delay': delay,
              'actions': routes.route(delay, True, True),
              'entropy': 'Contrived public phase offset'} for delay in (0, 1)]
    cases.extend(routes.random_cases(args.count, True))
    for case in cases:
        # Both real games execute this public idle hold. It admits the full
        # appearance Poof lifetime; it is not an observed counter/RNG offset.
        case['actions'].append({'key': None, 'frames': 100})
        routes.tape(case)
    print(json.dumps({'schema': 1, 'mode': 'familiar-appearance', 'bootEpochSeed': 1700000000,
        'source': 'Two contrived grants plus recorded unfiltered OS-random treasure rooms; no selected-item/health/cache filtering',
        'cases': cases}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
