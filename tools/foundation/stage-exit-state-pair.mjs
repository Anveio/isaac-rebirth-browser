// Boss-room exit and Basement II arrival from a fresh original/public-input
// production pair (tools/original/stage-exit-scenarios.py). Each game fights
// with its own keys before `exit-start`; only the shared tail is graded:
// the award trapdoor (TrapDoor 5711a0) and boss pedestal, the
// StageTransition 5e38b0 states and counter, the player's Trapdoor/Appear
// clips, visibility and controls, and the stage/room the transition admits.
// The trapdoor's spawn seed is an MT draw on each game's fight-dependent
// stream and stays a diagnostic.
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
const originalPath = resolve(directory, 'original/manifest.json');
const browserPath = resolve(directory, 'browser/manifest.json');
const native = load(originalPath), browser = load(browserPath);
const gate = load(resolve(dirname(directory), 'report.json'));
const replay = load(resolve(directory, 'case.json'));
assert.equal(browser.status, 'pass', 'Actual production DOM/WASM capture must pass');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
// `exit-start` labels a step; the graded captures follow it.
const marker = replay.pairedTape.findIndex(action => action.label === 'exit-start');
const labels = replay.pairedTape.slice(marker + 1).filter(action => action.type === 'capture').map(action => action.label);
assert.ok(marker >= 0 && labels.includes('next-stage'), 'The tape must declare exit-start and next-stage');
const checks = [], diagnostics = [];
const row = (list, label, field, expected, actual) => list.push({ label, field, expected, actual,
  status: JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
const check = (...args) => row(checks, ...args), diagnostic = (...args) => row(diagnostics, ...args);
const CLIPS = { Closed: 'closed', 'Open Animation': 'openAnimation' };
const PLAYER_CLIPS = ['Trapdoor', 'Appear'];
let finalStage;
for (const label of labels) {
  const source = observations.get(label)?.gameplay, candidate = frames.get(label);
  assert.ok(candidate, `Missing production checkpoint ${label}`);
  if (source?.status !== 'pass') { check(label, 'observation', 'pass', source?.status ?? 'missing'); continue; }
  assert.equal(source.memoryWritten, false);
  assert.equal(source.nativeBinarySha256, gate.sourceBinarySha256);
  const state = candidate.state, player = source.players[0];
  finalStage = source.level.stage;
  check(label, 'stage', source.level.stage, state.stage);
  check(label, 'stageTransition.state', source.stageTransition.state, state.stageTransition.state);
  check(label, 'stageTransition.counter', source.stageTransition.counter, state.stageTransition.counter);
  check(label, 'room', source.room.descriptorIndex, state.currentRoom);
  check(label, 'room.clear', source.room.combatClear, state.room.combatClear);
  check(label, 'player.visible', player.visible, state.player.visible);
  check(label, 'player.controls', player.controlsEnabled, state.player.controlsEnabled);
  const clip = PLAYER_CLIPS.includes(player.animation) ? player.animation : null;
  const actualClip = PLAYER_CLIPS.includes(state.playerAnimation.current) ? state.playerAnimation.current : null;
  check(label, 'player.clip', clip, actualClip);
  if (clip && clip === actualClip) diagnostic(label, 'player.clipFrame', Math.floor(player.animationFrame), Math.floor(state.playerAnimation.frame));
  const trapdoor = source.grid.find(cell => cell.type === 17), actual = state.room.trapdoor;
  check(label, 'trapdoor.index', trapdoor?.index ?? null, actual?.gridIndex ?? null);
  if (trapdoor && actual) {
    check(label, 'trapdoor.state', trapdoor.state, actual.state);
    check(label, 'trapdoor.clip', CLIPS[trapdoor.animation] ?? trapdoor.animation, actual.clip);
    check(label, 'trapdoor.frame', Math.floor(trapdoor.animationFrame), Math.floor(actual.frame));
    check(label, 'trapdoor.playing', trapdoor.animationPlaying, actual.playing);
    diagnostic(label, 'trapdoor.spawnSeed', trapdoor.spawnSeed, actual.spawnSeed);
  }
  const pedestals = source.entities.filter(entity => entity.type === 5 && entity.variant === 100 && !entity.removed)
    .map(entity => [entity.subtype, entity.position]);
  const actualPedestals = state.room.pickups.filter(pickup => pickup.variant === 100 && !pickup.removed)
    .map(pickup => [pickup.subtype, [pickup.position.x, pickup.position.y]]);
  check(label, 'pedestals', pedestals, actualPedestals);
}
check('next-stage', 'arrivedStage', 2, finalStage);
const report = { schema: 1, status: checks.every(item => item.status === 'pass') ? 'pass' : 'fail',
  scope: 'Shared tail from exit-start: boss award trapdoor and pedestal, StageTransition state/counter, player Trapdoor/Appear clip, visibility and controls, admitted stage and room. Per-game fights are not graded; MT trapdoor seeds are diagnostics',
  buildSha256: gate.build.sha256, sourceBinarySha256: gate.sourceBinarySha256,
  originalManifestSha256: hash(originalPath), browserManifestSha256: hash(browserPath),
  caseSha256: hash(resolve(directory, 'case.json')), checks, diagnostics };
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, fields: checks.length, failures: checks.filter(item => item.status === 'fail').slice(0, 20) }));
process.exitCode = report.status === 'pass' ? 0 : 1;
