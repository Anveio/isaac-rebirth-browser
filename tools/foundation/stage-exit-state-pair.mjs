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
import { manifestPath, readJsonEvidence } from './browser-evidence.mjs';
import { compareWallAssemblies, originalWallAssemblies } from './wall-quads.mjs';

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
  // Each fight leaves its own corner position; the shared tail's trapdoor
  // entry frame (and so the StageTransition counter) follows from it.
  diagnostic(label, 'player.position', [player.position[0] * 0.65 + 32, player.position[1] * 0.65 - 46],
    [state.player.x, state.player.y]);
  // Each fight also leaves its own health, which Basement II's generation
  // reads (a Challenge room needs full health).
  diagnostic(label, 'player.health', [player.redHp, player.maxRedHp, player.soulHp],
    [state.player.hp, state.player.maxHp, state.player.soulHp]);
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
    // A boss's tail poop can sit on the pedestal cell; each game's own fight
    // decides whether it is intact, broken or absent there (fix_award_pos).
    const width = state.room.grid.width, cell = actual.gridIndex + 4 * width;
    const sourceCell = source.grid.find(item => item.index === cell), actualCell = state.room.grid.cells[cell]?.entity;
    diagnostic(label, 'pedestalCell', sourceCell ? [sourceCell.type, sourceCell.state] : null,
      actualCell ? [actualCell.kind, actualCell.state] : null);
  }
  const pedestals = source.entities.filter(entity => entity.type === 5 && entity.variant === 100 && !entity.removed)
    .map(entity => [entity.subtype, entity.position]);
  const actualPedestals = state.room.pickups.filter(pickup => pickup.variant === 100 && !pickup.removed)
    .map(pickup => [pickup.subtype, [pickup.position.x, pickup.position.y]]);
  check(label, 'pedestals', pedestals, actualPedestals);
}
check('next-stage', 'arrivedStage', 2, finalStage);
// With `--gl-trace`, each room the port entered (the shared route, the boss
// room, the re-anchor's previous room and the boss room again, then Basement
// II) must equal an original wall cache draw, in entry order
// (wall-quads.mjs). A room entry is a change of room index or of its walls:
// the stage number turns over a few captures before the next level's start
// room loads, so it does not mark an entry.
if (native.graphicsTrace) {
  const start = browser.frames.findIndex(frame => frame.label === 'gameplay-start');
  const rooms = [];
  for (const frame of browser.frames.slice(start)) {
    const key = `${frame.state.currentRoom}:${JSON.stringify(frame.state.room.appearance?.wallQuads)}`;
    if (rooms.at(-1)?.key !== key)
      rooms.push({ key, label: frame.label, room: frame.state.currentRoom, appearance: frame.state.room.appearance });
  }
  checks.push(...compareWallAssemblies(originalWallAssemblies(native.graphicsTrace.path), rooms));
}
// The original logs each Level::Init seed; the last one is Basement II's.
check('next-stage', 'stageSeed', native.observedState.stageSeed, frames.get('next-stage').state.stageSeed);
const report = { schema: 1, status: checks.every(item => item.status === 'pass') ? 'pass' : 'fail',
  scope: 'Shared tail from exit-start: boss award trapdoor and pedestal, StageTransition state/counter, player Trapdoor/Appear clip, visibility and controls, admitted stage, stage seed and room. Per-game fights are not graded; MT trapdoor seeds are diagnostics',
  buildSha256: gate.build.sha256, sourceBinarySha256: gate.sourceBinarySha256,
  originalManifestSha256: hash(originalPath), browserManifestSha256: hash(browserPath),
  caseSha256: hash(resolve(directory, 'case.json')), checks, diagnostics };
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, fields: checks.length, failures: checks.filter(item => item.status === 'fail').slice(0, 20) }));
process.exitCode = report.status === 'pass' ? 0 : 1;
