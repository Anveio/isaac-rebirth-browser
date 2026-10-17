// Both live games own their independent pool state; source values never enter WASM.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile, rm, stat } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { gzipSync, gunzipSync } from 'node:zlib';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, assertSeededRun, enterSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';
import { compareRoomContext } from './item-room-evidence.mjs';
import { flag, selectCases } from './lib/cli.mjs';
import { snapshot, stepSnapshot } from './lib/game.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const value = flag => { const i = process.argv.indexOf(flag); return i < 0 ? undefined : process.argv[i + 1]; };
const activeUse = process.argv.includes('--active-use');
const statCache = process.argv.includes('--stat-cache');
const damageFireDelay = process.argv.includes('--damage-fire-delay');
assert.ok(!statCache || !damageFireDelay, 'Choose one complete cache cohort');
const statCoverage = statCache || damageFireDelay;
assert.ok(!(activeUse && statCoverage), 'Active use and cache scenarios require separate complete cohorts');
const acquisition = activeUse || statCoverage || process.argv.includes('--acquisition');
const replayOriginal = value('--replay-original') ? resolve(value('--replay-original')) : null;
assert.ok(!replayOriginal || acquisition, 'Frozen original recovery is a separately disclosed acquisition comparison');
const output = resolve(value('--output') || join(ROOT, activeUse ? '.cache/e2e/active-item-use' : damageFireDelay ? '.cache/e2e/damage-fire-delay' : statCache ? '.cache/e2e/stat-cache' : acquisition ? '.cache/e2e/item-acquisition' : '.cache/e2e/items', String(Date.now())));
await mkdir(output, { recursive: true });
const frozenReportBytes = replayOriginal ? await readFile(join(replayOriginal, 'report.json')) : null;
const frozenReport = frozenReportBytes ? JSON.parse(frozenReportBytes) : null;
const scenarios = replayOriginal ? JSON.parse(await readFile(join(replayOriginal, 'scenarios.json'), 'utf8')) :
  value('--scenarios') ? JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')) :
  JSON.parse((await run('python3', [activeUse ? 'tools/original/active-item-scenarios.py' : 'tools/original/item-scenarios.py', '--count', value('--count') || '2',
    ...(damageFireDelay ? ['--damage-fire-delay'] : statCache ? ['--stat-cache'] : acquisition && !activeUse ? ['--acquisition'] : [])], { cwd: ROOT, timeout: 600000 })).stdout);
assert.equal(scenarios.schema, 1);
assert.equal(scenarios.bootEpochSeed, 1700000000);
// A bank recorded under --case holds only the cases that ran; its whole bank
// met the count and cohort composition when it was recorded, and every case it
// holds still meets its per-case requirements.
const whole = !scenarios.caseFilter;
if (whole) assert.ok(scenarios.cases.length >= 4 && scenarios.cases.length <= (activeUse ? 12 : acquisition ? 8 : 6));
assert.equal(new Set(scenarios.cases.map(item => item.name)).size, scenarios.cases.length);
if (activeUse && !replayOriginal) {
  if (whole) assert.equal(scenarios.cases.length, 12, 'Active use requires the complete twelve-case fresh cohort');
  const fixed = JSON.parse((await run('python3', ['tools/original/active-item-scenarios.py', '--fixed-only'],
    { cwd: ROOT })).stdout);
  assert.equal(fixed.cases.length, 10);
  for (const required of fixed.cases) {
    const present = scenarios.cases.find(item => item.name === required.name);
    if (whole || present) assert.deepEqual(present, required,
      `Active use requires the unchanged public seed/phase/actions and witnesses for ${required.name}`);
  }
  const fresh = scenarios.cases.filter(item => item.entropy === 'OS secrets');
  if (whole) assert.deepEqual(fresh.map(item => [item.name, item.delay]).sort(),
    [['random-pedestal-0', 0], ['random-pedestal-1', 1]], 'Both fresh OS admission phases are required');
  for (const item of fresh) {
    const required = fixed.cases.find(fixed => fixed.name === `recorded-active-use-${item.delay}`);
    assert.deepEqual(item.actions, required.actions, 'Every new unfiltered OS outcome requires the complete acquisition/use tape');
    assert.equal(item.expectedItemId, undefined, 'Fresh OS outcomes cannot request a filtered item identity');
    assert.equal(item.requirePickupShotGuard, undefined, 'The held-Pickup witnesses belong to their fixed public cases');
  }
  if (whole) {
    assert.deepEqual(scenarios.cases.filter(item => item.requirePickupShotGuard).map(item => item.delay).sort(),
      [0, 1], 'Both held-Pickup admission phases are required');
    assert.deepEqual(scenarios.cases.filter(item => item.entropy === 'Recorded OS secrets').map(item => item.code).sort(),
      ['98C1GNPZ', 'DN8RQDXC', 'R74QYMYS', 'VRQPT844'], 'Every earlier unfiltered OS outcome is required');
    assert.equal(scenarios.cases.filter(item => item.entropy === 'OS secrets').length, 2,
      'Two new unfiltered OS outcomes are required');
  }
}

if (statCoverage) {
  if (whole) assert.equal(scenarios.cases.length, 8, 'Stat cache requires the complete eight-case cohort');
  const fixed = JSON.parse((await run('python3', ['tools/original/item-scenarios.py', '--fixed-only',
    damageFireDelay ? '--damage-fire-delay' : '--stat-cache'], { cwd: ROOT })).stdout);
  assert.equal(fixed.cases.length, 6);
  for (const required of fixed.cases) {
    const present = scenarios.cases.find(item => item.name === required.name);
    if (whole || present) assert.deepEqual(present, required,
      `Cache coverage requires the unchanged seed/phase/public actions for ${required.name}`);
  }
  const fresh = scenarios.cases.filter(item => !fixed.cases.some(required => required.name === item.name));
  if (whole) assert.deepEqual(fresh.map(item => [item.name, item.delay]).sort(),
    [['random-pedestal-0', 0], ['random-pedestal-1', 1]], 'Both new OS admission phases are required');
  for (const item of fresh) {
    assert.equal(item.entropy, 'OS secrets');
    assert.equal(item.expectedItemId, undefined, 'Fresh OS outcomes cannot request a filtered item identity');
    assert.ok(Number.isInteger(item.discoveryAttempts) && item.discoveryAttempts >= 1 && item.discoveryAttempts <= 10000);
    assert.deepEqual(item.actions, fixed.cases.find(required => required.name === `recorded-pedestal-${item.delay}`).actions,
      'Every new unfiltered OS outcome requires the complete acquisition/motion/shooting tape');
  }
  if (whole) {
    assert.deepEqual(scenarios.cases.filter(item => item.expectedItemId !== undefined)
      .map(item => [item.expectedItemId, item.delay]).sort(), damageFireDelay ? [[1, 0], [1, 1], [4, 0], [4, 1]] : [[14, 0], [14, 1], [143, 0], [143, 1]],
      'Both independent admission phases of each mapped treasure passive are required');
    assert.deepEqual(scenarios.cases.filter(item => item.entropy === 'Recorded OS secrets').map(item => item.code).sort(),
      ['R74QYMYS', 'VRQPT844'], 'The retained unfiltered active/passive outcomes cannot be replaced');
    assert.equal(scenarios.cases.filter(item => item.entropy === 'OS secrets').length, 2,
      'Two freshly recorded, item-unfiltered OS discoveries are required');
  }
}
for (const item of scenarios.cases) {
  assert.match(item.name, /^[a-zA-Z0-9_-]{1,32}$/);
  assert.match(item.code, /^[A-Z0-9]{8}$/);
  assert.ok(Number.isInteger(item.seedValue) && item.seedValue > 0 && item.seedValue <= 0xffffffff);
  assert.ok([0, 1].includes(item.delay));
  if (statCache) assert.equal(scenarios.mode, 'stat-cache', 'Stat coverage requires its complete declared post-grant route');
  if (damageFireDelay) assert.equal(scenarios.mode, 'damage-fire-delay', 'Damage/delay requires its complete declared post-grant route');
  if (acquisition && !activeUse && !replayOriginal && !['acquisition', 'stat-cache', 'damage-fire-delay'].includes(scenarios.mode)) {
    item.actions.push({key: 'w', frames: 32});
    if (item.entropy.startsWith('Contrived')) item.actions.push({key: 'Right', frames: 50}, {key: null, frames: 50});
    else item.actions.push({key: null, frames: 100});
  }
}
const selectedCase = flag('--case');
scenarios.cases = selectCases(scenarios.cases, item => item.name);
if (selectedCase) scenarios.caseFilter = selectedCase;
const catalogue = JSON.parse((await run('python3', ['tools/foundation/item-catalog.py'], { cwd: ROOT })).stdout);
await assertOriginalBinary();
await writeFile(join(output, 'scenarios.json'), JSON.stringify(scenarios, null, 2) + '\n');
const server = await startProductionServer({ directory: value('--production-directory') });
const browser = await chromium.launch(browserLaunchOptions());
const report = { schema: 1, status: 'running', production: server.build, originalBinarySha256: ORIGINAL_SHA,
  graphics: await inspectBrowserGraphics(browser), sourceHashes: catalogue.sourceHashes,
  scope: activeUse ? 'Natural independent original/browser acquisition and public active use; authored charge/count/temporary records/health/7d4/7d8/UseItem/motion/SFX admission are required. Full raw fields, inputs, backend histories and all random failures remain retained; pixels and complete shared MT stay diagnostics. Other active effects and charge modifiers remain explicitly unimplemented.' : damageFireDelay ? 'Natural public Sad Onion1 and Cricket Head4 acquisition at both phases, retained291/48 and two fresh OS outcomes conditioned only on a port-predicted public treasure route; outcomes are retained without native item filtering. Cached stats, firing counter/cooldown/head timer, Player motion, actual Tear birth ordinal/owner, current manager membership/death flag, birth damage/base damage/scale/radius/clip and stored velocity are blocking, as are every existing pool/count/grant/queue and input provenance comparison. Unsupported item/weapon effects and every mismatch remain retained; no source values enter WASM.' : statCache ? 'Natural public acquisition of Roid Rage14 and Speed Ball143 at both phase offsets, plus retained and fresh unfiltered OS outcomes. Spoon27/Belt28 are source-mapped but boss-only and have no live acquisition acceptance here. All actual cached stats/fire delay, player XY/VXY before and after grant, emitted tear velocity, grant/count/pool/queue fields and complete public seed/DOM provenance are blocking. Other item effects remain unsupported with every comparison failure retained; complete MT, unfamiliar callbacks and later child RNG states are explicit diagnostics.' : acquisition ? 'Natural first free collectible contact, real deferred Pickup animation/queue, all347 owned counts, authored inventory/health grants, first active ownership/charge, pool removal and collection sound admission. Baseline complete450 Player child RNGs are required. Actual stat/familiar/cache callbacks, later child RNG mutation, active swaps/use, achievements, paid/override contexts and pixels remain unaccepted; full MT/cache/child RNG differences are retained diagnostics.' : 'Fresh-profile authored catalogue, causal run/pool seeds, complete 16 pools/497 weights and ID maps, natural free treasure pedestal Init/selection, phase/animation/charge and public seed/input provenance. Acquisition, item effects/cache evaluation, familiars, paid/override contexts and pixels remain unaccepted.',
  activeUse: activeUse ? {scope: 'Public first acquisition and action9 use; native record/charge/health/ANM/input/motion and157 admission required. Other active leaves remain red when exercised, with every OS discovery retained.'} : null,
  caseFilter: scenarios.caseFilter, selection: scenarios.caseFilter ? { scope: 'Targeted recorded public cases; default gate runs the complete cohort', case: scenarios.caseFilter } : { scope: 'Complete cohort' }, cases: [] };
report.originalCapture = replayOriginal ? {mode: 'frozen-current-turn-original-recovery',
  reportPath: join(replayOriginal, 'report.json'), reportSha256: digest(frozenReportBytes),
  recordedProduction: frozenReport.production,
  scope: `New live production browser replay against all ${frozenReport.cases.length} exact complete original recordings from the earlier frozen capture; this is not a fresh native recapture`} :
  {mode: 'fresh-live-original-and-browser', scope: 'Default required CI runs both real games freshly'};
const f32 = value => Math.fround(value);
const words = rng => [rng.state, rng.rightA, rng.left, rng.rightB];
const poolRows = pools => pools.map(pool => ({ ...pool, rng: undefined,
  rngWords: pool.rng.flatMap(words), baseWeightTotal: f32(pool.baseWeightTotal),
  entries: pool.entries.map(entry => ({ ...entry, baseWeight: f32(entry.baseWeight),
    currentWeight: f32(entry.currentWeight), decreaseBy: f32(entry.decreaseBy), removeOn: f32(entry.removeOn) })) }));
const archiveCompletedNative = async directory => {
  const path = join(directory, 'original/manifest.json');
  const bytes = await readFile(path).catch(error => { if (error.code === 'ENOENT') return null; throw error; });
  if (!bytes) return null;
  // Capture/validation has finished. Keep exact failed or passing raw bytes,
  // independently verify the lossless archive, then release its inactive JSON.
  JSON.parse(bytes);
  const metadata = await stat(path), archive = path + '.gz';
  await writeFile(archive, gzipSync(bytes));
  const restored = gunzipSync(await readFile(archive));
  assert.equal(restored.length, bytes.length); assert.equal(digest(restored), digest(bytes));
  const evidence = {path: archive, encoding: 'gzip', rawSha256: digest(bytes), rawBytes: bytes.length,
    originalPath: path, mode: metadata.mode & 0o777, mtimeMs: metadata.mtimeMs,
    scope: 'Losslessly archived completed native capture; status/failures and every source observation preserved'};
  await writeFile(join(directory, 'native-manifest-archive.json'), JSON.stringify(evidence, null, 2) + '\n');
  await rm(path);
  return evidence;
};
try {
  for (const item of scenarios.cases) {
    const directory = join(output, item.name);
    await mkdir(directory, { recursive: true });
    const result = { ...item, status: 'running', failures: [] };
    report.cases.push(result);
    let page;
    const states = [], checkWitnesses = [];
    const archiveBrowser = async (trace, captureStatus, traceError) => {
      const browserBytes = Buffer.from(JSON.stringify({schema: 1, states, trace, captureStatus, traceError, checkWitnesses}) + '\n');
      const browserPath = join(directory, 'browser-states.json.gz');
      await writeFile(browserPath, gzipSync(browserBytes));
      result.browserEvidence = {path: browserPath, encoding: 'gzip', rawSha256: digest(browserBytes),
        rawBytes: browserBytes.length, captureStatus, traceError};
    };
    try {
      await writeFile(join(directory, 'case.json'), JSON.stringify(item, null, 2) + '\n');
      const tape = (await run('python3', ['tools/original/item-scenarios.py', '--case', join(directory, 'case.json')], { cwd: ROOT })).stdout;
      await writeFile(join(directory, 'tape.json'), tape);
      let nativeBytes, nativePath;
      if (replayOriginal) {
        const recorded = frozenReport.cases.find(recorded => recorded.name === item.name);
        assert.ok(recorded); assert.equal(recorded.code, item.code); assert.equal(recorded.seedValue, item.seedValue);
        assert.deepEqual(recorded.actions, item.actions, 'Frozen recovery must replay the exact originally requested action plan');
        assert.equal(digest(Buffer.from(tape)), recorded.tapeSha256, 'The complete original public tape must be unchanged');
        const evidence = recorded.nativeEvidence;
        assert.equal(evidence.encoding, 'gzip');
        nativeBytes = gunzipSync(await readFile(evidence.path));
        assert.equal(nativeBytes.length, evidence.rawBytes); assert.equal(digest(nativeBytes), evidence.rawSha256);
        assert.equal(digest(nativeBytes), recorded.nativeManifestSha256);
        result.originalRecording = evidence;
        nativePath = evidence.path;
      } else {
        await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
          '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
          '--state-observer', 'tools/original/item-observer.py', '--sfx-volume', '1', '--music-volume', '1',
          '--boot-epoch-seed', String(scenarios.bootEpochSeed), '--frame-timeout', '120',
          '--output', join(directory, 'original')], { cwd: ROOT, timeout: 600000, maxBuffer: 2 * 1024 * 1024 });
        nativePath = join(directory, 'original/manifest.json'); nativeBytes = await readFile(nativePath);
      }
      const native = JSON.parse(nativeBytes);
      assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      const provenance = JSON.parse((await run('python3', ['tools/original/item-scenarios.py', '--validate-original', nativePath], { cwd: ROOT })).stdout);
      assert.equal(provenance.status, 'pass'); assert.deepEqual(provenance.failures, []);
      const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
      assert.equal(baseline?.observation.status, 'pass');
      assert.equal(baseline.context.publicLogState.seedValue, item.seedValue);
      assert.deepEqual(baseline.observation.configs, catalogue.collectibles.map(item => ({
        id: item.id, kind: item.kind, achievement: item.achievement, cacheFlags: item.cacheFlags,
        inventory: item.inventory, maxCharges: item.maxCharges, cooldown: item.cooldown, special: item.special,
        eligible: item.achievement < 0 || Boolean(baseline.observation.achievements[item.achievement]),
      })), 'All 343 native configurations must match the independently pinned authored catalogue');
      const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row]));
      page = await browser.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(String(error)));
      await page.goto(server.url + '/?manual=1'); await waitForReady(page);
      await page.evaluate(({ seedValue, bootEpochSeed }) => {
        window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed, audioPreferences: {sfxVolume: 1, musicVolume: 1}, fullscreenPreferences: {fullscreen: false} });
      }, { seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed });
      const start = await enterSeededRun(page, item.code);
      assertSeededRun(start, item.code, item.seedValue);
      const expected = [...manualSeedCommands(item.code)];
      let checks = 0, presentations = 0, passedPresentations = 0, admitted = 0, queuedPresentations = 0, grantedPresentations = 0;
      const selectedItemIds = new Set(), callbackDiagnostics = [], mtDiagnostics = [];
      const grantedItemIds = new Set(); let activeGrantObserved = false, healthGrantObserved = false;
      const pickupShotGuardPhases = new Set();
      let callbackDiagnosticSignature = null, mtDiagnosticSignature = null;
      const ageClockDiagnostics = [];
      const equal = (actual, expected, field, presentation) => {
        checks++;
        let passed = true;
        try { assert.deepEqual(actual, expected, `${item.name}:${presentation}:${field}`); }
        catch (error) { passed = false; result.failures.push({ field, presentation, actual, expected, error: String(error.message) }); }
        checkWitnesses.push([field, presentation, passed]);
      };
      const compare = (state, row, presentation) => {
        const source = row.observation;
        assert.equal(source.status, 'pass'); assert.equal(source.memoryWritten, false);
        assert.equal(source.executableMappingsRead, false); assert.equal(source.nativeBinarySha256, ORIGINAL_SHA);
        assert.equal(source.gamePresent, true);
        const prior = result.failures.length;
        for (const check of compareRoomContext(state, row)) equal(check.actual, check.expected, check.field, presentation);
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
        const owner = state.playerItems, nativePlayer = source.players[0];
        equal(source.players.length, 1, 'single normal Player owner', presentation);
        equal(owner.initializerSeed, source.runSeeds.initializedFields[12], 'actual Player initializer handoff', presentation);
        equal(owner.entitySpawnSeed, nativePlayer.entitySpawnSeed, 'causal Player entity seed', presentation);
        equal(words(owner.managerRng), nativePlayer.managerRngWords, 'Player manager RNG owner', presentation);
        if (nativePlayer.ownedCounts.every(count => count === 0)) {
          equal(owner.itemRngs.map(words), nativePlayer.itemRngWords, 'all347 initialized item RNG children', presentation);
          equal(owner.trinketRngs.map(words), nativePlayer.trinketRngWords, 'all62 initialized trinket RNG children', presentation);
          equal(owner.pocketRngs.map(words), nativePlayer.pocketRngWords, 'all41 initialized pocket RNG children', presentation);
        }
        if (acquisition) {
          const nativeActor = source.world.players.find(entity => entity.type === 1 && entity.variant === 0);
          assert.ok(nativeActor, 'Independent native normal Isaac actor must be present');
          const playerWorld = [f32((state.player.x - 32) / 0.65), f32((state.player.y + 46) / 0.65)];
          const playerVelocity = [f32(state.player.vx / 0.65), f32(state.player.vy / 0.65)];
          if (statCoverage || nativePlayer.ownedCounts.every(count => count === 0)) {
            equal(playerWorld, nativeActor.position, 'actual Player world position', presentation);
            equal(playerVelocity, nativeActor.velocity, 'actual Player world velocity', presentation);
          }
          if (statCoverage) {
            assert.ok(nativePlayer.cachedStats, 'Fresh native cached-stat readback is required; old acquisition recordings cannot certify it');
            equal(state.maxFireDelay, nativePlayer.cachedStats.maxFireDelay, 'cached fire delay', presentation);
            for (const field of ['shotSpeed', 'damage', 'tearHeight', 'tearFallingSpeed', 'tearFallingAcceleration', 'moveSpeed'])
              equal(f32(state.playerStats[field]), nativePlayer.cachedStats[field], `cached ${field}`, presentation);
            if (statCache) {
              // Retain this comparison; native +31f death and port pool membership
              // are distinct owners pending typed observation.
              const actualTears = state.tears.filter(tear => tear.motion.lifetime !== 'removed');
              const nativeTears = source.world.entities.filter(entity => entity.type === 2 && !entity.removed);
              equal(actualTears.length, nativeTears.length, 'actual regular tear cardinality', presentation);
              for (let index = 0; index < Math.min(actualTears.length, nativeTears.length); index++)
                equal([f32(actualTears[index].motion.velocity.x), f32(actualTears[index].motion.velocity.y)],
                  nativeTears[index].velocity, `tear${index}:actual stored velocity`, presentation);
            }
            if (damageFireDelay) {
              assert.ok(Number.isInteger(nativePlayer.shotsFired), 'Fresh native Player+c84 readback is required');
              equal(state.player.shotsFired, nativePlayer.shotsFired, 'actual Player firing ordinal counter', presentation);
              equal(state.fireCooldown, nativePlayer.fireCooldown, 'actual firing cooldown', presentation);
              equal(state.headTimer, nativePlayer.headTimer, 'actual shooting head timer', presentation);
              const actualTears = state.tears.filter(tear => tear.motion.lifetime !== 'removed');
              const nativeTears = source.world.entities.filter(entity => entity.type === 2);
              equal(actualTears.length, nativeTears.length, 'actual current-manager regular tear cardinality', presentation);
              const actualOrdinals = actualTears.map(tear => tear.shotOrdinal);
              const nativeOrdinals = nativeTears.map(tear => tear.shotOrdinal);
              equal(new Set(actualOrdinals).size, actualOrdinals.length, 'unique browser regular firing ordinals', presentation);
              equal(new Set(nativeOrdinals).size, nativeOrdinals.length, 'unique native regular firing ordinals', presentation);
              equal(actualOrdinals.sort((a, b) => a - b), nativeOrdinals.sort((a, b) => a - b), 'actual regular birth ordinal set', presentation);
              const actualByOrdinal = new Map(actualTears.map(tear => [tear.shotOrdinal, tear]));
              for (const original of nativeTears) {
                equal(original.managerMember, true, `tear${original.shotOrdinal}:actual native manager pointer`, presentation);
                equal(original.tearOwner, {type: 1, variant: 0, sequenceId: nativeActor.sequenceId},
                  `tear${original.shotOrdinal}:actual linked Player owner`, presentation);
                const actual = actualByOrdinal.get(original.shotOrdinal);
                if (!actual) continue; // The required ordinal-set comparison retains each missing actor.
                const label = `tear${original.shotOrdinal}`;
                equal(actual.motion.lifetime === 'dying', original.dead, `${label}:actual death flag`, presentation);
                equal(f32(actual.collisionDamage), original.collisionDamage, `${label}:actual collision damage`, presentation);
                equal(f32(actual.baseDamage), original.baseDamage, `${label}:actual base damage`, presentation);
                equal(f32(actual.scale), original.tearScale, `${label}:actual stored scale`, presentation);
                equal(f32(actual.collisionRadius), original.collisionRadius, `${label}:actual collision radius`, presentation);
                equal(`RegularTear${actual.animationTier}`, original.animation, `${label}:authored regular clip`, presentation);
                equal([f32(actual.motion.velocity.x), f32(actual.motion.velocity.y)], original.velocity,
                  `${label}:actual stored velocity by birth ordinal`, presentation);
              }
            }
          }
          equal(owner.ownedCounts, nativePlayer.ownedCounts, 'all347 actual owned counts', presentation);
          if (statCoverage && item.expectedItemId !== undefined && nativePlayer.ownedCounts[item.expectedItemId] > 0) {
            equal(owner.unevaluatedCacheFlags, 0, 'mapped passive cache completion', presentation);
            equal(nativePlayer.cacheFlags, 0, 'actual completed source cache word', presentation);
          }
          equal(owner.queuedId, nativePlayer.queuedId, 'deferred config identity', presentation);
          equal(owner.queuedCharge, nativePlayer.queuedCharge, 'deferred requested charge/reset', presentation);
          if (nativePlayer.queuedId !== null) {
            queuedPresentations++;
            equal(owner.queuedTouched, Boolean(nativePlayer.queuedTouched), 'active queue touched argument', presentation);
            equal(state.tears.filter(tear => tear.motion.lifetime === 'alive').length,
              source.world.entities.filter(entity => entity.type === 2 && !entity.removed).length,
              'actual normal tear emission while Pickup is queued', presentation);
          }
          if (nativePlayer.ownedCounts.some(count => count > 0)) {
            grantedPresentations++;
            nativePlayer.ownedCounts.forEach((count, id) => { if (count > 0) grantedItemIds.add(id); });
            activeGrantObserved ||= nativePlayer.activeId !== 0;
            healthGrantObserved ||= JSON.stringify(nativePlayer.health) !== JSON.stringify(baseline.observation.players[0].health);
          }
          equal([state.player.maxHp, state.player.hp, state.player.soulHp, state.player.blackHeartMask], nativePlayer.health,
            'authored normal Isaac health helpers', presentation);
          equal(state.inventory, nativePlayer.inventory, 'authored inventory helper grants', presentation);
          equal(owner.pickupAnimationGuard || (activeUse && state.playerAnimation.guard), nativePlayer.pickupAnimationGuard, 'actual animation guard', presentation);
          equal(owner.collectionDelay, nativePlayer.pickupCooldown, 'Player collection delay', presentation);
          for (const field of ['activeId', 'activeCharge', 'activeSubordinateCharge', 'activeStateD80', 'activeStateD84', 'reserveCharge']) {
            equal(owner[field], nativePlayer[field], `Player:${field}`, presentation);
          }
          if (nativePlayer.activeId !== 0) equal(owner.activeFlag3076, nativePlayer.activeFlag3076, 'active replacement flag', presentation);
          if (nativePlayer.pickupAnimationGuard && (!activeUse || nativePlayer.mainSprite.animation === 'Pickup')) {
            equal(nativePlayer.mainSprite.animation, 'Pickup', 'source authored Pickup clip', presentation);
            equal(owner.pickupAnimationFrame, nativePlayer.mainSprite.animationFrame, 'authored Pickup frame', presentation);
            equal(owner.pickupAnimationPlaying, nativePlayer.mainSprite.animationPlaying, 'authored Pickup playing', presentation);
          }
          if (activeUse) {
            equal(state.player.controlsEnabled, nativePlayer.controlsEnabled, 'persistent7d4 controls', presentation);
            equal(state.player.inputCooldown, nativePlayer.inputCooldown, 'input7d8 cooldown', presentation);
            equal(state.fireCooldown, nativePlayer.fireCooldown, 'actual guarded-shot fire timer', presentation);
            equal(state.headTimer, nativePlayer.headTimer, 'actual guarded-shot head timer', presentation);
            if (item.requirePickupShotGuard) {
              // Legacy shootingPressed reads DB0, the opposing-direction pair
              // flag. Actual native HasShoot is not observed by this field.
              assert.equal(typeof nativePlayer.shootingPressed, 'boolean', 'Actual native opposed-pair DB0 readback required');
              equal(nativePlayer.shootingPressed, false, 'actual native DB0 under single-direction public tape', presentation);
              if (owner.pickupAnimationGuard && state.pressed.includes('ArrowRight')) {
                equal(state.player.shooting, false, 'actual port processed shooting under own Pickup guard', presentation);
              }
              if (state.pressed.includes('ArrowRight') && nativePlayer.mainSprite.animation === 'Pickup'
                  && nativePlayer.pickupAnimationGuard) {
                pickupShotGuardPhases.add(source.systemCounter % 2);
                equal(state.tears.filter(tear => tear.motion.lifetime === 'alive').length,
                  source.world.entities.filter(entity => entity.type === 2 && !entity.removed).length,
                  'actual Pickup-guard emission', presentation);
              }
            }
            equal(owner.temporaryCollectibles, nativePlayer.temporaryCollectibles, 'actual temporary use records', presentation);
            equal(playerWorld, nativeActor.position, 'public active-use stored position', presentation);
            equal(playerVelocity, nativeActor.velocity, 'public active-use stored velocity', presentation);
            if (nativePlayer.mainSprite.animation === 'UseItem' || state.playerAnimation.current === 'UseItem') {
              equal(state.tears.filter(tear => tear.motion.lifetime === 'alive').length,
                source.world.entities.filter(entity => entity.type === 2 && !entity.removed).length,
                'actual guarded-shot emission', presentation);
              equal(state.playerAnimation.current, nativePlayer.mainSprite.animation, 'actual UseItem name', presentation);
              equal(state.playerAnimation.frame, nativePlayer.mainSprite.animationFrame, 'actual UseItem frame', presentation);
              equal(state.playerAnimation.playing, nativePlayer.mainSprite.animationPlaying, 'actual UseItem playing', presentation);
            }
          }
          if (owner.pickupPresentation) {
            equal(nativePlayer.pickupSprite.animation, 'PlayerPickupSparkle', 'actual copied presentation clip', presentation);
            equal(owner.pickupPresentation.animationFrame, nativePlayer.pickupSprite.animationFrame, 'copied presentation frame', presentation);
            equal(owner.pickupPresentation.animationPlaying, nativePlayer.pickupSprite.animationPlaying, 'copied presentation playback', presentation);
          }
          for (const slot of source.collectionSounds) {
            const initial = baseline.observation.collectionSounds.find(initial => initial.id === slot.id);
            if (slot.lastTick === initial.lastTick) continue;
            const actual = state.audio.slots[String(slot.id)];
            equal(actual.loaded, slot.loaded, `SFX${slot.id}:actual loaded admission`, presentation);
            equal(f32(actual.lastVolume), slot.lastVolume, `SFX${slot.id}:gain`, presentation);
            equal(actual.lastTick - Math.floor(start.systemClock.systemCounter / 2),
              slot.lastTick - Math.floor(baseline.observation.systemCounter / 2), `SFX${slot.id}:relative public tick`, presentation);
            equal(actual.nextAllowed - actual.lastTick, slot.nextAllowed - slot.lastTick, `SFX${slot.id}:cooldown`, presentation);
          }
          const callback = {nativeCacheFlags: nativePlayer.cacheFlags, browserUnevaluatedCacheFlags: owner.unevaluatedCacheFlags,
            nativeItemRngSha256: digest(JSON.stringify(nativePlayer.itemRngWords)), browserItemRngSha256: digest(JSON.stringify(owner.itemRngs.map(words))),
            nativeFamiliars: source.world.entities.filter(entity => entity.type === 3).map(entity => ({variant: entity.variant, subtype: entity.subtype, spawnSeed: entity.spawnSeed, position: entity.position})),
            postGrantMotion: nativePlayer.ownedCounts.some(count => count > 0) ? {
              nativePosition: nativeActor.position, browserPosition: playerWorld,
              nativeVelocity: nativeActor.velocity, browserVelocity: playerVelocity,
              scope: statCoverage ? 'Every post-grant Player XY/VXY is a required paired comparison' : 'After grant, item-specific movement/stat effects remain outside this count/grant leaf',
            } : null};
          const signature = JSON.stringify(callback);
          if (signature !== callbackDiagnosticSignature) {
            callbackDiagnosticSignature = signature; callbackDiagnostics.push({presentation, ...callback});
          }
        }
        const nativeMtSha256 = digest(JSON.stringify([source.sharedMt.index, source.sharedMt.words]));
        const browserMtSha256 = digest(JSON.stringify([state.bootLifecycle.shared.index, state.bootLifecycle.shared.words]));
        const mtSignature = nativeMtSha256 + browserMtSha256;
        if (mtSignature !== mtDiagnosticSignature) {
          mtDiagnosticSignature = mtSignature;
          mtDiagnostics.push({presentation, status: nativeMtSha256 === browserMtSha256 ? 'pass' : 'mismatch', nativeMtSha256, browserMtSha256,
            nativeIndex: source.sharedMt.index, browserIndex: state.bootLifecycle.shared.index});
        }
        const originals = source.world.entities.filter(entity => entity.type === 5 && entity.variant === 100);
        const actuals = state.room.pickups.filter(pickup => pickup.variant === 100);
        equal(actuals.length, originals.length, 'natural pedestal admission', presentation);
        admitted = Math.max(admitted, originals.length);
        for (let i = 0; i < originals.length; i++) {
          const actual = actuals[i], original = originals[i]; if (!actual) continue;
          if (original.subtype !== 0) selectedItemIds.add(original.subtype);
          for (const field of ['subtype', 'spawnSeed', 'animation', 'animationFrame', 'animationPlaying', 'collisionRadius']) {
            equal(actual[field], original[field], `pedestal${i}:${field}`, presentation);
          }
          // These native/Rust fields are binary32. Recover their exact f32
          // values from serde's shortest decimal JSON; integers stay exact.
          equal([f32(actual.presentedPosition.x), f32(actual.presentedPosition.y)], original.position, `pedestal${i}:world position`, presentation);
          if (acquisition) {
            equal([f32(actual.velocity.x), f32(actual.velocity.y)], original.velocity, `pedestal${i}:stored velocity`, presentation);
            equal(f32(actual.currentFriction), original.friction, `pedestal${i}:current friction`, presentation);
            equal(f32(actual.collisionMass), original.collisionMass, `pedestal${i}:authored mass`, presentation);
          }
          equal(actual.charge, source.pedestals[i].charge, `pedestal${i}:authored charge`, presentation);
          equal(actual.touched, source.pedestals[i].touched, `pedestal${i}:touched`, presentation);
          equal(actual.overlay, original.overlay, `pedestal${i}:actual overlay`, presentation);
          equal(actual.overlayFrame, original.overlayFrame, `pedestal${i}:actual overlay frame`, presentation);
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
          equal(actual.collected, acquisition ? original.subtype === 0 : false, `pedestal${i}:collection boundary`, presentation);
        }
        if (result.failures.length === prior) passedPresentations++;
      };
      compare(start, baseline, 0);
      for (const {key, frames} of item.actions) {
        const code = key === 'space' ? 'Space' : key ? ['Right', 'Left', 'Up', 'Down'].includes(key) ? 'Arrow' + key : 'Key' + key.toUpperCase() : null;
        const domKey = key === 'space' ? 'Space' : key && ['Right', 'Left', 'Up', 'Down'].includes(key) ? code : key;
        if (key) { await page.keyboard.down(domKey); expected.push({type: 'key', code, down: true}); }
        for (let index = 0; index < frames; index++) {
          const state = await stepSnapshot(page); states.push(state);
          expected.push({type: 'tick', count: 1});
          presentations++;
          compare(state, rows.get(baseline.context.presentedFrame + presentations), presentations);
        }
        if (key) { await page.keyboard.up(domKey); expected.push({type: 'key', code, down: false}); }
      }
      // Preserve the completed independent browser capture before a required
      // coverage or provenance assertion can fail.
      const trace = await page.evaluate(() => window.game.trace());
      await archiveBrowser(trace, 'complete');
      Object.assign(result, {checks, comparedPresentations: presentations + 1,
        passedPresentations, admittedPedestals: admitted, selectedItemIds: [...selectedItemIds],
        queuedPresentations, grantedPresentations, pickupShotGuardPhases: [...pickupShotGuardPhases].sort(),
        inputObservationScope: item.requirePickupShotGuard ? {
          nativeDb0: 'Actual opposed-direction pair byte, preserved under its legacy shootingPressed field name',
          nativeHasShoot: 'Unobserved internal InputManager action4..7 query owner; public tape is not its readback',
          portShooting: 'Actual processed port shooting state under its own Pickup guard',
        } : undefined,
        nativeManifestSha256: digest(nativeBytes), tapeSha256: digest(Buffer.from(tape))});
      assert.ok(admitted > 0, 'Every case must naturally admit a source-selected pedestal');
      if (acquisition) {
        assert.ok(queuedPresentations > 0, 'Every acquisition case must observe the actual deferred queue');
        assert.ok(grantedPresentations > 0, 'Every acquisition case must observe a later actual grant');
      }
      if (item.requirePickupShotGuard) {
        assert.deepEqual([...pickupShotGuardPhases].sort(), [0, 1],
          'Public held shooting must exercise the actual original Pickup guard on both presentations');
      }
      if (activeUse && item.expectedItemId !== undefined) {
        assert.ok(grantedItemIds.has(item.expectedItemId), 'Contrived use requires actual independent native45 acquisition');
        assert.ok(states.some(state => state.playerAnimation.current === 'UseItem'), 'Concrete browser use callback must execute');
        assert.ok([...rows.values()].some(row => row.observation.players?.some(player => player.mainSprite.animation === 'UseItem')), 'Concrete original use callback must execute');
      }
      if (statCoverage && item.expectedItemId !== undefined) {
        equal([...selectedItemIds], [item.expectedItemId], 'declared natural passive selection', presentations);
        equal([...grantedItemIds], [item.expectedItemId], 'declared natural passive grant', presentations);
        // Include the actual late coverage comparisons in the complete archive.
        // The earlier archive still preserves evidence if coverage throws.
        result.checks = checks;
        await archiveBrowser(trace, 'complete');
      }
      assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed);
      if (!acquisition) assert.equal(trace.stateHistoryTruncated, false, 'Selection requires the complete bounded state history');
      assert.ok(trace.backendEvents.every(event => event.state),
        'Every actual asynchronous backend callback must retain its own full state; gameplay snapshots cannot replace it');
      assert.equal(states.length, item.actions.reduce((sum, action) => sum + action.frames, 0),
        'Every declared gameplay presentation must have its independent complete snapshot');
      assertSeededRun(await snapshot(page), item.code, item.seedValue);
      assert.deepEqual(errors, []);
      Object.assign(result, {status: result.failures.length ? 'fail' : 'pass', checks, comparedPresentations: presentations + 1,
        passedPresentations, admittedPedestals: admitted, selectedItemIds: [...selectedItemIds],
        queuedPresentations, grantedPresentations, ageClockDiagnostics, callbackDiagnostics, mtDiagnostics,
        grantCoverage: {nativeGrantedConfigs: [...grantedItemIds].map(id => {
          const config = catalogue.collectibles.find(config => config.id === id);
          return {id, kind: config.kind, inventory: config.inventory, maxCharges: config.maxCharges};
        }), firstActiveGrant: activeGrantObserved ? 'observed' : 'unaccepted: no active transition in this case',
          healthGrant: healthGrantObserved ? 'observed' : 'unaccepted: unchanged health does not prove helper boundaries',
          healthClampBoundaries: 'Unaccepted until required dedicated cap/parity cases are retained'},
        unresolvedScope: {cacheAndItemEffects: statCoverage ? 'Selected normal-Isaac1/4 damage/delay and14/27/28/143 shot-speed/range/speed callbacks are implemented. Anti-Gravity222 cache2 alone is mapped; flag32 and its tear updates remain pending. Every cached field/post-grant motion mismatch remains required; other requested owners remain explicit.' : 'Pending requested flags and native stat/familiar callbacks are not interchangeable; raw native/browser states preserve all differences',
          tearAssociation: damageFireDelay ? 'Required native Tear838/linked Player ownership and browser birth ordinal are checked separately from death/pool membership. If owner or ordinal uniqueness fails, remaining row comparisons are candidates; every failure stays blocking. No proximity/manager-row matching or native input injection.' : 'Stored tear velocities compare ordinal rows. Different counts or birth identities make them candidate comparisons, not proven same-actor motion defects; all required failures remain retained.',
          sharedMt: 'Complete624-word hashes are diagnostic until remaining real run/ambient/item callbacks are implemented; no cursor skips or native state inputs',
          stateHistory: 'Every presentation is independently retained in browserEvidence.states; bounded production trace truncation remains separately disclosed'},
        nativeManifestSha256: digest(nativeBytes), tapeSha256: digest(Buffer.from(tape)),
        inputProvenance: {status: 'pass', commands: trace.commands.length, stateHistoryTruncated: trace.stateHistoryTruncated,
          stateHistoryTruncationReason: trace.stateHistoryTruncationReason, stateEncodedBytes: trace.stateEncodedBytes,
          maxStateEncodedBytes: trace.maxStateEncodedBytes, independentlyCapturedStates: states.length,
          inputTraceTruncated: trace.inputTraceTruncated, seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed}});
    } catch (error) { result.status = 'fail'; result.error = String(error); }
    finally {
      if (page && !result.browserEvidence) {
        let trace = null, traceError;
        try { trace = await page.evaluate(() => window.game.trace()); }
        catch (error) { traceError = String(error); }
        try { await archiveBrowser(trace, 'partial', traceError); }
        catch (error) { result.status = 'fail'; result.browserArchiveError = String(error); }
      }
      if (page) await page.close();
      if (acquisition && !replayOriginal) {
        try { result.nativeEvidence = await archiveCompletedNative(directory); }
        catch (error) { result.status = 'fail'; result.archiveError = String(error); }
      }
    }
    await writeFile(join(directory, 'report.json'), JSON.stringify(result, null, 2) + '\n');
    console.log(JSON.stringify({case: item.name, status: result.status, checks: result.checks, failures: result.failures.length, error: result.error}));
  }
  report.status = report.cases.every(result => result.status === 'pass') ? 'pass' : 'fail';
} finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser.close(); await server.close();
  console.log(JSON.stringify({status: report.status, report: join(output, 'report.json'), cases: report.cases.length}));
  process.exitCode = report.status === 'pass' ? 0 : 1;
}
