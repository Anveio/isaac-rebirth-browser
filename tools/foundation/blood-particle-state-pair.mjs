// Compare actual original fields against production snapshots from public tapes.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const option = name => { const at = process.argv.indexOf(name); assert.ok(at >= 0 && process.argv[at + 1]); return process.argv[at + 1]; };
const directory = resolve(option('--case'));
const load = path => JSON.parse(readFileSync(path, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const nativePath = resolve(directory, 'original/manifest.json'), browserPath = resolve(directory, 'browser/manifest.json');
const native = load(nativePath), browser = load(browserPath), tape = load(resolve(directory, 'case.json'));
const pair = load(resolve(directory, 'report.json'));
assert.equal(native.status, 'pass'); assert.equal(browser.status, 'pass');
assert.equal(pair.original.status, 'pass'); assert.equal(pair.browser.status, 'pass');
const originals = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
const actuals = new Map(browser.frames.map(row => [row.label, row.state]));
// Normalize only source f32 values: physical/presented P/V, radii, mass,
// friction, HP/damage, sprite clock/speed/scale/tint/offset and particle floats.
// IDs, seeds, sequence, flags, enums and counters always retain integer equality.
const f32 = value => {
  assert.ok(typeof value === 'number' && Number.isFinite(value), 'Required f32 field must be a finite number');
  return Math.fround(value);
};
const vector = value => [f32(value.x ?? value[0]), f32(value.y ?? value[1])];
const failures = [], uninitialized = [], execution = Object.fromEntries(tape.bloodParticleRequired.map(name => [name, false]));
let checks = 0, presentations = 0, started = false;
const check = (label, field, expected, actual) => {
  assert.notEqual(expected, undefined, field); assert.notEqual(actual, undefined, field); checks++;
  if (JSON.stringify(expected) !== JSON.stringify(actual)) failures.push({label, field, expected, actual});
};
for (const action of tape.pairedTape) {
  if (action.type !== 'capture') continue;
  const label = action.label, observed = originals.get(label), state = actuals.get(label);
  assert.ok(observed && state, `Missing actual paired capture${label}`);
  if (label === 'gameplay-start') started = true;
  if (!started) continue;
  assert.equal(observed.status, 'pass'); assert.equal(observed.gameplay.status, 'pass');
  assert.equal(observed.gameplay.memoryWritten, false); presentations++;
  const source = observed.gameplay, phase = source.nativePhase.systemIterations % 2;
  check(label, 'room.identity', source.room.descriptorIndex, state.currentRoom);
  check(label, 'system.phase', phase, state.systemClock.systemCounter % 2);
  const original = source.entities.filter(actor => actor.type === 1000 && actor.variant === 5 && actor.active && !actor.removed);
  const port = state.spriteEffects.filter(actor => actor.variant === 5 && actor.active && !actor.removed);
  check(label, 'particle.orderedSpawnSeeds', original.map(actor => actor.spawnSeed), port.map(actor => actor.spawnSeed));
  if (original.length) execution['native-particles'] = true;
  if (port.length) execution['browser-particles'] = true;
  if (source.entities.some(actor => actor.type === 244 && actor.removed)) execution['native-death'] = true;
  if (state.room.npcs.some(actor => actor.kind === 244 && actor.dead)) execution['browser-death'] = true;
  for (const actor of original) {
    const actual = port.find(candidate => candidate.spawnSeed === actor.spawnSeed);
    if (!actual) continue; // Presence/ordered seeds above remain a required fail.
    const particle = actual.bloodParticle; assert.ok(particle, 'Admitted5 needs its real controller owner');
    const identity = state.actorScheduler.identities.find(row => row.actor.kind === 'effect' && row.actor.id === actual.id);
    for (const [field, expected, value] of [
      ['sequenceId', actor.sequenceId, identity?.sequenceId], ['subtype', actor.subtype, actual.subtype],
      ['birthGameUpdate', actor.effectBirthGameUpdate, actual.birthGameUpdate], ['parent', actor.effectParent, actual.parent],
      ['spawnerType', actor.parentType, actual.spawnerType], ['spawnerVariant', actor.parentVariant, actual.spawnerVariant],
      ['flags', actor.entityFlags, actual.flags], ['visible', actor.visible, actual.visible],
      ['gridClass', actor.gridCollisionClass, actual.gridCollisionClass], ['entityClass', actor.entityCollisionClass, actual.entityCollisionClass],
      ['actorRng', actor.actorRngState, actual.actorRng.state], ['state', actor.bloodParticleState, particle.state],
      ['actorRngWords', actor.actorRngWords, [actual.actorRng.state, actual.actorRng.rightA, actual.actorRng.left, actual.actorRng.rightB]],
      ['raw7d8', actor.effectRaw7d8, actual.raw7d8], ['timeToLive', actor.effectTimeToLive, actual.timeToLive],
      ['raw7f4', actor.effectRaw7f4, actual.raw7f4], ['depth', actor.renderOrderOffset, actual.renderOrderOffset],
      ['animation', actor.animation, actual.sprite.animation], ['playing', actor.animationPlaying, actual.sprite.playing],
      ['flipX', actor.spriteFlipX, actual.sprite.flipX], ['flipY', actor.spriteFlipY, actual.sprite.flipY],
      ['gridContact', actor.collidedWithGrid, particle.gridContact],
    ]) check(label, `particle${actor.spawnSeed}.${field}`, expected, value);
    for (const [field, expected, value] of [
      ['position', vector(actor.position), vector(phase === 0 ? actual.presentedPosition : actual.position)],
      ['velocity', vector(actor.velocity), vector(actual.velocity)], ['renderOffset', vector(actor.entityRenderOffset), vector(actual.renderOffset)],
      ['gridNormal', vector(actor.gridCollisionNormal), vector(particle.gridNormal)],
      ['previousVelocity', vector(actor.preGridCollisionVelocity), vector(particle.previousVelocity)],
      ['scale', vector(actor.spriteScale), vector(actual.sprite.scale)],
      ['tint', actor.spriteTint.map(f32), actual.sprite.tint.map(f32)], ['offsets', actor.spriteOffsets.map(f32), actual.sprite.offsets.map(f32)],
    ]) check(label, `particle${actor.spawnSeed}.${field}`, expected, value);
    for (const [field, expected, value] of [
      ['radius', actor.collisionRadius, actual.collisionRadius], ['mass', actor.collisionMass, actual.collisionMass],
      ['damage', actor.collisionDamage, actual.collisionDamage], ['hp', actor.hp, actual.hp], ['maxHp', actor.maxHp, actual.maxHp],
      ['friction', actor.friction, actual.currentFriction], ['animationFrame', actor.animationFrame, actual.sprite.frame],
      ['animationSpeed', actor.animationSpeed, actual.sprite.speed], ['rotation', actor.bloodParticleLayer0Rotation, particle.layerRotation],
      ['raw7f8', actor.effectRaw7f8, actual.raw7f8],
    ]) check(label, `particle${actor.spawnSeed}.${field}`, f32(expected), f32(value));
    check(label, `particle${actor.spawnSeed}.freshInheritedColor`, [1,1,1,1,0,0,0,0,0,0,0], actor.inheritedColor.map(f32));
    if (actor.bloodParticleState !== 0) {
      for (const [field, expected, value] of [
        ['height', actor.bloodParticleHeight, particle.height], ['verticalVelocity', actor.bloodParticleVerticalVelocity, particle.verticalVelocity],
        ['gravity', actor.bloodParticleGravity, particle.gravity],
      ]) check(label, `particle${actor.spawnSeed}.${field}`, f32(expected), f32(value));
    } else uninitialized.push({label, seed: actor.spawnSeed, sourceBits: actor.bloodParticleControllerBits,
      scope: 'Storage before first actual5 Update is retained, not accepted as initialized controller values.'});
  }
}
for (const [field, actual] of Object.entries(execution)) check('execution', field, true, actual);
const report = {schema: 1, status: failures.length ? 'fail' : 'pass', presentations, checks, failures, execution, uninitialized,
  sourceManifest: {path: nativePath, sha256: hash(nativePath)}, browserManifest: {path: browserPath, sha256: hash(browserPath)},
  verifierSha256: hash('tools/foundation/blood-particle-state-pair.mjs'),
  scope: 'Actual244death/5presence and matched real seed bodies, sequence, cached spawner/NULL parent, initialized controller, full/fast motion, color and clips are required. No original value enters WASM.',
  unaccepted: ['Monstro2/42/77 chain', 'Status/champion/perk/color modifiers', 'Full background render-array ordering/persistence', 'Controller storage before first Update', 'Full-frame pixels and whole sharedMT']};
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({status: report.status, presentations, checks, failures: failures.length, execution}));
process.exitCode = report.status === 'pass' ? 0 : 1;
