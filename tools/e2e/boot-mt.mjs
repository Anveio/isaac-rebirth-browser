// Live source startup/menu lifecycle; native state never enters production WASM.
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
import { assertInputProvenance, assertSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const ORIGINAL_SHA = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';
const GATE_SOURCES = ["tools/e2e/boot-mt.mjs", "tools/original/boot-mt-observer.py", "tools/original/native-identity.py", "tools/original/state-observer.py", "tools/original/foundation-clock-run.py", "tools/original/foundation-clock-gate.c", "tools/original/foundation-seeds.py", "tools/original/debug-run.py", "tools/original/with-display.sh", "tools/foundation/browser.mjs", "tools/foundation/production-server.mjs", "tools/foundation/verify.mjs"];
const sourceHashes = async () => Object.fromEntries(await Promise.all(GATE_SOURCES.map(async path => [path, createHash('sha256').update(await readFile(join(ROOT, path))).digest('hex')])));
const gateSources = await sourceHashes();
const value = flag => { const i = process.argv.indexOf(flag); return i < 0 ? undefined : process.argv[i + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/boot-mt', String(Date.now())));
const count = Number(value('--count') || 4);
assert.ok(Number.isInteger(count) && count >= 2 && count <= 16);
await mkdir(output, { recursive: true });
const generated = await run('python3', [join(ROOT, 'tools/original/foundation-seeds.py'), '--count', String(count)], { cwd: ROOT });
const vectors = JSON.parse(generated.stdout).vectors;
let cases = [
  { name: 'zero-epoch', bootEpochSeed: 0, code: 'QSYVKMSE' },
  { name: 'fixed-epoch', bootEpochSeed: 1700000000, code: 'ZG800TRJ' },
  ...vectors.map((vector, i) => ({ name: `random-${i}`, bootEpochSeed: randomInt(0x100000000), code: vector.code.replaceAll(' ', ''), seedValue: vector.seedValue })),
];
if (value('--scenarios')) cases = JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')).cases;
assert.ok(Array.isArray(cases) && cases.length >= 4 && cases.length <= 18);
assert.equal(new Set(cases.map(c => c.name)).size, cases.length);
for (const c of cases) {
  assert.ok(/^[a-zA-Z0-9_-]{1,32}$/.test(c.name) && typeof c.code === 'string' && c.code.length === 8);
  assert.ok(Number.isInteger(c.bootEpochSeed) && c.bootEpochSeed >= 0 && c.bootEpochSeed <= 0xffffffff);
}
const binary = await readFile(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64'));
assert.equal(createHash('sha256').update(binary).digest('hex'), ORIGINAL_SHA);
await writeFile(join(output, 'scenarios.json'), JSON.stringify({ schema: 1, entropy: 'OS secrets run seeds and Node crypto randomInt boot epochs', cases }, null, 2) + '\n');

const production = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = { schema: 1, status: 'running', gateSources, originalBinarySha256: ORIGINAL_SHA, production: production.build,
  graphics: await inspectBrowserGraphics(browser),
  scope: 'Causal startup and initialized menu fields at Title: full 624-word MT state, saved player seed, nine saved grid pairs, menu run seed and 179 stat poses. Later menu navigation, run/room/ambient random routing and tear fall uniforms remain incomplete.', cases: [] };
try {
  for (const scenario of cases) {
    const directory = join(output, scenario.name); await mkdir(directory, { recursive: true });
    const result = { ...scenario, status: 'running' }; report.cases.push(result);
    let page;
    try {
      const tapeResult = await run('python3', ['-c', "import importlib.util,json,sys;from pathlib import Path;p=Path('tools/original/foundation-clock-run.py');s=importlib.util.spec_from_file_location('clock',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);t=m.default_tape(sys.argv[1]);print(json.dumps(t[:next(i for i,a in enumerate(t) if a.get('label')=='gameplay-start')+1]))", scenario.code], { cwd: ROOT });
      await writeFile(join(directory, 'tape.json'), tapeResult.stdout);
      await run('bash', [join(ROOT, 'tools/original/with-display.sh'), 'python3', join(ROOT, 'tools/original/foundation-clock-run.py'),
        '--seed', scenario.code, '--boot-epoch-seed', String(scenario.bootEpochSeed), '--tape', join(directory, 'tape.json'),
        '--state-observer', join(ROOT, 'tools/original/boot-mt-observer.py'), '--output', join(directory, 'original')], { cwd: ROOT, timeout: 180000, maxBuffer: 1024 * 1024 });
      const bytes = await readFile(join(directory, 'original/manifest.json')); const native = JSON.parse(bytes);
      result.nativeManifestSha256 = createHash('sha256').update(bytes).digest('hex');
      assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      assert.equal(native.bootEpochSeed, scenario.bootEpochSeed); assert.equal(native.seedReadbackMatchesCodec, true);
      const observed = native.debugObservations.find(row => row.context.label === 'title').observation;
      assert.equal(observed.nativeBinarySha256, ORIGINAL_SHA);
      assert.equal(observed.memoryWritten, false); assert.equal(observed.executableMappingsRead, false);
      assert.equal(observed.title.loaded, true);
      page = await browser.newPage(); const errors = []; page.on('pageerror', error => errors.push(String(error)));
      await page.goto(`${production.url}/?manual=1`); await waitForReady(page);
      const initialSeed = scenario.seedValue || native.seedValue;
      const state = await page.evaluate(({ initialSeed, bootEpochSeed }) => { window.game.pause(); return window.game.reset(initialSeed, { bootEpochSeed, trace: true }); }, { initialSeed, bootEpochSeed: scenario.bootEpochSeed });
      assert.equal(state.screen, 'title'); const boot = state.bootLifecycle;
      await writeFile(join(directory, 'browser-title-state.json'), JSON.stringify(state, null, 2) + '\n');
      assert.deepEqual(boot.shared, { words: observed.mtState, index: observed.mtIndex }, `${scenario.name}: complete shared MT`);
      assert.equal(boot.savedPlayerSeed, observed.stateSeed9688, `${scenario.name}: saved player seed`);
      assert.equal(boot.menuRunSeed, observed.title.menuRunSeed, `${scenario.name}: menu run seed`);
      assert.deepEqual(boot.secretPoses, observed.title.statsSamples.map(sample => sample.pose), `${scenario.name}: authored stat poses`);
      assert.deepEqual(boot.savedGridSeedSamples, observed.gridSeedSamples.map(sample => ({
        roomRecord: sample.roomRecord, gridIndex: sample.gridIndex, seeds: [sample.seed10, sample.seed18],
      })), `${scenario.name}: bounded saved grid pairs`);
      // The future manually entered seed is independent of startup time.
      for (let i = 0; i < 3; i++) await page.keyboard.press('Enter');
      await page.keyboard.press('Tab'); await page.keyboard.type(scenario.code); await page.keyboard.press('Enter'); await page.keyboard.press('Enter');
      const seeded = await page.evaluate(() => window.game.snapshot());
      const trace = await page.evaluate(() => window.game.trace());
      await writeFile(join(directory, 'browser-input.json'), JSON.stringify({ schema: 1, seeded, trace,
        expectedCommands: manualSeedCommands(scenario.code) }, null, 2) + '\n');
      assertSeededRun(seeded, scenario.code, native.seedValue);
      assertInputProvenance(trace, manualSeedCommands(scenario.code), initialSeed, scenario.bootEpochSeed);
      assert.deepEqual(errors, []);
      result.status = 'pass'; result.comparedStateWords = 624; result.comparedGridPairs = 9; result.comparedStatPoses = 179;
      result.inputProvenance = { status: 'pass', commands: trace.commands.length, initialSeed: trace.initialSeed,
        bootEpochSeed: trace.bootEpochSeed, manualSeed: scenario.code };
    } catch (error) { result.status = 'fail'; result.error = String(error); }
    finally {
      if (page) await page.close().catch(error => { result.status = 'fail'; result.cleanupError = String(error); });
      await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
    }
  }
  report.gateSourcesEnd = await sourceHashes();
  report.gateSourcesStable = JSON.stringify(report.gateSources) === JSON.stringify(report.gateSourcesEnd);
  assert.equal(report.gateSourcesStable, true, 'Boot MT gate source files changed during capture');
  report.status = report.cases.every(c => c.status === 'pass') ? 'pass' : 'fail';
  if (report.status === 'fail') throw new AggregateError(report.cases.filter(c => c.status === 'fail').map(c => new Error(`${c.name}: ${c.error}`)), 'Selected boot lifecycle cases failed; every case was attempted');
} catch (error) { report.status = 'fail'; report.error = String(error); throw error; }
finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser.close(); await production.close();
  console.log(JSON.stringify({ status: report.status, cases: report.cases.length, report: join(output, 'report.json') }));
}
