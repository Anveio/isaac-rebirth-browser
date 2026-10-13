import { test } from "node:test";
import assert from "node:assert/strict";
import { seedList } from "./seeds.mjs";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { assertGameplayInvariants, invariantTape, verifySeedInvariants, wasmReplay } from "./seed-invariants.mjs";

test("entropy seeds or a saved list replay native Rust/WASM movement, shooting and pause invariants", async () => {
  const report = await verifySeedInvariants();
  assert.equal(report.status, "pass", JSON.stringify(report));
  assert.equal(report.cases.length, report.initialSeeds.length);
  assert.ok(report.initialSeeds.length > 0);
  for (const item of report.cases) {
    assert.equal(item.status, "pass", `Random seed ${item.initialSeed} failed`);
    assert.equal(item.directionChecks.length, 4, `Random seed ${item.initialSeed} must check all four shooting directions`);
    assert.ok(item.opposingChecks > 0);
    assert.ok(item.effectChecks > 0);
    assert.ok(item.roomChecks > 0);
  }
  assert.equal(report.originalStateParity, false);
});

test("semantic assertions reject aliased controls, incoherent world effects and invalid room geometry", async () => {
  const initialSeed = seedList({ count: 1, replay: null })[0];
  const initial = (await wasmReplay(initialSeed))[0];
  const commands = invariantTape(initial.runSeed);
  const states = await wasmReplay(initialSeed, commands);
  assertGameplayInvariants(initialSeed, commands, states);
  const left = commands.findIndex((command, index) => command.type === "tick" && states[index].pressed.includes("ArrowLeft"));
  assert.ok(left >= 0);
  const aliased = structuredClone(states);
  aliased[left + 1].player.facing = "right";
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, aliased), /ArrowLeft: held shooting must face/);
  const wrongVelocity = structuredClone(states);
  for (const tear of wrongVelocity[left + 1].tears) if (tear.id >= states[left].nextTear) tear.vx = Math.abs(tear.vx);
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, wrongVelocity), /ArrowLeft: newest live tear must travel/);
  const opposite = commands.findIndex((command, index) => command.type === "tick" && states[index].pressed.includes("KeyA") && states[index].pressed.includes("KeyD"));
  assert.ok(opposite >= 0);
  const biased = structuredClone(states);
  biased[opposite + 1].player.x += 1;
  biased[opposite + 1].stageEffects.lights[0].x += 1;
  biased[opposite + 1].stageEffects.shadowCasters[0].x += 1;
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, biased), /Opposing horizontal keys must cancel/);
  const staleLight = structuredClone(states);
  staleLight[left + 1].stageEffects.lights[0].x -= 1;
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, staleLight), /Light follows the physical player/);
  const fractionalRaster = structuredClone(states);
  fractionalRaster[left + 1].stageEffects.lights[0].destinationRect.x += 0.5;
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, fractionalRaster), /finite integer corners/);
  const emptyRaster = structuredClone(states);
  emptyRaster[left + 1].stageEffects.shadowCasters[0].destinationRect.width = 0;
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, emptyRaster), /positive raster dimensions/);
  const invalidCrop = structuredClone(states);
  invalidCrop[left + 1].stageEffects.overlay.sourceCrop.x = 99999;
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, invalidCrop), /Overlay crop must lie within/);
  const wrongClock = structuredClone(states);
  wrongClock[left + 1].stageEffects.overlay.simulationTick += 1;
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, wrongClock), /Stage effects use the simulation clock/);
  const firstPlaying = states.findIndex((state) => state.screen === "playing");
  const placeholderPatch = structuredClone(states);
  placeholderPatch[firstPlaying].room.appearance.floorQuads[0].sourcePatchIndex = 6;
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, placeholderPatch), /authored placeholder regions/);
  const truncatedFloor = structuredClone(states);
  truncatedFloor[firstPlaying].room.appearance.floorQuads.pop();
  assert.throws(() => assertGameplayInvariants(initialSeed, commands, truncatedFloor), /floors contain four submitted quads/);
});

test("saved random-seed lists replay exactly and invalid or duplicate coverage is rejected", () => {
  const seeds = seedList({ count: 4, replay: null });
  assert.equal(seeds.length, 4);
  assert.deepEqual(seedList({ replay: JSON.stringify(seeds) }), seeds);
  for (const invalid of ["[]", "[0]", "[-1]", "[0.5]", "[4294967296]", "[1,1]"])
    assert.throws(() => seedList({ replay: invalid }));
});

test("accepted unsigned seed boundaries execute the full tape without a preset fallback", async (t) => {
  const directory = mkdtempSync(join(tmpdir(), "isaac-seed-boundary-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const report = await verifySeedInvariants({ seeds: [1, 0xffffffff], output: directory });
  assert.equal(report.status, "pass", JSON.stringify(report));
  assert.deepEqual(report.initialSeeds, [1, 0xffffffff]);
  assert.equal(report.seedSource, "explicit caller seed list");
  assert.equal(report.cases.length, 2);
  assert.ok(report.cases.every((item) => item.status === "pass" && item.directionChecks.length === 4));
  await assert.rejects(wasmReplay(0), /seed zero is reserved/, "The built WASM ABI must reject the original game's reserved seed, independently of JS validation");
});
