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
import { snapshot, stepSnapshot } from './lib/game.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
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
assert.ok(scenarios.cases.length >= 4 && scenarios.cases.length <= 11);
assert.equal(new Set(scenarios.cases.map(item => item.name)).size, scenarios.cases.length);
for (const item of scenarios.cases) {
  assert.match(item.name, /^[a-zA-Z0-9_-]{1,32}$/);
  assert.match(item.code, /^[A-Z0-9]{8}$/);
  assert.ok(Number.isInteger(item.seedValue) && item.seedValue > 0 && item.seedValue <= 0xffffffff);
  assert.ok([0, 1].includes(item.delay));
}
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
await assertOriginalBinary();
await writeFile(join(output, 'scenarios.json'), JSON.stringify(scenarios, null, 2) + '\n');
const server = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = {
  schema: 1, status: 'running', originalBinarySha256: ORIGINAL_SHA, production: server.build,
  graphics: await inspectBrowserGraphics(browser),
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
    await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
      '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
      '--state-observer', 'tools/original/foundation-pickup-observe.py', '--output', join(directory, 'original')],
      { cwd: ROOT, timeout: 180000, maxBuffer: 2 * 1024 * 1024 });
    const manifestBytes = await readFile(join(directory, 'original/manifest.json'));
    const native = JSON.parse(manifestBytes);
    assert.equal(native.status, 'pass');
    assert.deepEqual(native.failures, []);
    const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
    assert.ok(baseline && baseline.observation.status === 'pass');
    assert.equal(baseline.observation.memoryWritten, false);
    assert.equal(baseline.observation.nativePhase.systemIterations % 2, 0);
    const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row.observation]));
    const page = await browser.newPage();
    const errors = [];
    const states = [];
    page.on('pageerror', error => errors.push(String(error)));
    let comparisons = 0;
    let firstCollection;
    let admitted = 0;
    let passedPresentations = 0;
    const equal = (actual, expected, field, presentation) => {
      try { assert.deepEqual(actual, expected, `${item.name}:${presentation}:${field}`); }
      catch (error) { caseResult.failures.push({ field, presentation, actual, expected, error: String(error.message) }); }
    };
    try {
      await page.goto(`${server.url}/?manual=1`);
      await waitForReady(page);
      await page.evaluate(({ seedValue, bootEpochSeed, bootElapsed }) => {
        window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed, bootElapsed });
      }, { seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed, bootElapsed: resetBootElapsed(native.tape) });
      const start = await enterSeededRun(page, item.code);
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
      let presentation = 0;
      for (const [held, frames] of route) {
        if (held) await page.keyboard.down('s');
        for (let index = 0; index < frames; index++) {
          const state = await stepSnapshot(page);
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
        if (held) await page.keyboard.up('s');
      }
      assert.ok(admitted >= 1, 'Gate must observe naturally admitted pickups');
      assert.ok(firstCollection !== undefined, 'Gate must collect a natural coin through public movement');
      assert.equal(states[firstCollection - 1].systemClock.systemCounter % 2, 1, 'Source pickup contact is full-only');
      const trace = await page.evaluate(() => window.game.trace());
      assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed);
      assertSeededRun(await snapshot(page), item.code, item.seedValue);
      assert.deepEqual(errors, []);
      Object.assign(caseResult, { status: caseResult.failures.length ? 'fail' : 'pass', comparedPresentations: comparisons,
        passedPresentations, admittedPickups: admitted, firstCollection,
        inputProvenance: { status: 'pass', commands: trace.commands.length, initialSeed: trace.initialSeed,
          bootEpochSeed: trace.bootEpochSeed, manualSeed: item.code },
        nativeManifestSha256: digest(manifestBytes), nativeManifest: join(directory, 'original/manifest.json') });
    } finally {
      // Evidence capture must not replace the case's real error with its own.
      const trace = await page.evaluate(() => window.game.trace()).catch(error => ({ unavailable: String(error) }));
      await writeFile(join(directory, 'browser-states.json'), JSON.stringify({ schema: 1, states, trace }, null, 2) + '\n');
      await page.close();
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
  await browser.close(); await server.close();
  console.log(JSON.stringify({ status: report.status, report: join(output, 'report.json'), cases: report.cases.length }));
  process.exitCode = report.status === 'pass' ? 0 : 1;
}
