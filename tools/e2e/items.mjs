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

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const PIN = '724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const files = ['tools/e2e/items.mjs', 'tools/original/item-scenarios.py', 'tools/original/item-observer.py',
  'tools/e2e/item-room-evidence.mjs',
  'tools/foundation/item-catalog.py', 'tools/foundation/decode-rooms.py', 'tools/original/foundation-seeds.py',
  'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c',
  'tools/original/foundation-clock-verify.py', 'tools/original/foundation-pickup-observe.py',
  'tools/original/state-observer.py', 'tools/original/with-display.sh', 'tools/foundation/verify.mjs',
  'tools/foundation/browser.mjs', 'tools/foundation/production-server.mjs', 'web/trace.ts',
  'engine/src/items.rs', 'engine/src/seed.rs', 'engine/src/pickups.rs', 'engine/src/lib.rs', 'engine/build.rs'];
files.push('web/main.ts', 'engine/src/phase.rs', 'engine/src/run_admission.rs', 'engine/src/active_items.rs', 'engine/src/public_actions.rs', 'engine/src/player_animation.rs',
  'tools/original/active-item-scenarios.py', 'engine/src/player_items.rs', 'web/engine.ts', 'web/animation.ts', 'web/pickups.ts', 'web/renderer.ts');
files.push('web/scene-data.json', 'web/pickup-data.json');
files.push('engine/src/combat.rs', 'engine/src/grid.rs', 'engine/src/room.rs');
const hashes = async () => Object.fromEntries(await Promise.all(files.map(async path => [path, digest(await readFile(join(ROOT, path)))])));
const before = await hashes();
const value = flag => { const i = process.argv.indexOf(flag); return i < 0 ? undefined : process.argv[i + 1]; };
const activeUse = process.argv.includes('--active-use');
const acquisition = activeUse || process.argv.includes('--acquisition');
const replayOriginal = value('--replay-original') ? resolve(value('--replay-original')) : null;
assert.ok(!replayOriginal || acquisition, 'Frozen original recovery is a separately disclosed acquisition comparison');
const output = resolve(value('--output') || join(ROOT, activeUse ? '.cache/e2e/active-item-use' : acquisition ? '.cache/e2e/item-acquisition' : '.cache/e2e/items', String(Date.now())));
await mkdir(output, { recursive: true });
const frozenReportBytes = replayOriginal ? await readFile(join(replayOriginal, 'report.json')) : null;
const frozenReport = frozenReportBytes ? JSON.parse(frozenReportBytes) : null;
const scenarios = replayOriginal ? JSON.parse(await readFile(join(replayOriginal, 'scenarios.json'), 'utf8')) :
  value('--scenarios') ? JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')) :
  JSON.parse((await run('python3', [activeUse ? 'tools/original/active-item-scenarios.py' : 'tools/original/item-scenarios.py', '--count', value('--count') || '2',
    ...(acquisition && !activeUse ? ['--acquisition'] : [])], { cwd: ROOT, timeout: 600000 })).stdout);
assert.equal(scenarios.schema, 1);
assert.equal(scenarios.bootEpochSeed, 1700000000);
assert.ok(scenarios.cases.length >= 4 && scenarios.cases.length <= (activeUse ? 10 : acquisition ? 8 : 6));
assert.equal(new Set(scenarios.cases.map(item => item.name)).size, scenarios.cases.length);
for (const item of scenarios.cases) {
  assert.match(item.name, /^[a-zA-Z0-9_-]{1,32}$/);
  assert.match(item.code, /^[A-Z0-9]{8}$/);
  assert.ok(Number.isInteger(item.seedValue) && item.seedValue > 0 && item.seedValue <= 0xffffffff);
  assert.ok([0, 1].includes(item.delay));
  if (acquisition && !activeUse && !replayOriginal && scenarios.mode !== 'acquisition') {
    item.actions.push({key: 'w', frames: 32});
    if (item.entropy.startsWith('Contrived')) item.actions.push({key: 'Right', frames: 50}, {key: null, frames: 50});
    else item.actions.push({key: null, frames: 100});
  }
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
  scope: activeUse ? 'Natural independent original/browser acquisition and public active use; authored charge/count/temporary records/health/7d4/7d8/UseItem/motion/SFX admission are required. Full raw fields, inputs, backend histories and all random failures remain retained; pixels and complete shared MT stay diagnostics. Other active effects and charge modifiers remain explicitly unimplemented.' : acquisition ? 'Natural first free collectible contact, real deferred Pickup animation/queue, all347 owned counts, authored inventory/health grants, first active ownership/charge, pool removal and collection sound admission. Baseline complete450 Player child RNGs are required. Actual stat/familiar/cache callbacks, later child RNG mutation, active swaps/use, achievements, paid/override contexts and pixels remain unaccepted; full MT/cache/child RNG differences are retained diagnostics.' : 'Fresh-profile authored catalogue, causal run/pool seeds, complete 16 pools/497 weights and ID maps, natural free treasure pedestal Init/selection, phase/animation/charge and public seed/input provenance. Acquisition, item effects/cache evaluation, familiars, paid/override contexts and pixels remain unaccepted.',
  activeUse: activeUse ? {scope: 'Public first acquisition and action9 use; native record/charge/health/ANM/input/motion and157 admission required. Other active leaves remain red when exercised, with every OS discovery retained.'} : null,
  selection: selectedCase ? { scope: 'Targeted recorded public case; default gate runs the complete cohort', case: selectedCase } : { scope: 'Complete cohort' }, cases: [] };
report.originalCapture = replayOriginal ? {mode: 'frozen-current-turn-original-recovery',
  reportPath: join(replayOriginal, 'report.json'), reportSha256: digest(frozenReportBytes),
  recordedProduction: frozenReport.production, recordedGateSources: frozenReport.gateSources,
  scope: 'New live production browser replay against exact complete original recordings from the earlier frozen four-case capture; this is not a fresh native recapture'} :
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
      const errors = [], states = [];
      page.on('pageerror', error => errors.push(String(error)));
      await page.goto(server.url + '/?manual=1'); await waitForReady(page);
      await page.evaluate(({ seedValue, bootEpochSeed }) => {
        window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed });
      }, { seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed });
      const start = await enterSeededRun(page, item.code);
      assertSeededRun(start, item.code, item.seedValue);
      const expected = [...manualSeedCommands(item.code)];
      let checks = 0, presentations = 0, passedPresentations = 0, admitted = 0, queuedPresentations = 0, grantedPresentations = 0;
      const selectedItemIds = new Set(), callbackDiagnostics = [], mtDiagnostics = [];
      const grantedItemIds = new Set(); let activeGrantObserved = false, healthGrantObserved = false;
      let callbackDiagnosticSignature = null, mtDiagnosticSignature = null;
      const ageClockDiagnostics = [];
      const equal = (actual, expected, field, presentation) => {
        checks++;
        try { assert.deepEqual(actual, expected, `${item.name}:${presentation}:${field}`); }
        catch (error) { result.failures.push({ field, presentation, actual, expected, error: String(error.message) }); }
      };
      const compare = (state, row, presentation) => {
        const source = row.observation;
        assert.equal(source.status, 'pass'); assert.equal(source.memoryWritten, false);
        assert.equal(source.executableMappingsRead, false); assert.equal(source.nativeBinarySha256, PIN);
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
          if (nativePlayer.ownedCounts.every(count => count === 0)) {
            equal(playerWorld, nativeActor.position, 'ordinary motion through queued Pickup', presentation);
            equal(playerVelocity, nativeActor.velocity, 'ordinary velocity through queued Pickup', presentation);
          }
          equal(owner.ownedCounts, nativePlayer.ownedCounts, 'all347 actual owned counts', presentation);
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
            equal(owner.temporaryCollectibles, nativePlayer.temporaryCollectibles, 'actual temporary use records', presentation);
            equal(playerWorld, nativeActor.position, 'public active-use stored position', presentation);
            equal(playerVelocity, nativeActor.velocity, 'public active-use stored velocity', presentation);
            if (nativePlayer.mainSprite.animation === 'UseItem' || state.playerAnimation.current === 'UseItem') {
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
              scope: 'After grant, item-specific movement/stat effects remain outside this count/grant leaf',
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
          await page.evaluate(() => { window.game.step(1); });
          expected.push({type: 'tick', count: 1});
          const state = await page.evaluate(() => window.game.snapshot()); states.push(state);
          presentations++;
          compare(state, rows.get(baseline.context.presentedFrame + presentations), presentations);
        }
        if (key) { await page.keyboard.up(domKey); expected.push({type: 'key', code, down: false}); }
      }
      assert.ok(admitted > 0, 'Every case must naturally admit a source-selected pedestal');
      if (acquisition) {
        assert.ok(queuedPresentations > 0, 'Every acquisition case must observe the actual deferred queue');
        assert.ok(grantedPresentations > 0, 'Every acquisition case must observe a later actual grant');
      }
      if (activeUse && item.expectedItemId !== undefined) {
        assert.ok(grantedItemIds.has(item.expectedItemId), 'Contrived use requires actual independent native45 acquisition');
        assert.ok(states.some(state => state.playerAnimation.current === 'UseItem'), 'Concrete browser use callback must execute');
        assert.ok([...rows.values()].some(row => row.observation.players?.some(player => player.mainSprite.animation === 'UseItem')), 'Concrete original use callback must execute');
      }
      const trace = await page.evaluate(() => window.game.trace());
      assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed);
      if (!acquisition) assert.equal(trace.stateHistoryTruncated, false, 'Selection requires the complete bounded state history');
      assert.ok(trace.backendEvents.every(event => event.state),
        'Every actual asynchronous backend callback must retain its own full state; gameplay snapshots cannot replace it');
      assert.equal(states.length, item.actions.reduce((sum, action) => sum + action.frames, 0),
        'Every declared gameplay presentation must have its independent complete snapshot');
      assertSeededRun(await page.evaluate(() => window.game.snapshot()), item.code, item.seedValue);
      assert.deepEqual(errors, []);
      const browserBytes = Buffer.from(JSON.stringify({schema: 1, states, trace}) + '\n');
      const browserPath = join(directory, 'browser-states.json.gz');
      await writeFile(browserPath, gzipSync(browserBytes));
      Object.assign(result, {status: result.failures.length ? 'fail' : 'pass', checks, comparedPresentations: presentations + 1,
        passedPresentations, admittedPedestals: admitted, selectedItemIds: [...selectedItemIds],
        queuedPresentations, grantedPresentations, ageClockDiagnostics, callbackDiagnostics, mtDiagnostics,
        grantCoverage: {nativeGrantedConfigs: [...grantedItemIds].map(id => {
          const config = catalogue.collectibles.find(config => config.id === id);
          return {id, kind: config.kind, inventory: config.inventory, maxCharges: config.maxCharges};
        }), firstActiveGrant: activeGrantObserved ? 'observed' : 'unaccepted: no active transition in this case',
          healthGrant: healthGrantObserved ? 'observed' : 'unaccepted: unchanged health does not prove helper boundaries',
          healthClampBoundaries: 'Unaccepted until required dedicated cap/parity cases are retained'},
        unresolvedScope: {cacheAndItemEffects: 'Pending requested flags and native stat/familiar callbacks are not interchangeable; raw native/browser states preserve all differences',
          sharedMt: 'Complete624-word hashes are diagnostic until remaining real run/ambient/item callbacks are implemented; no cursor skips or native state inputs',
          stateHistory: 'Every presentation is independently retained in browserEvidence.states; bounded production trace truncation remains separately disclosed'},
        browserEvidence: {path: browserPath, encoding: 'gzip', rawSha256: digest(browserBytes), rawBytes: browserBytes.length},
        nativeManifestSha256: digest(nativeBytes), tapeSha256: digest(Buffer.from(tape)),
        inputProvenance: {status: 'pass', commands: trace.commands.length, stateHistoryTruncated: trace.stateHistoryTruncated,
          stateHistoryTruncationReason: trace.stateHistoryTruncationReason, stateEncodedBytes: trace.stateEncodedBytes,
          maxStateEncodedBytes: trace.maxStateEncodedBytes, independentlyCapturedStates: states.length,
          inputTraceTruncated: trace.inputTraceTruncated, seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed}});
    } catch (error) { result.status = 'fail'; result.error = String(error); }
    finally {
      if (page) await page.close();
      if (acquisition && !replayOriginal) {
        try { result.nativeEvidence = await archiveCompletedNative(directory); }
        catch (error) { result.status = 'fail'; result.archiveError = String(error); }
      }
    }
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
