// Item and pickup interactions on a sandbox Challenge start room, one boot per
// case with several interactions per boot where they do not disturb each other.
// Both games receive the same generated Challenge XML / Start Room catalogue and
// the same public keys; each interaction is graded on its own labels.
//
//   npm run test:e2e -- pickup-interactions                 # fixed + 1 OS boot epoch
//   npm run test:e2e -- pickup-interactions --case 'sandbox-*'
//   node tools/e2e/pickup-interactions.mjs --scenarios BANK [--case P] [--production DIST]
//
// Cases live in tools/original/pickup-interaction-scenarios.py; the interaction
// inventory and port status is docs/pickup-interactions.md.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdir, open, readFile, writeFile, cp } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { chromium } from 'playwright';
import { ROOT, flag, hasFlag, outputDirectory, selectCases } from './lib/cli.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';
import { evaluateJson } from './lib/game.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { browserLaunchOptions } from '../foundation/browser.mjs';
import { resetBootElapsed } from '../foundation/verify.mjs';
import { DECLARED_NATIVE_INTRO } from '../foundation/composition-wasm-public.mjs';

const output = outputDirectory('pickup-interactions');
const save = (path, value) => writeFile(path, JSON.stringify(value, null, 2) + '\n');
const jobs = Number(flag('--native-jobs', '2'));

async function child(command, args, log) {
  const file = await open(log, 'w');
  try {
    const process = spawn(command, args, { cwd: ROOT, env: { ...globalThis.process.env, LP_NUM_THREADS: '2', ISAAC_ORIGINAL_SHA: ORIGINAL_SHA },
      stdio: ['ignore', file.fd, file.fd] });
    return await new Promise((done, reject) => {
      process.once('error', reject);
      process.once('close', exitCode => done(exitCode));
    });
  } finally { await file.close(); }
}

// --- Observations normalized to one shape for both games ---------------------
// World coordinates (native units). Effects (1000), the player and dead
// entities are excluded; pickups that were collected are gone in both.
function nativeObservation(observation) {
  const player = observation.players[0];
  const live = observation.entities.filter(entity => !entity.dead);
  return {
    gameAge: observation.fullGameUpdates,
    informationAnimation: observation.informationAnimation,
    room: { gridWidth: observation.room.gridWidth, gridHeight: observation.room.gridHeight },
    renderScroll: observation.room.renderScroll ?? null,
    player: { position: player.position, inventory: player.inventory, hearts: player.health.slice(0, 2),
      activeId: player.activeId, activeCharge: player.activeCharge, reserveCharge: player.reserveCharge,
      pockets: player.heldPockets.map(slot => slot.id), pocketTags: player.heldPockets.map(slot => slot.tag),
      item252RngWords: player.item252RngWords,
      owned: player.ownedItems, animationGuard: player.animationGuard,
      mainAnimation: { clip: player.animation, ...player.mainAnimation, guard: player.animationGuard },
      ownedAll: player.ownedAll ?? null },
    // EvaluateItems outputs; graded only for item-stat interactions.
    stats: player.cachedStats ? { ...player.cachedStats, luck: player.luck, flying: player.flying, cacheFlags: player.cacheFlags,
      primaryWeapon: player.primaryWeapon, tearFlags: player.tearFlags } : null,
    familiars: live.filter(entity => entity.type === 3).map(entity => entity.variant).sort((a, b) => a - b),
    familiarStates: live.filter(entity => entity.type === 3).map(entity => ({ variant: entity.variant, subtype: 0,
      position: entity.position, velocity: entity.velocity, animation: entity.animation, frame: entity.frame, fireCooldown: entity.f58,
      shootHold: entity.f5c, orbitLayer: entity.orbitLayer })),
    bombs: live.filter(entity => entity.type === 4).map(entity => ({ variant: entity.variant, position: entity.position })),
    pickups: live.filter(entity => entity.type === 5).map(entity => ({ variant: entity.variant, subtype: entity.subtype, position: entity.position,
      animation: entity.animation ?? null })),
    pocketBirths: live.filter(entity => entity.type === 5 && [70, 300].includes(entity.variant))
      .map(entity => [entity.variant, entity.subtype, entity.spawnSeed, entity.spawnerType, entity.spawnerVariant, entity.linksEmpty])
      .sort((a, b) => a[0] - b[0] || a[1] - b[1] || a[2] - b[2]),
    npcs: live.filter(entity => entity.type >= 10 && entity.type < 1000)
      .map(entity => ({ type: entity.type, variant: entity.variant, position: entity.position, hp: [entity.hp, entity.maxHp] })),
    gridPoops: observation.gridPoops,
    tears: live.filter(entity => entity.type === 2).map(entity => ({ variant: entity.variant, position: entity.position })),
    // Diagnostic only (not graded): the shared MT index and three words.
    sharedMt: observation.sharedMt ? [observation.sharedMt.index, ...[0, 311, 623].map(i => observation.sharedMt.words[i])] : null,
  };
}

function portObservation() {
  // Entity positions as presented this phase (the original observer reads the
  // live entity Position, which FastUpdate moves between full updates).
  const state = window.game.snapshot();
  const items = state.playerItems;
  const scale = 0.65;
  const owned = id => items?.ownedCounts?.[id] ?? 0;
  return {
    gameAge: state.gameUpdateCount,
    informationAnimation: { state: state.informationAnimation.state, delay: state.informationAnimation.delay,
      actor: state.informationAnimation.currentActor, clip: state.informationAnimation.currentClip,
      frame: state.informationAnimation.frame, playing: state.informationAnimation.playing,
      definitionLoaded: state.informationAnimation.definitionLoaded },
    room: { gridWidth: state.room.grid?.width ?? null, gridHeight: state.room.grid?.height ?? null },
    renderScroll: state.camera ? [Math.fround(state.camera.scroll.x), Math.fround(state.camera.scroll.y)] : null,
    player: { position: [(state.player.x - 32) / scale, (state.player.y + 46) / scale],
      inventory: { bombs: state.inventory.bombs, keys: state.inventory.keys, coins: state.inventory.coins },
      hearts: [state.player.maxHp, state.player.hp],
      activeId: items?.activeId ?? 0, activeCharge: items?.activeCharge ?? 0, reserveCharge: items?.reserveCharge ?? 0,
      pockets: items?.heldPockets ? items.heldPockets.slots.map(slot => slot.id) : [0, 0],
      pocketTags: items?.heldPockets?.slots.map(slot => Number(slot.card)) ?? null,
      pocketFrontier: items?.heldPockets?.frontier ?? null,
      activeWordFrontiers: items?.activeChargeHistory?.unowned ?? null,
      temporary: items?.temporaryCollectibles ?? null,
      item252RngWords: items?.itemRngs?.[252] ? [items.itemRngs[252].state, items.itemRngs[252].rightA,
        items.itemRngs[252].left, items.itemRngs[252].rightB] : null,
      mainAnimation: { clip: state.playerAnimation.current, frame: state.playerAnimation.frame,
        playing: state.playerAnimation.playing, guard: state.playerAnimation.guard },
      owned: { 63: owned(63), 115: owned(115), 286: owned(286) },
      ownedAll: Object.fromEntries((items?.ownedCounts ?? []).flatMap((count, id) => count !== 0 ? [[String(id), count]] : [])) },
    stats: { maxFireDelay: state.maxFireDelay, shotSpeed: state.playerStats.shotSpeed, damage: state.playerStats.damage,
      tearHeight: state.playerStats.tearHeight, tearFallingSpeed: state.playerStats.tearFallingSpeed,
      tearFallingAcceleration: state.playerStats.tearFallingAcceleration, moveSpeed: state.playerStats.moveSpeed,
      luck: state.playerStats.luck, flying: state.player.flying,
      cacheFlags: items?.unevaluatedCacheFlags ?? null, primaryWeapon: state.weaponCache.primary,
      tearFlags: state.weaponCache.flagsC98 },
    familiars: state.familiars.filter(familiar => !familiar.removed).map(familiar => familiar.variant).sort((a, b) => a - b),
    familiarStates: state.familiars.filter(familiar => !familiar.removed).map(familiar => ({ variant: familiar.variant, subtype: 0,
      position: [familiar.presentedPosition.x, familiar.presentedPosition.y], velocity: [familiar.velocity.x, familiar.velocity.y],
      animation: familiar.sprite.animation, frame: familiar.sprite.frame,
      fireCooldown: familiar.fireCooldown, shootHold: familiar.shootHold, orbitLayer: familiar.orbitLayer })),
    bombs: state.bombs.filter(bomb => bomb.motion.lifetime === 'alive').map(bomb => ({ variant: bomb.variant, position: [bomb.motion.presentedPosition.x, bomb.motion.presentedPosition.y] })),
    pickups: state.room.pickups.filter(pickup => !pickup.removed && (!pickup.collected || pickup.variant === 100))
      .map(pickup => ({ variant: pickup.variant, subtype: pickup.subtype, position: [pickup.position.x, pickup.position.y],
        animation: pickup.animation })),
    pocketBirths: state.room.pickups.filter(pickup => !pickup.removed && !pickup.collected && [70, 300].includes(pickup.variant))
      .map(pickup => [pickup.variant, pickup.subtype, pickup.spawnSeed, pickup.sourceSpawner?.entityType ?? null,
        pickup.sourceSpawner?.variant ?? null, pickup.morphLinks
          ? [pickup.morphLinks.parent498, pickup.morphLinks.link4a0, pickup.morphLinks.link4a8].every(link => link === null) : null])
      .sort((a, b) => a[0] - b[0] || a[1] - b[1] || a[2] - b[2]),
    npcs: state.room.npcs.filter(npc => !npc.removed && !npc.dead)
      .map(npc => ({ type: npc.kind, variant: npc.variant, position: [npc.presentedPosition.x, npc.presentedPosition.y], hp: [npc.hp, npc.maxHp] })),
    tears: state.tears.filter(tear => tear.motion.lifetime === 'alive')
      .map(tear => ({ variant: tear.sourceVariant ?? null, position: [tear.motion.presentedPosition.x, tear.motion.presentedPosition.y] })),
    gridPoops: (state.room.grid?.cells ?? []).flatMap((cell, index) => cell.entity?.kind === 14
      ? [[index, cell.entity.variant, cell.entity.collisionClass]] : []),
    sharedMt: [state.bootLifecycle.shared.index, ...[0, 311, 623].map(i => state.bootLifecycle.shared.words[i])],
  };
}

// --- Comparison --------------------------------------------------------------
const distance = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
const TOLERANCE = { player: 2, bomb: 4, pickup: 2 };

function matchPositions(native, port, tolerance) {
  // Greedy nearest matching within the same identity.
  const left = [...port];
  const misses = [];
  for (const item of native) {
    let best = -1, bestDistance = Infinity;
    left.forEach((candidate, index) => {
      if (candidate.variant !== item.variant || candidate.subtype !== item.subtype) return;
      const gap = distance(candidate.position, item.position);
      if (gap < bestDistance) { best = index; bestDistance = gap; }
    });
    if (best < 0) misses.push({ native: item, port: null });
    else {
      if (bestDistance > tolerance) misses.push({ native: item, port: left[best], distance: bestDistance });
      left.splice(best, 1);
    }
  }
  return [...misses, ...left.map(item => ({ native: null, port: item }))];
}

const pathIdentity = value => value?.replaceAll('\\', '/').replace(/^resources\//i, '').toLowerCase() ?? null;

function compareLabel(label, native, port, { tears = false, pocketModifier = false, owned = false, d4Animation = false, stats = false,
  pickupAnimations = false, camera = false } = {}) {
  const rows = [];
  const row = (field, expected, actual, passed) => rows.push({ label, field, native: expected, port: actual, passed });
  const same = (field, expected, actual) => row(field, expected, actual, JSON.stringify(expected) === JSON.stringify(actual));
  same('room.grid', native.room, port.room);
  // Room+15B8 render scroll against the port's CameraStyle1 scroll, binary32.
  if (camera) same('camera.scroll', native.renderScroll, port.renderScroll);
  row('player.position', native.player.position, port.player.position, distance(native.player.position, port.player.position) <= TOLERANCE.player);
  same('player.inventory', native.player.inventory, port.player.inventory);
  same('player.hearts', native.player.hearts, port.player.hearts);
  same('player.active', [native.player.activeId, native.player.activeCharge, native.player.reserveCharge],
    [port.player.activeId, port.player.activeCharge, port.player.reserveCharge]);
  same('player.pockets', native.player.pockets, port.player.pockets);
  if (pocketModifier) {
    same('player.pocketTags', native.player.pocketTags, port.player.pocketTags);
    same('player.item252RngWords', native.player.item252RngWords, port.player.item252RngWords);
    same('pickups.pocketSeedSpawnerLinks', native.pocketBirths, port.pocketBirths);
  }
  same('player.owned', native.player.owned, port.player.owned);
  // Every held collectible count, only where a case's interaction rerolls them.
  if (owned) same('player.ownedAll', native.player.ownedAll, port.player.ownedAll);
  const bombs = matchPositions(native.bombs.map(bomb => ({ ...bomb, subtype: 0 })), port.bombs.map(bomb => ({ ...bomb, subtype: 0 })), TOLERANCE.bomb);
  row('bombs', native.bombs, port.bombs, bombs.length === 0);
  const pickups = matchPositions(native.pickups, port.pickups, TOLERANCE.pickup);
  row('pickups', native.pickups, port.pickups, pickups.length === 0);
  // Each pickup's main clip, where a case's interaction re-inits pickups
  // (a Morph plays Appear).
  if (pickupAnimations) {
    const clips = list => list.map(pickup => [pickup.variant, pickup.subtype, pickup.animation])
      .sort((a, b) => a[0] - b[0] || a[1] - b[1] || String(a[2]).localeCompare(String(b[2])));
    same('pickups.animation', clips(native.pickups), clips(port.pickups));
  }
  // Enemies by type and variant, positions within the pickup tolerance, and
  // each one's [HP, max HP] in f32.
  const npcs = matchPositions(native.npcs.map(npc => ({ ...npc, variant: npc.type, subtype: npc.variant })),
    port.npcs.map(npc => ({ ...npc, variant: npc.type, subtype: npc.variant })), TOLERANCE.pickup);
  row('npcs', native.npcs, port.npcs, npcs.length === 0);
  const health = list => list.map(npc => [npc.type, npc.variant, ...npc.hp.map(Math.fround)]).sort((a, b) => a[0] - b[0] || a[2] - b[2]);
  same('npcs.hp', health(native.npcs), health(port.npcs));
  // Poop grid entities by cell: [index, variant, collision class].
  same('grid.poops', native.gridPoops, port.gridPoops);
  // EvaluateItems outputs field by field: f32 stats compared as binary32.
  if (stats) {
    for (const field of ['maxFireDelay', 'shotSpeed', 'damage', 'tearHeight', 'tearFallingSpeed', 'tearFallingAcceleration', 'moveSpeed', 'luck'])
      row(`stats.${field}`, native.stats?.[field] ?? null, port.stats?.[field] ?? null,
        native.stats != null && Math.fround(native.stats[field]) === Math.fround(port.stats?.[field] ?? NaN));
    for (const field of ['flying', 'cacheFlags', 'primaryWeapon', 'tearFlags'])
      same(`stats.${field}`, native.stats?.[field] ?? null, port.stats?.[field] ?? null);
    same('familiars', native.familiars ?? null, port.familiars ?? null);
    // Each Familiar: position within the player tolerance, then its clip,
    // F58, F5C and orbit layer exactly (sorted by variant and position).
    const nativeFamiliars = native.familiarStates ?? [], portFamiliars = port.familiarStates ?? [];
    const placed = matchPositions(nativeFamiliars, portFamiliars, TOLERANCE.player);
    row('familiars.position', nativeFamiliars.map(f => [f.variant, f.position]), portFamiliars.map(f => [f.variant, f.position]),
      placed.length === 0);
    const detail = list => list.map(f => [f.variant, f.animation, f.fireCooldown, f.shootHold, f.orbitLayer])
      .sort((a, b) => a[0] - b[0] || String(a[1]).localeCompare(String(b[1])) || a[2] - b[2] || a[3] - b[3]);
    same('familiars.state', detail(nativeFamiliars), detail(portFamiliars));
  }
  // Player tears, only where a case's interaction is about them.
  if (tears) {
    const misses = matchPositions(native.tears.map(tear => ({ ...tear, subtype: 0 })), port.tears.map(tear => ({ ...tear, subtype: 0 })), TOLERANCE.pickup);
    row('tears', native.tears, port.tears, misses.length === 0);
  }
  if (d4Animation) {
    same('Game.age', native.gameAge, port.gameAge);
    for (const field of ['state', 'delay', 'clip', 'frame', 'playing', 'definitionLoaded'])
      same(`Info.${field}`, native.informationAnimation[field], port.informationAnimation[field]);
    same('Info.actor', pathIdentity(native.informationAnimation.actor), pathIdentity(port.informationAnimation.actor));
    // Walking owns the ordinary main clip outside the forced animation owner.
    // Any actual UseItem on either side still requires all four exact fields.
    if (native.player.mainAnimation.clip === 'UseItem' || port.player.mainAnimation.clip === 'UseItem')
      same('player.mainAnimation', native.player.mainAnimation, port.player.mainAnimation);
  }
  return rows;
}

const D4_LABELS = ['settle', 'book-request-1', 'book-request-2', 'use-active', 'book-delay', 'book-start',
  'book-mid-1', 'book-mid-2', 'after-active', ...Array.from({ length: 8 }, (_, i) => `book-finish-${i + 1}`), 'book-resumed'];

function d4LifecycleRows(native, port) {
  const coverage = observations => {
    const rows = D4_LABELS.map(label => observations[label]).filter(Boolean);
    const request = rows.findIndex(row => row.informationAnimation?.state === 1 && row.informationAnimation.delay === 3);
    const start = rows.findIndex((row, index) => index > request && row.informationAnimation?.state === 2);
    const complete = rows.findIndex((row, index) => start >= 0 && index > start && row.informationAnimation?.state === 0
      && row.informationAnimation.clip === 'Appear' && row.informationAnimation.frame === 33 && !row.informationAnimation.playing);
    const active = rows.filter(row => row.informationAnimation?.state === 2);
    const pose = row => JSON.stringify(row.player.mainAnimation);
    return {
      'actual delayed request': request >= 0,
      'actual delay countdown': [2, 1].every(delay => rows.some(row => row.informationAnimation?.state === 1 && row.informationAnimation.delay === delay)),
      'actual ordered request/start/completion': request >= 0 && start > request && complete > start,
      'actual GiantBook actor/Appear owner': start >= 0 && rows.slice(request, complete < 0 ? undefined : complete + 1)
        .every(row => pathIdentity(row.informationAnimation?.actor) === 'gfx/ui/giantbook/giantbook.anm2'
          && row.informationAnimation.clip === 'Appear' && row.informationAnimation.definitionLoaded),
      'actual playing first frame': active.some(row => row.informationAnimation.frame === 0 && row.informationAnimation.playing),
      'actual playing middle frame': active.some(row => row.informationAnimation.frame > 0 && row.informationAnimation.frame < 33 && row.informationAnimation.playing),
      'actual retained stopped last frame': active.some(row => row.informationAnimation.frame === 33 && !row.informationAnimation.playing),
      'actual frozen Game age and guarded UseItem pose': active.length >= 3 && Number.isInteger(active[0].gameAge)
        && active.every(row => row.gameAge === active[0].gameAge && row.player.mainAnimation.clip === 'UseItem'
          && row.player.mainAnimation.guard && row.player.mainAnimation.playing && pose(row) === pose(active[0])),
      'actual post-book resumption': complete >= 0 && active.length > 0 && rows.some((row, index) => index > complete
        && row.informationAnimation?.state === 0 && row.gameAge > rows[complete].gameAge
        && pose(row) !== pose(active[0])),
    };
  };
  const source = coverage(native), browser = coverage(port);
  return Object.keys(source).map(field => ({ label: 'd4-lifecycle', field,
    native: { required: true, observed: source[field] }, port: { required: true, observed: browser[field] },
    passed: source[field] && browser[field] }));
}

// Each interaction names the labels it is graded on and a predicate that the
// ORIGINAL must satisfy there, so a case that no longer exercises its
// interaction fails as coverage rather than passing vacuously.
const bombX = observation => observation.bombs[0]?.position[0];
const pickupCount = observation => observation.pickups.length;
export const INTERACTIONS = {
  'bomb-walk-push': { labels: ['place-bomb', 'pause-beside-bomb', 'walk-right-into-bomb', 'keep-pushing', 'after-push'],
    covered: obs => bombX(obs['after-push']) - bombX(obs['pause-beside-bomb']) > 15 },
  'bomb-tear-push': { labels: ['step-left-off-bomb', 'shoot-right-at-bomb', 'after-shots'],
    covered: obs => bombX(obs['after-shots']) - bombX(obs['step-left-off-bomb']) > 20 },
  'spectral-tear-bomb-push': { labels: ['step-left-off-bomb', 'shoot-right-at-bomb', 'after-shots'],
    covered: obs => obs['settle'].player.owned[115] === 1 && bombX(obs['after-shots']) - bombX(obs['step-left-off-bomb']) > 20 },
  'battery-overcharge': { labels: ['settle', 'walk-to-battery', 'after-battery'],
    covered: obs => obs['after-battery'].player.reserveCharge > 0 },
  'blank-card-jera': { labels: ['settle-before-active', 'use-active', 'after-active'],
    covered: obs => pickupCount(obs['after-active']) === 2 * pickupCount(obs['settle-before-active']) && obs['after-active'].player.pockets[0] === 29 },
  // Q Jera after the Blank Card copy duplicates the duplicates as well.
  'jera-after-blank-card': { labels: ['use-pocket', 'after-pocket'],
    covered: obs => obs['after-pocket'].player.pockets[0] === 0 && pickupCount(obs['after-pocket']) === 2 * pickupCount(obs['after-active']) },
  'jera-pocket': { labels: ['settle', 'use-jera', 'after-jera'],
    covered: obs => pickupCount(obs['after-jera']) > pickupCount(obs['settle']) && obs['after-jera'].player.pockets[0] === 0 },
  'card-collect': { labels: ['walk-to-diamonds', 'after-diamonds', 'walk-to-clubs', 'after-clubs'],
    covered: obs => obs['after-diamonds'].player.pockets[0] === 24 && obs['after-clubs'].player.pockets[0] === 23 },
  'two-of-diamonds': { labels: ['use-diamonds', 'after-diamonds-use'],
    covered: obs => obs['after-diamonds-use'].player.inventory.coins !== obs['after-diamonds'].player.inventory.coins },
  'two-of-clubs': { labels: ['use-clubs', 'after-clubs-use'],
    covered: obs => obs['after-clubs-use'].player.inventory.bombs === 2 * obs['after-clubs'].player.inventory.bombs },
  'blank-card-card': { labels: ['settle', 'use-active', 'after-active'],
    covered: obs => obs['after-active'].player.activeCharge === 0 && obs['after-active'].player.pockets[0] === 23 },
  'blank-card-pocket-conversion': { labels: ['settle', 'after-conversion', 'use-active', 'after-active'],
    covered: obs => obs['settle'].player.pockets[0] === 23 && obs['settle'].player.pocketTags[0] === 1
      && obs['after-conversion'].player.pockets[0] > 0 && obs['after-conversion'].player.pocketTags[0] === 0
      && obs['after-conversion'].player.item252RngWords?.length === 4
      && obs['after-conversion'].player.item252RngWords[0] !== obs['settle'].player.item252RngWords?.[0]
      && obs['after-conversion'].pocketBirths.some(pickup => pickup[0] === 70 && pickup[2] > 0
        && pickup[3] === 1 && pickup[4] === 0 && pickup[5] === true)
      && obs['after-active'].player.activeCharge === 0
      && obs['after-active'].player.inventory.bombs === obs['after-conversion'].player.inventory.bombs,
    portContract: activeOwnershipContract },
  // The 2x2 Start Room scrolls: the camera follows Isaac from his first steps
  // while the stage name is still showing (#965).
  'room-2x2': { camera: true, labels: ['settle', 'walk-up-start', 'walk-up-mid', 'walk-to-battery', 'after-battery', 'walk-back'],
    covered: obs => obs['settle'].room?.gridWidth === 28
      && obs['walk-up-mid'].renderScroll?.[1] !== obs['settle'].renderScroll?.[1] },
  // Every free pickup keeps its place and at least one changes kind.
  'd20-reroll': { pickupAnimations: true, labels: ['settle', 'use-active', 'reroll-appear', 'after-active'],
    covered: obs => obs['after-active'].player.activeCharge === 0
      && pickupCount(obs['after-active']) === pickupCount(obs['settle'])
      && JSON.stringify(obs['after-active'].pickups.map(p => [p.variant, p.subtype]))
        !== JSON.stringify(obs['settle'].pickups.map(p => [p.variant, p.subtype])) },
  // Every enemy is replaced: none of the settle enemies' types survive
  // unchanged in count, and the enemy count stays the same.
  'd10-reroll': { labels: ['settle', 'use-active', 'after-active'],
    covered: obs => obs['after-active'].player.activeCharge === 0
      && obs['settle'].npcs.length > 0 && obs['after-active'].npcs.length === obs['settle'].npcs.length
      && JSON.stringify(obs['after-active'].npcs.map(n => n.type).sort())
        !== JSON.stringify(obs['settle'].npcs.map(n => n.type).sort()) },
  // A poop appears under Isaac (passable while he stands on it), then
  // blocks him once he has walked off and back.
  'the-poop': { labels: ['settle', 'use-active', 'after-active', 'walk-off', 'walk-back'],
    covered: obs => obs['settle'].gridPoops.length === 0 && obs['after-active'].gridPoops.length === 1 },
  // Troll Bombs land around the room on every fifth TEMP65 count.
  'anarchist-cookbook': { labels: ['settle', 'use-active', 'after-active', 'bombs-1', 'bombs-2', 'bombs-3', 'bombs-4', 'after-blasts', 'chain-1', 'chain-2', 'chain-3', 'chain-4'],
    covered: obs => ['bombs-1', 'bombs-2', 'bombs-3', 'bombs-4', 'after-blasts'].some(label => obs[label].bombs.filter(bomb => bomb.variant === 3).length >= 3) },
  // The D4 spends its charge, stays held, and changes the passive collectibles.
  'd4-reroll': { owned: true, labels: [...D4_LABELS, 'd4-lifecycle'],
    covered: obs => obs['settle'].player.activeCharge > 0 && obs['after-active'].player.activeId === 284
      && obs['after-active'].player.activeCharge === 0
      && JSON.stringify(obs['after-active'].player.ownedAll) !== JSON.stringify(obs['settle'].player.ownedAll) },
  // The D100 spends its charge, stays held, and changes the passives, the
  // pedestal and the loose pickups.
  'd100-reroll': { owned: true, pickupAnimations: true, labels: ['settle', 'use-active', 'reroll-appear', 'after-active'],
    covered: obs => obs['settle'].player.activeCharge > 0 && obs['after-active'].player.activeId === 283
      && obs['after-active'].player.activeCharge === 0
      && JSON.stringify(obs['after-active'].player.ownedAll) !== JSON.stringify(obs['settle'].player.ownedAll)
      && JSON.stringify(obs['after-active'].pickups.map(p => [p.variant, p.subtype]))
        !== JSON.stringify(obs['settle'].pickups.map(p => [p.variant, p.subtype])) },
  // A ring of ten tears leaves Isaac at once.
  'tammys-head': { tears: true, labels: ['settle', 'use-active', 'after-active', 'tears-later'],
    covered: obs => obs['use-active'].tears.length >= 10 || obs['after-active'].tears.length >= 10 },
  // A granted item set: every EvaluateItems output, then tears fired with it.
  'item-stats': { stats: true, tears: true, owned: true, labels: ['settle', 'shoot-right', 'after-shots'],
    covered: obs => obs['settle'].stats != null && obs['shoot-right'].tears.length > 0 },
  // A generated item set (--items [--trinket] [--enemy]): every EvaluateItems
  // output, the Familiars, then tears in each direction and the enemy's HP.
  'item-set': { stats: true, tears: true, owned: true,
    labels: ['settle', 'shoot-right', 'shoot-up', 'shoot-left', 'shoot-down', 'after-shots'],
    covered: obs => obs['settle'].stats != null
      && ['shoot-right', 'shoot-up', 'shoot-left', 'shoot-down'].some(label => obs[label].tears.length > 0) },
  'battery-recharge': { labels: ['walk-to-battery', 'after-battery'], covered: obs => obs['after-battery'].player.activeCharge > 0 },
};

// This is a conditional runtime ownership contract, separate from the strict
// Native comparisons above. A future owned conversion still needs those rows;
// it does not have to retain today's unimplemented-conversion frontier.
function activeOwnershipContract(observations) {
  const before = observations['after-conversion']?.player;
  const after = observations['after-active']?.player;
  if (!before || !after) return { status: 'fail', unknownOwnershipObserved: false,
    rows: [{ field: 'active-ownership-observations', passed: false }] };
  const frontier = before.activeWordFrontiers;
  const unknown = frontier === null || Object.hasOwn(frontier, 'activeId') || Object.hasOwn(frontier, 'charge');
  if (!unknown) return { status: 'not-applicable', unknownOwnershipObserved: false, rows: [],
    scope: 'Active ID and charge are owned; the strict Native rows grade the conversion and use.' };
  const rows = [
    ['active-unknown.inventory', before.inventory, after.inventory],
    ['active-unknown.temporary', before.temporary, after.temporary],
    ['active-unknown.charge', before.activeCharge, after.activeCharge],
    ['active-unknown.frontier', before.activeWordFrontiers, after.activeWordFrontiers],
  ].map(([field, expected, actual]) => ({ field, expected, actual,
    passed: JSON.stringify(expected) === JSON.stringify(actual) }));
  return { status: rows.every(row => row.passed) ? 'pass' : 'fail', unknownOwnershipObserved: true, rows,
    scope: 'Observed unknown active operands at the physical Space request; no historical active body may run. This does not qualify Little Baggy conversion.' };
}

// --- Browser capture -----------------------------------------------------------
const keyNames = { Return: 'Enter', Tab: 'Tab', Down: 'ArrowDown', Up: 'ArrowUp', Left: 'ArrowLeft', Right: 'ArrowRight', space: 'Space' };

async function capturePort(browser, server, declaration, base, directory) {
  const xml = await readFile(resolve(base, declaration.fixture.authoredResourcePath), 'utf8');
  const stb = await readFile(resolve(base, declaration.fixture.specialRoomsPath));
  const resources = { challengesXml: xml, specialRooms: Array.from(stb) };
  const context = await browser.newContext({ viewport: { width: 960, height: 540 } });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  const observations = {};
  try {
    await page.goto(server.url + '/?manual=1');
    // A loaded host can take longer than the default 30 s to boot the build.
    await page.waitForFunction(() => ['true', 'failed'].includes(document.body.dataset.ready), null, { timeout: 180000 });
    assert.equal(await page.evaluate(() => document.body.dataset.ready), 'true', 'Production browser must boot');
    await page.evaluate(input => { window.game.pause(); window.game.reset(1, input); },
      { bootEpochSeed: declaration.bootEpochSeed, resources, audioPreferences: { sfxVolume: 0, musicVolume: 0 },
        fullscreenPreferences: { fullscreen: false }, bootElapsed: resetBootElapsed(DECLARED_NATIVE_INTRO) });
    const step = count => page.evaluate(count => { window.game.step(count); }, count);
    const press = async (key, settle) => {
      await page.keyboard.down(keyNames[key] ?? key); await step(12);
      await page.keyboard.up(keyNames[key] ?? key); await step(settle);
    };
    // The composition-run.py Challenge route: Title to profile, profile to main, Down to
    // Challenges, Challenge 1, then 300 admission presentations.
    await step(63);
    await press('Return', 72); await press('Return', 72);
    await press('Down', 72); await press('Return', 72); await press('Return', 300);
    const screen = (await evaluateJson(page, () => window.game.snapshot().screen));
    assert.equal(screen, 'playing', `Sandbox Challenge did not start (screen ${screen})`);
    if (declaration.initialPhaseOffset) await step(1);
    let held = new Set();
    for (const action of declaration.actions) {
      const requested = new Set(action.keys);
      for (const key of [...held].filter(key => !requested.has(key)).sort()) await page.keyboard.up(keyNames[key] ?? key);
      for (const key of [...requested].filter(key => !held.has(key)).sort()) await page.keyboard.down(keyNames[key] ?? key);
      await step(action.presentations);
      observations[action.label] = await evaluateJson(page, portObservation);
      // Actual rendered canvas evidence, beside the original's capture PNGs.
      // Native sheet identity is unobserved; these images need visual review.
      if (declaration.interactions.includes('d4-reroll'))
        await page.locator('#game').screenshot({ path: join(directory, `port-${action.label}.png`) });
      held = requested;
    }
    for (const key of [...held].sort()) await page.keyboard.up(keyNames[key] ?? key);
  } catch (error) {
    errors.push(String(error?.message ?? error).split('\n')[0]);
    const feedback = await page.evaluate(() => JSON.stringify(window.game.feedback().events.slice(-2))).catch(() => null);
    if (feedback) errors.push('Runtime feedback: ' + feedback.slice(0, 2000));
  } finally { await context.close(); }
  return { observations, errors };
}

// --- Main ----------------------------------------------------------------------
const report = { schema: 1, suite: 'pickup-interactions', status: 'running', startedUtc: new Date().toISOString(), cases: [], interactions: {} };
let browser, server;
try {
  await assertOriginalBinary();
  let bankPath = flag('--scenarios');
  if (!bankPath) {
    const generated = join(output, 'declared-inputs');
    const only = process.argv.flatMap((arg, index) => ['--only', '--items', '--trinket', '--enemy'].includes(arg)
      ? [arg, process.argv[index + 1]] : []);
    const exit = await child('python3', [join(ROOT, 'tools/original/pickup-interaction-scenarios.py'), '--output', generated,
      '--count', flag('--count', '1'), ...only], join(output, 'declaration.log'));
    assert.equal(exit, 0, 'Case declaration failed; see declaration.log');
    bankPath = join(generated, 'scenarios.json');
  }
  bankPath = resolve(bankPath);
  const bank = JSON.parse(await readFile(bankPath, 'utf8'));
  assert.equal(bank.suite, 'pickup-interactions');
  // Fixture paths are relative to the bank; keep a copy beside this run's
  // scenarios.json so it replays on its own.
  const base = dirname(bankPath);
  if (base !== output) await cp(join(base, 'resources'), join(output, 'resources'), { recursive: true });
  for (const item of bank.cases) for (const name of item.interactions) assert.ok(INTERACTIONS[name], `Unknown interaction ${name}`);
  const cases = selectCases(bank.cases, item => item.name);
  const scenarios = join(output, 'scenarios.json');
  if (base !== output) await save(scenarios, { ...bank, caseFilter: flag('--case'), cases });
  const resourceBase = output;

  server = await startProductionServer({ directory: resolve(flag('--production', join(ROOT, 'dist'))) });
  report.production = server.build;
  const launch = browserLaunchOptions();
  browser = await chromium.launch({ ...launch, headless: false, args: [...launch.args, '--no-sandbox'] });

  // Native captures run a few at a time while the browser plays each case.
  const native = new Map();
  const queue = [...cases];
  const workers = Array.from({ length: Math.max(1, jobs) }, async () => {
    while (queue.length) {
      const declaration = queue.shift();
      const directory = join(output, declaration.name);
      await mkdir(directory, { recursive: true });
      // Each original gets its own virtual display: two games on one X server
      // would steal each other's focus and keys.
      const exit = await child('bash', [join(ROOT, 'tools/original/with-display.sh'), 'python3',
        join(ROOT, 'tools/original/pickup-interaction-run.py'), '--scenarios', scenarios,
        '--case', declaration.name, '--output', join(directory, 'original'), '--frame-timeout', flag('--native-frame-timeout', '60')],
      join(directory, 'original.log'));
      native.set(declaration.name, exit);
    }
  });
  const nativeDone = Promise.all(workers);
  const port = new Map();
  for (const declaration of cases) {
    const directory = join(output, declaration.name);
    await mkdir(directory, { recursive: true });
    const captured = await capturePort(browser, server, declaration, resourceBase, directory);
    await save(join(directory, 'port.json'), captured);
    port.set(declaration.name, captured);
  }
  await nativeDone;

  for (const declaration of cases) {
    const directory = join(output, declaration.name);
    const result = { name: declaration.name, about: declaration.about, errors: [], interactions: {} };
    report.cases.push(result);
    let nativeObservations = {};
    try {
      assert.equal(native.get(declaration.name), 0, 'Original capture failed; see original.log');
      const manifest = JSON.parse(await readFile(join(directory, 'original/run/manifest.json'), 'utf8'));
      for (const entry of manifest.debugObservations ?? []) {
        if (entry.observation.status === 'pass' && !entry.context.label.includes('-observe-step'))
          nativeObservations[entry.context.label] = nativeObservation(entry.observation);
      }
    } catch (error) { result.errors.push('Original: ' + String(error.message ?? error)); }
    const captured = port.get(declaration.name);
    result.errors.push(...captured.errors.map(error => 'Port: ' + error));
    const rows = [];
    for (const action of declaration.actions) {
      const nativeLabel = nativeObservations[action.label], portLabel = captured.observations[action.label];
      if (!nativeLabel || !portLabel) rows.push({ label: action.label, field: 'observation', native: Boolean(nativeLabel), port: Boolean(portLabel), passed: false });
      else rows.push(...compareLabel(action.label, nativeLabel, portLabel, {
        tears: declaration.interactions.some(name => INTERACTIONS[name].tears),
        stats: declaration.interactions.some(name => INTERACTIONS[name].stats),
        pocketModifier: declaration.interactions.includes('blank-card-pocket-conversion'),
        owned: declaration.interactions.some(name => INTERACTIONS[name].owned),
        d4Animation: declaration.interactions.includes('d4-reroll'),
        pickupAnimations: declaration.interactions.some(name => INTERACTIONS[name].pickupAnimations),
        camera: declaration.interactions.some(name => INTERACTIONS[name].camera),
      }));
    }
    if (declaration.interactions.includes('d4-reroll'))
      rows.push(...d4LifecycleRows(nativeObservations, captured.observations));
    await save(join(directory, 'comparison.json'), { rows, native: nativeObservations, port: captured.observations });
    for (const name of declaration.interactions) {
      const spec = INTERACTIONS[name];
      let covered = false;
      try { covered = Boolean(spec.covered(nativeObservations)); } catch { covered = false; }
      const graded = rows.filter(row => spec.labels.includes(row.label));
      const failed = graded.filter(row => !row.passed);
      const portContract = spec.portContract?.(captured.observations);
      const status = !covered ? 'uncovered' : graded.length && !failed.length && portContract?.status !== 'fail' ? 'pass' : 'fail';
      result.interactions[name] = { status, checks: graded.length, failed: failed.length,
        firstFailures: failed.slice(0, 4).map(row => `${row.label} ${row.field}: original ${JSON.stringify(row.native)} port ${JSON.stringify(row.port)}`),
        ...(portContract ? { portContract } : {}) };
      (report.interactions[name] ??= []).push({ case: declaration.name, status });
    }
    result.status = !result.errors.length && Object.values(result.interactions).every(item => item.status === 'pass') ? 'pass' : 'fail';
    console.log(JSON.stringify({ case: declaration.name, status: result.status,
      interactions: Object.fromEntries(Object.entries(result.interactions).map(([name, value]) => [name, value.status])), errors: result.errors }));
  }
  report.status = report.cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
} catch (error) {
  report.error = String(error?.stack ?? error);
  report.status = 'fail';
} finally {
  for (const resource of [browser, server]) if (resource) await resource.close().catch(() => {});
  report.closedUtc = new Date().toISOString();
  await save(join(output, 'report.json'), report);
}
console.log('Interactions:');
for (const [name, runs] of Object.entries(report.interactions))
  console.log(`  ${name.padEnd(26)} ${runs.map(run => `${run.case}=${run.status}`).join(' ')}`);
if (report.error) console.error(report.error);
console.log(JSON.stringify({ status: report.status, cases: report.cases.length, output }));
if (report.status !== 'pass') console.log(`Replay: npm run test:e2e -- pickup-interactions --scenarios ${join(output, 'scenarios.json')}`);
process.exitCode = report.status === 'pass' ? 0 : 1;
