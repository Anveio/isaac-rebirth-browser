"""Public pocket fixtures: retained cases plus every freshly recorded OS choice."""
import argparse
import copy
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(allow_abbrev=False)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--mode', choices=('all', 'init', 'collection'), default='all')
    parser.add_argument('--count', type=int, default=1)
    args = parser.parse_args()
    if not 0 <= args.count <= 64:
        raise ValueError('Fresh unfiltered OS count must be0..64')
    source = ROOT / 'references/isaac/foundation/pockets/history/scenarios.json'
    original = json.loads(source.read_text())
    args.output.mkdir(parents=True, exist_ok=False)
    kinds = ('init', 'collection') if args.mode == 'all' else (args.mode,)
    def kind(case):
        if ('pocketInit' in case) == ('pocketCollection' in case):
            raise ValueError('One exact pocket contract per case required')
        return 'init' if 'pocketInit' in case else 'collection'
    cases = [copy.deepcopy(case) for case in original['cases'] if kind(case) in kinds]
    for case in cases:
        for key in ('authoredResourcePath', 'specialRoomsPath'):
            case['fixture'][key] = str((source.parent / case['fixture'][key]).resolve())
    templates = {name: next(case for case in cases if kind(case) == name
                            and case['publicMenuRoute'] == 'retained-unfiltered-challenge') for name in kinds}
    entropy_path = args.output / 'os-entropy.jsonl'
    with entropy_path.open('x') as journal:
        for index in range(args.count):
            raw = os.urandom(4)
            epoch = int.from_bytes(raw, 'little')
            choice = dict(index=index, rawHex=raw.hex(), bootEpochSeed=epoch,
                          initialPhaseOffset=(epoch >> 31) & 1,
                          rule='Direct unfiltered little-endianu32 epoch; highbitphase, no redraw/seed search')
            journal.write(json.dumps(choice)+'\n'); journal.flush(); os.fsync(journal.fileno())
            for name in kinds:
                case = copy.deepcopy(templates[name])
                case['name'] = 'pocket-' + name + '-fresh-os-' + str(index)
                case['bootEpochSeed'] = epoch
                case['initialPhaseOffset'] = choice['initialPhaseOffset']
                case['entropy'] = copy.deepcopy(choice)
                #Positive settings do not weaken muted-case assertions; every
                #new preference is independently passed to both real games.
                case['sfxVolume'] = .5 if epoch & 1 else 1
                cases.append(case)
    result = dict(schema=1, cases=cases,
        source='Retained fixed and recorded OS cases, plus every fresh journaled unfiltered OS projection; no game/output selection.')
    (args.output / 'scenarios.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(status='declared-not-executed', cases=len(cases), output=str(args.output))))


if __name__ == '__main__':
    main()
