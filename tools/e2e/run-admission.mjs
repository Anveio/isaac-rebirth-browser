// Actual original + production WASM admission clocks. No source elapsed time,
// counters, fade value, seed stream or hidden state is supplied to the browser.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, assertSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';
import { writeBrowserEvidence } from '../foundation/browser-evidence.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/run-admission', `${Date.now()}-${process.pid}`));
const sources = ['tools/e2e/run-admission.mjs', 'tools/original/run-admission-scenarios.py',
  'tools/original/run-admission-observe.py', 'tools/original/native-identity.py', 'tools/original/state-observer.py',
  'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-seeds.py',
  'tools/original/foundation-clock-verify.py', 'tools/original/foundation-random-seeds.py', 'tools/original/foundation-clock-scenarios.py',
  'tools/original/debug-run.py', 'tools/original/with-display.sh', 'tools/foundation/verify.mjs',
  'tools/foundation/browser-evidence.mjs', 'tools/foundation/browser.mjs', 'tools/foundation/production-server.mjs',
  'engine/src/run_admission.rs', 'engine/src/lib.rs', 'engine/src/phase.rs', 'engine/src/motion.rs',
  'web/main.ts', 'web/engine.ts', 'web/trace.ts'];
const pinSources = async () => Object.fromEntries(await Promise.all(sources.map(async path => [path, sha(await readFile(join(ROOT, path)))])));
const before = await pinSources();
await mkdir(output, { recursive: true });
const bank = value('--scenarios') ? JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8'))
  : JSON.parse((await run('python3', ['tools/original/run-admission-scenarios.py', '--count', value('--count') || '2'], {cwd: ROOT})).stdout);
assert.equal(bank.schema, 1);
assert.equal(bank.clockInput.presentationIntervalNanoseconds, 16_667_000);
assert.ok(Array.isArray(bank.cases) && bank.cases.length >= 4 && bank.cases.length <= 18);
assert.equal(new Set(bank.cases.map(c => c.name)).size, bank.cases.length);
await writeFile(join(output, 'scenarios.json'), JSON.stringify(bank, null, 2) + '\n');
for (const c of bank.cases) {
  assert.match(c.name, /^[a-zA-Z0-9_-]{1,40}$/);
  assert.match(c.code, /^[A-Z0-9]{8}$/);
  assert.ok(Number.isSafeInteger(c.runSeed) && c.runSeed > 0 && c.runSeed <= 0xffffffff);
  assert.ok(Number.isSafeInteger(c.bootEpochSeed) && c.bootEpochSeed >= 0 && c.bootEpochSeed <= 0xffffffff);
  assert.ok(c.beforeRunWait === 0 || c.beforeRunWait === 1);
  assert.ok(c.heldKey === null || ['d', 's'].includes(c.heldKey));
  assert.ok(Array.isArray(c.tape));
}
if (process.argv.includes('--prepare-only')) {
  console.log(JSON.stringify({status: 'prepared', cases: bank.cases.map(c => ({name: c.name, code: c.code})), output}));
  process.exit(0);
}
const production = await startProductionServer();
let browser;
const report = {schema: 1, status: 'running', scope: 'Live real original and production WASM full-only initial fade, owned public clocks, actual menu request, controls and absolute active Game age. No counter/time baseline or source state injection.',
  exclusions: ['Source585500/593660 stage-title and input-reset callbacks remain explicitly pending in runtime; this slice does not accept their visual or cache effects.',
    'Native introductory cutscene and absolute pre-Title system counter are omitted; raw source/browser counters and relative clocks remain observable.',
    'Unmapped room/NPC/background-effect constructors, rendering and complete gameplay parity remain separate.'],
  production: production.build, clockInput: bank.clockInput, gateSources: before, cases: []};
try {
  browser = await chromium.launch(browserLaunchOptions());
  report.graphics = await inspectBrowserGraphics(browser);
  for (const c of bank.cases) {
    const directory = join(output, c.name); await mkdir(directory, {recursive: true});
    const result = {...c, tape: undefined, status: 'fail', checks: 0, comparedPhases: 0, failures: []};
    report.cases.push(result);
    let page;
    try {
      const tapePath = join(directory, 'tape.json');
      await writeFile(tapePath, JSON.stringify(c.tape, null, 2) + '\n');
      await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
        '--seed', c.code, '--boot-epoch-seed', String(c.bootEpochSeed), '--step-ns', '16667000',
        '--tape', tapePath, '--state-observer', 'tools/original/run-admission-observe.py', '--observe-every-step',
        '--output', join(directory, 'original')], {cwd: ROOT, timeout: 600000, maxBuffer: 1024 * 1024});
      const nativePath = join(directory, 'original/manifest.json');
      const raw = await readFile(nativePath); const native = JSON.parse(raw);
      result.nativeManifest = {path: nativePath, bytes: raw.length, sha256: sha(raw)};
      assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      assert.deepEqual(native.tape, c.tape); assert.equal(native.seedValue, c.runSeed);
      assert.equal(native.bootEpochSeed, c.bootEpochSeed); assert.equal(native.seedReadbackMatchesCodec, true);
      result.originalInputProvenance = JSON.parse((await run('python3',
        ['tools/original/run-admission-scenarios.py', '--validate-original', nativePath], {cwd: ROOT})).stdout);
      assert.equal(result.originalInputProvenance.status, 'pass');
      const baseline = native.debugObservations.find(row => row.context.label === 'before-run-confirmation');
      assert.ok(baseline, 'Original must observe the actual pre-confirmation boundary');
      const nativeRows = native.debugObservations.filter(row => row.context.label.startsWith('step-')
        && row.context.presentedFrame > baseline.context.presentedFrame);
      assert.equal(nativeRows.length, 342, 'Every declared dense original presentation must be observed');
      assert.ok(nativeRows.every(row => row.observation.status === 'pass'
        && row.observation.nativeBinarySha256 === '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13'
        && row.observation.memoryWritten === false && row.observation.executableMappingsRead === false));
      const start = c.tape.findIndex(action => action.label === 'before-run-confirmation');
      const route = c.tape.slice(start + 1);
      const keyCodes = {Return: 'Enter', d: 'KeyD', s: 'KeyS'};
      const prefix = manualSeedCommands(c.code).slice(0, -4);
      if (c.beforeRunWait) prefix.push({type: 'tick', count: c.beforeRunWait});
      const expectedCommands = [...prefix, ...route.flatMap(action => action.type === 'step'
        ? [{type: 'tick', count: action.frames}] : action.type === 'key'
          ? [{type: 'key', code: keyCodes[action.key], down: action.down}] : [])];
      assert.ok(expectedCommands.every(command => command.type !== 'key' || command.code));
      page = await browser.newPage({viewport: {width: 960, height: 540}});
      const errors = []; page.on('pageerror', error => errors.push(String(error)));
      await page.goto(production.url + '/?manual=1'); await waitForReady(page);
      await page.evaluate(c => { window.game.pause(); window.game.reset(c.runSeed, {trace: true, bootEpochSeed: c.bootEpochSeed}); }, c);
      const deliver = async command => {
        if (command.type === 'tick') await page.evaluate(count => window.game.step(count), command.count);
        else if (command.type === 'key') await page.keyboard[command.down ? 'down' : 'up'](command.code);
        else { await page.keyboard.down(command.text.toLowerCase()); await page.keyboard.up(command.text.toLowerCase()); }
      };
      for (const command of prefix) await deliver(command);
      const seeded = await page.evaluate(() => window.game.snapshot());
      assert.equal(seeded.screen, 'character'); assertSeededRun(seeded, c.code, c.runSeed);
      assert.equal(seeded.runAdmission.requestedTimeAnchorMs, null,
        'Zero-tick public Enter does not initialize the actual MenuController request clock');
      const states = [];
      const check = (label, field, actual, expected) => {
        result.checks++;
        try { assert.deepEqual(actual, expected); }
        catch { result.failures.push({label, field, actual, expected}); }
      };
      for (const action of route) {
        if (action.type === 'capture') continue;
        await deliver(action.type === 'step' ? {type: 'tick', count: action.frames}
          : {type: 'key', code: keyCodes[action.key], down: action.down});
        if (action.type !== 'step') continue;
        const state = await page.evaluate(() => window.game.snapshot());
        const original = nativeRows[result.comparedPhases++];
        const source = original.observation;
        const label = `run-step-${result.comparedPhases}`;
        const actual = state.runAdmission;
        states.push({label, state, native: source, nativeContext: original.context});
        check(label, 'actual-system-playing-admission', state.screen === 'playing', source.systemState === 2);
        check(label, 'own-request-anchor-admitted', actual.requestedTimeAnchorMs !== null,
          source.requestedTimeAnchorMs !== baseline.observation.requestedTimeAnchorMs);
        if (actual.requestedTimeAnchorMs !== null && source.requestedTimeAnchorMs !== baseline.observation.requestedTimeAnchorMs) {
          const sourceNow = c.bootEpochSeed * 1000 + Math.floor(original.context.virtualElapsedNs / 1e6);
          check(label, 'own-relative-request-clock-ms', actual.clock.milliseconds - actual.requestedTimeAnchorMs,
            sourceNow - source.requestedTimeAnchorMs);
        }
        if (source.systemState === 2 && state.screen === 'playing') {
          check(label, 'initial-fade-f32', Math.fround(actual.initialFade), Math.fround(source.initialFade));
          check(label, 'fade-rate-f32', Math.fround(actual.fadeRate), Math.fround(source.fadeRate));
          check(label, 'primary-controls-enabled-raw-byte', state.player.controlsEnabled, source.primaryControlsEnabled);
          check(label, 'absolute-active-game-age', state.gameUpdateCount, source.gameAge);
          check(label, 'source-full-fast-parity', state.systemClock.systemCounter % 2, source.systemIterations % 2);
          check(label, 'player-world-position-f32', [Math.fround((state.player.x - 32) / .65), Math.fround((state.player.y + 46) / .65)], source.primaryPosition);
          check(label, 'player-world-velocity-f32', [Math.fround(state.player.vx / .65), Math.fround(state.player.vy / .65)], source.primaryVelocity);
        }
        assertSeededRun(state, c.code, c.runSeed);
      }
      const trace = await page.evaluate(() => window.game.trace());
      result.traceHistory = {truncated: trace.truncated, inputTraceTruncated: trace.inputTraceTruncated,
        stateHistoryTruncated: trace.stateHistoryTruncated, backendStateTruncated: trace.backendStateTruncated,
        backendEvents: trace.backendEvents.length};
      result.browserEvidence = await writeBrowserEvidence(join(directory, 'browser-states.json.gz'), {states, trace, expectedCommands});
      assertInputProvenance(trace, expectedCommands, c.runSeed, c.bootEpochSeed);
      assert.deepEqual(trace.clockInput, {initialTimestampMillis: c.bootEpochSeed * 1000,
        initialSubMillisecondNanoseconds: 0, presentationIntervalNanoseconds: 16_667_000});
      assert.equal(trace.backendStateTruncated, false);
      assert.ok(trace.backendEvents.every(event => event.state?.schema === 1));
      assert.deepEqual(errors, []);
      result.inputProvenance = {status: 'pass', commands: trace.commands.length, initialSeed: trace.initialSeed,
        bootEpochSeed: trace.bootEpochSeed, manualSeed: c.code, clockInput: trace.clockInput};
      result.status = result.failures.length ? 'fail' : 'pass';
    } catch (error) { result.error = String(error); }
    finally { await page?.close(); }
  }
} finally {
  await browser?.close(); await production.close();
  const after = await pinSources();
  report.sourceStability = {status: JSON.stringify(before) === JSON.stringify(after) ? 'pass' : 'fail', before, after};
  report.status = report.cases.every(c => c.status === 'pass') && report.sourceStability.status === 'pass' ? 'pass' : 'fail';
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
}
console.log(JSON.stringify({status: report.status, cases: report.cases.map(c => ({name: c.name, code: c.code,
  phases: c.comparedPhases, failures: c.failures.length, error: c.error})), report: join(output, 'report.json')}));
process.exitCode = report.status === 'pass' ? 0 : 1;
