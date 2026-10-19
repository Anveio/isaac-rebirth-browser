// Boss-room exit and next-floor arrival (Basement II, or Caves I after a
// descent fixture) from a fresh original/public-input production pair
// (tools/original/stage-exit-scenarios.py). Each game fights with its own
// keys before `exit-start`; only the shared tail is graded:
// the award trapdoor (TrapDoor 5711a0) and boss pedestal, the
// StageTransition 5e38b0 states and counter, the player's Trapdoor/Appear
// clips, visibility and controls, the stage/room the transition admits and
// the health and coin predicates the next floor's generation reads.
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
assert.match(gate.build?.sha256 ?? '', /^[a-f0-9]{64}$/, 'The pair must identify its actual production artifact');
for (const [phase, build] of [['fight', replay.stageExit?.recordedBuilds?.fight],
  ['exit', replay.stageExit?.recordedBuilds?.exit], ['replay', replay.portRecordedBuild]])
  assert.equal(build, gate.build.sha256, `The ${phase} recording must match the actual paired artifact`);
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
  // Each fight also leaves its own health, which the next floor's generation
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
// The next floor's generation reads the departing Player's health and coins
// (floor.rs): full health admits the Challenge and Sacrifice rooms, a half
// heart without soul hearts the bedrooms, more than four coins an even
// floor's Arcade. Each game fights with its own keys, so the raw values stay
// diagnostics; the predicates the generator reads must agree. They are read
// at the re-anchor, after the fight and before the pickup-free exit walk.
const generation = (red, maxRed, soul, coins) => ({ fullHealth: maxRed <= red + soul,
  lowHealth: (soul <= 0 && red <= 1) || (soul <= 2 && maxRed === 0), arcadeCoins: coins > 4 });
{
  const source = observations.get('re-anchor'), frame = frames.get('re-anchor');
  const player = source?.gameplay?.players?.[0], coins = source?.carriedItems?.players?.[0]?.inventory?.coins;
  const actual = frame?.state;
  check('re-anchor', 'generation.inputs',
    player && Number.isInteger(coins) ? generation(player.redHp, player.maxRedHp, player.soulHp, coins) : 'unobserved',
    actual ? generation(actual.player.hp, actual.player.maxHp, actual.player.soulHp, actual.inventory.coins) : 'unobserved');
  diagnostic('re-anchor', 'coins', coins ?? null, actual?.inventory.coins ?? null);
}
// A case predating `arrivedStage` is a Basement I exit.
check('next-stage', 'arrivedStage', replay.stageExit?.arrivedStage ?? 2, finalStage);
// The authored active fixture must retain a positive carried-item contract
// even on replay; an old bank lacking it cannot silently pass the new gate.
const expectedActiveId = replay.stageExit?.carriedActiveItemId;
if (replay.stageExit?.fixture === 'Y9QL8QZM-larry-active')
  assert.equal(expectedActiveId, 56, 'The active-item fixture must declare its authored Lemon Mishap');
if (expectedActiveId !== undefined) {
  assert.ok(Number.isInteger(expectedActiveId) && expectedActiveId > 0 && expectedActiveId <= 346,
    'Expected carried item must be a positive authored collectible ID');
  for (const [label, stage] of [['re-anchor', 1], ['next-stage', 2]]) {
    assert.ok(labels.includes(label), `Carried item requires the public ${label} checkpoint`);
    const observation = observations.get(label), frame = frames.get(label);
    const items = observation?.carriedItems, gameplay = observation?.gameplay;
    assert.equal(items?.status, 'pass', `${label} requires actual bounded Native item evidence`);
    assert.equal(items.memoryWritten, false);
    assert.equal(items.executableMappingsRead, false);
    assert.equal(items.nativeBinarySha256, gate.sourceBinarySha256);
    assert.equal(items.gamePresent, true);
    assert.ok(Array.isArray(items.players) && items.players.length === 1,
      `${label} requires one actual Native Player item owner`);
    assert.equal(gameplay?.status, 'pass');
    const original = items.players[0], actual = frame?.state?.playerItems;
    assert.ok(actual, `${label} requires the actual Browser item owner`);
    const history = actual.activeChargeHistory;
    assert.ok(history && typeof history === 'object' && !Array.isArray(history)
      && history.unowned && typeof history.unowned === 'object' && !Array.isArray(history.unowned),
    `${label} requires actual Browser active-word ownership history`);
    for (const word of ['activeId', 'charge', 'reserve'])
      check(label, `carriedActive.browserOwned.${word}`, true, !Object.hasOwn(history.unowned, word));
    for (const field of ['activeId', 'activeCharge', 'reserveCharge']) {
      assert.ok(Number.isInteger(original[field]), `${label} missing Native ${field}`);
      assert.ok(Number.isInteger(actual[field]), `${label} missing Browser ${field}`);
    }
    check(label, 'carriedActive.nativeId', expectedActiveId, original.activeId);
    check(label, 'carriedActive.browserId', expectedActiveId, actual.activeId);
    //Both fields are actual i32 reads in the existing item observer and
    //actual public WASM state, never reconstructed from the expected ID.
    check(label, 'carriedActive.charge', original.activeCharge, actual.activeCharge);
    check(label, 'carriedActive.reserve', original.reserveCharge, actual.reserveCharge);
    check(label, 'carriedActive.nativeStage', stage, gameplay.level.stage);
    check(label, 'carriedActive.browserStage', stage, frame.state.stage);
    check(label, 'carriedActive.nativeTransitionSettled', 0, gameplay.stageTransition.state);
    check(label, 'carriedActive.browserTransitionSettled', 0, frame.state.stageTransition.state);
  }
  check('next-stage', 'carriedActive.nativeDescentObserved', true,
    labels.some(label => observations.get(label)?.gameplay?.stageTransition?.state > 0));
  check('next-stage', 'carriedActive.browserDescentObserved', true,
    labels.some(label => frames.get(label)?.state?.stageTransition?.state > 0));
}

// With `--gl-trace`, each room the port entered (the shared route, the boss
// room, the re-anchor's previous room and the boss room again, then the next
// floor) must equal an original wall cache draw, in entry order
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
// The original logs each Level::Init seed; the last one is the next floor's.
check('next-stage', 'stageSeed', native.observedState.stageSeed, frames.get('next-stage').state.stageSeed);
const report = { schema: 1, status: checks.every(item => item.status === 'pass') ? 'pass' : 'fail',
  scope: 'Shared tail from exit-start: boss award trapdoor and pedestal, StageTransition state/counter, player Trapdoor/Appear clip, visibility and controls, admitted stage, stage seed and room, and the health and coin predicates the next floor generation reads. Declared carried active identity, charge and reserve require bounded Native reads and settled stage1→2 descent. Per-game fights are not graded; MT trapdoor seeds are diagnostics',
  buildSha256: gate.build.sha256, sourceBinarySha256: gate.sourceBinarySha256,
  originalManifestSha256: hash(originalPath), browserManifestSha256: hash(browserPath),
  caseSha256: hash(resolve(directory, 'case.json')), checks, diagnostics };
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, fields: checks.length, failures: checks.filter(item => item.status === 'fail').slice(0, 20) }));
process.exitCode = report.status === 'pass' ? 0 : 1;
