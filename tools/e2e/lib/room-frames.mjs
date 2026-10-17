// Frame-by-frame paired runs of a single starting-room route: both games enter
// the case's seed through public menu input, receive the same held-key actions,
// and every presented frame of the browser is compared with the matching
// read-only native observation.
//
// A suite supplies its scenario helper and observer, plus:
//   compare(frame)  checks for its own subject (doors, rocks, pickups, sounds)
//   finish(run)     whole-case assertions (e.g. "both keys were collected")
// The harness owns the native run, the provenance and seed checks, the input
// trace and the common per-frame player/clock/bomb/pickup/actor comparisons.
// It records each mismatch as a field failure and keeps comparing.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { execFile } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { promisify } from 'node:util';
import { gzipSync } from 'node:zlib';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../../foundation/browser.mjs';
import { startProductionServer } from '../../foundation/production-server.mjs';
import { assertInputProvenance, enterSeededRun, manualSeedCommands, waitForReady } from '../../foundation/verify.mjs';
import { ROOT, flag, outputDirectory, selectCases, writeJson } from './cli.mjs';
import { stepSnapshot, readTrace } from './game.mjs';
import { createPublicWasmRuntime, createPublicWasmJournal } from '../../foundation/composition-wasm-public.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './original.mjs';

const run = promisify(execFile);
const python = async (script, args) => (await run('python3', [script, ...args], { cwd: ROOT, timeout: 600000, maxBuffer: 16 * 1024 * 1024 })).stdout;

function validateScenarios(scenarios, [min, max]) {
  assert.equal(scenarios.schema, 1);
  assert.equal(scenarios.bootEpochSeed, 1700000000);
  // A bank recorded under --case holds only the cases that ran; its full bank
  // met the count when it was recorded.
  if (!scenarios.caseFilter) assert.ok(scenarios.cases.length >= min && scenarios.cases.length <= max, `Expected ${min}..${max} cases`);
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
}

// Native half-tick sound slots are compared relative to each game's run start.
export function compareSound(frame, slot, original, label, fields = []) {
  const { state, start, baseline, equal } = frame;
  const actual = state.audio.slots[slot];
  const baselineHalfTick = baseline.observation.audioSystemCounter >> 1;
  const browserHalfTick = start.systemClock.systemCounter >> 1;
  equal(actual.lastTick - browserHalfTick, original.lastTick - baselineHalfTick, `${label} admission tick`);
  equal(actual.nextAllowed - browserHalfTick, original.nextAllowed - baselineHalfTick, `${label} cooldown`);
  equal(Math.fround(actual.lastVolume), original.lastVolume, `${label} volume`);
  for (const field of fields) equal(actual[field], original[field], `${label} ${field}`);
}

// Authored room doors: admission plus variant, state and collision class.
export function compareDoors(frame) {
  const { state, source, equal } = frame;
  const doors = source.grid.filter(cell => cell.type === 16);
  equal(state.room.grid.cells.flatMap((cell, index) => cell.entity?.kind === 16 ? [index] : []), doors.map(cell => cell.index), 'door admission');
  for (const door of doors) {
    const actual = state.room.grid.cells[door.index]?.entity;
    if (!actual) continue;
    for (const [field, expected] of [['variant', door.variant], ['state', door.state], ['collisionClass', door.collisionClass]])
      equal(actual[field], expected, `door${door.index}:${field}`);
  }
}

// Ordinary Rock-family grids: admission, retirement, metadata and destruction
// markers. The marker lives in the signed Room path costs (Room+e98).
export function compareRocks(frame) {
  const { state, source, equal } = frame;
  const grid = state.room.grid;
  const rocks = source.grid.filter(cell => [2, 4, 5, 6].includes(cell.type));
  equal(grid.cells.flatMap((cell, index) => [2, 4, 5, 6].includes(cell.entity?.kind) ? [index] : []), rocks.map(cell => cell.index), 'rock admission/retirement');
  for (const rock of rocks) {
    const actual = grid.cells[rock.index]?.entity;
    if (!actual) continue; // Missing actors already fail the admission comparison.
    for (const [field, expected] of [['kind', rock.type], ['variant', rock.variant], ['state', rock.state], ['collisionClass', rock.collisionClass], ['spawnSeed', rock.spawnSeed]])
      equal(actual[field], expected, `rock${rock.index}:${field}`);
    assert.ok(Array.isArray(state.room.pathCosts), 'Rock requires actual signed Room path costs');
    assert.ok(Array.isArray(source.pathCosts), 'Original must expose signed Room path costs');
    equal(state.room.pathCosts[rock.index], source.pathCosts[rock.index], `rock${rock.index}:destruction marker`);
  }
  return rocks;
}

function compareCommon(frame) {
  const { state, source, start, baseline, equal, presentation } = frame;
  equal(state.systemClock.systemCounter - start.systemClock.systemCounter, presentation, 'one presentation per public tick');
  equal(state.systemClock.systemCounter % 2, source.nativePhase.systemIterations % 2, 'phase');
  const player = source.players[0];
  for (const [field, actual] of [['coinInventory', state.inventory.coins], ['keyInventory', state.inventory.keys],
    ['bombInventory', state.inventory.bombs], ['redHp', state.player.hp], ['maxRedHp', state.player.maxHp]])
    equal(actual, player[field], field);
  equal([state.player.x, state.player.y], [player.position[0] * .65 + 32, player.position[1] * .65 - 46], 'player position');
  equal([state.player.vx, state.player.vy], [player.velocity[0] * .65, player.velocity[1] * .65], 'player velocity');
  equal(state.damageCooldown, player.damageCountdown, 'damage countdown');
  equal(state.gameUpdateCount - start.gameUpdateCount, source.nativePhase.fullGameUpdates - baseline.observation.nativePhase.fullGameUpdates, 'full game update clock');
  const grid = state.room.grid;
  assert.ok(grid, 'Actual authored room grid must be admitted');
  equal([grid.width, grid.height], [source.room.gridWidth, source.room.gridHeight], 'grid dimensions');

  const originalBombs = source.entities.filter(entity => entity.type === 4 && !entity.removed);
  const bombs = state.bombs.filter(bomb => bomb.motion.lifetime === 'alive');
  equal(bombs.length, originalBombs.length, 'live bomb admission/retirement');
  for (let index = 0; index < originalBombs.length; index++) {
    const original = originalBombs[index], actual = bombs[index];
    if (!actual) continue;
    // Wire vectors encode Rust f32 decimals; recover the exact source f32
    // values with no tolerance or quantization.
    equal([Math.fround(actual.motion.presentedPosition.x), Math.fround(actual.motion.presentedPosition.y)], original.position, `bomb${index}:position`);
    equal([Math.fround(actual.motion.velocity.x), Math.fround(actual.motion.velocity.y)], original.velocity, `bomb${index}:velocity`);
    equal(actual.motion.fuseUpdatesRemaining, original.bombFuseUpdatesRemaining, `bomb${index}:fuse`);
  }

  const pickups = source.entities.filter(entity => entity.type === 5);
  equal(state.room.pickups.length, pickups.length, 'pickup admission/retirement');
  for (let index = 0; index < pickups.length; index++) {
    const original = pickups[index], actual = state.room.pickups[index];
    if (!actual) continue;
    for (const field of ['variant', 'subtype', 'spawnSeed', 'animation', 'animationFrame', 'animationPlaying', 'collisionRadius'])
      equal(actual[field], original[field], `pickup${index}:${field}`);
    equal([Math.fround(actual.position.x), Math.fround(actual.position.y)], original.position, `pickup${index}:position`);
    equal(actual.collected, original.removed, `pickup${index}:collection flag`);
  }
  const handles = state.actorScheduler.actors;
  equal(handles.filter(actor => actor.kind === 'pickup').map(actor => actor.id), state.room.pickups.map(pickup => pickup.id), 'pickup actor handles');
  const sourceOrder = source.entities.filter(actor => actor.type === 1 || actor.type === 5).map(actor => actor.type === 1 ? 'player' : 'pickup');
  equal(handles.filter(actor => actor.kind === 'player' || actor.kind === 'pickup').map(actor => actor.kind), sourceOrder, 'native relative actor admission order');
  return { player, pickups };
}

async function runCase({ suite, item, scenarios, directory, server, caseResult, wasmNode, wasmSha, reusedOriginal }) {
  await writeJson(join(directory, 'case.json'), item);
  await writeFile(join(directory, 'tape.json'), await python(suite.helper, ['--case', join(directory, 'case.json')]));
  const originalDirectory = reusedOriginal ? resolve(reusedOriginal) : join(directory, 'original');
  if (reusedOriginal) {
    assert.deepEqual(JSON.parse(await readFile(join(originalDirectory, '..', 'case.json'))), item,
      'Retained Original case must equal the whole-bank declaration');
    assert.deepEqual(JSON.parse(await readFile(join(originalDirectory, '..', 'tape.json'))),
      JSON.parse(await readFile(join(directory, 'tape.json'))), 'Retained Original public tape must match');
  } else {
  await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
    '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
    '--state-observer', suite.observer, '--output', join(directory, 'original')],
  { cwd: ROOT, timeout: 180000, maxBuffer: 2 * 1024 * 1024 });
  }
  const nativeManifest = join(originalDirectory, 'manifest.json');
  let native = JSON.parse(await readFile(nativeManifest));
  caseResult.nativeManifest = nativeManifest;
  caseResult.originalReused = Boolean(reusedOriginal);
  assert.equal(native.status, 'pass');
  assert.deepEqual(native.failures, []);
  assert.equal(native.seedValue, item.seedValue);
  assert.equal(native.seedReadbackMatchesCodec, true);
  const provenance = JSON.parse(await python(suite.helper, ['--validate-original', nativeManifest]));
  assert.equal(provenance.status, 'pass');
  assert.deepEqual(provenance.failures, []);
  const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
  assert.ok(baseline && baseline.observation.status === 'pass');
  assert.equal(baseline.context.publicLogState.seedValue, item.seedValue);
  assert.equal(baseline.observation.memoryWritten, false);
  assert.equal(baseline.observation.nativePhase.systemIterations % 2, 0);
  suite.checkBaseline?.(baseline);
  const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row]));

  //The complete Native process and source validation finish before a
  //Browser is admitted. Keep only the actual rows used below.
  native = undefined;
  let browser, page, runtime, journal, beforeInputs, completedTape = false;
  const actualCommands = [];
  const send = command => { runtime.command(command); actualCommands.push(command); };
  const inputPaths = wasmNode ? [resolve(wasmNode), nativeManifest,
    join(originalDirectory, '..', 'case.json'), join(originalDirectory, '..', 'tape.json'),
    join(directory, 'case.json'), join(directory, 'tape.json'),
    join(ROOT, 'tools/foundation/composition-wasm-public.mjs')] : [];
  const inputPins = async () => Object.fromEntries(await Promise.all(inputPaths.map(async path => {
    const hash = createHash('sha256');
    for await (const chunk of createReadStream(path)) hash.update(chunk);
    return [path, hash.digest('hex')];
  })));
  const errors = [], states = [], track = {};
  let compared = 0, passed = 0, detonation;
  try {
    if (wasmNode) {
      beforeInputs = await inputPins();
      runtime = createPublicWasmRuntime({ wasm: resolve(wasmNode), expectedWasmSha: wasmSha });
      journal = createPublicWasmJournal(directory);
      //The same public reset as the existing Browser path; no resource or state override.
      runtime.reset(item.seedValue, scenarios.bootEpochSeed);
      await writeJson(join(directory, 'declared-reset-input.json'),
        { initialSeed: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed });
    } else {
    browser = await chromium.launch(browserLaunchOptions());
    caseResult.graphics = await inspectBrowserGraphics(browser);
    page = await browser.newPage();
    page.on('pageerror', error => errors.push(String(error)));
    await page.goto(`${server.url}/?manual=1`);
    await waitForReady(page);
    await page.evaluate(({ seedValue, bootEpochSeed }) => {
      window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed });
    }, { seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed });
    }
    // Menu/seed/run intervals and every tick are declared before execution;
    // deferred Music run admission is never bypassed.
    const expected = [...manualSeedCommands(item.code)];
    let start;
    if (wasmNode) {
      for (const command of manualSeedCommands(item.code)) send(command);
      start = runtime.snapshot();
      await journal.append({ kind: 'gameplay-start', state: start });
    } else start = await enterSeededRun(page, item.code);
    assert.equal(start.screen, 'playing');
    for (const [field, value] of [['seedText', item.code], ['seedValid', true], ['seededRun', true], ['runSeed', item.code], ['runSeedValue', item.seedValue]])
      assert.equal(start[field], value, `seed readback ${field}`);
    assert.equal(start.systemClock.systemCounter % 2, 0);

    let presentation = 0;
    for (const { key, frames } of item.actions) {
      const code = key ? 'Key' + key.toUpperCase() : null;
      if (key) { if (wasmNode) send({ type: 'key', code, down: true }); else await page.keyboard.down(key); expected.push({ type: 'key', code, down: true }); }
      for (let index = 0; index < frames; index++) {
        let state;
        if (wasmNode) {
          send({ type: 'tick', count: 1 }); state = runtime.snapshot();
          await journal.append({ kind: 'state', presentation: presentation + 1, state });
        } else state = await stepSnapshot(page);
        expected.push({ type: 'tick', count: 1 });
        states.push(state);
        presentation++;
        const row = rows.get(baseline.context.presentedFrame + presentation), source = row?.observation;
        assert.equal(source?.status, 'pass', `${item.name}:${presentation}: live source observation`);
        assert.equal(source.memoryWritten, false);
        assert.equal(source.nativeBinarySha256, ORIGINAL_SHA);
        if (state.explosions.length && detonation === undefined) detonation = presentation;
        const failuresBefore = caseResult.failures.length;
        const equal = (actual, expected, field) => {
          try { assert.deepEqual(actual, expected); }
          catch { caseResult.failures.push({ field, presentation, actual, expected }); }
        };
        const frame = { item, state, source, row, start, baseline, presentation, equal, track };
        Object.assign(frame, compareCommon(frame));
        suite.compare(frame);
        if (caseResult.failures.length === failuresBefore) passed++;
        compared++;
        if (presentation % 100 === 0) console.log(JSON.stringify({ case: item.name, presentation }));
      }
      if (key) { if (wasmNode) send({ type: 'key', code, down: false }); else await page.keyboard.up(key); expected.push({ type: 'key', code, down: false }); }
    }

    completedTape = true;
    if (wasmNode) {
      assert.deepEqual(actualCommands, expected, 'Every declared public ABI input is sent exactly once');
      await writeJson(join(directory, 'actual-public-wasm-commands.json'), actualCommands);
    } else {
    const trace = await readTrace(page);
    await writeFile(join(directory, 'browser-trace.json.gz'), gzipSync(JSON.stringify(trace), { level: 1 }));
    caseResult.trace = { inputTraceTruncated: trace.inputTraceTruncated, stateHistoryTruncated: trace.stateHistoryTruncated,
      stateHistoryTruncationReason: trace.stateHistoryTruncationReason, inputEncodedBytes: trace.inputEncodedBytes,
      stateEncodedBytes: trace.stateEncodedBytes, backendEvents: trace.backendEvents.length, completeStateRecords: trace.states.length };
    assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed);
    }
    Object.assign(caseResult, suite.finish({ item, states, last: states.at(-1), track, detonation }) ?? {});
    if (!wasmNode) assert.deepEqual(errors, []);
  } finally {
    try {
      Object.assign(caseResult, { comparedPresentations: compared, passedPresentations: passed, detonation, ...suite.summary?.(track) });
      if (journal) {
        caseResult.wasmJournal = await journal.close({ completed: completedTape });
        const afterInputs = await inputPins();
        await writeJson(join(directory, 'public-wasm-inputs.json'), { before: beforeInputs, after: afterInputs });
        assert.deepEqual(afterInputs, beforeInputs, 'WASM, public declarations and independent Original stay unchanged');
      }
      await writeFile(join(directory, wasmNode ? 'wasm-states.json.gz' : 'browser-states.json.gz'), gzipSync(JSON.stringify({ schema: 1, states }), { level: 1 }));
    } finally {
      try { await page?.close(); }
      finally {
      try { await browser?.close(); }
      catch (cause) {
        const error = new Error('Browser closure failed; stop before another Native launch', { cause });
        error.roomFrameBrowserClosureFailed = true;
        throw error;
      }
      }
    }
  }
}

export async function runRoomFrameSuite(suite) {
  const output = outputDirectory(suite.name);
  const wasmNode = flag('--wasm-node'), wasmSha = flag('--wasm-sha'), reusedOriginal = flag('--reuse-original');
  assert.equal(Boolean(wasmNode), Boolean(wasmSha), 'Both explicit WASM declarations required');
  if (wasmNode) assert.match(wasmSha, /^[a-f0-9]{64}$/);
  assert.ok(!reusedOriginal || wasmNode, 'Retained Original reuse requires the explicit functional-only WASM route');
  const recorded = flag('--scenarios');
  const scenarios = JSON.parse(recorded ? await readFile(recorded, 'utf8') : await python(suite.helper, ['--count', flag('--count', '2')]));
  validateScenarios(scenarios, suite.cases);
  scenarios.cases = selectCases(scenarios.cases, item => item.name);
  const selected = flag('--case');
  if (selected) scenarios.caseFilter = selected;
  assert.ok(!reusedOriginal || (selected && scenarios.cases.length === 1), 'One retained Original requires --case selecting exactly one case');
  await assertOriginalBinary();
  await writeJson(join(output, 'scenarios.json'), scenarios);

  const directory = flag('--production-directory');
  const qualifiedSha = flag('--qualified-production-sha');
  const qualifiedFiles = flag('--qualified-production-files');
  assert.equal(Boolean(qualifiedSha), Boolean(qualifiedFiles), 'Both production identity declarations required');
  if (qualifiedSha) {
    assert.match(qualifiedSha, /^[a-f0-9]{64}$/);
    assert.ok(Number.isInteger(Number(qualifiedFiles)) && Number(qualifiedFiles) > 0);
  }
  const server = wasmNode ? null : await startProductionServer({
    ...(directory ? {directory:resolve(directory)} : {}),
    ...(qualifiedSha ? {qualifiedBuildIdentity:{sha256:qualifiedSha,files:Number(qualifiedFiles)}} : {}),
  });
  const report = { schema: 2, suite: suite.name, status: 'running', originalBinarySha256: ORIGINAL_SHA, production: server?.build ?? null,
    graphics: null, scope: suite.scope, caseFilter: scenarios.caseFilter, cases: [] };
  report.captureBackend = wasmNode ? 'wasm-node' : 'browser';
  if (wasmNode) {
    report.wasmArtifact = { path: resolve(wasmNode), sha256: createHash('sha256').update(await readFile(resolve(wasmNode))).digest('hex') };
    assert.equal(report.wasmArtifact.sha256, wasmSha);
    report.unavailable = ['Browser DOM inputs', 'Renderer and PNGs', 'Audio backend and PCM'];
    report.scope += ' Public-WASM ABI functional assertions only; Browser, DOM, rendering and backend audio remain separate obligations.';
  }
  //Graphics belongs to each fresh Browser below; no Browser overlaps
  //Native capture or a following case's Native process.

  try {
    if (qualifiedSha && !wasmNode) assert.deepEqual({sha256:server.build.sha256,files:server.build.files},
      {sha256:qualifiedSha,files:Number(qualifiedFiles)}, 'Actual immutable production identity matches declaration');
    for (const item of scenarios.cases) {
      const directory = join(output, item.name);
      await mkdir(directory, { recursive: true });
      const caseResult = { ...item, status: 'running', failures: [] };
      report.cases.push(caseResult);
      try {
        await runCase({ suite, item, scenarios, directory, server, caseResult, wasmNode, wasmSha, reusedOriginal });
        caseResult.status = caseResult.failures.length ? 'fail' : 'pass';
      } catch (error) {
        caseResult.status = 'fail'; caseResult.error = String(error);
        if (error.roomFrameBrowserClosureFailed) throw error;
      }
      if (caseResult.graphics && report.graphics === null) report.graphics = caseResult.graphics;
      await writeJson(join(directory, 'report.json'), caseResult);
      console.log(JSON.stringify({ case: item.name, cohort: item.cohort, status: caseResult.status,
        comparedPresentations: caseResult.comparedPresentations, failures: caseResult.failures.length, error: caseResult.error }));
    }
    report.status = report.cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
  } catch (error) {
    report.status = 'fail'; report.error = String(error);
  } finally {
    await writeJson(join(output, 'report.json'), report);
    await server?.close();
    console.log(JSON.stringify({ status: report.status, report: join(output, 'report.json'), cases: report.cases.length }));
    process.exitCode = report.status === 'pass' ? 0 : 1;
  }
}
