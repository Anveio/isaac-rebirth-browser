import { pickupFields } from './champion-drop.mjs';

// Ordinary room-clear award at hard (SpawnClearAward 5c9240, 5c9e1c and
// 5c9f93): in these single-Dip rooms the award RNG picks a heart, then draws
// RandomInt(100). HNXKNQ2T draws 14 and G6MD4K0A 2, so their hearts spawn at
// the room centre; WPTB6SQK draws 58 (above 34), so it clears with no award.
// 0JLB1WMP's hard floor rolls Curse of the Labyrinth (the floor that differed
// before the port rolled curses) and holds a Dip room at slot 2. A Curse of
// the Maze floor (EK3SH68W) is left out: ChangeRoom's Maze draws are a gap
// (docs/difficulty.md), so its fight drifts from the original's.
// The fight leaves no capture inside the clear frame, so the clear itself is
// graded by the captures after it, not by the clear marker.
export default {
  type: 217, variant: 0,
  hard: true,
  about: 'Hard room-clear heart award: kept only on a RandomInt(100) of at most 34',
  contrived: ['HNXKNQ2T:2', 'G6MD4K0A:1', 'WPTB6SQK:3', '0JLB1WMP:2'],
  count: 0,
  fight: 3600,
  required: ['admission'],
  roomFields: pickupFields,
};
