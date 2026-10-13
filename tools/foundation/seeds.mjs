import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";

// Save this list with a failure and supply FOUNDATION_TEST_SEEDS='[...numbers]'
// to replay that exact coverage. Ordinary runs draw new OS entropy.
export function seedList({ count = 6, replay = process.env.FOUNDATION_TEST_SEEDS } = {}) {
  if (replay !== undefined && replay !== null) {
    const seeds = JSON.parse(replay);
    assert.ok(Array.isArray(seeds) && seeds.length > 0 && seeds.length <= 32, "Seed replay requires 1..32 seeds");
    assert.ok(seeds.every((seed) => Number.isInteger(seed) && seed > 0 && seed <= 0xffffffff), "Every replay seed must be a nonzero unsigned 32-bit integer");
    assert.equal(new Set(seeds).size, seeds.length, "Seed coverage cannot duplicate a seed");
    return seeds;
  }
  assert.ok(Number.isInteger(count) && count > 0 && count <= 32);
  const seeds = new Set();
  while (seeds.size < count) {
    const seed = randomBytes(4).readUInt32LE();
    if (seed !== 0) seeds.add(seed);
  }
  return [...seeds];
}
