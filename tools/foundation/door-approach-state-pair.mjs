// Off-centre door approaches, from a fresh original/public-input production
// pair (tools/original/door-approach-scenarios.py). Every frame of the walk
// into the doorway and back out grades the room and the player's exact
// position and velocity, so a different doorway capture area, wall or corner
// collision or slide shows as the first diverging frame. The frame each game
// changes room is reported. A walk the original's wall stops is graded the
// same way: the frames pin where the doorway stops letting Isaac through.
import assert from 'node:assert/strict';
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
const native = readJsonEvidence(manifestPath(directory, 'original'));
const browser = readJsonEvidence(manifestPath(directory, 'browser'));
const gate = load(resolve(dirname(directory), 'report.json'));
const replay = load(resolve(directory, 'case.json'));
assert.equal(browser.status, 'pass', 'Actual production DOM/WASM capture must pass');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
const marker = replay.pairedTape.findIndex(action => action.label === 'gameplay-start');
const labels = replay.pairedTape.slice(marker + 1).filter(action => action.type === 'capture').map(action => action.label);
assert.ok(marker >= 0 && labels.includes('returned'), 'The tape must declare gameplay-start and returned');
const checks = [];
const check = (label, field, expected, actual) => checks.push({ label, field, expected, actual,
  status: JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
const rooms = { original: [], port: [] };
const trajectory = [];
for (const label of labels) {
  const source = observations.get(label), candidate = frames.get(label);
  assert.ok(candidate, `Missing production checkpoint ${label}`);
  if (source?.status !== 'pass') { check(label, 'observation', 'pass', source?.status ?? 'missing'); continue; }
  assert.equal(source.contents.memoryWritten, false);
  assert.equal(source.contents.nativeBinarySha256, gate.sourceBinarySha256);
  const state = candidate.state, actor = source.contents.players[0];
  const expected = [actor.position[0] * .65 + 32, actor.position[1] * .65 - 46, actor.velocity[0] * .65, actor.velocity[1] * .65];
  const actual = [state.player.x, state.player.y, state.player.vx, state.player.vy];
  check(label, 'screen', 'playing', state.screen);
  check(label, 'room', source.floor.currentDescriptorIndex, state.currentRoom);
  check(label, 'motion', expected, actual);
  rooms.original.push(source.floor.currentDescriptorIndex); rooms.port.push(state.currentRoom);
  trajectory.push({ label, room: [source.floor.currentDescriptorIndex, state.currentRoom],
    original: [actor.position, actor.velocity], port: [[state.player.x, state.player.y], [state.player.vx, state.player.vy]] });
}
// The room changes once on the way in and once on the way out.
const changes = list => list.flatMap((room, index) => index && room !== list[index - 1] ? [labels[index]] : []);
const transitions = { original: changes(rooms.original), port: changes(rooms.port) };
check('route', 'transitionFrames', transitions.original, transitions.port);
const failed = checks.filter(row => row.status === 'fail');
const result = { schema: 1, status: failed.length ? 'fail' : 'pass', case: replay.scenario.id,
  doorApproach: replay.doorApproach, transitions, firstDivergence: failed[0]?.label ?? null, checks, trajectory };
writeFileSync(option('--output'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({ status: result.status, checks: checks.length, failures: failed.length, transitions }));
process.exitCode = result.status === 'pass' ? 0 : 1;
