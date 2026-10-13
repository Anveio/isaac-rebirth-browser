import { test, expect } from "@playwright/test";
import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { browserScenario, nativeReplay, waitForReady } from "../foundation/verify.mjs";
import { seedList } from "../foundation/seeds.mjs";
import { assertGameplayInvariants } from "../foundation/seed-invariants.mjs";
import { compareStageMaskTargets, loadStageMaskReference, readStageMaskCases } from "../foundation/stage-masks.mjs";
import { inspectBrowserGraphics } from "../foundation/browser.mjs";
import type { Command, State } from "../../web/engine";
const ARTIFACTS = process.env.FOUNDATION_BROWSER_ARTIFACTS ?? "test-results/foundation";

test.beforeAll(async ({ browser }) => {
  const graphics = await inspectBrowserGraphics(browser);
  mkdirSync(ARTIFACTS, { recursive: true });
  writeFileSync(`${ARTIFACTS}/graphics.json`, JSON.stringify(graphics, null, 2));
});

test("actual keyboard opens a seeded run, moves and shoots with exact native Rust/WASM state", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  const run = await browserScenario(page);
  expect(errors).toEqual([]);
  expect(run.frames.map((frame) => frame.state.screen)).toEqual([
    "title", "profile", "main", "character", "seed", "character", "playing", "playing", "playing", "playing",
  ]);
  const initial = run.frames.find((frame) => frame.label === "gameplay-start")!.state;
  const movement = run.frames.find((frame) => frame.label === "movement")!.state;
  const shooting = run.frames.find((frame) => frame.label === "shooting")!.state;
  expect(initial.runSeed.replaceAll(" ", "")).toBe("T1PK6Y0W");
  expect(movement.player.x).toBeGreaterThan(initial.player.x);
  expect(movement.player.y).toBe(initial.player.y);
  expect(shooting.tears.length).toBeGreaterThan(0);
  expect(shooting.tears.every((tear) => tear.vx > 0)).toBe(true);
  expect(shooting.pressed).not.toContain("ArrowRight");
  const build = spawnSync("cargo", ["build", "--locked", "--manifest-path", "engine/Cargo.toml", "--bin", "replay"], { encoding: "utf8" });
  expect(build.status, build.stderr).toBe(0);
  const replay = spawnSync("engine/target/debug/replay", [], { input: JSON.stringify({ initialSeed: run.initialSeed, commands: run.commands }), encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  expect(replay.status, replay.stderr).toBe(0);
  const native: State[] = replay.stdout.trim().split("\n").map((line) => JSON.parse(line));
  expect(native).toEqual(run.states);
  mkdirSync(ARTIFACTS, { recursive: true });
  for (const frame of run.frames) writeFileSync(`${ARTIFACTS}/${frame.label}.png`, Buffer.from(frame.png, "base64"));
});

test("game pause freezes movement and release prevents stuck keyboard input", async ({ page }) => {
  await page.goto("/");
  await browserScenario(page);
  await page.keyboard.down("KeyD");
  await page.keyboard.press("Escape");
  const paused = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(paused.paused).toBe(true);
  await page.evaluate(() => (window as any).game.step(60));
  const after = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(after.player).toEqual(paused.player);
  expect(after.runTick).toBe(paused.runTick);
  await page.keyboard.up("KeyD");
  await page.keyboard.press("Escape");
  await page.evaluate(() => (window as any).game.step(30));
  const released = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(released.paused).toBe(false);
  expect(released.pressed).not.toContain("KeyD");
  expect(released.player.x).toBe(after.player.x);
  expect(released.player.vx).toBe(0);
});

test("releasing a movement key while Control is held clears the actual DOM input state", async ({ page }) => {
  await page.goto("/"); await waitForReady(page);
  await page.evaluate(() => { const game = (window as any).game; game.pause(); game.reset(game.snapshot().initialSeed); });
  for (let index = 0; index < 4; index++) await page.keyboard.press("Enter");
  const before = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(before.screen).toBe("playing");
  await page.keyboard.down("KeyD"); await page.keyboard.down("ControlLeft");
  await page.keyboard.up("KeyD"); await page.keyboard.up("ControlLeft");
  await page.evaluate(() => (window as any).game.step(30));
  const trace = await page.evaluate(() => (window as any).game.trace());
  const after = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(after.pressed).not.toContain("KeyD");
  expect(after.player.x).toBe(before.player.x); expect(after.player.vx).toBe(0);
  expect(trace.commands).toContainEqual({ type: "key", code: "KeyD", down: false });
  expect(trace.commands.some((command: Command) => command.type === "key" && command.code === "ControlLeft")).toBe(false);
  expect(trace.truncated).toBe(false); expect(nativeReplay(trace.initialSeed, trace.commands)).toEqual(trace.states);
});

test("automatic simulation catch-up advances ticks without repeatedly presenting the full canvas in one animation frame", async ({ page }) => {
  await page.addInitScript(() => {
    const observations = { rafCallbacks: 0, canvasPresents: 0, maxTicksInRaf: 0 };
    (window as any).__frameObservations = observations;
    const requestFrame = window.requestAnimationFrame.bind(window);
    window.requestAnimationFrame = (callback) => requestFrame((timestamp) => {
      const game = (window as any).game, before = typeof game?.snapshot === "function" ? game.snapshot().tick : undefined;
      observations.rafCallbacks++; callback(timestamp);
      if (before !== undefined) observations.maxTicksInRaf = Math.max(observations.maxTicksInRaf, game.snapshot().tick - before);
    });
    const drawImage = CanvasRenderingContext2D.prototype.drawImage;
    CanvasRenderingContext2D.prototype.drawImage = function (...arguments_: any[]) {
      const source = arguments_[0];
      if (this.canvas.id === "game" && source instanceof HTMLCanvasElement && source.width === this.canvas.width && source.height === this.canvas.height) observations.canvasPresents++;
      return Reflect.apply(drawImage, this, arguments_);
    };
  });
  await page.goto("/"); await waitForReady(page);
  await page.evaluate(() => { const game = (window as any).game; game.pause(); game.reset(game.snapshot().initialSeed); });
  for (let index = 0; index < 4; index++) await page.keyboard.press("Enter");
  await page.keyboard.down("KeyD");
  const measured = await page.evaluate(async () => {
    const game = (window as any).game, observations = (window as any).__frameObservations;
    observations.rafCallbacks = 0; observations.canvasPresents = 0; observations.maxTicksInRaf = 0;
    const before = game.snapshot(); game.resume();
    await new Promise((done) => setTimeout(done, 50));
    const blockedAt = performance.now(); while (performance.now() - blockedAt < 80) { /* Deliberately force a real catch-up frame. */ }
    await new Promise((done) => setTimeout(done, 100));
    game.pause();
    return { ...observations, ticks: game.snapshot().tick - before.tick, moved: game.snapshot().player.x > before.player.x, forcedMainThreadPauseMs: 80, scope: "Actual RAF callbacks and complete front-canvas composites after forced catch-up; not original-game speed parity" };
  });
  await page.keyboard.up("KeyD");
  expect(measured.rafCallbacks).toBeGreaterThan(0); expect(measured.canvasPresents).toBeGreaterThan(0);
  expect(measured.maxTicksInRaf, "The test must actually exercise multiple logical ticks in one animation frame").toBeGreaterThan(1);
  expect(measured.canvasPresents, "Catch-up ticks must not repeatedly composite the complete displayed frame").toBeLessThanOrEqual(measured.rafCallbacks);
  expect(measured.moved).toBe(true);
  const trace = await page.evaluate(() => (window as any).game.trace());
  expect(trace.truncated).toBe(false); expect(nativeReplay(trace.initialSeed, trace.commands)).toEqual(trace.states);
  mkdirSync(ARTIFACTS, { recursive: true });
  writeFileSync(`${ARTIFACTS}/frame-catchup.json`, JSON.stringify(measured, null, 2));
});

test("OS-random seeds support ordinary and manually seeded browser runs with native/WASM replay", async ({ page }) => {
  const initialSeeds = seedList({ count: 4 });
  test.setTimeout(30_000 + initialSeeds.length * 15_000);
  expect(initialSeeds.length).toBeGreaterThanOrEqual(2);
  mkdirSync(`${ARTIFACTS}/random`, { recursive: true });
  writeFileSync(`${ARTIFACTS}/random/seeds.json`, JSON.stringify(initialSeeds));
  await page.goto("/");
  const unseededImages = new Set<string>(), roomDescriptors = new Set<string>();
  const cases: Array<{ initialSeed: number; manualSeed: boolean; status: "pass" | "fail"; states?: number; directionChecks?: unknown[]; pauseChecks?: number; error?: string }> = [];
  for (const initialSeed of initialSeeds) {
    for (const manualSeed of [false, true]) {
      try {
        const encodedSeed = nativeReplay(initialSeed, [])[0]!.runSeed;
        const run = await browserScenario(page, { initialSeed, manualSeedText: manualSeed ? encodedSeed.replaceAll(" ", "") : null });
        const playing = run.frames.find((frame) => frame.label === "gameplay-start")!;
        expect(playing.state.initialSeed).toBe(initialSeed);
        expect(playing.state.runSeedValue).toBe(initialSeed);
        expect(playing.state.seededRun).toBe(manualSeed);
        expect(playing.state.achievementEligibility).toBe(!manualSeed);
        expect(run.frames.find((frame) => frame.label === "movement")!.state.player.x).toBeGreaterThan(playing.state.player.x);
        expect(run.frames.find((frame) => frame.label === "shooting")!.state.tears.length).toBeGreaterThan(0);
        const down = (code: string): Command => ({ type: "key", code, down: true }), up = (code: string): Command => ({ type: "key", code, down: false });
        const tick = (count: number): Command => ({ type: "tick", count });
        const tail = [down("KeyW"), tick(8), up("KeyW"), tick(30), down("KeyA"), down("KeyD"), tick(30), up("KeyA"), up("KeyD"), down("KeyS"), down("Escape"), up("Escape"), tick(60), up("KeyS"), down("Escape"), up("Escape"), tick(30), down("ArrowLeft"), tick(30), up("ArrowLeft"), down("ArrowUp"), tick(30), up("ArrowUp"), down("ArrowDown"), tick(30), up("ArrowDown"), tick(120)];
        for (const command of tail) {
          if (command.type === "key") await page.keyboard[command.down ? "down" : "up"](command.code);
          else if (command.type === "tick") await page.evaluate((count) => (window as any).game.step(count), command.count);
          run.commands.push(command);
          run.states.push(await page.evaluate(() => (window as any).game.snapshot()));
        }
        const trace = await page.evaluate(() => (window as any).game.trace());
        expect(trace.truncated).toBe(false);
        expect(trace.initialSeed).toBe(initialSeed);
        expect(trace.commands).toEqual(run.commands);
        const invariants = assertGameplayInvariants(initialSeed, run.commands, run.states, { manualSeed });
        expect(invariants.directionChecks.length).toBe(4);
        expect(invariants.pauseChecks).toBeGreaterThan(0);
        expect(nativeReplay(initialSeed, run.commands)).toEqual(run.states);
        if (!manualSeed) {
          unseededImages.add(playing.png);
          roomDescriptors.add(JSON.stringify(playing.state.room.appearance));
        }
        writeFileSync(`${ARTIFACTS}/random/${initialSeed}-${manualSeed ? "manual" : "ordinary"}.json`, JSON.stringify({ initialSeed, commands: run.commands, states: run.states }));
        cases.push({ initialSeed, manualSeed, status: "pass", states: run.states.length, directionChecks: invariants.directionChecks, pauseChecks: invariants.pauseChecks });
      } catch (error) {
        cases.push({ initialSeed, manualSeed, status: "fail", error: error instanceof Error ? error.message : String(error) });
      }
    }
  }
  writeFileSync(`${ARTIFACTS}/random/report.json`, JSON.stringify({ initialSeeds, cases, skipped: 0, scope: "Random-seed browser/native/WASM invariants; original room-generation parity remains unverified" }, null, 2));
  expect(cases.length).toBe(initialSeeds.length * 2);
  expect(cases.filter((item) => item.status === "fail"), "Every selected seed/mode must execute and pass").toEqual([]);
  expect(roomDescriptors.size).toBeGreaterThan(1);
  expect(unseededImages.size, "Rendering must consume changing room generation state, not reuse a captured/preset backdrop").toBeGreaterThan(1);
});

test("production boot consumes real crypto entropy and invalid explicit reset seeds reject atomically", async ({ page }) => {
  await page.addInitScript(() => {
    const original = Crypto.prototype.getRandomValues;
    const values: number[] = [];
    (window as any).__entropySeeds = values;
    Crypto.prototype.getRandomValues = function(array) {
      const result = Reflect.apply(original, this, [array]);
      if (array instanceof Uint32Array && array.length === 1) values.push(array[0]);
      return result;
    };
  });
  await page.goto("/");
  await waitForReady(page);
  const result = await page.evaluate(() => {
    const game = (window as any).game;
    game.pause();
    const before = JSON.stringify(game.snapshot());
    const initialSeed = game.snapshot().initialSeed;
    const rejected: boolean[] = [];
    for (const seed of [0, -1, 0.5, NaN, Infinity, 4294967296]) {
      let failed = false;
      try { game.reset(seed); } catch { failed = true; }
      rejected.push(failed && JSON.stringify(game.snapshot()) === before && game.trace().initialSeed === initialSeed);
    }
    return { initialSeed, entropySeeds: (window as any).__entropySeeds, rejected };
  });
  expect(result.entropySeeds).toContain(result.initialSeed);
  expect(result.rejected).toEqual([true, true, true, true, true, true]);
});

test("original GPU shadow and light masks match all complete native targets for independent seeds and movement", async ({ page }) => {
  const cases = readStageMaskCases();
  test.setTimeout(30_000 + cases.length * 15_000);
  const output = `${ARTIFACTS}/stage-masks`;
  mkdirSync(output, { recursive: true });
  await page.goto("/");
  const results: Array<{ id: string; status: "pass" | "fail"; error?: string; checkpoint?: string }> = [];
  for (const selected of cases) {
    try {
      const reference = loadStageMaskReference(selected);
      let actual: unknown;
      const run = await browserScenario(page, {
        initialSeed: reference.initialSeed, manualSeedText: reference.seedText.replaceAll(" ", ""),
        onCheckpoint: async (frame) => {
          if (frame.label === reference.checkpoint) actual = await page.evaluate(() => (window as any).game.rendererTargets());
        },
      });
      const start = run.frames.find((frame) => frame.label === "gameplay-start")!.state;
      expect(start.runSeedValue).toBe(reference.observed.seedValue);
      expect(start.stage).toBe(reference.observed.stage);
      expect(start.stageSeed).toBe(reference.observed.stageSeed);
      expect(nativeReplay(run.initialSeed, run.commands)).toEqual(run.states);
      expect(actual, `Selected original checkpoint ${reference.checkpoint} must execute`).toBeTruthy();
      const report = compareStageMaskTargets(actual, reference, `${output}/${reference.id}`);
      expect(report.status, `${reference.id}: all original target RGBA bytes must match`).toBe("pass");
      expect(report.originalStateParity).toBe(false);
      results.push({ id: reference.id, status: "pass", checkpoint: reference.checkpoint });
    } catch (error) { results.push({ id: selected.id, status: "fail", error: error instanceof Error ? error.message : String(error) }); }
  }
  writeFileSync(`${output}/report.json`, JSON.stringify({ cases: results, selected: cases.length, skipped: 0, originalStateParity: false, originalSimulationTickDelivery: false }, null, 2));
  expect(results.length).toBe(cases.length);
  expect(results.filter((item) => item.status === "fail"), "Every selected native target case must execute and match").toEqual([]);
});
