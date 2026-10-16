// Cracked bedroom door under one bomb, from a fresh original/public-input
// production pair (tools/original/bedroom-door-scenarios.py). Door::SetRoomTypes
// 565b00 gives a doorway into room type 18/19 the cracked variant3. A blast
// (TryBlowOpen 564870) damages it to state4 and scatters Random(5)+6 Wood
// Particles 1000.27 (Effect update 45dc20), which fly, land and leave for the
// background paint pass. Every capture after gameplay-start grades the door
// object, the room and every live wood particle; the original must reach the
// damaged state, or the case did not exercise the door.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { manifestPath, readJsonEvidence } from './browser-evidence.mjs';

const option = name => {
  const at = process.argv.indexOf(name);
  assert.ok(at >= 0 && process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
const directory = resolve(option('--case'));
const load = path => JSON.parse(readFileSync(path, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const originalPath = manifestPath(directory, 'original');
const browserPath = manifestPath(directory, 'browser');
const native = readJsonEvidence(originalPath), browser = readJsonEvidence(browserPath);
const gate = load(resolve(dirname(directory), 'report.json'));
const replay = load(resolve(directory, 'case.json'));
assert.equal(browser.status, 'pass', 'Actual production DOM/WASM capture must pass');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
const marker = replay.pairedTape.findIndex(action => action.label === 'gameplay-start');
const labels = replay.pairedTape.slice(marker + 1).filter(action => action.type === 'capture').map(action => action.label);
assert.ok(marker >= 0 && labels.includes('pushed'), 'The tape must declare gameplay-start and pushed');
const slot = replay.bedroomDoor.slot;
const checks = [];
const check = (label, field, expected, actual) => checks.push({ label, field, expected, actual,
  status: JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
const f32 = value => Math.fround(value);
const vector = value => [f32(value[0] ?? value.x), f32(value[1] ?? value.y)];
let damaged = false, scattered = 0;
for (const label of labels) {
  const source = observations.get(label)?.gameplay, candidate = frames.get(label);
  assert.ok(candidate, `Missing production checkpoint ${label}`);
  if (source?.status !== 'pass') { check(label, 'observation', 'pass', source?.status ?? 'missing'); continue; }
  assert.equal(source.memoryWritten, false);
  assert.equal(source.nativeBinarySha256, gate.sourceBinarySha256);
  const state = candidate.state, phase = source.nativePhase.systemIterations % 2;
  check(label, 'room', source.room.descriptorIndex, state.currentRoom);
  // The starting room is 13x7: door slots 0..3 sit at cells 60, 7, 74, 127.
  const index = [60, 7, 74, 127][slot];
  const door = source.grid.find(cell => cell.index === index), actual = state.room.grid?.cells[index]?.entity;
  check(label, 'door', door ? [door.type, door.variant, door.state, door.collisionClass] : null,
    actual ? [actual.kind, actual.variant, actual.state, actual.collisionClass] : null);
  if (door?.variant === 3 && door.state === 4) damaged = true;
  const wood = source.entities.filter(entity => entity.type === 1000 && entity.variant === 27 && !entity.removed);
  scattered = Math.max(scattered, wood.length);
  const effects = state.spriteEffects.filter(effect => effect.variant === 27 && effect.active && !effect.removed);
  check(label, 'wood.seeds', wood.map(entity => entity.spawnSeed).sort((a, b) => a - b),
    effects.map(effect => effect.spawnSeed).sort((a, b) => a - b));
  for (const entity of wood) {
    const effect = effects.find(candidate => candidate.spawnSeed === entity.spawnSeed);
    if (!effect) continue;
    const field = name => `wood${entity.spawnSeed}.${name}`;
    check(label, field('position'), vector(entity.position), vector(phase === 0 ? effect.presentedPosition : effect.position));
    check(label, field('velocity'), vector(entity.velocity), vector(effect.velocity));
    check(label, field('height'), f32(entity.entityRenderOffset[1]), f32(effect.renderOffset.y));
    check(label, field('clip'), [entity.animation, Math.floor(entity.animationFrame), entity.animationPlaying],
      [effect.sprite.animation, Math.floor(effect.sprite.frame), effect.sprite.playing]);
    check(label, field('gridCollisionClass'), entity.gridCollisionClass, effect.gridCollisionClass);
  }
}
check('pushed', 'door.damagedByBlast', true, damaged);
check('pushed', 'wood.scatteredSixToTen', true, scattered >= 6 && scattered <= 10);
const report = { schema: 1, status: checks.every(item => item.status === 'pass') ? 'pass' : 'fail',
  scope: 'Cracked bedroom doorway (variant3) under the starting bomb: door object variant/state/collision, room, and every live Wood Particle27 seed, position, velocity, height, clip and grid class at each capture',
  buildSha256: gate.build.sha256, sourceBinarySha256: gate.sourceBinarySha256,
  originalManifestSha256: hash(originalPath), browserManifestSha256: hash(browserPath),
  caseSha256: hash(resolve(directory, 'case.json')), checks };
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, fields: checks.length, failures: checks.filter(item => item.status === 'fail').slice(0, 20) }));
process.exitCode = report.status === 'pass' ? 0 : 1;
