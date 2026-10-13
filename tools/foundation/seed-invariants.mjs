import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { fileURLToPath } from "node:url";
import { exactStateDifference } from "./compare.mjs";
import { seedList } from "./seeds.mjs";
import { nativeReplay } from "./verify.mjs";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const key = (code, down) => ({ type: "key", code, down });
const tick = (count) => ({ type: "tick", count });
const press = (code) => [key(code, true), key(code, false)];
const textureDimensions = new Map();
function dimensions(asset) {
  if (!textureDimensions.has(asset)) {
    const bytes = readFileSync(join(ROOT, "references/isaac/rebirth/resources", asset));
    // Original textures can contain bytes following IEND. Dimensions come
    // from their actual IHDR rather than requiring a stricter re-encoding.
    assert.ok(bytes.length >= 33 && bytes.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])) && bytes.toString("ascii", 12, 16) === "IHDR", `Missing PNG dimensions: ${asset}`);
    const width = bytes.readUInt32BE(16), height = bytes.readUInt32BE(20);
    assert.ok(width > 0 && height > 0, `Empty authored texture: ${asset}`);
    // Public native graphics submissions allocate power-of-two textures;
    // authored UV crops may legitimately sample their transparent padding.
    textureDimensions.set(asset, { width, height, allocatedWidth: 2 ** Math.ceil(Math.log2(width)), allocatedHeight: 2 ** Math.ceil(Math.log2(height)) });
  }
  return textureDimensions.get(asset);
}
export function invariantTape(encodedSeed) {
  return [
    ...press("Enter"), ...press("Enter"), ...press("Enter"),
    ...press("Tab"), { type: "text", text: encodedSeed.replaceAll(" ", "") }, ...press("Enter"), ...press("Enter"),
    key("KeyD", true), tick(27), key("KeyD", false), tick(30),
    key("KeyW", true), tick(8), key("KeyW", false), tick(30),
    key("KeyA", true), key("KeyD", true), tick(30), key("KeyA", false), key("KeyD", false),
    key("ArrowRight", true), tick(54), key("ArrowRight", false),
    key("KeyS", true), ...press("Escape"), tick(60), key("KeyS", false), ...press("Escape"), tick(30),
    key("ArrowLeft", true), tick(30), key("ArrowLeft", false),
    key("ArrowUp", true), tick(30), key("ArrowUp", false),
    key("ArrowDown", true), tick(30), key("ArrowDown", false), tick(120),
  ];
}
export async function wasmReplay(initialSeed, commands = []) {
  const bytes = readFileSync(join(ROOT, "web/generated/engine.wasm"));
  const { instance } = await WebAssembly.instantiate(bytes);
  const wasm = instance.exports;
  assert.equal(wasm.abi_version?.(), 1, "Generated WASM ABI is stale; run npm run build:wasm");
  const text = (ptr, length) => new TextDecoder().decode(new Uint8Array(wasm.memory.buffer, ptr, length));
  const resetStatus = wasm.reset(initialSeed);
  assert.equal(resetStatus, 0, `WASM reset ABI failed: ${text(wasm.error_ptr(), wasm.error_len())}`);
  const snapshot = () => { const ptr = wasm.snapshot_ptr(); return JSON.parse(text(ptr, wasm.snapshot_len())); };
  const states = [snapshot()];
  for (const command of commands) {
    const input = new TextEncoder().encode(JSON.stringify(command));
    const ptr = wasm.request_ptr(input.length);
    assert.notEqual(ptr, 0);
    new Uint8Array(wasm.memory.buffer, ptr, input.length).set(input);
    const status = wasm.command();
    assert.equal(status, 0, text(wasm.error_ptr(), wasm.error_len()));
    states.push(snapshot());
  }
  return states;
}
function assertFinite(value, path = "$") {
  if (typeof value === "number") assert.ok(Number.isFinite(value), `Nonfinite state field ${path}`);
  else if (value && typeof value === "object") for (const [key, child] of Object.entries(value)) assertFinite(child, `${path}.${key}`);
}
export function assertGameplayInvariants(initialSeed, commands, states, { manualSeed = true } = {}) {
  assert.equal(states.length, commands.length + 1);
  assert.equal(states[0].initialSeed, initialSeed);
  let playing, shots = 0, pauseChecks = 0, opposingChecks = 0, effectChecks = 0, roomChecks = 0;
  const directionChecks = [];
  const shootingDirections = { ArrowLeft: ["left", "vx", -1], ArrowRight: ["right", "vx", 1], ArrowUp: ["up", "vy", -1], ArrowDown: ["down", "vy", 1] };
  for (let index = 0; index < states.length; index++) {
    const state = states[index];
    assertFinite(state);
    if (state.screen !== "playing") continue;
    playing ||= state;
    assert.equal(state.runSeedValue, initialSeed, "Arbitrary encoded seed must round-trip through the actual seed dialog");
    assert.equal(state.seededRun, manualSeed);
    assert.equal(state.achievementEligibility, !manualSeed);
    assert.ok(state.player.x >= state.room.left && state.player.x <= state.room.right);
    assert.ok(state.player.y >= state.room.top && state.player.y <= state.room.bottom);
    assert.equal(state.player.hp, state.player.maxHp, "This empty tutorial-room tape cannot damage health");
    if (roomChecks === 0) {
      const appearance = state.room.appearance, atlas = dimensions(appearance.backdrop), patch = appearance.backdropPatchSize;
      assert.equal(appearance.floorQuads.length, 4, "Observed starting-room floors contain four submitted quads");
      assert.equal(appearance.wallQuads.length, 44, "Observed starting-room walls contain forty-four submitted quads");
      for (const [quads, domain] of [[appearance.floorQuads, 6], [appearance.wallQuads, 4]]) for (const quad of quads) {
        assert.ok(Number.isInteger(quad.sourcePatchIndex) && quad.sourcePatchIndex >= 0 && quad.sourcePatchIndex < domain, "Backdrop patch selection cannot enter authored placeholder regions");
        const source = quad.sourceRect, destination = quad.destRect;
        assert.ok(source.x >= 0 && source.y >= 0 && source.width > 0 && source.height > 0 && source.x + source.width <= patch.width && source.y + source.height <= patch.height, "Room source crop must lie within its authored patch");
        const atlasX = quad.sourcePatchIndex % appearance.backdropPatchColumns * patch.width + source.x;
        const atlasY = Math.floor(quad.sourcePatchIndex / appearance.backdropPatchColumns) * patch.height + source.y;
        assert.ok(atlasX + source.width <= atlas.width && atlasY + source.height <= atlas.height, "Room crop must lie within the real authored backdrop PNG");
        assert.ok(destination.x >= 0 && destination.y >= 0 && destination.width > 0 && destination.height > 0 && destination.x + destination.width <= appearance.columns * appearance.tileSize && destination.y + destination.height <= appearance.rows * appearance.tileSize, "Room geometry must remain inside its declared cache");
        assert.equal(typeof quad.flipX, "boolean"); assert.equal(typeof quad.flipY, "boolean");
      }
      roomChecks++;
    }
    const effects = state.stageEffects;
    assert.ok(effects, "Playing state must expose generated stage effects");
    assert.equal(effects.maskRaster.pixelSize, 1, "Original mask raster uses one logical pixel");
    assert.equal(effects.maskRaster.rounding, "half-away-from-zero", "Signed mask corners must expose the observed raster rule");
    for (const entry of [...effects.lights, ...effects.shadowCasters]) {
      const destination = entry.destinationRect;
      assert.ok(destination && [destination.x, destination.y, destination.width, destination.height].every(Number.isInteger) && destination.width > 0 && destination.height > 0, "Mask destination rectangles must have finite integer corners and positive raster dimensions");
    }
    const light = effects.lights.find((entry) => entry.ownerId === 0), shadow = effects.shadowCasters.find((entry) => entry.ownerId === 0);
    assert.ok(light && light.radius > 0, "The player must own a nonempty world light");
    assert.equal(light.x, state.player.x, "Light follows the physical player position");
    assert.equal(light.y, state.player.y, "Light follows the physical player position");
    assert.ok(shadow, "The player must own a shadow caster");
    assert.equal(shadow.x, state.player.x + state.player.renderOffset.x, "Shadow follows the authored sprite position");
    assert.equal(shadow.y, state.player.y + state.player.renderOffset.y, "Shadow follows the authored sprite position");
    assert.equal(effects.overlay.simulationTick, state.runTick, "Stage effects use the simulation clock");
    const rectangle = effects.overlay.sourceCrop, texture = dimensions(effects.overlay.asset);
    assert.equal(rectangle.width, effects.viewport.width);
    assert.equal(rectangle.height, effects.viewport.height);
    assert.ok(rectangle.x >= 0 && rectangle.y >= 0 && rectangle.x + rectangle.width <= texture.allocatedWidth && rectangle.y + rectangle.height <= texture.allocatedHeight, "Overlay crop must lie within the original's allocated texture, including authored transparent padding");
    for (const rgba of [effects.ambientColor, effects.groundShadowCompositeRGBA, effects.lightingCompositeRGBA, effects.overlay.modulationRGBA]) assert.ok(rgba.length === 4 && rgba.every((channel) => channel >= 0 && channel <= 1), "Normalized stage colors stay within the color domain");
    effectChecks++;
    assert.equal(new Set(state.tears.map((tear) => tear.id)).size, state.tears.length);
    if (state.tears.length) shots++;
    if (index && commands[index - 1].type === "tick" && states[index - 1].paused) {
      assert.deepEqual(state.player, states[index - 1].player, "Pause must freeze actor state");
      assert.deepEqual(state.tears, states[index - 1].tears, "Pause must freeze tear state");
      assert.deepEqual(state.stageEffects, states[index - 1].stageEffects, "Pause must freeze generated stage effects");
      assert.equal(state.runTick, states[index - 1].runTick);
      pauseChecks++;
    }
    if (index && commands[index - 1].type === "tick" && !states[index - 1].paused) {
      const before = states[index - 1];
      for (const [code, [facing, axis, sign]] of Object.entries(shootingDirections)) {
        if (!before.pressed.includes(code)) continue;
        assert.equal(state.player.facing, facing, `${code}: held shooting must face the requested direction`);
        assert.equal(state.player.shooting, true, `${code}: held shooting must stay active`);
        const newest = state.tears.filter((tear) => tear.id >= before.nextTear).sort((a, b) => b.id - a.id)[0];
        assert.ok(newest, `${code}: this held-shooting batch must create a live new tear`);
        assert.ok(newest[axis] * sign > 0, `${code}: newest live tear must travel in the requested direction`);
        directionChecks.push({ code, commandIndex: index - 1, newestTearId: newest.id });
      }
      if (before.pressed.includes("KeyA") && before.pressed.includes("KeyD")) {
        assert.equal(before.player.vx, 0, "Opposing movement scenario must begin after horizontal inertia has settled");
        assert.equal(state.player.x, before.player.x, "Opposing horizontal keys must cancel rather than choose one side");
        assert.equal(state.player.vx, 0, "Opposing horizontal keys cannot create horizontal velocity");
        opposingChecks++;
      }
    }
  }
  assert.ok(playing, "Random-seed input tape must reach gameplay");
  assert.ok(shots > 0, "Random-seed input tape must emit live tears");
  assert.ok(pauseChecks > 0, "A paused tick command must execute");
  assert.deepEqual(new Set(directionChecks.map((check) => check.code)), new Set(Object.keys(shootingDirections)), "Every shooting direction must execute its own semantic assertions");
  assert.ok(opposingChecks > 0, "Opposing horizontal keys must execute their cancellation assertion");
  assert.ok(states.at(-1).player.x > playing.player.x, "Move-right command must move from the spawn position");
  assert.equal(states.at(-1).pressed.length, 0, "Released keys cannot stay held");
  return { states: states.length, liveShotStates: shots, pauseChecks, directionChecks, opposingChecks, effectChecks, roomChecks };
}

export async function verifySeedInvariants({ seeds: requestedSeeds, output = join(ROOT, ".cache/foundation/random-seed-invariants/latest") } = {}) {
  const seeds = requestedSeeds ?? seedList();
  mkdirSync(output, { recursive: true });
  writeFileSync(join(output, "seeds.json"), JSON.stringify(seeds) + "\n");
  const cases = [];
  for (const initialSeed of seeds) {
    try {
      const initial = (await wasmReplay(initialSeed))[0];
      assert.equal(initial.initialSeed, initialSeed);
      const commands = invariantTape(initial.runSeed);
      const wasm = await wasmReplay(initialSeed, commands), repeated = await wasmReplay(initialSeed, commands), native = nativeReplay(initialSeed, commands);
      for (const [label, states] of [["repeat WASM", repeated], ["native Rust", native]]) {
        assert.equal(states.length, wasm.length);
        for (let index = 0; index < states.length; index++) assert.equal(exactStateDifference(wasm[index], states[index]), null, `${label}: initialSeed ${initialSeed}, command ${index}`);
      }
      const checks = assertGameplayInvariants(initialSeed, commands, wasm);
      const path = join(output, `${initialSeed}.json`);
      writeFileSync(path, JSON.stringify({ initialSeed, commands, states: wasm }) + "\n");
      cases.push({ initialSeed, status: "pass", ...checks, replay: path });
    } catch (error) { cases.push({ initialSeed, status: "fail", error: error.message }); }
  }
  const report = { schema: 1, status: cases.length > 0 && cases.every((item) => item.status === "pass") ? "pass" : "fail", initialSeeds: seeds, seedSource: requestedSeeds !== undefined ? "explicit caller seed list" : process.env.FOUNDATION_TEST_SEEDS ? "explicit replay list" : "OS entropy", cases, scope: "Seed invariants and exact native Rust/WASM replay; original room-generation parity remains unverified", originalStateParity: false };
  writeFileSync(join(output, "report.json"), JSON.stringify(report, null, 2) + "\n");
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const report = await verifySeedInvariants();
  console.log(JSON.stringify(report));
  process.exitCode = report.status === "pass" ? 0 : 1;
}
