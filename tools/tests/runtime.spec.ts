import { test, expect } from "@playwright/test";
import { waitForReady } from "../foundation/verify.mjs";

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
  // Menu transitions and run admission follow the declared native intervals
  // (manualSeedCommands): settle Title, hold each key 12 ticks, then settle.
  const step = (count: number) => page.evaluate((frames) => { (window as any).game.step(frames); }, count);
  const press = async (code: string, settle: number) => { await page.keyboard.down(code); await step(12); await page.keyboard.up(code); await step(settle); };
  await step(63);
  for (let index = 0; index < 3; index++) await press("Enter", 72);
  await page.keyboard.down("KeyE");
  expect((await page.evaluate(() => (window as any).game.snapshot())).pressed).toContain("KeyE");
  await press("Tab", 108); await page.keyboard.up("KeyE");
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
  await press("Enter", 72); await press("Enter", 300);
  const before = await page.evaluate(() => (window as any).game.snapshot());
  expect(before.screen).toBe("playing"); expect(before.inventory.bombs).toBe(1);
  // Bomb placement samples the held action on the next simulation tick.
  await page.keyboard.down("KeyE"); await step(1); await page.keyboard.up("KeyE");
  const after = await page.evaluate(() => (window as any).game.snapshot());
  expect(after.inventory.bombs).toBe(0); expect(after.bombs).toHaveLength(1);
  expect(after.pressed).not.toContain("KeyE");
});
