"""Independent retained-game input/transport/closed-evidence readback."""
import argparse, configparser, gzip, hashlib, importlib.util, json
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--case', type=Path, required=True)
p.add_argument('--directory', type=Path, required=True)
p.add_argument('--native-harness', type=Path, default=Path(__file__).resolve().parents[2])
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
checks = []


def checked(name, fn):
    try:
        fn()
        checks.append({'field': name, 'passed': True})
    except Exception as error:
        checks.append({'field': name, 'passed': False, 'error': str(error)})


def expect(value, expected):
    assert value == expected, (value, expected)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


case = json.loads(a.case.read_bytes())
native_dir, browser_dir = a.directory / 'original', a.directory / 'browser'
pins = {str(path): digest(path) for directory in (native_dir, browser_dir) if directory.exists()
        for path in directory.rglob('*') if path.is_file() and not path.is_symlink() and '__pycache__' not in path.parts}
native, browser = None, None


def native_readback():
    global native
    manifest = native_dir / 'manifest.json.gz'
    for path in (manifest, Path(str(manifest) + '.meta.json'), native_dir / 'native-progress.jsonl.gz',
                 native_dir / 'native-progress.jsonl.gz.meta.json'):
        assert path.is_file(), 'Actual declared138 archive/metadata/journal required: ' + str(path)
    module = a.native_harness / 'tools/original/native-evidence.py'
    spec = importlib.util.spec_from_file_location('composition_native_evidence', module)
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    native = reader.read_manifest(manifest)  # Exact journal reconstruction, CRC, SHA/bytes, finish/closed.
    expect(native['status'], 'pass')
    expect(native['failures'], [])
    expect(native['seedMode'], 'source-challenge-menu')
    expect(native['bootEpochSeed'], case['bootEpochSeed'])
    expect(native['profile'], {'fresh': True, 'isolated': True})
    expect(native['declaredOptionsKept'], True)
    expect(native['hostExecution']['lpNumThreads'], '2')
    expect(native['debugObserver']['everyTapeStep'], True)
    expect(native['focusHandoffs'] if 'focusHandoffs' in native else [], [])
    expect(native['inputDelivery']['serverAutoRepeat'], False)
    # PR176 exposed long-profile default fallback. Grade actual retained
    # options for this frozen producer rather than assuming its declarations.
    options = configparser.ConfigParser()
    assert options.read(native_dir / 'profile/data/binding of isaac rebirth/options.ini')
    for name, expected in [('VSync', 0), ('PauseOnFocusLost', 0), ('WindowWidth', 960),
                           ('WindowHeight', 540), ('MusicVolume', 0), ('SFXVolume', 0)]:
        expect(options.getfloat('Options', name), expected)
    # Independent native bootstrap grammar. Dense admission begins at the
    # actual confirm; no gameplay state during those312 steps is discarded.
    tape = [{'type': 'step', 'frames': 180}, {'type': 'capture', 'label': 'intro'}]
    for key, label, settle in [('Return', 'title', 72), ('Return', 'profile', 72), ('Return', 'main', 72),
                               ('Down', 'challenge-selected', 72), ('Return', 'challenges', 72), ('Return', 'gameplay-start', 300)]:
        tape.append({'type': 'key', 'key': key, 'down': True})
        tape += [{'type': 'step', 'frames': 1}] * 12 if label == 'gameplay-start' else [{'type': 'step', 'frames': 12}]
        tape.append({'type': 'key', 'key': key, 'down': False})
        tape += [{'type': 'step', 'frames': 1}] * settle if label == 'gameplay-start' else [{'type': 'step', 'frames': settle}]
        tape.append({'type': 'capture', 'label': label})
    if case['initialPhaseOffset']:
        tape.append({'type': 'step', 'frames': 1})
    held = set()
    for action in case['actions']:
        keys = set(action['keys'])
        tape += [{'type': 'key', 'key': key, 'down': False} for key in sorted(held - keys)]
        tape += [{'type': 'key', 'key': key, 'down': True} for key in sorted(keys - held)]
        for index in range(1, action['presentations'] + 1):
            tape.append({'type': 'step', 'frames': 1})
            if index in (3, 4):
                tape.append({'type': 'capture', 'label': action['label'] + '-observe-step' + str(index)})
        tape.append({'type': 'capture', 'label': action['label']})
        held = keys
    tape += [{'type': 'key', 'key': key, 'down': False} for key in sorted(held)]
    expect(native['tape'], tape)
    phase = native['bootstrapPresentedFrames']
    issued, captures, phases = [], [], [0, phase]
    interval = native['clock']['stepNs']
    for action in tape:
        if action['type'] == 'step':
            phase += action['frames']
            phases.append(phase)
        elif action['type'] == 'key':
            issued.append((action['key'], action['down'], phase, phase * interval))
        else:
            captures.append((action['label'], phase, phase * interval))
    expect([row['presentedFrame'] for row in native['presentations']], phases)
    expect([(event['key'], event['down'], event['presentedFrame'], event['virtualElapsedNs']) for event in native['inputs']], issued)
    expect([(frame['label'], frame['presentedFrame'], frame['virtualElapsedNs']) for frame in native['frames']], captures)
    assert all(ack['virtualElapsedNs'] == ack['presentedFrame'] * interval for ack in native['presentations'])
    # Every real delivered key must be consumed once on the next public gate.
    # Final grammar intervals are released, so there is no unstepped trailing key.
    consumed = [event for ack in native['presentations'] for event in ack['consumedX11Events']]
    expect([(event['down'], event['keysym'], event['presentedFramePhase']) for event in consumed],
           [(event['down'], event['keysym'], event['presentedFrame'] + 1) for event in native['inputs'] if event['presentedFrame'] < phase])
    expect([event['virtualElapsedNs'] for event in consumed], [event['presentedFramePhase'] * interval for event in consumed])
    assert all(ack['inputOverflow'] is False for ack in native['presentations'])
    for previous, current in zip(native['presentations'], native['presentations'][1:]):
        requested = current.get('requestedPresentationFrames')
        if requested is not None:
            expect(current['presentedFrame'] - previous['presentedFrame'], requested)
    observed_steps = [row for row in native['debugObservations'] if row['context']['label'].startswith('step-')]
    expect(len(observed_steps), sum(action['type'] == 'step' for action in native['tape']))
    assert all(row['observation']['status'] == 'pass' and row['observation']['memoryWritten'] is False
               and row['observation']['executableMappingsRead'] is False for row in native['debugObservations'])
    input_meta = json.loads((native_dir / 'composition-input.json').read_bytes())
    expect(input_meta['nativeDepotChanged'], False)
    expect(input_meta['nativeObservationUsedAsInput'], False)
    expect(input_meta['declaredResourcesBeforeEqualsEnd'], True)
    expect(input_meta['declaredResourceHashes'], input_meta['declaredResourceHashesAfter'])
    expect(input_meta['declaredResourceHashes']['challengesXml'], digest(Path(case['fixture']['authoredResourcePath'])))
    if case['fixture'].get('specialRoomsPath'):
        expect(input_meta['declaredResourceHashes']['specialRooms'], case['fixture']['specialRoomsSha256'])
    for frame in native['frames']:
        assert Path(frame['path']).is_file(), 'Actual original screenshot required'


def browser_readback():
    global browser
    archive = browser_dir / 'browser-evidence.json.gz'
    meta = json.loads((browser_dir / 'browser-evidence-meta.json').read_bytes())
    encoded = archive.read_bytes()
    expect(meta['encoding'], 'gzip')
    expect((len(encoded), hashlib.sha256(encoded).hexdigest()), (meta['archiveBytes'], meta['archiveSha256']))
    raw = gzip.decompress(encoded)  # Includes actual gzip CRC/trailer.
    expect((len(raw), hashlib.sha256(raw).hexdigest()), (meta['rawBytes'], meta['rawSha256']))
    browser = json.loads(raw)
    expect(browser['captureStatus'], 'complete')
    expect(json.loads((browser_dir / 'capture-report.json').read_bytes())['status'], 'pass')
    expect(browser['errors'], [])
    expect(browser.get('traceError'), None)
    trace = browser['trace']
    expect((trace['backendStateTruncated'], trace['backendStateTruncationReason']), (False, None))
    expect(trace['enabled'], True)
    expect(trace['inputTraceTruncated'], False)
    expect(trace['initialSeed'], 1)
    expect(trace['bootEpochSeed'], case['bootEpochSeed'])
    expect(trace['commands'], browser['expectedCommands'])
    # Independently declare the actual DOM bootstrap and densely released game
    # intervals. No captured native seed, stats or actor fields appear here.
    commands = [{'type': 'tick', 'count': 63}]
    key_codes = {'Return': 'Enter', 'Down': 'ArrowDown', 'Up': 'ArrowUp', 'Left': 'ArrowLeft',
                 'Right': 'ArrowRight', 'space': 'Space'}
    for key in ('Return', 'Return', 'Down', 'Return'):
        commands += [{'type': 'key', 'code': key_codes[key], 'down': True}, {'type': 'tick', 'count': 12},
                     {'type': 'key', 'code': key_codes[key], 'down': False}, {'type': 'tick', 'count': 72}]
    commands += [{'type': 'key', 'code': 'Enter', 'down': True}] + [{'type': 'tick', 'count': 1}] * 12
    commands += [{'type': 'key', 'code': 'Enter', 'down': False}] + [{'type': 'tick', 'count': 1}] * 300
    if case['initialPhaseOffset']:
        commands += [{'type': 'tick', 'count': 1}]
    held = set()
    for action in case['actions']:
        keys = set(action['keys'])
        for key in sorted(held - keys):
            commands.append({'type': 'key', 'code': key_codes.get(key, 'Key' + key.upper()), 'down': False})
        for key in sorted(keys - held):
            commands.append({'type': 'key', 'code': key_codes.get(key, 'Key' + key.upper()), 'down': True})
        commands += [{'type': 'tick', 'count': 1}] * action['presentations']
        held = keys
    for key in sorted(held):
        commands.append({'type': 'key', 'code': key_codes.get(key, 'Key' + key.upper()), 'down': False})
    expect(trace['commands'], commands)
    expect([(event['code'], event['down']) for event in browser['actualDomInputs']],
           [(command['code'], command['down']) for command in commands if command['type'] == 'key'])
    expect(len(browser['preludeStates']), 312)
    expect([row['presentation'] for row in browser['preludeStates']], list(range(400, 712)))
    expect([row['presentation'] for row in browser['states']], list(range(711, 712 + case['initialPhaseOffset'] + sum(x['presentations'] for x in case['actions']))))
    assert trace['backendEvents'] and all(event.get('state') for event in trace['backendEvents'])
    for frame in browser['captures']:
        expect(digest(Path(frame['path'])), frame['sha256'])
    resources = json.loads((browser_dir / 'declared-reset-input.json').read_bytes())
    expect(resources['initialSeed'], 1)
    expect(resources['bootEpochSeed'], case['bootEpochSeed'])
    expect(resources['resources']['challengesXml'].encode(), Path(case['fixture']['authoredResourcePath']).read_bytes())
    expect(browser['declaredResourceHashes']['challengesXml'], digest(Path(case['fixture']['authoredResourcePath'])))
    if case['fixture'].get('specialRoomsPath'):
        expect(bytes(resources['resources']['specialRooms']), Path(case['fixture']['specialRoomsPath']).read_bytes())
        expect(browser['declaredResourceHashes']['specialRooms'], case['fixture']['specialRoomsSha256'])
    expect(set(resources), {'initialSeed', 'bootEpochSeed', 'resources'})
    expect(set(resources['resources']), {'challengesXml', 'specialRooms'} if case['fixture'].get('specialRoomsPath') else {'challengesXml'})
    assert browser['declaredFullResetJsonBytes'] <= 512 * 1024


def browser_progress_readback():
    assert browser is not None, 'Final complete browser evidence is mandatory; a prefix cannot pass'
    archive = browser_dir / 'browser-progress.jsonl.gz'
    meta = json.loads(Path(str(archive) + '.meta.json').read_bytes())
    expect((meta['schema'], meta['status'], meta['encoding']), (1, 'closed', 'gzip-members-jsonl'))
    encoded = archive.read_bytes()
    expect((len(encoded), hashlib.sha256(encoded).hexdigest()),
           (meta['committedArchiveBytes'], meta['archiveSha256']))
    raw = gzip.decompress(encoded)  # Validate all committed members and CRCs.
    expect((len(raw), hashlib.sha256(raw).hexdigest()), (meta['rawBytes'], meta['rawSha256']))
    rows = [json.loads(line) for line in raw.splitlines()]
    expect(len(rows), meta['records'])
    expect(rows[0]['kind'], 'start')
    expect(rows[0]['value']['declarationName'], case['name'])
    expect(rows[0]['value']['resetInput'], json.loads((browser_dir / 'declared-reset-input.json').read_bytes()))
    expect(rows[0]['value']['declaredResourceHashes'], browser['declaredResourceHashes'])
    for kind, field in [('preludeState', 'preludeStates'), ('state', 'states'), ('capture', 'captures')]:
        expect([row['value'] for row in rows if row['kind'] == kind], browser[field])
    expect([row['kind'] for row in rows if row['kind'] not in ('preludeState', 'state', 'capture')],
           ['start', 'beforeTrace', 'archiveComplete'])
    expect(rows[-2]['kind'], 'beforeTrace')
    before = rows[-2]['value']
    expect(before['completedTape'], True)
    expect(before['presentation'], browser['states'][-1]['presentation'])
    expect(before['gameplayPresentation'], browser['states'][-1]['gameplayPresentation'])
    expect(before['commands'], browser['expectedCommands'])
    expect(before['inputs'], browser['actualDomInputs'])
    expect(before['errors'], [])
    expect(rows[-1]['kind'], 'archiveComplete')
    expect(rows[-1]['value']['completedTape'], True)
    expect(rows[-1]['value'].get('traceError'), None)
    expect(rows[-1]['value']['evidence'], json.loads((browser_dir / 'browser-evidence-meta.json').read_bytes()))


checked('actual native closed journal/clock/X11/resource readback', native_readback)
checked('actual browser raw gzip/DOM/backend/resource readback', browser_readback)
checked('actual committed browser journal equals final complete evidence', browser_progress_readback)
checked('actual archive files before equals end', lambda: expect({path: digest(Path(path)) for path in pins}, pins))
result = {'schema': 1, 'status': 'pass' if all(check['passed'] for check in checks) else 'fail', 'checks': checks,
          'protectedArchivePins': pins, 'sourceSha256': digest(Path(__file__)),
          'scope': 'Independent actual retained capture validity/input/readback only. Native/browser functional fields require the separate comparator. No synthetic observations or native state inputs.'}
a.output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'checks': checks}))
raise SystemExit(0 if result['status'] == 'pass' else 1)
