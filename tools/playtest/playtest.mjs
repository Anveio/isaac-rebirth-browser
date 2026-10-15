// Self-playtest of a playtest bundle (tools/playtest/bundle.mjs): boot it the
// way the Artifact page boots, go through the menus into a fresh OS-random run
// and play for a few minutes with real keyboard input. This is a smoke/play
// harness for publishing decisions, not a parity test or a live e2e suite.
import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import { createServer } from 'node:http';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { extname, join, normalize, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { chromium } from 'playwright';
import { PNG } from 'pngjs';
import { browserLaunchOptions } from '../foundation/browser.mjs';
import { doorKeys as routeDoorKeys, toward as routeToward, waypoint as routeWaypoint } from '../e2e/lib/route.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const { values } = parseArgs({ options: {
  bundle: { type: 'string', default: join(ROOT, '.cache/playtest/bundle') },
  out: { type: 'string' },
  seconds: { type: 'string', default: '180' },
  'initial-seed': { type: 'string' },
  'run-seed': { type: 'string' },
  'boot-epoch-seed': { type: 'string' },
  headed: { type: 'boolean', default: false },
  boss: { type: 'boolean', default: false },
} });
const BUNDLE = resolve(values.bundle);
const PLAY_MS = Number(values.seconds) * 1000;
const OUT = resolve(values.out || join(ROOT, '.cache/playtest/runs', new Date().toISOString().replace(/[:.]/g, '-')));
await mkdir(OUT, { recursive: true });
const manifest = JSON.parse(await readFile(join(BUNDLE, 'manifest.json'), 'utf8'));

// Mirror the Artifact host: the page fragment is wrapped in a document skeleton
// and every other file is served next to it.
const TYPES = { '.js': 'text/javascript', '.wasm': 'application/wasm', '.ogg': 'audio/ogg', '.html': 'text/html' };
const page = await readFile(join(BUNDLE, 'index.html'), 'utf8');
const server = createServer(async (request, response) => {
  const path = decodeURIComponent(new URL(request.url, 'http://local').pathname);
  if (path === '/favicon.ico') { response.writeHead(204); return response.end(); }
  if (path === '/' || path === '/index.html') {
    response.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    return response.end(`<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><style>:root{color-scheme:light;padding:env(safe-area-inset-top) 0 env(safe-area-inset-bottom)}body{margin:0;font:14px system-ui}img{max-width:100%}[hidden]{display:none!important}</style></head><body>${page}</body></html>`);
  }
  const file = normalize(join(BUNDLE, path));
  if (!file.startsWith(BUNDLE + sep) || file.endsWith('manifest.json')) { response.writeHead(404); return response.end(); }
  try {
    const bytes = await readFile(file);
    response.writeHead(200, { 'Content-Type': TYPES[extname(file)] || 'application/octet-stream' });
    response.end(bytes);
  } catch { response.writeHead(404); response.end(); }
});
await new Promise(done => server.listen(0, '127.0.0.1', done));
const url = `http://127.0.0.1:${server.address().port}/`;

const issues = { pageErrors: [], consoleErrors: [], failedRequests: [], failures: [], warnings: [] };
const fail = (message) => { issues.failures.push(message); console.error(`FAIL ${message}`); };
const warn = (message) => { issues.warnings.push(message); console.warn(`WARN ${message}`); };
const log = (message) => console.log(`[${((Date.now() - started) / 1000).toFixed(1).padStart(6)}s] ${message}`);
const started = Date.now();
const screenshots = [];
const timeline = [];

// Cloud sessions ship a Playwright Chromium outside the pinned revision.
const launch = browserLaunchOptions({ profile: 'swiftshader' });
if (!launch.executablePath && existsSync('/opt/pw-browsers/chromium')) launch.executablePath = '/opt/pw-browsers/chromium';
const browser = await chromium.launch({ ...launch, headless: !values.headed });
const browserVersion = browser.version();
const context = await browser.newContext({ viewport: { width: 1280, height: 760 } });
const tab = await context.newPage();
tab.on('pageerror', error => { issues.pageErrors.push(String(error.stack || error)); console.error(`pageerror ${error}`); });
tab.on('console', message => { if (message.type() === 'error') { issues.consoleErrors.push(message.text()); console.error(`console.error ${message.text()}`); } });
tab.on('requestfailed', request => issues.failedRequests.push(`${request.url()} ${request.failure()?.errorText}`));
tab.on('response', response => { if (response.status() >= 400) issues.failedRequests.push(`${response.url()} ${response.status()}`); });
await tab.addInitScript(() => {
  const frames = window.__playtestFrames = [];
  let last = performance.now();
  const tick = (now) => { frames.push(now - last); last = now; requestAnimationFrame(tick); };
  requestAnimationFrame(tick);
});

async function shot(label) {
  const bytes = await tab.locator('#game').screenshot();
  const png = PNG.sync.read(bytes);
  let lit = 0; const colors = new Set();
  for (let i = 0; i < png.data.length; i += 16) {
    const [r, g, b] = [png.data[i], png.data[i + 1], png.data[i + 2]];
    if (r + g + b > 30) lit++;
    if (colors.size < 4096) colors.add((r << 16) | (g << 8) | b);
  }
  const litFraction = lit / (png.data.length / 16);
  const file = `${String(screenshots.length).padStart(2, '0')}-${label}.png`;
  await writeFile(join(OUT, file), bytes);
  screenshots.push({ file, label, litFraction: Number(litFraction.toFixed(3)), colors: colors.size, atMs: Date.now() - started });
  return { litFraction, colors: colors.size };
}

// Compact view of the game state, computed in the page.
const observe = () => tab.evaluate(() => {
  const s = window.game.snapshot();
  // Fight what holds the doors shut; fireplaces and similar props do not.
  const npcs = s.room.npcs.filter(n => !n.dead && !n.removed && n.visible && n.hp > 0 && n.canShutDoors);
  return {
    tick: s.tick, screen: s.screen, paused: s.paused, pauseMenu: s.pauseMenu.phase,
    // Player x/y are presentation coordinates; the rest of the room is in world units.
    player: { x: (s.player.x - 32) / 0.65, y: (s.player.y + 46) / 0.65, hp: s.player.hp, maxHp: s.player.maxHp, dead: s.player.dead, controls: s.player.controlsEnabled },
    room: s.currentRoom, visited: s.visitedRooms.length, transition: !!s.roomTransition, clear: s.room.combatClear,
    bounds: { left: s.room.left, right: s.room.right, top: s.room.top, bottom: s.room.bottom },
    npcs: npcs.map(n => ({ x: n.position.x, y: n.position.y, boss: n.boss, kind: n.kind, hp: n.hp, maxHp: n.maxHp })),
    pickups: s.room.pickups.filter(p => !p.collected && !p.removed).map(p => ({ x: p.position.x, y: p.position.y, variant: p.variant })),
    // Solid grid cells (walls, rocks, pits); door cells stay walkable for routing.
    // Authored cells without a grid entity (e.g. pits, selector 3000) still block.
    grid: s.room.grid && { w: s.room.grid.width, h: s.room.grid.height,
      solid: s.room.grid.cells.map(c => ((c.entity ? c.entity.collisionClass !== 0 && c.entity.kind !== 16 : c.selector !== 0) ? 1 : 0)).join('') },
    doors: s.room.appearance.doors.map(d => ({ side: d.side, state: d.state, target: d.targetRoom, x: d.worldPosition.x, y: d.worldPosition.y })),
    shots: (s.enemyProjectiles ?? []).map(t => t.position ?? t).filter(t => Number.isFinite(t?.x) && Number.isFinite(t?.y)).map(t => ({ x: t.x, y: t.y })),
    tears: s.tears.length, bombs: s.bombs.length, explosions: s.explosions.length,
    inventory: s.inventory, items: s.playerItems ? s.playerItems.ownedCounts.reduce((a, b) => a + b, 0) : 0,
    gameOver: !!s.playerDeath?.gameOver, runSeed: s.runSeed, npcKinds: s.room.npcs.filter(n => !n.removed).map(n => `${n.kind}.${n.variant}${n.champion ? "c" + n.champion.color : ""}`), introduction: s.runIntroduction.active, stage: s.stage,
    // Floor layout for --boss: each room's grid index and type (5 = boss).
    rooms: s.floor ? s.floor.descriptors.map(d => [d.gridIndex, d.template.type]) : [],
  };
});

// Tears fly along one axis: an enemy behind rocks needs a firing post in its
// row or column with no solid cell between. Grid cell centres are (c*40+40, r*40+120).
const solidAt = (grid, x, y) => { const c = Math.round((x - 40) / 40), r = Math.round((y - 120) / 40); return c < 0 || r < 0 || c >= grid.w || r >= grid.h || grid.solid[r * grid.w + c] === '1'; };
function lineOfFire(grid, from, to) {
  if (!grid) return true;
  const horizontal = Math.abs(to.x - from.x) > Math.abs(to.y - from.y);
  if (Math.abs(horizontal ? to.y - from.y : to.x - from.x) > 20) return false;
  const steps = Math.ceil(Math.hypot(to.x - from.x, to.y - from.y) / 20);
  for (let i = 1; i < steps; i++) if (solidAt(grid, from.x + (to.x - from.x) * i / steps, from.y + (to.y - from.y) * i / steps)) return false;
  return true;
}
function firingPost(grid, from, target) {
  if (!grid) return null;
  const posts = [];
  for (let k = 2; k <= 6; k++) for (const [ox, oy] of [[k, 0], [-k, 0], [0, k], [0, -k]]) {
    const post = { x: target.x + ox * 40, y: target.y + oy * 40 };
    if (!solidAt(grid, post.x, post.y) && lineOfFire(grid, post, target)) posts.push(post);
  }
  return posts.sort((a, b) => Math.hypot(a.x - from.x, a.y - from.y) - Math.hypot(b.x - from.x, b.y - from.y))[0] ?? null;
}

const held = new Set();
async function hold(keys) {
  for (const key of [...held]) if (!keys.includes(key)) { await tab.keyboard.up(key); held.delete(key); }
  for (const key of keys) if (!held.has(key)) { await tab.keyboard.down(key); held.add(key); }
}
const release = () => hold([]);
async function tap(key, ms = 120) { await tab.keyboard.down(key); await tab.waitForTimeout(ms); await tab.keyboard.up(key); }

let exitCode = 0;
const stats = { roomsEntered: new Set(), returnedToStart: false, leftStart: false, maxTears: 0, bombsPlaced: 0, explosionsSeen: 0,
  pickupsCollected: 0, itemsGained: 0, coinsMax: 0, enemiesSeen: 0, enemyRoomsCleared: 0, bossesKilled: 0, bossHpLeast: null, damageTaken: 0, deaths: 0, pauses: 0,
  distance: 0, stalls: 0, maxFrozenMs: 0, stages: new Set() };
try {
  // Boot.
  await tab.goto(url);
  await tab.waitForFunction(() => document.body.dataset.ready === 'true' || document.body.dataset.ready === 'failed', null, { timeout: 180_000 });
  assert.equal(await tab.evaluate(() => document.body.dataset.ready), 'true', `Game failed to boot: ${await tab.locator('#status').textContent()}`);
  log(`booted in ${Date.now() - started}ms`);
  if (values['initial-seed']) {
    // Replay a recorded run: the production boot draws these from the OS clock and RNG.
    await tab.evaluate(([seed, epoch]) => window.game.reset(seed, { bootEpochSeed: epoch }), [Number(values['initial-seed']), Number(values['boot-epoch-seed'] || 0)]);
  }
  const seeds = await tab.evaluate(() => { const s = window.game.snapshot(); return { initialSeed: s.initialSeed, bootEpochSeed: s.bootEpochSeed }; });
  timeline.push({ event: 'boot', atMs: Date.now() - started, ...seeds });
  log(`initial seed ${seeds.initialSeed}, boot epoch seed ${seeds.bootEpochSeed}`);
  if (await tab.locator('#playtest-start').isVisible()) await tab.locator('#playtest-start').click();
  else await tab.locator('#game').click();
  const titleTick = (await observe()).tick;
  await tab.waitForTimeout(1500);
  const title = await observe();
  if (title.tick <= titleTick) fail('Title screen state does not advance in real time');
  log(`title ${title.screen}, ${title.tick - titleTick} ticks in 1.5s`);
  const titleShot = await shot('title');
  if (titleShot.litFraction < 0.05) fail('Title screen renders black');

  // Menus: Enter through title, profile, main menu and character select.
  // With --run-seed, type it on the character screen the way a player does (Tab, code, Enter).
  let seedTyped = !values['run-seed'];
  for (let attempt = 0; attempt < 20; attempt++) {
    const state = await observe();
    if (state.screen === 'playing') break;
    if (state.screen === 'character' && !seedTyped) {
      await tap('Tab'); await tab.waitForTimeout(1800);
      for (const character of values['run-seed'].replace(/\s/g, '').toLowerCase()) { await tap(character, 200); await tab.waitForTimeout(100); }
      await tap('Enter'); await tab.waitForTimeout(1200);
      seedTyped = true;
      continue;
    }
    if (state.screen === 'seed' && seedTyped) await tap('Escape'); else await tap('Enter');
    await tab.waitForTimeout(900);
    if (attempt === 1) await shot('menu');
  }
  let state = await observe();
  if (state.screen !== 'playing') throw new Error(`Menus did not reach a run; stuck on ${state.screen}`);
  if (values['run-seed'] && state.runSeed.replace(/\s/g, '') !== values['run-seed'].replace(/\s/g, '').toUpperCase())
    throw new Error(`Typed run seed ${values['run-seed']} but the run started with ${state.runSeed}`);
  for (let i = 0; i < 50 && (state.introduction || !state.player.controls); i++) { await tab.waitForTimeout(200); state = await observe(); }
  const startRoom = state.room;
  stats.roomsEntered.add(startRoom);
  log(`run started in room ${startRoom} on stage ${state.stage} (seed ${state.runSeed})`);
  timeline.push({ event: 'run', atMs: Date.now() - started, room: startRoom, runSeed: state.runSeed });
  await shot('run-start');

  // Play: fight, collect, explore through doors and return, bomb and pause.
  const playStart = Date.now(), playStartTick = state.tick;
  let previous = state, lastAdvance = Date.now(), lastMoveCheck = { at: Date.now(), x: state.player.x, y: state.player.y };
  let previousRoom = startRoom, cameFromDoor = null, segment = 0, lastPause = Date.now(), blackStreak = 0, nextShot = Date.now() + 20_000;
  const blockedDoors = new Set(), lastVisit = new Map(), visits = new Map(), orbit = { dir: 1, until: 0 };
  let doorGoal = null, wantReturn = false, stuckSince = null, wiggle = null, itemsBefore = state.items;
  const pickupsAtStart = new Map();
  while (Date.now() - playStart < PLAY_MS) {
    state = await observe();
    const now = Date.now();
    // Liveness and input responsiveness.
    if (state.tick > previous.tick) { stats.maxFrozenMs = Math.max(stats.maxFrozenMs, now - lastAdvance); lastAdvance = now; }
    else if (now - lastAdvance > 3000) {
      fail(`Game state frozen for ${now - lastAdvance}ms at tick ${state.tick} in room ${state.room} (run seed ${state.runSeed}, NPCs ${state.npcKinds.join(' ') || 'none'})`);
      timeline.push({ event: 'frozen', atMs: now - started, room: state.room, runSeed: state.runSeed, npcKinds: state.npcKinds, player: state.player });
      await shot('frozen');
      break;
    }
    stats.distance += Math.hypot(state.player.x - previous.player.x, state.player.y - previous.player.y);
    stats.maxTears = Math.max(stats.maxTears, state.tears);
    stats.explosionsSeen += state.explosions > previous.explosions ? 1 : 0;
    stats.coinsMax = Math.max(stats.coinsMax, state.inventory.coins);
    stats.enemiesSeen = Math.max(stats.enemiesSeen, state.npcs.length);
    // Boss progress: the lowest boss HP fraction reached, and kills (a boss room cleared).
    for (const npc of state.npcs.filter(n => n.boss && n.maxHp > 0)) stats.bossHpLeast = Math.min(stats.bossHpLeast ?? 1, Math.round(npc.hp / npc.maxHp * 100) / 100);
    if (previous.npcs.some(n => n.boss) && !state.npcs.some(n => n.boss) && state.room === previous.room && state.clear && !state.player.dead) stats.bossesKilled++;
    stats.stages.add(state.stage);
    if (state.player.hp < previous.player.hp) stats.damageTaken += previous.player.hp - state.player.hp;
    if (state.items > itemsBefore) { stats.itemsGained += state.items - itemsBefore; itemsBefore = state.items; log(`collected an item (${state.items} owned)`); await shot('item'); }
    if (state.pickups.length < previous.pickups.length && state.room === previous.room) stats.pickupsCollected += previous.pickups.length - state.pickups.length;
    if (previous.npcs.length > 0 && state.npcs.length === 0 && state.room === previous.room) { stats.enemyRoomsCleared++; log(`cleared room ${state.room}`); }
    if (state.room !== previousRoom) {
      log(`entered room ${state.room} from ${previousRoom} (${state.npcs.length} enemies, ${state.pickups.length} pickups)`);
      timeline.push({ event: 'room', atMs: now - started, from: previousRoom, room: state.room, runSeed: state.runSeed, npcKinds: state.npcKinds });
      cameFromDoor = previousRoom; previousRoom = state.room; lastVisit.set(state.room, now); visits.set(state.room, (visits.get(state.room) ?? 0) + 1);
      if (state.room !== startRoom) stats.leftStart = true;
      if (state.room === startRoom && stats.leftStart) { stats.returnedToStart = true; wantReturn = false; }
      if (!stats.roomsEntered.has(state.room) && stats.roomsEntered.size === 1) { wantReturn = true; await shot('first-new-room'); }
      stats.roomsEntered.add(state.room);
      if (state.npcs.some(n => n.boss)) await shot('boss');
    }
    if (state.gameOver || state.player.dead) {
      stats.deaths++;
      log('player died; restarting');
      await release(); await shot('death');
      // Let the death animation reach the game-over screen, then Enter through to a new run.
      for (let i = 0; i < 40 && !(await observe()).gameOver; i++) await tab.waitForTimeout(250);
      await shot('game-over');
      for (let i = 0; i < 30; i++) {
        const s = await observe();
        if (s.screen === 'playing' && !s.gameOver && !s.player.dead && s.player.controls) break;
        if (i === 29) throw new Error(`Could not start a new run after dying (screen ${s.screen}, gameOver ${s.gameOver})`);
        if (s.screen === 'seed') await tap('Escape'); else await tap('Enter');
        await tab.waitForTimeout(800);
      }
      state = await observe(); previous = state; previousRoom = state.room; lastAdvance = Date.now();
      stats.roomsEntered.add(state.room); cameFromDoor = null; doorGoal = null;
      log(`new run started in room ${state.room} (seed ${state.runSeed})`);
      timeline.push({ event: 'run', atMs: Date.now() - started, room: state.room, runSeed: state.runSeed });
      continue;
    }
    if (now >= nextShot) {
      nextShot = now + 30_000;
      const image = await shot(`play-${Math.round((now - playStart) / 1000)}s`);
      blackStreak = image.litFraction < 0.03 ? blackStreak + 1 : 0;
      if (blackStreak >= 2) fail('Playing screen rendered black in consecutive screenshots');
    }
    // Pause roughly every 50s: open, wait, check it paused the run, close.
    if (now - lastPause > 50_000 && !state.transition) {
      lastPause = now; await release();
      await tap('Escape'); await tab.waitForTimeout(800);
      const paused = await observe();
      // Escape can land as the player crosses a door; retry after the transition.
      if (!paused.paused && (paused.transition || paused.room !== state.room)) { lastPause = now - 45_000; previous = paused; lastAdvance = Date.now(); continue; }
      if (!paused.paused && paused.pauseMenu === 'hidden') fail(`Escape did not open the pause menu (screen ${paused.screen}, tick ${paused.tick}, room ${paused.room}, controls ${paused.player.controls}, dead ${paused.player.dead})`);
      else { stats.pauses++; await shot('paused'); }
      await tab.keyboard.down('KeyD'); await tab.waitForTimeout(400); await tab.keyboard.up('KeyD');
      const still = await observe();
      if (paused.paused && Math.hypot(still.player.x - paused.player.x, still.player.y - paused.player.y) > 1) fail('Player moved while the game was paused');
      await tap('Escape'); await tab.waitForTimeout(800);
      const resumed = await observe();
      if (resumed.paused) { await tap('Escape'); await tab.waitForTimeout(600); if ((await observe()).paused) fail('Pause menu does not close with Escape'); }
      previous = await observe(); lastAdvance = Date.now();
      continue;
    }
    // Choose keys.
    const p = state.player;
    const keys = [];
    const waypoint = (x, y) => routeWaypoint(state.grid, p, { x, y });
    const toward = (x, y, slack = 8) => keys.push(...routeToward(p, { x, y }, slack));
    segment++;
    if (wiggle && now < wiggle.until) keys.push(...wiggle.keys);
    else if (state.npcs.length) {
      // Fight: face the nearest enemy along its dominant axis, keep distance.
      const target = state.npcs.reduce((a, b) => Math.hypot(a.x - p.x, a.y - p.y) < Math.hypot(b.x - p.x, b.y - p.y) ? a : b);
      const dx = target.x - p.x, dy = target.y - p.y, distance = Math.hypot(dx, dy);
      if (target.boss) {
        // Orbit the boss at ~170 units, aiming along the axis with the larger
        // gap; reverse when a wall or rock is ahead, sidestep nearby shots.
        const r = { x: -dx / (distance || 1), y: -dy / (distance || 1) };
        let move = { x: -r.y * orbit.dir + r.x * (170 - distance) / 60, y: r.x * orbit.dir + r.y * (170 - distance) / 60 };
        for (const shot of state.shots) {
          const sx = p.x - shot.x, sy = p.y - shot.y, near = Math.hypot(sx, sy);
          if (near < 70) { move.x += sx / near * 1.5; move.y += sy / near * 1.5; }
        }
        const length = Math.hypot(move.x, move.y) || 1;
        move = { x: move.x / length, y: move.y / length };
        const blocked = state.grid && solidAt(state.grid, p.x + move.x * 34, p.y + move.y * 34);
        if (blocked && now > orbit.until) { orbit.dir = -orbit.dir; orbit.until = now + 1200; }
        if (move.x < -0.38) keys.push('KeyA'); else if (move.x > 0.38) keys.push('KeyD');
        if (move.y < -0.38) keys.push('KeyW'); else if (move.y > 0.38) keys.push('KeyS');
        keys.push(Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'ArrowRight' : 'ArrowLeft') : (dy > 0 ? 'ArrowDown' : 'ArrowUp'));
      } else {
        const post = !lineOfFire(state.grid, p, target) && firingPost(state.grid, p, target);
        if (post) { const step = waypoint(post.x, post.y); toward(step.x, step.y, 4); keys.push(Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'ArrowRight' : 'ArrowLeft') : (dy > 0 ? 'ArrowDown' : 'ArrowUp')); }
        else if (Math.abs(dx) > Math.abs(dy)) { keys.push(dx > 0 ? 'ArrowRight' : 'ArrowLeft'); if (Math.abs(dy) > 16) keys.push(dy > 0 ? 'KeyS' : 'KeyW'); }
        else { keys.push(dy > 0 ? 'ArrowDown' : 'ArrowUp'); if (Math.abs(dx) > 16) keys.push(dx > 0 ? 'KeyD' : 'KeyA'); }
        if (distance < 130) keys.push(Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'KeyA' : 'KeyD') : (dy > 0 ? 'KeyW' : 'KeyS'));
        if (segment % 40 === 0 && state.inventory.bombs > 0 && state.player.hp > 2 && distance < 180) keys.push('KeyE');
      }
    } else if (state.pickups.length && (pickupsAtStart.get(state.room) ?? 0) < 60) {
      pickupsAtStart.set(state.room, (pickupsAtStart.get(state.room) ?? 0) + 1);
      const target = state.pickups.reduce((a, b) => Math.hypot(a.x - p.x, a.y - p.y) < Math.hypot(b.x - p.x, b.y - p.y) ? a : b);
      const step = waypoint(target.x, target.y);
      toward(step.x, step.y, 2);
    } else {
      const open = state.doors.filter(d => /^open/i.test(d.state));
      const back = open.find(d => d.target === cameFromDoor);
      // Keep one door per room visit; re-pick only when it stops being usable.
      if (doorGoal && doorGoal.room === state.room && now > doorGoal.until && !blockedDoors.has(`${state.room}->${doorGoal.target}`)) {
        blockedDoors.add(`${state.room}->${doorGoal.target}`);
        const door = state.doors.find(d => d.target === doorGoal.target);
        warn(`Open ${door?.side} door from room ${state.room} to room ${doorGoal.target} did not let the player through in 15s (run seed ${state.runSeed})`);
        await shot('blocked-door');
      }
      if (!doorGoal || doorGoal.room !== state.room || !open.some(d => d.target === doorGoal.target) || now > doorGoal.until) {
        const usable = open.filter(d => !blockedDoors.has(`${state.room}->${d.target}`));
        // Explore: the least recently visited neighbour, never straight back unless it is the only way.
        const onward = usable.filter(d => d !== back);
        const pool = (!values.boss && wantReturn && back && usable.includes(back) && [back]) || (onward.length && onward) || (usable.length && usable) || open;
        const age = d => lastVisit.get(d.target) ?? -1;
        // --boss: head for the boss room by floor-grid distance (13 columns),
        // with visits as a penalty so dead ends are not retried forever.
        const bossCell = state.rooms.find(([, type]) => type === 5)?.[0];
        const toBoss = d => { const cell = state.rooms[d.target]?.[0]; return cell == null ? 99 : Math.abs(cell % 13 - bossCell % 13) + Math.abs(Math.floor(cell / 13) - Math.floor(bossCell / 13)) + 3 * (visits.get(d.target) ?? 0); };
        const score = values.boss && bossCell != null && state.rooms[state.room]?.[1] !== 5 ? toBoss : age;
        const oldest = Math.min(...pool.map(score));
        const pickFrom = pool.filter(d => score(d) === oldest);
        const door = pickFrom[Math.floor(Math.random() * pickFrom.length)];
        doorGoal = door && { room: state.room, target: door.target, until: now + 15_000 };
      }
      const choice = doorGoal && open.find(d => d.target === doorGoal.target);
      if (choice) keys.push(...routeDoorKeys(state.grid, p, choice));
      else keys.push(['KeyW', 'KeyA', 'KeyS', 'KeyD'][Math.floor(segment / 8) % 4]);
      // Semi-random shooting and occasional bombs while exploring.
      if (segment % 7 < 3) keys.push(['ArrowUp', 'ArrowRight', 'ArrowDown', 'ArrowLeft'][segment % 4]);
      // Never bomb on one heart or less: the bot cannot dodge its own blast reliably.
      if (segment % 90 === 45 && state.inventory.bombs > 0 && state.player.hp > 2) keys.push('KeyE');
    }
    if (process.env.PLAYTEST_DEBUG && segment % 15 === 0) log(`p=${Math.round(p.x)},${Math.round(p.y)} npcs=${JSON.stringify(state.npcs.map(n => [Math.round(n.x), Math.round(n.y), n.kind]))} doors=${JSON.stringify(state.doors.map(d => [d.side, d.state, d.target, Math.round(d.x), Math.round(d.y)]))} clear=${state.clear} keys=${keys.join('+')}`);
    if (keys.includes('KeyE') && !held.has('KeyE')) stats.bombsPlaced++;
    await hold(keys.filter((key, i) => keys.indexOf(key) === i));
    await tab.waitForTimeout(120);
    if (!keys.includes('KeyE')) held.has('KeyE') && await hold(keys.filter(key => key !== 'KeyE'));
    // Stuck detection: moving keys held but no movement for 1.5s -> wiggle.
    if (now - lastMoveCheck.at > 1500) {
      const moved = Math.hypot(state.player.x - lastMoveCheck.x, state.player.y - lastMoveCheck.y);
      const wantsMove = keys.some(key => key.startsWith('Key') && key !== 'KeyE');
      if (wantsMove && moved < 2 && state.player.controls && !state.transition && !state.paused) {
        stats.stalls++; stuckSince ??= now;
        const options = [['KeyW', 'KeyA'], ['KeyW', 'KeyD'], ['KeyS', 'KeyA'], ['KeyS', 'KeyD'], ['KeyW'], ['KeyS'], ['KeyA'], ['KeyD']];
        wiggle = { until: now + 700, keys: options[(stats.stalls * 3) % options.length] };
      } else stuckSince = null;
      if (stuckSince && now - stuckSince > 20_000) { fail('Player has not moved for 20s while movement keys were held'); break; }
      lastMoveCheck = { at: now, x: state.player.x, y: state.player.y };
    }
    previous = state;
  }
  await release();
  state = await observe();
  stats.ticksPerSecond = Number(((state.tick - playStartTick) / ((Date.now() - playStart) / 1000)).toFixed(1));
  if (stats.ticksPerSecond < 40) warn(`Simulation ran at ${stats.ticksPerSecond} ticks/s instead of 60`);
  await shot('end');
  if (!stats.leftStart) fail('Player never left the starting room');
  if (stats.maxTears === 0) fail('No tears were observed while shooting');
  if (stats.distance < 800) fail(`Player barely moved (${Math.round(stats.distance)} units)`);
  if (!stats.returnedToStart) warn('Did not return to the starting room through a door');
  if (stats.pauses === 0 && !issues.failures.length) fail('Pause menu was never exercised');
} catch (error) {
  fail(String(error.stack || error));
} finally {
  const shims = await tab.evaluate(() => window.__playtestShims).catch(() => null);
  if (shims && (shims.imageDecodeFallbacks || shims.vorbisEndTrims))
    warn(`Browser compatibility shims used: ${shims.imageDecodeFallbacks} image decode fallbacks, ${shims.vorbisEndTrims} Vorbis end trims`);
  const frames = await tab.evaluate(() => window.__playtestFrames.slice(30)).catch(() => []);
  frames.sort((a, b) => a - b);
  const pick = q => frames.length ? Number(frames[Math.min(frames.length - 1, Math.floor(q * frames.length))].toFixed(1)) : null;
  const frameStats = { frames: frames.length, p50: pick(0.5), p95: pick(0.95), p99: pick(0.99), max: pick(1), over50ms: frames.filter(f => f > 50).length };
  if (issues.pageErrors.length) fail(`${issues.pageErrors.length} uncaught page error(s)`);
  if (issues.consoleErrors.length) fail(`${issues.consoleErrors.length} console error(s)`);
  if (issues.failedRequests.length) fail(`${issues.failedRequests.length} failed request(s)`);
  const verdict = issues.failures.length ? 'fail' : 'pass';
  const report = {
    verdict, bundle: { commit: manifest.commit, label: manifest.label, version: manifest.version }, browser: browserVersion, playedSeconds: PLAY_MS / 1000,
    stats: { ...stats, roomsEntered: stats.roomsEntered.size, stages: [...stats.stages], distance: Math.round(stats.distance) },
    frameStats, frameStatsScope: 'requestAnimationFrame intervals in headless Chromium with SwiftShader software WebGL; not representative of real hardware', shims, ...issues, screenshots, timeline, completedAt: new Date().toISOString(),
  };
  await writeFile(join(OUT, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify({ verdict, out: OUT, stats: report.stats, frameStats, failures: issues.failures, warnings: issues.warnings }, null, 2));
  if (verdict !== 'pass') exitCode = 1;
  await browser.close();
  server.close();
}
process.exit(exitCode);
