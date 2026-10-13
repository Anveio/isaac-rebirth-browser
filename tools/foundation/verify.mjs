import assert from "node:assert/strict";
import { chromium } from "@playwright/test";
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import { actorForegroundComparison, compareProjectileSprites, compareReferenceSamples, decodePng, exactStateDifference, scaleNearest } from "./compare.mjs";
import { tasksForRegion } from "./systems.mjs";
import { browserLaunchOptions, inspectBrowserGraphics } from "./browser.mjs";

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
export async function browserScenario(page, { initialSeed = FOUNDATION_REFERENCE_INITIAL_SEED, manualSeedText = FOUNDATION_SEED, onCheckpoint } = {}) {
  await waitForReady(page);
  await page.evaluate((initialSeed) => { window.game.pause(); window.game.reset(initialSeed); window.game.pause(); }, initialSeed);
  const commands = [], states = [await page.evaluate(() => window.game.snapshot())], frames = [], timings = [];
  assert.equal(states[0].initialSeed, initialSeed, "Browser reset must use the requested initial seed, never a production preset");
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
  }
  await press("Enter"); await capture("gameplay-start", "playing");
  if (manualSeedText !== null) assert.equal(states.at(-1).seedText.replaceAll(" ", ""), manualSeedText);
  await key("KeyD", true); await tick(27); await key("KeyD", false); await tick(30);
  await capture("movement", "playing");
  await key("ArrowRight", true); await tick(54); await key("ArrowRight", false);
  await capture("shooting", "playing");
  assert.ok(states.at(-1).tears.length > 0, "Held shooting must produce live tears");
  await tick(30);
  await capture("after-fire", "playing");
  const actualTrace = await page.evaluate(() => window.game.trace());
  assert.equal(actualTrace.truncated, false, "Actual keyboard trace must preserve every command");
  assert.equal(actualTrace.initialSeed, initialSeed, "Replays must contain the actual initial seed");
  assert.deepEqual(actualTrace.commands, commands, "Browser DOM input delivery must match the canonical command tape");
  return { initialSeed, commands, states, frames, timings, tickRate: 60, seed: states.at(-1).runSeed, inputTransport: "actual DOM keyboard; manually stepped WASM simulation" };
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
export function nativeReplay(initialSeed, commands) {
  if (!nativeReplayBuilt) {
    const build = spawnSync("cargo", ["build", "--locked", "--manifest-path", "engine/Cargo.toml", "--bin", "replay"], { cwd: ROOT, encoding: "utf8" });
    assert.equal(build.status, 0, build.stderr);
    nativeReplayBuilt = true;
  }
  const replay = spawnSync(join(ROOT, "engine/target/debug/replay"), [], { cwd: ROOT, input: JSON.stringify({ initialSeed, commands }), encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  assert.equal(replay.status, 0, replay.stderr);
  const states = replay.stdout.trim().split("\n").map((line) => JSON.parse(line));
  assert.equal(states.length, commands.length + 1, "Native replay must emit initial and every command state");
  return states;
}

export async function verify({ references, output = join(ROOT, ".cache/foundation/verification/latest"), baseURL: providedURL } = {}) {
  const baseURL = providedURL || process.env.FOUNDATION_BROWSER_URL || "http://127.0.0.1:5173";
  mkdirSync(output, { recursive: true });
  const report = { schema: 1, status: "running", seed: FOUNDATION_SEED, checks: [], exactOriginalPixels: false, originalStateParity: false, originalSimulationTickDelivery: false };
  let server, browser;
  try {
    const original = readReferences(references || [join(ROOT, "references/isaac/foundation/baseline/manifest.json"), join(ROOT, "references/isaac/foundation/repeat/manifest.json")]);
    let ready = false;
    try { ready = (await fetch(baseURL)).ok; } catch {}
    if (!ready) {
      assert.ok(!providedURL && !process.env.FOUNDATION_BROWSER_URL, `Provided foundation browser server is unavailable: ${baseURL}`);
      server = spawn("npm", ["run", "dev"], { cwd: ROOT, stdio: "ignore", detached: true });
      for (let tries = 0; tries < 100 && !ready; tries++) {
        await new Promise((done) => setTimeout(done, 100));
        try { ready = (await fetch(baseURL)).ok; } catch {}
      }
      assert.ok(ready, "Foundation browser server did not become ready");
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
      const native = nativeReplay(run.initialSeed, run.commands);
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
    if (server) { try { process.kill(-server.pid, "SIGTERM"); } catch (error) { if (error.code !== "ESRCH") throw error; } }
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
