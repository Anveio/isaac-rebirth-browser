// Records a public key tape that walks the production build from gameplay
// start through a planned chain of rooms into the boss room. The recorder is
// closed-loop in the port only (frame-stepped, routing on its own state); the
// resulting tape is then replayed open-loop by both games, and the scenario
// helper rejects any seed whose original does not arrive in the boss room.
//
//   node tools/e2e/boss-route-record.mjs --case CASE.json --rooms 6,7,9,0 [--production-directory dist]
//     [--collect 7] [--arrival boss-room-arrival]
//
// --collect names rooms whose key pickups the player walks over before leaving
// them; a key-locked door on the route is then pushed open with a held key.
// --arrival names the final checkpoint (shop routes use `shop-arrival`).
// --idle FRAMES holds no keys for FRAMES after the last doorway settles, before
// the arrival checkpoint: entering through a door places the player exactly,
// and the idle lets its velocity decay, so games that left the previous room
// on different frames meet in the same state.
// --from names the scripted prefix label (default gameplay-start) after which
// closed-loop routing begins; the 60-frame settle applies only at gameplay-start.
// --fight FRAMES then fights the room clear within FRAMES (lib/route.mjs
// fightKeys) and parks the player in the bottom-left corner (`corner`).
// --exit FRAMES walks through --via X,Y onto the boss-room trapdoor and waits
// out the stage transition (`next-stage`). --arrival '' skips that checkpoint.
//
// CASE.json is a paired case (pairedTape up to gameplay-start, initialSeed,
// bootEpochSeed). Prints { status, actions, checkpoints, frames, rooms } on stdout.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { chromium } from '@playwright/test';
import { browserLaunchOptions } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { flag } from './lib/cli.mjs';
import { doorKeys, fightKeys, routingView, toward, waypoint } from './lib/route.mjs';

const caseData = JSON.parse(readFileSync(flag('--case'), 'utf8'));
const rooms = flag('--rooms').split(',').map(Number);
// Frames allowed per room before the route is abandoned.
const ROOM_FRAMES = Number(flag('--room-frames', '600'));
const collect = new Set((flag('--collect', '') || '').split(',').filter(Boolean).map(Number));
const ARRIVAL = flag('--arrival', 'boss-room-arrival');
const FROM = flag('--from', 'gameplay-start');
const FIGHT = Number(flag('--fight', '0')), EXIT = Number(flag('--exit', '0'));
const VIA = flag('--via') && flag('--via').split(',').map(Number);
const IDLE = Number(flag('--idle', '0'));
// Cells around the boss-award trapdoor (grid 37) and pedestal (grid 97) that
// the corner walk and the exit approach keep clear of.
const AWARD_CELLS = [22, 36, 37, 38, 52, 82, 96, 97, 98, 112];
const avoiding = (grid, cells) => grid && { ...grid, solid: [...grid.solid].map((c, i) => (cells.includes(i) ? '1' : c)).join('') };
// The player re-plans on every other frame; the tape holds keys in between.
const STRIDE = 2;
const namedKeys = { Return: 'Enter', Up: 'ArrowUp', Down: 'ArrowDown', Left: 'ArrowLeft', Right: 'ArrowRight', space: 'Space' };
// DOM codes back to the tape's X11 names.
const tapeKey = { KeyW: 'w', KeyA: 'a', KeyS: 's', KeyD: 'd', ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right' };

// Walk toward a key pickup, routing through fireplaces; a fire standing in the
// next cell is shot out along its axis instead of walked into.
function collectKeys(state, key) {
  const next = waypoint(state.openGrid, state.player, key);
  const fire = state.fires.find(f => Math.abs(f.x - next.x) < 20 && Math.abs(f.y - next.y) < 20);
  if (!fire) return toward(state.player, next, 4);
  const dx = fire.x - state.player.x, dy = fire.y - state.player.y;
  if (Math.abs(dx) > Math.abs(dy)) return Math.abs(dy) > 6 ? [dy < 0 ? 'KeyW' : 'KeyS'] : [dx < 0 ? 'ArrowLeft' : 'ArrowRight'];
  return Math.abs(dx) > 6 ? [dx < 0 ? 'KeyA' : 'KeyD'] : [dy < 0 ? 'ArrowUp' : 'ArrowDown'];
}

const server = await startProductionServer({ directory: flag('--production-directory') });
const browser = await chromium.launch({ ...browserLaunchOptions(), headless: true });
const actions = [], checkpoints = {};
let frames = 0, status = 'fail', reason;
try {
  const page = await browser.newPage({ viewport: { width: 960, height: 540 } });
  await page.goto(server.url + '/?manual=1');
  await page.waitForFunction(() => ['true', 'failed'].includes(document.body.dataset.ready));
  await page.evaluate(({ initialSeed, bootEpochSeed }) => {
    window.game.pause(); window.game.reset(initialSeed, { bootEpochSeed, trace: true });
  }, caseData);
  const start = caseData.pairedTape.findIndex(action => action.label === FROM);
  assert.ok(start >= 0, `The case tape must reach ${FROM}`);
  // A labelled step marker is itself replayed; captures are no-ops here.
  for (const action of caseData.pairedTape.slice(0, start + 1)) {
    if (action.type === 'step') await page.evaluate(count => window.game.step(count), action.frames);
    else if (action.type === 'key') await page.keyboard[action.down ? 'down' : 'up'](namedKeys[action.key] || action.key);
  }
  // Each capture also records where the port stood, for divergence reports.
  const checkpoint = label => {
    actions.push({ type: 'capture', label });
    checkpoints[label] = { room: state.room, player: [state.player.x, state.player.y] };
  };
  const view = async () => routingView(JSON.parse(await page.evaluate(() => window.game.snapshotJson())));
  const held = new Set();
  const hold = async keys => {
    for (const code of [...held]) if (!keys.includes(code)) {
      held.delete(code); await page.keyboard.up(code); actions.push({ type: 'key', key: tapeKey[code], down: false });
    }
    for (const code of keys) if (!held.has(code)) {
      held.add(code); await page.keyboard.down(code); actions.push({ type: 'key', key: tapeKey[code], down: true });
    }
  };
  const step = async count => {
    await page.evaluate(n => window.game.step(n), count);
    frames += count;
    const last = actions.at(-1);
    if (last?.type === 'step') last.frames += count; else actions.push({ type: 'step', frames: count });
  };
  // The paired preamble settles 60 frames before any route input.
  if (FROM === 'gameplay-start') await step(60);
  let state = await view();
  assert.equal(state.room, rooms[0], `Route must start in room ${rooms[0]}, not ${state.room}`);
  for (let leg = 1; leg < rooms.length; leg++) {
    const target = rooms[leg];
    let budget = ROOM_FRAMES;
    while (state.room !== target) {
      if (budget <= 0 && process.env.BOSS_ROUTE_DEBUG) writeFileSync(process.env.BOSS_ROUTE_DEBUG, await page.evaluate(() => window.game.snapshotJson()));
      if (budget <= 0) throw new Error(`No way from room ${state.room} to room ${target} in ${ROOM_FRAMES} frames`);
      const door = state.doors.find(d => d.target === target);
      if (!door) throw new Error(`Room ${state.room} has no door to room ${target}`);
      const key = collect.has(state.room) && state.pickups.find(p => p.variant === 30);
      const usable = /^open/i.test(door.state) || (door.state === 'KeyClosed' && state.keys > 0);
      await hold(state.transition || !state.player.controls ? []
        : key ? collectKeys(state, key)
        // A cleared boss room's open trapdoor must not be walked over.
        : !usable ? [] : doorKeys(state.trapdoor ? avoiding(state.grid, AWARD_CELLS) : state.grid, state.player, door));
      if (process.env.BOSS_ROUTE_DEBUG && budget % 30 === 0) process.stderr.write(`${JSON.stringify({ room: state.room, budget, player: state.player, door, keys: [...held], transition: state.transition })}\n`);
      await step(STRIDE); budget -= STRIDE;
      // Route checkpoints let both games' positions be compared on the way.
      state = await view();
      if (frames % 20 === 0) checkpoint(`route-${frames}`);
    }
  }
  // Settle out of the doorway transition before the arrival checkpoint.
  await hold([]);
  do { await step(1); state = await view(); } while (state.transition);
  if (IDLE) { await step(IDLE); state = await view(); }
  if (ARRIVAL) checkpoint(ARRIVAL);
  const walk = async (goal, cells, budget, label) => {
    for (let left = budget; Math.hypot(goal.x - state.player.x, goal.y - state.player.y) > 6; left -= STRIDE) {
      if (left <= 0) throw new Error(`${label}: no way to (${goal.x}, ${goal.y}) in ${budget} frames`);
      await hold(toward(state.player, waypoint(avoiding(state.grid, cells), state.player, goal), 4));
      await step(STRIDE); state = await view();
    }
  };
  if (FIGHT) {
    const memory = { orbit: 1 };
    for (let left = FIGHT; !(state.clear && !state.foes.length); left -= STRIDE) {
      if (left <= 0) throw new Error(`Room not cleared in ${FIGHT} fight frames`);
      await hold(state.player.controls ? fightKeys(state, memory) : []);
      await step(STRIDE); state = await view();
      if (frames % 20 === 0) checkpoint(`fight-${frames}`);
    }
    checkpoint('clear');
    await walk({ x: 80, y: 400 }, AWARD_CELLS, 600, 'corner walk');
    // Pressing into the corner makes the parked position independent of the fight.
    await hold(['KeyS', 'KeyA']); await step(60); await hold([]); await step(1);
    state = await view();
    checkpoint('corner');
  }
  if (EXIT) {
    if (VIA) await walk({ x: VIA[0], y: VIA[1] }, AWARD_CELLS, EXIT, 'exit via');
    const index = state.trapdoor?.gridIndex;
    if (index === undefined) throw new Error('The cleared boss room has no trapdoor');
    const door = { x: 40 + index % 15 * 40, y: 120 + Math.floor(index / 15) * 40 };
    const pedestal = AWARD_CELLS.filter(cell => cell > 60);
    for (let left = EXIT; state.stageTransition === 0; left -= STRIDE) {
      if (left <= 0) throw new Error(`No trapdoor entry in ${EXIT} frames`);
      await hold(toward(state.player, waypoint(avoiding(state.grid, pedestal), state.player, door), 2));
      await step(STRIDE); state = await view();
    }
    await hold([]);
    for (let left = 1200; state.stageTransition !== 0 || !state.player.controls; left--) {
      if (left <= 0) throw new Error('The stage transition did not finish in 1200 frames');
      await step(1); state = await view();
      if (frames % 4 === 0) checkpoint(`stage-transition-${frames}`);
    }
    checkpoint('next-stage');
  }
  status = 'pass';
} catch (error) {
  reason = String(error.message || error);
} finally {
  await browser.close();
  await server.close();
}
process.stdout.write(JSON.stringify({ status, reason, actions, checkpoints, frames, rooms }) + '\n');
process.exitCode = status === 'pass' ? 0 : 1;
