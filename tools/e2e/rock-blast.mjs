// Ordinary Rock destruction: public movement and a starting bomb blast authored
// Rock-family grids. Contrived routes, retained regressions and OS-random
// rooms with ordinary rocks in reach.
import assert from 'node:assert/strict';
import { compareRocks, runRoomFrameSuite } from './lib/room-frames.mjs';

await runRoomFrameSuite({
  name: 'rock-blast',
  helper: 'tools/original/rock-blast-scenarios.py',
  observer: 'tools/original/foundation-pickup-observe.py',
  cases: [8, 11],
  scope: 'Ordinary Rock admission, movement blockage, bomb placement/detonation, sorted destruction/link repair, marker aging and passage through former cells. Tinted Rock admission/drops remain blocking regressions; cosmetic effect actors, shared MT alignment and pixels are out of scope.',

  compare({ track, ...frame }) {
    const rocks = compareRocks(frame, { marker: 'pathCost' });
    track.admitted = Math.max(track.admitted ?? 0, rocks.length);
    track.destroyed = Math.max(track.destroyed ?? 0, rocks.filter(rock => rock.state === 2).length);
  },

  finish({ states, track, detonation }) {
    assert.ok(track.admitted >= 1, 'Rock grids are naturally admitted');
    assert.ok(track.destroyed >= 1, 'An original Rock is destroyed');
    assert.ok(detonation !== undefined, 'A player bomb detonates through public input');
    assert.equal(states[detonation - 1].systemClock.systemCounter % 2, 1, 'Bomb detonation is full-only');
  },

  summary: track => ({ admittedRocks: track.admitted, destroyedRocks: track.destroyed }),
});
