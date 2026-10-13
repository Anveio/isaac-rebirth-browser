#!/usr/bin/env python3
"""Public natural Troll/Megatroll routes with required unfiltered OS draws."""
import argparse
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('troll_public_pickups', ROOT / 'tools/original/bomb-pickup-scenarios.py')
pickups = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pickups)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--count', type=int, default=2)
    parser.add_argument('--case', type=Path)
    parser.add_argument('--validate-original', type=Path)
    args = parser.parse_args()
    if args.validate_original:
        verifier = pickups.load('troll_provenance', 'tools/original/foundation-clock-verify.py')
        failures = verifier.validate_run(json.loads(args.validate_original.read_text()))
        print(json.dumps({'status': 'fail' if failures else 'pass', 'failures': failures}))
        return 1 if failures else 0
    if args.case:
        print(json.dumps(pickups.rock.tape(json.loads(args.case.read_text()))))
        return 0
    if not 2 <= args.count <= 4:
        raise ValueError('Conversion coverage requires 2..4 new unfiltered OS-random draws')
    recorded = json.loads((ROOT / 'references/isaac/foundation/troll-bombs/contrived-cases.json').read_text())
    print(json.dumps({'schema': 1, 'bootEpochSeed': 1700000000,
                      'source': 'Actual public seeds and held-key intervals; no captured state enters production',
                      'scope': 'Retained XD0 and natural Megatroll research routes plus newly recorded OS-random ordinary-rock access. Pickup subtype outcomes are unfiltered; every unsupported draw remains required and blocking.',
                      'cases': recorded['cases'] + pickups.random_cases(args.count)}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
