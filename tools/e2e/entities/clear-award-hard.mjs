import { pickupFields } from './champion-drop.mjs';

// Ordinary room-clear award at hard (SpawnClearAward 5c9240, 5c9e1c and
// 5c9f93): in these single-Dip rooms the award RNG picks a heart, then draws
// RandomInt(100). HNXKNQ2T draws 14 and G6MD4K0A 2, so their hearts spawn at
// the room centre; WPTB6SQK draws 58 (above 34), so it clears with no award.
// The fight leaves no capture inside the clear frame, so the clear itself is
// graded by the captures after it, not by the clear marker.
export default {
  type: 217, variant: 0,
  hard: true,
  about: 'Hard room-clear heart award: kept only on a RandomInt(100) of at most 34',
  contrived: ['HNXKNQ2T:2', 'G6MD4K0A:1', 'WPTB6SQK:3'],
  count: 0,
  fight: 3600,
  required: ['admission'],
  roomFields: pickupFields,
};
