import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { createHash } from 'node:crypto';
import { readFile, writeFile, mkdir, rm, stat } from 'node:fs/promises';
import { gzipSync, gunzipSync } from 'node:zlib';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, assertSeededRun, enterSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';
import { compareRoomContext } from './item-room-evidence.mjs';
import { snapshot, stepSnapshot } from './lib/game.mjs';
import { flag, selectCases } from './lib/cli.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const f32 = Math.fround;
const vector = point => [f32(point.x), f32(point.y)];
const words = rng => [rng.state, rng.rightA, rng.left, rng.rightB];
const value = flag => { const at = process.argv.indexOf(flag); return at < 0 ? undefined : process.argv[at + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/familiars', String(Date.now())));
await mkdir(output, { recursive: true });
const scenarios = value('--scenarios') ? JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')) :
  JSON.parse((await run('python3', ['tools/original/familiar-scenarios.py', '--count', value('--count') || '2'],
    {cwd: ROOT, timeout: 600000})).stdout);
assert.equal(scenarios.schema, 1); assert.equal(scenarios.mode, 'familiar-appearance');
assert.equal(scenarios.bootEpochSeed, 1700000000);
// A bank recorded under --case holds only the cases that ran.
if (!scenarios.caseFilter) assert.ok(scenarios.cases.length >= 4 && scenarios.cases.length <= 6);
assert.equal(new Set(scenarios.cases.map(item => item.name)).size, scenarios.cases.length);
if (!scenarios.caseFilter) assert.ok(scenarios.cases.filter(item => item.entropy === 'OS secrets').length >= 2);
scenarios.cases = selectCases(scenarios.cases, item => item.name);
if (flag('--case')) scenarios.caseFilter = flag('--case');
await writeFile(join(output, 'scenarios.json'), JSON.stringify(scenarios, null, 2) + '\n');
await assertOriginalBinary();
const server = await startProductionServer();
const browser = await chromium.launch(browserLaunchOptions());
const report = {schema: 1, status: 'running', production: server.build,
  nativeBinarySha256: ORIGINAL_SHA, graphics: await inspectBrowserGraphics(browser),
  scope: 'Natural free grant/cache, child270 and every actual Familiar/effect presence, source actor order/room sequence, full/fast motion, appearance/countdown/sprite/Poof callback and retirement. Placement and Poof seed differences are mandatory failures. Full624-word MT and absolute Game ages stay explicit diagnostics until missing causal consumers are implemented. Path/target/contact/heal/modifiers and other Familiar subclasses remain unaccepted.', caseFilter: scenarios.caseFilter, cases: []};

const modelActor = (state, ref) => {
  let actor, type;
  if (ref.kind === 'player') return {type: 1, variant: 0, subtype: 0, spawnSeed: state.playerItems.entitySpawnSeed};
  if (ref.kind === 'familiar') { actor = state.familiars.find(actor => actor.id === ref.id); type = 3; }
  else if (ref.kind === 'pickup') { actor = state.room.pickups.find(actor => actor.id === ref.id); type = 5; }
  else if (ref.kind === 'tear') { actor = state.tears.find(actor => actor.id === ref.id); type = 2; }
  else if (ref.kind === 'bomb') { actor = state.bombs.find(actor => actor.id === ref.id); type = 4; }
  else if (ref.kind === 'npc') { actor = state.room.npcs.find(actor => actor.id === ref.id); type = actor?.kind; }
  else if (ref.kind === 'projectile') { actor = state.enemyProjectiles.find(actor => actor.id === ref.id); type = 9; }
  else if (ref.kind === 'effect') {
    actor = state.spriteEffects.find(actor => actor.id === ref.id) || state.fireParticles.find(actor => actor.id === ref.id);
    type = 1000;
  }
  assert.ok(actor, 'Every admitted handle must join its actual runtime owner');
  return {type, variant: ref.kind === 'effect' && actor.parentNpc !== undefined ? 66 : actor.variant || 0,
    subtype: actor.subtype || 0, spawnSeed: actor.spawnSeed};
};

const archive = async directory => {
  const path = join(directory, 'original/manifest.json'), bytes = await readFile(path);
  JSON.parse(bytes); const metadata = await stat(path), compressed = path + '.gz';
  await writeFile(compressed, gzipSync(bytes));
  const restored = gunzipSync(await readFile(compressed));
  assert.equal(restored.length, bytes.length); assert.equal(digest(restored), digest(bytes));
  const record = {path: compressed, originalPath: path, encoding: 'gzip', rawBytes: bytes.length,
    rawSha256: digest(bytes), mode: metadata.mode & 0o777, mtimeMs: metadata.mtimeMs,
    scope: 'Lossless completed original capture; all observations/failures preserved'};
  await writeFile(join(directory, 'native-manifest-archive.json'), JSON.stringify(record, null, 2) + '\n');
  await rm(path); return record;
};

try {
  for (const item of scenarios.cases) {
    assert.match(item.name, /^[a-zA-Z0-9_-]{1,32}$/); assert.match(item.code, /^[A-Z0-9]{8}$/);
    assert.ok(Number.isInteger(item.seedValue) && item.seedValue > 0 && item.seedValue <= 0xffffffff);
    assert.ok([0, 1].includes(item.delay));
    const directory = join(output, item.name); await mkdir(directory, {recursive: true});
    const result = {...item, status: 'running', checks: 0, failures: [], mtDiagnostics: [], ageDiagnostics: []};
    report.cases.push(result); let page, nativeBytes;
    try {
      const casePath = join(directory, 'case.json'); await writeFile(casePath, JSON.stringify(item));
      const tape = (await run('python3', ['tools/original/familiar-scenarios.py', '--case', casePath], {cwd: ROOT})).stdout;
      await writeFile(join(directory, 'tape.json'), tape);
      await run('bash', ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
        '--seed', item.code, '--tape', join(directory, 'tape.json'), '--observe-every-step',
        '--state-observer', 'tools/original/familiar-observer.py', '--sfx-volume', '1', '--music-volume', '1',
        '--boot-epoch-seed', String(scenarios.bootEpochSeed), '--frame-timeout', '120',
        '--output', join(directory, 'original')], {cwd: ROOT, timeout: 900000, maxBuffer: 2 * 1024 * 1024});
      const nativePath = join(directory, 'original/manifest.json'); nativeBytes = await readFile(nativePath);
      const native = JSON.parse(nativeBytes); assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
      assert.deepEqual(native.tape, JSON.parse(tape), 'Original must execute the independently requested complete public tape');
      assert.equal(native.seed, item.code); assert.equal(native.seedValue, item.seedValue);
      assert.equal(native.bootEpochSeed, scenarios.bootEpochSeed);
      assert.equal(native.observedState.seed.replaceAll(' ', ''), item.code);
      assert.equal(native.observedState.seedValue, item.seedValue);
      const transport = JSON.parse((await run('python3', ['tools/original/familiar-scenarios.py', '--validate-original', nativePath], {cwd: ROOT})).stdout);
      assert.equal(transport.status, 'pass'); assert.deepEqual(transport.failures, []);
      const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
      assert.equal(baseline?.observation.status, 'pass');
      assert.equal(baseline.context.seedValue, item.seedValue);
      assert.equal(baseline.context.publicLogState.seedValue, item.seedValue);
      const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row]));
      page = await browser.newPage(); const errors = [], states = [];
      page.on('pageerror', error => errors.push(String(error)));
      await page.goto(server.url + '/?manual=1'); await waitForReady(page);
      await page.evaluate(({seedValue, bootEpochSeed}) => {
        window.game.pause(); window.game.reset(seedValue, { trace: true, bootEpochSeed, audioPreferences: {sfxVolume: 1, musicVolume: 1}, fullscreenPreferences: {fullscreen: false} });
      }, {seedValue: item.seedValue, bootEpochSeed: scenarios.bootEpochSeed});
      const start = await enterSeededRun(page, item.code); assertSeededRun(start, item.code, item.seedValue);
      const expected = [...manualSeedCommands(item.code)]; let presentation = 0;
      let grants = 0, familiarPresentations = 0, poofPresentations = 0, lastMt = null;
      const nativeCounters = new Set(), browserCounters = new Set();
      let nativePoofSeen = false, browserPoofSeen = false, nativePoofRetired = false, browserPoofRetired = false;
      const equal = (actual, expected, field, tick) => {
        result.checks++;
        try { assert.deepEqual(actual, expected, `${item.name}:${tick}:${field}`); }
        catch (error) { result.failures.push({field, presentation: tick, actual, expected, error: String(error.message)}); }
      };
      const compare = (state, row, tick) => {
        assert.ok(row, 'Every browser presentation requires an independently retained original phase');
        const source = row.observation;
        assert.equal(source.status, 'pass'); assert.equal(source.memoryWritten, false);
        assert.equal(source.executableMappingsRead, false); assert.equal(source.nativeBinarySha256, ORIGINAL_SHA);
        assert.equal(source.gamePresent, true); assert.equal(source.systemState, 2);
        equal(source.actorOrder.length, source.actorCount, 'complete counted native actor list', tick);
        for (const check of compareRoomContext(state, row)) equal(check.actual, check.expected, check.field, tick);
        equal(state.systemClock.systemCounter % 2, source.systemCounter % 2, 'source full/fast phase', tick);
        equal(source.players.length, 1, 'single normal Isaac owner', tick);
        const owner = state.playerItems, nativeOwner = source.players[0];
        const nativePlayer = source.world.players.find(actor => actor.type === 1 && actor.variant === 0);
        assert.ok(nativePlayer, 'Actual original normal Isaac actor is required');
        equal([f32((state.player.x - 32) / 0.65), f32((state.player.y + 46) / 0.65)], nativePlayer.position,
          'actual Player world pose through grant and Familiar contacts', tick);
        equal([f32(state.player.vx / 0.65), f32(state.player.vy / 0.65)], nativePlayer.velocity,
          'actual Player stored velocity through grant and Familiar contacts', tick);
        equal(owner.ownedCounts, nativeOwner.ownedCounts, 'all347 permanent counts', tick);
        equal(owner.temporaryCollectibles, nativeOwner.temporaryCollectibles, 'actual temporary owner records', tick);
        equal(owner.queuedId, nativeOwner.queuedId, 'actual deferred queue', tick);
        equal(owner.itemRngs.map(words), nativeOwner.itemRngs, 'all347 actual child RNG words', tick);
        equal(words(owner.itemRngs[270]), nativeOwner.child270, 'actual child270', tick);
        if (nativeOwner.ownedCounts.some(count => count > 0)) grants++;
        const playerIdentity = state.actorScheduler.identities.find(identity => identity.actor.kind === 'player' && identity.actor.id === 0);
        equal(playerIdentity?.sequenceId, nativeOwner.sequenceId, 'retained actual Player sequence', tick);
        equal(state.actorScheduler.roomNextSequence, source.nextSequenceId, 'actual Room next sequence owner', tick);
        const order = state.actorScheduler.actors.map(ref => ({...modelActor(state, ref),
          sequenceId: state.actorScheduler.identities.find(identity => identity.actor.kind === ref.kind && identity.actor.id === ref.id)?.sequenceId}));
        // Ordinary seed mismatches remain raw in diagnostics; the complete
        // admitted type/variant/subtype/sequence order is always blocking.
        equal(order.map(({spawnSeed, ...actor}) => actor), source.actorOrder.map(({spawnSeed, ...actor}) => actor), 'all actual actor presence/order/sequence', tick);
        const familiars = source.actors.filter(actor => actor.type === 3 && !actor.removed);
        equal(state.familiars.filter(actor => !actor.removed).length, familiars.length, 'every actual Familiar including unsupported subclasses', tick);
        const effects = source.actors.filter(actor => actor.type === 1000 && !actor.removed);
        const nativePoof = effects.some(actor => actor.variant === 15);
        const browserPoof = state.spriteEffects.some(actor => actor.variant === 15 && !actor.removed);
        if (nativePoofSeen && !nativePoof) nativePoofRetired = true;
        if (browserPoofSeen && !browserPoof) browserPoofRetired = true;
        nativePoofSeen ||= nativePoof; browserPoofSeen ||= browserPoof;
        source.actors.filter(actor => actor.type === 3 && actor.variant === 56).forEach(actor => nativeCounters.add(actor.appearanceCounter));
        state.familiars.filter(actor => actor.variant === 56).forEach(actor => browserCounters.add(actor.appearanceCounter));
        equal(state.spriteEffects.length + state.fireParticles.length, effects.length, 'every actual effect presence', tick);
        const consumed = new Set();
        for (const nativeActor of source.actors) {
          const refKind = nativeActor.type === 3 ? 'familiar' : 'effect';
          const family = nativeActor.type === 3;
          const actual = family ? state.familiars.find(actor => !consumed.has('familiar/' + actor.id) && actor.spawnSeed === nativeActor.spawnSeed && actor.variant === nativeActor.variant) :
            state.spriteEffects.find(actor => !consumed.has('effect/' + actor.id) && actor.variant === nativeActor.variant && actor.subtype === nativeActor.subtype);
          // Effect66 already has a distinct actual production owner. Its
          // detailed lifecycle belongs to the registered Fireplace gate; this
          // leaf requires its real presence/order and retains every raw field.
          if (!actual && nativeActor.type === 1000 && nativeActor.variant === 66) {
            const ember = state.fireParticles.find(actor => !consumed.has('effect/' + actor.id));
            equal(Boolean(ember), true, '1000/66:existing typed Ember owner', tick);
            if (ember) {
              consumed.add('effect/' + ember.id);
              equal(vector(ember.presentedPosition), nativeActor.position, '1000/66:actual f32 world position', tick);
              equal(vector(ember.velocity), nativeActor.velocity, '1000/66:actual f32 velocity', tick);
              equal(ember.spawnSeed, nativeActor.spawnSeed, '1000/66:actual shared spawn seed', tick);
            }
            continue;
          }
          equal(Boolean(actual), true, `${nativeActor.type}/${nativeActor.variant}:typed actor`, tick);
          if (!actual) continue;
          consumed.add(refKind + '/' + actual.id);
          if (nativeActor.type === 3 && nativeActor.variant === 56) familiarPresentations++;
          if (nativeActor.type === 1000 && nativeActor.variant === 15) poofPresentations++;
          const name = `${nativeActor.type}/${nativeActor.variant}/${nativeActor.sequenceId}`;
          const identity = state.actorScheduler.identities.find(identity => identity.actor.kind === refKind && identity.actor.id === actual.id);
          equal(identity?.sequenceId, nativeActor.sequenceId, `${name}:actual sequence`, tick);
          for (const field of ['variant', 'subtype', 'spawnSeed', 'gridCollisionClass', 'entityCollisionClass', 'flags', 'visible', 'active', 'removed']) {
            equal(actual[field], nativeActor[field], `${name}:${field}`, tick);
          }
          for (const field of ['configuredFriction', 'currentFriction', 'collisionRadius', 'collisionMass', 'collisionDamage', 'hp', 'maxHp']) {
            equal(f32(actual[field]), nativeActor[field], `${name}:f32 ${field}`, tick);
          }
          equal(vector(actual.presentedPosition), nativeActor.position, `${name}:f32 world position`, tick);
          equal(vector(actual.velocity), nativeActor.velocity, `${name}:f32 stored velocity`, tick);
          equal(words(actual.actorRng), nativeActor.actorRng, `${name}:actual actor RNG30`, tick);
          equal([actual.sprite.animation, f32(actual.sprite.frame), actual.sprite.playing, f32(actual.sprite.speed)],
            [nativeActor.animation, nativeActor.animationFrame, nativeActor.animationPlaying, nativeActor.animationSpeed], `${name}:actual main Sprite`, tick);
          equal(vector(actual.sprite.scale), nativeActor.spriteScale, `${name}:f32 scale`, tick);
          equal(vector(actual.sprite.offset), nativeActor.spriteOffset, `${name}:f32 offset`, tick);
          equal(actual.sprite.flipX, nativeActor.flipX, `${name}:flipX`, tick);
          equal((state.gameUpdateCount - actual.birthGameUpdate) >>> 0,
            (source.gameUpdateCount - nativeActor.birthGameUpdate) >>> 0, `${name}:real birth age`, tick);
          if (nativeActor.type === 3) {
            equal([actual.parentType, actual.parentVariant], [nativeActor.parentType, nativeActor.parentVariant], `${name}:copied parent fields`, tick);
            equal([actual.owner.kind, actual.owner.id, nativeActor.ownerOrdinal, nativeActor.ownerSpawnSeed],
              ['player', 0, 0, owner.entitySpawnSeed], `${name}:actual Player owner join`, tick);
            equal([actual.appearanceCounter, actual.followerIndex], [nativeActor.appearanceCounter, nativeActor.followerIndex], `${name}:appearance/follower fields`, tick);
          } else {
            equal([actual.parent?.entityType || 0, actual.parent?.variant || 0], [nativeActor.parentType, nativeActor.parentVariant], `${name}:copied parent fields`, tick);
            for (const field of ['renderOrderOffset', 'lifetimeDivisor', 'timeToLive', 'raw7f4']) equal(actual[field], nativeActor[field], `${name}:${field}`, tick);
            equal(f32(actual.raw7f8), nativeActor.raw7f8, `${name}:f32 raw7f8`, tick);
          }
          result.ageDiagnostics.push({presentation: tick, actor: name,
            native: {born: nativeActor.birthGameUpdate, current: source.gameUpdateCount},
            browser: {born: actual.birthGameUpdate, current: state.gameUpdateCount},
            unresolvedAiUpdates: actual.unresolvedAiUpdates || 0});
        }
        const nativeMt = digest(JSON.stringify([source.sharedMt.index, source.sharedMt.words]));
        const browserMt = digest(JSON.stringify([state.bootLifecycle.shared.index, state.bootLifecycle.shared.words]));
        if (nativeMt + browserMt !== lastMt) {
          result.mtDiagnostics.push({presentation: tick, status: nativeMt === browserMt ? 'pass' : 'mismatch',
            nativeIndex: source.sharedMt.index, browserIndex: state.bootLifecycle.shared.index,
            nativeSha256: nativeMt, browserSha256: browserMt, nativeCacheFlags: nativeOwner.cacheFlags,
            browserUnevaluatedCacheFlags: owner.unevaluatedCacheFlags, nativeActorOrder: source.actorOrder, browserActorOrder: order});
          lastMt = nativeMt + browserMt;
        }
      };
      compare(start, baseline, 0);
      for (const {key, frames} of item.actions) {
        const arrow = key && ['Right', 'Left', 'Up', 'Down'].includes(key);
        const code = key ? arrow ? 'Arrow' + key : 'Key' + key.toUpperCase() : null;
        const domKey = arrow ? code : key;
        if (key) { await page.keyboard.down(domKey); expected.push({type: 'key', code, down: true}); }
        for (let frame = 0; frame < frames; frame++) {
          const state = await stepSnapshot(page); expected.push({type: 'tick', count: 1});
          states.push(state); presentation++;
          compare(state, rows.get(baseline.context.presentedFrame + presentation), presentation);
        }
        if (key) { await page.keyboard.up(domKey); expected.push({type: 'key', code, down: false}); }
      }
      assert.ok(grants > 0, 'Every unfiltered route must execute an actual natural grant');
      if (item.entropy.startsWith('Contrived')) {
        assert.ok(familiarPresentations > 0, 'Both contrived phases must observe actual Familiar admission');
        assert.ok(poofPresentations > 0, 'Both contrived phases must observe the real appearance Poof15');
        for (const counter of [4, 3, 2, 1, 0]) {
          assert.ok(nativeCounters.has(counter), 'Native must observe every declared appearance counter including reveal');
          assert.ok(browserCounters.has(counter), 'Browser must observe every declared appearance counter including reveal');
        }
        assert.ok(nativePoofRetired && browserPoofRetired, 'Both actual Poof15 actors must retire within the declared hold');
      }
      const trace = await page.evaluate(() => window.game.trace());
      assertInputProvenance(trace, expected, item.seedValue, scenarios.bootEpochSeed);
      assert.equal(trace.backendStateTruncated, false, 'Actual backend state evidence must remain complete');
      assert.ok(trace.backendEvents.every(event => event.state?.schema === 1), 'Every real backend callback requires its full state');
      assert.equal(states.length, item.actions.reduce((total, action) => total + action.frames, 0));
      assertSeededRun(await snapshot(page), item.code, item.seedValue);
      assert.deepEqual(errors, []);
      const bytes = Buffer.from(JSON.stringify({schema: 1, baseline: start, states, trace}) + '\n');
      const path = join(directory, 'browser-states.json.gz'); await writeFile(path, gzipSync(bytes));
      Object.assign(result, {status: result.failures.length ? 'fail' : 'pass', comparedPresentations: presentation + 1,
        grants, familiarPresentations, poofPresentations, appearanceCoverage: {nativeCounters: [...nativeCounters], browserCounters: [...browserCounters], nativePoofRetired, browserPoofRetired}, nativeManifestSha256: digest(nativeBytes), tapeSha256: digest(Buffer.from(tape)),
        browserEvidence: {path, encoding: 'gzip', rawSha256: digest(bytes), rawBytes: bytes.length},
        inputProvenance: {status: 'pass', commands: trace.commands.length, inputTraceTruncated: trace.inputTraceTruncated,
          stateHistoryTruncated: trace.stateHistoryTruncated, stateHistoryTruncationReason: trace.stateHistoryTruncationReason,
          independentlyCapturedStates: states.length},
        unaccepted: ['Path/target/contact/heal/modifiers/other Familiar subclasses', 'Absolute cutscene/Game age alignment',
          'Complete shared MT numeric parity and unmodeled real consumers', 'Full-frame pixel parity']});
    } catch (error) { result.status = 'fail'; result.error = String(error); }
    finally {
      if (page) await page.close();
      try { result.nativeEvidence = await archive(directory); }
      catch (error) { result.status = 'fail'; result.archiveError = String(error); }
    }
    await writeFile(join(directory, 'report.json'), JSON.stringify(result, null, 2) + '\n');
    console.log(JSON.stringify({case: item.name, status: result.status, checks: result.checks, failures: result.failures.length, error: result.error}));
  }
  report.status = report.cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
} finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser.close(); await server.close();
  console.log(JSON.stringify({status: report.status, report: join(output, 'report.json'), cases: report.cases.length}));
  process.exitCode = report.status === 'pass' ? 0 : 1;
}
