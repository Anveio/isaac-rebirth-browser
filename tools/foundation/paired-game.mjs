import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, statfsSync, symlinkSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { chromium } from '@playwright/test';
import { PNG } from 'pngjs';
import { browserLaunchOptions, inspectBrowserGraphics } from './browser.mjs';
import { manifestPath as browserManifestPath, readJsonEvidence, writeJsonEvidence } from './browser-evidence.mjs';
import { compareMtDraws } from './mt-draws.mjs';
import { readNativeEvidence } from './native-evidence.mjs';
import { pixelDifference } from './compare.mjs';
import { comparePixelPairs } from './pixel-pairs.mjs';
import { flag, selectCases } from '../e2e/lib/cli.mjs';
import { startProductionServer } from './production-server.mjs';
import { audioObservedBrowser } from '../e2e/lib/audio-owners.mjs';
import { evaluateJson, readTrace } from '../e2e/lib/game.mjs';
import {readBrowserControls,installPhysicalObserver} from './controls-browser-observe.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const SOURCE = 'tools/original/foundation-paired-scenarios.py';
export const DEFAULT_PAIRED_SCENARIOS = ['paired-title-menu', 'paired-movement-fire', 'paired-pause-release', 'paired-bomb-moving-even', 'paired-bomb-moving-odd', 'paired-room-combat', 'paired-pause-before-transition', 'paired-pause-transition', 'paired-wall-hugger-contact'];
export const DEFAULT_RANDOM_CASE_COUNT = 2;
const runProcess = promisify(execFile);
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const option = (name, fallback) => {
  const at = process.argv.indexOf(name);
  if (at < 0) return fallback;
  assert.ok(process.argv[at + 1] && !process.argv[at + 1].startsWith('--'), `Missing ${name} value`);
  return process.argv[at + 1];
};
const countOption = (name, fallback, minimum = 1) => {
  const value = Number(option(name, fallback));
  assert.ok(Number.isInteger(value) && value >= minimum && value <= 32, `${name} must be ${minimum}..32`);
  return value;
};
async function python(args) {
  const { stdout } = await runProcess('python3', [SOURCE, ...args], { cwd: ROOT, maxBuffer: 8 * 1024 * 1024 });
  return JSON.parse(stdout);
}

export async function pairedScenarioDeclarations() {
  return python(['--list']);
}

// Public Original tapes use real X11 keysym names. Translate their physical
// positions to the engine-owned KeyboardEvent.code catalog, never logical aliases.
const namedKeys = {
  space: 'Space', apostrophe: 'Quote', comma: 'Comma', minus: 'Minus', period: 'Period', slash: 'Slash',
  semicolon: 'Semicolon', equal: 'Equal', bracketleft: 'BracketLeft', backslash: 'Backslash',
  bracketright: 'BracketRight', grave: 'Backquote',
  Escape: 'Escape', Return: 'Enter', Tab: 'Tab', BackSpace: 'Backspace', Insert: 'Insert', Delete: 'Delete',
  Right: 'ArrowRight', Left: 'ArrowLeft', Down: 'ArrowDown', Up: 'ArrowUp',
  Prior: 'PageUp', Next: 'PageDown', Home: 'Home', End: 'End',
  Caps_Lock: 'CapsLock', Scroll_Lock: 'ScrollLock', Num_Lock: 'NumLock', Print: 'PrintScreen', Pause: 'Pause',
  KP_Decimal: 'NumpadDecimal', KP_Divide: 'NumpadDivide', KP_Multiply: 'NumpadMultiply',
  KP_Subtract: 'NumpadSubtract', KP_Add: 'NumpadAdd', KP_Enter: 'NumpadEnter', KP_Equal: 'NumpadEqual',
  Shift_L: 'ShiftLeft', Control_L: 'ControlLeft', Alt_L: 'AltLeft', Super_L: 'MetaLeft',
  Shift_R: 'ShiftRight', Control_R: 'ControlRight', Alt_R: 'AltRight', Super_R: 'MetaRight', Menu: 'ContextMenu',
};
function keyCode(key) {
  if (Object.hasOwn(namedKeys, key)) return namedKeys[key];
  if (/^[a-z]$/i.test(key)) return 'Key' + key.toUpperCase();
  if (/^[0-9]$/.test(key)) return 'Digit' + key;
  if (/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(key)) return key;
  if (/^KP_[0-9]$/.test(key)) return 'Numpad' + key.slice(3);
  throw new Error(`No public physical browser key transport for X11 keysym ${key}`);
}
class UnsupportedBrowserKeyTransport extends Error {
  constructor(code) {
    super(`Installed Playwright keyboard transport does not expose ${code}`);
    this.name = 'UnsupportedBrowserKeyTransport';
    this.physicalCode = code;
  }
}
function browserKeyCode(key) {
  const code = keyCode(key);
  // This installed Playwright layout ends at F12 and omits NumpadEqual. Keep
  // canonical engine ingress, but do not replace missing DOM events with aliases.
  if (/^F(?:1[3-9]|2[0-4])$/.test(code) || code === 'NumpadEqual')
    throw new UnsupportedBrowserKeyTransport(code);
  return code;
}
function modifierName(code) {
  return /^(Shift|Control|Alt|Meta)(Left|Right)$/.exec(code)?.[1];
}
function seedInputText(code, modifiers) {
  if (modifiers.has('Control') || modifiers.has('Alt') || modifiers.has('Meta')) return null;
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code) && !modifiers.has('Shift')) return code.slice(5);
  if (/^Numpad[0-9]$/.test(code) && modifiers.has('Shift')) return code.slice(6);
  return null;
}

// Where a case's games start playing their own keys (its first
// pairedIndependent scope), as the port's game update count at that capture.
function independentDraws(caseData, directory) {
  const after = caseData.pairedIndependent?.[0]?.after;
  if (!after) return undefined;
  const frame = readJsonEvidence(browserManifestPath(directory, 'browser')).frames.find(item => item.label === after);
  assert.ok(Number.isInteger(frame?.state?.gameUpdateCount), `pairedIndependent capture ${after} has no game update count`);
  return { label: after, gameUpdateCount: frame.state.gameUpdateCount };
}

// Opt-in paired clocks use only the declared source-only prelude and the
// native launcher defaults, then require the actual clock acknowledgements.
export function declarePublicClockPlan(data) {
  const nativeBootstrapPresentations = 30; // foundation-clock-run.py step30
  const presentationIntervalNanoseconds = 16_667_000; // launcher's --step-ns default
  assert.ok(Number.isInteger(data.bootEpochSeed) && data.bootEpochSeed >= 0 && data.bootEpochSeed <= 0xffffffff);
  const prelude = data.bootstrap?.sourceOnlyPrelude;
  assert.ok(Array.isArray(prelude) && Array.isArray(data.pairedTape) && Array.isArray(data.tape));
  assert.deepEqual(data.tape, [...prelude, ...data.pairedTape], 'Full public tape must exactly retain its declared source-only prelude and paired suffix');
  assert.deepEqual(data.pairedTape[0], {type: 'capture', label: data.bootstrap.boundaryLabel});
  let total = 0;
  for (const action of data.tape) {
    assert.ok(['step', 'key', 'capture'].includes(action.type));
    if (action.type === 'step') {
      assert.ok(Number.isSafeInteger(action.frames) && action.frames > 0 && action.frames <= 36000);
      total += action.frames;
    }
  }
  assert.ok(total <= 36000);
  const nativeEntryPresentation = nativeBootstrapPresentations + prelude.reduce((sum, action) => sum + (action.type === 'step' ? action.frames : 0), 0);
  const nativeAcknowledgements = [0, nativeBootstrapPresentations].map(presentedFrame => ({presentedFrame, virtualElapsedNs: presentedFrame * presentationIntervalNanoseconds}));
  const nativeCaptures = [];
  let frame = nativeBootstrapPresentations;
  for (const action of data.tape) {
    if (action.type === 'step') {
      frame += action.frames;
      nativeAcknowledgements.push({presentedFrame: frame, virtualElapsedNs: frame * presentationIntervalNanoseconds});
    } else if (action.type === 'capture') nativeCaptures.push({label: action.label, presentedFrame: frame, virtualElapsedNs: frame * presentationIntervalNanoseconds});
  }
  const browserTickCommands = [];
  frame = nativeEntryPresentation;
  for (const action of data.pairedTape) if (action.type === 'step') {
    for (let index = 0; index < action.frames; index++) {
      frame++;
      const timestampMillis = data.bootEpochSeed * 1000 + Math.floor(frame * presentationIntervalNanoseconds / 1e6);
      assert.ok(Number.isSafeInteger(timestampMillis) && timestampMillis >= 0);
      browserTickCommands.push({type: 'tick', count: 1, timestampMillis});
    }
  }
  return {schema: 1, scope: 'Independent public clock declaration before either capture; no observed Native clock or counter is an input',
    bootEpochSeed: data.bootEpochSeed, nativeBootstrapPresentations, presentationIntervalNanoseconds,
    nativeEntryPresentation, sourceOnlyPrelude: prelude, pairedTape: data.pairedTape,
    nativeAcknowledgements, nativeCaptures, browserTickCommands};
}

export function validateNativeClockPlan(data, plan, native) {
  assert.equal(native.status, 'pass');
  assert.equal(native.provenance?.status, 'pass', 'Existing canonical Native provenance remains required');
  const manifest = native.manifest;
  assert.deepEqual(manifest.tape, data.tape);
  assert.equal(manifest.bootEpochSeed, plan.bootEpochSeed);
  assert.equal(manifest.bootstrapPresentedFrames, plan.nativeBootstrapPresentations);
  assert.equal(manifest.clock.stepNs, plan.presentationIntervalNanoseconds);
  assert.deepEqual(manifest.presentations.map(({presentedFrame, virtualElapsedNs}) => ({presentedFrame, virtualElapsedNs})), plan.nativeAcknowledgements);
  assert.deepEqual(manifest.frames.map(({label, presentedFrame, virtualElapsedNs}) => ({label, presentedFrame, virtualElapsedNs})), plan.nativeCaptures);
  return {status: 'pass', acknowledgements: plan.nativeAcknowledgements.length, captures: plan.nativeCaptures.length,
    scope: 'Actual independent Native acknowledgement/capture times validate the prior public declaration; no readback changes a Browser command'};
}

export async function nativeCapture(caseData, directory, nativeFrameTimeout, originalObserver, archiveNativeEvidence, originalSfxVolume, mtDraws, glTrace, deferEvidence = false, reuseExisting = false, publicClockPlan, audioOwnerTrace, optionsAudioOwners = false, originalMusicVolume) {
  mkdirSync(directory, { recursive: true });
  const args = ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
    '--seed', caseData.seed, '--tape', join(dirname(directory), 'tape.json'),
    '--state-observer', originalObserver, '--frame-timeout', String(nativeFrameTimeout), '--output', directory];
  if (publicClockPlan) args.push('--boot-epoch-seed', String(publicClockPlan.bootEpochSeed),
    '--step-ns', String(publicClockPlan.presentationIntervalNanoseconds));
  if (optionsAudioOwners) {
    assert.ok(audioOwnerTrace, 'Options owners require the actual audio observer library');
    args.push('--audio-options-owners');
  }
  if (archiveNativeEvidence) args.push('--archive-evidence');
  if (originalSfxVolume !== undefined) args.push('--sfx-volume', String(originalSfxVolume));
  if (originalMusicVolume !== undefined) {
    assert.ok(Number.isFinite(originalMusicVolume) && originalMusicVolume >= 0 && originalMusicVolume <= 1, 'Explicit Native Music volume must be within 0..1');
    assert.ok(!audioOwnerTrace || originalMusicVolume === 1, 'Audio owner tracing already declares Music volume1');
    if (!audioOwnerTrace) args.push('--music-volume', String(originalMusicVolume));
  }
  if (audioOwnerTrace) {
    assert.equal(originalSfxVolume, 1);
    args.push('--audio-trace', audioOwnerTrace, '--audio-sfx-owners', '--music-volume', '1');
  }
  if (mtDraws) args.push('--mt-trace');
  if (glTrace) args.push('--gl-trace', glTrace);
  let error;
  if (!reuseExisting) try {
    const result = await runProcess('bash', args, { cwd: ROOT, maxBuffer: 8 * 1024 * 1024 });
    writeFileSync(join(directory, 'controller-output.txt'), result.stdout + result.stderr);
  } catch (failure) {
    error = String(failure.message);
    writeFileSync(join(directory, 'controller-output.txt'), String(failure.stdout || '') + String(failure.stderr || '') + error);
  }
  if (deferEvidence) return { status: error ? 'fail' : 'deferred-evidence', error, directory,
    manifestPath: join(directory, archiveNativeEvidence ? 'manifest.json.gz' : 'manifest.json'),
    deferredEvidence: () => readCapturedNative(directory, archiveNativeEvidence, error) };
  return readCapturedNative(directory, archiveNativeEvidence, error);
}

async function readCapturedNative(directory, archiveNativeEvidence, error) {
  let manifest;
  const manifestPath = join(directory, archiveNativeEvidence ? 'manifest.json.gz' : 'manifest.json');
  let evidence;
  try {
    if (archiveNativeEvidence) {
      ({ manifest, evidence } = await readNativeEvidence(manifestPath));
    } else manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  }
  catch (failure) { return { status: 'fail', error: error || String(failure.message), directory, manifestPath }; }
  let provenance;
  try { provenance = await python(['--validate-original', manifestPath]); }
  catch (failure) {
    try { provenance = JSON.parse(failure.stdout); }
    catch { provenance = { status: 'fail', failures: [String(failure.message)] }; }
  }
  return { status: error || provenance.status !== 'pass' ? 'fail' : 'pass', error, provenance, directory, manifest, manifestPath, evidence };
}

export async function browserCapture(browser, server, caseData, directory, archiveBrowserEvidence, captureBrowserAudio, mtDraws, publicClockPlan,
  audioPreferences = {sfxVolume: 0, musicVolume: 0}, captureControlsOwners = false) {
  mkdirSync(directory, { recursive: true });
  assert.equal(typeof captureControlsOwners, 'boolean');
  const context = await browser.newContext({ viewport: { width: 960, height: 540 } });
  if (captureControlsOwners) await context.addInitScript(installPhysicalObserver);
  const page = await context.newPage();
  const pageErrors = [];
  if (captureControlsOwners) page.on('pageerror',error=>pageErrors.push({message:String(error),stack:error.stack}));
  const frames = [], inputs = [], expectedCommands = [];
  // Independent launcher inputs, never Original readback; before either capture.
  assert.ok(audioPreferences && typeof audioPreferences === 'object' && !Array.isArray(audioPreferences));
  assert.deepEqual(Object.keys(audioPreferences).sort(), ['musicVolume', 'sfxVolume']);
  assert.ok([audioPreferences.sfxVolume, audioPreferences.musicVolume].every(value => Number.isFinite(value) && value >= 0 && value <= 1));
  const startup = {audioPreferences: {...audioPreferences}, fullscreenPreferences: {fullscreen: false}, inputConfigs: {devices: []}};
  save(join(directory, 'declared-startup-preferences.json'), startup);
  const expectedPressed = new Set();
  // DOM held keys and Playwright modifier flags are distinct from engine pressed
  // state: seed text does not create a physical engine keydown.
  const browserPressed = new Set(), browserModifiers = new Set();
  let expectedSeedDialog = false;
  const failures = [];
  try {
    await page.goto(server.url + '/?manual=1');
    await page.waitForFunction(() => ['true', 'failed'].includes(document.body.dataset.ready));
    assert.equal(await page.evaluate(() => document.body.dataset.ready), 'true', 'Production browser must boot');
    await page.evaluate(({ initialSeed, bootEpochSeed, mtDraws, startup }) => {
      window.game.pause(); window.game.reset(initialSeed, { bootEpochSeed, trace: true, ...startup });
      if (mtDraws) window.game.mtDraws.enable();
    }, { ...caseData, mtDraws, startup });
    let presentations = 0;
    let publicTickIndex = 0;
    for (const action of caseData.pairedTape) {
      if (action.type === 'step') {
        // Steps transfer no state: serializing a full snapshot per step costs
        // several times the step itself (see tools/e2e/lib/game.mjs).
        if (publicClockPlan) {
          const commands = publicClockPlan.browserTickCommands.slice(publicTickIndex, publicTickIndex + action.frames);
          assert.equal(commands.length, action.frames);
          await page.evaluate(commands => { for (const command of commands) window.game.step(1, command.timestampMillis); }, commands);
          publicTickIndex += action.frames;
          expectedCommands.push(...commands);
        } else {
          await page.evaluate(count => { window.game.step(count); }, action.frames);
          expectedCommands.push({ type: 'tick', count: action.frames });
        }
        presentations += action.frames;
      } else if (action.type === 'key') {
        const code = browserKeyCode(action.key);
        const repeat = action.down && browserPressed.has(code);
        const modifier = modifierName(code);
        if (action.down) browserPressed.add(code); else browserPressed.delete(code);
        if (modifier) {
          if (action.down) browserModifiers.add(modifier); else browserModifiers.delete(modifier);
        }
        // Derive text/edges from declared public events and the driver's actual
        // modifier protocol. No observed screen or State can redefine missed input.
        const text = expectedSeedDialog ? seedInputText(code, browserModifiers) : null;
        await page.keyboard[action.down ? 'down' : 'up'](code);
        if (!action.down) {
          if (expectedPressed.delete(code)) expectedCommands.push({ type: 'key', code, down: false });
        } else if (!repeat) {
          if (text !== null) expectedCommands.push({ type: 'text', text });
          else {
            expectedCommands.push({ type: 'key', code, down: true });
            const newlyPressed = !expectedPressed.has(code);
            expectedPressed.add(code);
            if (newlyPressed && code === 'Tab') expectedSeedDialog = !expectedSeedDialog;
            else if (newlyPressed && (code === 'Enter' || code === 'Escape')) expectedSeedDialog = false;
          }
        }
        inputs.push({ ...action, normalizedCode: code, browserPresentation: presentations });
      } else {
        // capture() renders and encodes the snapshot it takes first; the state
        // crosses the page boundary as that snapshot's own engine JSON text.
        const captured = await page.evaluate(includeAudio => {
          const stateJson = window.game.snapshotJson();
          const { state: ignoredState, ...result } = window.game.capture();
          if (includeAudio) result.browserAudio = window.game.audio();
          return { ...result, stateJson };
        }, captureBrowserAudio);
        captured.state = JSON.parse(captured.stateJson);
        if (captureControlsOwners) {
          captured.controls = await readBrowserControls(page);
          assert.equal(captured.controls.tick,captured.state.tick,'Actual Controls owners must share the captured manual presentation');
          assert.equal(captured.controls.screen,captured.state.screen);
        }
        const bytes = Buffer.from(captured.png, 'base64');
        const path = join(directory, action.label + '.png');
        writeFileSync(path, bytes);
        frames.push({ label: action.label, path, sha256: digest(bytes), browserPresentation: presentations,
          state: captured.state, ...(captureControlsOwners ? {controls: captured.controls} : {}), render: captured.render, ...(captureBrowserAudio ? {browserAudio: captured.browserAudio} : {}) });
      }
    }
    if (mtDraws) save(join(directory, 'mt-draws.json'), await evaluateJson(page, () => window.game.mtDraws.take()));
    const trace = await readTrace(page);
    if (archiveBrowserEvidence) {
      const evidence = await writeJsonEvidence(join(directory, 'trace.json.gz'), trace);
      save(join(directory, 'trace-archive.json'), evidence);
    } else save(join(directory, 'trace.json'), trace);
    if (publicClockPlan) {
      assert.equal(publicTickIndex, publicClockPlan.browserTickCommands.length, 'Every declared public timestamp must be consumed');
      assert.deepEqual(trace.commands.filter(command => command.type === 'tick'), publicClockPlan.browserTickCommands);
    }
    assert.equal(trace.enabled, true); assert.equal(trace.inputTraceTruncated, false);
    assert.equal(trace.backendStateTruncated, false, 'Every actual backend callback must retain its complete snapshot');
    assert.equal(trace.backendStateTruncationReason, null);
    assert.ok(Array.isArray(trace.backendEvents) && trace.backendEvents.every(event =>
      event.state && typeof event.state === 'object' && !Array.isArray(event.state)),
    'Every actual backend event requires its recorded WASM state');
    assert.equal(trace.bootEpochSeed, caseData.bootEpochSeed);
    assert.equal(trace.initialSeed, caseData.initialSeed);
    assert.deepEqual(trace.bootAudioPreferences, startup.audioPreferences, 'Actual boot must retain the independently declared audio settings');
    assert.deepEqual(trace.bootFullscreenPreferences, startup.fullscreenPreferences, 'Actual reset must retain its declared window default');
    assert.deepEqual(trace.bootInputConfigs, startup.inputConfigs, 'Fresh Original profiles require independently declared fresh keyboard defaults');
    assert.deepEqual(trace.commands, expectedCommands, 'Every intended DOM input/interval must reach the actual WASM trace');
    const final = frames.at(-1)?.state;
    assert.equal(final?.runSeedValue, caseData.seedValue, 'Browser must read back the public manually entered run seed');
    const requestedSeed = caseData.seed.replaceAll(' ', '').toUpperCase();
    assert.equal(final?.seedText, requestedSeed, 'Browser must read back every requested seed character');
    assert.equal(final?.seedValid, true, 'Browser must validate the requested seed');
    assert.equal(final?.seededRun, true, 'Browser must commit the public manual seed entry');
    assert.equal(final?.runSeed, requestedSeed, 'Browser must read back the committed public run code');
  } catch (error) { failures.push(String(error.message)); }
  finally {
    if (captureControlsOwners) {
      try {
        const physical=await page.evaluate(()=>window.__controlsPhysicalInputs());
        save(join(directory,'physical-inputs.json'),physical);
        assert.equal(physical.truncated,false);
        const expected=caseData.pairedTape.filter(action=>action.type==='key').map(action=>({type:action.down?'keydown':'keyup',code:browserKeyCode(action.key)}));
        assert.deepEqual(physical.events.map(({type,code})=>({type,code})),expected,'Actual trusted physical edges must equal the complete declared tape');
        assert.ok(physical.events.every(event=>event.isTrusted&&!event.repeat&&!event.ctrlKey&&!event.altKey&&!event.metaKey&&!event.shiftKey));
      } catch(error) { failures.push(String(error.message)); }
      save(join(directory,'page-errors.json'),pageErrors);
      if (pageErrors.length) failures.push('Actual Browser pageerror(s): '+JSON.stringify(pageErrors));
    }
    await context.close();
  }
  const result = { status: failures.length ? 'fail' : 'pass', failures, frames, inputs };
  if (archiveBrowserEvidence) {
    const evidence = await writeJsonEvidence(join(directory, 'manifest.json.gz'), result);
    save(join(directory, 'manifest-archive.json'), evidence);
  // Compact JSON: long routes hold hundreds of full snapshots, and indented
  // output can exceed V8's maximum string length before it is written.
  } else writeFileSync(join(directory, 'manifest.json'), JSON.stringify(result) + '\n');
  return result;
}

export function scopedState(source, actual, gameplay) {
  assert.equal(source?.status, 'pass', 'Every paired capture requires a successful typed source observation');
  assert.equal(source.memoryWritten, false, 'Source observations must remain read-only');
  const checks = [];
  const check = (field, expected, observed) => checks.push({ field, expected, actual: observed, status: JSON.stringify(expected) === JSON.stringify(observed) ? 'pass' : 'fail' });
  if (!gameplay) {
    assert.equal(source.menu?.status, 'pass', 'Pre-run captures require typed source menu evidence');
    assert.equal(source.menu.menuInitialized, true, 'Pre-run captures require an initialized source menu');
    assert.ok([1, 2, 3, 4].includes(source.menu.menuScreen), 'Pre-run captures require a known source menu family');
    // Numeric menu families are corroborated by this gate's fresh native
    // title/profile/main/character labels; the seed dialog belongs to family4.
    const menuScreens = { 1: 'title', 2: 'profile', 3: 'main' };
    if (menuScreens[source.menu.menuScreen]) check('screen', menuScreens[source.menu.menuScreen], actual.screen);
    else if (source.menu.menuScreen === 4) check('characterMenuFamily', true, ['character', 'seed'].includes(actual.screen));
    check('titleAnimationFrame', source.menu.titleAnimation.frame, actual.titleAnimationFrame);
    check('menuFade.active', source.menu.menuFade.active, actual.menuFade.active);
    check('menuFade.alpha', source.menu.menuFade.alpha, actual.menuFade.alpha);
    check('menuFade.color', source.menu.menuFade.color, actual.menuFade.color);
  } else {
    assert.equal(source.gameplay?.status, 'pass', 'Gameplay captures require typed source gameplay evidence');
    assert.ok(Array.isArray(source.gameplay.players) && source.gameplay.players.length > 0, 'Gameplay captures require an observed source player');
    assert.ok(Array.isArray(source.gameplay.entities), 'Gameplay captures require observed source entities');
    assert.equal(typeof source.pause?.paused, 'boolean', 'Gameplay captures require typed source pause evidence');
    const player = source.gameplay.players[0];
    check('screen', 'playing', actual.screen);
    // The port's public player coordinates use the observed affine room
    // projection; source memory supplies world f32 values independently.
    check('player.x', player.position[0] * 0.65 + 32, actual.player.x);
    check('player.y', player.position[1] * 0.65 - 46, actual.player.y);
    check('player.vx', player.velocity[0] * 0.65, actual.player.vx);
    check('player.vy', player.velocity[1] * 0.65, actual.player.vy);
    check('player.hp', player.redHp, actual.player.hp);
    check('damageCooldown', player.damageCountdown, actual.damageCooldown);
    const admitted = source.gameplay.entities.filter(entity => !entity.removed);
    for (const tear of actual.tears) assert.ok(['alive', 'dying', 'removed'].includes(tear.motion?.lifetime), 'Paired tears require a typed lifetime');
    // Source +0x31f marks dying while the manager still retains the object.
    const activeTears = actual.tears.filter(tear => tear.motion.lifetime === 'alive');
    check('tears.alive', admitted.filter(entity => entity.type === 2).length, activeTears.length);
    const livingBombs = actual.bombs.filter(bomb => bomb.motion === undefined || bomb.motion.lifetime === 'alive');
    check('bombs.alive', admitted.filter(entity => entity.type === 4).length, livingBombs.length);
    check('paused', source.pause.paused, actual.paused);
    if (source.pause.paused) {
      check('pauseMenu.selection', source.pause.selection, actual.pauseMenu.selection);
      const phases = { Appear: 'appearing', Idle: 'idle', Dissapear: 'disappearing' };
      if (phases[source.pause.animation]) check('pauseMenu.phase', phases[source.pause.animation], actual.pauseMenu.phase);
    }
  }
  assert.ok(checks.length > 0, 'Every paired capture must compare typed functional fields');
  return checks;
}

// The labels whose pixel diagnostics comparePair reaches, as pool jobs. Any
// precondition comparePair would reject leaves its label (or the whole case)
// to comparePair itself, so no difference image is written that it would not.
function pixelJobs(caseData, native, browser, directory) {
  try {
    const originals = new Map((native.manifest?.frames || []).map(frame => [frame.label, frame]));
    const candidates = new Map(browser.frames.map(frame => [frame.label, frame]));
    const labels = caseData.pairedTape.filter(action => action.type === 'capture').map(action => action.label);
    if (labels.indexOf('gameplay-start') < 0) return [];
    for (const scope of caseData.pairedIndependent || [])
      if (!(labels.indexOf(scope.after) >= 0 && scope.fields?.length && scope.reason)) return [];
    const first = originals.get(caseData.bootstrap.boundaryLabel);
    if (!first) return [];
    mkdirSync(directory, { recursive: true });
    const jobs = [];
    for (const label of labels) {
      const original = originals.get(label), candidate = candidates.get(label);
      if (!original || !candidate || !Object.is(original.presentedFrame - first.presentedFrame, candidate.browserPresentation)
        || typeof original.path !== 'string' || typeof candidate.path !== 'string') continue;
      jobs.push({ label, sourcePath: resolve(dirname(join(native.directory, 'manifest.json')), original.path),
        candidatePath: candidate.path, differencePath: join(directory, label + '-difference.png') });
    }
    return jobs;
  } catch { return []; }
}

function comparePair(caseData, native, browser, directory, exactPixels, stateComparator, pixelResults = new Map()) {
  mkdirSync(directory, { recursive: true });
  const checks = [];
  const originals = new Map((native.manifest?.frames || []).map(frame => [frame.label, frame]));
  const observations = new Map((native.manifest?.debugObservations || []).map(row => [row.context.label, row.observation]));
  const candidates = new Map(browser.frames.map(frame => [frame.label, frame]));
  const labels = caseData.pairedTape.filter(action => action.type === 'capture').map(action => action.label);
  const gameplayStart = labels.indexOf('gameplay-start');
  assert.ok(gameplayStart >= 0, 'Selected paired tape must declare its gameplay boundary');
  const first = originals.get(caseData.bootstrap.boundaryLabel);
  // A case may declare state fields that stop being comparable after a
  // checkpoint because each game then plays its own inputs (stage-exit's
  // per-game boss fights). They stay in the report as 'independent'.
  const independent = (caseData.pairedIndependent || []).map(scope => {
    const after = labels.indexOf(scope.after);
    assert.ok(after >= 0 && scope.fields?.length && scope.reason, `Independent scope needs a captured 'after' label, fields and a reason`);
    return { ...scope, after };
  });
  for (const [index, label] of labels.entries()) {
    const check = { label, status: 'fail' };
    try {
      const original = originals.get(label), candidate = candidates.get(label);
      assert.ok(original && candidate, `Missing actual paired checkpoint ${label}`);
      assert.equal(original.presentedFrame - first.presentedFrame, candidate.browserPresentation, 'Both games must receive identical paired presentation intervals');
      let pixels, sourcePNGHash;
      const pooled = pixelResults.get(label);
      if (pooled) {
        if (Object.hasOwn(pooled, 'error')) throw new Error(pooled.error);
        ({ pixels, sourcePNGHash } = pooled);
      } else {
        const sourceBytes = readFileSync(resolve(dirname(join(native.directory, 'manifest.json')), original.path));
        const difference = pixelDifference(PNG.sync.read(sourceBytes), PNG.sync.read(readFileSync(candidate.path)));
        writeFileSync(join(directory, label + '-difference.png'), PNG.sync.write(difference.difference));
        const { difference: ignoredImage, ...stats } = difference;
        pixels = stats;
        sourcePNGHash = digest(sourceBytes);
      }
      check.pixels = { ...pixels, status: pixels.exactRGBA ? 'pass' : 'fail', required: exactPixels };
      check.sourcePNGHash = sourcePNGHash;
      check.state = stateComparator(observations.get(label), candidate.state, index >= gameplayStart, candidate);
      assert.ok(Array.isArray(check.state) && check.state.length > 0, 'Every paired checkpoint requires typed state checks');
      // Only an observed scalar mismatch may be incomparable. Missing/error
      // evidence remains blocking, and an existing exact PASS is retained.
      const observedScalar = value => typeof value === 'number' ? Number.isFinite(value)
        : typeof value === 'string' || typeof value === 'boolean';
      for (const field of check.state) {
        const scopes = independent.filter(scope => index > scope.after && scope.fields.includes(field.field));
        if (!scopes.length) continue;
        const comparisonStatus = field.status;
        Object.assign(field, { comparisonStatus,
          independentScopes: scopes.map(scope => ({ after: labels[scope.after], reason: scope.reason })) });
        if (!['pass', 'fail'].includes(comparisonStatus) || Object.hasOwn(field, 'error') || field.missing
            || !observedScalar(field.expected) || !observedScalar(field.actual)
            || typeof field.expected !== typeof field.actual) field.status = 'fail';
        else if (comparisonStatus === 'fail')
          Object.assign(field, { status: 'independent', reason: scopes.at(-1).reason });
      }
      check.status = (!exactPixels || pixels.exactRGBA) && check.state.every(field => ['pass', 'independent'].includes(field.status)) ? 'pass' : 'fail';
    } catch (error) { check.error = String(error.message); }
    checks.push(check);
  }
  assert.ok(checks.length > 0, 'Selected paired scenario must compare actual checkpoints');
  return checks;
}

export async function runPairedGate({ output, selected, randomCount = 2, replaySeeds = [], replayCases = [], exactPixels = false, productionDirectory, reuseOriginalDirectory,
  nativeFrameTimeout = 60, originalObserver = 'tools/original/foundation-paired-observer.py',
  stateComparator = scopedState, captureControlsOwners = false, movementComparator, archiveBrowserEvidence = false, archiveNativeEvidence = false, captureBrowserAudio = false, captureAudioOwners = false,
  originalSfxVolume, originalMusicVolume, filterCases = false, publicClockSchedule = false, mtDraws = process.argv.includes('--mt-draws') || process.env.E2E_MT_DRAWS === '1',
  glTrace = process.argv.includes('--gl-trace') }) {
  assert.ok(Number.isFinite(nativeFrameTimeout) && nativeFrameTimeout > 0 && nativeFrameTimeout <= 300,
    'Native frame timeout must be more than 0 and at most 300 host-wall seconds');
  assert.equal(typeof publicClockSchedule, 'boolean', 'Public clock scheduling is an explicit gate opt-in');
  assert.equal(typeof captureAudioOwners, 'boolean', 'Audio owner instrumentation is an explicit gate opt-in');
  assert.equal(typeof captureControlsOwners, 'boolean', 'Controls owners require explicit opt-in');
  assert.ok(movementComparator === undefined || typeof movementComparator === 'function', 'Aggregate movement must be a callable comparator');
  assert.ok(!captureAudioOwners || (captureBrowserAudio && originalSfxVolume === 1), 'Audio owners require actual Browser audio and explicit public SFX volume1');
  const audioPreferences = {sfxVolume: originalSfxVolume ?? 0, musicVolume: originalMusicVolume ?? (captureAudioOwners ? 1 : 0)};
  assert.ok([audioPreferences.sfxVolume, audioPreferences.musicVolume].every(value => Number.isFinite(value) && value >= 0 && value <= 1));
  assert.ok(!captureAudioOwners || audioPreferences.musicVolume === 1, 'Audio owner tracing requires Music volume1');
  mkdirSync(output, { recursive: true });
  let server, browser;
  const report = { schema: 1, status: 'starting', mode: reuseOriginalDirectory ? 'closed independent Original paired with fresh immutable Browser capture' : 'fresh original process paired with immutable browser production artifact',
    nativeFrameTimeout, originalSfxVolume, publicClockSchedule, declaredAudioPreferences: audioPreferences,
    originalObserver, archiveBrowserEvidence, archiveNativeEvidence, captureBrowserAudio, mtDraws, glTrace,
    ...(captureAudioOwners && {captureAudioOwners}), ...(captureControlsOwners && {captureControlsOwners}),
    comparisons: { functional: 'Real input/phase/readback and source title, fade, player motion, pause/controller, HP, immunity, active tear/bomb counts',
      pixels: 'Strict full RGBA diagnostic; every difference remains a failure in its own pixel result',
      exactPixelsRequired: exactPixels, completeOriginalStateParity: false },
    cases: [] };
  try {
    const declarations = await pairedScenarioDeclarations();
    const known = new Set(declarations.map(item => item.id));
    for (const id of selected) assert.ok(known.has(id), `Unknown paired scenario ${id}`);
    report.sourceBinarySha256 = digest(readFileSync(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64')));
    // The public GL observer the original preloads (`--gl-trace`): it
    // records submitted draws, including every room-cache backdrop draw.
    const glTraceLibrary = glTrace ? join(output, 'gl-trace.so') : null;
    if (glTraceLibrary)
      await runProcess('cc', ['-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared', '-Wl,--no-undefined',
        join(ROOT, 'tools/original/foundation-gl-trace.c'), join(ROOT, 'tools/original/foundation-gl-state.c'),
        '-o', glTraceLibrary, '-ldl', '-lm'], { cwd: ROOT });
    const audioOwnerTrace = captureAudioOwners ? join(output, 'audio-trace.so') : undefined;
    if (audioOwnerTrace) {
      await runProcess('cc', ['-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
        join(ROOT, 'tools/original/audio-trace.c'), '-o', audioOwnerTrace, '-ldl', '-pthread'], {cwd: ROOT});
      report.audioOwnerTraceLibrarySha256 = digest(readFileSync(audioOwnerTrace));
    }
    server = await startProductionServer({ directory: productionDirectory });
    report.build = server.build;
    if (captureControlsOwners && !reuseOriginalDirectory) {
      const disk = statfsSync(output);
      assert.ok(Number(disk.bavail) * Number(disk.bsize) > 4 * 1024 ** 3, 'Fresh Native Controls requires4GiB free after the immutable production snapshot');
    }
    report.captureSchedule = 'sequential independent Original then Browser, one Chromium lifetime per case';
    report.status = 'running';
    let definitions = selected.map(id => ({ id, seed: 'T1PK6Y0W' }));
    for (let index = 0; index < randomCount; index++) definitions.push({ id: 'paired-random-run', seed: replaySeeds[index] });
    for (const path of replayCases) definitions.push({ id: JSON.parse(readFileSync(path, 'utf8')).scenario.id, replayCase: path });
    // Only the CLI opts in: wrapper drivers filter their own banks before
    // handing them over as replay cases. Fresh OS-random cases all share one id.
    if (filterCases) { definitions = selectCases(definitions, item => item.id); report.caseFilter = flag('--case'); }
    assert.ok(!reuseOriginalDirectory || definitions.length === 1, 'Closed Original reuse requires exactly one declared case');
    for (const [index, definition] of definitions.entries()) {
      const directory = join(output, `${String(index).padStart(2, '0')}-${definition.id}`);
      const args = ['--scenario', definition.id, '--output', directory];
      if (definition.seed) args.push('--seed', definition.seed);
      let caseData, result, native, candidate, publicClockPlan, publicClockValidation;
      try {
        if (definition.replayCase) {
          caseData = JSON.parse(readFileSync(definition.replayCase, 'utf8'));
          mkdirSync(directory, { recursive: true });
          save(join(directory, 'case.json'), caseData); save(join(directory, 'tape.json'), caseData.tape);
        } else caseData = await python(args);
        if (reuseOriginalDirectory) {
          const source = resolve(reuseOriginalDirectory);
          assert.deepEqual(JSON.parse(readFileSync(join(dirname(source), 'case.json'), 'utf8')), caseData,
            'Reused Original declaration must equal the independently supplied Browser case');
          assert.equal(digest(readFileSync(join(dirname(source), 'tape.json'))), digest(readFileSync(join(directory, 'tape.json'))),
            'Reused Original must have the exact same public tape bytes');
          symlinkSync(source, join(directory, 'original'), 'dir');
          report.reusedOriginal = { directory: source, caseSha256: digest(readFileSync(join(directory, 'case.json'))),
            scope: 'Caller-pinned completed independent Original; Browser inputs still come only from the declared case bank' };
        }
        if (publicClockSchedule) {
          publicClockPlan = declarePublicClockPlan(caseData);
          save(join(directory, 'public-clock-plan.json'), publicClockPlan);
        }
        // Reject unavailable physical DOM transport before either game capture.
        for (const action of caseData.pairedTape)
          if (action.type === 'key') browserKeyCode(action.key);
        process.stdout.write(`Paired ${definition.id} seed ${caseData.seed}\n`);
        // Both independent captures are still attempted after a rejection.
        // Original settles before Chromium starts; each Browser closes before
        // the next Original. Both use declared inputs; comparison laws stay unchanged.
        const [nativeResult] = await Promise.allSettled([
          nativeCapture(caseData, join(directory, 'original'), nativeFrameTimeout, originalObserver, archiveNativeEvidence, originalSfxVolume, mtDraws, glTraceLibrary, true, Boolean(reuseOriginalDirectory), publicClockPlan, audioOwnerTrace, false, originalMusicVolume),
        ]);
        native = nativeResult.status === 'fulfilled' ? nativeResult.value
          : {status: 'fail', error: String(nativeResult.reason), directory: join(directory, 'original')};
        let browserResult;
        try {
          browser = await chromium.launch({ ...browserLaunchOptions(), headless: false });
          report.browserGraphics = await inspectBrowserGraphics(browser);
          [browserResult] = await Promise.allSettled([
            browserCapture(captureAudioOwners ? audioObservedBrowser(browser, api => save(join(directory, 'browser', 'api.json'), api)) : browser,
              server, caseData, join(directory, 'browser'), archiveBrowserEvidence, captureBrowserAudio, mtDraws, publicClockPlan, audioPreferences, captureControlsOwners),
          ]);
        } catch (error) {
          browserResult = { status: 'rejected', reason: error };
        } finally {
          if (browser) await browser.close();
          browser = undefined;
        }
        candidate = browserResult.status === 'fulfilled' ? browserResult.value
          : {status: 'fail', failures: [String(browserResult.reason)], frames: []};
        if (native.deferredEvidence) native = await native.deferredEvidence();
        // Deferred evidence stays after Browser close. Validate launcher inputs;
        // Original readback cannot repair or alter the already executed reset.
        try { assert.deepEqual(native.manifest?.audioOptions, audioPreferences, 'Actual Native public audio settings match the independent declaration'); }
        catch (error) { native.status = 'fail'; native.error = [native.error, String(error.message)].filter(Boolean).join('; '); }
        if (publicClockPlan) {
          try {
            publicClockValidation = validateNativeClockPlan(caseData, publicClockPlan, native);
          } catch (error) {
            native.status = 'fail';
            native.error = [native.error, String(error.message)].filter(Boolean).join('; ');
            publicClockValidation = {status: 'fail', error: String(error.message)};
          }
          save(join(directory, 'native-public-clock-validation.json'), publicClockValidation);
        }
        const pixelResults = await comparePixelPairs(pixelJobs(caseData, native, candidate, join(directory, 'comparison')));
        const checks = comparePair(caseData, native, candidate, join(directory, 'comparison'), exactPixels, stateComparator, pixelResults);
        if (movementComparator) {
          const state = movementComparator(native,candidate);
          assert.ok(Array.isArray(state) && state.length > 0,'Actual independent movement/coasting aggregate is required');
          checks.push({label:'required-movement-and-old-key-refusal',state,status:state.every(row=>row.status==='pass')?'pass':'fail'});
        }
        result = { id: definition.id, seed: caseData.seed, seedValue: caseData.seedValue, bootEpochSeed: caseData.bootEpochSeed,
          caseSha256: digest(readFileSync(join(directory, 'case.json'))),
          directory, original: { status: native.status, error: native.error, provenance: native.provenance,
            manifestPath: native.manifestPath, evidence: native.evidence,
            ...(publicClockPlan && {publicClockValidation}) },
          browser: { status: candidate.status, failures: candidate.failures }, checks,
          ...(publicClockPlan && {publicClockPlan: {path: join(directory, 'public-clock-plan.json'), nativeEntryPresentation: publicClockPlan.nativeEntryPresentation}}),
          status: native.status === 'pass' && candidate.status === 'pass'
            && (!publicClockPlan || publicClockValidation?.status === 'pass')
            && checks.every(check => check.status === 'pass') ? 'pass' : 'fail' };
        if (mtDraws) {
          try {
            result.mtDraws = compareMtDraws(join(directory, 'original', 'mt-trace'),
              JSON.parse(readFileSync(join(directory, 'browser', 'mt-draws.json'), 'utf8')), 6, independentDraws(caseData, directory));
          } catch (error) { result.mtDraws = { status: 'diagnostic', error: String(error) }; }
        }
      } catch (error) {
        result = {id: definition.id, seed: caseData?.seed, seedValue: caseData?.seedValue,
          directory, status: 'fail', error: String(error), checks: [],
          ...(error instanceof UnsupportedBrowserKeyTransport && {inputTransportFailure: {kind: 'unsupportedBrowserPhysicalCode', code: error.physicalCode}}),
          original: {status: native?.status || 'fail', error: native?.error},
          browser: {status: candidate?.status || 'fail', failures: candidate?.failures},
          scope: 'Declared case aborted; retained raw/partial captures have no complete paired acceptance'};
      }
      report.cases.push(result);
      try { save(join(directory, 'report.json'), result); }
      catch (error) { result.status = 'fail'; result.evidenceWriteError = String(error); }
      process.stdout.write(`${result.status}: ${result.checks.filter(check => check.status === 'pass').length}/${result.checks.length} paired checkpoints\n`);
      // An unresolved Chromium close must not overlap the next Original.
      // Its failed case/report is retained; outer finally retries cleanup.
      if (browser) break;
    }
    report.status = report.cases.length > 0 && report.cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
    report.exactOriginalPixels = report.cases.every(item => item.checks.length > 0 && item.checks.every(check => check.pixels?.exactRGBA));
  } catch (error) {
    report.status = 'fail'; report.fatalError = String(error);
    throw error;
  } finally {
    const closed = await Promise.allSettled([browser?.close(), server?.close()]);
    const failures = closed.filter(result => result.status === 'rejected').map(result => String(result.reason));
    if (failures.length) { report.status = 'fail'; report.cleanupFailures = failures; }
    save(join(output, 'report.json'), report);
  }
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.includes('--list')) console.log(JSON.stringify(await pairedScenarioDeclarations(), null, 2));
  else {
    const replayCases = [...option('--replay-cases', '').split(',').filter(Boolean),
      ...option('--replay-case', '').split(',').filter(Boolean)].map(path => resolve(path));
    const selected = option('--scenarios', replayCases.length ? '' : DEFAULT_PAIRED_SCENARIOS.join(',')).split(',').filter(Boolean);
    const replaySeeds = option('--random-seeds', '').split(',').filter(Boolean);
    const randomCount = countOption('--random-count', replaySeeds.length || (replayCases.length ? 0 : DEFAULT_RANDOM_CASE_COUNT), 0);
    assert.ok(!replaySeeds.length || replaySeeds.length === randomCount, 'Replay seed count must match randomized case count');
    const output = resolve(option('--output', `.cache/foundation/paired-${Date.now()}`));
    const report = await runPairedGate({ output, selected, randomCount, replaySeeds, replayCases,
      publicClockSchedule: process.argv.includes('--public-clock-schedule'),
      exactPixels: process.argv.includes('--require-exact-pixels'), productionDirectory: option('--production-directory'),
      archiveNativeEvidence: process.argv.includes('--archive-native-evidence'), filterCases: true,
      nativeFrameTimeout: Number(option('--native-frame-timeout', 60)) });
    console.log(JSON.stringify({ status: report.status, report: join(output, 'report.json'), artifact: report.build.sha256 }));
    process.exitCode = report.status === 'pass' ? 0 : 1;
  }
}
