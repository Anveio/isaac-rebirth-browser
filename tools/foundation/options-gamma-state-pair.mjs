// Bounded Gamma owners on the existing Options public-clock transport.
import assert from 'node:assert/strict';

const equal = (expected, actual) => expected !== undefined && actual !== undefined &&
  JSON.stringify(expected) === JSON.stringify(actual);
const word = value => {
  assert.ok(Number.isFinite(value), 'Finite f32 owner required');
  const bytes = Buffer.alloc(4); bytes.writeFloatLE(value); return bytes.readUInt32LE();
};

export function validateGammaDeclaration(data, contract) {
  assert.equal(contract.schema, 1);
  assert.equal(contract.gamma, true); assert.equal(contract.naturalReload, true);
  assert.equal(contract.preferenceSchema, 4); assert.equal(contract.storageKey, 'isaac-rebirth.options.v4');
  assert.equal(data.declarationMode, 'gamma-main-options-source-v1');
  assert.equal(contract.caseDeclarationMode, data.declarationMode);
  assert.deepEqual(data.tape, [...data.bootstrap.sourceOnlyPrelude, ...data.pairedTape]);
  for (const [name, tape, count] of [['nativeLabels', data.tape, 21], ['pairedLabels', data.pairedTape, 20]]) {
    const labels = tape.filter(action => action.type === 'capture').map(action => action.label);
    assert.equal(labels.length, count); assert.equal(new Set(labels).size, count);
    assert.deepEqual(contract[name], labels); assert.deepEqual(data.captureDeclaration[name], labels);
  }
  assert.deepEqual(contract.sourceOnlyLabels, ['intro']);
  const groups = [contract.menuLabels, contract.playingLabels, contract.provenanceOnlyLabels];
  assert.equal(new Set(groups.flat()).size, 21);
  assert.deepEqual(groups.flat().sort(), [...contract.nativeLabels].sort());
  assert.deepEqual(contract.playingLabels, ['gameplay-start']);
  assert.deepEqual(data.declaredAudioOptions, {sfxVolume: 1, musicVolume: 1});
  assert.equal(data.declaredFullscreen, false);
  assert.deepEqual(Object.keys(contract.rectangles), ['gammaPercentage']);
  assert.equal(contract.rectangles.gammaPercentage.required, true);
  assert.deepEqual(contract.rectangles.gammaPercentage.region, {x: 544, y: 268, width: 112, height: 48});
}

function gammaInputLaw(data) {
  const keys = new Set(), captures = new Map();
  let active = false, gamma = Math.fround(1), left = 0, right = 0, presentation = 0;
  for (const action of data.pairedTape) {
    if (action.type === 'key') {
      if (action.down) keys.add(action.key); else keys.delete(action.key);
    } else if (action.type === 'step') {
      for (let step = 0; step < action.frames; step++) {
        presentation++;
        if (!active) continue;
        const l = keys.has('Left'), r = keys.has('Right'), prior = l ? left : right;
        if ((l || r) && (prior === 0 || prior > 49 && prior % 2 === 0))
          gamma = l ? Math.max(0.5, Math.fround(gamma - Math.fround(0.01)))
            : Math.min(1.5, Math.fround(gamma + Math.fround(0.01)));
        left = l ? left + 1 : 0; right = r ? right + 1 : 0;
      }
    } else {
      if (action.label === 'options-gamma-row-6') active = true;
      captures.set(action.label, {gamma, left, right, presentation});
      if (action.label === 'options-gamma-back-saved') active = false;
    }
  }
  return captures;
}

export function compareOptionsGamma({data, contract, clock, native, browser, observed, calls, scopedState}) {
  const checks = [], diagnostics = [];
  const check = (field, expected, actual) => checks.push({field, expected, actual,
    status: equal(expected, actual) ? 'pass' : 'fail'});
  const f32 = (field, expected, actual) => check(field + '.word', word(expected),
    Number.isFinite(actual) ? word(actual) : undefined);
  const attempt = (field, read) => {
    try { read(); } catch (error) { check(field + '.owner', 'present', String(error.stack ?? error)); }
  };
  // Declaration failures are blocking; no partial schema is silently admitted.
  try { validateGammaDeclaration(data, contract); }
  catch (error) { check('declaration', 'valid Gamma inputs', String(error.stack ?? error)); return {status: 'fail', checks, diagnostics}; }
  check('Native.capture', 'pass', native.status);
  check('Native.provenance', 'pass', native.provenance?.status);
  check('Native.clock', 'pass', native.publicClockValidation?.status);
  check('Browser.capture', 'pass', browser.status);
  check('Browser.errors', [], observed.errors); check('Browser.failures', [], observed.failures);
  check('Native.labels', contract.nativeLabels, native.manifest?.frames?.map(frame => frame.label));
  check('Browser.labels', contract.pairedLabels, browser.frames?.map(frame => frame.label));
  const ns = new Map((native.manifest?.debugObservations ?? []).map(row => [row.context.label, row.observation]));
  const nf = new Map((native.manifest?.frames ?? []).map(row => [row.label, row]));
  const bf = new Map((browser.frames ?? []).map(row => [row.label, row]));
  const law = gammaInputLaw(data);
  for (const label of contract.pairedLabels) {
    const source = ns.get(label), frame = bf.get(label), state = frame?.state;
    const expected = law.get(label), marker = observed.checkpoints?.find(row => row.label === label);
    attempt(label + '.shared', () => {
      assert.equal(source?.standard?.status, 'pass'); assert.ok(state && nf.get(label));
      check(label + '.sourceAggregate', 'pass', source.status);
      check(label + '.readOnly', false, source.memoryWritten);
      check(label + '.presentation', nf.get(label).presentedFrame - nf.get('title-ready').presentedFrame,
        frame.browserPresentation);
      if (!contract.provenanceOnlyLabels.includes(label)) {
        const counter = source.options?.systemCounter ?? source.standard.gameplay?.nativePhase?.systemIterations;
        check(label + '.relativeSystemCounter', Number.isInteger(counter) ? counter - ns.get('title-ready').options.systemCounter : undefined,
          state.systemClock.systemCounter - bf.get('title-ready').state.systemClock.systemCounter);
      }
      if (contract.provenanceOnlyLabels.includes(label) || label === 'gameplay-start') {
        for (const row of scopedState(source.standard, state, label === 'gameplay-start'))
          checks.push({...row, field: label + '.standard.' + row.field});
      } else {
        const menu = source.options;
        assert.equal(menu?.status, 'pass');
        check(label + '.screen', ({1: 'title', 2: 'profile', 3: 'main', 4: 'character', 8: 'options'})[menu.menu.screen], state.screen);
        for (const name of ['selection', 'leftHeld', 'rightHeld', 'verticalHeld'])
          check(label + '.options.' + name, menu.options[name], state.optionsMenu[name]);
        check(label + '.menuMoving', menu.menu.moving, state.menuPresentation?.moving);
        for (let axis = 0; axis < 2; axis++)
          check(label + '.menuView.' + axis + '.word', menu.menu.rawView?.[axis]?.bits,
            Number.isFinite(state.menuPresentation?.view?.[axis]) ? word(state.menuPresentation.view[axis]) : undefined);
        check(label + '.SFX.word', menu.settings.sfxVolume.bits, word(state.audio.sfxVolume));
        check(label + '.Music.word', menu.settings.rawMusicVolume.bits, word(state.music.volume));
        check(label + '.Fullscreen', menu.settings.fullscreenRequested, state.fullscreen.requested);
      }
    });
    if (contract.provenanceOnlyLabels.includes(label)) continue;
    attempt(label + '.NativeGamma', () => {
      const owner = source.gamma;
      assert.equal(owner?.status, 'pass');
      check(label + '.Native.enabled', true, owner.enabled);
      for (const [name, value] of [['exposure', 1], ['brightness', 0], ['contrast', 1]])
        check(label + '.Native.' + name + '.word', word(value), owner[name]?.bits);
      check(label + '.Native.Gamma.word', word(expected.gamma), owner.value?.bits);
      if (label.startsWith('gamma-') || label === 'options-gamma-row-6') {
        const percent = Math.floor(Math.fround(Math.fround(expected.gamma * 100) + 0.5));
        check(label + '.Native.lastCounterDigit', Number(String(percent)[0]), owner.lastCounterDigit);
      }
    });
    attempt(label + '.BrowserGamma', () => {
      assert.ok(state && marker);
      f32(label + '.Browser.Gamma', expected.gamma, state.gammaSettings?.gamma);
      const record = marker.graphics?.records?.at(-1);
      check(label + '.GPU.complete', false, marker.graphics?.truncated);
      check(label + '.GPU.error', null, marker.graphics?.error);
      check(label + '.GPU.actualFinalTarget', 0, record?.framebuffer);
      check(label + '.GPU.actualSourceTarget', true, record?.sourceTarget);
      check(label + '.GPU.linked', true, record?.linked);
      check(label + '.GPU.actualComposedSize', [960, 540], record?.dimensions);
      check(label + '.GPU.actualViewport', [0, 0, 960, 540], record?.viewport);
      check(label + '.GPU.draw', [4, 0, 6], record && [record.mode, record.first, record.count]);
      check(label + '.GPU.nonidentityProgram', expected.gamma !== 1, record?.gammaProgram);
      if (expected.gamma !== 1) f32(label + '.GPU.latchedGamma', expected.gamma, record?.gamma);
    });
    attempt(label + '.V4IO', () => {
      assert.ok(marker);
      check('V4.' + label + '.runtimeKey', contract.storageKey, marker.preferences?.key);
      check('V4.' + label + '.storageKey', contract.storageKey, marker.storage?.key);
      check('V4.' + label + '.IOComplete', false, marker.preferences?.truncated);
      check('V4.' + label + '.IOErrors', [], marker.preferences?.events?.filter(event => event.status === 'fail'));
      check(label + '.feedbackComplete', false, marker.feedback?.truncated);
      check(label + '.feedbackErrors', [], marker.feedback?.events?.filter(event => event.status === 'fail'));
    });
  }
  attempt('stepTransport', () => {
    check('steps.complete', false, observed.returns?.truncated);
    check('steps.error', null, observed.returns?.error);
    check('steps.timestamps', clock.browserTickCommands.map(command => command.timestampMillis),
      observed.returns?.rows?.map(row => row.timestampMillis));
    check('steps.count', clock.browserTickCommands.length, observed.returns?.rows?.length);
  });
  attempt('Gamma.ownSharedMT', () => {
    const start = bf.get('options-gamma-row-6').state.bootLifecycle.shared;
    const end = bf.get('gamma-right-held-255').state.bootLifecycle.shared;
    assert.ok(Array.isArray(start.words) && start.words.length === 624 && Array.isArray(end.words) && end.words.length === 624);
    check('Gamma.Browser.sharedMTUnchangedBeforeBack', start, end);
    diagnostics.push({field: 'Gamma.Native.sharedMT', status: 'Source callback contains no RNG; live cursor/draw stream unobserved',
      scope: 'The Browser own-state equality is not complete Original shared-MT qualification'});
  });
  attempt('Gamma.no237', () => {
    assert.equal(calls[0]?.api, 'trace-start'); assert.ok(calls.length > 1);
    const lo = nf.get('options-gamma-row-6').virtualElapsedNs;
    const hi = nf.get('gamma-right-held-255').virtualElapsedNs;
    check('Gamma.Native.actual237Starts', 0, calls.filter(call => call.clockDomain === 'gate-main-thread' &&
      call.virtualElapsedNs > lo && call.virtualElapsedNs <= hi && call.api === 'alSourcePlay' &&
      call.sfxOwner?.references?.some(reference => reference.id === 237)).length);
    const start = law.get('options-gamma-row-6').presentation, end = law.get('gamma-right-held-255').presentation;
    assert.ok(Array.isArray(observed.returns?.rows));
    check('Gamma.Browser.actual237Admissions', 0, observed.returns.rows.filter(row => row.presentation > start && row.presentation <= end)
      .flatMap(row => row.audioEvents.filter(event => event.id === 237)).length);
  });
  attempt('V4.Back', () => {
    const back = ns.get('options-gamma-back-saved').options.file;
    const matches = [...back.text.matchAll(/^Gamma=([^\r\n]+)$/gm)];
    assert.equal(matches.length, 1);
    check('V4.Back.NativeGamma', '1.5000', matches[0][1]);
    const marker = observed.checkpoints.find(row => row.label === 'options-gamma-back-saved');
    const saved = {schema: 4, sfxVolume: '1.0000', musicVolume: '1.0000', fullscreen: false, mapOpacity: '0.3000', gamma: '1.5000'};
    check('V4.Back.actualStorage', saved, JSON.parse(marker.storage.text));
    const events = marker.preferences.events.filter(event => event.operation === 'save' && event.status === 'pass');
    check('V4.Back.actualSaveCount', 1, events.length);
    check('V4.Back.actualSaveKey', contract.storageKey, events[0]?.key);
    check('V4.Back.actualSaveValue', saved, events[0]?.value);
    check('V4.reload.ready', 'true', observed.reload?.ready);
    check('V4.reload.actualLoadValue', saved, JSON.parse(observed.reload?.storage));
    f32('V4.reload.loadedGamma', 1.5, observed.reload?.state?.gammaSettings?.gamma);
    check('V4.reload.bootGamma', {gamma: 1.5}, observed.reload?.trace?.bootGammaPreferences);
    check('V4.reload.actualLoadObserved', true, observed.reload?.preferences?.events?.some(event =>
      event.operation === 'load' && event.status === 'pass' && event.key === contract.storageKey && equal(saved, event.value)));
    const draw = observed.reload?.graphics?.records?.at(-1);
    check('V4.reload.GPUProgram', true, draw?.gammaProgram);
    f32('V4.reload.GPUGamma', 1.5, draw?.gamma);
  });
  return {schema: 1, status: checks.length && checks.every(row => row.status === 'pass') ? 'pass' : 'fail', checks, diagnostics,
    scope: 'Bounded default-config Gamma callback, percentage, actual final GPU, V4 Back and separate Browser reload. No HQ4X, Popups, complete Native MT, wholeRGBA or Native process-reload claim.'};
}
