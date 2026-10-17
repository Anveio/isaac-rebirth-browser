"""Independent retained-game input/transport/closed-evidence readback."""
import argparse, configparser, gzip, hashlib, importlib.util, json
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--case', type=Path, required=True)
p.add_argument('--directory', type=Path, required=True)
p.add_argument('--native-harness', type=Path, default=Path(__file__).resolve().parents[2])
p.add_argument('--output', type=Path, required=True)
p.add_argument('--wasm-node', action='store_true')
p.add_argument('--wasm-artifact', type=Path)
p.add_argument('--wasm-sha')
p.add_argument('--original-directory', type=Path)
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
common_sku = case.get('commonSku', {}).get('schema') == 'source-public-common-sku-v1'
pocket = 'pocketInit' in case or 'pocketCollection' in case or common_sku
normal_pocket = pocket and case['publicMenuRoute'] == 'seeded-normal'
native_dir, browser_dir = a.original_directory or a.directory / 'original', a.directory / ('wasm' if a.wasm_node else 'browser')
pins = {str(path): digest(path) for directory in (native_dir, browser_dir) if directory.exists()
        for path in directory.rglob('*') if path.is_file() and not path.is_symlink() and '__pycache__' not in path.parts}
native, browser = None, None


def compact_native_readback(reader, manifest, progress):
    """Validate complete canonical bytes; keep only observation validity here.

    The functional comparer independently streams actual bodies from the same
    complete journal. No observation or error is removed from stored evidence.
    Default item-composition still uses its existing complete read_manifest.
    """
    manifest_meta = json.loads(manifest.with_name(manifest.name + '.meta.json').read_text())
    progress_meta = json.loads(progress.with_name(progress.name + '.meta.json').read_text())
    reader.validate_manifest_format(manifest, manifest_meta)
    reader.validate_progress_format(progress, progress_meta)
    expect((manifest_meta['status'], progress_meta['status']), ('closed', 'closed'))
    expect(reader.file_digest(manifest), (manifest_meta['archiveBytes'], manifest_meta['archiveSha256']))
    expect(reader.file_digest(progress), (progress_meta['committedArchiveBytes'], progress_meta['archiveSha256']))

    def journal():
        digest_value = hashlib.sha256()
        byte_count = records = 0
        with reader.open_progress(progress) as stream:
            for raw in stream:
                digest_value.update(raw); byte_count += len(raw); records += 1
                yield json.loads(raw)
        expect((records, byte_count, digest_value.hexdigest()),
               (progress_meta['records'], progress_meta['rawBytes'], progress_meta['rawSha256']))

    class Observations(reader.JournalValues):
        def __init__(self):
            pass

        def __iter__(self):
            for row in journal():
                if row['kind'] == 'debugObservation':
                    yield row['value']

    def validity(observation):
        selected = {key: observation[key] for key in
                    ('status', 'memoryWritten', 'executableMappingsRead', 'error') if key in observation}
        selected['sourceStatuses'] = {key: validity(value) for key, value in observation.items()
                                      if isinstance(value, dict) and 'status' in value}
        return selected

    report, observations, finished = None, [], False
    for index, row in enumerate(journal()):
        assert not finished, 'Native journal follows finish'
        kind, value = row['kind'], row['value']
        if kind == 'start' and report is None and index == 0:
            report = value
        elif report is None:
            raise ValueError('Native journal lacks initial report')
        elif kind == 'acknowledgement':
            report['presentations'].append(value)
        elif kind == 'advanceComplete':
            expect(report['presentations'][-1]['presentedFrame'], value['presentedFrame'])
            report['presentations'][-1] = value
        elif kind == 'field' and value['name'] in ('bootstrapPresentedFrames', 'declaredOptionsKept', 'initialState'):
            report[value['name']] = value['value']
        elif kind == 'debugObservation':
            observations.append(dict(context=value['context'], observation=validity(value['observation'])))
            report.setdefault('debugObservations', [])
        elif kind in ('input', 'capture', 'audioHold', 'focusHandoff'):
            field = {'input': 'inputs', 'capture': 'frames', 'audioHold': 'hostAudioHolds', 'focusHandoff': 'focusHandoffs'}[kind]
            report.setdefault(field, []).append(value)
        elif kind == 'finish':
            report.update(value); finished = True
        else:
            raise ValueError('Unknown Native journal row: ' + str(kind))
    assert finished, 'Native journal lacks finish'
    if 'debugObservations' in report:
        report['debugObservations'] = Observations()
    rebuilt_hash, rebuilt_bytes = hashlib.sha256(), 0
    for text in reader.manifest_chunks(report):
        raw = text.encode('utf8'); rebuilt_hash.update(raw); rebuilt_bytes += len(raw)
    rebuilt_hash.update(b'\n'); rebuilt_bytes += 1
    expect((rebuilt_bytes, rebuilt_hash.hexdigest()), (manifest_meta['rawBytes'], manifest_meta['rawSha256']))
    actual_hash, actual_bytes = hashlib.sha256(), 0
    with reader.open_manifest(manifest) as stream:
        while raw := stream.read(65536):
            actual_hash.update(raw); actual_bytes += len(raw)
    expect((actual_bytes, actual_hash.hexdigest()), (manifest_meta['rawBytes'], manifest_meta['rawSha256']))
    report['debugObservations'] = observations
    return report


def native_readback():
    global native
    manifests = [native_dir / name for name in ('manifest.json.gz', 'manifest.json.zst')
                 if (native_dir / name).is_file()]
    assert len(manifests) == 1, 'Exactly one actual declared Native manifest required'
    manifest = manifests[0]
    journals = [native_dir / name for name in ('native-progress.jsonl.gz', 'native-progress.jsonl.zst')
                if (native_dir / name).is_file()]
    assert len(journals) == 1, 'Exactly one actual declared Native journal required'
    progress = journals[0]
    for path in (manifest, Path(str(manifest) + '.meta.json'), progress,
                 progress.with_name(progress.name + '.meta.json')):
        assert path.is_file(), 'Actual declared138 archive/metadata/journal required: ' + str(path)
    module = a.native_harness / 'tools/original/native-evidence.py'
    spec = importlib.util.spec_from_file_location('composition_native_evidence', module)
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    if manifest.suffix == '.zst':
        assert hasattr(reader, 'validate_manifest_format'), 'Native harness does not own Zstd manifest validation'
        reader.validate_manifest_format(manifest, json.loads(manifest.with_name(manifest.name + '.meta.json').read_text()))
    if progress.suffix == '.zst':
        assert hasattr(reader, 'validate_progress_format'), 'Native harness does not own Zstd journal validation'
        reader.validate_progress_format(progress, json.loads(progress.with_name(progress.name + '.meta.json').read_text()))
    native = (compact_native_readback(reader, manifest, progress) if pocket else reader.read_manifest(manifest))
    # Both paths check exact complete journal/manifest bytes, finish and closed status.
    expect(native['status'], 'pass')
    expect(native['failures'], [])
    expect(native['seedMode'], 'public-manual-entry' if normal_pocket else 'source-challenge-menu')
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
                           ('WindowHeight', 540), ('MusicVolume', 0), ('SFXVolume', case['sfxVolume'] if pocket else 0)]:
        expect(options.getfloat('Options', name), expected)
    # Independent native bootstrap grammar. Dense admission begins at the
    # actual confirm; no gameplay state during those312 steps is discarded.
    tape = [{'type': 'step', 'frames': 180}, {'type': 'capture', 'label': 'intro'}]
    def native_press(key, label, settle):
        tape.append({'type': 'key', 'key': key, 'down': True})
        tape.extend([{'type': 'step', 'frames': 1}] * 12 if label == 'gameplay-start'
                    else [{'type': 'step', 'frames': 12}])
        tape.append({'type': 'key', 'key': key, 'down': False})
        tape.extend([{'type': 'step', 'frames': 1}] * settle if label == 'gameplay-start'
                    else [{'type': 'step', 'frames': settle}])
        if label is not None:
            tape.append({'type': 'capture', 'label': label})
    if normal_pocket:
        for label in ('title', 'profile', 'main', 'character'):
            native_press('Return', label, 72)
        native_press('Tab', None, 108)
        for character in case['publicSeedCode'].lower():
            native_press(character, None, 6)
        tape.append({'type': 'capture', 'label': 'seed-entry'})
        native_press('Return', 'character-seeded', 72)
        native_press('Return', 'gameplay-start', 300)
    else:
        for key, label, settle in [('Return', 'title', 72), ('Return', 'profile', 72), ('Return', 'main', 72),
                                   ('Down', 'challenge-selected', 72), ('Return', 'challenges', 72), ('Return', 'gameplay-start', 300)]:
            native_press(key, label, settle)
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
    if pocket:
        expect(input_meta['declaredSfxVolume'], case['sfxVolume'])
        if common_sku:
            # These are declared input obligations, not captured game state.
            # Legacy captures retain extra source-provenance metadata unchanged.
            fields = ('schema', 'sourceOnlyFixture', 'challengeId', 'acquiredActiveId',
                      'target', 'publicCheckpointAliases')
            actual = input_meta['requiredCommonSkuContract']
            expect({key: actual[key] for key in fields},
                   {key: case['commonSku'][key] for key in fields})
        else:
            expect(input_meta['requiredPocketContract'], case['pocketInit'] if 'pocketInit' in case else case['pocketCollection'])
    if 'pillUse' in case:
        expect(input_meta['requiredPillUseContract'], case['pillUse'])
    expect(input_meta['nativeDepotChanged'], False)
    expect(input_meta['nativeObservationUsedAsInput'], False)
    expect(input_meta['declaredResourcesBeforeEqualsEnd'], True)
    expect(input_meta['declaredResourceHashes'], input_meta['declaredResourceHashesAfter'])
    expect(input_meta['declaredResourceHashes']['challengesXml'], digest(Path(case['fixture']['authoredResourcePath'])))
    if case['fixture'].get('specialRoomsPath'):
        expect(input_meta['declaredResourceHashes']['specialRooms'], digest(Path(case['fixture']['specialRoomsPath'])))
        if 'specialRoomsSha256' in case['fixture']:
            expect(input_meta['declaredResourceHashes']['specialRooms'], case['fixture']['specialRoomsSha256'])
    for frame in native['frames']:
        assert Path(frame['path']).is_file(), 'Actual original screenshot required'


def declared_startup():
    """Choose a version by retained pre-reset declaration/source metadata, never
    by the observed presence or absence of a successful volume command."""
    reset = json.loads((browser_dir / 'declared-reset-input.json').read_bytes())
    record_file = browser_dir / 'declared-startup-inputs.json'
    if not record_file.exists():
        #Historical producer grammar remains explicit and strict. A new
        #manifest with deleted metadata cannot masquerade as this old prefix.
        expect(set(reset), {'initialSeed', 'bootEpochSeed', 'resources'})
        if browser is not None:
            assert 'startupProtocol' not in browser and 'startupRecord' not in browser
        return None
    record = json.loads(record_file.read_bytes())
    expect(set(record), {'schema', 'protocol', 'phase', 'resetInput', 'captureSource'})
    expect((record['schema'], record['protocol'], record['phase']),
           (1, 'declared-preboot-audio-fullscreen-v1', 'before-reset'))
    expect(record['resetInput'], reset)
    expect(set(reset), {'initialSeed', 'bootEpochSeed', 'resources', 'audioPreferences', 'fullscreenPreferences', 'bootElapsed'})
    # verify.mjs resetBootElapsed(DECLARED_NATIVE_INTRO): Title's first render after the declared intro.
    expect(set(reset['bootElapsed']), {'milliseconds', 'subMillisecondNanoseconds'})
    assert all(isinstance(value, int) and value >= 0 for value in reset['bootElapsed'].values())
    expect(reset['initialSeed'], 1)
    expect(reset['bootEpochSeed'], case['bootEpochSeed'])
    expected_audio = {'sfxVolume': case['sfxVolume'] if pocket else 0, 'musicVolume': 0}
    expect(reset['audioPreferences'], expected_audio)
    expect(reset['fullscreenPreferences'], {'fullscreen': False})
    expect(set(record['captureSource']), {'path', 'sha256', 'archive'})
    producer = 'composition-wasm-public' if a.wasm_node else 'composition-capture'
    expect(Path(record['captureSource']['path']).name, producer + '.mjs')
    expect(record['captureSource']['archive'], producer + '-source.mjs')
    capture_source = browser_dir / record['captureSource']['archive']
    assert capture_source.is_file() and not capture_source.is_symlink()
    #The actual producer retained these executing bytes before reset. Do not
    #require its mutable canonical execution path to remain unchanged forever.
    expect(digest(capture_source), record['captureSource']['sha256'])
    #Require the actual independently launched Native profile's options, not
    #only the declaration that requested them. No Native output enters runtime.
    actual_options = configparser.ConfigParser()
    assert actual_options.read(native_dir / 'profile/data/binding of isaac rebirth/options.ini')
    for name, value in [('SFXVolume', expected_audio['sfxVolume']), ('MusicVolume', 0), ('Fullscreen', 0)]:
        expect(actual_options.getfloat('Options', name), value)
    if browser is not None:
        expect(browser['startupProtocol'], record['protocol'])
        expect(browser['startupRecord'], record)
        expect(browser['trace']['bootAudioPreferences'], reset['audioPreferences'])
        expect(browser['trace']['bootFullscreenPreferences'], reset['fullscreenPreferences'])
    return record


def declared_port_commands(preboot=False):
    commands = ([{'type': 'audioVolume', 'sfx_volume': case['sfxVolume']}] if pocket and not preboot else [])
    commands.append({'type': 'tick', 'count': 63})
    key_codes = {'Return': 'Enter', 'Tab': 'Tab', 'Down': 'ArrowDown', 'Up': 'ArrowUp', 'Left': 'ArrowLeft',
                 'Right': 'ArrowRight', 'space': 'Space'}
    dom_keys = []
    def browser_press(key, settle=72):
        code = key_codes[key]
        commands.extend([{'type': 'key', 'code': code, 'down': True}, {'type': 'tick', 'count': 12},
                         {'type': 'key', 'code': code, 'down': False}, {'type': 'tick', 'count': settle}])
        dom_keys.extend([(code, True), (code, False)])
    browser_press('Return'); browser_press('Return')
    if normal_pocket:
        browser_press('Return'); browser_press('Tab', 108)
        for character in case['publicSeedCode']:
            code = ('Digit' if character.isdigit() else 'Key') + character
            commands.extend([{'type': 'text', 'text': character}, {'type': 'tick', 'count': 12}, {'type': 'tick', 'count': 6}])
            dom_keys.extend([(code, True), (code, False)])
        browser_press('Return')
    else:
        browser_press('Down'); browser_press('Return')
    commands += [{'type': 'key', 'code': 'Enter', 'down': True}] + [{'type': 'tick', 'count': 1}] * 12
    commands += [{'type': 'key', 'code': 'Enter', 'down': False}] + [{'type': 'tick', 'count': 1}] * 300
    dom_keys.extend([('Enter', True), ('Enter', False)])
    gameplay_start = sum(command['count'] for command in commands if command['type'] == 'tick')
    if case['initialPhaseOffset']:
        commands += [{'type': 'tick', 'count': 1}]
    held = set()
    for action in case['actions']:
        keys = set(action['keys'])
        for key in sorted(held - keys):
            code = key_codes.get(key, 'Key' + key.upper())
            commands.append({'type': 'key', 'code': code, 'down': False})
            dom_keys.append((code, False))
        for key in sorted(keys - held):
            code = key_codes.get(key, 'Key' + key.upper())
            commands.append({'type': 'key', 'code': code, 'down': True})
            dom_keys.append((code, True))
        commands += [{'type': 'tick', 'count': 1}] * action['presentations']
        held = keys
    for key in sorted(held):
        code = key_codes.get(key, 'Key' + key.upper())
        commands.append({'type': 'key', 'code': code, 'down': False})
        dom_keys.append((code, False))
    return commands, dom_keys, gameplay_start


def wasm_node_readback():
    """The existing public ABI executor's complete raw records, without DOM claims."""
    assert pocket, 'Public-WASM fallback is currently scoped to pocket cases'
    assert a.wasm_artifact and a.wasm_sha
    result_path = browser_dir / 'actual-replay-result.json'
    result = json.loads(result_path.read_bytes())
    startup_record = declared_startup()
    expect(result['captureBackend'], 'wasm-node')
    expect(result['status'], 'CAPTURE_COMPLETE')
    expect(result['completedTape'], True)
    expect(result.get('failure'), None)
    expect(result['before'], result['after'])
    expect(result['before']['wasm'], dict(path=str(a.wasm_artifact.resolve()), sha256=a.wasm_sha))
    expect(result['before']['case'], dict(path=str(a.case.resolve()), sha256=digest(a.case)))
    if startup_record is not None:
        expect(result['before']['donor'], {key: startup_record['captureSource'][key] for key in ('path', 'sha256')})
    for name, record in result['before'].items():
        #New captures retain the exact executing producer before reset. Later
        #legitimate edits to its canonical path cannot invalidate those bytes.
        if name == 'donor' and startup_record is not None:
            continue
        expect(digest(Path(record['path'])), record['sha256'])
    expect(digest(a.wasm_artifact), a.wasm_sha)
    reset = json.loads((browser_dir / 'declared-reset-input.json').read_bytes())
    expect(set(reset), {'initialSeed', 'bootEpochSeed', 'resources', 'audioPreferences', 'fullscreenPreferences', 'bootElapsed'}
           if startup_record is not None else {'initialSeed', 'bootEpochSeed', 'resources'})
    if startup_record is None:
        assert 'startupProtocol' not in result and 'startupRecord' not in result
    else:
        expect(result['startupProtocol'], startup_record['protocol'])
        expect(result['startupRecord'], startup_record)
    expect(reset['initialSeed'], 1)
    expect(reset['bootEpochSeed'], case['bootEpochSeed'])
    resources = reset['resources']
    expect(resources['challengesXml'].encode(), Path(case['fixture']['authoredResourcePath']).read_bytes())
    expect(bytes(resources['specialRooms']), Path(case['fixture']['specialRoomsPath']).read_bytes())
    expect(set(resources), {'challengesXml', 'specialRooms'})
    expected_commands, _dom_keys, gameplay_start = declared_port_commands(preboot=startup_record is not None)
    expect(result['commands'], expected_commands)
    final_gp = case['initialPhaseOffset'] + sum(action['presentations'] for action in case['actions'])
    expect(result['gameplayPresentation'], final_gp)
    expect(result['presentation'], gameplay_start + final_gp)
    archive = browser_dir / 'actual-states.jsonl.gz'
    meta = json.loads(Path(str(archive) + '.meta.json').read_bytes())
    expect((meta['schema'], meta['status'], meta['encoding']), (1, 'closed', 'gzip-jsonl'))
    expect((archive.stat().st_size, digest(archive)), (meta['committedArchiveBytes'], meta['archiveSha256']))
    raw_hash, raw_bytes, count = hashlib.sha256(), 0, 0
    prelude, states, start, tail, last = [], [], None, None, None
    with gzip.open(archive, 'rb') as stream:
        for raw in stream:
            raw_hash.update(raw); raw_bytes += len(raw); count += 1
            row = json.loads(raw)
            if row['kind'] == 'start':
                assert start is None and count == 1
                start = row
            elif row['kind'] == 'prelude':
                assert start is not None and not states and tail is None
                prelude.append(row['presentation'])
                assert isinstance(row['state'], dict)
            elif row['kind'] == 'state':
                assert start is not None and len(prelude) == 312 and tail is None
                states.append((row['presentation'], row['gameplayPresentation']))
                assert isinstance(row['state'], dict)
            elif row['kind'] == 'beforeAssertions':
                assert start is not None and states and tail is None
                tail = row
            else:
                raise AssertionError('Undeclared public-WASM journal record: ' + row['kind'])
            last = row['kind']
    expect((count, raw_bytes, raw_hash.hexdigest()), (meta['records'], meta['rawBytes'], meta['rawSha256']))
    expect(last, 'beforeAssertions')
    expect(start['before'], result['before'])
    expect(start['caseName'], case['name'])
    if startup_record is None:
        assert 'startupRecord' not in start
    else:
        expect(start['startupRecord'], startup_record)
    expect((start['initialSeed'], start['bootEpochSeed']), (1, case['bootEpochSeed']))
    expect(prelude, list(range(gameplay_start - 311, gameplay_start + 1)))
    expect(states, [(gameplay_start + gp, gp) for gp in range(final_gp + 1)])
    expect(result['observations'], len(prelude) + len(states))
    for field in ['completedTape', 'presentation', 'gameplayPresentation', 'observations', 'commands', 'checkpoints', 'failure']:
        expect(tail.get(field), result.get(field))
    checkpoints = result['checkpoints']
    expected = ([('title', 'title'), ('profile', 'profile'), ('main', 'main'),
                 ('character', 'character'), ('seed-entry', 'seed'), ('character-seeded', 'character'),
                 ('gameplay-start', 'playing')] if normal_pocket else
                [('title', 'title'), ('profile', 'profile'), ('main', 'main'),
                 ('challenge-selected', 'main'), ('challenges', 'challenge'), ('gameplay-start', 'playing')])
    expect([(row['label'], row['screen']) for row in checkpoints[:len(expected)]], expected)
    expect([row['label'] for row in checkpoints[len(expected):]], [action['label'] for action in case['actions']])


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
    startup_record = declared_startup()
    commands, dom_keys, gameplay_start = declared_port_commands(preboot=startup_record is not None)
    expect(trace['commands'], commands)
    expect([(event['code'], event['down']) for event in browser['actualDomInputs']],
           dom_keys)
    expect(len(browser['preludeStates']), 312)
    expect([row['presentation'] for row in browser['preludeStates']], list(range(gameplay_start - 311, gameplay_start + 1)))
    expect([row['presentation'] for row in browser['states']], list(range(gameplay_start, gameplay_start + 1 + case['initialPhaseOffset'] + sum(x['presentations'] for x in case['actions']))))
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
        expect(browser['declaredResourceHashes']['specialRooms'], digest(Path(case['fixture']['specialRoomsPath'])))
        if 'specialRoomsSha256' in case['fixture']:
            expect(browser['declaredResourceHashes']['specialRooms'], case['fixture']['specialRoomsSha256'])
    expect(set(resources), {'initialSeed', 'bootEpochSeed', 'resources', 'audioPreferences', 'fullscreenPreferences', 'bootElapsed'}
           if startup_record is not None else {'initialSeed', 'bootEpochSeed', 'resources'})
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
    startup_record = declared_startup()
    if startup_record is None:
        assert 'startupRecord' not in rows[0]['value']
    else:
        expect(rows[0]['value']['startupRecord'], startup_record)
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
if a.wasm_node:
    checked('actual complete public-WASM ABI/resource/journal readback', wasm_node_readback)
else:
    checked('actual browser raw gzip/DOM/backend/resource readback', browser_readback)
    checked('actual committed browser journal equals final complete evidence', browser_progress_readback)
checked('actual archive files before equals end', lambda: expect({path: digest(Path(path)) for path in pins}, pins))
result = {'schema': 1, 'status': 'pass' if all(check['passed'] for check in checks) else 'fail', 'checks': checks,
          'protectedArchivePins': pins, 'sourceSha256': digest(Path(__file__)),
          'scope': 'Independent actual retained capture validity/input/readback only. Native/browser functional fields require the separate comparator. No synthetic observations or native state inputs.'}
if a.wasm_node:
    result['captureBackend'] = 'wasm-node'
    result['unavailable'] = ['DOM inputs', 'Browser renderer and PNGs', 'Audio backend and PCM']
    result['scope'] = 'Complete independent Original and public-WASM ABI input/evidence readback. Actual functional fields require the unchanged pocket comparer; DOM/backend are not qualified.'
if 'pillUse' in case:
    result['scope'] = 'Complete independent Original and public-input capture readback. Positive Pill functional fields require the separate real-use reader; private order/render/backend remain unavailable.'
a.output.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({'status': result['status'], 'checks': checks}))
raise SystemExit(0 if result['status'] == 'pass' else 1)
