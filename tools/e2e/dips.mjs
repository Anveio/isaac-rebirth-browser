// Fresh original and immutable production WASM; all selected outcomes retained.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {runPairedGate, scopedState} from '../foundation/paired-game.mjs';

const option = (name, fallback) => {
  const at = process.argv.indexOf(name);
  if (at < 0) return fallback;
  assert.ok(process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
const output = resolve(option('--output', `.cache/e2e/dips/${Date.now()}-${process.pid}`));
mkdirSync(output, {recursive: true});
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n');
const sources = ['tools/e2e/dips.mjs', 'tools/original/dip-scenarios.py', 'tools/original/foundation-combat-observe.py',
  'engine/src/dips.rs', 'engine/src/npcs.rs', 'engine/src/familiars.rs', 'engine/src/lib.rs', 'engine/build.rs',
  'engine/src/poop_effects.rs', 'engine/src/gib_counts.rs', 'web/engine.ts', 'web/renderer.ts',
  'web/npc-data.json', 'web/familiar-data.json', 'docs/dip-source-contract.json', 'docs/poop-effects-source-contract.json'];
const hashes = () => Object.fromEntries(sources.map(path => [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const before = hashes();
const scenarios = JSON.parse(execFileSync('python3', ['tools/original/dip-scenarios.py', '--count', option('--count', '2')],
  {encoding: 'utf8', maxBuffer: 8 * 1024 * 1024}));
save(resolve(output, 'scenarios.json'), scenarios);
const tapes = scenarios.cases.map((value, n) => {
  const path = resolve(output, `case-${n}.json`); save(path, value); return path;
});
const check = (rows, field, expected, actual) => rows.push({field, expected, actual,
  status: expected !== undefined && actual !== undefined && JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail'});
const f32 = value => typeof value === 'number' && Number.isFinite(value) ? Math.fround(value) : undefined;
const vector = value => value ? [f32(value.x ?? value[0]), f32(value.y ?? value[1])] : undefined;
const identity = actor => [actor.type ?? actor.kind, actor.variant, actor.subtype];

function compare(source, actual, gameplay) {
  const rows = scopedState(source, actual, gameplay);
  if (!gameplay || actual.screen !== 'playing') return rows;
  assert.equal(source.gameplay.status, 'pass');
  assert.equal(source.gameplay.memoryWritten, false);
  const original = source.gameplay;
  const npcs = original.entities.filter(actor => actor.type >= 10 && actor.type < 1000);
  const candidates = actual.room.npcs.filter(actor => !actor.removed);
  check(rows, 'NPC.identities', npcs.map(identity), candidates.map(identity));
  const dips = npcs.filter(actor => actor.type === 217);
  const phase = original.nativePhase.systemIterations % 2;
  const poofs = original.entities.filter(actor => actor.type === 1000 && actor.variant === 15 && actor.parentType === 217);
  const portPoofs = actual.spriteEffects.filter(actor => actor.active && !actor.removed && actor.variant === 15 && actor.spawnerType === 217);
  check(rows, 'DipPoof.births', poofs.map(actor => [actor.subtype, actor.spawnSeed]),
    portPoofs.map(actor => [actor.subtype, actor.spawnSeed]));
  for (const [index, poof] of poofs.entries()) {
    const port = portPoofs.find(actor => actor.spawnSeed === poof.spawnSeed);
    if (!port) continue;
    for (const [field, expected, observed] of [
      ['spawner', [poof.parentType, poof.parentVariant], [port.spawnerType, port.spawnerVariant]],
      ['linkedParent', poof.effectParent, port.parent], ['flags', poof.entityFlags, port.flags],
      ['animation', poof.animation, port.sprite.animation], ['frame', f32(poof.animationFrame), f32(port.sprite.frame)],
      ['playing', poof.animationPlaying, port.sprite.playing],
      ['position', vector(poof.position), vector(phase === 0 ? port.presentedPosition : port.position)],
      ['height', f32(poof.effectRaw7f8), f32(port.raw7f8)],
    ]) check(rows, `DipPoof${index}.${field}`, expected, observed);
  }
  for (const [index, npc] of dips.entries()) {
    const port = candidates.find(actor => actor.kind === 217 && actor.spawnSeed === npc.spawnSeed);
    check(rows, `Dip${index}.spawnSeed`, npc.spawnSeed, port?.spawnSeed);
    if (!port) continue;
    for (const [name, expected, observed] of [
      ['state', npc.npcState, port.state], ['appearanceCounter', npc.npcFrameAge, port.frameAge],
      ['savedInitialization', npc.npcSavedInitializedState, port.savedInitializedState],
      ['hp', f32(npc.hp), f32(port.hp)], ['maxHp', f32(npc.maxHp), f32(port.maxHp)],
      ['radius', f32(npc.collisionRadius), f32(port.collisionRadius)], ['mass', f32(npc.collisionMass), f32(port.collisionMass)],
      ['contactDamage', f32(npc.collisionDamage), f32(port.collisionDamage)],
      ['gridClass', npc.gridCollisionClass, port.gridCollisionClass], ['entityClass', npc.entityCollisionClass, port.entityCollisionClass],
      ['countable', npc.canShutDoors, port.canShutDoors], ['boss', npc.boss, port.boss],
      // The legacy original observer calls31f "removed";31e remains separate.
      ['dead31f', npc.removed, port.dead], ['active31e', npc.active, !port.removed], ['visible', npc.visible, port.visible],
      ['flags', npc.entityFlags, port.dip?.entityFlags], ['friction', f32(npc.friction), f32(port.dip?.currentFriction)],
      ['flipX', npc.spriteFlipX, port.dip?.flipX], ['animation', npc.animation, port.animation],
      ['frame', f32(npc.animationFrame), f32(port.animationClock)], ['playing', npc.animationPlaying, port.animationPlaying],
      ['pendingDamage', f32(npc.pendingDamage), f32(port.damage.pendingDamage)], ['pendingFlags', npc.pendingFlags, port.damage.pendingFlags],
      ['orderedDamage', npc.pendingDamageRecords.map(event => [f32(event.amount), event.flags, event.sourceType]),
        port.damage.events.map(event => [f32(event.amount), {tear: 0, bomb: 4, fireContact: 2, roomEffect: 0}[event.kind], event.source.entityType])],
      ['position', vector(npc.position), vector(phase === 0 ? port.presentedPosition : port.position)],
      ['velocity', vector(npc.velocity), vector(port.velocity)],
      ['renderOffset', vector(npc.entityRenderOffset), vector(port.dip?.renderOffset)],
      ['statusCountdown', npc.statusCountdown, port.dip?.statusCountdown],
      ['tint', npc.spriteTint.map(f32), port.dip?.tint.map(f32)], ['offsets', npc.spriteOffsets.map(f32), port.dip?.offsets.map(f32)],
    ]) check(rows, `Dip${index}.${name}`, expected, observed);
  }
  // Every actual58/43 body remains required; zero category5 count is legal.
  const poop = original.entities.filter(actor => actor.type === 1000 && [58, 43].includes(actor.variant));
  const effects = actual.spriteEffects.filter(actor => actor.active && !actor.removed && [58, 43].includes(actor.variant));
  check(rows, 'poopEffects.births', poop.map(actor => [actor.variant, actor.subtype, actor.spawnSeed]),
    effects.map(actor => [actor.variant, actor.subtype, actor.spawnSeed]));
  for (const [index, effect] of poop.entries()) {
    const port = effects.find(actor => actor.variant === effect.variant && actor.spawnSeed === effect.spawnSeed);
    if (!port) continue;
    const controller = port.poopEffect;
    const particle = controller?.particle58;
    const flight = typeof particle === 'object' ? particle.flying : undefined;
    const state = effect.variant === 58 ? (particle === 'initializing' ? 0 : flight ? 1 : undefined)
      : controller?.explosion43?.frameTarget;
    for (const [field, expected, observed] of [
      ['state', effect.poopEffectState, state],
      ['position', vector(effect.position), vector(phase === 0 ? port.presentedPosition : port.position)],
      ['velocity', vector(effect.velocity), vector(port.velocity)],
      ['renderOffset', vector(effect.entityRenderOffset), vector(port.renderOffset)],
      ['spawner', [effect.parentType, effect.parentVariant], [port.spawnerType, port.spawnerVariant]],
      ['linkedParent', effect.effectParent, port.parent], ['flags', effect.entityFlags, port.flags],
      ['radius', f32(effect.collisionRadius), f32(port.collisionRadius)],
      ['mass', f32(effect.collisionMass), f32(port.collisionMass)],
      ['damage', f32(effect.collisionDamage), f32(port.collisionDamage)],
      ['gridClass', effect.gridCollisionClass, port.gridCollisionClass],
      ['entityClass', effect.entityCollisionClass, port.entityCollisionClass],
      ['friction', f32(effect.friction), f32(port.currentFriction)],
      ['hp', f32(effect.hp), f32(port.hp)], ['maxHp', f32(effect.maxHp), f32(port.maxHp)],
      ['animation', effect.animation, port.sprite.animation],
      ['frame', f32(effect.animationFrame), f32(port.sprite.frame)],
      ['playing', effect.animationPlaying, port.sprite.playing],
      ['speed', f32(effect.animationSpeed), f32(port.sprite.speed)],
      ['flip', [effect.spriteFlipX, effect.spriteFlipY], [port.sprite.flipX, port.sprite.flipY]],
      ['scale', vector(effect.spriteScale), vector(port.sprite.scale)],
      ['tint', effect.spriteTint.map(f32), port.sprite.tint.map(f32)],
      ['offsets', effect.spriteOffsets.map(f32), port.sprite.offsets.map(f32)],
      ['raw7d8', effect.effectRaw7d8, port.raw7d8],
      ['ttl', effect.effectTimeToLive, port.timeToLive],
      ['raw7f4', effect.effectRaw7f4, port.raw7f4],
      ['raw7f8', f32(effect.effectRaw7f8), f32(port.raw7f8)],
      ['depth', effect.renderOrderOffset, port.renderOrderOffset],
    ]) check(rows, `poopEffects${index}.${field}`, expected, observed);
    if (effect.variant === 58 && effect.poopEffectState !== 0) {
      for (const [field, expected, observed] of [
        ['height', effect.poopParticleHeight, flight?.height],
        ['verticalVelocity', effect.poopParticleVerticalVelocity, flight?.verticalVelocity],
        ['gravity', effect.poopParticleGravity, flight?.gravity],
        ['layerRotation', effect.poopParticleLayer0Rotation, flight?.layerRotation],
      ]) check(rows, `poopEffects${index}.${field}`, f32(expected), f32(observed));
    }
  }
  for (const field of ['countablePending', 'countableCurrent', 'bossPending', 'bossCurrent']) {
    check(rows, `manager.${field}`, original.room[field], actual.room.managerCounts[field]);
  }
  return rows;
}

const paired = await runPairedGate({output: resolve(output, 'paired'), selected: [], randomCount: 0, replayCases: tapes,
  productionDirectory: option('--production-directory'), nativeFrameTimeout: Number(option('--native-frame-timeout', '120')),
  stateComparator: compare, additionalSources: sources});
function loadManifest(directory, side) {
  const base = resolve(directory, side), raw = resolve(base, 'manifest.json');
  if (existsSync(raw)) return JSON.parse(readFileSync(raw));
  const receipt = JSON.parse(readFileSync(resolve(base, 'manifest-archive.json')));
  const compressed = readFileSync(resolve(base, 'manifest.json.gz'));
  const sha = bytes => createHash('sha256').update(bytes).digest('hex');
  assert.equal(compressed.length, receipt.archiveBytes);
  assert.equal(sha(compressed), receipt.archiveSha256);
  const bytes = gunzipSync(compressed);
  assert.equal(bytes.length, receipt.rawBytes);
  assert.equal(sha(bytes), receipt.rawSha256);
  return JSON.parse(bytes);
}
const execution = scenarios.cases.map((scenario, n) => {
  const directory = resolve(output, 'paired', `${String(n).padStart(2, '0')}-${scenario.scenario.id}`);
  const markers = {};
  for (const side of ['original', 'browser']) {
    const seen = {admission: false, full: false, fast: false, poof: false, idle: false, move: false, death: false, retirement: false, explosion43: false};
    try {
      const manifest = loadManifest(directory, side);
      let rows;
      if (side === 'original') rows = manifest.debugObservations.map(row => row.observation?.gameplay);
      else rows = manifest.frames.map(row => row.state);
      let previousDead = false;
      for (const row of rows) {
        if (!row) continue;
        seen.explosion43 ||= side === 'original' ? row.entities?.some(actor => actor.type === 1000 && actor.variant === 43)
          : row.spriteEffects?.some(actor => actor.variant === 43);
        const actors = side === 'original' ? row.entities?.filter(actor => actor.type === 217) : row.room?.npcs.filter(actor => actor.kind === 217 && !actor.removed);
        if (!actors) continue;
        seen.poof ||= side === 'original' ? row.entities.some(actor => actor.type === 1000 && actor.variant === 15 && actor.parentType === 217)
          : row.spriteEffects.some(actor => actor.variant === 15 && actor.spawnerType === 217);
        const phase = side === 'original' ? row.nativePhase?.systemIterations % 2 : row.systemClock.systemCounter % 2;
        if (actors.length) {seen.admission = true; seen.full ||= phase === 1; seen.fast ||= phase === 0;}
        seen.retirement ||= previousDead && !actors.length;
        for (const actor of actors) {
          seen.idle ||= actor.animation === 'Idle' && actor.animationPlaying;
          seen.move ||= actor.animation === 'Move' && actor.animationPlaying;
          const dead = side === 'original' ? actor.removed : actor.dead;
          seen.death ||= dead && actor.hp < 0;
          previousDead ||= dead;
        }
      }
    } catch (error) {seen.error = String(error);}
    markers[side] = seen;
  }
  return {id: scenario.scenario.id, required: scenario.dipRequired, markers,
    status: Object.values(markers).every(side => !side.error && (!scenario.dipRequired ||
      ['admission', 'full', 'fast', 'poof', 'idle', 'move', 'death', 'retirement'].every(key => side[key]))) ? 'pass' : 'fail'};
});
assert.deepEqual(hashes(), before, 'Dip source and comparison must remain frozen');
const report = {schema: 1, status: paired.status === 'pass' && execution.every(row => row.status === 'pass') ? 'pass' : 'fail',
  pairedStatus: paired.status, declaredCaseCount: scenarios.cases.length, randomVectors: scenarios.randomVectors, sourcesStable: true, sources: before, execution,
  scope: 'Both games execute the same public tapes. All actor/HP/AI/geometry/queue/Poop58/43 comparisons remain required; random seeds are unfiltered. Death effects and inherited modifiers/colors remain incomplete, with their actual failures retained.'};
save(resolve(output, 'report.json'), report);
console.log(JSON.stringify(report));
process.exitCode = report.status === 'pass' ? 0 : 1;
