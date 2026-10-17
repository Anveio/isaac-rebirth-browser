// Natural authored pickups in both live games; source observations never enter WASM.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, assertSeededRun, enterSeededRun, manualSeedCommands, resetBootElapsed, waitForReady } from '../foundation/verify.mjs';
import { flag, selectCases } from './lib/cli.mjs';
import { snapshot, stepSnapshot, readTrace } from './lib/game.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';
import { createPublicWasmRuntime, createPublicWasmJournal } from '../foundation/composition-wasm-public.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const wasmNode = value('--wasm-node'), wasmSha = value('--wasm-sha');
const selectedName = flag('--case'), reusedOriginal = value('--reuse-original');
assert.equal(Boolean(wasmNode), Boolean(wasmSha), 'Explicit WASM and expected SHA are both required');
assert.ok(!reusedOriginal || wasmNode && selectedName && value('--scenarios'),
  'One retained Original requires the functional backend and a declared bank/case');
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/pickups', String(Date.now())));
await mkdir(output, { recursive: true });
let scenarios;
if (value('--scenarios')) scenarios = JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8'));
else {
  const result = await run('python3', ['tools/original/pickup-scenarios.py', '--count', value('--count') || '2'], { cwd: ROOT, timeout: 180000 });
  scenarios = JSON.parse(result.stdout);
}
assert.equal(scenarios.schema, 1);
assert.equal(scenarios.bootEpochSeed, 1700000000);
// A bank recorded under --case holds only the cases that ran.
if (!scenarios.caseFilter) assert.ok(scenarios.cases.length >= 4 && scenarios.cases.length <= 11);
assert.equal(new Set(scenarios.cases.map(item => item.name)).size, scenarios.cases.length);
for (const item of scenarios.cases) {
  assert.match(item.name, /^[a-zA-Z0-9_-]{1,32}$/);
  assert.match(item.code, /^[A-Z0-9]{8}$/);
  assert.ok(Number.isInteger(item.seedValue) && item.seedValue > 0 && item.seedValue <= 0xffffffff);
  assert.ok([0, 1].includes(item.delay));
}
scenarios.cases = selectCases(scenarios.cases, item => item.name);
if (selectedName) scenarios.caseFilter = selectedName;
assert.ok(!reusedOriginal || scenarios.cases.length === 1, 'One retained Original requires --case selecting exactly one case');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
await assertOriginalBinary();
await writeFile(join(output, 'scenarios.json'), JSON.stringify(scenarios, null, 2) + '\n');
const server = wasmNode ? null : await startProductionServer({ directory: value('--production-directory') });
const browser = wasmNode ? null : await chromium.launch(browserLaunchOptions());
const report = {
  schema: 1, status: 'running', originalBinarySha256: ORIGINAL_SHA, production: server?.build ?? null,
  graphics: browser ? await inspectBrowserGraphics(browser) : null,
  captureBackend: wasmNode ? 'wasm-node' : 'browser',
  ...(wasmNode ? { wasmArtifact: { path: resolve(wasmNode), sha256: wasmSha },
    unavailable: ['Browser DOM inputs', 'Renderer and PNGs', 'Audio backend and PCM'] } : {}),
  caseFilter: scenarios.caseFilter, allDeclaredCasesRetained: true, declaredOrder: scenarios.cases.map(item => item.name),
  scope: 'Live public-seed natural coin-room admission, seeds/subtypes, full-phase animation/contact/collection, inventory and actor cleanup. Unsupported typed selectors remain with their owning systems.',
  cases: [],
};
try {
  for (const item of scenarios.cases) {
    const directory = join(output, item.name);
    await mkdir(directory, { recursive: true });
    const caseResult = { ...item, cohort: item.cohort || 'hazard-free-coin', status: 'running', failures: [] };
    report.cases.push(caseResult);
    try {
    const tapeResult = await run('python3', ['tools/original/pickup-scenarios.py', '--seed', item.code, '--delay', String(item.delay)], { cwd: ROOT });
    await writeFile(join(directory, 'tape.json'), tapeResult.stdout);
    const originalDirectory = reusedOriginal ? resolve(reusedOriginal) : join(directory, 'original');
    const protectedPaths = [], sourceDigests = {};
    const pinInput = async path => {
      const absolute = resolve(path); protectedPaths.push(absolute);
      sourceDigests[absolute] = digest(await readFile(absolute));
    };
    if (reusedOriginal) {
      const priorRoot = resolve(originalDirectory, '../..');
      const priorBankPath = join(priorRoot, 'scenarios.json'), priorReportPath = join(priorRoot, 'report.json');
      const priorBank = JSON.parse(await readFile(priorBankPath));
      const priorCase = priorBank.cases.filter(entry => entry.name === item.name);
      assert.deepEqual(priorCase, [item], 'Retained Native declaration must equal the requested case');
      assert.deepEqual(JSON.parse(await readFile(join(originalDirectory, '..', 'tape.json'))),
        JSON.parse(tapeResult.stdout), 'Retained Native tape must equal the current canonical public tape');
      const priorReport = JSON.parse(await readFile(priorReportPath));
      const priorResult = priorReport.cases.filter(entry => entry.name === item.name);
      assert.equal(priorReport.status, 'pass');
      assert.equal(priorResult.length, 1); assert.equal(priorResult[0].status, 'pass');
      await pinInput(priorBankPath); await pinInput(priorReportPath);
      await pinInput(join(originalDirectory, '..', 'tape.json'));
      caseResult.reusedNativeExpectedRawSha256 = priorResult[0].nativeManifestSha256;
    } else {
      await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
        '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
        '--state-observer', 'tools/original/foundation-pickup-observe.py', '--output', originalDirectory],
        { cwd: ROOT, timeout: 180000, maxBuffer: 2 * 1024 * 1024 });
    }
    const nativeManifest = join(originalDirectory, 'manifest.json');
    const manifestBytes = await readFile(nativeManifest);
    if (reusedOriginal) assert.equal(digest(manifestBytes), caseResult.reusedNativeExpectedRawSha256,
      'Retained Native bytes must equal the independently recorded successful capture');
    const native = JSON.parse(manifestBytes);
    if (wasmNode) {
      const checked = await run('python3', ['tools/original/foundation-paired-scenarios.py',
        '--validate-original', nativeManifest], { cwd: ROOT, maxBuffer: 8 * 1024 * 1024 });
      const provenance = JSON.parse(checked.stdout);
      assert.equal(provenance.status, 'pass'); assert.deepEqual(provenance.failures, []);
      assert.equal(native.seedValue, item.seedValue); assert.equal(native.seedReadbackMatchesCodec, true);
      for (const path of [wasmNode, fileURLToPath(import.meta.url), nativeManifest,
        join(directory, 'tape.json'), value('--scenarios') || join(output, 'scenarios.json'),
        join(ROOT, 'tools/foundation/composition-wasm-public.mjs'),
        join(ROOT, 'tools/original/pickup-scenarios.py'),
        join(ROOT, 'tools/original/foundation-paired-scenarios.py'),
        join(ROOT, 'tools/original/foundation-clock-verify.py')]) await pinInput(path);
      assert.equal(sourceDigests[resolve(wasmNode)], wasmSha);
    }
    assert.equal(native.status, 'pass');
    assert.deepEqual(native.failures, []);
    const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
    assert.ok(baseline && baseline.observation.status === 'pass');
    assert.equal(baseline.observation.memoryWritten, false);
    assert.equal(baseline.observation.nativePhase.systemIterations % 2, 0);
    const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row.observation]));
    const bootElapsed = resetBootElapsed(native.tape);
    const page = browser ? await browser.newPage() : null;
    const errors = [];
    const states = [];
    page?.on('pageerror', error => errors.push(String(error)));
    const runtime = wasmNode ? await createPublicWasmRuntime({ wasm: resolve(wasmNode), expectedWasmSha: wasmSha }) : null;
    const journal = runtime ? createPublicWasmJournal(directory, { scope: 'Actual public-WASM coin observations; Browser/render/audio unavailable' }) : null;
    const commands = [], pending = []; let sequence = 0, completed = false;
    const issue = command => { runtime.command(command); commands.push(structuredClone(command)); pending.push(structuredClone(command)); };
    const actualNodeSnapshot = async () => {
      const state = runtime.snapshot();
      await journal.append({ schema: 1, sequence: sequence++, commands: pending.splice(0), state });
      return state;
    };
    let comparisons = 0;
    let firstCollection;
    let admitted = 0;
    let passedPresentations = 0;
    const equal = (actual, expected, field, presentation) => {
      try { assert.deepEqual(actual, expected, `${item.name}:${presentation}:${field}`); }
      catch (error) { caseResult.failures.push({ field, presentation, actual, expected, error: String(error.message) }); }
    };
    try {
      let start;
      if (runtime) {
        runtime.reset(item.seedValue, scenarios.bootEpochSeed, undefined, {sfxVolume: 0, musicVolume: 0}, {fullscreen: false}, bootElapsed);
        for (const command of manualSeedCommands(item.code)) issue(command);
        start = await actualNodeSnapshot();
      } else {
        await page.goto(`${server.url}/?manual=1`);
        await waitForReady(page);
        await page.evaluate(({ seedValue, bootEpochSeed, bootElapsed }) => {
          window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed, bootElapsed, audioPreferences: {sfxVolume: 0, musicVolume: 0}, fullscreenPreferences: {fullscreen: false} });
        }, { seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed, bootElapsed });
        start = await enterSeededRun(page, item.code);
      }
      assert.equal(start.screen, 'playing');
      assert.equal(start.runSeedValue, item.seedValue);
      assertSeededRun(start, item.code, item.seedValue);
      assert.equal(start.systemClock.systemCounter % 2, 0);
      const route = [[true, 17], [false, 100 + item.delay], [true, 25], [false, 30]];
      const expected = [...manualSeedCommands(item.code), ...route.flatMap(([held, frames]) => [
        ...(held ? [{ type: 'key', code: 'KeyS', down: true }] : []),
        ...Array.from({ length: frames }, () => ({ type: 'tick', count: 1 })),
        ...(held ? [{ type: 'key', code: 'KeyS', down: false }] : []),
      ])];
      let presentation = 0, previousCoins = start.inventory.coins;
      const coinCollections = [];
      for (const [held, frames] of route) {
        if (held) { if (runtime) issue({ type: 'key', code: 'KeyS', down: true }); else await page.keyboard.down('s'); }
        for (let index = 0; index < frames; index++) {
          if (runtime) issue({ type: 'tick', count: 1 });
          const state = runtime ? await actualNodeSnapshot() : await stepSnapshot(page);
          states.push(state);
          presentation++;
          const source = rows.get(baseline.context.presentedFrame + presentation);
          assert.equal(source?.status, 'pass', `${item.name}:${presentation}: live source observation`);
          assert.equal(source.memoryWritten, false);
          const beforeFailures = caseResult.failures.length;
          equal(state.systemClock.systemCounter % 2, source.nativePhase.systemIterations % 2, 'phase', presentation);
          const player = source.players[0];
          for (const [field, actual] of [['coinInventory', state.inventory.coins], ['keyInventory', state.inventory.keys],
            ['bombInventory', state.inventory.bombs], ['redHp', state.player.hp], ['maxRedHp', state.player.maxHp]]) {
            equal(actual, player[field], field, presentation);
          }
          if (state.inventory.coins > start.inventory.coins && firstCollection === undefined) firstCollection = presentation;
          if (state.inventory.coins > previousCoins) coinCollections.push({ presentation, amount: state.inventory.coins - previousCoins });
          previousCoins = state.inventory.coins;
          const pickups = source.entities.filter(entity => entity.type === 5);
          equal(state.room.pickups.length, pickups.length, 'pickup admission/retirement', presentation);
          admitted = Math.max(admitted, pickups.length);
          for (let index = 0; index < pickups.length; index++) {
            const original = pickups[index], actual = state.room.pickups[index];
            if (!actual) continue;
            for (const field of ['variant', 'subtype', 'spawnSeed', 'animation', 'animationFrame', 'animationPlaying', 'collisionRadius']) {
              equal(actual[field], original[field], `pickup${index}:${field}`, presentation);
            }
            equal([actual.position.x, actual.position.y], original.position, `pickup${index}:position`, presentation);
            equal(actual.collected, original.removed, `pickup${index}:collection flag`, presentation);
          }
          const handles = state.actorScheduler.actors;
          equal(handles.filter(actor => actor.kind === 'pickup').map(actor => actor.id), state.room.pickups.map(pickup => pickup.id), 'pickup actor handles', presentation);
          const sourceOrder = source.entities.filter(actor => actor.type === 1 || actor.type === 5)
            .map(actor => actor.type === 1 ? 'player' : 'pickup');
          equal(handles.filter(actor => actor.kind === 'player' || actor.kind === 'pickup').map(actor => actor.kind),
            sourceOrder, 'native relative actor admission order', presentation);
          if (caseResult.failures.length === beforeFailures) passedPresentations++;
          comparisons++;
        }
        if (held) { if (runtime) issue({ type: 'key', code: 'KeyS', down: false }); else await page.keyboard.up('s'); }
      }
      assert.ok(admitted >= 1, 'Gate must observe naturally admitted pickups');
      assert.ok(firstCollection !== undefined, 'Gate must collect a natural coin through public movement');
      assert.equal(states[firstCollection - 1].systemClock.systemCounter % 2, 1, 'Source pickup contact is full-only');
      let trace;
      if (runtime) assert.deepEqual(commands, expected, 'Every actual public ABI command equals the declared tape');
      else { trace = await readTrace(page); assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed); }
      const finalState = runtime ? await actualNodeSnapshot() : await snapshot(page);
      assertSeededRun(finalState, item.code, item.seedValue);
      assert.deepEqual(errors, []);
      Object.assign(caseResult, { status: caseResult.failures.length ? 'fail' : 'pass', comparedPresentations: comparisons,
        passedPresentations, admittedPickups: admitted, firstCollection, coinCollections,
        inventoryBefore: start.inventory, inventoryAfter: finalState.inventory,
        actualRemainingPickups: finalState.room.pickups.length,
        inputProvenance: { status: 'pass', commands: runtime ? commands.length : trace.commands.length,
          initialSeed: runtime ? item.seedValue : trace.initialSeed,
          bootEpochSeed: runtime ? scenarios.bootEpochSeed : trace.bootEpochSeed, manualSeed: item.code,
          scope: runtime ? 'Actual reset/key/tick ABI calls and state readback; no DOM trace claim' : 'Browser input trace' },
        nativeManifestSha256: digest(manifestBytes), nativeManifest, originalReused: Boolean(reusedOriginal) });
      completed = true;
    } finally {
      if (runtime) {
        caseResult.wasmJournal = await journal.close({ completed });
        const after = Object.fromEntries(await Promise.all(protectedPaths.map(async path => [path, digest(await readFile(path))])));
        caseResult.inputsBefore = sourceDigests; caseResult.inputsAfter = after;
        caseResult.sourceBeforeEqualsEnd = JSON.stringify(sourceDigests) === JSON.stringify(after);
        assert.deepEqual(after, sourceDigests, 'Actual protected Native/tool/WASM inputs remain unchanged');
      } else {
        // Evidence capture must not replace the case's real error with its own.
        const trace = await readTrace(page).catch(error => ({ unavailable: String(error) }));
        await writeFile(join(directory, 'browser-states.json'), JSON.stringify({ schema: 1, states, trace }, null, 2) + '\n');
        await page.close();
      }
    }
    } catch (error) {
      caseResult.status = 'fail'; caseResult.error = String(error);
    }
    await writeFile(join(directory, 'report.json'), JSON.stringify(caseResult, null, 2) + '\n');
    console.log(JSON.stringify({ case: item.name, cohort: caseResult.cohort, status: caseResult.status,
      comparedPresentations: caseResult.comparedPresentations, failures: caseResult.failures.length }));
  }
  report.status = report.cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
} catch (error) {
  report.status = 'fail'; report.error = String(error); throw error;
} finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  if (browser) await browser.close(); if (server) await server.close();
  console.log(JSON.stringify({ status: report.status, report: join(output, 'report.json'), cases: report.cases.length }));
  process.exitCode = report.status === 'pass' ? 0 : 1;
}
