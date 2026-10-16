// Grid routing shared by the playtest bot and the boss-route recorder.
// Positions are world units; a room grid cell centre is (col*40+40, row*40+120).

const cellOf = (x, y) => [Math.round((x - 40) / 40), Math.round((y - 120) / 40)];

// Breadth-first route over `grid` ({ w, h, solid: '0'/'1' string }) from the
// player to the goal; returns a waypoint two cells ahead, or the goal itself
// when there is no grid or no route.
export function waypoint(grid, player, goal) {
  if (!grid) return { x: goal.x, y: goal.y };
  const [sc, sr] = cellOf(player.x, player.y), [gc, gr] = cellOf(goal.x, goal.y);
  const inside = (c, r) => c >= 0 && r >= 0 && c < grid.w && r < grid.h;
  if (!inside(sc, sr) || !inside(gc, gr)) return { x: goal.x, y: goal.y };
  const key = (c, r) => r * grid.w + c, previous = new Map([[key(sc, sr), -1]]), queue = [[sc, sr]];
  while (queue.length) {
    const [c, r] = queue.shift();
    if (c === gc && r === gr) break;
    for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const nc = c + dc, nr = r + dr, k = key(nc, nr);
      if (!inside(nc, nr) || previous.has(k) || (grid.solid[k] === '1' && !(nc === gc && nr === gr))) continue;
      previous.set(k, key(c, r)); queue.push([nc, nr]);
    }
  }
  if (!previous.has(key(gc, gr))) return { x: goal.x, y: goal.y };
  const path = [];
  for (let k = key(gc, gr); k !== -1; k = previous.get(k)) path.unshift(k);
  const next = path[Math.min(2, path.length - 1)];
  return next === key(gc, gr) ? { x: goal.x, y: goal.y } : { x: (next % grid.w) * 40 + 40, y: Math.floor(next / grid.w) * 40 + 120 };
}

// Movement keys (WASD) that take the player toward a point, with `slack` units of dead zone.
export function toward(player, point, slack = 8) {
  const keys = [];
  if (point.x < player.x - slack) keys.push('KeyA'); else if (point.x > player.x + slack) keys.push('KeyD');
  if (point.y < player.y - slack) keys.push('KeyW'); else if (point.y > player.y + slack) keys.push('KeyS');
  return keys;
}

// Keys that walk the player to an open door ({ side, x, y }): route around the
// grid until the doorway is the next waypoint, then align the cross axis and
// push outward through it.
export function doorKeys(grid, player, door) {
  const step = waypoint(grid, player, door);
  if (step.x !== door.x || step.y !== door.y) return toward(player, step, 4);
  const keys = [];
  if (door.side === 'left' || door.side === 'right') {
    if (Math.abs(door.y - player.y) > 6) keys.push(door.y < player.y ? 'KeyW' : 'KeyS');
    keys.push(door.side === 'left' ? 'KeyA' : 'KeyD');
  } else {
    if (Math.abs(door.x - player.x) > 6) keys.push(door.x < player.x ? 'KeyA' : 'KeyD');
    keys.push(door.side === 'up' ? 'KeyW' : 'KeyS');
  }
  return keys;
}

// Solid cells (walls, rocks, pits, poops, spikes; door cells stay walkable), plus the cells
// under live NPCs such as fireplaces, which block the player off the grid.
function solidGrid(grid, npcs) {
  // Poops carry an authored selector without a grid entity in the snapshot;
  // treat any such authored cell as blocking.
  // Spikes (8) do not collide but hurt, so routes go around them too.
  const blocks = entity => (entity.collisionClass !== 0 && entity.kind !== 16) || entity.kind === 8;
  const solid = grid.cells.map(c => ((c.entity ? blocks(c.entity) : c.selector !== 0) ? '1' : '0'));
  for (const npc of npcs) {
    const [c, r] = cellOf(npc.position.x, npc.position.y);
    if (c >= 0 && r >= 0 && c < grid.width && r < grid.height) solid[r * grid.width + c] = '1';
  }
  return { w: grid.width, h: grid.height, solid: solid.join('') };
}

// The routing view of a parsed window.game snapshot: player in world units,
// the solid grid and doors.
export function routingView(s) {
  return {
    room: s.currentRoom, transition: !!s.roomTransition, clear: s.room.combatClear,
    player: { x: (s.player.x - 32) / 0.65, y: (s.player.y + 46) / 0.65, controls: s.player.controlsEnabled },
    grid: s.room.grid && solidGrid(s.room.grid, s.room.npcs.filter(n => !n.dead && !n.removed)),
    doors: s.room.appearance.doors.map(d => ({ side: d.side, state: d.state, target: d.targetRoom, x: d.worldPosition.x, y: d.worldPosition.y })),
    keys: s.inventory.keys,
    // Live fireplaces, which a route may shoot out rather than walk around.
    fires: s.room.npcs.filter(n => n.kind === 33 && !n.dead && !n.removed).map(n => ({ x: n.position.x, y: n.position.y })),
    openGrid: s.room.grid && solidGrid(s.room.grid, s.room.npcs.filter(n => !n.dead && !n.removed && n.kind !== 33)),
    pickups: s.room.pickups.filter(p => !p.removed).map(p => ({ variant: p.variant, subtype: p.subtype, x: p.position.x, y: p.position.y })),
    // Living door-shutting enemies and enemy shots, for fight().
    foes: s.room.npcs.filter(n => n.canShutDoors && !n.dead && !n.removed && n.hp > 0)
      .map(n => ({ kind: n.kind, boss: n.boss, x: n.position.x, y: n.position.y })),
    shots: s.enemyProjectiles.filter(p => !p.dead && !p.removed).map(p => ({ x: p.position.x, y: p.position.y })),
    stage: s.stage, stageTransition: s.stageTransition.state, trapdoor: s.room.trapdoor,
  };
}

// One closed-loop fight decision: aim along the dominant axis at the nearest
// foe and orbit it (bosses at 170 units, others at 110), steering away from
// walls, solid cells, foes and enemy shots; `memory.orbit` flips when boxed in.
// Room bounds are the 1x1 playable box.
export function fightKeys(view, memory) {
  const { player, grid, foes, shots } = view;
  if (!foes.length) return [];
  const target = foes.reduce((a, b) => Math.hypot(a.x - player.x, a.y - player.y) <= Math.hypot(b.x - player.x, b.y - player.y) ? a : b);
  const dx = target.x - player.x, dy = target.y - player.y, distance = Math.hypot(dx, dy) || 1;
  const aim = Math.abs(dx) >= Math.abs(dy) ? (dx > 0 ? 'ArrowRight' : 'ArrowLeft') : (dy > 0 ? 'ArrowDown' : 'ArrowUp');
  const radius = target.boss ? 170 : 110, pull = (radius - distance) / radius * 2;
  const tangent = [-dy / distance * memory.orbit, dx / distance * memory.orbit];
  const want = [tangent[0] - dx / distance * pull, tangent[1] - dy / distance * pull];
  const solid = (x, y) => {
    const [c, r] = cellOf(x, y);
    return grid && c >= 0 && r >= 0 && c < grid.w && r < grid.h && grid.solid[r * grid.w + c] === '1';
  };
  let best;
  for (const [ux0, uy0, keys] of [[1, 0, ['KeyD']], [1, 1, ['KeyD', 'KeyS']], [0, 1, ['KeyS']], [-1, 1, ['KeyS', 'KeyA']],
    [-1, 0, ['KeyA']], [-1, -1, ['KeyA', 'KeyW']], [0, -1, ['KeyW']], [1, -1, ['KeyW', 'KeyD']], [0, 0, []]]) {
    const norm = Math.hypot(ux0, uy0) || 1, ux = ux0 / norm, uy = uy0 / norm;
    let score = ux * want[0] + uy * want[1];
    for (const look of [20, 45]) {
      const nx = player.x + ux * look, ny = player.y + uy * look, cost = look === 20 ? 3 : 1;
      if (nx < 72 || nx > 568 || ny < 152 || ny > 408) score -= cost;
      if (solid(nx, ny)) score -= cost;
    }
    for (const foe of foes) {
      const near = Math.hypot(foe.x - (player.x + ux * 40), foe.y - (player.y + uy * 40)), reach = foe.boss ? 90 : 40;
      if (near < reach) score -= (90 - near) / 30;
    }
    for (const shot of shots) {
      const near = Math.hypot(shot.x - (player.x + ux * 25), shot.y - (player.y + uy * 25));
      if (near < 45) score -= (45 - near) / 15;
    }
    if (!best || score > best.score) best = { score, keys, ux, uy };
  }
  if (best.score < 0.2 && Math.abs(tangent[0] * best.ux + tangent[1] * best.uy) < 0.5) memory.orbit = -memory.orbit;
  return [...best.keys, aim];
}
