// Exit mid-run and Continue, from a fresh original/public-input production
// pair (tools/original/continue-run-scenarios.py). Every gameplay capture
// grades the room, the player's exact motion, controls, health and
// consumables, the Level's previous room and the visited set. At Continue the
// current room's NPCs and pickups are graded as multisets: Room.Save keeps
// pickups, NPCs come back from the layout (Room.Init's revisit branch), so the
// first post-Continue capture (cont-60, before any NPC moved) pins every
// respawned NPC's position and health. Owned collectibles are graded from the
// original's Player+E1C counts.
//
// Every actual Native NPC at a declared gameplay capture is required,
// including families absent from the Browser. Admission gaps stay failures.
// Status, friction, color and complete shared-MT parity remain unqualified.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { manifestPath, readJsonEvidence } from './browser-evidence.mjs';

// The public route owner validates its own complete recipes without entropy,
// floor generation or either game. Imports have no comparator CLI side effects.
export function validateContinueRunScenarios(bank, { allowSubset = false } = {}) {
  assert.equal(typeof allowSubset, 'boolean');
  const helper = fileURLToPath(new URL('../original/continue-run-scenarios.py', import.meta.url));
  return JSON.parse(execFileSync('python3', [helper, '--validate-bank', '-', ...(allowSubset ? ['--allow-subset'] : [])],
    { input: JSON.stringify(bank), encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 }));
}

const option = name => {
  const at = process.argv.indexOf(name);
  assert.ok(at >= 0 && process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
function main() {
  const directory = resolve(option('--case'));
  const load = path => JSON.parse(readFileSync(path, 'utf8'));
  const native = readJsonEvidence(manifestPath(directory, 'original'));
  const browser = readJsonEvidence(manifestPath(directory, 'browser'));
  const gate = load(resolve(dirname(directory), 'report.json'));
  const replay = load(resolve(directory, 'case.json'));
  const declaration = validateContinueRunScenarios({ schema: 1, cases: [replay] }, { allowSubset: true });
  assert.equal(browser.status, 'pass', 'Actual production DOM/WASM capture must pass');
  const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
  const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
  const marker = replay.pairedTape.findIndex(action => action.label === 'gameplay-start');
  const labels = replay.pairedTape.slice(marker).filter(action => action.type === 'capture').map(action => action.label);
  assert.ok(marker >= 0 && labels.includes('cont-60'), 'The tape must declare gameplay-start and a Continue');
  const checks = [], diagnostics = [];
  const same = (expected, actual) => JSON.stringify(expected) === JSON.stringify(actual);
  const check = (label, field, expected, actual) => checks.push({ label, field, expected, actual,
    status: same(expected, actual) ? 'pass' : 'fail' });
  const vector = value => Array.isArray(value) ? value : [value.x, value.y];
  const isNpc = type => type >= 10 && type < 1000;
  const npcTuple = npc => JSON.stringify([npc.type, npc.variant, npc.subtype, vector(npc.position), npc.hp]);
  const pickupTuple = pickup => JSON.stringify([pickup.variant, pickup.subtype, vector(pickup.position)]);
  const sorted = list => [...list].sort();
  const pedestals = {};
  for (const label of labels) {
    const source = observations.get(label), candidate = frames.get(label);
    assert.ok(candidate, `Missing production checkpoint ${label}`);
    if (source?.status !== 'pass') { check(label, 'observation', 'pass', source?.status ?? 'missing'); continue; }
    assert.equal(source.contents.memoryWritten, false);
    assert.equal(source.contents.nativeBinarySha256, gate.sourceBinarySha256);
    const state = candidate.state, floor = source.floor, actor = source.contents.players[0];
    check(label, 'screen', 'playing', state.screen);
    check(label, 'room', floor.currentDescriptorIndex, state.currentRoom);
    check(label, 'previousRoom', floor.previousGridIndex, state.savedRoomEntry?.previousRoom ?? null);
    check(label, 'motion', [actor.position[0] * .65 + 32, actor.position[1] * .65 - 46, actor.velocity[0] * .65, actor.velocity[1] * .65],
      [state.player.x, state.player.y, state.player.vx, state.player.vy]);
    check(label, 'controlsEnabled', actor.controlsEnabled, state.player.controlsEnabled);
    check(label, 'hearts', [actor.redHp, actor.maxRedHp], [state.player.hp, state.player.maxHp]);
    check(label, 'consumables', [actor.coinInventory, actor.bombInventory, actor.keyInventory],
      [state.inventory.coins, state.inventory.bombs, state.inventory.keys]);
    check(label, 'collectibles', actor.collectibleCounts,
      Object.fromEntries((state.playerItems?.ownedCounts ?? []).flatMap((count, id) => count ? [[String(id), count]] : [])));
    check(label, 'visited', floor.descriptors.filter(room => room.visited).map(room => room.index), state.visitedRooms);
    check(label, 'combatClear', floor.descriptors[floor.currentDescriptorIndex].combatClear, state.room.combatClear);
    const nativeNpcs = source.contents.entities.filter(entity => isNpc(entity.type) && !entity.removed);
    const portNpcs = (state.room.npcs ?? []).filter(npc => !npc.removed).map(npc => ({ ...npc, type: npc.kind }));
    check(label, 'npcCount', nativeNpcs.length, portNpcs.length);
    // Positions are exact only before NPC AI draws diverge: at cont-60 every
    // NPC stands on its layout cell. Later captures grade identities only.
    if (label === 'cont-60' || label === 'second-cont-60')
      check(label, 'npcs', sorted(nativeNpcs.map(npcTuple)), sorted(portNpcs.map(npcTuple)));
    else check(label, 'npcIdentities', sorted(nativeNpcs.map(npc => JSON.stringify([npc.type, npc.variant]))),
      sorted(portNpcs.map(npc => JSON.stringify([npc.type, npc.variant]))));
    const nativePickups = source.contents.entities.filter(entity => entity.type === 5 && !entity.removed);
    const portPickups = (state.room.pickups ?? []).filter(pickup => !pickup.removed);
    check(label, 'pickups', sorted(nativePickups.map(pickupTuple)), sorted(portPickups.map(pickupTuple)));
    for (const pickup of nativePickups.filter(pickup => pickup.variant === 100))
      pedestals[floor.currentDescriptorIndex] = pickup.subtype;
  }
  // The Exit really happened in the original, and Continue resumed the same room.
  const before = observations.get('before-quit'), after = observations.get('cont-60');
  check('route', 'continuedRoom', before?.floor.currentDescriptorIndex, after?.floor.currentDescriptorIndex);
  check('route', 'continuedControlsDisabled', false, after?.contents.players[0].controlsEnabled);
  const result = { schema: 1, status: checks.every(row => row.status === 'pass') ? 'pass' : 'fail',
    case: replay.scenario.id, continueRun: replay.continueRun, declaration,
    scope: { required: 'Every actual Native NPC at every declared gameplay capture; missing Browser actors fail count and multiset checks',
      unqualified: ['NPC status', 'NPC friction', 'NPC color', 'complete shared MT'] },
    pedestals, checks, diagnostics };
  writeFileSync(option('--output'), JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify({ status: result.status, checks: checks.length,
    failures: checks.filter(row => row.status === 'fail').length, diagnostics: diagnostics.length }));
  process.exitCode = result.status === 'pass' ? 0 : 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
