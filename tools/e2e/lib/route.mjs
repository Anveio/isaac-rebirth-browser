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
  };
}
