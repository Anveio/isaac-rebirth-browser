import { test, expect } from "@playwright/test";
import { nativeReplay, waitForReady } from "../foundation/verify.mjs";
import type { Command, State } from "../../web/engine";

test("independent boot RNG input survives browser traces and native replay, with atomic invalid resets", async ({ page }) => {
  await page.goto("/?manual=1"); await waitForReady(page);
  const trace = await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(3350817387, { trace: true, bootEpochSeed: 0xffffffff });
    game.step(63);
    return game.trace();
  }) as { initialSeed: number; bootEpochSeed: number; commands: Command[]; states: State[] };
  expect(trace.bootEpochSeed).toBe(0xffffffff);
  expect(trace.states.every(state => state.bootEpochSeed === trace.bootEpochSeed)).toBe(true);
  expect(nativeReplay(trace.initialSeed, trace.commands, trace.bootEpochSeed)).toEqual(trace.states);
  const validation = await page.evaluate(() => {
    const game = (window as any).game;
    const before = JSON.stringify({ state: game.snapshot(), trace: game.trace() });
    const errors = [-1, 0x100000000, 0.5, NaN].map(bootEpochSeed => {
      try { game.reset(1, { trace: true, bootEpochSeed }); return null; }
      catch (error) { return String(error); }
    });
    const unchanged = JSON.stringify({ state: game.snapshot(), trace: game.trace() }) === before;
    game.reset(3350817387, { bootEpochSeed: 0 });
    return { errors, unchanged, zero: game.snapshot().bootEpochSeed };
  });
  expect(validation.errors.every(error => error?.includes("Boot epoch seed"))).toBe(true);
  expect(validation.unchanged).toBe(true);
  expect(validation.zero).toBe(0);
});

test("production boot keeps replay history off and reset enables complete evidence explicitly", async ({ page }) => {
  await page.goto("/"); await waitForReady(page);
  const boot = await page.evaluate(() => (window as any).game.trace());
  expect(boot.enabled).toBe(false);
  expect(boot.commands).toEqual([]); expect(boot.states).toEqual([]);
  await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(game.snapshot().initialSeed, { trace: true }); game.step(12);
  });
  const traced = await page.evaluate(() => (window as any).game.trace());
  expect(traced.enabled).toBe(true); expect(traced.truncated).toBe(false);
  expect(traced.commands).toEqual([{ type: "tick", count: 12 }]);
  expect(traced.states.map((state: { tick: number }) => state.tick)).toEqual([0, 12]);
  await page.evaluate(() => {
    const game = (window as any).game;
    game.reset(game.snapshot().initialSeed); game.step(12);
  });
  expect((await page.evaluate(() => (window as any).game.trace())).states).toEqual([]);
});

test("focus lifecycle clears held controls and resumes without undoing explicit diagnostic pause", async ({ page }) => {
  await page.goto("/"); await waitForReady(page);
  await page.keyboard.down("KeyD");
  await page.evaluate(() => window.dispatchEvent(new Event("blur")));
  const suspended = await page.evaluate(() => (window as any).game.snapshot());
  expect(suspended.pressed).not.toContain("KeyD");
  await page.waitForTimeout(120);
  expect((await page.evaluate(() => (window as any).game.snapshot())).tick).toBe(suspended.tick);
  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect.poll(async () => (await page.evaluate(() => (window as any).game.snapshot())).tick).toBeGreaterThan(suspended.tick);
  await page.evaluate(() => {
    (window as any).game.pause();
    window.dispatchEvent(new Event("blur")); window.dispatchEvent(new Event("focus"));
  });
  const manual = await page.evaluate(() => (window as any).game.snapshot());
  await page.waitForTimeout(120);
  expect((await page.evaluate(() => (window as any).game.snapshot())).tick).toBe(manual.tick);
});

test("releasing a held bomb key in seed entry preserves text-only typing and the first gameplay press", async ({ page }) => {
  await page.goto("/"); await waitForReady(page);
  await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(3350817387, { trace: true });
  });
  for (let index = 0; index < 3; index++) await page.keyboard.press("Enter");
  await page.keyboard.down("KeyE");
  expect((await page.evaluate(() => (window as any).game.snapshot())).pressed).toContain("KeyE");
  await page.keyboard.press("Tab"); await page.keyboard.up("KeyE");
  const released = await page.evaluate(() => ({
    state: (window as any).game.snapshot(), trace: (window as any).game.trace(),
  }));
  expect(released.state.screen).toBe("seed");
  expect(released.state.pressed).not.toContain("KeyE");
  expect(released.trace.commands.at(-1)).toEqual({ type: "key", code: "KeyE", down: false });
  const seed = "3BMBMFTE";
  await page.keyboard.type(seed);
  const typed = await page.evaluate(() => (window as any).game.trace());
  expect(typed.commands.slice(released.trace.commands.length)).toEqual([...seed].map(text => ({ type: "text", text })));
  expect(typed.truncated).toBe(false);
  await page.keyboard.press("Enter"); await page.keyboard.press("Enter");
  const before = await page.evaluate(() => (window as any).game.snapshot());
  expect(before.screen).toBe("playing"); expect(before.inventory.bombs).toBe(1);
  await page.keyboard.press("KeyE");
  const after = await page.evaluate(() => (window as any).game.snapshot());
  expect(after.inventory.bombs).toBe(0); expect(after.bombs).toHaveLength(1);
  expect(after.pressed).not.toContain("KeyE");
});
