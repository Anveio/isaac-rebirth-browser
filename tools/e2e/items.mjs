// Both live games own their independent pool state; source values never enter WASM.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, assertSeededRun, enterSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const PIN = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const files = ['tools/e2e/items.mjs', 'tools/original/item-scenarios.py', 'tools/original/item-observer.py',
  'tools/foundation/item-catalog.py', 'tools/foundation/decode-rooms.py', 'tools/original/foundation-seeds.py',
  'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c',
  'tools/original/foundation-clock-verify.py', 'tools/original/foundation-pickup-observe.py',
  'tools/original/state-observer.py', 'tools/original/with-display.sh', 'tools/foundation/verify.mjs',
  'tools/foundation/browser.mjs', 'tools/foundation/production-server.mjs', 'web/trace.ts',
  'engine/src/items.rs', 'engine/src/seed.rs', 'engine/src/pickups.rs', 'engine/src/lib.rs', 'engine/build.rs'];
const hashes = async () => Object.fromEntries(await Promise.all(files.map(async path => [path, digest(await readFile(join(ROOT, path)))])));
const before = await hashes();
const value = flag => { const i = process.argv.indexOf(flag); return i < 0 ? undefined : process.argv[i + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/items', String(Date.now())));
await mkdir(output, { recursive: true });
const scenarios = value('--scenarios') ? JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')) :
  JSON.parse((await run('python3', ['tools/original/item-scenarios.py', '--count', value('--count') || '2'], { cwd: ROOT, timeout: 600000 })).stdout);
assert.equal(scenarios.schema, 1);
assert.equal(scenarios.bootEpochSeed, 1700000000);
assert.ok(scenarios.cases.length >= 4 && scenarios.cases.length <= 6);
assert.equal(new Set(scenarios.cases.map(item => item.name)).size, scenarios.cases.length);
for (const item of scenarios.cases) {
  assert.match(item.name, /^[a-zA-Z0-9_-]{1,32}$/);
  assert.match(item.code, /^[A-Z0-9]{8}$/);
  assert.ok(Number.isInteger(item.seedValue) && item.seedValue > 0 && item.seedValue <= 0xffffffff);
  assert.ok([0, 1].includes(item.delay));
}
const selectedCase = value('--case');
if (selectedCase) assert.ok(scenarios.cases.some(item => item.name === selectedCase));
const catalogue = JSON.parse((await run('python3', ['tools/foundation/item-catalog.py'], { cwd: ROOT })).stdout);
assert.equal(digest(await readFile(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64'))), PIN);
await writeFile(join(output, 'scenarios.json'), JSON.stringify(scenarios, null, 2) + '\n');
const server = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = { schema: 1, status: 'running', production: server.build, originalBinarySha256: PIN,
  gateSources: before, graphics: await inspectBrowserGraphics(browser), sourceHashes: catalogue.sourceHashes,
  scope: 'Fresh-profile authored catalogue, causal run/pool seeds, complete 16 pools/497 weights and ID maps, natural free treasure pedestal Init/selection, phase/animation/charge and public seed/input provenance. Acquisition, item effects/cache evaluation, familiars, paid/override contexts and pixels remain unaccepted.',
  selection: selectedCase ? { scope: 'Targeted recorded public case; default gate runs the complete cohort', case: selectedCase } : { scope: 'Complete cohort' }, cases: [] };
const f32 = value => Math.fround(value);
const words = rng => [rng.state, rng.rightA, rng.left, rng.rightB];
const poolRows = pools => pools.map(pool => ({ ...pool, rng: undefined,
  rngWords: pool.rng.flatMap(words), baseWeightTotal: f32(pool.baseWeightTotal),
  entries: pool.entries.map(entry => ({ ...entry, baseWeight: f32(entry.baseWeight),
    currentWeight: f32(entry.currentWeight), decreaseBy: f32(entry.decreaseBy), removeOn: f32(entry.removeOn) })) }));
try {
  for (const item of scenarios.cases) {
    if (selectedCase && item.name !== selectedCase) continue;
    const directory = join(output, item.name);
    await mkdir(directory, { recursive: true });
    const result = { ...item, status: 'running', failures: [] };
    report.cases.push(result);
    let page;
    try {
      await writeFile(join(directory, 'case.json'), JSON.stringify(item, null, 2) + '\n');
      const tape = (await run('python3', ['tools/original/item-scenarios.py', '--case', join(directory, 'case.json')], { cwd: ROOT })).stdout;
      await writeFile(join(directory, 'tape.json'), tape);
      await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
        '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
        '--state-observer', 'tools/original/item-observer.py', '--sfx-volume', '1', '--music-volume', '1',
        '--boot-epoch-seed', String(scenarios.bootEpochSeed), '--frame-timeout', '120',
        '--output', join(directory, 'original')], { cwd: ROOT, timeout: 600000, maxBuffer: 2 * 1024 * 1024 });
      const nativeBytes = await readFile(join(directory, 'original/manifest.json'));
      const native = JSON.parse(nativeBytes);
      assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      const provenance = JSON.parse((await run('python3', ['tools/original/item-scenarios.py', '--validate-original', join(directory, 'original/manifest.json')], { cwd: ROOT })).stdout);
      assert.equal(provenance.status, 'pass'); assert.deepEqual(provenance.failures, []);
      const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
      assert.equal(baseline?.observation.status, 'pass');
      assert.equal(baseline.context.publicLogState.seedValue, item.seedValue);
      assert.deepEqual(baseline.observation.configs, catalogue.collectibles.map(item => ({
        id: item.id, kind: item.kind, achievement: item.achievement, cacheFlags: item.cacheFlags,
        inventory: item.inventory, maxCharges: item.maxCharges, cooldown: item.cooldown, special: item.special,
        eligible: item.achievement < 0 || Boolean(baseline.observation.achievements[item.achievement]),
      })), 'All 343 native configurations must match the independently pinned authored catalogue');
      const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row.observation]));
      page = await browser.newPage();
      const errors = [], states = [];
      page.on('pageerror', error => errors.push(String(error)));
      await page.goto(server.url + '/?manual=1'); await waitForReady(page);
      await page.evaluate(({ seedValue, bootEpochSeed }) => {
        window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed });
      }, { seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed });
      const start = await enterSeededRun(page, item.code);
      assertSeededRun(start, item.code, item.seedValue);
      const expected = [...manualSeedCommands(item.code)];
      let checks = 0, presentations = 0, passedPresentations = 0, admitted = 0;
      const ageClockDiagnostics = [];
      const equal = (actual, expected, field, presentation) => {
        checks++;
        try { assert.deepEqual(actual, expected, `${item.name}:${presentation}:${field}`); }
        catch (error) { result.failures.push({ field, presentation, actual, expected, error: String(error.message) }); }
      };
      const compare = (state, source, presentation) => {
        assert.equal(source.status, 'pass'); assert.equal(source.memoryWritten, false);
        assert.equal(source.executableMappingsRead, false); assert.equal(source.nativeBinarySha256, PIN);
        assert.equal(source.gamePresent, true);
        const prior = result.failures.length;
        equal(state.systemClock.systemCounter % 2, source.systemCounter % 2, 'source phase', presentation);
        equal(state.profiles[state.profile - 1].achievements, source.achievements.map(Boolean), 'fresh eligibility bytes', presentation);
        equal(state.runSeeds, {startSeed: source.runSeeds.startSeed, streamState: source.runSeeds.rngWords[0],
          initializedFields: source.runSeeds.initializedFields}, 'causal run seed owner', presentation);
        const pools = state.itemPools;
        equal(pools.seed, source.poolSeed, 'causal pool initializer', presentation);
        equal(words(pools.rootRng), source.rootRngWords, 'complete root RNG', presentation);
        equal(words(pools.secondaryRng), source.secondaryRngWords, 'complete secondary RNG', presentation);
        equal(pools.pillEffects, source.pillEffects, 'initializer pill mapping', presentation);
        equal(pools.roomRoutes, source.roomRoutes, 'all room routes', presentation);
        equal(poolRows(pools.pools).map(({rng, ...pool}) => pool), source.pools, 'all ordered pool metadata/weights/RNG/eligibility', presentation);
        equal(f32(pools.specialFactor), source.specialFactor, 'global special probability', presentation);
        equal(pools.removed, source.removed.map(Boolean), 'all removed IDs', presentation);
        equal(pools.selected, source.selected.map(Boolean), 'all selected IDs', presentation);
        const originals = source.world.entities.filter(entity => entity.type === 5 && entity.variant === 100);
        const actuals = state.room.pickups.filter(pickup => pickup.variant === 100);
        equal(actuals.length, originals.length, 'natural pedestal admission', presentation);
        admitted = Math.max(admitted, originals.length);
        for (let i = 0; i < originals.length; i++) {
          const actual = actuals[i], original = originals[i]; if (!actual) continue;
          for (const field of ['subtype', 'spawnSeed', 'animation', 'animationFrame', 'animationPlaying', 'collisionRadius']) {
            equal(actual[field], original[field], `pedestal${i}:${field}`, presentation);
          }
          equal([actual.position.x, actual.position.y], original.position, `pedestal${i}:world position`, presentation);
          equal(actual.charge, source.pedestals[i].charge, `pedestal${i}:authored charge`, presentation);
          equal(actual.touched, source.pedestals[i].touched, `pedestal${i}:touched`, presentation);
          equal(actual.age, source.pedestals[i].age, `pedestal${i}:source birth age`, presentation);
          equal(actual.age, state.gameUpdateCount - actual.birthGameUpdate, `pedestal${i}:own Game-clock law`, presentation);
          equal(source.pedestals[i].age, source.pedestals[i].currentGameUpdate - source.pedestals[i].bornGameUpdate,
            `pedestal${i}:native Game-clock law`, presentation);
          if (source.pedestals[i].age === 0 || presentation === item.actions.reduce((sum, action) => sum + action.frames, 0)) {
            ageClockDiagnostics.push({presentation, pedestal: i,
              native: {born: source.pedestals[i].bornGameUpdate, current: source.pedestals[i].currentGameUpdate, age: source.pedestals[i].age},
              browser: {born: actual.birthGameUpdate, current: state.gameUpdateCount, age: actual.age},
              scope: 'Real internal Game age counters; absolute run-admission/cutscene counter difference remains diagnostic, never supplied to WASM'});
          }
          equal(actual.collected, false, `pedestal${i}:selection boundary precedes acquisition`, presentation);
        }
        if (result.failures.length === prior) passedPresentations++;
      };
      compare(start, baseline.observation, 0);
      for (const {key, frames} of item.actions) {
        const code = key ? 'Key' + key.toUpperCase() : null;
        if (key) { await page.keyboard.down(key); expected.push({type: 'key', code, down: true}); }
        for (let index = 0; index < frames; index++) {
          await page.evaluate(() => window.game.step(1));
          expected.push({type: 'tick', count: 1});
          const state = await page.evaluate(() => window.game.snapshot()); states.push(state);
          presentations++;
          compare(state, rows.get(baseline.context.presentedFrame + presentations), presentations);
        }
        if (key) { await page.keyboard.up(key); expected.push({type: 'key', code, down: false}); }
      }
      assert.ok(admitted > 0, 'Every case must naturally admit a source-selected pedestal');
      const trace = await page.evaluate(() => window.game.trace());
      assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed);
      assertSeededRun(await page.evaluate(() => window.game.snapshot()), item.code, item.seedValue);
      assert.deepEqual(errors, []);
      const browserBytes = Buffer.from(JSON.stringify({schema: 1, states, trace}) + '\n');
      const browserPath = join(directory, 'browser-states.json.gz');
      await writeFile(browserPath, gzipSync(browserBytes));
      Object.assign(result, {status: result.failures.length ? 'fail' : 'pass', checks, comparedPresentations: presentations + 1,
        passedPresentations, admittedPedestals: admitted, selectedItemIds: states.at(-1).room.pickups.filter(p => p.variant === 100).map(p => p.subtype),
        ageClockDiagnostics,
        browserEvidence: {path: browserPath, encoding: 'gzip', rawSha256: digest(browserBytes), rawBytes: browserBytes.length},
        nativeManifestSha256: digest(nativeBytes), tapeSha256: digest(Buffer.from(tape)),
        inputProvenance: {status: 'pass', commands: trace.commands.length, stateHistoryTruncated: trace.stateHistoryTruncated,
          inputTraceTruncated: trace.inputTraceTruncated, seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed}});
    } catch (error) { result.status = 'fail'; result.error = String(error); }
    finally { if (page) await page.close(); }
    await writeFile(join(directory, 'report.json'), JSON.stringify(result, null, 2) + '\n');
    console.log(JSON.stringify({case: item.name, status: result.status, checks: result.checks, failures: result.failures.length, error: result.error}));
  }
  const after = await hashes();
  report.sourceStability = {status: JSON.stringify(before) === JSON.stringify(after) ? 'pass' : 'fail', before, after};
  report.status = report.cases.every(result => result.status === 'pass') && report.sourceStability.status === 'pass' ? 'pass' : 'fail';
} finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser.close(); await server.close();
  console.log(JSON.stringify({status: report.status, report: join(output, 'report.json'), cases: report.cases.length}));
  process.exitCode = report.status === 'pass' ? 0 : 1;
}
