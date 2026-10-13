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

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const ORIGINAL_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';
const GATE_SOURCES = ['tools/e2e/ordinary-run.mjs', 'tools/original/boot-mt-observer.py', 'tools/original/native-identity.py', 'tools/original/state-observer.py', 'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-seeds.py', 'tools/original/debug-run.py', 'tools/original/with-display.sh', 'tools/foundation/browser.mjs', 'tools/foundation/production-server.mjs', 'tools/foundation/verify.mjs', 'engine/src/lib.rs', 'engine/src/mt.rs', 'engine/src/saved_game.rs', 'engine/src/items.rs', 'engine/src/menu_lifecycle.rs', 'engine/src/seed.rs', 'engine/src/audio.rs', 'engine/src/music.rs', 'web/main.ts', 'web/trace.ts'];
const hashes = async () => Object.fromEntries(await Promise.all(GATE_SOURCES.map(async path => [path, createHash('sha256').update(await readFile(join(ROOT, path))).digest('hex')])));
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const savedGame = process.argv.includes('--saved-game');
const originalBank = value('--original-bank') ? resolve(value('--original-bank')) : null;
const bankReportBytes = originalBank ? await readFile(join(originalBank, 'report.json')) : null;
const bankReport = bankReportBytes ? JSON.parse(bankReportBytes) : null;
if (bankReport) {
  assert.equal(bankReport.originalBinarySha256, ORIGINAL_SHA);
  assert.equal(bankReport.gateSourcesStable, true);
}
const levelDescriptors = process.argv.includes('--level-descriptors') || savedGame;
const output = resolve(value('--output') || join(ROOT, savedGame ? '.cache/e2e/saved-game' : levelDescriptors ? '.cache/e2e/level-descriptors' : '.cache/e2e/ordinary-run', String(Date.now())));
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
if (savedGame) report.scope += ' SavedGame Clear/ordinary-Level/ItemPool/Player copy ownership and every bounded saved seed pair are blocking. Full post-Game MT, actual grid occupancy and saved counter are compared with their actual diagnostic results. Grid object admission/secondary seed routing remains unimplemented and explicitly exposed; the null-cell save law uses the actual supported live grid model.';
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
        assert.equal(summary.currentRoomRecord, observed.savedRun.currentRoomRecord, 'Actual current descriptor row identity must match source');
        assert.ok(observed.currentRoomSeedSamples?.length === 9, 'Current Room.Save requires independently observed active-row samples');
        const currentPairDifferences = [];
        for (const [index, source] of observed.currentRoomSeedSamples.entries()) {
          const sample = summary.currentRoomSeedSamples[index];
          assert.equal(sample.gridIndex, source.gridIndex);
          assert.deepEqual(sample.levelSeeds, sample.savedSeeds, 'Production current row must retain actual Level→Saved copy ownership');
          assert.deepEqual(source.levelSeeds, source.savedSeeds, 'Native current row must retain actual Level→Saved copy ownership');
          for (const field of ['levelSeeds', 'savedSeeds', 'occupancy']) if (JSON.stringify(sample[field]) !== JSON.stringify(source[field])) currentPairDifferences.push({gridIndex: source.gridIndex, field, original: source[field], production: sample[field], sourceGridObject: source.object || null});
        }
        result.currentRoomSeedDiagnostic = { status: currentPairDifferences.length ? 'fail' : 'pass', roomRecord: summary.currentRoomRecord, comparedCells: 9, differences: currentPairDifferences, reason: 'Actual current-row numerical/occupancy mismatches remain unaccepted; source samples never enter production' };
        const emptyIndices = state.room.grid.cells.flatMap((cell, index) => cell.entity ? [] : [index]);
        result.gridOccupancyDiagnostic = { status: JSON.stringify(emptyIndices) === JSON.stringify(observed.liveGrid.emptyIndices) ? 'pass' : 'fail', originalNullCells: observed.liveGrid.emptyIndices.length, productionNullCells: emptyIndices.length, originalOccupiedProductionNull: emptyIndices.filter(index => !observed.liveGrid.emptyIndices.includes(index)), originalNullProductionOccupied: observed.liveGrid.emptyIndices.filter(index => !emptyIndices.includes(index)), reason: 'Grid factory admission and secondary state remain incomplete; observed occupancy never changes production state' };
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
        assert.deepEqual(summary.occupiedGridSeedsPending, state.room.grid.cells.flatMap((cell, index) => cell.entity ? [index] : []));
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
        result.savedGameOwnership = { status: 'pass', comparedPairs: expectedPairs.length, copiedPairs: expectedPairs.filter(pair => copiedRecords.includes(pair.roomRecord)).length, emptyCells: emptyIndices.length, occupiedSecondarySeedsPending: summary.occupiedGridSeedsPending.length };
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
      result.status = 'pass'; result.comparedMenuCheckpoints = 4; result.comparedStateWords = 2496; result.comparedGridPairs = 36; result.comparedStatPoses = 716;
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
