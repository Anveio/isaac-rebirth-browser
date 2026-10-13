// Natural key admission and collection in both live games; source observations never enter WASM.
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
import { waitForReady } from '../foundation/verify.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const ORIGINAL_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const gateSources = ['tools/e2e/keys.mjs', 'tools/original/key-scenarios.py',
  'tools/original/rock-blast-scenarios.py', 'tools/foundation/decode-rooms.py',
  'tools/original/foundation-seeds.py',
  'tools/original/foundation-pickup-observe.py', 'tools/original/foundation-clock-run.py',
  'tools/original/foundation-clock-gate.c', 'tools/original/foundation-clock-verify.py',
  'tools/original/state-observer.py', 'tools/original/with-display.sh',
  'tools/foundation/production-server.mjs', 'tools/foundation/browser.mjs',
  'web/trace.ts',
  'tools/original/key-observer.py', 'tools/original/audio-observer.py'];
const sourceHashes = async () => Object.fromEntries(await Promise.all(gateSources.map(async path =>
  [path, digest(await readFile(join(ROOT, path)))])));
const beforeSources = await sourceHashes();
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/keys', String(Date.now())));
await mkdir(output, { recursive: true });
let scenarios;
if (value('--scenarios')) scenarios = JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8'));
else {
  const result = await run('python3', ['tools/original/key-scenarios.py', '--count', value('--count') || '2'], { cwd: ROOT, timeout: 600000 });
  scenarios = JSON.parse(result.stdout);
}
assert.equal(scenarios.schema, 1);
assert.equal(scenarios.bootEpochSeed, 1700000000);
assert.ok(scenarios.cases.length >= 4 && scenarios.cases.length <= 6);
assert.equal(new Set(scenarios.cases.map(item => item.name)).size, scenarios.cases.length);
for (const item of scenarios.cases) {
  assert.match(item.name, /^[a-zA-Z0-9_-]{1,32}$/);
  assert.match(item.code, /^[A-Z0-9]{8}$/);
  assert.ok(Number.isInteger(item.seedValue) && item.seedValue > 0 && item.seedValue <= 0xffffffff);
  assert.ok([0, 1].includes(item.delay));
  assert.ok(Array.isArray(item.actions) && item.actions.length > 0);
  for (const action of item.actions) {
    assert.deepEqual(Object.keys(action).sort(), ['frames', 'key']);
    assert.ok([null, 'w', 'a', 's', 'd', 'e'].includes(action.key));
    assert.ok(Number.isInteger(action.frames) && action.frames >= 1 && action.frames <= 200);
  }
}
const stateLast = states => states[states.length - 1];
const selectedCase = value('--case');
if (selectedCase) assert.ok(scenarios.cases.some(item => item.name === selectedCase), 'Targeted replay requires a recorded public case name');
assert.equal(digest(await readFile(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64'))), ORIGINAL_SHA);
await writeFile(join(output, 'scenarios.json'), JSON.stringify(scenarios, null, 2) + '\n');
const server = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = {
  schema: 1, status: 'running', originalBinarySha256: ORIGINAL_SHA, production: server.build,
  selection: selectedCase ? { scope: 'Explicit targeted recorded-case replay; default CI executes the complete cohort', case: selectedCase } : { scope: 'Complete selected cohort' },
  gateSources: beforeSources,
  graphics: await inspectBrowserGraphics(browser),
  scope: 'Live public-seed secret-room regular key admission, local subtype RNG, full-phase animation/contact/collection, inventory, actor cleanup and actual native key SFX slot admission. Golden keys, item modifiers, keeper NPC behavior, total shared MT/sample identity, key spending and pixels remain unaccepted.',
  cases: [],
};
try {
  for (const item of scenarios.cases) {
    if (selectedCase && item.name !== selectedCase) continue;
    const directory = join(output, item.name);
    await mkdir(directory, { recursive: true });
    const caseResult = { ...item, cohort: item.cohort, status: 'running', failures: [] };
    report.cases.push(caseResult);
    try {
    await writeFile(join(directory, 'case.json'), JSON.stringify(item, null, 2) + '\n');
    const tapeResult = await run('python3', ['tools/original/key-scenarios.py', '--case', join(directory, 'case.json')], { cwd: ROOT });
    await writeFile(join(directory, 'tape.json'), tapeResult.stdout);
    await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
      '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
      '--state-observer', 'tools/original/key-observer.py', '--output', join(directory, 'original')],
      { cwd: ROOT, timeout: 180000, maxBuffer: 2 * 1024 * 1024 });
    const manifestBytes = await readFile(join(directory, 'original/manifest.json'));
    const native = JSON.parse(manifestBytes);
    assert.equal(native.status, 'pass');
    assert.deepEqual(native.failures, []);
    assert.equal(native.seedValue, item.seedValue);
    assert.equal(native.seedReadbackMatchesCodec, true);
    const provenance = JSON.parse((await run('python3', ['tools/original/key-scenarios.py', '--validate-original', join(directory, 'original/manifest.json')], { cwd: ROOT })).stdout);
    assert.equal(provenance.status, 'pass');
    assert.deepEqual(provenance.failures, []);
    const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
    assert.ok(baseline && baseline.observation.status === 'pass');
    assert.equal(baseline.context.publicLogState.seedValue, item.seedValue);
    assert.equal(baseline.observation.memoryWritten, false);
    assert.equal(baseline.observation.nativePhase.systemIterations % 2, 0);
    const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row.observation]));
    const page = await browser.newPage();
    const errors = [];
    const states = [];
    page.on('pageerror', error => errors.push(String(error)));
    let comparisons = 0;
    let detonation;
    let admitted = 0;
    const collections = [];
    let previousKeys = 0;
    let passedPresentations = 0;
    const equal = (actual, expected, field, presentation) => {
      try { assert.deepEqual(actual, expected, `${item.name}:${presentation}:${field}`); }
      catch (error) { caseResult.failures.push({ field, presentation, actual, expected, error: String(error.message) }); }
    };
    try {
      await page.goto(`${server.url}/?manual=1`);
      await waitForReady(page);
      await page.evaluate(({ seedValue, bootEpochSeed }) => {
        window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed });
      }, { seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed });
      // Independently declared complete input tape, beginning at reset. A
      // reset seed cannot stand in for actual manual seed entry/readback.
      const press = code => [{ type: 'key', code, down: true }, { type: 'key', code, down: false }];
      const expected = [
        ...press('Enter'), ...press('Enter'), ...press('Enter'), ...press('Tab'),
        ...[...item.code].map(text => ({ type: 'text', text })),
        ...press('Enter'), ...press('Enter'),
      ];
      for (let index = 0; index < 3; index++) await page.keyboard.press('Enter');
      await page.keyboard.press('Tab');
      await page.keyboard.type(item.code);
      await page.keyboard.press('Enter');
      await page.keyboard.press('Enter');
      const start = await page.evaluate(() => window.game.snapshot());
      assert.equal(start.screen, 'playing');
      assert.equal(start.seedText, item.code);
      assert.equal(start.seedValid, true);
      assert.equal(start.seededRun, true);
      assert.equal(start.runSeed, item.code);
      assert.equal(start.runSeedValue, item.seedValue);
      assert.equal(start.systemClock.systemCounter % 2, 0);
      caseResult.seedReadback = { seedText: start.seedText, seedValid: start.seedValid,
        seededRun: start.seededRun, runSeed: start.runSeed, runSeedValue: start.runSeedValue };
      let presentation = 0;
      for (const { key, frames } of item.actions) {
        const code = key ? 'Key' + key.toUpperCase() : null;
        if (key) { await page.keyboard.down(key); expected.push({ type: 'key', code, down: true }); }
        for (let index = 0; index < frames; index++) {
          await page.evaluate(() => window.game.step(1));
          expected.push({ type: 'tick', count: 1 });
          const state = await page.evaluate(() => window.game.snapshot());
          states.push(state);
          presentation++;
          const source = rows.get(baseline.context.presentedFrame + presentation);
          assert.equal(source?.status, 'pass', `${item.name}:${presentation}: live source observation`);
          assert.equal(source.memoryWritten, false);
          assert.equal(source.nativeBinarySha256, ORIGINAL_SHA);
          const beforeFailures = caseResult.failures.length;
          equal(state.systemClock.systemCounter - start.systemClock.systemCounter, presentation, 'one presentation per public tick', presentation);
          equal(state.systemClock.systemCounter % 2, source.nativePhase.systemIterations % 2, 'phase', presentation);
          const player = source.players[0];
          for (const [field, actual] of [['coinInventory', state.inventory.coins], ['keyInventory', state.inventory.keys],
            ['bombInventory', state.inventory.bombs], ['redHp', state.player.hp], ['maxRedHp', state.player.maxHp]]) {
            equal(actual, player[field], field, presentation);
          }
          if (state.explosions.length && detonation === undefined) detonation = presentation;
          equal([state.player.x, state.player.y], [player.position[0] * .65 + 32, player.position[1] * .65 - 46], 'player position', presentation);
          equal([state.player.vx, state.player.vy], [player.velocity[0] * .65, player.velocity[1] * .65], 'player velocity', presentation);
          equal(state.damageCooldown, player.damageCountdown, 'damage countdown', presentation);
          equal(state.gameUpdateCount - start.gameUpdateCount, source.nativePhase.fullGameUpdates - baseline.observation.nativePhase.fullGameUpdates, 'full game update clock', presentation);
          const grid = state.room.grid;
          assert.ok(grid, 'Actual authored room grid must be admitted');
          equal([grid.width, grid.height], [source.room.gridWidth, source.room.gridHeight], 'grid dimensions', presentation);
          const doors = source.grid.filter(cell => cell.type === 16);
          equal(grid.cells.flatMap((cell, index) => cell.entity?.kind === 16 ? [index] : []),
            doors.map(cell => cell.index), 'door admission', presentation);
          for (const door of doors) {
            const actual = grid.cells[door.index]?.entity;
            if (!actual) continue;
            for (const [field, expected] of [['variant', door.variant], ['state', door.state], ['collisionClass', door.collisionClass]]) {
              equal(actual[field], expected, `door${door.index}:${field}`, presentation);
            }
          }
          const originalBombs = source.entities.filter(entity => entity.type === 4 && !entity.removed);
          const bombs = state.bombs.filter(bomb => bomb.motion.lifetime === 'alive');
          equal(bombs.length, originalBombs.length, 'live bomb admission/retirement', presentation);
          for (let index = 0; index < originalBombs.length; index++) {
            const original = originalBombs[index], actual = bombs[index];
            if (!actual) continue;
            // The wire vectors encode Rust f32 decimals; recover their exact
            // source f32 values, with no tolerance or position quantization.
            equal([Math.fround(actual.motion.presentedPosition.x), Math.fround(actual.motion.presentedPosition.y)], original.position, `bomb${index}:position`, presentation);
            equal([Math.fround(actual.motion.velocity.x), Math.fround(actual.motion.velocity.y)], original.velocity, `bomb${index}:velocity`, presentation);
            equal(actual.motion.fuseUpdatesRemaining, original.bombFuseUpdatesRemaining, `bomb${index}:fuse`, presentation);
          }
          const pickups = source.entities.filter(entity => entity.type === 5);
          admitted = Math.max(admitted, pickups.filter(pickup => pickup.variant === 30).length);
          if (player.keyInventory !== previousKeys) {
            collections.push({ presentation, amount: player.keyInventory - previousKeys });
            previousKeys = player.keyInventory;
            equal(state.systemClock.systemCounter % 2, 1, 'key collection full-only', presentation);
          }
          if (collections.length) {
            const actual = state.audio.slots[58], original = source.keySound;
            const baselineHalfTick = baseline.observation.audioSystemCounter >> 1;
            equal(actual.lastTick, original.lastTick - baselineHalfTick, 'key sound admission tick', presentation);
            equal(actual.nextAllowed, original.nextAllowed - baselineHalfTick, 'key sound cooldown', presentation);
            equal(Math.fround(actual.lastVolume), original.lastVolume, 'key sound volume', presentation);
          }
          equal(state.room.pickups.length, pickups.length, 'pickup admission/retirement', presentation);

          for (let index = 0; index < pickups.length; index++) {
            const original = pickups[index], actual = state.room.pickups[index];
            if (!actual) continue;
            for (const field of ['variant', 'subtype', 'spawnSeed', 'animation', 'animationFrame', 'animationPlaying', 'collisionRadius']) {
              equal(actual[field], original[field], `pickup${index}:${field}`, presentation);
            }
            equal([Math.fround(actual.position.x), Math.fround(actual.position.y)], original.position, `pickup${index}:position`, presentation);
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
          if (presentation % 100 === 0) console.log(JSON.stringify({ case: item.name, presentation }));
        }
        if (key) { await page.keyboard.up(key); expected.push({ type: 'key', code, down: false }); }
      }
      assert.equal(admitted, 2, 'Gate must observe both naturally authored original keys');
      assert.equal(stateLast(states).inventory.keys, 2, 'Public movement must collect both original regular keys');
      assert.equal(collections.length, 2, 'Gate must observe two separate native collection contacts');
      assert.equal(stateLast(states).room.pickups.length, 0, 'Both Collect clips must finish and retire actors');
      assert.ok(detonation !== undefined, 'Public starting bomb must reveal the secret doorway');
      assert.equal(stateLast(states).inventory.bombs, 0, 'Public E action must consume the starting bomb');
      const trace = await page.evaluate(() => window.game.trace());
      assert.equal(trace.inputTraceTruncated, false, 'Every public command must remain recorded');
      assert.deepEqual(trace.commands, expected, 'Complete reset/menu/manual-seed/gameplay input tape');
      caseResult.trace = { inputTraceTruncated: trace.inputTraceTruncated,
        stateHistoryTruncated: trace.stateHistoryTruncated,
        stateHistoryTruncationReason: trace.stateHistoryTruncationReason,
        inputEncodedBytes: trace.inputEncodedBytes, stateEncodedBytes: trace.stateEncodedBytes };
      assert.deepEqual(errors, []);
      Object.assign(caseResult, { status: caseResult.failures.length ? 'fail' : 'pass', comparedPresentations: comparisons,
        passedPresentations, admittedKeys: admitted, collections, detonation,
        nativeManifestSha256: digest(manifestBytes), nativeManifest: join(directory, 'original/manifest.json') });
    } finally {
      Object.assign(caseResult, { comparedPresentations: comparisons, passedPresentations,
        admittedKeys: admitted, collections, detonation });
      await writeFile(join(directory, 'browser-states.json'), JSON.stringify({ schema: 1, states }, null, 2) + '\n');
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
  report.gateSourcesStable = JSON.stringify(beforeSources) === JSON.stringify(await sourceHashes());
  if (!report.gateSourcesStable) report.status = 'fail';
} catch (error) {
  report.status = 'fail'; report.error = String(error); throw error;
} finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser.close(); await server.close();
  console.log(JSON.stringify({ status: report.status, report: join(output, 'report.json'), cases: report.cases.length }));
  process.exitCode = report.status === 'pass' ? 0 : 1;
}
