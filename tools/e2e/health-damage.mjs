// Fresh original and immutable production browser through identical public inputs.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {runPairedGate, scopedState} from '../foundation/paired-game.mjs';
import { ORIGINAL_SHA } from './lib/original.mjs';
const option = (name, fallback) => {const at = process.argv.indexOf(name); return at < 0 ? fallback : process.argv[at + 1];};
const output = resolve(option('--output', `.cache/e2e/health-damage/${Date.now()}-${process.pid}`));
mkdirSync(output, {recursive: true});
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n');
const sha = value => createHash('sha256').update(value).digest('hex');
// The whole OS cohort is drawn and persisted before any run or grading.
const scenarioFile = option('--scenario-file', undefined);
const scenarios = scenarioFile ? JSON.parse(readFileSync(resolve(scenarioFile))) :
  JSON.parse(execFileSync('python3', ['tools/original/health-damage-scenarios.py', '--count', option('--count', '2')],
    {encoding: 'utf8', maxBuffer: 8 * 1024 * 1024}));
assert.equal(scenarios.schema, 1);
assert.ok(Array.isArray(scenarios.cases) && scenarios.cases.length >= 6 && scenarios.cases.length <= 8);
const fixedBankPath = 'references/isaac/foundation/health-damage/first-prepared-cases.json';
const fixedBank = JSON.parse(readFileSync(fixedBankPath));
const mandatory = new Map(['soul', 'black'].flatMap(family => [0, 1].map(phase => {
  const id = `health-${family}-phase${phase}`, fixture = fixedBank.cases.find(c => c.scenario.id === id);
  assert.ok(fixture); return [id, fixture];
})));
const ids = scenarios.cases.map(c => c.scenario.id);
assert.equal(new Set(ids).size, ids.length, 'Health case IDs must be unique');
const identityFields = ['seed', 'seedValue', 'initialSeed', 'bootEpochSeed', 'bootstrap', 'tape', 'pairedTape', 'healthRoute', 'phaseOffset'];
for (const [id, fixture] of mandatory) {
  const selected = scenarios.cases.find(c => c.scenario.id === id); assert.ok(selected, `Missing mandatory ${id}`);
  for (const field of identityFields) assert.deepEqual(selected[field], fixture[field], `${id}: sealed identity and public phase tape`);
  assert.equal(selected.scenario.randomized, false);
}
const osCases = scenarios.cases.filter(c => !mandatory.has(c.scenario.id));
assert.ok(Array.isArray(scenarios.randomVectors) && scenarios.randomVectors.length >= 2 && scenarios.randomVectors.length <= 4);
assert.equal(osCases.length, scenarios.randomVectors.length, 'Retain every recorded OS outcome');
// Build declared OS tapes from recorded words without drawing new entropy or running either game.
const declaredOs = JSON.parse(execFileSync('python3', ['-c',
  "import json,runpy,sys; m=runpy.run_path('tools/original/health-damage-scenarios.py',run_name='declared_health'); print(json.dumps([m['build'](f'health-unfiltered-{n}',v['code'],'soul' if n%2==0 else 'black',n%2,False) for n,v in enumerate(json.loads(sys.argv[1]))]))",
  JSON.stringify(scenarios.randomVectors)], {encoding: 'utf8', maxBuffer: 8 * 1024 * 1024}));
for (const [index, selected] of osCases.entries()) {
  assert.equal(selected.scenario.id, declaredOs[index].scenario.id);
  assert.equal(selected.scenario.randomized, true);
  assert.equal(selected.seedValue, scenarios.randomVectors[index].seedValue);
  for (const field of identityFields) assert.deepEqual(selected[field], declaredOs[index][field], 'Recorded OS identity and declared public tape');
}
save(resolve(output, 'scenarios.json'), scenarios);
const pathIdentity = value => value?.replaceAll('\\', '/').replace(/^resources\//i, '').toLowerCase() ?? null;

function compare(source, actual, gameplay, capture) {
  assert.equal(source?.health?.status, 'pass', 'Every checkpoint needs actual bounded health/System owner');
  assert.equal(source?.death?.status, 'pass', 'Every checkpoint needs actual Player/Death owner');
  const h = source.health, d = source.death;
  const checks = scopedState(source, actual, gameplay && h.systemState === 2);
  const check = (field, expected, value) => checks.push({field, expected, actual: value,
    status: JSON.stringify(expected) === JSON.stringify(value) ? 'pass' : 'fail'});
  assert.equal(source.processExecutable?.sha256, ORIGINAL_SHA);
  if (h.systemState === 2) {
    check('health.containers', h.containers, [actual.player.maxHp, actual.player.hp, actual.player.soulHp,
      actual.player.blackHeartMask, actual.player.eternalHearts]);
    check('health.damageCountdown', h.damageCountdown, actual.damageCooldown);
    if (h.damageCountdown > 0 || d.dead) check('health.incomingFlags', h.damageFlags, actual.playerDeath.lastDamageFlags);
    check('player.dead', d.dead, actual.player.dead);
    check('player.visible', d.visible, actual.player.visible);
    check('player.controls7d4', d.controlsEnabled, actual.player.controlsEnabled);
    // Native Game+ce9f4 starts at the run loader's 1; game_update_count is 0.
    check('Game.fullUpdates', d.fullGameUpdates, actual.gameUpdateCount + 1);
    check('items.ownedCounts', h.ownedCounts, actual.playerItems?.ownedCounts);
    check('items.temporaryRecords', h.temporaryCollectibles, actual.playerItems?.temporaryCollectibles);
    check('items.activeD80', h.activeStateD80, actual.playerItems?.activeStateD80);
    if (h.activeFlag3076 !== null) check('items.active3076', h.activeFlag3076, actual.playerItems?.activeFlag3076);
    check('items.queuedId', h.queuedId, actual.playerItems?.queuedId);
    check('items.queuedCharge', h.queuedCharge, actual.playerItems?.queuedCharge);
    if (h.queuedId !== null) check('items.queuedTouched', h.queuedTouched, actual.playerItems?.queuedTouched);
    check('items.collectionDelay', h.collectionDelay, actual.playerItems?.collectionDelay);
    const legacy = actual.playerItems?.pickupAnimationGuard || actual.playerItems?.pickupAnimationPlaying;
    const main = actual.playerAnimation.current ?? (legacy ? 'Pickup' : null);
    if (d.mainAnimation.name === 'Pickup' || legacy) {
      check('Pickup.current', d.mainAnimation.name, main);
      check('Pickup.frame', d.mainAnimation.frame, actual.playerItems?.pickupAnimationFrame);
      check('Pickup.playing', d.mainAnimation.playing, actual.playerItems?.pickupAnimationPlaying);
      check('Pickup.guard', d.animationGuard, actual.playerItems?.pickupAnimationGuard);
    }
    check('main.exclusiveOwner', false, Boolean(actual.playerAnimation.current && legacy));
    if (h.copiedItemSprite.clip || actual.playerItems?.pickupPresentation) {
      const copy = actual.playerItems?.pickupPresentation;
      check('copy.current', h.copiedItemSprite.clip, copy ? 'PlayerPickupSparkle' : null);
      check('copy.frame', h.copiedItemSprite.frame, copy?.animationFrame);
      check('copy.playing', h.copiedItemSprite.playing, copy?.animationPlaying);
    }
    if (['Hit', 'Death'].includes(d.mainAnimation.name) || ['Hit', 'Death'].includes(actual.playerAnimation.current)) {
      check('damageClip.current', d.mainAnimation.name, actual.playerAnimation.current);
      check('damageClip.frame', d.mainAnimation.frame, actual.playerAnimation.frame);
      check('damageClip.playing', d.mainAnimation.playing, actual.playerAnimation.playing);
      check('damageClip.guard', d.animationGuard, actual.playerAnimation.guard);
    }
    const i = h.informationAnimation, a = actual.informationAnimation;
    check('Info.state', i.state, a.state); check('Info.delay', i.delay, a.delay);
    check('Info.actor', pathIdentity(i.actor), pathIdentity(a.currentActor));
    check('Info.clip', i.clip, a.currentClip); check('Info.frame', i.frame, a.frame);
    check('Info.playing', i.playing, a.playing); check('Info.loaded', i.definitionLoaded, a.definitionLoaded);
    // Every actual NPC participates, including unsupported217 in the Black
    // fixture. Match authored identities by type/variant/subtype occurrence;
    // missing identities/cardinality never become an omitted comparison.
    const actualNpcs = source.gameplay.entities.filter(entity => entity.type >= 10 && entity.type < 1000);
    for (const npc of actualNpcs) assert.equal(typeof npc.active, 'boolean', 'Every NPC needs its actual manager-active flag');
    const npcs = actualNpcs.filter(entity => entity.active);
    const candidates = actual.room.npcs.filter(npc => !npc.removed);
    const identity = npc => [npc.type ?? npc.kind, npc.variant, npc.subtype].join('/');
    check('NPC.identities', npcs.map(identity), candidates.map(identity));
    const used = new Set();
    for (const [index, npc] of npcs.entries()) {
      const at = candidates.findIndex((candidate, n) => !used.has(n) && identity(candidate) === identity(npc));
      if (at >= 0) used.add(at);
      const candidate = candidates[at];
      check(`NPC${index}.dead`, npc.removed, candidate?.dead);
      check(`NPC${index}.hp`, npc.hp, candidate?.hp);
      check(`NPC${index}.maxHp`, npc.maxHp, candidate?.maxHp);
      check(`NPC${index}.pendingDamage`, npc.pendingDamage, candidate?.damage.pendingDamage);
      check(`NPC${index}.pendingFlags`, npc.pendingFlags, candidate?.damage.pendingFlags);
    }
  }
  const sounds = source.healthSounds;
  assert.equal(sounds?.status, 'pass', 'SFX requires actual original wrappers and backend identities');
  check('SFX.catalogCountsMatch', true, sounds.catalogCountsMatch);
  check('SFX.globalVolume', sounds.globalSfxVolume, actual.audio.sfxVolume);
  for (const slot of sounds.slots) {
    const port = actual.audio.slots[String(slot.id)];
    check(`SFX${slot.id}.sampleCount`, slot.count, port?.samples.length);
    check(`SFX${slot.id}.loaded`, slot.loaded, port?.loaded);
    if (slot.nextAllowed >= 0 || port?.nextAllowed >= 0) {
      // Compare actual admission ages rather than injecting the original's
      // unported cutscene/System age into the browser. Virgin lastTick is
      // deliberately unknown; nextAllowed=-1 is the initialized discriminator.
      check(`SFX${slot.id}.admissionAge`, slot.nextAllowed < 0 ? null :
        [(sounds.systemCounter >> 1) - slot.lastTick, slot.nextAllowed - (sounds.systemCounter >> 1)],
        port?.nextAllowed < 0 ? null :
          [(actual.systemClock.systemCounter >> 1) - port?.lastTick, port?.nextAllowed - (actual.systemClock.systemCounter >> 1)]);
    }
    if (slot.loaded || port?.loaded) {
      check(`SFX${slot.id}.lastVolume`, slot.lastVolume, port?.lastVolume);
      for (const [index, sample] of slot.samples.entries()) {
        check(`SFX${slot.id}/${index}.settings`, [sample.loop, sample.gain, sample.pitch],
          port?.samples[index] && [port.samples[index].looping, port.samples[index].gain, port.samples[index].pitch]);
      }
    }
  }
  assert.ok(capture.browserAudio, 'Every selected snapshot requires actual browser audio owner evidence');
  check('browserAudio.traceTruncated', false, capture.browserAudio.truncated);
  if (gameplay) check('browserAudio.context', 'running', capture.browserAudio.contextState);
  return checks;
}

function manifest(directory, browser) {
  const base = resolve(directory, browser ? 'browser' : 'original');
  const raw = resolve(base, 'manifest.json');
  if (existsSync(raw)) return {document: JSON.parse(readFileSync(raw)), evidence: {encoding: 'json', rawSha256: sha(readFileSync(raw))}};
  const descriptor = JSON.parse(readFileSync(resolve(base, 'manifest-archive.json')));
  const archive = readFileSync(resolve(base, 'manifest.json.gz'));
  assert.equal(archive.length, descriptor.archiveBytes, 'Complete browser archive byte count');
  assert.equal(sha(archive), descriptor.archiveSha256, 'Complete browser archive hash');
  const bytes = gunzipSync(archive);
  assert.equal(bytes.length, descriptor.rawBytes, 'Complete browser raw byte count');
  assert.equal(sha(bytes), descriptor.rawSha256, 'Complete browser raw hash');
  return {document: JSON.parse(bytes), evidence: descriptor};
}
function coverage(definition, directory) {
  const source = manifest(directory, false), browser = manifest(directory, true);
  const original = source.document.debugObservations.map(row => ({label: row.context.label, ...row.observation}));
  const candidates = browser.document.frames;
  const required = mandatory.get(definition.scenario.id)?.healthRequired ?? null;
  const sourceAcquired = original.some(row => row.health?.containers?.[2] > 0 &&
    (required === 'black' ? row.health.containers[3] !== 0 : row.health.ownedCounts[72] > 0));
  const browserAcquired = candidates.some(row => row.state.player.soulHp > 0 &&
    (required === 'black' ? row.state.player.blackHeartMask !== 0 : row.state.playerItems?.ownedCounts[72] > 0));
  const sourceSoulDamage = original.some((row, n) => n > 0 && row.health?.containers?.[2] < original[n - 1].health?.containers?.[2]
    && row.health.containers[1] === original[n - 1].health.containers[1]);
  const browserSoulDamage = candidates.some((row, n) => n > 0 && row.state.player.soulHp < candidates[n - 1].state.player.soulHp
    && row.state.player.hp === candidates[n - 1].state.player.hp);
  const sourceEffect = original.some(row => row.health?.temporaryCollectibles?.some(record => record.itemId === 35 && record.count > 0));
  const browserEffect = candidates.some(row => row.state.playerItems?.temporaryCollectibles.some(record => record.itemId === 35 && record.count > 0));
  const sourceBook = original.some(row => row.health?.informationAnimation?.state === 2 && row.health.informationAnimation.clip === 'Shake');
  const browserBook = candidates.some(row => row.state.informationAnimation.state === 2 && row.state.informationAnimation.currentClip === 'Shake');
  const sourceComplete = original.some(row => row.health?.informationAnimation?.state === 0 && row.health.informationAnimation.clip === 'Shake'
    && row.health.informationAnimation.frame === 34 && !row.health.informationAnimation.playing);
  const browserComplete = candidates.some(row => row.state.informationAnimation.state === 0 && row.state.informationAnimation.currentClip === 'Shake'
    && row.state.informationAnimation.frame === 34 && !row.state.informationAnimation.playing);
  const sourceHit = original.some(row => row.death?.mainAnimation?.name === 'Hit' && row.death.animationGuard);
  const browserHit = candidates.some(row => row.state.playerAnimation.current === 'Hit' && row.state.playerAnimation.guard);
  const sourceQueued40 = original.some(row => row.gameplay?.entities?.some(actor => actor.pendingDamageRecords?.some(record => record.amount === 40 && record.flags === 0 && record.sourceType === 0)));
  const browserQueued40 = candidates.some(row => row.state.room.npcs.some(npc => npc.damage.pendingDamage >= 40 && npc.damage.pendingFlags === 0));
  const frozen = rows => {
    const pairs = rows.slice(1).map((row, n) => [rows[n], row]).filter(([a, b]) => a.active && b.active);
    return pairs.length > 1 && pairs.every(([a, b]) => a.age === b.age && JSON.stringify(a.player) === JSON.stringify(b.player));
  };
  const sourceFrozen = frozen(original.map(row => ({active: row.health?.informationAnimation?.state === 2,
    age: row.death?.fullGameUpdates, player: row.gameplay?.players?.[0]?.position})));
  const browserFrozen = frozen(candidates.map(row => ({active: row.state.informationAnimation.state === 2,
    age: row.state.gameUpdateCount, player: [row.state.player.x, row.state.player.y]})));
  const sourceSound = id => original.some(row => row.healthSounds?.slots?.some(slot => slot.id === id && slot.nextAllowed >= 0 && slot.lastVolume === 1));
  const checks = [];
  const require = (field, passed) => checks.push({field, status: passed ? 'pass' : 'fail'});
  if (required) {
    require('actual acquisition in both games', sourceAcquired && browserAcquired);
    require('actual Soul damage preserves red in both games', sourceSoulDamage && browserSoulDamage);
    require('actual guarded Hit in both games', sourceHit && browserHit);
    require('actual SFX55 original admission', sourceSound(55));
    require('actual SFX55 browser output', candidates.some(row => row.browserAudio?.events.some(event => event.id === 55 && event.output === 'started')));
    if (required === 'black') {
      require('actual forced35 temporary record in both games', sourceEffect && browserEffect);
      require('actual Book16 state2 in both games', sourceBook && browserBook);
      require('actual Book16 authored completion in both games', sourceComplete && browserComplete);
      require('actual Book16 freezes Game age and Player in both games', sourceFrozen && browserFrozen);
      require('actual queued40 flags0 targets in both games', sourceQueued40 && browserQueued40);
      require('actual SFX33 original admission', sourceSound(33));
      require('actual SFX33 browser output', candidates.some(row => row.browserAudio?.events.some(event => event.id === 33 && event.output === 'started')));
    }
  }
  return {required, sourceAcquired, browserAcquired, sourceSoulDamage, browserSoulDamage,
    sourceEffect, browserEffect, sourceBook, browserBook, sourceComplete, browserComplete,
    sourceHit, browserHit, sourceQueued40, browserQueued40, sourceFrozen, browserFrozen,
    checks, manifests: {original: source.evidence, browser: browser.evidence},
    status: checks.every(check => check.status === 'pass') ? 'pass' : 'fail'};
}
const result = {schema: 1, status: 'running', scenariosSha256: sha(readFileSync(resolve(output, 'scenarios.json'))),
  cases: [], scope: 'Normal health containers/Hit/Book16, source NPC membership/HP/pending queue and actual SFX wrappers/browser audio owner. Functional fields and contrived execution are blocking; full pixels are retained diagnostics. Wrapper/backend identities are source observations, not a complete original output-PCM parity claim. No native state supplies runtime.'};
save(resolve(output, 'report.json'), result);
for (const [index, definition] of scenarios.cases.entries()) {
  const root = resolve(output, `case-${index}`), replay = resolve(output, `replay-${index}.json`);
  save(replay, definition);
  const row = {id: definition.scenario.id, seed: definition.seed, required: mandatory.get(definition.scenario.id)?.healthRequired ?? null, status: 'fail'};
  try {
    // Isolate each invocation so a case-level storage/transport abort does not
    // hide the remaining predeclared OS/contrived attempts.
    const paired = await runPairedGate({output: root, selected: [], randomCount: 0, replayCases: [replay],
      productionDirectory: option('--production-directory', undefined), originalObserver: 'tools/original/health-damage-observe.py',
      stateComparator: compare, archiveBrowserEvidence: true, captureBrowserAudio: true,
      // The browser's public SFX preference defaults to 1; declare the same
      // option to the original so SFX.globalVolume compares like with like.
      originalSfxVolume: 1});
    row.pairedStatus = paired.status; row.build = paired.build;
    row.coverage = coverage(definition, paired.cases[0].directory);
    row.directory = paired.cases[0].directory;
    row.status = paired.status === 'pass' && row.coverage.status === 'pass' ? 'pass' : 'fail';
  } catch (error) {row.error = String(error.stack || error); row.scope = 'Failed/unattempted captures remain required failures; complete available evidence retained';}
  result.cases.push(row); save(resolve(output, 'report.json'), result);
}
result.productionStable = result.cases.every(row => row.build && row.build.sha256 === result.cases[0]?.build?.sha256);
result.status = result.productionStable && result.cases.length === scenarios.cases.length
  && result.cases.every(row => row.status === 'pass') ? 'pass' : 'fail';
save(resolve(output, 'report.json'), result);
console.log(JSON.stringify({status: result.status, cases: result.cases.length, report: resolve(output, 'report.json')}));
process.exitCode = result.status === 'pass' ? 0 : 1;
