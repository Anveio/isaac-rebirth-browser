"""Join optional typed controlled-primary lifecycle events to saved Native tuples.

Remove-post and BaseUninit-entry are separate observations. Following Full
manager/reference cleanup is graded independently; neither is Uninit completion.
Existing functional reports are pinned inputs and are never rewritten.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
READER = HERE / 'composition-native-tuples.py'
IDENTITY = HERE.parent / 'original/native-identity.py'


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1048576), b''):
            h.update(block)
    return h.hexdigest()


def integer(value):
    if type(value) is not int or value < 0:
        raise ValueError('Actual nonnegative integer required')
    return value


def boolean(value):
    if type(value) is not bool:
        raise ValueError('Actual boolean owner required')
    return value


def identity(value):
    return tuple(integer(value[k]) for k in ('kind', 'variant', 'subtype', 'sequenceId', 'spawnSeed'))


def pin(path):
    st = path.stat()
    return dict(sha256=digest(path), bytes=st.st_size, mode=st.st_mode & 0o777,
                device=st.st_dev, inode=st.st_ino, mtimeNs=st.st_mtime_ns)


class Observations:
    """Only actual fields consumed by this new join, not a replacement snapshot."""
    def __init__(self):
        self.rows, self.births = [], {}

    def __len__(self):
        return len(self.rows)

    def append(self, row):
        source = row['source']
        out = dict(presentation=integer(row['presentation']),
                   absoluteGate=integer(row['nativeContext']['presentedFrame']),
                   counter=integer(row['nativeCounter']), actors={})
        if source['status'] != 'pass':
            raise ValueError('Actual source observation must pass')
        for actor in source['attacks']:
            key = identity(actor['identity'])
            if key in out['actors']:
                raise ValueError('Duplicate actual actor identity in one snapshot')
            try:
                seen = dict(manager=boolean(actor['inActiveManager']), active=boolean(actor['active']),
                            dead=boolean(actor['dead']), birth=integer(actor['bornGameAge']),
                            parent=None if actor['linkedParent'] is None else identity(actor['linkedParent']))
                self.births.setdefault(key, set()).add(seen['birth'])
            except (KeyError, ValueError, TypeError) as error:
                seen = dict(error=str(error))
            out['actors'][key] = seen
        try:
            refs = []
            for player in source['players']:
                for name in ('c18', 'c20', 'c28', 'c30'):
                    slot = player['actorSlots'][name]
                    if slot is not None:
                        refs.append(identity(slot['identity']))
            for actor in source['attacks'] + source['impacts']:
                parent = actor['linkedParent']
                if parent is not None:
                    refs.append(identity(parent))
                if actor['identity']['kind'] == 7:
                    child = actor['laser']['child']
                    if child is not None:
                        refs.append(identity(child))
            out['references'] = refs
        except (KeyError, ValueError, TypeError) as error:
            out['referencesError'] = str(error)
        self.rows.append(out)


def read_events(path, report, native_host, original, sources):
    if not (report['schema'] == 2 and report['actorScope'] == 'controlled-primary-v1'
            and report['status'] == 'pass' and report['errors'] == []
            and report['actualTracerExit'] == report['compile']['actualExitCode'] == 0
            and report['sourceBeforeEqualsEnd'] is True
            and report['sourceBefore'] == report['sourceEnd']
            and report['nativeBinarySha256'] == original):
        raise ValueError('Actual closed controlled-primary lifecycle tap required')
    for name, expected in sources.items():
        matches = [value for key, value in report['sourceBefore'].items() if Path(key).name == name]
        if len(matches) != 1 or matches[0]['sha256'] != expected:
            raise ValueError('Wrong actual lifecycle source owner')
    if report['journal'] != pin(path):
        raise ValueError('Actual lifecycle journal pin changed')
    target = report['target']
    games = native_host['ownedNativeGameClosure']['games']
    owned = [x for x in games if x['pid'] == target['pid'] and x['startTicks'] == target['startTicks']]
    if len(owned) != 1 or not (owned[0]['observedOwnedDuringRun'] is True
            and owned[0]['pidfdExitObserved'] is True and owned[0]['forcedCleanup'] is False
            and owned[0]['exePhysical'][:2] == [target['executableDevice'], target['executableInode']]):
        raise ValueError('Lifecycle target is not the actual owned Native game')
    events, gates, finish = [], [], None
    last_gate = last_ns = last_delivery = 0
    with path.open() as stream:
        for number, line in enumerate(stream, 1):
            row = json.loads(line)
            if row['record'] != number or finish is not None:
                raise ValueError('Lifecycle record order changed')
            kind = row['kind']
            if number == 1:
                if not (kind == 'start' and row['schema'] == 2
                        and row['threadId'] == target['pid'] and row['ownerPid'] == target['parentPid']
                        and row['startTicks'] == target['startTicks']
                        and row['removePost'] == 0x441bc7 and row['uninitEntry'] == 0x448040
                        and row['scope'] == 'leader-thread2/0/0bit33-and7/1/1-common-remove-post-and-base-uninit-entry'
                        and row['actorMemoryWritten'] is False and row['instructionsWritten'] is False):
                    raise ValueError('Actual typed lifecycle start is unbound')
            elif kind == 'blocked-gate':
                gate, ns = integer(row['presentedFrame']), integer(row['virtualElapsedNs'])
                if gate < last_gate or ns < last_ns:
                    raise ValueError('Actual blocked gates moved backwards')
                last_gate, last_ns = gate, ns
                gates.append(row)
            elif kind == 'event':
                if type(row['attackFlags']) is not str or not row['attackFlags'].isascii() or not row['attackFlags'].isdigit():
                    raise ValueError('Actual decimal u64 attack flags required')
                key, flags = identity(row['actor']), int(row['attackFlags'])
                if flags > 0xffffffffffffffff:
                    raise ValueError('Actual attack flags overflow u64')
                if (row['eventId'] != len(events) + 1 or integer(row['delivery']) <= last_delivery
                        or row['threadId'] != target['pid'] or integer(row['address']) == 0
                        or row['previousBlockedGate'] != last_gate or row['previousVirtualElapsedNs'] != last_ns):
                    raise ValueError('Actual event identity/order/gate is unbound')
                last_delivery = row['delivery']
                if key[:3] == (2, 0, 0):
                    if row['vptr'] != 0x671270 or not flags & (1 << 33) or row['fast322'] is not None:
                        raise ValueError('Actual controlled Tear scope is invalid')
                elif key[:3] == (7, 1, 1):
                    if row['vptr'] != 0x66cf70:
                        raise ValueError('Actual controlled Laser class is invalid')
                    boolean(row['fast322'])
                else:
                    raise ValueError('Event is outside the explicit controlled-primary scope')
                integer(row['systemCounter']); integer(row['parent498'])
                for name in ('visible', 'active', 'dead'):
                    boolean(row[name])
                pc = {'remove-post': 0x441bc7, 'base-uninit-entry': 0x448040}.get(row['event'])
                if pc != row['sourcePc'] or (row['event'] == 'remove-post'
                        and (row['active'] is not False or row['dead'] is not True)):
                    raise ValueError('Actual boundary PC/flags are invalid')
                events.append(row)
            elif kind in ('group-stop-listen', 'signal-delivery'):
                integer(row['signal'])
            elif kind == 'finish':
                if not (row['status'] == 'pass' and row['detached'] is True
                        and row['debugRegistersRestoredExactly'] is True
                        and row['events'] == len(events) and row['gates'] == len(gates)
                        and integer(row['deliveries']) >= last_delivery):
                    raise ValueError('Actual lifecycle closure failed')
                finish = row
            else:
                raise ValueError('Unknown lifecycle record kind')
    if finish is None or finish != report['finish'] or report['journal'] != pin(path):
        raise ValueError('Actual lifecycle finish/counts/pins changed')
    return events


def compare(case, observations, events):
    checks, joins, chains = [], [], []
    def check(name, expected, actual, event=None):
        passed = type(expected) is type(actual) and expected == actual
        checks.append(dict(field=name, expected=expected, actual=actual, passed=passed,
                           eventId=None if event is None else event['eventId']))
        return passed
    def missing(name, reason):
        check(name, 'observed', 'unobservable: ' + reason)
    expected_kinds = set()
    for stage in case['requires']['actorStages']:
        actor = stage['sourceExpectedActorContract']
        if (actor['family'], actor['variant'], actor['subtype']) in ((2, 0, 0), (7, 1, 1)):
            expected_kinds.add(actor['family'])
    by_actor = {}
    for event in events:  # Preserve delivery order; never sort an event trace.
        key = identity(event['actor'])
        prior = [r for r in observations.rows if r['absoluteGate'] <= event['previousBlockedGate']
                 and r['counter'] <= event['systemCounter'] and key in r['actors']]
        birth = observations.births.get(key, set())
        joined = bool(prior) and len(birth) == 1
        check('exact event identity and unique observed source birth', True, joined, event)
        joins.append(dict(event=event, nativeBirth=None if len(birth) != 1 else next(iter(birth)),
                          priorObservedGate=None if not prior else prior[-1]['absoluteGate'], joined=joined))
        by_actor.setdefault(key, []).append(event)
        if not joined:
            continue
        check('actual event Parent pointer nonnull', True, event['parent498'] != 0, event)
        check('prior observed actual ParentPlayer identity', True,
              any(r['actors'][key].get('parent', ()) is not None
                  and r['actors'][key].get('parent', ())[:3] == (1, 0, 0) for r in prior), event)
    for key, actual in by_actor.items():
        for entry in (x for x in actual if x['event'] == 'base-uninit-entry'):
            earlier = [x for x in actual if x['event'] == 'remove-post' and x['record'] < entry['record']]
            if not check('actual same-actor Remove-post precedes BaseUninit-entry', True, bool(earlier), entry):
                continue
            removal = earlier[-1]
            pair = (removal['address'], removal['parent498'])
            check('same event pointer pair retained to BaseUninit-entry', pair,
                  (entry['address'], entry['parent498']), entry)
            following = [r for r in observations.rows if r['absoluteGate'] > entry['previousBlockedGate']
                         and r['counter'] >= entry['systemCounter'] and r['counter'] % 2 == 1]
            chain = dict(identity=key, removeEventId=removal['eventId'], baseUninitEntryEventId=entry['eventId'],
                         followingFull=None, scope='Entry is not completion; cleanup is a separate observation.')
            if not following:
                missing('first captured following Full cleanup', 'No following actual Full snapshot')
            else:
                first = following[0]
                group = [r for r in following if r['absoluteGate'] == first['absoluteGate']]
                same = all(r == first for r in group)
                check('following Full duplicate cleanup owners agree', True, same, entry)
                actor = first['actors'].get(key)
                manager_known = actor is None or 'manager' in actor
                check('following Full manager owner observed', True, manager_known, entry)
                if manager_known:
                    check('independent following Full manager absence', False,
                          False if actor is None else actor['manager'], entry)
                refs_known = 'references' in first
                check('following Full known reference owners observed', True, refs_known, entry)
                if refs_known:
                    check('independent following Full observed references clear', False,
                          key in first['references'], entry)
                chain['followingFull'] = dict(absoluteGate=first['absoluteGate'], counter=first['counter'],
                    eventCounter=entry['systemCounter'], counterDistance=first['counter']-entry['systemCounter'])
            chains.append(chain)
    for kind in sorted(expected_kinds):
        for boundary in ('remove-post', 'base-uninit-entry'):
            check('required actual type' + str(kind) + ' ' + boundary, True,
                  any(e['actor']['kind'] == kind and e['event'] == boundary for e in events))
    missing('actual subclass Uninit completion', '448040 is BaseUninit entry only; later absence is not completion.')
    missing('actual admitted live-reapply callback', 'This tap does not observe51C73C/51C777.')
    missing('actual initial Full-before-C18 call-order', 'Remove/BaseUninit observations do not trace this call order.')
    missing('paired port Remove/BaseUninit callback events', 'No captured port callback sidecar is consumed.')
    return dict(status='PASS' if all(x['passed'] for x in checks) else 'FAIL', checks=checks, eventJoins=joins, chains=chains)


def main():
    p = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    for key in ('case', 'native-tuples', 'native-host', 'events', 'tap-report', 'predecessor-report', 'output'):
        p.add_argument('--'+key, type=Path, required=True)
    p.add_argument('--expected-case-sha256', required=True)
    p.add_argument('--lifecycle-c-source', type=Path, required=True,
                   help='Actual captured lifecycle C source, bound to the tap report.')
    p.add_argument('--lifecycle-python-source', type=Path, required=True,
                   help='Actual captured lifecycle Python source, bound to the tap report.')
    p.add_argument('--projector-source', type=Path, required=True,
                   help='Actual compact-tuple projector source, bound to projection metadata.')
    a = p.parse_args()
    inputs = [Path(__file__), READER, IDENTITY, a.lifecycle_c_source,
              a.lifecycle_python_source, a.projector_source, a.case, a.native_tuples, a.native_host, a.events, a.tap_report, a.predecessor_report]
    assert a.output.resolve() not in [x.resolve() for x in inputs]
    before = {str(x): pin(x) for x in inputs}
    assert digest(a.case) == a.expected_case_sha256
    identity_spec = importlib.util.spec_from_file_location('event_join_native_identity', IDENTITY)
    identity_owner = importlib.util.module_from_spec(identity_spec)
    identity_spec.loader.exec_module(identity_owner)
    original = identity_owner.NATIVE_SHA
    sources = {'composition-lifecycle.c': digest(a.lifecycle_c_source),
               'composition-lifecycle.py': digest(a.lifecycle_python_source)}
    case, host, tap = [json.loads(x.read_text()) for x in (a.case, a.native_host, a.tap_report)]
    assert host['case'] == case['name'] and host['actualWait'] is True
    assert host['exitCode'] == 0 and host['guardStop'] is None and host['sourceAndDeclaredInputsBeforeEqualsEnd'] is True
    assert host['status'] == 'CAPTURE_PROCESS_CLOSED'
    spec = importlib.util.spec_from_file_location('controlled_event_tuple_reader', READER)
    reader = importlib.util.module_from_spec(spec); spec.loader.exec_module(reader)
    observations = Observations()
    retained = reader.native_input(a.native_tuples, observations)
    assert retained['case'] == case and retained['presentationBasis'] == 'actual-native-title-relative'
    assert retained['projection']['toolSha256'] == digest(a.projector_source)
    original_root = Path(retained['projection']['journalPath']).parent
    meta = Path(retained['projection']['journalPath']+'.meta.json')
    assert digest(meta) == retained['projection']['journalMetadataSha256'] and json.loads(meta.read_text())['status'] == 'closed'
    assert a.events.resolve() == (original_root/'lifecycle/events.jsonl').resolve()
    assert a.tap_report.resolve() == a.events.with_name('report.json').resolve()
    driver_path = original_root / 'composition-input.json'
    before[str(driver_path)] = pin(driver_path)
    driver = json.loads(driver_path.read_text())
    assert driver['execution'] == 'original-closed' and driver['originalExitStatus'] == 0
    assert driver['nativeBinarySha256'] == original
    assert driver['declaredResourcesBeforeEqualsEnd'] is True
    command = host['command']
    assert Path(command[command.index('--output')+1]).resolve() == original_root.resolve()
    assert '--composition-lifecycle-events' in command
    assert command[command.index('--composition-lifecycle-scope')+1] == 'controlled-primary-v1'
    assert all(r['absoluteGate']-r['presentation'] == retained['nativeTitlePresentedFrame'] for r in observations.rows)
    events = read_events(a.events, tap, host, original, sources)
    result = compare(case, observations, events)
    result.update(schema='controlled-primary-native-event-join-v1', actualEvents=len(events),
        predecessorReportSha256=before[str(a.predecessor_report)]['sha256'],
        sourceBindings=dict(nativeIdentityProvider=str(IDENTITY), nativeBinarySha256=original,
            lifecycleSources=sources, projectorSha256=digest(a.projector_source),
            driverInput=str(driver_path)),
        scope='Additive Native-only typed event evidence. Existing functional grades are untouched. Actual Remove-post and BaseUninit entry are distinct; no completion, port callback, live-reapply or initial-call-order inference.',
        parentPointerScope='Event pointer nonnull/constancy only; original Parent identity is independently observed in snapshots, not reread at the event.',
        referenceScope='Only actual observed Player slots, attack/Impact Parent and Laser Child fields; not a universal weak-reference list.', before=before)
    assert all(before[str(x)] == pin(x) for x in inputs + [driver_path])
    assert digest(meta) == retained['projection']['journalMetadataSha256']
    with a.output.open('x') as stream:
        json.dump(result, stream, indent=2); stream.write('\n')
    print(json.dumps(dict(status=result['status'], checks=len(result['checks']),
        failures=sum(not x['passed'] for x in result['checks']), actualEvents=len(events))))


if __name__ == '__main__':
    main()
