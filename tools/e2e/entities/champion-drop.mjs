import { vector } from '../../foundation/entity-compare.mjs';

// Champion death drops (Entity_NPC 4f27a0, docs/difficulty.md): the route's
// closed-loop fight kills a stationary champion Horf and its color drops
// its pickup.
// Every live pickup's identity and position are graded at each capture.
export const pickupFields = (source, actual) => {
  const native = source.entities.filter(row => row.type === 5 && !row.removed)
    .sort((a, b) => a.spawnSeed - b.spawnSeed);
  const port = actual.room.pickups.filter(row => !row.removed && !row.collected)
    .sort((a, b) => a.spawnSeed - b.spawnSeed);
  return [
    ['pickups', native.map(row => [row.variant, row.subtype, row.spawnSeed]),
      port.map(row => [row.variant, row.subtype, row.spawnSeed])],
    ['pickups.position', native.map(row => vector(row.position)), port.map(row => vector(row.position))],
  ];
};

export default {
  type: 12, variant: 0,
  about: 'Champion death drops at normal difficulty: the champion color\'s pickup at a free tile',
  // Champion Horfs fought to death: color 7 (key, EHL7N0AV) drops at the
  // same update with the same identity, seed and f32 position in both games;
  // color 1 (battery, 31T4YVZ3) drops the same pickup and seed at the same
  // update, at its own champion's position, which the fight's knockback had
  // already moved apart. Captures after the clear also grade the ordinary
  // room-clear award (SpawnClearAward 5c9240): a chest and a bomb.
  contrived: ['EHL7N0AV:3', '31T4YVZ3:1'],
  count: 0,
  fight: 1800,
  // The fight has no captures of its own, so the deaths show in the
  // identity rows of the captures after it rather than as a death marker.
  required: ['admission'],
  roomFields: pickupFields,
};
