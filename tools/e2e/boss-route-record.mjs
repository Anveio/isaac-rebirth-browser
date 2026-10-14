// Records a public key tape that walks the production build from gameplay
// start through a planned chain of rooms into the boss room. The recorder is
// closed-loop in the port only (frame-stepped, routing on its own state); the
// resulting tape is then replayed open-loop by both games, and the scenario
// helper rejects any seed whose original does not arrive in the boss room.
//
//   node tools/e2e/boss-route-record.mjs --case CASE.json --rooms 6,7,9,0 [--production-directory dist]
//
// CASE.json is a paired case (pairedTape up to gameplay-start, initialSeed,
// bootEpochSeed). Prints { status, actions, checkpoints, frames, rooms } on stdout.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { chromium } from '@playwright/test';
import { browserLaunchOptions } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { flag } from './lib/cli.mjs';
import { doorKeys, routingView } from './lib/route.mjs';

const caseData = JSON.parse(readFileSync(flag('--case'), 'utf8'));
const rooms = flag('--rooms').split(',').map(Number);
// Frames allowed per room before the route is abandoned.
const ROOM_FRAMES = Number(flag('--room-frames', '600'));
// The player re-plans on every other frame; the tape holds keys in between.
const STRIDE = 2;
const namedKeys = { Return: 'Enter', Up: 'ArrowUp', Down: 'ArrowDown', Left: 'ArrowLeft', Right: 'ArrowRight', space: 'Space' };
// DOM codes back to the tape's X11 names.
const tapeKey = { KeyW: 'w', KeyA: 'a', KeyS: 's', KeyD: 'd' };

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
  const start = caseData.pairedTape.findIndex(action => action.label === 'gameplay-start');
  assert.ok(start >= 0, 'The case tape must reach gameplay-start');
  for (const action of caseData.pairedTape.slice(0, start)) {
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
  await step(60);
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
      await hold(state.transition || !state.player.controls || !/^open/i.test(door.state) ? [] : doorKeys(state.grid, state.player, door));
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
  checkpoint('boss-room-arrival');
  status = 'pass';
} catch (error) {
  reason = String(error.message || error);
} finally {
  await browser.close();
  await server.close();
}
process.stdout.write(JSON.stringify({ status, reason, actions, checkpoints, frames, rooms }) + '\n');
process.exitCode = status === 'pass' ? 0 : 1;
