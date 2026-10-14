// Direct source actor fields from a fresh original/public-input production pair.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const option = (name, fallback) => {
  const at = process.argv.indexOf(name);
  if (at < 0 && fallback !== undefined) return fallback;
  assert.ok(at >= 0 && process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
const directory = resolve(option('--case'));
const required = option('--require', 'admission').split(',');
assert.ok(required.every(value => ['admission', 'contact', 'bomb', 'tear', 'clear', 'clear-delay', 'door-clear'].includes(value)));
const load = path => JSON.parse(readFileSync(path, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const originalPath = resolve(directory, 'original/manifest.json');
const browserPath = resolve(directory, 'browser/manifest.json');
const native = load(originalPath), browser = load(browserPath);
const gate = load(resolve(dirname(directory), 'report.json'));
const pair = load(resolve(directory, 'report.json'));
const replay = load(resolve(directory, 'case.json'));
assert.equal(pair.original.status, 'pass', 'Fresh original provenance must pass');
assert.equal(pair.browser.status, 'pass', 'Actual production DOM/WASM capture must pass');
assert.equal(browser.status, 'pass');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
const labels = replay.pairedTape.filter(action => action.type === 'capture').map(action => action.label);
const gameplayStart = labels.indexOf('gameplay-start');
assert.ok(gameplayStart >= 0);
assert.equal(observations.size, native.debugObservations.length);
const checks = [], diagnostics = [], unsupportedDiagnostics = [];
const check = (list, label, field, expected, actual) => {
  assert.ok(expected !== undefined && actual !== undefined, `Missing ${field} at ${label}`);
  list.push({ label, field, expected, actual, status: JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
};
const f32 = value => Math.fround(value);
const vector = value => Array.isArray(value) ? value.map(f32) : [f32(value.x), f32(value.y)];
const supported = actor => [13, 18].includes(actor.type) && actor.variant === 0 && actor.subtype === 0;
const execution = { admission: false, contact: false, bomb: false, tear: false, drain: false, death: false, retirement: false, clear: false };
const queued = new Set(), dead = new Set();
let hadLockedRoom = false;
const clearPoints = { native: [], browser: [] };
for (const [index, label] of labels.entries()) {
  const observation = observations.get(label), candidate = frames.get(label);
  assert.ok(candidate && observation, `Missing selected checkpoint ${label}`);
  assert.equal(observation.status, 'pass', `Source observation failed at ${label}`);
  assert.equal(observation.memoryWritten, false);
  const source = observation.gameplay;
  if (index < gameplayStart) {
    assert.notEqual(candidate.state.screen, 'playing');
    assert.equal(source?.status, 'skip');
    assert.equal(observation.menu?.status, 'pass');
    continue;
  }
  assert.equal(candidate.state.screen, 'playing');
  assert.equal(source?.status, 'pass', `Every selected gameplay observation must pass at ${label}`);
  assert.equal(source.memoryWritten, false);
  assert.equal(source.executableMappingsRead, false);
  assert.equal(source.nativeBinarySha256, gate.sourceBinarySha256);
  const actual = candidate.state;
  const natives = source.entities.filter(supported), port = actual.room.npcs;
  assert.ok(Array.isArray(port));
  check(checks, label, 'npc.spawnSeedOrder', natives.map(actor => actor.spawnSeed), port.map(actor => actor.spawnSeed));
  const byId = new Map(port.map(actor => [actor.id, actor]));
  const sourcePickup = actor => actor.type === 5 && actor.variant === 20 && actor.subtype >= 1 && actor.subtype <= 4
    && actual.room.descriptor?.template.roomType === 1;
  const pickupsById = new Map(actual.room.pickups.map(pickup => [pickup.id, pickup]));
  check(checks, label, 'manager.supportedActorOrder', source.entities.filter(actor => supported(actor) || sourcePickup(actor) || actor.type === 1)
    .map(actor => actor.type === 1 ? 'player' : actor.spawnSeed), actual.actorScheduler.actors
    .filter(actor => ['npc', 'player', 'pickup'].includes(actor.kind)).map(actor => actor.kind === 'player' ? 'player'
      : (actor.kind === 'npc' ? byId : pickupsById).get(actor.id)?.spawnSeed));
  check(checks, label, 'system.phase', source.nativePhase.systemIterations % 2, actual.systemClock.systemCounter % 2);
  check(diagnostics, label, 'game.fullUpdates', source.nativePhase.fullGameUpdates, actual.gameUpdateCount);
  if (natives.length && port.length) execution.admission = true;
  for (const native of natives) {
    const actor = port.find(actor => actor.spawnSeed === native.spawnSeed);
    if (!actor) continue; // Missing actor remains a required seed-set failure.
    for (const [field, expected, observed] of [
      ['kind', native.type, actor.kind], ['variant', native.variant, actor.variant], ['subtype', native.subtype, actor.subtype],
      ['hp', f32(native.hp), f32(actor.hp)], ['maxHp', f32(native.maxHp), f32(actor.maxHp)],
      ['pendingDamage', f32(native.pendingDamage), f32(actor.damage.pendingDamage)],
      ['pendingFlags', native.pendingFlags, actor.damage.pendingFlags],
      ['orderedDamageRecords', native.pendingDamageRecords.map(record => [f32(record.amount), record.flags, record.sourceType]),
        actor.damage.events.map(event => [f32(event.amount), { tear: 0, bomb: 4, roomEffect: 0 }[event.kind], event.source.entityType])],
      ['dead', native.removed, actor.dead], ['retired', false, actor.removed],
      ['collisionRadius', f32(native.collisionRadius), f32(actor.collisionRadius)],
      ['collisionMass', f32(native.collisionMass), f32(actor.collisionMass)],
      ['collisionDamage', f32(native.collisionDamage), f32(actor.collisionDamage)],
      ['gridCollisionClass', native.gridCollisionClass, actor.gridCollisionClass],
      ['entityCollisionClass', native.entityCollisionClass, actor.entityCollisionClass],
      ['canShutDoors', native.canShutDoors, actor.canShutDoors], ['boss', native.boss, actor.boss],
      ['state', native.npcState, actor.state], ['frameAge', native.npcFrameAge, actor.frameAge],
    ]) check(checks, label, `npc${native.spawnSeed}.${field}`, expected, observed);
    for (const [index, record] of native.pendingDamageRecords.entries()) {
      const event = actor.damage.events[index];
      if (!event) continue;
      if (event.kind === 'roomEffect') {
        // Black-heart Use35 passes a real blank source, not an actor reference.
        check(checks, label, `npc${native.spawnSeed}.damage${index}.nativeBlankSource`, [0, 0, null, null, null],
          [record.sourceType, record.sourceVariant, record.sourceActorType, record.sourceActorSequenceId, record.sourceActorSpawnSeed]);
        check(checks, label, `npc${native.spawnSeed}.damage${index}.browserBlankSource`, [0, 0],
          [event.source.entityType, event.source.id]);
        continue;
      }
      assert.equal(record.sourceActorType, record.sourceType, 'Native pending record must join its actual retained attacker');
      const retained = event.source.entityType === 2 ? actual.tears.find(tear => tear.id === event.source.id)
        : actual.bombs.find(bomb => bomb.id === event.source.id);
      assert.ok(retained, 'Port pending record must reference an actually admitted public Tear/Bomb');
      const field = `npc${native.spawnSeed}.damage${index}.sourceSpawnSeed`;
      if (event.source.entityType === 4) {
        unsupportedDiagnostics.push({ label, field, source: record.sourceActorSpawnSeed,
          status: 'unaccepted', reason: 'Regular Bomb seed production is not yet admitted to the runtime. Ordered damage flags, amount and actual attacker references remain required.' });
      } else {
        check(diagnostics, label, field, record.sourceActorSpawnSeed, retained.spawnSeed);
      }
    }
    // Serialized f32 JSON uses a shortest decimal. fround restores exact bits;
    // this is an exact comparison and introduces no epsilon.
    for (const [field, expected, observed] of [
      ['position', vector(native.position), vector(source.nativePhase.systemIterations % 2 === 0 ? actor.presentedPosition : actor.position)], ['velocity', vector(native.velocity), vector(actor.velocity)],
      ['target', vector(native.npcTarget), vector(actor.target)], ['wanderVelocity', vector(native.npcWanderVelocity), vector(actor.wanderVelocity)],
    ]) check(diagnostics, label, `npc${native.spawnSeed}.${field}`, expected, observed);
    if (native.pendingDamage > 0 && native.pendingDamage === actor.damage.pendingDamage) {
      queued.add(native.spawnSeed);
      if (native.pendingDamage === 60) execution.bomb = true;
      if (native.pendingDamage === 3.5) execution.tear = true;
    }
    if (queued.has(native.spawnSeed) && native.pendingDamage === 0 && actor.damage.pendingDamage === 0 && native.hp < 0 && actor.hp === native.hp) execution.drain = true;
    if (native.removed && actor.dead) { dead.add(native.spawnSeed); execution.death = true; }
  }
  for (const seed of dead) if (!natives.some(actor => actor.spawnSeed === seed) && !port.some(actor => actor.spawnSeed === seed)) execution.retirement = true;
  const player = source.players[0];
  assert.ok(player.lastDamageSource, 'Source player must expose the retained attacker identity');
  if (player.damageCountdown === 60 && actual.damageCooldown === 60 && player.lastDamageSource.type === 18
      && player.lastDamageSource.flags === 0 && natives.some(actor => actor.spawnSeed === player.lastDamageSource.activeActorSpawnSeed)) execution.contact = true;
  assert.equal(typeof source.room.combatClear, 'boolean', 'Every gameplay capture requires direct native descriptor clear');
  assert.ok(Number.isInteger(source.room.combatClearCountdown) && source.room.combatClearCountdown >= 0);
  assert.ok(Number.isInteger(source.room.countableCurrent) && Number.isInteger(source.room.countablePending));
  check(checks, label, 'manager.countablePolicy', source.room.countableCurrent + source.room.countablePending > 0,
    port.some(actor => actor.canShutDoors && !actor.removed));
  {
    // Descriptor+40 is a separate surprise-miniboss substitution flag.
    // Ordinary NPC countability does not write it.
    assert.equal(typeof source.room.descriptorFlag40, 'boolean');
    check(checks, label, 'room.combatClear', source.room.combatClear, actual.room.combatClear);
    check(checks, label, 'room.combatClearCountdown', source.room.combatClearCountdown, actual.room.combatClearCountdown);
    if (source.nativePhase.systemIterations % 2 === 1) {
      clearPoints.native.push({ label, counter: source.nativePhase.systemIterations, count: source.room.countableCurrent + source.room.countablePending, countdown: source.room.combatClearCountdown, clear: source.room.combatClear, doors: source.grid.filter(cell => cell.type === 16).map(cell => ({index: cell.index, state: cell.state, collisionClass: cell.collisionClass})) });
      clearPoints.browser.push({ label, counter: actual.systemClock.systemCounter, count: port.filter(actor => actor.canShutDoors && !actor.removed).length, countdown: actual.room.combatClearCountdown, clear: actual.room.combatClear, doors: actual.room.grid.cells.flatMap((cell, index) => cell.entity?.kind === 16 ? [{ index, state: cell.entity.state, collisionClass: cell.entity.collisionClass }] : []) });
    }
    hadLockedRoom ||= source.room.combatClear === false;
    if (hadLockedRoom && source.room.combatClear && actual.room.combatClear && !natives.length && !port.length) execution.clear = true;
    for (const door of source.grid.filter(cell => cell.type === 16)) {
      const admitted = actual.room.grid.cells[door.index]?.entity;
      assert.ok(admitted, `Missing admitted door ${door.index}`);
      check(diagnostics, label, `door${door.index}.state`, door.state, admitted.state);
      check(diagnostics, label, `door${door.index}.collisionClass`, door.collisionClass, admitted.collisionClass);
    }
  }
}
const clearDelayContract = {};
for (const [game, points] of Object.entries(clearPoints)) {
  const start = points.findIndex((point, index) => point.count === 0 && !point.clear && point.countdown === 9
    && points.slice(index, index + 11).length === 11
    && points.slice(index, index + 11).every((row, offset) => row.count === 0 && row.counter === point.counter + offset * 2
      && row.countdown === Math.max(9 - offset, 0) && row.clear === (offset === 10)));
  const before = points[start + 9], cleared = points[start + 10], following = points[start + 11];
  const doorTransitions = start < 0 || !following || following.counter !== cleared.counter + 2 ? []
    : cleared.doors.filter(door => door.state === 2 && before.doors.some(prior => prior.index === door.index && prior.state === 1))
      .map(door => ({ index: door.index, before: before.doors.find(prior => prior.index === door.index), cleared: door,
        following: following.doors.find(next => next.index === door.index) }));
  const doorStatus = doorTransitions.length && doorTransitions.every(door => door.before.collisionClass === 4
    && door.cleared.collisionClass === 4 && door.following?.state === 2 && door.following?.collisionClass === 5) ? 'pass' : 'fail';
  clearDelayContract[game] = { status: start >= 0 ? 'pass' : 'fail', boundary: start >= 0 ? points.slice(start, start + 11) : [], doorStatus, doorTransitions };
}
execution['clear-delay'] = Object.values(clearDelayContract).every(contract => contract.status === 'pass');
execution['door-clear'] = Object.values(clearDelayContract).every(contract => contract.doorStatus === 'pass')
  && JSON.stringify(clearDelayContract.native.doorTransitions.map(door => door.index))
    === JSON.stringify(clearDelayContract.browser.doorTransitions.map(door => door.index));
for (const name of required) {
  check(checks, 'execution', name, true, execution[name]);
  if (['bomb', 'tear'].includes(name)) for (const event of ['drain', 'death', 'retirement']) check(checks, 'execution', event, true, execution[event]);
}
const report = { schema: 1, status: checks.every(check => check.status === 'pass') ? 'pass' : 'fail',
  diagnosticStatus: diagnostics.every(check => check.status === 'pass') ? 'pass' : 'fail',
  scope: 'Direct supported Fly13/AttackFly18 admission, configuration, source NPC/Player actor order, appearance state, queued damage, HP, dead flag, retirement and observed room clear/countdown. Native +31f is dead; absence from manager is retirement. Native world position is authoritative on full phases and presented half-motion on intermediate phases. Actual f32 trajectories/counters and door lifecycle remain separately recorded exact diagnostics.',
  required, execution, clearDelayContract, unsupportedDiagnostics, buildSha256: gate.build.sha256, sourceBinarySha256: gate.sourceBinarySha256,
  comparatorSha256: hash('tools/foundation/npc-state-pair.mjs'),
  originalManifestSha256: hash(originalPath), browserManifestSha256: hash(browserPath), caseSha256: hash(resolve(directory, 'case.json')), checks, diagnostics };
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, fields: checks.length, failures: checks.filter(check => check.status === 'fail'), execution,
  diagnosticFields: diagnostics.length, diagnosticFailures: diagnostics.filter(check => check.status === 'fail').length }));
process.exitCode = report.status === 'pass' ? 0 : 1;
