import { pickupFields } from './champion-drop.mjs';

// Champion death drops at hard (4f27d8): the champion's own RNG must draw 0
// from RandomInt(3) for its color to drop. The pill champion Horf in 7VFTKQCK
// draws 0 and drops pill 5.70.4 at the same update, seed and f32 position in
// both games; the red champion Horf in 4J3JGA0V draws 2, and both games kill
// it without a heart.
export default {
  type: 12, variant: 0,
  hard: true,
  about: 'Champion death drops at hard difficulty: a drop only when the champion\'s RandomInt(3) is 0',
  contrived: ['7VFTKQCK:0', '4J3JGA0V:0'],
  count: 0,
  fight: 1800,
  // The fight has no captures of its own, so the deaths show in the
  // identity rows of the captures after it rather than as a death marker.
  required: ['admission'],
  roomFields: pickupFields,
};
