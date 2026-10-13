// Actual33 factory/damage fields from both public-input games; no native state enters WASM.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const option = name => {
  const at = process.argv.indexOf(name);
  assert.ok(at >= 0 && process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
const directory = resolve(option('--case'));
const load = path => JSON.parse(readFileSync(path, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const originalPath = resolve(directory, 'original/manifest.json'), browserPath = resolve(directory, 'browser/manifest.json');
const original = load(originalPath), browser = load(browserPath), replay = load(resolve(directory, 'case.json'));
const gate = load(resolve(dirname(directory), 'report.json')), pair = load(resolve(directory, 'report.json'));
assert.equal(pair.original.status, 'pass');
assert.equal(pair.browser.status, 'pass');
assert.equal(original.status, 'pass');
assert.equal(browser.status, 'pass');
assert.equal(gate.gateSourcesStable, true);
assert.match(gate.build?.sha256 ?? '', /^[a-f0-9]{64}$/, 'Paired capture must identify its actual production artifact');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const observations = new Map(original.debugObservations.map(row => [row.context.label, row.observation]));
assert.equal(frames.size, browser.frames.length, 'Browser capture labels must be unique');
assert.equal(observations.size, original.debugObservations.length, 'Native capture labels must be unique');
const f32 = value => Math.fround(value);
const vector = point => [f32(point.x ?? point[0]), f32(point.y ?? point[1])];
const checks = [], diagnostics = [], bodies = [];
const check = (rows, label, field, expected, actual) => {
  assert.notEqual(expected, undefined, `${label}:${field}`);
  assert.notEqual(actual, undefined, `${label}:${field}`);
  rows.push({ label, field, expected, actual, status: JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
};
const modes = ['admission', 'tear-queue', 'tear-drain', 'bomb-queue', 'extinguish', 'empty-persistence', 'ember-birth', 'ember-first-full', 'ember-retirement'];
assert.ok(replay.fireRequired.every(name => modes.includes(name)));
const execution = Object.fromEntries(['native', 'browser'].map(game => [game, Object.fromEntries(modes.map(name => [name, false]))]));
const prior = { native: new Map(), browser: new Map() };
const priorEmbers = { native: new Map(), browser: new Map() };
const emberBirths = { native: new Map(), browser: new Map() };
const parentOrdinals = { native: new Map(), browser: new Map() };
const previousWorldCount = { native: undefined, browser: undefined };
const emberCoverage = Object.fromEntries(['native', 'browser'].map(game => [game,
  { fastFreeze: 0, fastMotion: 0, fullMotion: 0, birthFence: 0, retirement: 0 }]));
let presentation = 0, previousCapturePresentation, previousRoom, segment = 0;
const add = (a, b) => a.map((value, axis) => f32(value + b[axis]));
const scale = (point, amount) => point.map(value => f32(value * amount));
let gameplayStarted = false;
assert.ok(replay.pairedTape.some(action => action.type === 'capture' && action.label === 'gameplay-start'));
for (const action of replay.pairedTape) {
  if (action.type === 'step') presentation += action.frames;
  if (action.type !== 'capture') continue;
  const label = action.label;
  if (label === 'gameplay-start') gameplayStarted = true;
  const observation = observations.get(label), actual = frames.get(label)?.state;
  assert.ok(observation && actual, `Missing declared capture ${label}`);
  assert.equal(observation.status, 'pass');
  assert.equal(observation.memoryWritten, false);
  const source = observation.gameplay;
  if (!gameplayStarted) {
    assert.notEqual(actual.screen, 'playing');
    assert.equal(source?.status, 'skip');
    continue;
  }
  assert.equal(actual.screen, 'playing', 'No post-start menu fallback may skip a required gameplay read');
  assert.equal(source?.status, 'pass');
  assert.equal(source.memoryWritten, false);
  assert.equal(source.executableMappingsRead, false);
  assert.equal(source.nativeBinarySha256, gate.sourceBinarySha256);
  const native = source.entities.filter(actor => actor.type === 33 && actor.variant === 0);
  const port = actual.room.npcs.filter(actor => actor.kind === 33 && actor.variant === 0);
  const phase = source.nativePhase.systemIterations % 2;
  const room = source.room.descriptorIndex;
  const dense = previousCapturePresentation !== undefined && presentation === previousCapturePresentation + 1 && room === previousRoom;
  if (!dense) segment += 1;
  check(checks, label, 'fire.spawnSeedOrder', native.map(actor => actor.spawnSeed), port.map(actor => actor.spawnSeed));
  check(checks, label, 'room.identity', source.room.descriptorIndex, actual.room.descriptor?.index ?? actual.currentRoom);
  check(checks, label, 'system.phase', phase, actual.systemClock.systemCounter % 2);
  check(diagnostics, label, 'game.fullUpdates', source.nativePhase.fullGameUpdates, actual.gameUpdateCount);
  assert.equal(typeof source.room.combatClear, 'boolean');
  assert.ok(Number.isInteger(source.room.combatClearCountdown));
  assert.ok(Number.isInteger(source.room.countableCurrent) && Number.isInteger(source.room.countablePending));
  check(checks, label, 'room.combatClear', source.room.combatClear, actual.room.combatClear);
  check(checks, label, 'room.combatClearCountdown', source.room.combatClearCountdown, actual.room.combatClearCountdown);
  check(checks, label, 'manager.countablePolicy', source.room.countableCurrent + source.room.countablePending > 0,
    actual.room.npcs.some(actor => actor.canShutDoors && !actor.removed));
  for (const actor of native) {
    const npc = port.find(npc => npc.spawnSeed === actor.spawnSeed);
    if (!npc) continue; // Its missing seed remains a required seed-order failure.
    const fire = npc.fireplace;
    assert.ok(fire, 'Every admitted ordinary33 requires actual Fire subclass state');
    for (const [field, expected, observed] of [
      ['kind', actor.type, npc.kind], ['variant', actor.variant, npc.variant], ['subtype', actor.subtype, npc.subtype],
      ['hp', f32(actor.hp), f32(npc.hp)], ['maxHp', f32(actor.maxHp), f32(npc.maxHp)],
      ['radius', f32(actor.collisionRadius), f32(npc.collisionRadius)], ['mass', f32(actor.collisionMass), f32(npc.collisionMass)],
      ['contactDamage', f32(actor.collisionDamage), f32(npc.collisionDamage)], ['friction', f32(actor.friction), f32(npc.baseFriction)],
      ['gridClass', actor.gridCollisionClass, npc.gridCollisionClass], ['entityClass', actor.entityCollisionClass, npc.entityCollisionClass],
      ['canShutDoors', actor.canShutDoors, npc.canShutDoors], ['boss', actor.boss, npc.boss],
      ['state', actor.npcState, npc.state], ['appearanceCounter', actor.npcFrameAge, npc.frameAge],
      ['savedInitializedState', actor.npcSavedInitializedState, npc.savedInitializedState],
      ['animation', actor.animation, npc.animation], ['animationFrame', f32(actor.animationFrame), f32(npc.animationClock)],
      ['animationPlaying', actor.animationPlaying, npc.animationPlaying], ['visible', actor.visible, npc.visible],
      ['active', actor.active, !npc.removed], ['dead', actor.removed, npc.dead],
      ['position', vector(actor.position), vector(phase === 0 ? npc.presentedPosition : npc.position)],
      ['velocity', vector(actor.velocity), vector(npc.velocity)], ['anchor', vector(actor.npcTarget), vector(fire.anchor)],
      ['entityFlags', actor.entityFlags, fire.entityFlags], ['gridIndex', actor.authoredGridIndex, fire.authoredGridIndex],
      ['renderOrderOffset', actor.renderOrderOffset, fire.renderOrderOffset], ['actorRngState', actor.actorRngState, fire.actorRngState],
      ['countdown', actor.fireCountdown, fire.countdown], ['pendingDamage', f32(actor.pendingDamage), f32(npc.damage.pendingDamage)],
      ['pendingFlags', actor.pendingFlags, npc.damage.pendingFlags],
      ['orderedDamage', actor.pendingDamageRecords.map(record => [f32(record.amount), record.flags, record.sourceType]),
        npc.damage.events.map(event => [f32(event.amount), { tear: 0, bomb: 4, fireContact: 2 }[event.kind], event.source.entityType])],
    ]) check(checks, label, `fire${actor.spawnSeed}.${field}`, expected, observed);
    const index = actor.authoredGridIndex;
    assert.ok(Number.isInteger(index) && index >= 0 && index < source.gridSelectors.length);
    assert.ok(actual.room.grid?.cells[index]);
    check(checks, label, `fire${actor.spawnSeed}.selector`, source.gridSelectors[index], actual.room.grid.cells[index].selector);
    check(checks, label, `fire${actor.spawnSeed}.nullGrid`, source.grid.some(cell => cell.index === index), actual.room.grid.cells[index].entity !== null);
    for (const [at, record] of actor.pendingDamageRecords.entries()) {
      assert.equal(record.sourceActorType, record.sourceType, 'Native ordered damage must join the actual retained attacker');
      const event = npc.damage.events[at];
      if (!event) continue;
      if (record.sourceType === 2) {
        const tear = actual.tears.find(tear => tear.id === event.source.id);
        assert.ok(tear, 'Browser Tear damage must join its actual retained actor');
        check(checks, label, `fire${actor.spawnSeed}.queue${at}.tearSeed`, record.sourceActorSpawnSeed, tear.spawnSeed);
      } else if (record.sourceType === 4) {
        assert.ok(actual.bombs.some(bomb => bomb.id === event.source.id), 'Browser Bomb damage must join its actual retained actor');
        // Regular Bomb does not yet retain its source spawnSeed field. Do not
        // invent one or allow this missing diagnostic to abort the required FIFO.
        diagnostics.push({ label, field: `fire${actor.spawnSeed}.queue${at}.bombSeed`, expected: record.sourceActorSpawnSeed,
          status: 'unsupported', reason: 'Current regular Bomb struct has no source spawnSeed field' });
      }
    }
  }
  const games = {
    native: { count: source.nativePhase.fullGameUpdates, actors: native.map(actor => ({ seed: actor.spawnSeed, hp: actor.hp, maxHp: actor.maxHp,
      state: actor.npcState, gridClass: actor.gridCollisionClass, entityClass: actor.entityCollisionClass,
      dead: actor.removed, removed: !actor.active, visible: actor.visible, records: actor.pendingDamageRecords,
      animation: actor.animation, canShutDoors: actor.canShutDoors })) },
    browser: { count: actual.gameUpdateCount, actors: port.map(actor => ({ seed: actor.spawnSeed, hp: actor.hp, maxHp: actor.maxHp,
      state: actor.state, gridClass: actor.gridCollisionClass, entityClass: actor.entityCollisionClass,
      dead: actor.dead, removed: actor.removed, visible: actor.visible,
      records: actor.damage.events.map(event => ({ amount: event.amount, flags: { tear: 0, bomb: 4, fireContact: 2 }[event.kind], sourceType: event.source.entityType })),
      animation: actor.animation, canShutDoors: actor.canShutDoors })) },
  };
  for (const [game, world] of Object.entries(games)) {
    if (world.actors.length) execution[game].admission = true;
    for (const actor of world.actors) {
      check(checks, label, `${game}.fire${actor.seed}.minimumHp`, true, actor.hp >= 1 && actor.hp <= actor.maxHp);
      check(checks, label, `${game}.fire${actor.seed}.noncountable`, false, actor.canShutDoors);
      if (actor.records.some(record => record.sourceType === 2 && record.flags === 0 && f32(record.amount) === f32(3.5))) execution[game]['tear-queue'] = true;
      if (actor.records.some(record => record.sourceType === 4 && (record.flags & 4) !== 0)) execution[game]['bomb-queue'] = true;
      const previous = prior[game].get(actor.seed);
      if (previous && world.count > previous.count) {
        if (previous.actor.records.some(record => record.sourceType === 2) && !actor.records.length && actor.hp < previous.actor.hp) execution[game]['tear-drain'] = true;
        if (previous.actor.state === 8 && previous.actor.records.length && actor.state === 3) execution[game].extinguish = true;
        if (previous.actor.state === 3 && actor.state === 3 && actor.hp === 1 && !actor.dead && !actor.removed && actor.visible) execution[game]['empty-persistence'] = true;
      }
      if (actor.state === 3) {
        for (const [field, expected, observed] of [['hp', 1, actor.hp], ['gridClass', 0, actor.gridClass], ['entityClass', 0, actor.entityClass],
          ['dead', false, actor.dead], ['active', true, !actor.removed], ['visible', true, actor.visible]])
          check(checks, label, `${game}.empty${actor.seed}.${field}`, expected, observed);
      }
      prior[game].set(actor.seed, { count: world.count, actor });
    }
  }
  const player = source.players[0];
  assert.ok(player);
  check(checks, label, 'player.redHp', player.redHp, actual.player.hp);
  check(checks, label, 'player.immunity', player.damageCountdown, actual.damageCooldown);
  const nativeEmbers = source.entities.filter(actor => actor.type === 1000 && actor.variant === 66);
  const browserEmbers = actual.fireParticles;
  assert.ok(Array.isArray(browserEmbers));
  check(checks, label, 'ember.activeCount', nativeEmbers.filter(actor => actor.active).length, browserEmbers.filter(actor => !actor.removed).length);
  check(diagnostics, label, 'ember.seedOrder', nativeEmbers.map(actor => actor.spawnSeed), browserEmbers.map(actor => actor.spawnSeed));
  const emberGames = {
    native: { count: source.nativePhase.fullGameUpdates, paused: observation.pause.paused,
      actors: nativeEmbers.map(actor => ({ id: `${actor.sequenceId}:${actor.spawnSeed}:${actor.effectBirthGameUpdate}`, seed: actor.spawnSeed,
      birthCount: actor.effectBirthGameUpdate,
      position: actor.position, velocity: actor.velocity, divisor: actor.effectLifetimeDivisor, ttl: actor.effectTimeToLive,
      active: actor.active, dead: actor.removed, animation: actor.animation, frame: actor.animationFrame,
      playing: actor.animationPlaying, parentType: actor.parentType, parentVariant: actor.parentVariant,
      hp: actor.hp, maxHp: actor.maxHp, radius: actor.collisionRadius, mass: actor.collisionMass, damage: actor.collisionDamage,
      friction: actor.friction, gridClass: actor.gridCollisionClass, entityClass: actor.entityCollisionClass })) },
    browser: { count: actual.gameUpdateCount, paused: actual.paused,
      actors: browserEmbers.map(actor => ({ id: `${actor.id}:${actor.spawnSeed}:${actor.birthGameUpdate}`, seed: actor.spawnSeed,
      birthCount: actor.birthGameUpdate,
      position: vector(phase === 0 ? actor.presentedPosition : actor.position), velocity: vector(actor.velocity),
      divisor: actor.lifetimeDivisor, ttl: actor.timeToLive, active: !actor.removed, dead: actor.dead,
      animation: 'Idle', frame: actor.animationClock, playing: actor.animationPlaying, parentType: actor.parentType, parentVariant: actor.parentVariant,
      hp: actor.hp, maxHp: actor.maxHp, radius: actor.collisionRadius, mass: actor.collisionMass, damage: actor.collisionDamage,
      friction: actor.baseFriction, gridClass: actor.gridCollisionClass, entityClass: actor.entityCollisionClass })) },
  };
  const currentLinked = { native: new Map(), browser: new Map() };
  for (const [game, world] of Object.entries(emberGames)) {
    const present = new Set(world.actors.map(actor => actor.id));
    for (const [id, previous] of priorEmbers[game]) {
      if (!present.has(id)) {
        if (dense && world.count === previous.count + 1) {
          check(checks, label, `${game}.ember${id}.retireOnlyAfterDead`, true, previous.actor.dead && previous.actor.active);
          check(checks, label, `${game}.ember${id}.retireAtZeroTtl`, 0, previous.actor.ttl);
          if (previous.actor.dead && previous.actor.active) execution[game]['ember-retirement'] = true;
          emberCoverage[game].retirement += 1;
          if (previous.link) emberBirths[game].get(previous.link).retirementPresentation = presentation;
        }
        priorEmbers[game].delete(id);
      }
    }
    for (const actor of world.actors) {
      for (const [field, expected, observed] of [['hp', 0, actor.hp], ['maxHp', 0, actor.maxHp], ['radius', 0, actor.radius],
        ['mass', 0, actor.mass], ['collisionDamage', 22, actor.damage], ['friction', 1, actor.friction],
        ['gridClass', 0, actor.gridClass], ['entityClass', 0, actor.entityClass], ['parentType', 33, actor.parentType],
        ['parentVariant', 0, actor.parentVariant], ['animation', 'Idle', actor.animation]])
        check(checks, label, `${game}.ember${actor.seed}.${field}`, expected, observed);
      const previous = priorEmbers[game].get(actor.id);
      let link = previous?.link;
      if (!previous && actor.divisor === 0 && actor.birthCount === world.count) {
        //550a40 retains only parent type/variant at+1c/+20, not a Fire
        //pointer/seed. Join the actual parent tag and observed append ordinal
        //within this contiguous capture segment; never invent a parent seed.
        const parent = `${room}:${segment}:${actor.parentType}:${actor.parentVariant}`;
        const ordinal = parentOrdinals[game].get(parent) || 0;
        parentOrdinals[game].set(parent, ordinal + 1);
        link = `${parent}:${ordinal}`;
        emberBirths[game].set(link, { label, parent: [actor.parentType, actor.parentVariant], ordinal,
          birthPresentation: presentation, birthCount: world.count, seed: actor.seed });
      }
      if (actor.divisor === 0) {
        execution[game]['ember-birth'] = true;
        check(checks, label, `${game}.ember${actor.seed}.birthTtl`, -1, actor.ttl);
        check(checks, label, `${game}.ember${actor.seed}.birthDead`, false, actor.dead);
        check(checks, label, `${game}.ember${actor.seed}.birthFrame`, 0, actor.frame);
        check(checks, label, `${game}.ember${actor.seed}.birthPlaying`, true, actor.playing);
      }
      if (actor.birthCount === world.count) {
        emberCoverage[game].birthFence += 1;
        check(checks, label, `${game}.ember${actor.id}.fullBirthFence`, 0, actor.divisor);
        check(checks, label, `${game}.ember${actor.id}.unadvancedBirthTtl`, -1, actor.ttl);
      }
      if (previous && previous.actor.divisor === 0 && actor.divisor > 0 && world.count === previous.count + 1) {
        execution[game]['ember-first-full'] = true;
        check(checks, label, `${game}.ember${actor.seed}.lifetimeRange`, true, actor.divisor >= 2 && actor.divisor <= 31);
        check(checks, label, `${game}.ember${actor.seed}.firstFullTtl`, actor.divisor - 2, actor.ttl);
        check(checks, label, `${game}.ember${actor.id}.firstFullCounterFence`, actor.birthCount + 1, world.count);
        check(checks, label, `${game}.ember${actor.id}.firstFullPlaying`, true, actor.playing);
        check(checks, label, `${game}.ember${actor.id}.firstFullFrame`, Math.min(8, Math.trunc(f32(9 * f32((actor.ttl + 1) / actor.divisor)))), actor.frame);
        check(checks, label, `${game}.ember${actor.id}.firstFullDead`, actor.ttl === 0, actor.dead);
        if (link) Object.assign(emberBirths[game].get(link), { firstFullPresentation: presentation, divisor: actor.divisor, initialTtl: actor.ttl });
      }
      if (previous && previous.actor.divisor > 0 && world.count === previous.count + 1) {
        check(checks, label, `${game}.ember${actor.seed}.divisorRetained`, previous.actor.divisor, actor.divisor);
        check(checks, label, `${game}.ember${actor.seed}.ttlDecrement`, Math.max(0, previous.actor.ttl - 1), actor.ttl);
        check(checks, label, `${game}.ember${actor.seed}.manualFrame`, Math.min(8, Math.trunc(f32(9 * f32(previous.actor.ttl / actor.divisor)))), actor.frame);
        check(checks, label, `${game}.ember${actor.seed}.deathFence`, actor.ttl === 0, actor.dead);
      }
      if (previous && dense && world.count === previous.count) {
        emberCoverage[game].fastFreeze += 1;
        for (const field of ['ttl', 'divisor', 'frame', 'playing', 'dead', 'active', 'velocity'])
          check(checks, label, `${game}.ember${actor.id}.fastFrozen.${field}`, previous.actor[field], actor[field]);
      }
      //An odd system presentation can still be a frozen pause-clear or room
      //transition phase. Only an observed full Game pass or an unadvanced
      //full-phase birth establishes ground; a frozen predicted pose does not.
      let ground = previous?.ground;
      if (phase === 1 && !world.paused &&
        (world.count > previousWorldCount[game] || (!previous && actor.divisor === 0))) ground = actor.position;
      const activeFull = previous && dense && !world.paused && phase === 1 && world.count === previous.count + 1;
      if (activeFull && previous.ground && previous.actor.active) {
        emberCoverage[game].fullMotion += 1;
        check(checks, label, `${game}.ember${actor.id}.fullMotion`, add(previous.ground, scale(previous.actor.velocity, previous.actor.friction)), actor.position);
        const basePlaying = previous.actor.playing && f32(previous.actor.frame + 1) < 9;
        check(checks, label, `${game}.ember${actor.id}.basePlayingThenManualFrame`, basePlaying, actor.playing);
        if (previous.actor.dead) check(checks, label, `${game}.ember${actor.id}.nextFullRemoved`, false, actor.active);
      }
      //Native transition admission can update NPCs at a frozen Game counter.
      //A prior observed active full pass establishes the fast motion law;
      //birth/TTL/frame freezes above remain required through transition waits.
      const activeFullSeen = previous?.activeFullSeen || activeFull;
      if (previous && dense && phase === 0 && world.count === previous.count && !world.paused && activeFullSeen && previous.actor.active) {
        if (!actual.roomTransition && previous.ground) {
          emberCoverage[game].fastMotion += 1;
          check(checks, label, `${game}.ember${actor.id}.fastMotion`, add(previous.ground, scale(scale(actor.velocity, actor.friction), .5)), actor.position);
        }
      }
      if (previous && dense && world.count === previous.count + 1 && !previous.actor.dead && actor.dead && link)
        emberBirths[game].get(link).deathPresentation = presentation;
      if (!actor.active && link) emberBirths[game].get(link).retirementPresentation ??= presentation;
      if (link) currentLinked[game].set(link, actor);
      priorEmbers[game].set(actor.id, { count: world.count, actor, ground, activeFullSeen, link });
    }
    previousWorldCount[game] = world.count;
  }
  for (const link of new Set([...currentLinked.native.keys(), ...currentLinked.browser.keys()])) {
    const native = currentLinked.native.get(link), browser = currentLinked.browser.get(link);
    check(checks, label, `${link}.pairedPresence`, Boolean(native), Boolean(browser));
    if (!native || !browser) continue;
    for (const field of ['ttl', 'divisor', 'frame', 'playing', 'dead', 'active', 'parentType', 'parentVariant'])
      check(checks, label, `${link}.paired.${field}`, native[field], browser[field]);
    for (const field of ['position', 'velocity'])
      check(native.seed === browser.seed ? checks : diagnostics, label, `${link}.paired.${field}`, native[field], browser[field]);
  }
  diagnostics.push({ label, field: 'fire.lootActorAdmission', status: 'unsupported',
    browserProducers: port.filter(actor => actor.fireplace?.lootBirth).map(actor => actor.fireplace.lootBirth),
    reason: 'Fresh local35 coin/heart admission is implemented; no live loot fixture or heart collection/modifier acceptance is claimed' });
  bodies.push({ label, native: source, browser: { fires: port, embers: browserEmbers, player: actual.player, immunity: actual.damageCooldown, room: actual.room, audio: actual.audio } });
  previousCapturePresentation = presentation;
  previousRoom = room;
}
const linkedBirths = new Set([...emberBirths.native.keys(), ...emberBirths.browser.keys()]);
let completePairedLifetimes = 0;
for (const link of linkedBirths) {
  const native = emberBirths.native.get(link), browser = emberBirths.browser.get(link);
  check(checks, 'ember-birth-links', `${link}.birthWitness`, Boolean(native), Boolean(browser));
  if (!native || !browser) continue;
  for (const field of ['parent', 'birthPresentation', 'firstFullPresentation', 'divisor', 'initialTtl', 'deathPresentation', 'retirementPresentation'])
    check(checks, 'ember-birth-links', `${link}.${field}`, native[field] ?? null, browser[field] ?? null);
  check(diagnostics, 'ember-birth-links', `${link}.seed`, native.seed, browser.seed);
  if (['birthPresentation', 'firstFullPresentation', 'divisor', 'initialTtl', 'deathPresentation', 'retirementPresentation']
    .every(field => Number.isInteger(native[field]) && Number.isInteger(browser[field]))) completePairedLifetimes += 1;
}
if (replay.fireRequired.includes('ember-birth')) {
  for (const game of ['native', 'browser']) for (const [field, count] of Object.entries(emberCoverage[game]))
    check(checks, 'ember-coverage', `${game}.${field}`, true, count > 0);
  check(checks, 'ember-coverage', 'pairedCompleteLifetime', true, completePairedLifetimes > 0);
}
for (const game of ['native', 'browser']) for (const required of replay.fireRequired)
  check(checks, 'execution', `${game}.${required}`, true, execution[game][required]);
const report = { schema: 1, status: checks.every(row => row.status === 'pass') ? 'pass' : 'fail',
  scope: 'Ordinary33 raw1400 factory, actual ordered Tear/Bomb FIFO, subclass extinguish and Effect66 lifetime are required. Exact shared stream/particle seeds, live loot/heart collection/modifiers, blend/depth ordering and saved-room reconstruction remain explicit incomplete contracts.',
  provenance: { originalManifest: originalPath, originalSha256: hash(originalPath), browserManifest: browserPath, browserSha256: hash(browserPath),
    originalBinarySha256: gate.sourceBinarySha256, browserArtifactSha256: gate.build.sha256, comparatorSha256: hash(new URL(import.meta.url)), gateSourcesStable: true },
  parentJoinScope: 'Source GameSpawn retains type/variant parent identity only, not actual Fire pointer/seed. Paired links use those real tags plus observed append ordinal within each contiguous capture segment; gaps start a new segment. No spatial/captured-parent inference.',
  exactTrajectoryScope: 'Shared-RNG-dependent seeds/paired numeric positions remain diagnostics; independent own-actor full/fast motion, playing/lifetime/birth/retirement laws and paired observed lifetime timing are required.',
  emberBirths: Object.fromEntries(Object.entries(emberBirths).map(([game, births]) => [game, Object.fromEntries(births)])),
  emberCoverage, completePairedLifetimes,
  execution, checks, diagnostics, bodies };
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, checks: checks.length, failures: checks.filter(row => row.status === 'fail').length,
  diagnosticFields: diagnostics.length, execution }));
process.exitCode = report.status === 'pass' ? 0 : 1;
