// Registered TNT common-callback gate: canonical public transport and actual
// owners. The complete fixed+OS bank is declared before either game.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {chromium} from '@playwright/test';
import {nativeCapture, browserCapture, declarePublicClockPlan, validateNativeClockPlan, scopedState} from '../foundation/paired-game.mjs';
import {startProductionServer} from '../foundation/production-server.mjs';
import {browserLaunchOptions, inspectBrowserGraphics} from '../foundation/browser.mjs';
import {assertOriginalBinary, ORIGINAL_SHA} from './lib/original.mjs';
import {flag, selectCases} from './lib/cli.mjs';
import {tntObservedBrowser} from './tnt-observer.mjs';
import {compareTnt} from '../foundation/tnt-state-pair.mjs';
const ROOT = path.resolve(fileURLToPath(new URL('../..', import.meta.url)));
const read = name => JSON.parse(fs.readFileSync(name, 'utf8'));
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const fileHash = name => digest(fs.readFileSync(name));
const explicitProduction = flag('--production-directory'), recorded = flag('--scenarios'), reuse = flag('--reuse-original');
for (const name of ['--production-directory', '--scenarios', '--reuse-original', '--output'])
  assert.ok(flag(name) === undefined || flag(name).length > 0, `${name} requires a nonempty value`);
const production = explicitProduction ?? path.join(ROOT, 'dist');
const isReplay = recorded !== undefined, isReuse = reuse !== undefined;
const output = path.resolve(flag('--output') ?? path.join(ROOT, '.cache/e2e/tnt', `${Date.now()}-${process.pid}`));
assert.ok(path.isAbsolute(production), 'Immutable production directory must be absolute');
assert.ok(!isReplay || !process.argv.includes('--count'), 'Recorded replay must omit fresh --count');
assert.ok(!isReuse || isReplay, 'Native reuse requires a recorded public bank');
assert.ok(!process.argv.includes('--scenario-file'), 'Replay the complete recorded bank with --scenarios');
const count = flag('--count', '1');
assert.equal(count, '1', 'This bounded bank contains one fixed plus one first-draw OS case');
const sourcePaths = [
  'tools/original/tnt-scenarios.py', 'references/isaac/cases/tnt/right-entry-shoot-music1-v2.json',
  'tools/e2e/tnt.mjs', 'tools/e2e/tnt-observer.mjs', 'tools/foundation/tnt-state-pair.mjs', 'tools/original/tnt-observe.py',
  'tools/e2e/lib/game.mjs', 'tools/foundation/pixel-pairs.mjs',
  'tools/foundation/controls-browser-observe.mjs', 'tools/verification/input-catalog.json', 'references/isaac/foundation/combat/tapes.json',
  'tools/foundation/paired-game.mjs',
  'tools/foundation/save-continue-browser.mjs', 'tools/foundation/save-continue-guards.mjs',
  'tools/foundation/save-continue-movement.mjs',
  'tools/foundation/production-server.mjs', 'tools/foundation/browser.mjs',
  'tools/foundation/compare.mjs', 'tools/foundation/mt-draws.mjs', 'tools/foundation/native-evidence.mjs', 'tools/foundation/browser-evidence.mjs',
  'tools/foundation/native-journal-zstd.mjs', 'tools/original/native-journal-zstd.py',
  'tools/e2e/lib/cli.mjs', 'tools/e2e/lib/original.mjs', 'tools/e2e/lib/host-slots.mjs', 'tools/e2e/lib/audio-owners.mjs', 'tools/e2e/audio-api-observer.mjs',
  'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-clock-verify.py',
  'tools/original/foundation-paired-observer.py', 'tools/original/foundation-menu-observer.py', 'tools/original/foundation-combat-observe.py',
  'tools/original/foundation-pickup-observe.py', 'tools/original/familiar-observer.py', 'tools/original/audio-observer.py', 'tools/original/audio-trace.c',
  'tools/original/state-observer.py', 'tools/original/native-identity.py', 'tools/original/native-evidence.py', 'tools/original/with-display.sh',
  'tools/original/debug-run.py', 'tools/original/debug-config.json', 'tools/original/foundation-paired-scenarios.py',
  'tools/original/foundation-clock-scenarios.py', 'tools/original/foundation-bomb-scenario.py', 'tools/original/foundation-presentation-scenario.py',
  'tools/original/foundation-title-scenario.py', 'tools/original/foundation-combat-scenario.py', 'tools/original/foundation-random-seeds.py',
  'tools/original/foundation-seeds.py', 'web/audio-data.json', 'package-lock.json',
];
fs.mkdirSync(output, {recursive: true});
assert.deepEqual(fs.readdirSync(output), [], 'Use a fresh empty output directory; never overwrite partial evidence');
const save = (name, value) => fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + '\n');
const identity = () => Object.fromEntries(sourcePaths.map(name => [name, fileHash(path.join(ROOT, name))]));
const before = identity();
for (const name of sourcePaths) {
  const target = path.join(output, 'executing-source', name);
  fs.mkdirSync(path.dirname(target), {recursive: true}); fs.copyFileSync(path.join(ROOT, name), target);
}
save('source-before.json', before);
let bank, selected;
try {
  const args = isReplay ? ['--scenarios', path.resolve(recorded)] : ['--count', count];
  const generated = spawnSync('python3', ['tools/original/tnt-scenarios.py', ...args], {cwd: ROOT, encoding: 'utf8', maxBuffer: 4 * 1024 ** 2});
  fs.writeFileSync(path.join(output, 'scenario-helper.log'), String(generated.stdout ?? '') + String(generated.stderr ?? '') + (generated.error ? String(generated.error.stack ?? generated.error) : ''));
  assert.equal(generated.status, 0, 'Scenario helper failed; actual stdout/stderr retained');
  assert.equal(generated.signal, null, 'Scenario helper was interrupted');
  assert.equal(generated.error, undefined);
  bank = JSON.parse(generated.stdout);
  //Retain the complete validated bank even for a filtered replay. Runner replay
  //retains --case separately; filtered-out rows never receive OS credit.
  save('scenarios.json', bank);
  selected = selectCases(bank.cases, data => data.id);
  save('selection.json', {caseFilter: flag('--case') ?? null, ids: selected.map(data => data.id), total: bank.cases.length});
  assert.ok(!isReuse || selected.length === 1, 'Native reuse requires exactly one selected case');
  for (const data of selected) {
    assert.deepEqual(data.startup, {audioPreferences: {sfxVolume: 1, musicVolume: 1}, fullscreenPreferences: {fullscreen: false}, inputConfigs: {devices: []}});
    assert.deepEqual(data.declaredAudioOptions, {sfxVolume: 1, musicVolume: 1});
    declarePublicClockPlan(data);
  }
} catch (error) {
  save('report.json', {schema: 1, status: 'fail', error: String(error.stack ?? error), scope: 'Failed before compiler/game; Source and any authored public bank retained'});
  save('source-after.json', identity()); throw error;
}
if (process.argv.includes('--prepare-only')) {
  const after = identity(); save('source-after.json', after); assert.deepEqual(after, before);
  save('prepared.json', {status: 'PREPARED_ONLY', node: process.version, ids: selected.map(data => data.id), source: before}); process.exit(0);
}
const catalogue = read(path.join(ROOT, 'web/audio-data.json'));
const samplePaths = catalogue.sounds.find(row => row.id === 182).samples.map(row => row.path);
assert.ok(samplePaths.length === 3 && samplePaths.every(name => catalogue.sounds.filter(row => row.samples.some(sample => sample.path === name)).length === 1));
const samples = samplePaths.map(name => {
  const bytes = fs.readFileSync(path.join(ROOT, 'references/isaac/rebirth/resources', name));
  assert.equal(digest(bytes), catalogue.assets[name].sha256); assert.equal(bytes.toString('ascii', 0, 4), 'RIFF');
  let pcm, channels, sampleRate, format;
  for (let at = 12; at + 8 <= bytes.length;) {
    const size = bytes.readUInt32LE(at + 4), begin = at + 8; assert.ok(begin + size <= bytes.length);
    if (bytes.toString('ascii', at, at + 4) === 'fmt ') { format = bytes.readUInt16LE(begin); channels = bytes.readUInt16LE(begin + 2); sampleRate = bytes.readUInt32LE(begin + 4); assert.equal(bytes.readUInt16LE(begin + 14), 16); }
    if (bytes.toString('ascii', at, at + 4) === 'data') pcm = bytes.subarray(begin, begin + size);
    at = begin + size + (size & 1);
  }
  assert.ok(pcm && channels && sampleRate); assert.equal(format, 1);
  return {path: name, fileSha256: digest(bytes), pcmSha256: digest(pcm), pcmBytes: pcm.length, channels, sampleRate};
});
const report = {schema: 1, status: 'running', source: before, originalBinary: ORIGINAL_SHA, samples,
  cases: [], caseFilter: flag('--case'), randomCasesSelected: selected.filter(data => data.entropyDraws.length).length,
  scope: 'Nonrare TNT common callback: fuse, blast impulse, effect1/Wood27 and SFX182 actual request/backend/selected PCM. Transient18, rare pool, mixed order, Player damage, reentry, fullMT and RGBA remain open.'};
let server, fatal, library;
try {
  // The registered runner owns the suite slot; a second acquisition can deadlock.
  await assertOriginalBinary(); assert.deepEqual(identity(), before);
  server = await startProductionServer({directory: production}); report.build = server.build;
  library = path.join(output, 'audio-trace.so');
  try {
    const result = execFileSync('cc', ['-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared',
      'tools/original/audio-trace.c', '-o', library, '-ldl', '-pthread'], {cwd: ROOT, encoding: 'utf8', stdio: 'pipe'});
    fs.writeFileSync(path.join(output, 'audio-observer-compile.log'), result);
  } catch (error) {
    fs.writeFileSync(path.join(output, 'audio-observer-compile.log'), String(error.stdout ?? '') + String(error.stderr ?? '') + String(error.stack ?? error)); throw error;
  }
  const librarySha256 = fileHash(library); report.librarySha256 = librarySha256;
  for (const [index, data] of selected.entries()) {
    const directory = path.join(output, `${String(index).padStart(2, '0')}-${data.id}`);
    fs.mkdirSync(directory, {recursive: false});
    const caseSave = (name, value) => fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + '\n');
    const clock = declarePublicClockPlan(data);
    caseSave('case.json', data); caseSave('tape.json', data.tape); caseSave('public-clock-plan.json', clock);
    const item = {schema: 1, id: data.id, status: 'running', directory,
      observerIdentity: {schema: 1, kind: 'tnt-owner-v1', librarySha256, source: before,
        caseSha256: fileHash(path.join(directory, 'case.json')), clockSha256: fileHash(path.join(directory, 'public-clock-plan.json'))}};
    caseSave('observer-identity.json', item.observerIdentity);
    let native, browser, browserResult = {status: 'fail', frames: []};
    const observed = {errors: []};
    const retain = value => fs.writeFileSync(path.join(directory, 'actual-browser-evidence.json.gz'), gzipSync(JSON.stringify(value)));
    try {
      assert.deepEqual(identity(), before, 'Executing Source changed before this case');
      assert.equal(fileHash(library), librarySha256);
      const free = fs.statfsSync(output);
      assert.ok(free.bavail * free.bsize >= 4 * 1024 ** 3, 'Native requires unchanged 4 GiB reserve after private production copy');

      if (isReuse) {
        const target = path.resolve(reuse), parent = path.dirname(target), previous = read(path.join(parent, 'native-status.json'));
        assert.deepEqual(read(path.join(parent, 'case.json')), data); assert.deepEqual(read(path.join(parent, 'public-clock-plan.json')), clock);
        assert.deepEqual(read(path.join(parent, 'observer-identity.json')), item.observerIdentity);
        assert.equal(previous.status, 'pass'); assert.equal(previous.provenance.status, 'pass'); assert.equal(previous.publicClockValidation.status, 'pass');
        fs.symlinkSync(target, path.join(directory, 'original'), 'dir');
      }
      try { native = await nativeCapture(data, path.join(directory, 'original'), 120, path.join(ROOT, 'tools/original/tnt-observe.py'), true, 1, false, false, true, isReuse, clock, library, false); }
      catch (error) { native = {status: 'fail', error: String(error.stack ?? error)}; }
      try {
        browser = await chromium.launch({...browserLaunchOptions(), headless: false}); item.graphics = await inspectBrowserGraphics(browser);
        browserResult = await browserCapture(tntObservedBrowser(browser, observed, retain), server, data, path.join(directory, 'browser'), true, true, false, clock, data.startup.audioPreferences);
      } catch (error) { observed.errors.push({type: 'browserCapture', message: String(error.stack ?? error)}); }
      finally {
        try { await browser?.close(); }
        catch (error) { observed.errors.push({type: 'browserClose', message: String(error.stack ?? error)}); }
        browser = undefined;
      }
      if (native.deferredEvidence) {
        try { native = await native.deferredEvidence(); }
        catch (error) { native = {...native, status: 'fail', error: String(error.stack ?? error)}; }
      }
      try { if (native.status === 'pass') native.publicClockValidation = validateNativeClockPlan(data, clock, native); }
      catch (error) { native.status = 'fail'; native.error = String(error.stack ?? error); }
      item.nativeStatus = {status: native.status, error: native.error, provenance: native.provenance, publicClockValidation: native.publicClockValidation};
      item.browserStatus = {status: browserResult.status, failures: browserResult.failures};
      caseSave('native-status.json', item.nativeStatus); caseSave('browser-status.json', item.browserStatus);
      let calls = []; const callsPath = path.join(directory, 'original/audio-trace/calls.jsonl');
      try { calls = fs.readFileSync(callsPath, 'utf8').trim().split('\n').map(JSON.parse); assert.equal(calls[0].api, 'trace-start'); calls.slice(1).forEach((row, index) => assert.equal(row.seq, index + 1)); }
      catch (error) { observed.errors.push({type: 'nativeAudioJournal', message: String(error.stack ?? error)}); }
      item.comparison = compareTnt({data, clock, native, browser: browserResult, observed, calls, samples, nativeAudioDirectory: path.dirname(callsPath), scopedState});
      caseSave('comparison.json', item.comparison);
      item.status = item.comparison.status;
    } catch (error) { item.status = 'fail'; item.error = String(error.stack ?? error); }
    finally {
      try { await browser?.close(); } catch (error) { item.status = 'fail'; item.cleanupError = String(error.stack ?? error); }
      item.nativeStatus ??= {status: native?.status ?? 'not-run', error: native?.error, directory: native?.directory};
      item.browserStatus ??= {status: browserResult.status, failures: browserResult.failures};
      if (observed.errors.length) item.status = 'fail';
      retain(observed); caseSave('native-status.json', item.nativeStatus); caseSave('browser-status.json', item.browserStatus);
      caseSave('result.json', item);
      const summary = {id: data.id, status: item.status, checks: item.comparison?.checks.length ?? 0,
        failures: item.comparison?.checks.filter(row => row.status === 'fail').length ?? 0, diagnostics: item.comparison?.diagnostics ?? [],
        original: item.nativeStatus, browser: item.browserStatus, observerErrors: observed.errors, error: item.error, directory};
      report.cases.push(summary); save('report.json', report); console.log(JSON.stringify(summary));
    }
  }
} catch (error) { fatal = String(error.stack ?? error); }
finally {
  try { await server?.close(); } catch (error) { fatal ??= String(error.stack ?? error); }
  try {
    const after = identity(); save('source-after.json', after); assert.deepEqual(after, before);
    if (library && report.librarySha256) {
      report.librarySha256After = fileHash(library); assert.equal(report.librarySha256After, report.librarySha256, 'Actual observer library changed');
    }
    for (const sample of samples) assert.equal(fileHash(path.join(ROOT, 'references/isaac/rebirth/resources', sample.path)), sample.fileSha256);
    await assertOriginalBinary();
  } catch (error) { fatal ??= String(error.stack ?? error); }
  report.error = fatal;
  report.status = !fatal && report.cases.length === selected.length && report.cases.every(row => row.status === 'pass') ? 'pass' : 'fail';
  save('report.json', report);
}
console.log(JSON.stringify({status: report.status, report: path.join(output, 'report.json')}));
process.exitCode = report.status === 'pass' ? 0 : 1;
