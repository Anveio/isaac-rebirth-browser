// Natural regular/double bomb pickups: a public starting bomb opens ordinary
// Rock access, then movement collects every authored bomb pickup. Retained
// scoped routes plus OS-random rooms with unfiltered pickup subtype draws.
import assert from 'node:assert/strict';
import { compareRocks, compareSound, runRoomFrameSuite } from './lib/room-frames.mjs';

await runRoomFrameSuite({
  name: 'bomb-pickups',
  helper: 'tools/original/bomb-pickup-scenarios.py',
  observer: 'tools/original/bomb-pickup-observer.py',
  cases: [4, 8],
  scope: 'Ordinary-room bomb pickup admission with unfiltered source subtype draws, local RNG, full-phase animation/contact/collection, inventory increments, actor cleanup and native SFX201 admission after real Rock access. Observed Troll/Megatroll conversions remain blocking. Inventory cap, item modifiers, blast impulses, special-room prices, shared MT identity and pixels are out of scope.',

  checkBaseline(baseline) {
    assert.equal(baseline.observation.players[0].bombInventory, 1, 'Only the fresh Isaac starting bomb supplies access');
  },

  compare(frame) {
    const { state, source, row, player, pickups, equal, track, presentation } = frame;
    const room = /^(\d+)\.(\d+)\(/.exec(row.context.publicLogState.room);
    assert.ok(room, 'Native public log identifies the current authored room');
    equal([state.room.descriptor?.template.type, state.room.descriptor?.template.variant, state.room.descriptor?.spawnSeed],
      [Number(room[1]), Number(room[2]), row.context.publicLogState.spawnSeed], 'authored room identity');
    const rocks = compareRocks(frame);
    track.destroyedRocks = Math.max(track.destroyedRocks ?? 0, rocks.filter(rock => rock.state === 2).length);

    track.admitted = Math.max(track.admitted ?? 0, pickups.filter(pickup => pickup.variant === 40).length);
    track.admittedSource ??= new Map();
    for (const pickup of pickups) if (!track.admittedSource.has(pickup.spawnSeed)) track.admittedSource.set(pickup.spawnSeed,
      { spawnSeed: pickup.spawnSeed, variant: pickup.variant, subtype: pickup.subtype, position: pickup.position, price: pickup.pickupPrice });
    track.collections ??= []; track.placements ??= []; track.bombs ??= 1;
    if (player.bombInventory > track.bombs) {
      track.collections.push({ presentation, amount: player.bombInventory - track.bombs });
      equal(state.systemClock.systemCounter % 2, 1, 'bomb pickup collection full-only');
    } else if (player.bombInventory < track.bombs) {
      track.placements.push({ presentation, amount: track.bombs - player.bombInventory });
    }
    track.bombs = player.bombInventory;
    if (track.collections.length) compareSound(frame, 201, source.bombSound, 'bomb pickup sound', ['loaded']);
    pickups.forEach((original, index) => {
      equal(original.pickupPrice, 0, `pickup${index}:native free price`);
      equal(state.room.pickups[index]?.collectionDelay, original.collectionDelay, `pickup${index}:collection delay`);
    });
  },

  finish({ last, track, detonation }) {
    assert.equal(track.admitted, 3, 'All naturally authored bomb pickups are admitted');
    assert.equal(track.collections.length, 3, 'Public movement makes three distinct native pickup contacts');
    assert.equal(track.placements.length, 1, 'The public E action spends exactly the starting bomb');
    assert.equal(track.placements[0].amount, 1);
    assert.ok(track.collections.every(contact => [1, 2].includes(contact.amount)), 'Only regular/double callbacks are accepted');
    assert.equal(last.inventory.bombs, track.collections.reduce((sum, contact) => sum + contact.amount, 0), 'Every regular/double amount persists');
    assert.equal(last.room.pickups.length, 0, 'Every Collect clip finishes and retires its actor');
    assert.ok(detonation !== undefined && track.destroyedRocks >= 1, 'The public starting bomb opens ordinary Rock access');
  },

  summary: track => ({ admittedBombPickups: track.admitted, admittedSource: [...(track.admittedSource?.values() ?? [])],
    collections: track.collections, placements: track.placements, destroyedRocks: track.destroyedRocks }),
});
