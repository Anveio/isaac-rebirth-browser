// Records a public key tape that walks the production build from gameplay
// start through a planned chain of rooms into the boss room. The recorder is
// closed-loop in the port only (frame-stepped, routing on its own state); the
// resulting tape is then replayed open-loop by both games, and the scenario
// helper rejects any seed whose original does not arrive in the boss room.
//
//   node tools/e2e/boss-route-record.mjs --case CASE.json --rooms 6,7,9,0 [--production-directory dist]
//     [--collect 7] [--take 3] [--arrival boss-room-arrival]
//
// --collect names rooms whose key pickups the player walks over before leaving
// them; a key-locked door on the route is then pushed open with a held key.
// --take names rooms whose collectible pedestals (5.100 with an item) the
// player walks onto, picking each up, before leaving them.
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
import { flag, hasFlag } from './lib/cli.mjs';
import { doorKeys, fightKeys, routingView, toward, waypoint } from './lib/route.mjs';

const caseData = JSON.parse(readFileSync(flag('--case'), 'utf8'));
const rooms = flag('--rooms').split(',').map(Number);
// Frames allowed per room before the route is abandoned.
const ROOM_FRAMES = Number(flag('--room-frames', '600'));
const collect = new Set((flag('--collect', '') || '').split(',').filter(Boolean).map(Number));
const take = new Set((flag('--take', '') || '').split(',').filter(Boolean).map(Number));
const ARRIVAL = flag('--arrival', 'boss-room-arrival');
const FROM = flag('--from', 'gameplay-start');
const FIGHT = Number(flag('--fight', '0')), EXIT = Number(flag('--exit', '0'));
const VIA = flag('--via') && flag('--via').split(',').map(Number);
// --via-candidates X,Y;X,Y;... (an OS-shuffled order) takes the first one the
// exit can use as its waypoint; the result reports it as `via`.
const VIA_CANDIDATES = (flag('--via-candidates', '') || '').split(';').filter(Boolean).map(item => item.split(',').map(Number));
let chosenVia = VIA ? { x: VIA[0], y: VIA[1] } : null;
const IDLE = Number(flag('--idle', '0'));
// Cells around the boss-award trapdoor (grid 37) and pedestal (grid 97) that
// the corner walk and the exit approach keep clear of.
const AWARD_CELLS = [22, 36, 37, 38, 52, 82, 96, 97, 98, 112];
// --avoid adds cells those walks also keep clear of: obstacles the original
// holds after its own fight (a solid Larry Jr. tail poop) that the port's own
// fight need not leave, so the shared keys take one path in both games.
const AVOID = (flag('--avoid', '') || '').split(',').filter(Boolean).map(Number);
// --route-avoid ROOM:CELL,CELL;ROOM:CELL marks cells the route legs walk
// around in those rooms: an authored slot machine (6.x) the port does not
// spawn yet still blocks the original's player (#645).
const ROUTE_AVOID = new Map((flag('--route-avoid', '') || '').split(';').filter(Boolean)
  .map(item => item.split(':')).map(([room, cells]) => [Number(room), cells.split(',').map(Number)]));
// --hurt: the original's fixture fight ends below full health. Basement II's
// generation reads it (a Challenge room needs full health), so a port fight
// that ends at full health takes one blast of its own bomb before parking.
const HURT = hasFlag('--hurt');
// Boss-room cells whose centre lies within 30 units of one of the port's own
// floor pickups (hearts, coins): a heart taken in one game alone changes the
// health Basement II's generation reads, so the shared walks keep off them.
const pickupCells = state => {
  const cells = [];
  for (let row = 1; row < 8; row++) for (let col = 1; col < 14; col++) {
    if (state.pickups.some(p => p.variant !== 100 && Math.hypot(40 + col * 40 - p.x, 120 + row * 40 - p.y) < 30)) cells.push(row * 15 + col);
  }
  return cells;
};
// The centre of the room grid cell holding a point.
const centreOf = point => ({ x: Math.round((point.x - 40) / 40) * 40 + 40, y: Math.round((point.y - 120) / 40) * 40 + 120 });
const avoiding = (grid, cells) => grid && { ...grid, solid: [...grid.solid].map((c, i) => (cells.includes(i) ? '1' : c)).join('') };
// The player re-plans on every other frame; the tape holds keys in between.
const STRIDE = 2;
const namedKeys = { Return: 'Enter', Up: 'ArrowUp', Down: 'ArrowDown', Left: 'ArrowLeft', Right: 'ArrowRight', space: 'Space' };
// DOM codes back to the tape's X11 names.
const tapeKey = { KeyW: 'w', KeyA: 'a', KeyS: 's', KeyD: 'd', KeyE: 'e', ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right' };

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
    window.game.pause(); window.game.reset(initialSeed, { bootEpochSeed, trace: true, audioPreferences: {sfxVolume: 0, musicVolume: 0}, fullscreenPreferences: {fullscreen: false} });
  }, caseData);
  const start = caseData.pairedTape.findIndex(action => action.label === FROM);
  assert.ok(start >= 0, `The case tape must reach ${FROM}`);
  // A labelled step marker is itself replayed; captures are no-ops here.
  for (const action of caseData.pairedTape.slice(0, start + 1)) {
    if (action.type === 'step') await page.evaluate(count => { window.game.step(count); }, action.frames);
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
    await page.evaluate(n => { window.game.step(n); }, count);
    frames += count;
    const last = actions.at(-1);
    if (last?.type === 'step') last.frames += count; else actions.push({ type: 'step', frames: count });
  };
  // The paired preamble settles 60 frames before any route input.
  if (FROM === 'gameplay-start') await step(60);
  let state = await view();
  assert.equal(state.room, rooms[0], `Route must start in room ${rooms[0]}, not ${state.room}`);
  // Cells beside an obstacle only one game holds: the original's own (--avoid)
  // or a solid poop the port's own fight left. A turn there carries the player
  // about ten units on toward it, so only that game's player meets it (#649 at
  // a waypoint; mid-walk below an original tail poop and beside a port poop
  // alike). The shared boss-room walks keep one cell clear of them wherever
  // the room still leaves a way to the goal.
  const margin = sources => [...new Set(sources.flatMap(cell => [cell - 15, cell + 15, cell - 1, cell + 1]))]
    .filter(cell => cell % 15 > 0 && cell % 15 < 14 && cell > 14 && cell < 120 && !sources.includes(cell));
  const pointCell = point => Math.round((point.y - 120) / 40) * 15 + Math.round((point.x - 40) / 40);
  const reaches = (grid, from, to) => {
    const seen = new Set([from]), queue = [from];
    while (queue.length) {
      const cell = queue.shift();
      if (cell === to) return true;
      for (const next of [cell - grid.w, cell + grid.w, cell - 1, cell + 1]) {
        if (next < 0 || next >= grid.solid.length || seen.has(next) || (grid.solid[next] === '1' && next !== to)) continue;
        seen.add(next); queue.push(next);
      }
    }
    return false;
  };
  const clearOf = (cells, goal) => {
    for (const sources of [[...AVOID, ...state.poops], AVOID]) {
      const wide = [...cells, ...margin(sources)];
      if (reaches(avoiding(state.grid, wide), pointCell(state.player), pointCell(goal))) return wide;
    }
    return cells;
  };
  for (let leg = 1; leg < rooms.length; leg++) {
    const target = rooms[leg];
    let budget = ROOM_FRAMES;
    // Each game's fight leaves its own poops, and a diagonal shortcut that
    // grazes a corner solid in one game alone would slow only that game's
    // player: walks in a cleared boss room target the next route cell's
    // centre once the player stands at the current one.
    let centre = centreOf(state.player);
    const centreKeys = (grid, door) => {
      if (Math.hypot(centre.x - state.player.x, centre.y - state.player.y) <= 4) centre = waypoint(grid, state.player, door, 1);
      return centre.x === door.x && centre.y === door.y ? doorKeys(grid, state.player, door) : toward(state.player, centre, 2);
    };
    while (state.room !== target) {
      if (budget <= 0 && process.env.BOSS_ROUTE_DEBUG) writeFileSync(process.env.BOSS_ROUTE_DEBUG, await page.evaluate(() => window.game.snapshotJson()));
      if (budget <= 0) throw new Error(`No way from room ${state.room} to room ${target} in ${ROOM_FRAMES} frames`);
      const door = state.doors.find(d => d.target === target);
      if (!door) throw new Error(`Room ${state.room} has no door to room ${target}`);
      const key = collect.has(state.room) && state.pickups.find(p => p.variant === 30)
        || take.has(state.room) && state.pickups.find(p => p.variant === 100 && p.subtype > 0);
      const usable = /^open/i.test(door.state) || (door.state === 'KeyClosed' && state.keys > 0);
      // A pedestal, taken or not, is solid but not a grid object. The take
      // walk can end past it inside its cell; step out sideways first.
      const cell = (x, y) => Math.round((y - 120) / 40) * state.grid.w + Math.round((x - 40) / 40);
      const pedestals = state.grid ? state.pickups.filter(p => p.variant === 100).map(p => cell(p.x, p.y)) : [];
      const inside = pedestals.includes(state.grid && cell(state.player.x, state.player.y));
      await hold(state.transition || !state.player.controls ? []
        : key ? collectKeys(state, key)
        // A cleared boss room's shared keys step from cell centre to cell
        // centre, off its open trapdoor, until the doorway is next. A centre
        // beside the trapdoor is 40 units from it, outside its entry radius.
        : !usable ? [] : inside ? ['KeyA'] : state.trapdoor
          ? centreKeys(avoiding(state.grid, clearOf([state.trapdoor.gridIndex, ...AVOID, ...pedestals, ...pickupCells(state)], door)), door)
          : doorKeys(avoiding(state.grid, [...pedestals, ...(ROUTE_AVOID.get(state.room) ?? [])]), state.player, door));
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
    // A poop the port's own fight drops in the cell inside an open doorway
    // would wall off the shared re-anchor walk. The port shoots it out from
    // a free side cell before parking; the original's fixture fight is fixed.
    for (const door of state.doors.filter(d => /^open/i.test(d.state))) {
      const cellAt = (x, y) => Math.round((y - 120) / 40) * state.grid.w + Math.round((x - 40) / 40);
      const across = door.side === 'up' || door.side === 'down';
      const inward = { up: [0, 40], down: [0, -40], left: [40, 0], right: [-40, 0] }[door.side];
      const inner = { x: door.x + inward[0], y: door.y + inward[1] };
      for (const sign of [-1, 1]) {
        if (state.grid.solid[cellAt(inner.x, inner.y)] !== '1') break;
        const stand = across ? { x: inner.x + sign * 40, y: inner.y } : { x: inner.x, y: inner.y + sign * 40 };
        const blocked = [...AWARD_CELLS, ...pickupCells(state), ...state.pickups.filter(p => p.variant === 100).map(p => cellAt(p.x, p.y))];
        if (state.grid.solid[cellAt(stand.x, stand.y)] === '1' || blocked.includes(cellAt(stand.x, stand.y))) continue;
        await walk(stand, blocked, 600, 'doorway poop');
        const aim = across ? (sign < 0 ? 'ArrowRight' : 'ArrowLeft') : (sign < 0 ? 'ArrowDown' : 'ArrowUp');
        for (let left = 600; state.grid.solid[cellAt(inner.x, inner.y)] === '1'; left -= STRIDE) {
          if (left <= 0) throw new Error(`Doorway poop at (${inner.x}, ${inner.y}) still stands`);
          await hold([aim]); await step(STRIDE); state = await view();
        }
        await hold([]);
      }
    }
    if (HURT && state.player.full) {
      // A cell two or more from the walls and three from the trapdoor and
      // pedestal, so the blast reaches neither nor a wall's secret door.
      const far = (cell, from) => Math.hypot((cell % 15 - from % 15) * 40, (Math.floor(cell / 15) - Math.floor(from / 15)) * 40) >= 120;
      const spots = [];
      for (let r = 2; r <= 6; r++) for (let c = 2; c <= 12; c++) {
        const cell = r * 15 + c;
        if (state.grid.solid[cell] !== '1' && far(cell, state.trapdoor.gridIndex) && far(cell, 97)) spots.push(cell);
      }
      const near = cell => Math.hypot(40 + cell % 15 * 40 - state.player.x, 120 + Math.floor(cell / 15) * 40 - state.player.y);
      const spot = spots.sort((a, b) => near(a) - near(b))[0];
      if (spot === undefined) throw new Error('No open cell for the health-matching bomb');
      await walk({ x: 40 + spot % 15 * 40, y: 120 + Math.floor(spot / 15) * 40 }, [...AWARD_CELLS, ...pickupCells(state)], 600, 'bomb spot');
      await hold([]); await step(10);
      await hold(['KeyE']); await step(1); await hold([]);
      for (let left = 300; state.player.full; left--) {
        if (left <= 0) throw new Error('The health-matching bomb did not hurt the player');
        await step(1); state = await view();
      }
      await step(30); state = await view();
    }
    // An authored corner object (a Monstro room's poop in cell 106) parks the
    // player in the nearest open bottom-left cell, pressed against it.
    const blocked = [...AWARD_CELLS, ...pickupCells(state)];
    const park = [106, 107, 91, 108, 92, 76].find(cell => state.grid.solid[cell] !== '1' && !blocked.includes(cell));
    if (park === undefined) throw new Error('No open bottom-left cell to park in');
    // Port-only poops can also cut the parking corner off from a doorway the
    // shared re-anchor walk leaves through: EV9BP18Q's Basement II room keeps
    // a fresh poop beside the doorway's inner cell while the original's own
    // poop and a heart cell (--avoid) close its other side. Shoot out the
    // first poop on the cheapest corner-to-doorway route until it is open.
    const w = state.grid.w, cellAt = (x, y) => Math.round((y - 120) / 40) * w + Math.round((x - 40) / 40);
    const keepOff = () => new Set([state.trapdoor?.gridIndex, ...AVOID, ...pickupCells(state),
      ...state.pickups.filter(p => p.variant === 100).map(p => cellAt(p.x, p.y))]);
    // 0-1 BFS over the room's interior: open cells cost 0, solid poops 1.
    const corridor = (from, to) => {
      const off = keepOff(), cost = new Map([[from, 0]]), previous = new Map([[from, -1]]), deque = [from];
      while (deque.length) {
        const cell = deque.shift();
        if (cell === to) break;
        for (const next of [cell - w, cell + w, cell - 1, cell + 1]) {
          const c = next % w, r = Math.floor(next / w);
          if (c < 1 || c > w - 2 || r < 1 || r > state.grid.h - 2 || (off.has(next) && next !== to)) continue;
          const poop = state.poops.includes(next);
          if (!poop && state.grid.solid[next] === '1') continue;
          const total = cost.get(cell) + (poop ? 1 : 0);
          if (cost.has(next) && cost.get(next) <= total) continue;
          cost.set(next, total); previous.set(next, cell);
          if (poop) deque.push(next); else deque.unshift(next);
        }
      }
      if (!previous.has(to)) return null;
      const path = [];
      for (let cell = to; cell !== -1; cell = previous.get(cell)) path.unshift(cell);
      return path;
    };
    for (const door of state.doors.filter(d => /^open/i.test(d.state))) {
      const inward = { up: [0, 40], down: [0, -40], left: [40, 0], right: [-40, 0] }[door.side];
      const inner = cellAt(door.x + inward[0], door.y + inward[1]);
      for (let tries = 0; ; tries++) {
        const path = corridor(park, inner);
        if (!path) throw new Error(`No corner route to the ${door.side} doorway`);
        const at = path.findIndex(cell => state.poops.includes(cell));
        if (at < 0) break;
        if (tries === 6) throw new Error(`Corner route to the ${door.side} doorway still crosses poops`);
        const poop = path[at], stand = path[at - 1];
        await walk({ x: 40 + stand % w * 40, y: 120 + Math.floor(stand / w) * 40 }, blocked, 600, 'route poop');
        const aim = { [1]: 'ArrowRight', [-1]: 'ArrowLeft', [w]: 'ArrowDown', [-w]: 'ArrowUp' }[poop - stand];
        for (let left = 600; state.grid.solid[poop] === '1'; left -= STRIDE) {
          if (left <= 0) throw new Error(`Route poop at cell ${poop} still stands`);
          await hold([aim]); await step(STRIDE); state = await view();
        }
        await hold([]);
      }
    }
    await walk({ x: 40 + park % 15 * 40, y: 120 + Math.floor(park / 15) * 40 }, blocked, 600, 'corner walk');
    // Pressing into the corner makes the parked position independent of the fight.
    await hold(['KeyS', 'KeyA']); await step(60); await hold([]); await step(1);
    state = await view();
    checkpoint('corner');
  }
  if (EXIT) {
    // The shared exit keys step from cell centre to cell centre, as the
    // cleared boss room's door walk does above.
    const centreWalk = async (goal, cells, arrived, failure) => {
      let target = centreOf(state.player);
      for (let left = EXIT; !arrived(); left -= STRIDE) {
        if (left <= 0) throw new Error(failure);
        if (Math.hypot(target.x - state.player.x, target.y - state.player.y) <= 4) target = waypoint(avoiding(state.grid, clearOf([...cells, ...pickupCells(state)], goal)), state.player, goal, 1);
        await hold(toward(state.player, target, 2));
        await step(STRIDE); state = await view();
      }
    };
    // The turn at a waypoint overshoots about ten units: one on or beside an
    // interior cell the port's own fight left solid would touch it in the
    // port alone (#649). Walls are in both games.
    const cellOfPoint = point => Math.round((point.y - 120) / 40) * 15 + Math.round((point.x - 40) / 40);
    const interior = next => next % 15 > 0 && next % 15 < 14 && next > 14 && next < 120;
    const usable = point => {
      const cell = cellOfPoint(point);
      return state.grid.solid[cell] !== '1'
        && ![cell - 15, cell + 15, cell - 1, cell + 1].some(next => interior(next) && state.grid.solid[next] === '1');
    };
    if (!chosenVia && VIA_CANDIDATES.length) {
      const found = VIA_CANDIDATES.find(([x, y]) => usable({ x, y }));
      if (!found) throw new Error('exit via: no candidate waypoint clear of solid cells');
      chosenVia = { x: found[0], y: found[1] };
    }
    if (chosenVia) {
      const via = chosenVia;
      if (!usable(via)) throw new Error(`exit via: (${via.x}, ${via.y}) is beside a solid cell`);
      await centreWalk(via, [...AWARD_CELLS, ...AVOID], () => Math.hypot(via.x - state.player.x, via.y - state.player.y) <= 6,
        `exit via: no way to (${via.x}, ${via.y}) in ${EXIT} frames`);
    }
    const index = state.trapdoor?.gridIndex;
    if (index === undefined) throw new Error('The cleared boss room has no trapdoor');
    const door = { x: 40 + index % 15 * 40, y: 120 + Math.floor(index / 15) * 40 };
    const pedestal = [...AWARD_CELLS.filter(cell => cell > 60), ...AVOID];
    // Re-entering the room re-closes the trapdoor, which opens only once no
    // player stands within 50 units (TrapDoor 571225). Step two cells away
    // from it and wait for the open clip before the approach.
    if (state.trapdoor.state !== 1) {
      const clear = cell => cell % 15 > 0 && cell % 15 < 14 && cell > 14 && cell < 120
        && state.grid.solid[cell] !== '1' && ![...AWARD_CELLS, ...AVOID].includes(cell);
      const away = [index - 2, index + 2, index + 30, index - 30].find(clear);
      if (away === undefined) throw new Error('No cell two away from the closed trapdoor');
      const spot = { x: 40 + away % 15 * 40, y: 120 + Math.floor(away / 15) * 40 };
      await centreWalk(spot, [...AWARD_CELLS, ...AVOID], () => Math.hypot(spot.x - state.player.x, spot.y - state.player.y) <= 4,
        `No way off the closed trapdoor in ${EXIT} frames`);
      await hold([]);
      for (let left = 300; state.trapdoor.state !== 1; left--) {
        if (left <= 0) throw new Error('The trapdoor did not reopen in 300 frames');
        await step(1); state = await view();
      }
    }
    await centreWalk(door, pedestal, () => state.stageTransition !== 0, `No trapdoor entry in ${EXIT} frames`);
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
process.stdout.write(JSON.stringify({ status, reason, actions, checkpoints, frames, rooms, build: server.build.sha256,
  via: chosenVia && [chosenVia.x, chosenVia.y] }) + '\n');
process.exitCode = status === 'pass' ? 0 : 1;
