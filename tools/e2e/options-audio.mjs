// Public Options audio/Back inputs, actual Native owners and Browser APIs.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {chromium} from '@playwright/test';
import {PNG} from 'pngjs';
import {flag, outputDirectory, selectCases, writeJson} from './lib/cli.mjs';
import {ORIGINAL_SHA, assertOriginalBinary} from './lib/original.mjs';
import {nativeCapture, browserCapture, declarePublicClockPlan, validateNativeClockPlan, scopedState} from '../foundation/paired-game.mjs';
import {startProductionServer} from '../foundation/production-server.mjs';
import {browserLaunchOptions, inspectBrowserGraphics} from '../foundation/browser.mjs';
import {pixelDifference} from '../foundation/compare.mjs';
import {compareOptionsAudio} from '../foundation/options-audio-state-pair.mjs';
import {optionsObservedBrowser} from './options-audio-observer.mjs';
import {optionsPreferenceContract} from '../foundation/options-preference-contract.mjs';

const ROOT = path.resolve(fileURLToPath(new URL('../..', import.meta.url)));
const read = name => JSON.parse(fs.readFileSync(name, 'utf8'));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const fileDigest = name => digest(fs.readFileSync(name));
const recorded = flag('--scenarios'), reuse = flag('--reuse-original'), count = flag('--count', '1');
for (const name of ['--scenarios', '--reuse-original', '--production-directory', '--output'])
  assert.ok(flag(name) === undefined || flag(name).length > 0, `${name} requires a nonempty value`);
// The current runtime writes V3. Historical V2 replay is an explicit input;
// it is never guessed from a bank, artifact, or observed Browser storage.
const preferencesVersion = flag('--preferences-schema', '3');
assert.ok(preferencesVersion === '2' || preferencesVersion === '3', '--preferences-schema must be 2 or 3');
const preferencesSchema = Number(preferencesVersion);
const preferenceContract = optionsPreferenceContract(preferencesSchema);
const output = outputDirectory('options-audio');
writeJson(path.join(output, 'preference-contract.json'), preferenceContract);
const helper = 'tools/original/options-scenarios.py';
const observer = path.join(ROOT, 'tools/original/options-observe.py');
const sourcePaths = [
  'tools/foundation/controls-browser-observe.mjs',
  'tools/e2e/lib/game.mjs', 'tools/foundation/pixel-pairs.mjs',
  'tools/e2e/options-audio.mjs', 'tools/e2e/options-audio-observer.mjs', 'tools/e2e/options-gain-observer.mjs',
  'tools/e2e/audio-api-observer.mjs', 'tools/e2e/lib/cli.mjs', 'tools/e2e/lib/original.mjs',
  'tools/foundation/paired-game.mjs', 'tools/foundation/options-audio-state-pair.mjs',
  'tools/foundation/options-preference-contract.mjs',
  'tools/foundation/browser.mjs', 'tools/foundation/production-server.mjs',
  'tools/foundation/native-evidence.mjs', 'tools/foundation/browser-evidence.mjs',
  'tools/foundation/compare.mjs', 'tools/foundation/mt-draws.mjs', 'tools/e2e/lib/audio-owners.mjs',
  'tools/foundation/native-journal-zstd.mjs', 'tools/original/native-journal-zstd.py',
  'tools/original/options-scenarios.py', 'tools/original/options-observe.py', 'tools/original/options-owner-reader.py',
  'references/isaac/cases/options-audio/audio-arrows-back-v1.json',
  'tools/original/audio-trace.c', 'tools/original/audio-observer.py', 'tools/original/music-observer.py',
  'tools/original/foundation-paired-observer.py', 'tools/original/foundation-menu-observer.py',
  'tools/original/foundation-combat-observe.py', 'tools/original/state-observer.py', 'tools/original/native-identity.py',
  'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c',
  'tools/original/foundation-clock-verify.py', 'tools/original/native-evidence.py',
  'tools/original/foundation-paired-scenarios.py', 'tools/original/foundation-clock-scenarios.py',
  'tools/original/foundation-bomb-scenario.py', 'tools/original/foundation-presentation-scenario.py',
  'tools/original/foundation-title-scenario.py', 'tools/original/foundation-combat-scenario.py',
  'tools/original/foundation-random-seeds.py', 'tools/original/foundation-seeds.py',
  'tools/original/debug-run.py', 'tools/original/debug-config.json', 'web/audio-data.json',
  'tools/original/with-display.sh', 'package-lock.json',
];
const sourceIdentity = () => Object.fromEntries(sourcePaths.map(name => [name, fileDigest(path.join(ROOT, name))]));
const before = sourceIdentity();
// Preserve the executing Source alongside evidence before either runtime;
// later canonical edits cannot silently reinterpret a retained capture.
for (const name of sourcePaths) {
  const target = path.join(output, 'executing-source', name);
  fs.mkdirSync(path.dirname(target), {recursive: true}); fs.copyFileSync(path.join(ROOT, name), target);
}
writeJson(path.join(output, 'source-before.json'), before);
// Archive the whole executing Source before replay validation or fresh draws.
const helperArgs = recorded !== undefined ? ['--scenarios', path.resolve(recorded)] : ['--count', count];
const bank = JSON.parse(execFileSync('python3', [helper, ...helperArgs], {cwd: ROOT, encoding: 'utf8', maxBuffer: 4 * 1024 * 1024}));
bank.cases = selectCases(bank.cases, data => data.id);
if (flag('--case')) bank.caseFilter = flag('--case');
writeJson(path.join(output, 'scenarios.json'), bank);
assert.ok(reuse === undefined || bank.cases.length === 1, 'Native reuse requires one exact declared case');

await assertOriginalBinary();
const library = path.join(output, 'options-audio-trace.so');
execFileSync('cc', ['-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
  'tools/original/audio-trace.c', '-o', library, '-ldl', '-pthread'], {cwd: ROOT});
const nativeSources = Object.fromEntries(sourcePaths.filter(name => name.startsWith('tools/original/')).map(name => [name, before[name]]));
const observerIdentity = {schema: 1, mode: preferencesSchema === 3 ? 'options-audio-v3' : 'options-audio-v1', binarySha256: ORIGINAL_SHA,
  captureProducerSha256: before['tools/foundation/paired-game.mjs'],
  librarySha256: fileDigest(library), sources: nativeSources, publicPreferences: {sfxVolume: 1, musicVolume: 1, fullscreen: false}};
const catalogue = read(path.join(ROOT, 'web/audio-data.json'));
const sound = catalogue.sounds.find(row => row.id === 237); assert.equal(sound.samples.length, 1);
const wav = fs.readFileSync(path.join(ROOT, 'references/isaac/rebirth/resources', sound.samples[0].path));
assert.equal(wav.toString('ascii', 0, 4), 'RIFF'); assert.equal(wav.toString('ascii', 8, 12), 'WAVE');
let pcm, waveFormat;
for (let at = 12; at + 8 <= wav.length;) {
  const size = wav.readUInt32LE(at + 4), begin = at + 8; assert.ok(begin + size <= wav.length);
  if (wav.toString('ascii', at, at + 4) === 'fmt ') waveFormat = {format: wav.readUInt16LE(begin), channels: wav.readUInt16LE(begin + 2), sampleRate: wav.readUInt32LE(begin + 4), bits: wav.readUInt16LE(begin + 14)};
  if (wav.toString('ascii', at, at + 4) === 'data') pcm = wav.subarray(begin, begin + size);
  at = begin + size + (size & 1);
}
assert.ok(pcm && waveFormat); assert.equal(waveFormat.format, 1); assert.equal(waveFormat.bits, 16); assert.equal(waveFormat.channels, 1);
const resourcePaths = [sound.samples[0].path, 'music/title screen intro.ogg', 'music/title screen.ogg'];
const inputIdentity = () => Object.fromEntries(resourcePaths.map(name => [name, fileDigest(path.join(ROOT, 'references/isaac/rebirth/resources', name))]));
const inputsBefore = inputIdentity();
for (const name of resourcePaths) assert.equal(inputsBefore[name], catalogue.assets[name].sha256, 'Actual authored input differs from its catalogue');
writeJson(path.join(output, 'input-before.json'), inputsBefore);
observerIdentity.authoredInputs = inputsBefore;
if (preferencesSchema === 3) observerIdentity.preferenceContract = preferenceContract;
const metadata = {waveFormat, pcm237Sha256: digest(pcm), pcm237Bytes: pcm.length,
  music61InputSha256s: ['music/title screen intro.ogg', 'music/title screen.ogg'].map(name => catalogue.assets[name].sha256)};
const report = {schema: preferencesSchema === 3 ? 2 : 1, status: 'running', caseFilter: bank.caseFilter, originalBinarySha256: ORIGINAL_SHA,
  observerIdentity, source: before, cases: [], scope: 'Main-family Options SFX/Music arrows and changed-value Back with actual request/backend owners, public clock and final public run seed. Pixels and complete state history remain explicit diagnostics; no full mix, complete MT or other callbacks.'};
if (preferencesSchema === 3) report.preferenceContract = preferenceContract;
let server, fatal;
try {
  server = await startProductionServer({directory: flag('--production-directory')}); report.production = server.build;
  for (const [index, data] of bank.cases.entries()) {
    const directory = path.join(output, `${String(index).padStart(2, '0')}-${data.id}`);
    fs.mkdirSync(directory, {recursive: true});
    writeJson(path.join(directory, 'case.json'), data); writeJson(path.join(directory, 'tape.json'), data.tape);
    const clock = declarePublicClockPlan(data); writeJson(path.join(directory, 'public-clock-plan.json'), clock);
    const identity = {...observerIdentity, caseSha256: fileDigest(path.join(directory, 'case.json')), clockSha256: digest(JSON.stringify(clock))};
    let native, browserResult, typed, error, browser;
    const observed = {checkpoints: [], errors: [], failures: []};
    const retain = value => fs.writeFileSync(path.join(directory, 'actual-browser-evidence.json.gz'), gzipSync(JSON.stringify(value)));
    try {
      if (reuse !== undefined) {
        const original = path.resolve(reuse), parent = path.dirname(original);
        assert.deepEqual(read(path.join(parent, 'case.json')), data, 'Reuse requires identical public case');
        assert.deepEqual(read(path.join(parent, 'public-clock-plan.json')), clock, 'Reuse requires identical declared clock');
        assert.deepEqual(read(path.join(parent, 'observer-identity.json')), identity, 'Reuse requires matching archived observer/source/library identity');
        const previous = read(path.join(parent, 'native-status.json'));
        assert.equal(previous.status, 'pass'); assert.equal(previous.provenance.status, 'pass');
        assert.equal(previous.publicClockValidation.status, 'pass'); assert.equal(previous.optionsAudioOwnerObserver, 1);
        fs.symlinkSync(original, path.join(directory, 'original'), 'dir');
      } else {
        const free = fs.statfsSync(output);
        assert.ok(free.bavail * free.bsize >= 4 * 1024 ** 3, 'Fresh Native requires 4 GiB reserve');
      }
      writeJson(path.join(directory, 'observer-identity.json'), identity);
      // Both independent captures are attempted. Native readback stays deferred
      // until Browser close and can never become an input or choose a route.
      try { native = await nativeCapture(data, path.join(directory, 'original'), 120, observer, true, 1, false, false, true, Boolean(reuse), clock, library, true); }
      catch (failure) { native = {status: 'fail', error: String(failure.stack ?? failure)}; }
      try {
        browser = await chromium.launch({...browserLaunchOptions(), headless: false});
        report.graphics = await inspectBrowserGraphics(browser);
        browserResult = await browserCapture(optionsObservedBrowser(browser, data, observed, retain, preferencesSchema), server, data,
          path.join(directory, 'browser'), true, true, false, clock, {sfxVolume: 1, musicVolume: 1});
      } catch (failure) { browserResult = {status: 'fail', failures: [String(failure.stack ?? failure)], frames: []}; }
      finally {
        try { await browser?.close(); } catch (failure) { observed.errors.push({type: 'cleanup', message: String(failure.stack ?? failure)}); }
        browser = undefined;
      }
      writeJson(path.join(directory, 'browser-status.json'), {status: browserResult.status, failures: browserResult.failures});
      try {
        if (native.deferredEvidence) native = await native.deferredEvidence();
        if (native.status === 'pass') native.publicClockValidation = validateNativeClockPlan(data, clock, native);
      } catch (failure) { native.status = 'fail'; native.error = String(failure.stack ?? failure); }
      writeJson(path.join(directory, 'native-status.json'), {status: native.status, provenance: native.provenance,
        error: native.error, publicClockValidation: native.publicClockValidation, optionsAudioOwnerObserver: native.manifest?.optionsAudioOwnerObserver});
      assert.deepEqual(native.manifest?.audioOptions, {sfxVolume: 1, musicVolume: 1});
      assert.equal(native.manifest?.optionsAudioOwnerObserver, 1, 'Actual Native Options instrumentation declaration required');
      const callsPath = path.join(directory, 'original/audio-trace/calls.jsonl');
      const calls = fs.readFileSync(callsPath, 'utf8').trim().split('\n').map(JSON.parse);
      assert.equal(calls[0].api, 'trace-start'); calls.slice(1).forEach((call, at) => assert.equal(call.seq, at + 1));
      typed = compareOptionsAudio({data, clock, native, browser: browserResult, observed, calls,
        nativeAudioDirectory: path.dirname(callsPath), metadata, scopedState, preferencesSchema});
      writeJson(path.join(directory, 'comparison.json'), typed);
      const pixels = [];
      for (const frame of browserResult.frames) {
        const original = native.manifest.frames.find(item => item.label === frame.label); assert.ok(original);
        const difference = pixelDifference(PNG.sync.read(fs.readFileSync(path.resolve(directory, 'original', original.path))), PNG.sync.read(fs.readFileSync(frame.path)));
        const {difference: image, ...values} = difference;
        fs.writeFileSync(path.join(directory, frame.label + '-difference.png'), PNG.sync.write(image));
        pixels.push({label: frame.label, ...values, status: values.exactRGBA ? 'pass' : 'fail'});
      }
      writeJson(path.join(directory, 'pixels.json'), {required: process.argv.includes('--require-exact-pixels'), checks: pixels});
      if (process.argv.includes('--require-exact-pixels')) assert.ok(pixels.length && pixels.every(row => row.status === 'pass'));
    } catch (failure) { error = {message: String(failure.message), stack: failure.stack}; }
    finally {
      try { await browser?.close(); } catch (failure) { observed.errors.push({type: 'cleanup', message: String(failure)}); }
      retain(observed);
      const result = {id: data.id, status: !error && native?.status === 'pass' && browserResult?.status === 'pass' && typed?.status === 'pass'
        && observed.errors.length === 0 && observed.failures.length === 0 ? 'pass' : 'fail', error,
        original: native?.status ?? 'missing', nativeError: native?.error, nativeProvenance: native?.provenance, publicClockValidation: native?.publicClockValidation,
        browser: browserResult?.status ?? 'missing', browserFailures: browserResult?.failures, checks: typed?.checks.length ?? 0,
        failures: typed?.checks.filter(row => row.status === 'fail').length ?? 0, diagnostics: typed?.diagnostics ?? [],
        observerErrors: observed.errors, observerFailures: observed.failures};
      report.cases.push(result); writeJson(path.join(directory, 'result.json'), result);
      writeJson(path.join(output, 'report.json'), report); console.log(JSON.stringify(result));
    }
  }
} catch (error) { fatal = {message: String(error.message), stack: error.stack}; }
finally {
  try { await server?.close(); } catch (error) { fatal ??= {message: String(error.message), stack: error.stack}; }
  try {
    const after = sourceIdentity(); writeJson(path.join(output, 'source-after.json'), after);
    assert.deepEqual(after, before, 'Executing Source changed during the suite');
    const inputsAfter = inputIdentity(); writeJson(path.join(output, 'input-after.json'), inputsAfter);
    assert.deepEqual(inputsAfter, inputsBefore, 'Authored inputs changed during the suite');
    report.librarySha256After = fileDigest(library);
    assert.equal(report.librarySha256After, observerIdentity.librarySha256, 'Actual observer library changed during the suite');
  } catch (error) { fatal ??= {message: String(error.message), stack: error.stack}; }
  report.error = fatal; report.status = !fatal && report.cases.length === bank.cases.length && report.cases.every(row => row.status === 'pass') ? 'pass' : 'fail';
  writeJson(path.join(output, 'report.json'), report);
}
console.log(JSON.stringify({status: report.status, report: path.join(output, 'report.json')}));
process.exitCode = report.status === 'pass' ? 0 : 1;
