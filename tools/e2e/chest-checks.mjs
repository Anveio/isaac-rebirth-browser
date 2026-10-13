// Actual native Chest callback/child readbacks. Nothing here supplies gameplay.
const vector = value => [Math.fround(value.x), Math.fround(value.y)];
const sorted = values => values.map(value => JSON.stringify(value)).sort();

export function compareChests(check, state, observed, baseline, seeded, label) {
  const room = observed.contents.room, playerBody = observed.contents.players[0];
  check.deepEqual([state.room.grid?.width, state.room.grid?.height, state.room.grid?.cells.length],
    [room.gridWidth, room.gridHeight, room.gridWidth * room.gridHeight], `${label}:complete native room dimensions`);
  check.deepEqual([state.room.left, state.room.right, state.room.top, state.room.bottom],
    [(room.boundsMin[0] + playerBody.collisionRadius) * .65 + 32,
      (room.boundsMax[0] - playerBody.collisionRadius) * .65 + 32,
      (room.boundsMin[1] + playerBody.collisionRadius) * .65 - 46,
      (room.boundsMax[1] - playerBody.collisionRadius) * .65 - 46], `${label}:native radius-inset room bounds`);
  const doors = state.room.grid.cells.flatMap((cell, index) => cell.entity?.kind === 16
    ? [[index, cell.entity.kind, cell.entity.variant, cell.entity.state, cell.entity.collisionClass]] : []);
  const nativeDoors = observed.contents.grid.filter(cell => cell.type === 16)
    .map(cell => [cell.index, cell.type, cell.variant, cell.state, cell.collisionClass]);
  check.deepEqual(sorted(doors), sorted(nativeDoors), `${label}:complete native door object set`);
  check.deepEqual(state.roomDisplayFlags, observed.floor.descriptors.map(room => room.displayFlags),
    `${label}:complete native discovery flags`);
  const native = observed.contents.entities.filter(entity => entity.type === 5);
  const actual = state.room.pickups;
  const tuple = pickup => [pickup.variant, pickup.subtype, pickup.spawnSeed,
    ...vector(pickup.presentedPosition), ...vector(pickup.velocity), pickup.animation,
    Math.fround(pickup.animationFrame), pickup.animationPlaying, pickup.collisionRadius,
    pickup.collisionMass, Math.fround(pickup.currentFriction), pickup.collectionDelay,
    pickup.chest?.gridCollisionClass ?? pickup.regularCollision?.gridCollisionClass ?? 5,
    pickup.chest?.entityCollisionClass ?? pickup.regularCollision?.entityCollisionClass ?? (pickup.collected ? 3 : pickup.canCollect ? 4 : 0),
    pickup.chest?.price ?? 0, pickup.chest?.cooldown ?? 0, pickup.removed];
  const original = pickup => [pickup.variant, pickup.subtype, pickup.spawnSeed,
    ...pickup.position, ...pickup.velocity, pickup.animation, pickup.animationFrame,
    pickup.animationPlaying, pickup.collisionRadius, pickup.collisionMass, pickup.friction,
    pickup.collectionDelay, pickup.gridCollisionClass, pickup.entityCollisionClass,
    pickup.pickupPrice, pickup.pickupCooldown, pickup.removed];
  // Includes every source child, including unsupported pills/trinkets/rare
  // replacements. An empty or partial concrete actor list cannot pass.
  check.deepEqual(sorted(actual.map(tuple)), sorted(native.map(original)),
    `${label}:complete native Chest and actual child actor multiset`);
  for (const chest of native.filter(pickup => pickup.variant === 50)) {
    const model = actual.find(pickup => pickup.variant === 50 && pickup.spawnSeed === chest.spawnSeed);
    check.ok(model?.chest, `${label}:every real Chest has one concrete actor`);
    check.ok(Array.isArray(chest.localRngWords) && chest.localRngWords.length === 4,
      `${label}:actual persistent Chest RNG30 readback is required`);
    if (model?.chest) check.deepEqual([
      model.chest.rng.state, model.chest.rng.rightA, model.chest.rng.left, model.chest.rng.rightB,
    ], chest.localRngWords, `${label}:all four persistent Chest RNG30 words`);
  }
  const nativeOrder = observed.contents.entities.filter(entity => entity.type === 1 || entity.type === 5)
    .map(entity => entity.type === 1 ? ['player'] : ['pickup', entity.variant, entity.spawnSeed]);
  const actualOrder = state.actorScheduler.actors.filter(actor => ['player', 'pickup'].includes(actor.kind))
    .map(actor => {
      if (actor.kind === 'player') return ['player'];
      const pickup = actual.find(value => value.id === actor.id);
      return ['pickup', pickup?.variant, pickup?.spawnSeed];
    });
  check.deepEqual(actualOrder, nativeOrder, `${label}:actual parent/newborn actor admission order`);
  for (const source of observed.chestSounds) {
    const model = state.audio.slots[source.id];
    check.equal(model.loaded, source.loaded, `${label}:native Chest sound${source.id} loading`);
    if (!source.loaded) continue;
    const sourceTick = baseline.audioSystemCounter >> 1;
    const browserTick = seeded.systemClock.systemCounter >> 1;
    // Native virgin lastTick bytes are not an event timestamp. The initialized
    // nextAllowed discriminator is also used by the shared live audio gate.
    check.equal(model.nextAllowed < 0 ? null : model.lastTick - browserTick,
      source.nextAllowed < 0 ? null : source.lastTick - sourceTick,
      `${label}:native Chest sound${source.id} admission phase`);
    check.equal(model.nextAllowed < 0 ? -1 : model.nextAllowed - browserTick,
      source.nextAllowed < 0 ? -1 : source.nextAllowed - sourceTick,
      `${label}:native Chest sound${source.id} cooldown`);
    check.equal(Math.fround(model.lastVolume), source.lastVolume,
      `${label}:native Chest sound${source.id} volume`);
    check.deepEqual(model.samples.map(sample => [sample.looping, Math.fround(sample.gain), Math.fround(sample.pitch)]),
      source.samples.map(sample => [sample.loop, sample.gain, sample.pitch]),
      `${label}:actual Chest sound${source.id} sample settings`);
  }
  const player = observed.contents.players[0];
  check.deepEqual([state.inventory.coins, state.inventory.keys, state.inventory.bombs, state.player.hp, state.damageCooldown],
    [player.coinInventory, player.keyInventory, player.bombInventory, player.redHp, player.damageCountdown],
    `${label}:natural Chest route inventory/health/immunity`);
  return {nativePickups: native, nativeChestSounds: observed.chestSounds,
    nativeSharedMt: observed.sharedMt,
    scope: 'Complete concrete pickup fields, persistent Chest RNG30, actor admission and real SFX21/22 are required. Shared MT is recorded diagnostically; no supplied cursor/vector or total-consumer acceptance.'};
}
