// Public native/menu inputs paired with the production WASM and Web Audio output.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash, randomInt } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { waitForReady } from '../foundation/verify.mjs';
import { installBrowserAudioObserver } from './audio-api-observer.mjs';
import { ORIGINAL_SHA } from './lib/original.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const value = flag => { const i = process.argv.indexOf(flag); return i < 0 ? undefined : process.argv[i + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/audio', String(Date.now())));
const count = Number(value('--count') || 2);
assert.ok(Number.isInteger(count) && count >= 1 && count <= 16);
await mkdir(output, { recursive: true });
const generated = JSON.parse((await run('python3', ['tools/original/foundation-seeds.py', '--count', String(count)], { cwd: ROOT })).stdout);
let cases = [
  { name: 'audible-restarts', code: 'QSYVKMSE', bootEpochSeed: 1700000000, volume: 1, taps: ['Left', 'Left', 'Right', 'Right'] },
  { name: 'muted-admissions', code: 'ZG800TRJ', bootEpochSeed: 0, volume: 0, taps: ['Right', 'Left', 'Right', 'Left'] },
  ...generated.vectors.map((vector, i) => ({ name: `random-${i}`, code: vector.code.replaceAll(' ', ''), bootEpochSeed: randomInt(0x100000000), volume: 1,
    taps: Array.from({ length: 4 }, () => randomInt(2) ? ['Left', 'Right'] : ['Right', 'Left']).flat() })),
];
if (value('--scenarios')) cases = JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')).cases;
assert.ok(Array.isArray(cases) && cases.length >= 1 && cases.length <= 18);
assert.equal(new Set(cases.map(c => c.name)).size, cases.length);
for (const c of cases) {
  assert.ok(/^[a-zA-Z0-9_-]{1,32}$/.test(c.name) && /^[a-zA-Z0-9]{8}$/.test(c.code));
  assert.ok(Number.isInteger(c.bootEpochSeed) && c.bootEpochSeed >= 0 && c.bootEpochSeed <= 0xffffffff);
  assert.ok(c.volume === 0 || c.volume === 1);
  assert.ok(Array.isArray(c.taps) && c.taps.length >= 2 && c.taps.length <= 32 && c.taps.every(key => ['Left', 'Right'].includes(key)));
}
const sources = ['tools/e2e/audio.mjs', 'tools/original/audio-observer.py', 'tools/original/audio-trace.c', 'tools/original/foundation-clock-run.py',
  'tools/e2e/audio-api-observer.mjs', 'tools/original/foundation-clock-gate.c', 'tools/original/state-observer.py', 'tools/original/foundation-seeds.py', 'tools/foundation/production-server.mjs', 'tools/foundation/browser.mjs'];
const sourceHashes = async () => Object.fromEntries(await Promise.all(sources.map(async path => [path, sha(await readFile(join(ROOT, path)))])));
const catalogue = JSON.parse(await readFile(join(ROOT, 'web/audio-data.json')));
const hashes = await sourceHashes();
assert.equal(sha(await readFile(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64'))), ORIGINAL_SHA);
await writeFile(join(output, 'scenarios.json'), JSON.stringify({ schema: 1, entropy: 'OS secrets run seeds and Node crypto boot epochs/menu taps', cases }, null, 2) + '\n');
await run('cc', ['-std=c11', '-Wall', '-Wextra', '-Werror', '-fPIC', '-shared', 'tools/original/audio-trace.c', '-o', join(output, 'audio-trace.so'), '-ldl', '-pthread'], { cwd: ROOT });
const observerLibrarySha256 = sha(await readFile(join(output, 'audio-trace.so')));

function tapes(taps, code) {
  const native = [ { type: 'step', frames: 180 }, { type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 },
    { type: 'key', key: 'Return', down: false }, { type: 'step', frames: 9 }, { type: 'capture', label: 'title-ready' } ];
  const browser = [{ type: 'capture', label: 'title-ready' }];
  const rest = [{ type: 'step', frames: 63 }, { type: 'capture', label: 'title' }];
  for (const label of ['profile', 'main', 'character']) rest.push({ type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 },
    { type: 'key', key: 'Return', down: false }, { type: 'step', frames: 72 }, { type: 'capture', label });
  taps.forEach((key, i) => rest.push({ type: 'key', key, down: true }, { type: 'step', frames: 1 }, { type: 'key', key, down: false },
    { type: 'step', frames: 1 }, { type: 'capture', label: `tap-${i}` }));
  // Complete the actual public seed transport; the clock runner checks native
  // run-seed readback even when this gate's audio comparisons concern menus.
  rest.push({ type: 'key', key: 'Tab', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Tab', down: false }, { type: 'step', frames: 108 });
  for (const key of code.toLowerCase()) rest.push({ type: 'key', key, down: true }, { type: 'step', frames: 12 },
    { type: 'key', key, down: false }, { type: 'step', frames: 6 });
  for (const [label, settle] of [['character-seeded', 72], ['gameplay-start', 300]]) rest.push({ type: 'key', key: 'Return', down: true },
    { type: 'step', frames: 12 }, { type: 'key', key: 'Return', down: false }, { type: 'step', frames: settle }, { type: 'capture', label });
  return { native: native.concat(rest), browser: browser.concat(rest) };
}

function sampleStarts(calls) {
  const buffers = new Map(), settings = new Map(), starts = [];
  for (const call of calls) {
    if (call.api === 'alBufferData') buffers.set(call.buffer, call);
    if (call.api === 'alSourcei' || call.api === 'alSourcef') {
      if (!settings.has(call.source)) settings.set(call.source, {});
      settings.get(call.source)[call.parameter] = call.value;
    }
    if (call.api === 'alSourcePlay' && call.clockDomain === 'gate-main-thread' && call.source >= 1 && call.source <= 16) {
      const params = settings.get(call.source), upload = params && buffers.get(params[0x1009]);
      assert.ok(upload && !upload.truncated && upload.bytes === upload.artifactBytes, 'Every actual sample start has complete PCM evidence');
      starts.push({ call, upload, looping: params[0x1007] === 1, pitch: params[0x1003], gain: params[0x100a] });
    }
  }
  return starts;
}

// Forward the actual browser audio APIs and observe their submitted buffers.
// This is independent of AudioRuntime's diagnostics and never changes inputs,
// buffers, parameters, admission, random state, or output scheduling.

const production = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = { schema: 1, status: 'running', originalBinarySha256: ORIGINAL_SHA, observerLibrarySha256, production: production.build, sourceHashes: hashes,
  graphics: await inspectBrowserGraphics(browser), scope: 'All 169 configured SFX and 303 sample wrappers; actual forward menu/character sound admissions and native OpenAL/browser Web Audio PCM starts. Complete shared MT is gated at Title/profile; later Main initialization and run random routing remain explicit diagnostics. Stable character sound admissions must each consume exactly one shared draw. Music/gameplay SFX callers are separate work.', cases: [] };
try {
  for (const scenario of cases) {
    const directory = join(output, scenario.name); await mkdir(directory, { recursive: true });
    const tape = tapes(scenario.taps, scenario.code); await writeFile(join(directory, 'tape.json'), JSON.stringify(tape.native, null, 2) + '\n');
    await writeFile(join(directory, 'browser-tape.json'), JSON.stringify(tape.browser, null, 2) + '\n');
    const result = { ...scenario, status: 'running', checks: [], diagnostics: [] }; report.cases.push(result);
    let page;
    const check = (field, actual, expected) => result.checks.push({ field, actual, expected, status: JSON.stringify(actual) === JSON.stringify(expected) ? 'pass' : 'fail' });
    try {
      await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py', '--seed', scenario.code,
        '--boot-epoch-seed', String(scenario.bootEpochSeed), '--tape', join(directory, 'tape.json'), '--state-observer', 'tools/original/audio-observer.py',
        '--sfx-volume', String(scenario.volume), '--music-volume', '1', '--audio-trace', join(output, 'audio-trace.so'), '--output', join(directory, 'original')],
      { cwd: ROOT, timeout: 180000, maxBuffer: 1024 * 1024 });
      const nativeBytes = await readFile(join(directory, 'original/manifest.json')), native = JSON.parse(nativeBytes);
      result.nativeManifestSha256 = sha(nativeBytes); assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      assert.equal(native.bootEpochSeed, scenario.bootEpochSeed); assert.equal(native.seedReadbackMatchesCodec, true);
      const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
      const origin = observations.get('title-ready').systemCounter;
      page = await browser.newPage(); const errors = []; page.on('pageerror', error => errors.push(String(error)));
      await page.addInitScript(installBrowserAudioObserver);
      await page.goto(production.url + '/?manual=1'); await waitForReady(page);
      await page.evaluate(({ seed, boot, volume }) => { window.game.pause(); window.game.reset(seed, { bootEpochSeed: boot, trace: true });
        window.game.command({ type: 'audioVolume', sfx_volume: volume }); }, { seed: native.seedValue, boot: scenario.bootEpochSeed, volume: scenario.volume });
      check('initial autoplay suspended', (await page.evaluate(() => window.game.audio())).contextState, 'suspended');
      await page.locator('#game').click();
      await page.waitForFunction(() => window.game.audio().contextState === 'running');
      const states = []; let previousTap, intendedSeedEntry = false;
      const intendedPressed = new Set(), expectedCommands = [{ type: 'audioVolume', sfx_volume: scenario.volume }];
      for (const action of tape.browser) {
        if (action.type === 'step') { await page.evaluate(frames => window.game.step(frames), action.frames); expectedCommands.push({ type: 'tick', count: action.frames }); }
        else if (action.type === 'key') {
          const code = ({ Return: 'Enter', Tab: 'Tab', Left: 'ArrowLeft', Right: 'ArrowRight' })[action.key] ||
            (/^[0-9]$/.test(action.key) ? 'Digit' + action.key : 'Key' + action.key.toUpperCase());
          await page.keyboard[action.down ? 'down' : 'up'](({ Return: 'Enter', Tab: 'Tab', Left: 'ArrowLeft', Right: 'ArrowRight' })[action.key] || action.key);
          // Derive the expectation from the declared public seed-mode tape.
          // A failed Tab transition must not rewrite intended text into keys.
          if (intendedSeedEntry && /^[a-z0-9]$/i.test(action.key)) {
            if (action.down) expectedCommands.push({ type: 'text', text: action.key.toUpperCase() });
            else if (intendedPressed.delete(code)) expectedCommands.push({ type: 'key', code, down: false });
          } else {
            expectedCommands.push({ type: 'key', code, down: action.down });
            if (action.down) intendedPressed.add(code); else intendedPressed.delete(code);
          }
          if (action.down && action.key === 'Tab') intendedSeedEntry = true;
          else if (action.down && action.key === 'Return' && intendedSeedEntry) intendedSeedEntry = false;
        }
        else {
          const actual = await page.evaluate(() => ({ state: window.game.snapshot(), audio: window.game.audio() }));
          states.push({ label: action.label, ...actual });
          const expected = observations.get(action.label); assert.equal(expected.status, 'pass');
          assert.equal(expected.nativeBinarySha256, ORIGINAL_SHA); assert.equal(expected.memoryWritten, false); assert.equal(expected.executableMappingsRead, false);
          check(`${action.label}: sample loads`, actual.audio.loadedSamples, 301);
          check(`${action.label}: browser voice capacity`, actual.audio.voiceCapacity, 16);
          check(`${action.label}: unique playing backend ownership`, new Set(actual.audio.activeVoices.map(voice => voice.path)).size, actual.audio.activeVoices.length);
          check(`${action.label}: source wrappers`, [expected.configuredSounds, expected.sampleReferences, expected.pathMismatches], [169, 303, []]);
          check(`${action.label}: system counter`, actual.state.systemClock.systemCounter, expected.systemCounter - origin);
          if (action.label === 'character-seeded' || action.label === 'gameplay-start') check(`${action.label}: manual public seed committed`,
            [actual.state.seededRun, actual.state.seedValid, actual.state.seedText.replaceAll(' ', '').toUpperCase(),
              actual.state.runSeed.replaceAll(' ', '').toUpperCase(), actual.state.runSeedValue],
            [true, true, scenario.code.toUpperCase(), scenario.code.toUpperCase(), native.seedValue]);
          if (action.label === 'title-ready') {
            check('all configured sounds loaded with authored samples', Object.entries(actual.state.audio.slots).map(([id, slot]) =>
              [Number(id), slot.loaded, slot.samples.length, slot.samples]), expected.slots.map(slot =>
              [slot.id, slot.loaded, slot.count, slot.samples.map(sample => ({ looping: sample.loop, gain: sample.gain, pitch: sample.pitch }))]));
            check('owned public native/browser volume', [actual.state.audio.sfxVolume, expected.globalSfxVolume], [scenario.volume, scenario.volume]);
          }
          const mtComparison = { field: `${action.label}: complete shared MT`, actual: actual.state.bootLifecycle.shared,
            expected: { words: expected.mtState, index: expected.mtIndex } };
          if (['title-ready', 'title', 'profile'].includes(action.label)) check(mtComparison.field, mtComparison.actual, mtComparison.expected);
          else result.diagnostics.push({ ...mtComparison, status: JSON.stringify(mtComparison.actual) === JSON.stringify(mtComparison.expected) ? 'pass' : 'fail',
            scope: 'Post-Title Main/menu and run random lifecycle is not implemented by this scoped sound patch; mismatch retained without injecting source state or skipping draws.' });
          if (action.label === 'character' || action.label.startsWith('tap-')) {
            if (previousTap) check(`${action.label}: exactly one admitted shared draw`,
              [(actual.state.bootLifecycle.shared.index - previousTap.actual + 624) % 624, (expected.mtIndex - previousTap.expected + 624) % 624], [1, 1]);
            previousTap = { actual: actual.state.bootLifecycle.shared.index, expected: expected.mtIndex };
          }
          for (const id of [8, 194, 195]) {
            const slot = expected.slots.find(slot => slot.id === id), observed = actual.state.audio.slots[id];
            // Source virgin lastTick bytes are not an event. nextAllowed is its initialized discriminator.
            check(`${action.label}: sound${id} admission`, [observed.lastTick, observed.nextAllowed, observed.lastVolume],
              [slot.nextAllowed < 0 ? null : slot.lastTick - (origin >> 1), slot.nextAllowed < 0 ? -1 : slot.nextAllowed - (origin >> 1), slot.lastVolume]);
            check(`${action.label}: sound${id} sample settings`, observed.samples,
              slot.samples.map(sample => ({ looping: sample.loop, gain: sample.gain, pitch: sample.pitch })));
          }
        }
      }
      const actualAudio = await page.evaluate(() => window.game.audio());
      const actualApi = await page.evaluate(async () => { await Promise.all(window.__audioApi.pending); return { events: window.__audioApi.events, truncated: window.__audioApi.truncated }; });
      // All authored base SFX are mono PCM16; music streams are stereo and
      // have their own live controller/queue acceptance cohort.
      const actualApiStarts = actualApi.events.filter(event => event.type === 'start' && event.channels === 1);
      check('actual browser output API trace bounded', actualApi.truncated, false);
      const traceBytes = await readFile(join(directory, 'original/audio-trace/calls.jsonl')); result.nativeTraceSha256 = sha(traceBytes);
      const calls = traceBytes.toString().trim().split('\n').map(line => JSON.parse(line));
      check('actual original process identity in all OpenAL calls', [...new Set(calls.map(call => call.pid))], [observations.get('title-ready').originalPid]);
      const starts = sampleStarts(calls);
      check('native 16voice allocation', calls.find(call => call.api === 'alGenSources').count, 16);
      check('native sample starts', starts.length, 3 + scenario.taps.length);
      check('browser sample starts', actualAudio.events.filter(event => event.output === 'started').length, starts.length);
      check('actual browser Web Audio starts', actualApiStarts.length, starts.length);
      check('browser output trace bounded', actualAudio.truncated, false);
      check('browser context sample rate', actualAudio.contextSampleRate, 44100);
      for (let i = 0; i < starts.length; i++) {
        const actual = actualAudio.events.filter(event => event.output === 'started')[i], expected = starts[i];
        assert.ok(actual, 'Actual browser sample start is required');
        const pcmBytes = await readFile(join(directory, 'original/audio-trace', expected.upload.pcmArtifact));
        const submitted = actualApiStarts[i]; assert.ok(submitted, 'Actual Web Audio submitted buffer is required');
        check(`start${i}: actual audio graph reaches device output`, submitted.reachesDestination, true);
        check(`start${i}: actual submitted PCM and parameters`, [submitted.pcmSha256, submitted.looping, submitted.pitch, submitted.gain],
          [sha(pcmBytes), expected.looping, expected.pitch, expected.gain]);
        check(`start${i}: runtime output diagnostic`, [actual.pcmSha256, actual.looping, actual.pitch, actual.gain],
          [submitted.pcmSha256, submitted.looping, submitted.pitch, submitted.gain]);
        check(`start${i}: native PCM format`, [expected.upload.format, expected.upload.rate, pcmBytes.length],
          [0x1101, 44100, catalogue.assets[actual.path].frames * 2]);
        check(`start${i}: actual browser PCM shape`, [submitted.channels, submitted.sampleRate, submitted.frames],
          [1, 44100, catalogue.assets[actual.path].frames]);
      }
      check('browser runtime errors', errors, []);
      check('browser actual public seed readback', (await page.evaluate(() => window.game.snapshot())).runSeedValue, native.seedValue);
      const wasmTrace = await page.evaluate(() => window.game.trace());
      check('actual reset/run and boot inputs recorded', [wasmTrace.initialSeed, wasmTrace.bootEpochSeed], [native.seedValue, scenario.bootEpochSeed]);
      check('all public inputs and intervals reached production WASM', wasmTrace.commands, expectedCommands);
      check('actual WASM input trace bounded', [wasmTrace.enabled, wasmTrace.inputTraceTruncated], [true, false]);
      result.traceHistory = { truncated: wasmTrace.truncated, inputTraceTruncated: wasmTrace.inputTraceTruncated,
        stateHistoryTruncated: wasmTrace.stateHistoryTruncated, retainedStates: wasmTrace.states.length,
        stateEncodedBytes: wasmTrace.stateEncodedBytes, maxStateEncodedBytes: wasmTrace.maxStateEncodedBytes };
      await writeFile(join(directory, 'browser.json'), JSON.stringify({ states, output: actualAudio, api: actualApi, trace: wasmTrace }, null, 2) + '\n');
      result.status = result.checks.every(check => check.status === 'pass') ? 'pass' : 'fail';
      console.log(JSON.stringify({ case: result.name, status: result.status, checks: result.checks.length, failed: result.checks.filter(check => check.status === 'fail').map(check => check.field) }));
    } catch (error) { result.status = 'fail'; result.error = String(error); }
    finally { if (page) await page.close(); await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n'); }
  }
  assert.deepEqual(await sourceHashes(), hashes, 'Live gate sources must remain immutable across all cases');
  report.status = report.cases.every(c => c.status === 'pass') ? 'pass' : 'fail';
} catch (error) { report.status = 'fail'; report.error = String(error); }
finally { await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n'); await browser.close(); await production.close(); }
console.log(JSON.stringify({ status: report.status, cases: report.cases.length, checks: report.cases.flatMap(c => c.checks).length, report: join(output, 'report.json') }));
process.exitCode = report.status === 'pass' ? 0 : 1;
