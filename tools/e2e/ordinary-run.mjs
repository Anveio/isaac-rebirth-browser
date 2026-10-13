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
const GATE_SOURCES = ['tools/e2e/ordinary-run.mjs', 'tools/original/boot-mt-observer.py', 'tools/original/native-identity.py', 'tools/original/state-observer.py', 'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-seeds.py', 'tools/original/debug-run.py', 'tools/original/with-display.sh', 'tools/foundation/browser.mjs', 'tools/foundation/production-server.mjs', 'tools/foundation/verify.mjs', 'engine/src/lib.rs', 'engine/src/mt.rs', 'engine/src/menu_lifecycle.rs', 'engine/src/seed.rs', 'engine/src/audio.rs', 'engine/src/music.rs', 'web/main.ts', 'web/trace.ts'];
const hashes = async () => Object.fromEntries(await Promise.all(GATE_SOURCES.map(async path => [path, createHash('sha256').update(await readFile(join(ROOT, path))).digest('hex')])));
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/ordinary-run', String(Date.now())));
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
try {
  for (const scenario of cases) {
    const directory = join(output, scenario.name); await mkdir(directory, { recursive: true });
    const result = { ...scenario, status: 'running' }; report.cases.push(result);
    const checkpoints = []; let page;
    try {
      await writeFile(join(directory, 'tape.json'), JSON.stringify(nativeTape, null, 2) + '\n');
      await run('bash', [join(ROOT, 'tools/original/with-display.sh'), 'python3', join(ROOT, 'tools/original/foundation-clock-run.py'), '--ordinary-run', '--boot-epoch-seed', String(scenario.bootEpochSeed), '--tape', join(directory, 'tape.json'), '--state-observer', join(ROOT, 'tools/original/boot-mt-observer.py'), '--output', join(directory, 'original')], { cwd: ROOT, timeout: 240000, maxBuffer: 1024 * 1024 });
      const bytes = await readFile(join(directory, 'original/manifest.json')), native = JSON.parse(bytes);
      result.nativeManifestSha256 = createHash('sha256').update(bytes).digest('hex');
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
        assert.deepEqual(boot.secretPoses, observed.title.statsSamples.map(sample => sample.pose));
        assert.deepEqual(boot.savedGridSeedSamples, observed.gridSeedSamples.map(sample => ({ roomRecord: sample.roomRecord, gridIndex: sample.gridIndex, seeds: [sample.seed10, sample.seed18] })));
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
      assert.equal(state.stageSeed, native.observedState.stageSeed); assert.equal(state.stage, native.observedState.stage);
      assert.equal(state.seededRun, false); assert.equal(state.seedValid, false); assert.equal(state.seedText, '');
      assert.equal(state.achievementEligibility, true); assert.equal(observed.title.characterSeedCommitted, false); assert.equal(observed.title.characterSeedInput, '');
      assert.equal(state.bootLifecycle.menuRunSeed, observed.title.menuRunSeed);
      assert.equal(state.bootLifecycle.menuSeedText, observed.title.menuSeedText.replaceAll(' ', ''));
      const actual = state.bootLifecycle.shared, differingWords = actual.words.flatMap((word, index) => word === observed.mtState[index] ? [] : [{ index, original: observed.mtState[index], production: word }]);
      result.runMtDiagnostic = { status: differingWords.length || actual.index !== observed.mtIndex ? 'fail' : 'pass', originalIndex: observed.mtIndex, productionIndex: actual.index, differingWords: differingWords.length, firstDifferences: differingWords.slice(0, 8), reason: 'Run/room/effect shared-stream consumers remain incomplete; source values and cursor adjustments are never supplied to production' };
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
        await page.evaluate(() => window.game.trace()).then(trace => writeFile(join(directory, 'last-trace.json'), JSON.stringify(trace) + '\n')).catch(error => { result.status = 'fail'; result.traceCaptureError = String(error); });
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
