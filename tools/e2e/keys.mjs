// Natural secret-room keys: a public starting bomb opens the secret door, then
// movement collects both authored regular keys. Contrived phase offsets on
// JF2DSF92 plus OS-random seeds whose secret room holds the same two keys.
import assert from 'node:assert/strict';
import { compareSound, runRoomFrameSuite } from './lib/room-frames.mjs';

await runRoomFrameSuite({
  name: 'keys',
  helper: 'tools/original/key-scenarios.py',
  observer: 'tools/original/key-observer.py',
  cases: [4, 6],
  scope: 'Secret-room regular key admission, local subtype RNG, full-phase animation/contact/collection, inventory, actor cleanup and native key SFX58 admission. Golden keys, item modifiers, Keeper behavior, shared MT identity, key spending and pixels are out of scope.',

  compare(frame) {
    const { state, source, player, pickups, equal, track, presentation } = frame;
    const doors = source.grid.filter(cell => cell.type === 16);
    equal(state.room.grid.cells.flatMap((cell, index) => cell.entity?.kind === 16 ? [index] : []), doors.map(cell => cell.index), 'door admission');
    for (const door of doors) {
      const actual = state.room.grid.cells[door.index]?.entity;
      if (!actual) continue;
      for (const [field, expected] of [['variant', door.variant], ['state', door.state], ['collisionClass', door.collisionClass]])
        equal(actual[field], expected, `door${door.index}:${field}`);
    }
    track.admitted = Math.max(track.admitted ?? 0, pickups.filter(pickup => pickup.variant === 30).length);
    track.collections ??= [];
    if (player.keyInventory !== (track.keys ?? 0)) {
      track.collections.push({ presentation, amount: player.keyInventory - (track.keys ?? 0) });
      track.keys = player.keyInventory;
      equal(state.systemClock.systemCounter % 2, 1, 'key collection full-only');
    }
    if (track.collections.length) compareSound(frame, 58, source.keySound, 'key sound');
  },

  finish({ last, track, detonation }) {
    assert.equal(track.admitted, 2, 'Both naturally authored original keys are admitted');
    assert.equal(last.inventory.keys, 2, 'Public movement collects both regular keys');
    assert.equal(track.collections.length, 2, 'Two separate native collection contacts');
    assert.equal(last.room.pickups.length, 0, 'Both Collect clips finish and retire their actors');
    assert.ok(detonation !== undefined, 'The public starting bomb reveals the secret doorway');
    assert.equal(last.inventory.bombs, 0, 'The public E action consumes the starting bomb');
  },

  summary: track => ({ admittedKeys: track.admitted, collections: track.collections }),
});
