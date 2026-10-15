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

/** Room lock, manager counts and door states. */
export function roomRows(rows, source, actual) {
  check(rows, 'room.identity', source.room.descriptorIndex, actual.currentRoom);
  check(rows, 'room.grid', [source.room.gridWidth, source.room.gridHeight], actual.room.grid && [actual.room.grid.width, actual.room.grid.height]);
  check(rows, 'room.combatClear', source.room.combatClear, actual.room.combatClear);
  // The Basement overlay's per-room offsets (fxlayers.xml fx 2): the only
  // layered fx entry; the port's origin adds the 663x429 centring offsets.
  const overlay = actual.room.appearance?.fx?.overlayOrigin;
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
  // The browser's own draw record for this capture: what it actually drew.
  check(rows, 'render.recorded', true, render !== undefined);
  // Intact rock-family grid entities: native Rock Render 56bf80 sets frame
  // variant of its type's grid_rock.anm2 animation (Init 56ed00). Destroyed
  // rubble needs per-piece offsets the engine does not model yet, and the
  // other grid kinds' render rules are not decoded; neither is claimed here.
  const rock = cell => ROCK_ANIMATIONS[cell.type] && cell.state === 1 &&
    (cell.type === 2 && cell.variant >= 1000 ? [cell.index, cell.type, 'big', cell.variant - 1000] : [cell.index, cell.type, ROCK_ANIMATIONS[cell.type], cell.variant]);
  check(rows, 'render.rocks', source.grid.map(rock).filter(Boolean),
    render?.grid.filter(drawn => ![7, 14, 16].includes(drawn.kind)).map(drawn => [drawn.index, drawn.kind, drawn.animation, drawn.frame]));
  // Door.LoadGraphics5629c0 overlays door_17_bardoor on variant4; a closed
  // door (AnimState563b70) holds its Idle clip. The opening Vanish fade is
  // not modelled, so only doors not yet opening (state != 2) are claimed.
  check(rows, 'render.doorBars', source.grid.filter(cell => cell.type === 16 && cell.variant === 4 && cell.state !== 2)
    .map(cell => [cell.index, 'Idle']),
    render?.grid.filter(drawn => drawn.kind === 16).map(drawn => [drawn.index, drawn.animation]));
  // Pit Render draws `pit` frame variant (the autotile frame 5684c0 stores).
  // Poop.Init569600 gives every ordinary poop (variant0) a grid_poop sprite;
  // an undrawn one is an invisible wall. Clip choice is not claimed here.
  check(rows, 'render.poops', source.grid.filter(cell => cell.type === 14 && cell.variant === 0).map(cell => cell.index),
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
  const shots = source.entities.filter(actor => actor.type === 9 && !actor.removed);
  check(rows, 'projectile.seeds', shots.map(actor => actor.spawnSeed),
    actual.enemyProjectiles.filter(actor => !actor.dead && !actor.removed).map(actor => actor.spawnSeed));
}

/** Read a side's manifest, raw or from its archived receipt. */
export const loadManifest = (directory, side) => readJsonEvidence(manifestPath(directory, side));

export const MARKERS = ['admission', 'damage', 'death', 'retirement', 'clear', 'shape', 'bossRoom', 'roomType'];

/** Lifecycle markers each game reaches on its own over a whole case. */
export function markers(directory, spec) {
  const result = {};
  for (const side of ['original', 'browser']) {
    const seen = Object.fromEntries(MARKERS.map(name => [name, false]));
    const manifest = loadManifest(directory, side);
    const rows = side === 'original' ? manifest.debugObservations.map(row => row.observation?.gameplay)
      : manifest.frames.map(row => row.state);
    const dead = new Set();
    let locked = false;
    for (const row of rows) {
      if (!row || (side === 'original' ? row.status !== 'pass' : row.screen !== 'playing')) continue;
      const actors = side === 'original'
        ? row.entities.filter(actor => spec.matches(actor.type, actor.variant)).map(actor => ({ seed: actor.spawnSeed, hp: actor.hp, maxHp: actor.maxHp, pending: actor.pendingDamage, dead: actor.removed }))
        : row.room.npcs.filter(actor => !actor.removed && spec.matches(actor.kind, actor.variant)).map(actor => ({ seed: actor.spawnSeed, hp: actor.hp, maxHp: actor.maxHp, pending: actor.damage.pendingDamage, dead: actor.dead }));
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
