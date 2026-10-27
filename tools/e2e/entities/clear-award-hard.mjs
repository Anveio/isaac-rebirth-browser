import { pickupFields } from './champion-drop.mjs';

// Ordinary room-clear award at hard (SpawnClearAward 5c9240, 5c9e1c and
// 5c9f93): in these two-Horf rooms the award RNG picks a heart, then draws
// RandomInt(100). 2GQFQMMN draws 16, 8XYCZE9W 23 and PG272DCV 13, so their
// hearts spawn; 6G8SZBDA draws 72 and N30SWLKV 74 (above 34), so they clear
// with no award. The fight leaves no capture inside the clear frame, so the
// clear itself is graded by the captures after it, not by the clear marker.
export default {
  type: 12, variant: 0,
  hard: true,
  about: 'Hard room-clear heart award: kept only on a RandomInt(100) of at most 34',
  contrived: ['2GQFQMMN:3', '8XYCZE9W:3', 'PG272DCV:0', '6G8SZBDA:3', 'N30SWLKV:3'],
  count: 0,
  fight: 3600,
  required: ['admission'],
  roomFields: pickupFields,
};
