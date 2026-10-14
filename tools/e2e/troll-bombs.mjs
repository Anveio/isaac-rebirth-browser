// Natural Troll/Megatroll conversion and lifetime in both live games; source observations never enter WASM.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { gzipSync } from 'node:zlib';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, enterSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';
import { stepSnapshot } from './lib/game.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/troll-bombs', String(Date.now())));
await mkdir(output, { recursive: true });
let scenarios;
if (value('--scenarios')) scenarios = JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8'));
else {
  const args = ['tools/original/troll-bomb-scenarios.py'];
  if (value('--count')) args.push('--count', value('--count'));
  const result = await run('python3', args, { cwd: ROOT, timeout: 600000 });
  scenarios = JSON.parse(result.stdout);
}
assert.equal(scenarios.schema, 1);
assert.equal(scenarios.bootEpochSeed, 1700000000);
assert.ok(scenarios.cases.length >= 3 && scenarios.cases.length <= 11);
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
await assertOriginalBinary();
await writeFile(join(output, 'scenarios.json'), JSON.stringify(scenarios, null, 2) + '\n');
const server = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = {
  schema: 1, status: 'running', originalBinarySha256: ORIGINAL_SHA, production: server.build,
  selection: selectedCase ? { scope: 'Explicit targeted recorded-case replay; default CI executes the complete cohort', case: selectedCase } : { scope: 'Complete selected cohort' },
  graphics: await inspectBrowserGraphics(browser),
  scope: 'Live public-seed natural Troll/Megatroll conversion, typed shared seed, zero-velocity child admission, relative actor order, full/fast motion and fuse, animation, lifetime, homing, grid/HP and remaining pickup behavior. Every field mismatch remains blocking, including shared-stream alignment and blast pickup impulse. Required recorded OS draws are unfiltered.',
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
    const tapeResult = await run('python3', ['tools/original/troll-bomb-scenarios.py', '--case', join(directory, 'case.json')], { cwd: ROOT });
    await writeFile(join(directory, 'tape.json'), tapeResult.stdout);
    await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
      '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
      '--state-observer', 'tools/original/troll-bomb-observer.py', '--output', join(directory, 'original')],
      { cwd: ROOT, timeout: 180000, maxBuffer: 2 * 1024 * 1024 });
    const manifestBytes = await readFile(join(directory, 'original/manifest.json'));
    caseResult.nativeManifestSha256 = digest(manifestBytes);
    caseResult.nativeManifest = join(directory, 'original/manifest.json');
    const native = JSON.parse(manifestBytes);
    assert.equal(native.status, 'pass');
    assert.deepEqual(native.failures, []);
    assert.equal(native.seedValue, item.seedValue);
    assert.equal(native.seedReadbackMatchesCodec, true);
    const provenance = JSON.parse((await run('python3', ['tools/original/troll-bomb-scenarios.py', '--validate-original', join(directory, 'original/manifest.json')], { cwd: ROOT })).stdout);
    assert.equal(provenance.status, 'pass');
    assert.deepEqual(provenance.failures, []);
    const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
    assert.ok(baseline && baseline.observation.status === 'pass');
    assert.equal(baseline.context.publicLogState.seedValue, item.seedValue);
    assert.equal(baseline.observation.memoryWritten, false);
    assert.equal(baseline.observation.nativePhase.systemIterations % 2, 0);
    assert.equal(baseline.observation.players[0].bombInventory, 1, 'Only the real fresh Isaac starting bomb supplies access');
    const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row]));
    const page = await browser.newPage();
    const errors = [];
    const states = [];
    page.on('pageerror', error => errors.push(String(error)));
    let comparisons = 0;
    let detonation;
    let admitted = 0;
    let admissionFamilySize;
    const collections = [];
    const admittedSource = new Map();
    const conversions = new Map();
    const pickupBirths = new Map();
    const pickupReadmissions = [];
    let previousBombs = 1;
    const placements = [];
    let destroyedRocks = 0;
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
      // Source-qualified menu/seed/run intervals and every tick are declared
      // before execution; deferred Music run admission is never bypassed.
      const expected = [...manualSeedCommands(item.code)];
      const start = await enterSeededRun(page, item.code);
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
          const state = await stepSnapshot(page);
          expected.push({ type: 'tick', count: 1 });
          states.push(state);
          presentation++;
          const row = rows.get(baseline.context.presentedFrame + presentation);
          const source = row?.observation;
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
          const identity = /^(\d+)\.(\d+)\(/.exec(row.context.publicLogState.room);
          assert.ok(identity, 'Native public log must identify the current authored room');
          equal([state.room.descriptor?.template.type, state.room.descriptor?.template.variant, state.room.descriptor?.spawnSeed],
            [Number(identity[1]), Number(identity[2]), row.context.publicLogState.spawnSeed], 'authored room identity', presentation);
          if (admissionFamilySize === undefined && Number(identity[1]) === 1 && Number(identity[2]) === 35) admissionFamilySize = source.entities.filter(entity => (entity.type === 5 && entity.variant === 40) || (entity.type === 4 && [3, 4].includes(entity.variant))).length;
          const grid = state.room.grid;
          assert.ok(grid, 'Actual authored room grid must be admitted');
          equal([grid.width, grid.height], [source.room.gridWidth, source.room.gridHeight], 'grid dimensions', presentation);
          const rocks = source.grid.filter(cell => [2, 4, 5, 6].includes(cell.type));
          equal(grid.cells.flatMap((cell, index) => [2, 4, 5, 6].includes(cell.entity?.kind) ? [index] : []),
            rocks.map(cell => cell.index), 'rock admission/retirement', presentation);
          destroyedRocks = Math.max(destroyedRocks, rocks.filter(rock => rock.state === 2).length);
          for (const rock of rocks) {
            const actual = grid.cells[rock.index]?.entity;
            if (!actual) continue;
            for (const [field, value] of [['kind', rock.type], ['variant', rock.variant], ['state', rock.state],
              ['collisionClass', rock.collisionClass], ['spawnSeed', rock.spawnSeed]]) {
              equal(actual[field], value, `rock${rock.index}:${field}`, presentation);
            }
            // The destruction marker lives in the signed Room path costs (Room+e98).
            assert.ok(Array.isArray(state.room.pathCosts) && Array.isArray(source.pathCosts), 'Rock markers require actual signed Room path costs');
            equal(state.room.pathCosts[rock.index], source.pathCosts[rock.index], `rock${rock.index}:destruction marker`, presentation);
          }
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
          const originalBombs = source.entities.filter(entity => entity.type === 4);
          const bombs = state.bombs;
          equal(bombs.length, originalBombs.length, 'live bomb admission/retirement', presentation);
          for (let index = 0; index < originalBombs.length; index++) {
            const original = originalBombs[index], actual = bombs[index];
            if ([3, 4].includes(original.variant) && !conversions.has(original.sequenceId)) conversions.set(original.sequenceId, { presentation, variant: original.variant, seed: original.spawnSeed, position: original.position, velocity: original.velocity, fuse: original.bombFuseUpdatesRemaining, spawnerType: original.spawnerType, spawnerVariant: original.spawnerVariant });
            if (!actual) continue;
            equal(actual.variant, original.variant, `bomb${index}:variant`, presentation);
            equal(actual.motion.lifetime === 'exploding', original.removed, `bomb${index}:death flag`, presentation);
            for (const field of ['animation', 'animationFrame', 'animationPlaying', 'friction']) equal(field === 'friction' ? Math.fround(actual[field]) : actual[field], original[field], `bomb${index}:${field}`, presentation);
            if ([3, 4].includes(original.variant)) {
              equal(actual.spawnSeed, original.spawnSeed, `bomb${index}:causal shared spawn seed`, presentation);
              equal(state.gameUpdateCount - actual.birthGameUpdate, source.nativePhase.fullGameUpdates - original.birthGameUpdate, `bomb${index}:age`, presentation);
              equal(original.spawnerType, 0, `bomb${index}:native null spawner`, presentation);
              equal(original.bombExplosionDamage, 60, `bomb${index}:native ordinary blast damage`, presentation);
            }
            // The wire vectors encode Rust f32 decimals; recover their exact
            // source f32 values, with no tolerance or position quantization.
            equal([Math.fround(actual.motion.presentedPosition.x), Math.fround(actual.motion.presentedPosition.y)], original.position, `bomb${index}:position`, presentation);
            equal([Math.fround(actual.motion.velocity.x), Math.fround(actual.motion.velocity.y)], original.velocity, `bomb${index}:velocity`, presentation);
            equal(actual.motion.fuseUpdatesRemaining, original.bombFuseUpdatesRemaining, `bomb${index}:fuse`, presentation);
          }
          const pickups = source.entities.filter(entity => entity.type === 5);
          admitted = Math.max(admitted, pickups.filter(pickup => pickup.variant === 40).length);
          for (const pickup of pickups) {
            if (!admittedSource.has(pickup.spawnSeed)) admittedSource.set(pickup.spawnSeed,
              { spawnSeed: pickup.spawnSeed, variant: pickup.variant, subtype: pickup.subtype,
                position: pickup.position, price: pickup.pickupPrice });
          }
          if (player.bombInventory > previousBombs) {
            collections.push({ presentation, amount: player.bombInventory - previousBombs });
            equal(state.systemClock.systemCounter % 2, 1, 'bomb pickup collection full-only', presentation);
          } else if (player.bombInventory < previousBombs) {
            placements.push({ presentation, amount: previousBombs - player.bombInventory });
          }
          previousBombs = player.bombInventory;
          equal(state.room.pickups.length, pickups.length, 'pickup admission/retirement', presentation);

          for (let index = 0; index < pickups.length; index++) {
            const original = pickups[index], actual = state.room.pickups[index];
            equal(original.pickupPrice, 0, `pickup${index}:native free price`, presentation);
            equal(actual?.collectionDelay, original.collectionDelay, `pickup${index}:collection delay`, presentation);
            const age = source.nativePhase.fullGameUpdates - original.birthGameUpdate;
            const previousBirth = pickupBirths.get(original.spawnSeed);
            if (previousBirth !== undefined && previousBirth !== original.birthGameUpdate) {
              pickupReadmissions.push({ presentation, spawnSeed: original.spawnSeed,
                birthGameUpdate: original.birthGameUpdate, age, position: original.position,
                velocity: original.velocity, rng: original.pickupRng,
                spawnerType: original.spawnerType, spawnerVariant: original.spawnerVariant });
              equal(original.spawnerType, 0, `pickup${index}:native saved-record null spawner`, presentation);
            }
            pickupBirths.set(original.spawnSeed, original.birthGameUpdate);
            if (!actual) continue;
            equal(actual.age, age, `pickup${index}:birth-clock age`, presentation);
            const rng = actual.regularRng;
            equal(rng ? [rng.state, rng.rightA, rng.left, rng.rightB] : null, original.pickupRng,
              `pickup${index}:persistent RNG30`, presentation);
            equal([Math.fround(actual.anchorPosition.x), Math.fround(actual.anchorPosition.y)],
              original.pickupAnchor, `pickup${index}:base anchor`, presentation);
            for (const field of ['variant', 'subtype', 'spawnSeed', 'animation', 'animationFrame', 'animationPlaying', 'collisionRadius']) {
              equal(actual[field], original[field], `pickup${index}:${field}`, presentation);
            }
            // Native448 is the presented position after a fast update. The
            // port keeps its full-update position separately from prediction.
            equal([Math.fround(actual.presentedPosition.x), Math.fround(actual.presentedPosition.y)], original.position, `pickup${index}:presented position`, presentation);
            if (source.nativePhase.systemIterations % 2 === 1) {
              equal([Math.fround(actual.position.x), Math.fround(actual.position.y)], original.position, `pickup${index}:full physical position`, presentation);
            }
            equal([Math.fround(actual.velocity.x), Math.fround(actual.velocity.y)], original.velocity, `pickup${index}:velocity`, presentation);
            equal(Math.fround(actual.currentFriction), original.friction, `pickup${index}:current friction`, presentation);
            equal(actual.collisionMass, original.collisionMass, `pickup${index}:collision mass`, presentation);
            equal(actual.regularCollision?.gridCollisionClass, original.gridCollisionClass, `pickup${index}:grid collision class`, presentation);
            equal(actual.regularCollision?.entityCollisionClass, original.entityCollisionClass, `pickup${index}:entity collision class`, presentation);
            equal(actual.collected, original.removed, `pickup${index}:collection flag`, presentation);
          }
          const handles = state.actorScheduler.actors;
          equal(handles.filter(actor => actor.kind === 'pickup').map(actor => actor.id), state.room.pickups.map(pickup => pickup.id), 'pickup actor handles', presentation);
          const sourceOrder = source.entities.filter(actor => [1, 4, 5].includes(actor.type))
            .map(actor => actor.type === 1 ? 'player' : actor.type === 4 ? 'bomb' : 'pickup');
          equal(handles.filter(actor => ['player', 'bomb', 'pickup'].includes(actor.kind)).map(actor => actor.kind),
            sourceOrder, 'native relative actor admission order', presentation);
          if (caseResult.failures.length === beforeFailures) passedPresentations++;
          comparisons++;
          if (presentation % 100 === 0) console.log(JSON.stringify({ case: item.name, presentation }));
        }
        if (key) { await page.keyboard.up(key); expected.push({ type: 'key', code, down: false }); }
      }
      const trace = await page.evaluate(() => window.game.trace());
      const traceBytes = Buffer.from(JSON.stringify(trace) + '\n');
      const traceArchive = gzipSync(traceBytes, { level: 1 });
      await writeFile(join(directory, 'browser-trace.json.gz'), traceArchive);
      caseResult.trace = { inputTraceTruncated: trace.inputTraceTruncated,
        stateHistoryTruncated: trace.stateHistoryTruncated,
        stateHistoryTruncationReason: trace.stateHistoryTruncationReason,
        inputEncodedBytes: trace.inputEncodedBytes, stateEncodedBytes: trace.stateEncodedBytes,
        backendEvents: trace.backendEvents.length, completeStateRecords: trace.states.length,
        archive: join(directory, 'browser-trace.json.gz'),
        uncompressedSha256: digest(traceBytes), archiveSha256: digest(traceArchive) };
      assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed);
      assert.equal(admissionFamilySize, 3, 'Every route must enter actual Room35 and admit all three authored bomb families, including converted children');
      if (item.requiredNativePickupSubtypes)
        assert.deepEqual([...admittedSource.values()].filter(actor => actor.variant === 40).map(actor => actor.subtype),
          item.requiredNativePickupSubtypes, 'Source-disproved KPF fixtures must retain their actual complete regular pickup family');
      for (const variant of item.requiredVariants || []) assert.ok([...conversions.values()].some(event => event.variant === variant), `Public native route must admit naturally selected bomb variant ${variant}`);
      for (const event of conversions.values()) {
        assert.deepEqual(event.velocity, [0, 0], 'Conversion must admit a stationary native child');
        assert.equal(event.fuse, 59, 'Conversion starts the native Bomb initializer fuse');
      }
      if (item.retainedTrollRegression) assert.ok(detonation !== undefined && destroyedRocks >= 1, 'Retained natural Troll route must execute the original blast and rock destruction');
      if (item.requiredRegularReadmissions) assert.ok(pickupReadmissions.length >= item.requiredRegularReadmissions,
        'The declared public return route must actually reconstruct its required native regular pickups');
      assert.deepEqual(errors, []);
      Object.assign(caseResult, { status: caseResult.failures.length ? 'fail' : 'pass', comparedPresentations: comparisons,
        passedPresentations, admittedBombPickups: admitted, admissionFamilySize, admittedSource: [...admittedSource.values()],
        collections, placements, conversions: [...conversions.values()], pickupReadmissions, detonation, destroyedRocks,
        nativeManifestSha256: digest(manifestBytes), nativeManifest: join(directory, 'original/manifest.json') });
    } finally {
      Object.assign(caseResult, { comparedPresentations: comparisons, passedPresentations,
        admittedBombPickups: admitted, admissionFamilySize, admittedSource: [...admittedSource.values()], collections, placements, conversions: [...conversions.values()], pickupReadmissions, detonation, destroyedRocks });
      const snapshotBytes = Buffer.from(JSON.stringify({ schema: 1, states }) + '\n');
      const snapshotArchive = gzipSync(snapshotBytes, { level: 1 });
      await writeFile(join(directory, 'browser-states.json.gz'), snapshotArchive);
      caseResult.gameplaySnapshots = { records: states.length, uncompressedBytes: snapshotBytes.length,
        uncompressedSha256: digest(snapshotBytes), archiveSha256: digest(snapshotArchive),
        archive: join(directory, 'browser-states.json.gz') };
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
