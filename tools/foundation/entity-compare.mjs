// Shared original/browser field sets for authored room entities.
// Native observations come from foundation-combat-observe.py; browser rows
// are the engine's own snapshot. Every row is exact: f32 values are restored
// with Math.fround from shortest-decimal JSON, never compared with epsilons.
import { manifestPath, readJsonEvidence } from './browser-evidence.mjs';

export const f32 = value => typeof value === 'number' && Number.isFinite(value) ? Math.fround(value) : undefined;
export const vector = value => value ? [f32(value.x ?? value[0]), f32(value.y ?? value[1])] : undefined;

/** One exact field row; a missing value on either side is a failure. */
export function check(rows, field, expected, actual) {
  rows.push({ field, expected, actual,
    status: expected !== undefined && actual !== undefined && JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
}

// An actor whose sprite overlaps the player's (both within 40 world units).
const overlapping = (actor, player) => Math.abs(actor[0] - player[0]) < 40 && Math.abs(actor[1] - player[1]) < 40;

const ROCK_ANIMATIONS = { 2: 'normal', 3: 'black', 4: 'tinted', 5: 'bombrock', 6: 'alt' };
const DAMAGE_FLAGS = { tear: 0, bomb: 4, fireContact: 2, roomEffect: 0, laser: 8 };

/** Fields every authored NPC shares, as [name, native, port] pairs. Native
 * position is authoritative on full phases and presented half-motion on
 * intermediate phases. Native +31f is dead; +31e is active. */
export function npcFields(native, actor, phase) {
  return [
    ['identity', [native.type, native.variant, native.subtype], [actor.kind, actor.variant, actor.subtype]],
    ['hp', f32(native.hp), f32(actor.hp)], ['maxHp', f32(native.maxHp), f32(actor.maxHp)],
    ['radius', f32(native.collisionRadius), f32(actor.collisionRadius)], ['mass', f32(native.collisionMass), f32(actor.collisionMass)],
    ['contactDamage', f32(native.collisionDamage), f32(actor.collisionDamage)],
    ['gridClass', native.gridCollisionClass, actor.gridCollisionClass], ['entityClass', native.entityCollisionClass, actor.entityCollisionClass],
    ['canShutDoors', native.canShutDoors, actor.canShutDoors], ['boss', native.boss, actor.boss],
    ['state', native.npcState, actor.state], ['frameAge', native.npcFrameAge, actor.frameAge],
    ['savedInitializedState', native.npcSavedInitializedState, actor.savedInitializedState],
    ['dead', native.removed, actor.dead], ['active', native.active, !actor.removed], ['visible', native.visible, actor.visible],
    ['animation', native.animation, actor.animation], ['animationFrame', f32(native.animationFrame), f32(actor.animationClock)],
    ['animationPlaying', native.animationPlaying, actor.animationPlaying],
    ['pendingDamage', f32(native.pendingDamage), f32(actor.damage.pendingDamage)], ['pendingFlags', native.pendingFlags, actor.damage.pendingFlags],
    ['orderedDamage', native.pendingDamageRecords?.map(row => [f32(row.amount), row.flags, row.sourceType]),
      actor.damage.events.map(row => [f32(row.amount), DAMAGE_FLAGS[row.kind], row.source.entityType])],
    ['position', vector(native.position), vector(phase === 0 ? actor.presentedPosition : actor.position)],
    ['velocity', vector(native.velocity), vector(actor.velocity)], ['target', vector(native.npcTarget), vector(actor.target)],
    ['championColor', native.npcChampionColor, actor.champion?.color ?? -1],
    // Sprite Color +2d0 (champion colour or timed hit flash), every component.
    ['spriteColor', native.spriteColor?.map(f32), actor.spriteColor &&
      [...actor.spriteColor.current.tint, ...actor.spriteColor.current.colorize, ...actor.spriteColor.current.offset].map(f32)],
  ];
}

/** Existing rectangular-room art contract, anchored to actual native bounds.
 * This does not infer an irregular contour from obstacle grid entities. */
export function roomArtRows(rows, source, actual) {
  const room = source.room, art = actual.room.appearance;
  check(rows, 'room.art.cacheDimensions', [room.gridWidth + 2, room.gridHeight + 2, 26],
    art && [art.columns, art.rows, art.tileSize]);
  const lower = room.boundsMin ?? room.boundsTopLeft;
  const upper = room.boundsMax ?? room.boundsBottomRight;
  const projected = lower && upper && art && [
    lower[0] * .65 + 32 - art.origin.x, lower[1] * .65 - 47 - art.origin.y,
    upper[0] * .65 + 32 - art.origin.x, upper[1] * .65 - 47 - art.origin.y];
  const bounds = quads => quads?.length ? [
    Math.min(...quads.map(quad => quad.destRect.x)), Math.min(...quads.map(quad => quad.destRect.y)),
    Math.max(...quads.map(quad => quad.destRect.x + quad.destRect.width)),
    Math.max(...quads.map(quad => quad.destRect.y + quad.destRect.height))] : undefined;
  const covered = (quads, x, y) => quads?.some(({ destRect: rect }) =>
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height);
  check(rows, 'room.art.floorBounds', projected, bounds(art?.floorQuads));
  check(rows, 'room.art.wallBounds', projected?.map((value, index) => value + (index < 2 ? -52 : 52)),
    bounds(art?.wallQuads));
  let floor = 0, walls = 0;
  if (projected) {
    const [left, top, right, bottom] = projected;
    for (let row = 0; row < room.gridHeight - 2; row++) {
      const y = top + (row + .5) * 26;
      for (let column = 0; column < room.gridWidth - 2; column++)
        floor += Number(covered(art.floorQuads, left + (column + .5) * 26, y));
      walls += Number(covered(art.wallQuads, left - 26, y))
        + Number(covered(art.wallQuads, right + 26, y));
    }
    for (let column = 0; column < room.gridWidth - 2; column++) {
      const x = left + (column + .5) * 26;
      walls += Number(covered(art.wallQuads, x, top - 26))
        + Number(covered(art.wallQuads, x, bottom + 26));
    }
  }
  check(rows, 'room.art.coveredFloorCells', (room.gridWidth - 2) * (room.gridHeight - 2), floor);
  check(rows, 'room.art.coveredWallCells', 2 * (room.gridWidth + room.gridHeight - 4), walls);
}

/** Room lock, manager counts and door states. */
export function roomRows(rows, source, actual) {
  check(rows, 'room.identity', source.room.descriptorIndex, actual.currentRoom);
  check(rows, 'room.grid', [source.room.gridWidth, source.room.gridHeight], actual.room.grid && [actual.room.grid.width, actual.room.grid.height]);
  check(rows, 'room.combatClear', source.room.combatClear, actual.room.combatClear);
  // The Basement overlay's per-room offsets (fxlayers.xml fx 2): the only
  // layered fx entry; the port's origin adds the 663x429 centring offsets.
  const overlay = actual.room.appearance?.fx?.overlayOrigin;
  // FX+0x38 (54a5b6): the selected fxRays group, -1 when none.
  check(rows, 'stage.fxRayGroup', source.fxRayGroup, actual.room.appearance.fxRays?.index ?? -1);
  // Render 547890 adds animationSpeed (+4) to each ray's f32 phase (+8) on
  // every call; the port counts its calls and the renderer carries its load
  // phase forward the same way.
  if (source.fxRayPhases) {
    const group = actual.room.appearance.fxRays;
    const calls = group?.calls ?? 0;
    check(rows, 'stage.fxRayPhases', source.fxRayPhases.map(ray => ray.map(f32)), group?.rays.map(ray => {
      let phase = f32(ray.phase);
      for (let call = 0; call < calls; call++) phase = f32(phase + f32(ray.animationSpeed));
      return [f32(ray.opacity), f32(ray.animationSpeed), phase];
    }) ?? []);
  }
  check(rows, 'stage.overlayOffset', source.fxLayers?.flat().map(([x, y]) => [f32(x), f32(y)]),
    overlay && [[f32(overlay.x - (663 - 480) * 0.5), f32(overlay.y - (429 - 270) * 0.5)]]);
  check(rows, 'room.combatClearCountdown', source.room.combatClearCountdown, actual.room.combatClearCountdown);
  // A 0/0 fill is NaN on both sides: native keeps its bits, JSON gives null.
  const nativeFill = source.bossBar && (source.bossBar.fill === null ? 'NaN' : f32(source.bossBar.fill));
  const portFill = actual.bossBar && (actual.bossBar.fill === null ? 'NaN' : f32(actual.bossBar.fill));
  check(rows, 'hud.bossBarFill', nativeFill, portFill);
  check(rows, 'hud.bossBarFlash', source.bossBar?.flash, actual.bossBar?.flash);
  for (const field of ['countablePending', 'countableCurrent', 'bossPending', 'bossCurrent'])
    check(rows, `manager.${field}`, source.room[field], actual.room.managerCounts?.[field]);
  // Admitted grid objects: Block3, Tinted Rock4, Alt Rock6, Pit7 (autotile
  // frame) and Spider Web10.
  for (const cell of source.grid.filter(cell => [3, 4, 6, 7, 10].includes(cell.type))) {
    const port = actual.room.grid?.cells[cell.index]?.entity;
    check(rows, `grid${cell.index}`, [cell.type, cell.variant, cell.state, cell.collisionClass],
      port && [port.kind, port.variant, port.state, port.collisionClass]);
  }
  for (const door of source.grid.filter(cell => cell.type === 16)) {
    const port = actual.room.grid?.cells[door.index]?.entity;
    // SetRoomTypes565b00 variant: 7 ordinary, 4 barred (uncleared miniboss).
    check(rows, `door${door.index}.variant`, door.variant, port?.variant);
    check(rows, `door${door.index}.state`, door.state, port?.state);
    check(rows, `door${door.index}.class`, door.collisionClass, port?.collisionClass);
  }
}

/** Collision class of the grid cell under a point (GetGridIndex: 40-unit
 * cells from (20,100)); a port cell without an entity falls back to its
 * authored selector like Grid::collision. */
function cellClass(cells, width, height, point) {
  const column = Math.floor((point[0] - 20) / 40), row = Math.floor((point[1] - 100) / 40);
  if (column < 0 || row < 0 || column >= width || row >= height) return 4;
  return cells(row * width + column);
}
const nativeCellClass = (source, point) => {
  const byIndex = new Map(source.grid.map(cell => [cell.index, cell.collisionClass]));
  return cellClass(index => byIndex.get(index) ?? 0, source.room.gridWidth, source.room.gridHeight, point);
};
const portCellClass = (actual, point) => {
  const grid = actual.room.grid;
  return grid && cellClass(index => {
    const cell = grid.cells[index];
    return cell.entity ? cell.entity.collisionClass : cell.selector === 3000 ? 1 : cell.selector >= 1000 ? 3 : 0;
  }, grid.width, grid.height, point);
};

/** Every authored NPC in the room, by identity and spawn seed, then each
 * selected actor's shared fields plus the spec's own rows. */
export function entityRows(rows, source, actual, spec, render) {
  const phase = source.nativePhase.systemIterations % 2;
  const npcs = source.entities.filter(actor => actor.type >= 10 && actor.type < 1000);
  const port = actual.room.npcs.filter(actor => !actor.removed);
  check(rows, 'npc.identities', npcs.map(actor => [actor.type, actor.variant, actor.spawnSeed]),
    port.map(actor => [actor.kind, actor.variant, actor.spawnSeed]));
  check(rows, 'system.phase', phase, actual.systemClock.systemCounter % 2);
  roomRows(rows, source, actual);
  if (spec.shape) roomArtRows(rows, source, actual);
  // The browser's own draw record for this capture: what it actually drew.
  check(rows, 'render.recorded', true, render !== undefined);
  // Room::Render draws the manager render array (Room+15d0 +80/+8c) in order,
  // so a later entry draws over an earlier one. Entity_Player and Entity_NPC
  // share the y key 441bd0; Player.Init sets the +460 addend to 5 (519485).
  // Draw order is visible only where sprites overlap: for each NPC within 40
  // world units of the player on both axes in the original that both the
  // original's array and the browser's draw hold, whether it draws over the
  // player; graded whenever both drew the player.
  const sorted = source.renderSortedEntities ?? [];
  const nativePlayer = sorted.findIndex(actor => actor.type === 1);
  if (render) check(rows, 'render.orderRecorded', true, Array.isArray(render.order) || undefined);
  const drawn = render?.order ?? [];
  const drawnPlayer = drawn.findIndex(([kind]) => kind === 1);
  const drawnAt = actor => drawn.findIndex(([kind, seed]) => kind === actor.type && seed === actor.spawnSeed);
  const shared = sorted.flatMap((actor, index) => actor.type >= 10 && actor.type < 1000 && drawnAt(actor) >= 0
    && nativePlayer >= 0 && overlapping(actor.position, sorted[nativePlayer].position) ? [[actor, index]] : []);
  if (nativePlayer >= 0 && drawnPlayer >= 0 && shared.length)
    check(rows, 'render.npcsOverPlayer', shared.map(([actor, index]) => [actor.type, actor.spawnSeed, index > nativePlayer]),
      shared.map(([actor]) => [actor.type, actor.spawnSeed, drawnAt(actor) > drawnPlayer]));
  // Intact rock-family grid entities: native Rock Render 56bf80 sets frame
  // variant of its type's grid_rock.anm2 animation (Init 56ed00). Destroyed
  // rubble needs per-piece offsets the engine does not model yet, and the
  // other grid kinds' render rules are not decoded; neither is claimed here.
  const rock = cell => ROCK_ANIMATIONS[cell.type] && cell.state === 1 &&
    (cell.type === 2 && cell.variant >= 1000 ? [cell.index, cell.type, 'big', cell.variant - 1000] : [cell.index, cell.type, ROCK_ANIMATIONS[cell.type], cell.variant]);
  check(rows, 'render.rocks', source.grid.map(rock).filter(Boolean),
    render?.grid.filter(drawn => ![1, 7, 14, 16].includes(drawn.kind)).map(drawn => [drawn.index, drawn.kind, drawn.animation, drawn.frame]));
  // Door.LoadGraphics5629c0 overlays door_17_bardoor on variant4; a closed
  // door (AnimState563b70) holds its Idle clip. The opening Vanish fade is
  // not modelled, so only doors not yet opening (state != 2) are claimed.
  check(rows, 'render.doorBars', source.grid.filter(cell => cell.type === 16 && cell.variant === 4 && cell.state !== 2)
    .map(cell => [cell.index, 'Idle']),
    render?.grid.filter(drawn => drawn.kind === 16).map(drawn => [drawn.index, drawn.animation]));
  // Pit Render draws `pit` frame variant (the autotile frame 5684c0 stores).
  // GridEntity_Decoration Init 561e60 gives every decoration the original
  // holds a Props clip, chosen by backdrop (Depths props in a sacrifice room,
  // Sheol in a curse room); crawlspaces (16) draw none. The port must draw
  // exactly the original's decoration cells; clip parity is a diagnostic.
  check(rows, 'render.decorations', source.room.type === 16 ? []
    : source.grid.filter(cell => cell.type === 1).map(cell => cell.index),
    render?.grid.filter(drawn => drawn.kind === 1).map(drawn => drawn.index));
  // Poop.Init569600 gives every ordinary poop (variant0) a grid_poop sprite;
  // an undrawn one is an invisible wall. This is a render contract against
  // the port's own grid: poop admission and placement parity with the
  // original belong to the grid rows, and clip choice is not claimed here.
  check(rows, 'render.poops', (actual.room.grid?.cells ?? []).flatMap((cell, index) =>
    cell.entity?.kind === 14 && cell.entity.variant === 0 ? [index] : []),
    render?.grid.filter(drawn => drawn.kind === 14).map(drawn => drawn.index));
  check(rows, 'render.pits', source.grid.filter(cell => cell.type === 7).map(cell => [cell.index, 'pit', cell.variant]),
    render?.grid.filter(drawn => drawn.kind === 7).map(drawn => [drawn.index, drawn.animation, drawn.frame]));
  for (const [field, expected, observed] of spec.roomFields?.(source, actual, phase) ?? []) check(rows, field, expected, observed);
  const selected = npcs.filter(actor => spec.matches(actor.type, actor.variant));
  for (const native of selected) {
    const actor = port.find(candidate => candidate.kind === native.type && candidate.spawnSeed === native.spawnSeed);
    if (!actor) continue; // The identity row already records the missing actor.
    const name = `${spec.name}${native.spawnSeed}`;
    for (const [field, expected, observed] of [...npcFields(native, actor, phase), ...(spec.fields?.(native, actor, phase) ?? [])])
      check(rows, `${name}.${field}`, expected, observed);
    // A ground walker (class5) never stands inside a solid grid cell (#229).
    if (native.gridCollisionClass === 5)
      check(rows, `${name}.insideSolid`, nativeCellClass(source, vector(native.position)) !== 0,
        portCellClass(actual, vector(actor.position)) !== 0);
    // Every drawn NPC uses the source sprite Scale (+2c4), 1:1 in logical
    // pixels; only positions take the 0.65 world projection.
    const drawn = render?.npcs.find(candidate => candidate.kind === native.type && candidate.spawnSeed === native.spawnSeed);
    if (drawn) check(rows, `${name}.spriteScale`, vector(native.spriteScale), vector(drawn.scale));
  }
  // Tear poofs 1000.12/13 take sprite rotation +2cc from the 45abe3 leaf on
  // every Update: Init's Random(4) * 90 (+814), negated unless FlipX.
  for (const native of source.entities.filter(actor => actor.type === 1000 && [12, 13].includes(actor.variant) && !actor.removed && actor.spriteRotation !== undefined)) {
    const effect = (actual.spriteEffects ?? []).find(candidate => candidate.variant === native.variant && candidate.spawnSeed === native.spawnSeed && !candidate.removed);
    // A poof whose seed the port did not draw is a spawn-order divergence
    // (#952), not graded here.
    if (!effect) continue;
    check(rows, `poof${native.spawnSeed}.spriteRotation`, f32(native.spriteRotation), f32(effect.sprite.rotation));
  }
  // Entity_Laser beams in room order, by spawn seed, with their source fields.
  const beams = source.entities.filter(actor => actor.type === 7 && !actor.removed);
  check(rows, 'laser.storeOwner', true, Array.isArray(actual.lasers) ? true : undefined);
  const stored = Array.isArray(actual.lasers) ? actual.lasers : [];
  for (const laser of stored)
    check(rows, `laser${laser.spawnSeed}.activeOwner`, 'boolean',
      typeof laser.active === 'boolean' ? 'boolean' : undefined);
  const ported = stored.filter(laser => laser.active === true);
  check(rows, 'laser.seeds', beams.map(actor => actor.spawnSeed), ported.map(laser => laser.spawnSeed));
  for (const beam of beams) {
    const laser = ported.find(candidate => candidate.spawnSeed === beam.spawnSeed);
    if (!laser) continue;
    const name = `laser${beam.spawnSeed}`;
    for (const [field, expected, observed] of [
      ['position', vector(beam.position), vector(phase === 0 ? laser.position : laser.savedPosition)],
      ['angle', f32(beam.laserAngle), f32(laser.angle)], ['timeout', beam.laserTimeout, laser.remainingUpdates],
      ['shrinking', beam.laserShrinking, laser.contactDisabled], ['length', f32(beam.laserLength), f32(laser.beamLength)],
      ['endpoint', vector(beam.laserEndpoint), vector(laser.endpoint)], ['scaleX', f32(beam.spriteScale?.[0]), f32(laser.sprite?.scale?.x)],
    ]) check(rows, `${name}.${field}`, expected, observed);
  }
  // Entity_Slot machines and beggars (type 6) by spawn seed: the idle body
  // the player collides with, anchored to its spawn position.
  const machines = source.entities.filter(actor => actor.type === 6 && !actor.removed);
  const slots = actual.room.slots ?? [];
  check(rows, 'slot.identities', machines.map(actor => [actor.type, actor.variant, actor.spawnSeed]),
    slots.map(slot => [6, slot.variant, slot.spawnSeed]));
  for (const machine of machines) {
    const slot = slots.find(candidate => candidate.spawnSeed === machine.spawnSeed);
    if (!slot) continue;
    const name = `slot${machine.spawnSeed}`;
    for (const [field, expected, observed] of [
      ['position', vector(machine.position), vector(phase === 0 ? slot.presentedPosition : slot.position)],
      ['velocity', vector(machine.velocity), vector(slot.velocity)],
      ['radius', f32(machine.collisionRadius), f32(slot.collisionRadius)], ['mass', f32(machine.collisionMass), f32(slot.collisionMass)],
      ['animation', machine.animation, slot.animation],
    ]) check(rows, `${name}.${field}`, expected, observed);
  }
  const shots = source.entities.filter(actor => actor.type === 9 && !actor.removed);
  check(rows, 'projectile.seeds', shots.map(actor => actor.spawnSeed),
    actual.enemyProjectiles.filter(actor => !actor.dead && !actor.removed).map(actor => actor.spawnSeed));
}

/** Read a side's manifest, raw or from its archived receipt. */
export const loadManifest = (directory, side) => readJsonEvidence(manifestPath(directory, side));

export const MARKERS = ['admission', 'damage', 'death', 'retirement', 'clear', 'shape', 'bossRoom', 'roomType', 'overPlayer', 'underPlayer'];


/** Lifecycle markers each game reaches on its own over a whole case. */
export function markers(directory, spec) {
  const result = {};
  for (const side of ['original', 'browser']) {
    const seen = Object.fromEntries(MARKERS.map(name => [name, false]));
    const manifest = loadManifest(directory, side);
    const rows = side === 'original' ? manifest.debugObservations.map(row => row.observation?.gameplay)
      : manifest.frames.map(row => row.state);
    const renders = side === 'original' ? [] : manifest.frames.map(row => row.render);
    const dead = new Set();
    let locked = false;
    for (const [at, row] of rows.entries()) {
      if (!row || (side === 'original' ? row.status !== 'pass' : row.screen !== 'playing')) continue;
      // Overlapping draw order: a spec actor drawn over (after) or under the
      // player, from the original's render array or the browser's draw order.
      if (side === 'original') {
        const sorted = row.renderSortedEntities ?? [], player = sorted.findIndex(actor => actor.type === 1);
        if (player >= 0) for (const [index, actor] of sorted.entries())
          if (spec.matches(actor.type, actor.variant) && overlapping(actor.position, sorted[player].position)) {
            seen.overPlayer ||= index > player; seen.underPlayer ||= index < player;
          }
      } else {
        const order = renders[at]?.order ?? [], player = order.findIndex(([kind]) => kind === 1);
        if (player >= 0) for (const actor of row.room.npcs.filter(actor => spec.matches(actor.kind, actor.variant))) {
          const index = order.findIndex(([kind, seed]) => kind === actor.kind && seed === actor.spawnSeed);
          // Port player x/y are screen units: world = ((x - 32) / 0.65, (y + 46) / 0.65).
          if (index >= 0 && overlapping([actor.position.x, actor.position.y], [(row.player.x - 32) / 0.65, (row.player.y + 46) / 0.65])) {
            seen.overPlayer ||= index > player; seen.underPlayer ||= index < player;
          }
        }
      }
      const actors = side === 'original'
        ? row.entities.filter(actor => spec.matches(actor.type, actor.variant)).map(actor => ({ seed: actor.spawnSeed, hp: actor.hp, maxHp: actor.maxHp, pending: actor.pendingDamage, dead: actor.removed }))
        : [...row.room.npcs.filter(actor => !actor.removed && spec.matches(actor.kind, actor.variant)).map(actor => ({ seed: actor.spawnSeed, hp: actor.hp, maxHp: actor.maxHp, pending: actor.damage.pendingDamage, dead: actor.dead })),
          ...(row.room.slots ?? []).filter(slot => spec.matches(6, slot.variant)).map(slot => ({ seed: slot.spawnSeed, hp: 0, maxHp: 0, pending: 0, dead: false }))];
      const clear = side === 'original' ? row.room.combatClear : row.room.combatClear;
      // The spec's authored shape plus the one-cell wall border.
      const grid = side === 'original' ? [row.room.gridWidth, row.room.gridHeight] : row.room.grid && [row.room.grid.width, row.room.grid.height];
      seen.shape ||= Boolean(spec.shape && grid && grid[0] === spec.shape[0] + 2 && grid[1] === spec.shape[1] + 2);
      // Room specs: the player stands in a room of the spec's authored type.
      seen.roomType ||= spec.roomType !== undefined
        && (side === 'original' ? row.room.type : row.room.descriptor?.template?.type) === spec.roomType;
      // Boss routes: the player stands in a type-5 boss room.
      seen.bossRoom ||= (side === 'original' ? row.room.type : row.room.descriptor?.template?.type) === 5;
      seen.admission ||= actors.length > 0;
      seen.damage ||= actors.some(actor => actor.pending > 0 || actor.hp < actor.maxHp);
      for (const actor of actors) if (actor.dead) { dead.add(actor.seed); seen.death = true; }
      seen.retirement ||= [...dead].some(seed => !actors.some(actor => actor.seed === seed));
      locked ||= seen.admission && clear === false;
      seen.clear ||= locked && clear === true;
    }
    result[side] = seen;
  }
  return result;
}
