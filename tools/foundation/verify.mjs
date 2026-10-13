import assert from "node:assert/strict";
import { chromium } from "@playwright/test";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import { actorForegroundComparison, compareProjectileSprites, compareReferenceSamples, decodePng, exactStateDifference, scaleNearest } from "./compare.mjs";
import { tasksForRegion } from "./systems.mjs";
import { browserLaunchOptions, inspectBrowserGraphics } from "./browser.mjs";
import { startProductionServer } from "./production-server.mjs";
import { CONTROLLED_BOMB_CHECKPOINTS } from "./mask-checkpoints.mjs";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
export const FOUNDATION_SEED = "T1PK6Y0W";
export const FOUNDATION_REFERENCE_INITIAL_SEED = 2184569263;
export const CHECKPOINTS = ["title", "profile", "main", "character", "seed-entry", "character-seeded", "gameplay-start", "movement", "shooting", "after-fire"];
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + "\n");

export function investigationForRegion(region, result, evidence) {
  if (result.passed) return { taskIds: [], tasks: [] };
  const tasks = tasksForRegion(region);
  assert.ok(tasks.length > 0, `Failed region ${region} must identify source-indexed investigation tasks`);
  return {
    taskIds: tasks.map((task) => task.id),
    status: "requires-reference-verification",
    interpretation: "Candidate systems to investigate; this screenshot difference does not establish a root cause",
    observedDifference: { changedPixels: result.changedPixels, meanAbsoluteChannelError: result.meanAbsoluteChannelError, budget: result.budget, error: result.error },
    tasks: tasks.map((task) => ({ ...task, observedEvidence: evidence })),
  };
}

export function readReferences(paths) {
  assert.ok(paths.length >= 2, "At least two real original runs are required; missing references cannot be skipped");
  const references = paths.map((path) => {
    const manifest = JSON.parse(readFileSync(path, "utf8"));
    assert.equal(manifest.schema, 1);
    assert.equal(manifest.status, "pass", JSON.stringify(manifest.failures));
    assert.equal(manifest.initialState?.seed?.replaceAll(" ", ""), FOUNDATION_SEED, "Actual original log seed must match the selected seed");
    assert.ok(manifest.profile?.fresh && manifest.profile?.isolated, "Original captures require isolated fresh profiles");
    assert.ok(Array.isArray(manifest.frames) && manifest.frames.length > 0);
    const frames = new Map(manifest.frames.map((frame) => [frame.label, { ...frame, path: resolve(dirname(resolve(path)), frame.path) }]));
    assert.equal(frames.size, manifest.frames.length, "Original checkpoint labels cannot be duplicated");
    for (const label of [...CHECKPOINTS, "gameplay-unpaused"]) {
      const frame = frames.get(label);
      assert.ok(frame, `Missing original checkpoint ${label}`);
      assert.ok(existsSync(frame.path), `Original frame missing: ${frame.path}`);
      assert.equal(frame.width, 960); assert.equal(frame.height, 540);
    }
    return { path, manifest, frames };
  });
  const identities = references.map((reference) => reference.manifest.captureId || reference.manifest.output);
  assert.ok(identities.every((identity) => typeof identity === "string" && identity.length), "Original runs need their captured run identity");
  assert.equal(new Set(identities).size, references.length, "Calibration runs must have independent output/profile directories");
  return references;
}

// All keys enter through the browser's DOM handlers. Paused manual simulation
// fixes WASM ticks; native video currently supplies wall-clock observations only.
export async function waitForReady(page) {
  await page.waitForFunction(() => ["true", "failed"].includes(document.body.dataset.ready));
  const boot = await page.evaluate(() => ({ ready: document.body.dataset.ready, error: document.querySelector("#status")?.textContent }));
  assert.equal(boot.ready, "true", `Browser boot failed: ${boot.error}`);
}

// The requested setup is independent of the observed trace and browser screen.
export function manualSeedCommands(seedText) {
  const commands = [];
  const press = code => commands.push({ type: "key", code, down: true }, { type: "key", code, down: false });
  for (let index = 0; index < 3; index++) press("Enter");
  if (seedText !== null) {
    press("Tab");
    for (const text of seedText) commands.push({ type: "text", text: text.toUpperCase() });
    press("Enter");
  }
  press("Enter");
  return commands;
}

export function assertSeededRun(state, seedText, seedValue) {
  const code = seedText.replaceAll(" ", "").toUpperCase();
  assert.equal(state.seededRun, true, "Browser must commit the public manual seed entry");
  assert.equal(state.seedValid, true, "Browser must validate the entered seed");
  assert.equal(state.seedText.replaceAll(" ", "").toUpperCase(), code, "Browser must read back every entered seed character");
  assert.equal(state.runSeed.replaceAll(" ", "").toUpperCase(), code, "The committed run code must match the public seed entry");
  if (seedValue !== undefined) assert.equal(state.runSeedValue, seedValue, "The committed run seed must match the declared case");
}

export function assertInputProvenance(trace, commands, initialSeed, bootEpochSeed = 0) {
  assert.equal(trace.enabled, true, "Live verification requires actual input capture");
  assert.equal(trace.inputTraceTruncated, false, "Every requested command from reset must be retained");
  assert.equal(trace.initialSeed, initialSeed, "The input trace must preserve the requested reset seed");
  assert.equal(trace.bootEpochSeed, bootEpochSeed, "The input trace must preserve the independent boot input");
  assert.deepEqual(trace.commands, commands, "Every requested DOM key/text and interval from reset must reach WASM exactly once");
}

export async function browserScenario(page, { initialSeed = FOUNDATION_REFERENCE_INITIAL_SEED, bootEpochSeed = 0, manualSeedText = FOUNDATION_SEED, onCheckpoint, bombCheckpoint } = {}) {
  await waitForReady(page);
  await page.evaluate(({ initialSeed, bootEpochSeed }) => { window.game.pause(); window.game.reset(initialSeed, { trace: true, bootEpochSeed }); window.game.pause(); }, { initialSeed, bootEpochSeed });
  const commands = [], states = [await page.evaluate(() => window.game.snapshot())], frames = [], timings = [];
  assert.equal(states[0].initialSeed, initialSeed, "Browser reset must use the requested initial seed, never a production preset");
  assert.equal(states[0].bootEpochSeed, bootEpochSeed, "Browser reset must preserve the independent boot RNG input");
  const record = async (command) => {
    commands.push(command);
    states.push(await page.evaluate(() => window.game.snapshot()));
  };
  const key = async (code, down) => {
    await page.keyboard[down ? "down" : "up"](code);
    await record({ type: "key", code, down });
  };
  const press = async (code) => { await key(code, true); await key(code, false); };
  const tick = async (count) => {
    const timing = await page.evaluate((count) => {
      const started = performance.now();
      window.game.step(count);
      return performance.now() - started;
    }, count);
    timings.push({ commandIndex: commands.length, ticks: count, durationMs: timing, scope: "Tick command CPU wall time including WASM bridge, state snapshot and Canvas redraw; excludes Playwright transport/GPU completion" });
    await record({ type: "tick", count });
  };
  const capture = async (label, expectedScreen) => {
    const capture = await page.evaluate(() => window.game.capture());
    assert.equal(capture.state.screen, expectedScreen, `${label}: actual keyboard navigation must reach ${expectedScreen}`);
    const decoded = decodePng(capture.png);
    const nativeSize = decoded.width === 480 && decoded.height === 270 ? scaleNearest(decoded, 2) : decoded;
    assert.equal(nativeSize.width, 960); assert.equal(nativeSize.height, 540);
    const frame = { label, ...capture, png: PNG.sync.write(nativeSize).toString("base64") };
    frames.push(frame);
    if (onCheckpoint) await onCheckpoint(frame);
  };
  const finish = async () => {
    const actualTrace = await page.evaluate(() => window.game.trace());
    assertInputProvenance(actualTrace, commands, initialSeed, bootEpochSeed);
    return { initialSeed, bootEpochSeed, commands, states: actualTrace.states, backendEvents: actualTrace.backendEvents,
      frames, timings, tickRate: 60, seed: states.at(-1).runSeed, inputTransport: "actual DOM keyboard; manually stepped WASM simulation; recorded audio backend callbacks" };
  };
  // The native tape reaches its settled Title checkpoint 63 presentations
  // after the first rendered Title snapshot (ANM2 frame 32). Runtime reset
  // starts at that first snapshot, so deliver the intervening presentations.
  await tick(63);
  await capture("title", "title");
  await press("Enter"); await capture("profile", "profile");
  await press("Enter"); await capture("main", "main");
  await press("Enter"); await capture("character", "character");
  if (manualSeedText !== null) {
    await press("Tab");
    for (const text of manualSeedText) {
      await page.keyboard.type(text);
      await record({ type: "text", text });
    }
    await capture("seed-entry", "seed");
    await press("Enter"); await capture("character-seeded", "character");
    assertSeededRun(states.at(-1), manualSeedText);
  }
  await press("Enter");
  const setup = [{ type: "tick", count: 63 }, ...manualSeedCommands(manualSeedText)];
  assert.deepEqual(commands, setup, "The complete menu setup must follow the declared command plan");
  assert.equal(states.at(-1).screen, "playing", "Actual keyboard must start the run before advancing its introduction");
  // Existing neutral original fixtures were captured after the authored stage
  // banner. Advance the explicit transition instead of suppressing its draw.
  for (let steps = 0; states.at(-1).runIntroduction.active; steps += 10) {
    assert.ok(steps < 600, "Run introduction must reach its neutral state within ten simulation seconds");
    await tick(10);
  }
  await capture("gameplay-start", "playing");
  if (manualSeedText !== null) assertSeededRun(states.at(-1), manualSeedText);
  await key("KeyD", true); await tick(27); await key("KeyD", false); await tick(30);
  await capture("movement", "playing");
  if (bombCheckpoint !== undefined) {
    const presentations = CONTROLLED_BOMB_CHECKPOINTS[bombCheckpoint];
    assert.ok(presentations, "Bomb mask verification requires an explicit original checkpoint");
    await key("KeyE", true); await tick(1); await key("KeyE", false); await tick(presentations - 1);
    await capture(bombCheckpoint, "playing");
    return finish();
  }
  await key("ArrowRight", true); await tick(54); await key("ArrowRight", false);
  await capture("shooting", "playing");
  assert.ok(states.at(-1).tears.length > 0, "Held shooting must produce live tears");
  await tick(30);
  await capture("after-fire", "playing");
  return finish();
}

function regionsFor(frame) {
  if (frame.state.screen !== "playing") return {
    menuContent: { x: 80, y: 30, width: 800, height: 480 },
  };
  const player = frame.state.player;
  const x = Math.max(0, Math.min(880, Math.floor(player.x * 2) - 40));
  const y = Math.max(0, Math.min(450, Math.floor(player.y * 2) - 75));
  return {
    roomArtwork: { x: 54, y: 0, width: 852, height: 540 },
    hud: { x: 45, y: 20, width: 220, height: 175 },
    playerAppearanceAndPosition: { x, y, width: 80, height: 90 },
    ...(frame.label === "shooting" ? { projectileAppearanceAndPosition: { x: 570, y: 320, width: 320, height: 145 } } : {}),
  };
}

export function compareCapturedFrames(frames, original, output) {
  assert.ok(frames.length > 0, "Captured-frame comparisons cannot omit every checkpoint");
  assert.equal(new Set(frames.map((frame) => frame.label)).size, frames.length, "Captured checkpoints must be unique");
  const checks = [];
  let allExact = true;
  for (const frame of frames) {
    const directory = join(output, frame.label);
    try {
      mkdirSync(directory, { recursive: true });
      const candidate = decodePng(frame.png);
      writeFileSync(join(directory, "browser.png"), PNG.sync.write(candidate));
      const referenceLabel = frame.label === "gameplay-start" ? "gameplay-unpaused" : frame.label;
      const originals = original.map((reference) => PNG.sync.read(readFileSync(reference.frames.get(referenceLabel).path)));
      const regions = regionsFor(frame);
      const comparison = compareReferenceSamples(candidate, originals, regions);
      const serializable = { ...comparison, regions: {} };
      for (const [name, result] of Object.entries(comparison.regions)) {
        writeFileSync(join(directory, `${name}-difference.png`), PNG.sync.write(result.difference));
        serializable.regions[name] = { ...result, difference: undefined, ...investigationForRegion(name, result, { label: frame.label, referenceLabel, directory }) };
      }
      if (regions.playerAppearanceAndPosition) {
        const actor = actorForegroundComparison(candidate, originals, regions.playerAppearanceAndPosition);
        writeFileSync(join(directory, "actor-foreground-difference.png"), PNG.sync.write(actor.difference));
        serializable.actorForegroundDiagnostic = { ...actor, difference: undefined, ...investigationForRegion("playerAppearanceAndPosition", actor, { label: frame.label, referenceLabel, directory }) };
      }
      if (frame.label === "shooting") {
        const projectile = compareProjectileSprites(candidate, originals, [frame.state.player.x * 2, frame.state.player.y * 2]);
        if (projectile.difference) writeFileSync(join(directory, "projectile-sprite-difference.png"), PNG.sync.write(projectile.difference));
        serializable.projectileSprites = { ...projectile, difference: undefined, ...investigationForRegion("projectileAppearanceAndPosition", projectile, { label: frame.label, referenceLabel, directory }) };
        comparison.passed &&= projectile.passed;
        serializable.passed = comparison.passed;
      }
      save(join(directory, "comparison.json"), serializable);
      save(join(directory, "state-and-performance.json"), { state: frame.state, timingMs: frame.timingMs, timingScope: frame.timingScope });
      checks.push({ id: frame.label, referenceLabel, status: comparison.passed ? "pass" : "fail", exactRGBA: comparison.exactRGBA, artifacts: directory, comparison: serializable });
      allExact &&= comparison.exactRGBA;
    } catch (error) {
      allExact = false;
      checks.push({ id: frame.label, status: "fail", exactRGBA: false, artifacts: directory, error: error.message || String(error), scope: "Checkpoint comparison failed; remaining independent captured checkpoints are still executed" });
    }
  }
  return { checks, exactOriginalPixels: allExact };
}

let nativeReplayBuilt = false;
export function nativeReplay(initialSeed, commands, bootEpochSeed = 0, backendEvents = []) {
  assert.ok(Number.isInteger(bootEpochSeed) && bootEpochSeed >= 0 && bootEpochSeed <= 0xffffffff, "Boot epoch seed must be an unsigned 32-bit value");
  if (!nativeReplayBuilt) {
    const build = spawnSync("cargo", ["build", "--locked", "--manifest-path", "engine/Cargo.toml", "--bin", "replay"], { cwd: ROOT, encoding: "utf8" });
    assert.equal(build.status, 0, build.stderr);
    nativeReplayBuilt = true;
  }
  const timeline = [], publicStateIndices = [0], backendStateIndices = []; let eventIndex = 0;
  assert.ok(Array.isArray(backendEvents) && commands.length + backendEvents.length <= 36000, 'Replay command budget includes audio callbacks');
  for (let i = 0; i <= commands.length; i++) {
    while (eventIndex < backendEvents.length && backendEvents[eventIndex].afterCommandCount === i) {
      const event = backendEvents[eventIndex++];
      assert.equal(event.command.type, 'musicBackend', 'Recorded backend stream accepts actual audio completion/loop callbacks only');
      timeline.push(event.command);
      backendStateIndices.push({ index: timeline.length, state: event.state });
    }
    if (i < commands.length) { timeline.push(commands[i]); publicStateIndices.push(timeline.length); }
  }
  assert.equal(eventIndex, backendEvents.length, 'Audio callback positions must be ordered and inside the public command tape');
  const replay = spawnSync(join(ROOT, "engine/target/debug/replay"), [], { cwd: ROOT, input: JSON.stringify({ initialSeed, bootEpochSeed, commands: timeline }), encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  assert.equal(replay.status, 0, replay.stderr);
  const allStates = replay.stdout.trim().split("\n").map((line) => JSON.parse(line));
  assert.equal(allStates.length, timeline.length + 1, 'Native replay must emit every public and actual audio callback state');
  for (const event of backendStateIndices) if (event.state)
    assert.equal(exactStateDifference(allStates[event.index], event.state), null, `Native Rust/WASM state divergence after actual audio callback ${event.index}`);
  const states = publicStateIndices.map(index => allStates[index]);
  assert.equal(states.length, commands.length + 1, "Native replay must emit initial and every command state");
  return states;
}

export async function verify({ references, output = join(ROOT, ".cache/foundation/verification/latest"), baseURL: providedURL } = {}) {
  let baseURL = providedURL || process.env.FOUNDATION_BROWSER_URL;
  mkdirSync(output, { recursive: true });
  const report = { schema: 1, status: "running", seed: FOUNDATION_SEED, checks: [], exactOriginalPixels: false, originalStateParity: false, originalSimulationTickDelivery: false };
  let server, browser;
  try {
    const original = readReferences(references || [join(ROOT, "references/isaac/foundation/baseline/manifest.json"), join(ROOT, "references/isaac/foundation/repeat/manifest.json")]);
    if (!baseURL) {
      server = await startProductionServer();
      baseURL = server.url;
      report.browserArtifact = server.build;
    } else {
      const ready = await fetch(baseURL, { signal: AbortSignal.timeout(10_000) });
      assert.ok(ready.ok, `Provided foundation browser server is unavailable: ${baseURL}`);
      report.browserArtifact = { source: baseURL, scope: "Explicit external browser server; immutable production provenance is not established" };
    }
    browser = await chromium.launch({ headless: true, ...browserLaunchOptions() });
    report.graphics = await inspectBrowserGraphics(browser);
    const page = await browser.newPage({ viewport: { width: 960, height: 540 }, deviceScaleFactor: 1 });
    const errors = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(baseURL);
    const run = await browserScenario(page);
    assert.deepEqual(run.frames.map((frame) => frame.label), CHECKPOINTS, "Every declared original checkpoint must be captured in order");
    assert.deepEqual(errors, [], "Browser keyboard/render pipeline cannot hide errors");
    save(join(output, "commands.json"), { initialSeed: run.initialSeed, commands: run.commands });
    save(join(output, "wasm-states.json"), run.states);
    save(join(output, "command-performance.json"), { tickRate: run.tickRate, measurements: run.timings, nativePhaseComparison: "Unavailable: native observations are process lifetime CPU/RSS, not corresponding simulation/render phase timers" });
    try {
      const native = nativeReplay(run.initialSeed, run.commands, run.bootEpochSeed, run.backendEvents);
      save(join(output, "rust-states.json"), native);
      for (let index = 0; index < native.length; index++) assert.equal(exactStateDifference(native[index], run.states[index]), null, `Native Rust/WASM state divergence after command ${index}`);
      report.checks.push({ id: "actual-keyboard-menu-seed-movement-shooting", status: "pass", commands: run.commands.length, statesCompared: native.length, scope: "Exact native Rust/WASM foundation state; original game hidden state is not observed" });
    } catch (error) { report.checks.push({ id: "actual-keyboard-menu-seed-movement-shooting", status: "fail", error: error.message }); }
    const comparisons = compareCapturedFrames(run.frames, original, output);
    report.checks.push(...comparisons.checks);
    report.exactOriginalPixels = comparisons.exactOriginalPixels;
    report.original = original.map((reference) => ({ manifest: reference.path, captureClock: reference.manifest.captureClock, simulationTicks: reference.manifest.simulationTicks, fullInternalState: reference.manifest.fullInternalState, performance: reference.manifest.performance }));
    report.performance = { browserCommands: run.timings, browserCapturePhases: run.frames.map((frame) => ({ label: frame.label, timingMs: frame.timingMs, scope: frame.timingScope })), comparison: "Browser CPU command/draw/capture measurements and native process lifetime CPU/RSS have different scopes; no comparative phase-speed claim" };
    report.scope = "Real-keyboard foundation menu/run state and original-referenced full-frame/critical-region pixels; visible fidelity budgets measured from original repeat captures. Native wall-clock input is not simulation lockstep.";
  } catch (error) {
    report.checks.push({ id: "foundation-pipeline", status: "fail", error: error.message || String(error) });
  } finally {
    await browser?.close();
    await server?.close();
  }
  report.status = report.checks.length > 0 && report.checks.every((check) => check.status === "pass") ? "pass" : "fail";
  save(join(output, "report.json"), report);
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const value = (flag) => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
  const report = await verify({ references: value("--references")?.split(",").map((path) => resolve(path)), output: value("--output") ? resolve(value("--output")) : undefined, baseURL: value("--base-url") });
  console.log(JSON.stringify({ status: report.status, checks: report.checks.length, failed: report.checks.filter((check) => check.status === "fail").length, exactOriginalPixels: report.exactOriginalPixels, originalStateParity: false }));
  process.exitCode = report.status === "pass" ? 0 : 1;
}
