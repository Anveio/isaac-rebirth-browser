// Grid routing shared by the playtest bot and the boss-route recorder.
// Positions are world units; a room grid cell centre is (col*40+40, row*40+120).

const cellOf = (x, y) => [Math.round((x - 40) / 40), Math.round((y - 120) / 40)];

// Breadth-first route over `grid` ({ w, h, solid: '0'/'1' string }) from the
// player to the goal; returns the centre of the cell `ahead` steps along the
// route, or the goal itself when there is no grid or no route.
export function waypoint(grid, player, goal, ahead = 2) {
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
  const next = path[Math.min(ahead, path.length - 1)];
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
// under live NPCs such as fireplaces and every cell a slot's body reaches into, all of which
// block the player off the grid.
function solidGrid(grid, npcs, slots) {
  // Poops carry an authored selector without a grid entity in the snapshot;
  // treat any such authored cell as blocking.
  // Spikes (8) do not collide but hurt, so routes go around them too.
  const blocks = entity => (entity.collisionClass !== 0 && entity.kind !== 16) || entity.kind === 8;
  const solid = grid.cells.map(c => ((c.entity ? blocks(c.entity) : c.selector !== 0) ? '1' : '0'));
  const mark = (c, r) => {
    if (c >= 0 && r >= 0 && c < grid.width && r < grid.height) solid[r * grid.width + c] = '1';
  };
  for (const npc of npcs) mark(...cellOf(npc.position.x, npc.position.y));
  // A beggar is a radius-12 circle; a machine's SizeMulti stretches it into a capsule
  // (1.5 x 0.75 for slot machines, 3 x 0.75 for the shell game, which reaches into the cells
  // beside it).
  for (const slot of slots) {
    const rx = slot.collisionRadius * slot.sizeMulti.x, ry = slot.collisionRadius * slot.sizeMulti.y;
    const [c0, r0] = cellOf(slot.position.x - rx, slot.position.y - ry), [c1, r1] = cellOf(slot.position.x + rx, slot.position.y + ry);
    for (let r = r0; r <= r1; r++) for (let c = c0; c <= c1; c++) mark(c, r);
  }
  return { w: grid.width, h: grid.height, solid: solid.join('') };
}

// The routing view of a parsed window.game snapshot: player in world units,
// the solid grid and doors.
export function routingView(s) {
  return {
    room: s.currentRoom, transition: !!s.roomTransition, clear: s.room.combatClear,
    player: { x: (s.player.x - 32) / 0.65, y: (s.player.y + 46) / 0.65, controls: s.player.controlsEnabled,
      full: s.player.maxHp <= s.player.hp + s.player.soulHp, hp: s.player.hp + s.player.soulHp },
    grid: s.room.grid && solidGrid(s.room.grid, s.room.npcs.filter(n => !n.dead && !n.removed), s.room.slots ?? []),
    doors: s.room.appearance.doors.map(d => ({ side: d.side, state: d.state, target: d.targetRoom, x: d.worldPosition.x, y: d.worldPosition.y })),
    keys: s.inventory.keys,
    // Live fireplaces, which a route may shoot out rather than walk around.
    fires: s.room.npcs.filter(n => n.kind === 33 && !n.dead && !n.removed).map(n => ({ x: n.position.x, y: n.position.y })),
    // Solid poop cells, which a walk may shoot out rather than route around.
    poops: s.room.grid ? s.room.grid.cells.flatMap((c, i) => (c.entity?.kind === 14 && c.entity.collisionClass !== 0 ? [i] : [])) : [],
    openGrid: s.room.grid && solidGrid(s.room.grid, s.room.npcs.filter(n => !n.dead && !n.removed && n.kind !== 33), s.room.slots ?? []),
    pickups: s.room.pickups.filter(p => !p.removed).map(p => ({ variant: p.variant, subtype: p.subtype, x: p.position.x, y: p.position.y })),
    // Living door-shutting enemies and enemy shots, for fight(). Death is
    // HP below 0 in both games: a Crazy Long Legs at exactly 0 still walks,
    // fires and keeps the doors shut.
    foes: s.room.npcs.filter(n => n.canShutDoors && !n.dead && !n.removed && n.hp >= 0)
      .map(n => ({ kind: n.kind, boss: n.boss, radius: n.collisionRadius, x: n.position.x, y: n.position.y, vx: n.velocity.x, vy: n.velocity.y })),
    shots: s.enemyProjectiles.filter(p => !p.dead && !p.removed).map(p => ({ x: p.position.x, y: p.position.y, vx: p.velocity?.x ?? 0, vy: p.velocity?.y ?? 0 })),
    stage: s.stage, stageTransition: s.stageTransition.state, trapdoor: s.room.trapdoor,
  };
}

// One closed-loop fight decision: aim along the dominant axis at the nearest
// foe and orbit it (bosses at 170 units, others at 110), steering away from
// walls, solid cells, foes and enemy shots; `memory.orbit` flips when boxed in.
// With `memory.lead` (fast wall-bouncing foes such as Envy's heads) the aim
// leads the foe by its velocity over a tear's flight time, and the orbit
// closes to 90 units. Room bounds are the 1x1 playable box.
export function fightKeys(view, memory) {
  const { player, grid, foes, shots } = view;
  if (!foes.length) return [];
  const target = foes.reduce((a, b) => Math.hypot(a.x - player.x, a.y - player.y) <= Math.hypot(b.x - player.x, b.y - player.y) ? a : b);
  const dx = target.x - player.x, dy = target.y - player.y, distance = Math.hypot(dx, dy) || 1;
  // Isaac's tears cover about 10 world units per update.
  const flight = memory.lead ? distance / 10 : 0;
  const ax = dx + (target.vx ?? 0) * flight, ay = dy + (target.vy ?? 0) * flight;
  const aim = Math.abs(ax) >= Math.abs(ay) ? (ax > 0 ? 'ArrowRight' : 'ArrowLeft') : (ay > 0 ? 'ArrowDown' : 'ArrowUp');
  // --dodge sizes foes by their collision radius (Mega Fatty 40) rather than the
  // boss flag, which the Fistula family's small pieces also carry.
  const big = foe => (memory.dodge ? (foe.radius ?? (foe.boss ? 40 : 13)) >= 30 : foe.boss);
  // --dodge holds dashers and burst shooters further out: a Squirt's (220) dash
  // covers 80 units in four frames and a Crazy Long Legs (207) bursts 8 ways.
  // Chasers such as Skinnies stay at 110, where the orbit keeps ahead of them.
  const wide = foe => memory.dodge && [207, 220].includes(foe.kind);
  const radius = memory.lead ? 90 : big(target) ? (memory.bossRadius ?? 170) : wide(target) ? 160 : 110, pull = (radius - distance) / radius * 2;
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
      const near = Math.hypot(foe.x - (player.x + ux * 40), foe.y - (player.y + uy * 40)), reach = big(foe) ? 90 : 40;
      if (near < reach) score -= (90 - near) / 30;
    }
    if (memory.dodge) {
      // Opt-in (--dodge): score each move against every shot's path over the
      // next frames and keep clear of lit fireplaces, which burn on contact.
      for (const shot of shots) for (const t of [2, 4, 6, 8, 12]) {
        const near = Math.hypot(player.x + ux * 3.5 * t - (shot.x + shot.vx * t), player.y + uy * 3.5 * t - (shot.y + shot.vy * t));
        if (near < 32) score -= (32 - near) / 8 * (t <= 6 ? 1.5 : 1);
      }
      for (const fire of view.fires ?? []) for (const look of [10, 20, 30]) {
        const near = Math.hypot(fire.x - (player.x + ux * look), fire.y - (player.y + uy * look));
        if (near < 36) score -= (36 - near) / 8;
      }
      // A foe's own motion (a boss's lunge) projected over the same frames.
      for (const foe of foes) for (const t of [3, 6, 10, 15]) {
        const near = Math.hypot(player.x + ux * 3.5 * t - (foe.x + (foe.vx ?? 0) * t), player.y + uy * 3.5 * t - (foe.y + (foe.vy ?? 0) * t));
        const reach = (foe.radius ?? (foe.boss ? 40 : 13)) + 24;
        if (near < reach) score -= (reach - near) / 12 * (t <= 6 ? 1.5 : 1);
      }
      // Keep room to dodge: walls within 50 units cost in proportion, corners
      // (two walls) more.
      const nx = player.x + ux * 20, ny = player.y + uy * 20;
      const gaps = [nx - 72, 568 - nx, ny - 152, 408 - ny];
      for (const gap of gaps) if (gap < 50) score -= (50 - gap) / 25;
      if (Math.min(gaps[0], gaps[1]) < 60 && Math.min(gaps[2], gaps[3]) < 60) score -= 1;
      // Pinned against a body (Mega Fatty parked by a corner): getting out
      // outranks every other term.
      for (const foe of foes) {
        const touch = (foe.radius ?? 13) + 20, now = Math.hypot(foe.x - player.x, foe.y - player.y);
        if (now < touch) score += (Math.hypot(foe.x - nx, foe.y - ny) - now) * 0.4;
      }
      // Tears fly along one axis: favour moves that line the player up with
      // the target on an axis, so they hit.
      const offNow = Math.min(Math.abs(dx), Math.abs(dy));
      const offNext = Math.min(Math.abs(target.x - nx), Math.abs(target.y - ny));
      if (offNow > 20) score += (offNow - offNext) / 20 * 0.6;
    } else for (const shot of shots) {
      const near = Math.hypot(shot.x - (player.x + ux * 25), shot.y - (player.y + uy * 25));
      if (near < 45) score -= (45 - near) / 15;
    }
    if (!best || score > best.score) best = { score, keys, ux, uy };
  }
  if (best.score < 0.2 && Math.abs(tangent[0] * best.ux + tangent[1] * best.uy) < 0.5) memory.orbit = -memory.orbit;
  return [...best.keys, aim];
}
