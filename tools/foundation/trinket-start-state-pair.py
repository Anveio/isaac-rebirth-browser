"""Add actual Challenge41 fields and Native call events to the ordinary grade.

The existing composition comparer remains unchanged and independently required.
Native observations are evidence only, never Browser inputs. No fitted clock,
identity, seed or absent-field substitution is used here.
"""
import argparse
import gzip
import hashlib
import json
from pathlib import Path


def load(path):
    raw = path.read_bytes()
    return json.loads(gzip.decompress(raw) if path.suffix == '.gz' else raw)


def file_pin(path):
    before = path.stat()
    raw = path.read_bytes()
    after = path.stat()
    fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_ctime_ns')
    if any(getattr(before, field) != getattr(after, field) for field in fields):
        raise ValueError('Captured evidence changed while reading')
    return {'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw),
            'mode': before.st_mode & 0o777, 'device': before.st_dev,
            'inode': before.st_ino, 'mtimeNs': before.st_mtime_ns}


def rng(value):
    if not isinstance(value, dict):
        raise ValueError('Unknown actual RNG owner')
    words = [value[key] for key in ('state', 'rightA', 'left', 'rightB')]
    if any(type(word) is not int or not 0 <= word <= 0xffffffff for word in words):
        raise ValueError('Invalid actual four-word RNG')
    return words


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--case', type=Path, required=True)
    parser.add_argument('--original', type=Path, required=True)
    parser.add_argument('--browser', type=Path, required=True)
    parser.add_argument('--base-comparison', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    checks, coverage = [], {'pairedPlaying': 0, 'postStartup': 0}

    def check(field, expected, actual, presentation=None):
        checks.append({'field': field, 'presentation': presentation,
                       'expected': expected, 'actual': actual,
                       'passed': type(expected) is type(actual) and json.dumps(expected, sort_keys=True, allow_nan=False) == json.dumps(actual, sort_keys=True, allow_nan=False)})

    def missing(field, error, presentation=None):
        checks.append({'field': field, 'presentation': presentation,
                       'passed': False, 'error': str(error)})

    try:
        case, native, browser = load(args.case), load(args.original), load(args.browser)
        base = load(args.base_comparison)
        check('unchanged ordinary/default-kind1 composition grade', 'pass', base['status'])
        check('declared startup contract', 'source-public-challenge-start41-v1',
              case['requires']['trinketKind2Startup']['schema'])
        check('declared typed starting config', [41], case['fixture']['startingTrinkets'])
        check('no starting collectible namespace alias', [], case['fixture']['startingItems'])
        check('actual complete Native', ('pass', []), (native['status'], native['failures']))
        check('actual complete Browser', 'complete', browser['captureStatus'])

        # This journal is produced by actual hardware deliveries, not by the
        # read-only presentation observer or a source-order prediction.
        directory = args.original.parent / 'trinket-start-events'
        metadata = native['trinketStartEvents']
        for field, expected in [('status', 'pass'), ('actorMemoryWritten', False),
                                ('instructionsWritten', False),
                                ('cpuDebugRegistersTemporarilyWritten', True),
                                ('sourceBeforeEqualsEnd', True), ('actualTracerExit', 0)]:
            check('actual event tap ' + field, expected, metadata[field])
        event_path = directory / 'events.jsonl'
        check('closed actual event bytes/identity', metadata['journal'], file_pin(event_path))
        events, finished, records = [], None, 0
        for records, line in enumerate(event_path.read_text().splitlines(), 1):
            row = json.loads(line)
            if finished is not None or row['record'] != records:
                raise ValueError('Actual event record order/finish changed')
            if row['kind'] == 'event':
                if row['eventId'] != len(events) + 1:
                    raise ValueError('Actual invocation IDs changed')
                events.append(row)
            elif row['kind'] == 'finish':
                finished = row
        if finished is None:
            raise ValueError('Actual event journal lacks finish')
        for field, expected in [('status', 'pass'), ('detached', True),
                                ('debugRegistersRestoredExactly', True), ('events', 4)]:
            check('actual event closure ' + field, expected, finished[field])
        expected_events = [('give-entry', 0x518890), ('give-return', 0x52e2da),
                           ('remove-entry', 0x581770), ('remove-return', 0x52e2ef)]
        check('actual four Native invocation boundaries', expected_events,
              [(row['event'], row['sourcePc']) for row in events])
        if len(events) != 4:
            raise ValueError('Four actual callee entry/return deliveries required')
        give, given, remove, removed = events
        check('actual Give callee stack return owner', 0x52e2da, give['returnAddress'])
        check('actual Remove callee stack return owner', 0x52e2ef, remove['returnAddress'])
        for index, row in enumerate(events):
            for field, expected in [('declaredTrinketId', 41), ('configKind', 2), ('configId', 41)]:
                check('actual invocation ' + field, expected, row[field], index)
            check('actual invocation Player type', (1, 0),
                  (row['player']['entityType'], row['player']['variant']), index)
            check('actual invocation collectible41 unchanged', 0, row['player']['collectible41Count'], index)
            check('actual invocation before final3074 writer', False, row['player']['initialized3074'], index)
            check('actual invocation same Player',
                  (give['player']['sequenceId'], give['player']['spawnSeed']),
                  (row['player']['sequenceId'], row['player']['spawnSeed']), index)
            check('actual invocation no secondary RNG draw', give['pool41']['secondaryRngWords'],
                  row['pool41']['secondaryRngWords'], index)
            check('actual invocation no child41 RNG draw', give['player']['child41RngWords'],
                  row['player']['child41RngWords'], index)
        check('actual Give entry held empty', [0, 0], give['player']['held'])
        for row in events[1:]:
            check('actual Give committed held41', [41, 0], row['player']['held'])
        check('actual Give did not remove pool41', give['pool41'], given['pool41'])
        check('actual Remove entry follows Give', given['pool41'], remove['pool41'])
        check('actual Remove41 initially available/eligible', (True, True),
              (remove['pool41']['available'], remove['pool41']['eligible']))
        check('actual Remove returned true', True, removed['removeReturnBool'])
        check('actual Remove clears availability', False, removed['pool41']['available'])
        check('actual Remove preserves eligibility', remove['pool41']['eligible'], removed['pool41']['eligible'])
        check('actual Remove decrements signed count', remove['pool41']['remainingCount'] - 1,
              removed['pool41']['remainingCount'])

        native_title = next(row['context']['presentedFrame'] for row in native['debugObservations']
                            if row['context']['label'] == 'title')
        browser_title = next(row['presentation'] for row in browser['captures'] if row['label'] == 'title')
        native_rows = {}
        for row in native['debugObservations']:
            t = row['context']['presentedFrame'] - native_title
            observed = row['observation']
            owner = observed.get('trinketStartup')
            players = observed.get('composition', {}).get('players', [])
            initialized = players[0].get('initialized3074') if len(players) == 1 else None
            value = (owner, initialized)
            if t in native_rows:
                check('same-gate additive observations identical', native_rows[t], value, t)
            else:
                native_rows[t] = value
        for captured in sorted([*browser['preludeStates'], *browser['states']],
                               key=lambda row: row['presentation']):
            t = captured['presentation'] - browser_title
            state = captured['state']
            if state['screen'] != 'playing':
                continue
            try:
                owner, initialized = native_rows[t]
                if not isinstance(owner, dict) or owner['status'] != 'pass':
                    raise ValueError('Actual same-gate trinket owner missing/failed')
                check('read-only Native owner', (False, False),
                      (owner['memoryWritten'], owner['executableMappingsRead']), t)
                check('actual sole Player', 1, owner['playerCount'], t)
                if len(owner['players']) != 1:
                    raise ValueError('Actual sole Player data missing')
                original, items = owner['players'][0], state['playerItems']
                pool = state['itemPools']['trinkets']
                if not isinstance(pool, dict) or not isinstance(pool['eligible'], list):
                    raise ValueError('Browser current pool/eligibility unknown')
                check('actual held slots', original['heldTrinketSlots'], items['trinketSlots'], t)
                check('actual collectible41 namespace', original['collectible41CountRaw'], items['ownedCounts'][41], t)
                check('actual pending cache', original['pendingCacheFlags'], items['unevaluatedCacheFlags'], t)
                check('actual direct startup queue absent', False, original['queued']['present'], t)
                #None kind is omitted by the actual serde contract; this
                #only accepts it together with the explicitly NULL queue ID.
                check('actual Browser direct startup queue absent', (None, None),
                      (items['queuedId'], items.get('queuedConfigKind')), t)
                check('actual child41 RNG', original['child41RngWords'], rng(items['trinketRngs'][41]), t)
                check('actual secondary ItemPool RNG', owner['trinketSelectionRngWords'],
                      rng(state['itemPools']['secondaryRng']), t)
                entries = owner['trinketPool']['entries']
                check('actual62 pool IDs', list(range(62)), [entry['id'] for entry in entries], t)
                check('actual62 availability bytes', [entry['available'] for entry in entries], pool['available'], t)
                check('actual62 eligibility bytes', [entry['eligible'] for entry in entries], pool['eligible'], t)
                check('actual signed pool count', owner['trinketPool']['remainingCount'], pool['count'], t)
                check('actual Native final initialized3074', True, initialized, t)
                check('actual final initialized3074', initialized, state['player']['initialized3074'], t)
                check('actual Native completed cache', 0, original['pendingCacheFlags'], t)
                check('actual Native startup identity',
                      (give['player']['sequenceId'], give['player']['spawnSeed']),
                      (original['sequenceId'], original['spawnSeed']), t)
                check('actual completed Native/Browser held41', ([41, 0], [41, 0]),
                      (original['heldTrinketSlots'], items['trinketSlots']), t)
                check('actual completed pool41 availability', False, entries[41]['available'], t)
                coverage['pairedPlaying'] += 1
                coverage['postStartup'] += 1
            except Exception as error:
                missing('actual same-gate Challenge41 fields', error, t)
        check('actual post-startup paired field witnesses', True, coverage['postStartup'] > 0)
    except Exception as error:
        missing('required actual startup evidence', error)
    result = {
        'schema': 1, 'status': 'pass' if checks and all(row['passed'] for row in checks) else 'fail',
        'checks': checks, 'coverage': coverage,
        'scope': 'Additive actual direct Challenge41 fields and Native entry/return invocations; ordinary/default-kind1 grade independently required and unchanged',
        'unqualified': ['Browser internal callee invocation count', 'typed queue/contact350', 'randomGet',
                        'all41 effect sites', '50/foreign grants', 'held Continue'],
    }
    raw = (json.dumps(result, separators=(',', ':')) + '\n').encode()
    args.output.write_bytes(gzip.compress(raw, mtime=0) if args.output.suffix == '.gz' else raw)
    print(json.dumps({'status': result['status'], 'checks': len(checks), 'coverage': coverage}))
    return 0 if result['status'] == 'pass' else 1


if __name__ == '__main__':
    raise SystemExit(main())
