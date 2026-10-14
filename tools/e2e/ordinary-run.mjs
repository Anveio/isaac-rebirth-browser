// Actual no-dialog Character seed admission. Native observations never enter WASM.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { createHash, randomInt } from 'node:crypto';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, waitForReady } from '../foundation/verify.mjs';
import { compareSavedRoomGrid } from '../foundation/saved-grid-compare.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const ORIGINAL_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';
const GATE_SOURCES = ['tools/e2e/ordinary-run.mjs', 'tools/foundation/saved-grid-compare.mjs', 'tools/original/boot-mt-observer.py', 'tools/original/native-identity.py', 'tools/original/state-observer.py', 'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-seeds.py', 'tools/original/debug-run.py', 'tools/original/with-display.sh', 'tools/foundation/browser.mjs', 'tools/foundation/production-server.mjs', 'tools/foundation/verify.mjs', 'engine/src/lib.rs', 'engine/src/mt.rs', 'engine/src/saved_game.rs', 'engine/src/player_save.rs', 'engine/src/player_items.rs', 'engine/src/grid_records.rs', 'engine/src/room_costs.rs', 'engine/src/grid.rs', 'engine/src/room.rs', 'engine/src/items.rs', 'engine/src/menu_lifecycle.rs', 'engine/src/seed.rs', 'engine/src/audio.rs', 'engine/src/music.rs', 'web/engine.ts', 'web/main.ts', 'web/trace.ts'];
const hashes = async () => Object.fromEntries(await Promise.all(GATE_SOURCES.map(async path => [path, createHash('sha256').update(await readFile(join(ROOT, path))).digest('hex')])));
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const ambientGrid = process.argv.includes('--ambient-grid');
const gridRecords = process.argv.includes('--grid-records') || ambientGrid;
const savedGame = process.argv.includes('--saved-game') || gridRecords;
const originalBank = value('--original-bank') ? resolve(value('--original-bank')) : null;
const bankReportBytes = originalBank ? await readFile(join(originalBank, 'report.json')) : null;
const bankReport = bankReportBytes ? JSON.parse(bankReportBytes) : null;
if (bankReport) {
  assert.equal(bankReport.originalBinarySha256, ORIGINAL_SHA);
  assert.equal(bankReport.gateSourcesStable, true);
}
const levelDescriptors = process.argv.includes('--level-descriptors') || savedGame;
const output = resolve(value('--output') || join(ROOT, ambientGrid ? '.cache/e2e/ambient-grid' : gridRecords ? '.cache/e2e/grid-records' : savedGame ? '.cache/e2e/saved-game' : levelDescriptors ? '.cache/e2e/level-descriptors' : '.cache/e2e/ordinary-run', String(Date.now())));
const count = Number(value('--count') || 4);
assert.ok(Number.isInteger(count) && count >= 2 && count <= 16);
let cases = [{ name: 'zero-epoch', bootEpochSeed: 0, initialSeed: 1 }, { name: 'fixed-epoch', bootEpochSeed: 1700000000, initialSeed: 0xffffffff }, ...Array.from({ length: count }, (_, index) => ({ name: `random-${index}`, bootEpochSeed: randomInt(0x100000000), initialSeed: randomInt(1, 0x100000000) }))];
if (value('--scenarios')) cases = JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')).cases;
assert.ok(Array.isArray(cases) && cases.length >= 4 && cases.length <= 18);
assert.equal(new Set(cases.map(c => c.name)).size, cases.length);
for (const c of cases) {
  assert.ok(/^[a-zA-Z0-9_-]{1,32}$/.test(c.name));
  assert.ok(Number.isInteger(c.initialSeed) && c.initialSeed > 0 && c.initialSeed <= 0xffffffff);
  assert.ok(Number.isInteger(c.bootEpochSeed) && c.bootEpochSeed >= 0 && c.bootEpochSeed <= 0xffffffff);
}
// Construct both plans before either trace. No Tab, glyph or manual commitment.
const commands = [];
for (let index = 0; index < 4; index++) commands.push({ type: 'key', code: 'Enter', down: true }, { type: 'tick', count: 12 }, { type: 'key', code: 'Enter', down: false }, { type: 'tick', count: index === 3 ? 300 : 72 });
const nativeTape = [{ type: 'step', frames: 180 }, { type: 'capture', label: 'intro' }, { type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Return', down: false }, { type: 'step', frames: 72 }, { type: 'capture', label: 'title' }];
for (const [index, label] of ['profile', 'main', 'character', 'gameplay-start'].entries()) nativeTape.push({ type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Return', down: false }, { type: 'step', frames: index === 3 ? 300 : 72 }, { type: 'capture', label });
await mkdir(output, { recursive: true });
await writeFile(join(output, 'scenarios.json'), JSON.stringify({ schema: 1, entropy: 'Node crypto OS entropy for independent boot epoch and browser reset seed; no requested run seed', cases }, null, 2) + '\n');
assert.equal(createHash('sha256').update(await readFile(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64'))).digest('hex'), ORIGINAL_SHA);
const gateSources = await hashes();
const production = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = { schema: 1, status: 'running', originalBinarySha256: ORIGINAL_SHA, gateSources, production: production.build, graphics: await inspectBrowserGraphics(browser), scope: 'Ordinary public no-dialog Character-to-run admission uses the generated menu seed, preserving unseeded eligibility. Complete shared MT, saved/menu fields, nine grid pairs and179 stat poses compare through Character; actual run seed/code/stage and System admission are blocking. Run shared-MT differences remain explicit diagnostics; no original state becomes browser input.', cases: [] };
report.nativeReference = originalBank ? { mode: 'reused-original-bank', directory: originalBank, reportSha256: createHash('sha256').update(bankReportBytes).digest('hex'), originalReportStatus: bankReport.status, originalGateSources: bankReport.gateSources } : { mode: 'fresh-owned-original-processes' };
if (levelDescriptors) report.scope += ' Level descriptor lifecycle additionally compares nine source Level seed pairs through menus and after actual Game admission; the real internal table is regenerated by587ec0 before floor construction. Full post-admission shared MT is retained and compared, with later room/effect differences explicit.';
if (gridRecords) report.scope += ' Supported fresh GridBase owners execute actual construction and primary overrides in source authored/door/wall order before Save. Complete seven-field records for sampled actual room cells additionally compare against native Level and Saved records, with active-row seeds/whole occupancy still blocking. Padding seed pairs remain blocking; no padding-record metadata acceptance or durable continue/load is claimed.';
if (ambientGrid) report.scope += ' Normal shape0 variable decorations use the actual descriptor-owned RNG35, occupied-pointer guard, signed path-cost guard and caller-seed-before-constructor order. All448 actual live path-cost words additionally compare as blocking fields. Wall/Door Init stores are admitted; later weighted-path reservations, decay, other derived callbacks and transition admission remain source gaps.';
if (savedGame) report.scope += ' SavedGame Clear/ordinary-Level/ItemPool/Player copy ownership and every bounded saved seed pair are blocking. All347 owned Player count slots,450 current child RNG states and4 active/reserve fields are blocking saved-owner comparisons. Actual current-row seed pairs/occupancy and whole live-grid occupancy are blocking. Full post-Game MT and saved counter retain diagnostic results. Only admitted grid owners supply occupied seed pairs; unsupported room/effect objects remain explicit gaps. The null-cell save law uses the actual supported live grid model. The Player restore store helper does not implement public Continue/profile loading or claim those callbacks executed.';
try {
  for (const scenario of cases) {
    const directory = join(output, scenario.name); await mkdir(directory, { recursive: true });
    const result = { ...scenario, status: 'running' }; report.cases.push(result);
    const checkpoints = []; let page;
    try {
      await writeFile(join(directory, 'tape.json'), JSON.stringify(nativeTape, null, 2) + '\n');
      if (!originalBank) await run('bash', [join(ROOT, 'tools/original/with-display.sh'), 'python3', join(ROOT, 'tools/original/foundation-clock-run.py'), '--ordinary-run', '--boot-epoch-seed', String(scenario.bootEpochSeed), '--tape', join(directory, 'tape.json'), '--state-observer', join(ROOT, 'tools/original/boot-mt-observer.py'), '--output', join(directory, 'original')], { cwd: ROOT, timeout: 240000, maxBuffer: 1024 * 1024 });
      const nativeDirectory = join(originalBank || output, scenario.name, 'original');
      const bytes = await readFile(join(nativeDirectory, 'manifest.json')), native = JSON.parse(bytes);
      result.nativeManifestSha256 = createHash('sha256').update(bytes).digest('hex');
      result.nativeDirectory = nativeDirectory;
      if (bankReport) {
        const reference = bankReport.cases.find(item => item.name === scenario.name);
        assert.ok(reference); assert.equal(reference.bootEpochSeed, scenario.bootEpochSeed); assert.equal(reference.initialSeed, scenario.initialSeed);
        assert.equal(result.nativeManifestSha256, reference.nativeManifestSha256);
      }
      assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      assert.equal(native.seedMode, 'ordinary-character-menu'); assert.equal(native.bootEpochSeed, scenario.bootEpochSeed);
      assert.deepEqual(native.tape, nativeTape); assert.equal(native.seedReadbackMatchesCodec, true);
      const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
      for (const observation of observations.values()) {
        assert.equal(observation.nativeBinarySha256, ORIGINAL_SHA); assert.equal(observation.memoryWritten, false); assert.equal(observation.executableMappingsRead, false);
      }
      page = await browser.newPage(); const errors = []; page.on('pageerror', error => errors.push(String(error)));
      await page.goto(`${production.url}/?manual=1`); await waitForReady(page);
      // Both browser inputs come only from the saved public case, never native.
      await page.evaluate(({ initialSeed, bootEpochSeed }) => { window.game.pause(); window.game.reset(initialSeed, { trace: true, bootEpochSeed }); }, scenario);
      const compareMenu = async label => {
        const state = await page.evaluate(() => window.game.snapshot()), observed = observations.get(label), boot = state.bootLifecycle;
        checkpoints.push({ label, state }); assert.ok(observed && observed.status === 'pass');
        assert.equal(state.screen, label); assert.equal(observed.systemState, 1);
        assert.equal(observed.title.menuScreen, { title: 1, profile: 2, main: 3, character: 4 }[label]);
        assert.deepEqual(boot.shared, { words: observed.mtState, index: observed.mtIndex }, `${scenario.name}:${label}: all624 MT words`);
        assert.equal(boot.savedPlayerSeed, observed.stateSeed9688);
        assert.equal(boot.menuRunSeed, observed.title.menuRunSeed);
        assert.equal(boot.menuSeedText, observed.title.menuSeedText.replaceAll(' ', ''));
        assert.equal(state.seedText, ''); assert.equal(observed.title.characterSeedInput, '');
        assert.equal(state.seededRun, false); assert.equal(observed.title.characterSeedCommitted, false);
        assert.equal(state.achievementEligibility, true);
        assert.equal(state.savedRunSummary, null);
        assert.deepEqual(boot.secretPoses, observed.title.statsSamples.map(sample => sample.pose));
        assert.deepEqual(boot.savedGridSeedSamples, observed.gridSeedSamples.map(sample => ({ roomRecord: sample.roomRecord, gridIndex: sample.gridIndex, seeds: [sample.seed10, sample.seed18] })));
        if (levelDescriptors) assert.deepEqual(boot.levelGridSeedSamples, observed.levelGridSeedSamples.map(sample => ({ roomRecord: sample.roomRecord, gridIndex: sample.gridIndex, seeds: [sample.seed10, sample.seed18] })), `${scenario.name}:${label}: actual Level grid seed pairs`);
      };
      await compareMenu('title');
      for (const label of ['profile', 'main', 'character']) {
        await page.keyboard.down('Enter'); await page.evaluate(() => window.game.step(12));
        await page.keyboard.up('Enter'); await page.evaluate(() => window.game.step(72));
        await compareMenu(label);
      }
      const characterSeed = checkpoints.at(-1).state.bootLifecycle.menuRunSeed;
      await page.keyboard.down('Enter'); await page.evaluate(() => window.game.step(12));
      await page.keyboard.up('Enter'); await page.evaluate(() => window.game.step(300));
      const state = await page.evaluate(() => window.game.snapshot()), observed = observations.get('gameplay-start');
      checkpoints.push({ label: 'gameplay-start', state });
      assert.equal(state.screen, 'playing'); assert.equal(observed.systemState, 2);
      assert.equal(state.runSeedValue, characterSeed, 'Actual generated Character seed must own run admission');
      assert.equal(state.runSeedValue, native.observedState.seedValue); assert.equal(state.runSeedValue, observed.gameRunSeed);
      assert.equal(state.runSeed.replaceAll(' ', ''), native.observedState.seed.replaceAll(' ', ''));
      assert.equal(state.runSeed, observed.gameSeedText.replaceAll(' ', ''));
      assert.equal(state.stageSeed, native.observedState.stageSeed); assert.equal(state.stage, native.observedState.stage);
      assert.equal(state.seededRun, false); assert.equal(state.seedValid, false); assert.equal(state.seedText, '');
      assert.equal(state.achievementEligibility, true);
      // Native startup destroys MenuManager. The live seeded flags now belong
      // to Game/System, after Character's false committed flag was checked.
      assert.equal(observed.title.loaded, false);
      assert.equal(observed.gameSeededRun, false); assert.equal(observed.systemSeededRun, false);
      const actual = state.bootLifecycle.shared, differingWords = actual.words.flatMap((word, index) => word === observed.mtState[index] ? [] : [{ index, original: observed.mtState[index], production: word }]);
      result.runMtDiagnostic = { status: differingWords.length || actual.index !== observed.mtIndex ? 'fail' : 'pass', originalIndex: observed.mtIndex, productionIndex: actual.index, differingWords: differingWords.length, firstDifferences: differingWords.slice(0, 8), reason: 'Run/room/effect shared-stream consumers remain incomplete; source values and cursor adjustments are never supplied to production' };
      if (savedGame) {
        const expectedPairs = observed.gridSeedSamples.map(sample => ({ roomRecord: sample.roomRecord, gridIndex: sample.gridIndex, seeds: [sample.seed10, sample.seed18] }));
        const pairDifferences = state.bootLifecycle.savedGridSeedSamples.flatMap((sample, index) => JSON.stringify(sample) === JSON.stringify(expectedPairs[index]) ? [] : [{ ...sample, original: expectedPairs[index].seeds }]);
        result.savedGridDiagnostic = { status: pairDifferences.length ? 'fail' : 'pass', comparedPairs: expectedPairs.length, differences: pairDifferences };
        const summary = state.savedRunSummary; assert.ok(summary && observed.savedRun);
        Object.assign(result, compareSavedRoomGrid(summary, state.room.grid, observed, { requireRecords: gridRecords }));
        const savedPlayer = observed.savedRun.playerItems, browserPlayer = summary.playerItems;
        assert.ok(savedPlayer && browserPlayer, 'Both actual Player save owners must be observed');
        let playerChecks = 0;
        for (const [field, count] of [['ownedCounts', 347], ['itemRngStates', 347],
          ['trinketRngStates', 62], ['pocketRngStates', 41]]) {
          assert.equal(savedPlayer[field]?.length, count, `Native saved ${field} must be complete`);
          assert.equal(browserPlayer[field]?.length, count, `Browser saved ${field} must be complete`);
          for (let index = 0; index < count; index++) {
            const expected = savedPlayer[field][index], actual = browserPlayer[field][index];
            assert.ok(Number.isInteger(expected) && Number.isInteger(actual));
            playerChecks++;
            if (actual !== expected) result.savedGridRequiredFailures.push({
              field: `player.savedItems.${field}.${index}`, expected, actual,
            });
          }
        }
        for (const field of ['activeId', 'activeCharge', 'activeSubordinateCharge', 'reserveCharge']) {
          const expected = savedPlayer[field], actual = browserPlayer[field];
          assert.ok(Number.isInteger(expected) && Number.isInteger(actual));
          playerChecks++;
          if (actual !== expected) result.savedGridRequiredFailures.push({
            field: `player.savedItems.${field}`, expected, actual,
          });
        }
        result.savedPlayerRequiredChecks = playerChecks;
        if (ambientGrid) {
          assert.equal(observed.pathCostContext?.shapeIndex, 0, 'Source decorator must use actual normal shape0');
          assert.ok(![11, 13].includes(observed.pathCostContext.backdropType), 'Source special backdrop decorator exclusion must be inactive');
          assert.equal(observed.pathCosts?.length, 448, 'Actual source cost table must be read independently');
          assert.equal(state.room.pathCosts?.length, 448, 'Actual production Room.Init must own the full signed table');
          const differences = [];
          for (const [index, expected] of observed.pathCosts.entries()) {
            const actual = state.room.pathCosts[index];
            assert.ok(Number.isInteger(expected) && expected >= -0x80000000 && expected <= 0x7fffffff);
            assert.ok(Number.isInteger(actual) && actual >= -0x80000000 && actual <= 0x7fffffff);
            if (actual !== expected) differences.push({ index, expected, actual });
          }
          result.pathCostDiagnostic = { status: differences.length ? 'fail' : 'pass', comparedWords: 448, differences,
            reason: 'Whole actual signed Room+e98 table is blocking; no selectors, collision labels or native values supply production costs' };
          result.savedGridRequiredFailures.push(...differences.map(difference => ({
            field: `room.pathCosts.${difference.index}`, expected: difference.expected, actual: difference.actual,
          })));
        }
        const emptyIndices = state.room.grid.cells.flatMap((cell, index) => cell.entity ? [] : [index]);
        assert.equal(summary.runSeed, observed.savedRun.seedText.replaceAll(' ', ''));
        assert.equal(summary.seededRun, observed.savedRun.seededRun);
        assert.equal(summary.stage, observed.savedRun.stage);
        assert.equal(summary.redHp, observed.savedRun.redHp); assert.equal(summary.maxRedHp, observed.savedRun.maxRedHp);
        assert.deepEqual(summary.inventory, observed.savedRun.inventory);
        // The first frozen bank called this field allocatedRoomRecords while
        // reading the actual !=-3 copy predicate. Preserve that bank verbatim;
        // only its semantic label is corrected here, never its source values.
        const copiedRecords = observed.savedRun.copiedRoomRecords || (originalBank && observed.savedRun.allocatedRoomRecords);
        assert.ok(Array.isArray(copiedRecords));
        assert.equal(summary.copiedDescriptorCount, copiedRecords.length);
        if (observed.savedRun.copiedRoomRecords) assert.equal(summary.descriptorCount, observed.savedRun.allocatedRoomRecords.length);
        assert.equal(state.bootLifecycle.savedPlayerSeed, observed.stateSeed9688);
        const savedPools = structuredClone(observed.savedItemPools); assert.ok(savedPools);
        for (const sample of savedPools.weightSamples) {
          assert.equal(sample.weight, sample.liveWeight, 'Source saved item weight must copy the actual live owner');
          delete sample.liveWeight;
        }
        assert.deepEqual(summary.itemPools, savedPools, 'Actual saved ItemPool fields must equal source');
        assert.equal(summary.emptyGridCellsSaved, emptyIndices.length);
        assert.deepEqual(summary.occupiedGridSeedsPending, state.room.grid.cells.flatMap((cell, index) => cell.entity && !cell.entity.sourceOwner ? [index] : []));
        for (const [index, pair] of expectedPairs.entries()) {
          const levelPair = observed.levelGridSeedSamples[index];
          const copied = copiedRecords.includes(pair.roomRecord);
          if (copied) {
            assert.deepEqual(pair.seeds, [levelPair.seed10, levelPair.seed18], 'Source ordinary SavedGame row must copy Level ownership');
            assert.deepEqual(state.bootLifecycle.savedGridSeedSamples[index], state.bootLifecycle.levelGridSeedSamples[index], 'Production ordinary SavedGame row must copy Level ownership');
            assert.deepEqual(state.bootLifecycle.savedGridSeedSamples[index], pair, 'Actual saved sample must equal source');
            assert.equal(observed.savedRows.find(row => row.roomRecord === pair.roomRecord).gridIndex, observed.levelRows.find(row => row.roomRecord === pair.roomRecord).gridIndex);
          } else {
            assert.equal(observed.levelRows.find(row => row.roomRecord === pair.roomRecord).gridIndex, -3);
          }
        }
        assert.deepEqual(state.bootLifecycle.savedGridSeedSamples, expectedPairs, 'Every bounded saved seed pair must equal source');
        result.savedGameOwnership = { status: 'pass', comparedPairs: expectedPairs.length, copiedPairs: expectedPairs.filter(pair => copiedRecords.includes(pair.roomRecord)).length, currentRowCopyStatus: 'pass', currentRowComparedPairs: 9, currentRowComparedRecords: gridRecords ? 12 : 0, emptyCells: emptyIndices.length, occupiedSecondarySeedsPending: summary.occupiedGridSeedsPending.length };
        result.savedCounterDiagnostic = { status: summary.gameUpdateCount === observed.savedRun.gameUpdateCount ? 'pass' : 'fail', original: observed.savedRun.gameUpdateCount, production: summary.gameUpdateCount };
      }
      if (levelDescriptors) {
        assert.deepEqual(state.bootLifecycle.levelGridSeedSamples, observed.levelGridSeedSamples.map(sample => ({ roomRecord: sample.roomRecord, gridIndex: sample.gridIndex, seeds: [sample.seed10, sample.seed18] })), `${scenario.name}: actual admitted Level seed pairs`);
        assert.notDeepEqual(state.bootLifecycle.levelGridSeedSamples, checkpoints.find(checkpoint => checkpoint.label === 'character').state.bootLifecycle.levelGridSeedSamples, 'Actual Level admission must replace its constructor seed table');
        result.comparedLevelGridPairs = 45;
      }
      const trace = await page.evaluate(() => window.game.trace());
      assertInputProvenance(trace, commands, scenario.initialSeed, scenario.bootEpochSeed);
      await writeFile(join(directory, 'trace.json'), JSON.stringify(trace) + '\n'); assert.deepEqual(errors, []);
      result.status = result.savedGridRequiredFailures?.length ? 'fail' : 'pass';
      if (result.status === 'fail') result.error = `${result.savedGridRequiredFailures.length} required SavedGame current-row/grid comparisons failed`;
      result.comparedMenuCheckpoints = 4; result.comparedStateWords = 2496; result.comparedGridPairs = 36; result.comparedStatPoses = 716;
      result.actualRun = { seedValue: state.runSeedValue, code: state.runSeed, stageSeed: state.stageSeed, seededRun: state.seededRun, achievementEligibility: state.achievementEligibility };
      result.inputProvenance = { status: 'pass', expectedCommands: commands.length, observedCommands: trace.commands.length, initialSeed: scenario.initialSeed, bootEpochSeed: scenario.bootEpochSeed };
    } catch (error) { result.status = 'fail'; result.error = String(error); }
    finally {
      await writeFile(join(directory, 'expected-commands.json'), JSON.stringify(commands, null, 2) + '\n');
      await writeFile(join(directory, 'browser-checkpoints.json'), JSON.stringify(checkpoints) + '\n');
      if (page) {
        await page.evaluate(() => window.game.trace()).then(async trace => {
          await writeFile(join(directory, 'last-trace.json'), JSON.stringify(trace) + '\n');
          assertInputProvenance(trace, commands, scenario.initialSeed, scenario.bootEpochSeed);
          result.inputProvenance = { status: 'pass', expectedCommands: commands.length, observedCommands: trace.commands.length, initialSeed: scenario.initialSeed, bootEpochSeed: scenario.bootEpochSeed };
        }).catch(error => { result.status = 'fail'; result.traceCaptureError = String(error); });
        await page.close().catch(error => { result.status = 'fail'; result.cleanupError = String(error); });
      }
      await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
    }
  }
  report.gateSourcesEnd = await hashes(); report.gateSourcesStable = JSON.stringify(gateSources) === JSON.stringify(report.gateSourcesEnd);
  assert.equal(report.gateSourcesStable, true, 'Ordinary admission gate sources changed during capture');
  report.status = report.cases.every(c => c.status === 'pass') ? 'pass' : 'fail';
  if (report.status === 'fail') throw new AggregateError(report.cases.filter(c => c.status === 'fail').map(c => new Error(`${c.name}: ${c.error}`)), 'Ordinary admission cases failed; every selected case was attempted');
} catch (error) { report.status = 'fail'; report.error = String(error); throw error; }
finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser.close(); await production.close();
  console.log(JSON.stringify({ status: report.status, cases: report.cases.length, report: join(output, 'report.json') }));
}
