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
  const step = async (count: number) => page.evaluate(count => (window as any).game.step(count), count);
  const press = async (code: string, settle: number) => {
    await page.keyboard.down(code); await step(12);
    await page.keyboard.up(code); if (settle) await step(settle);
  };
  // Follow the public menu tape's declared intervals, including New Run's
  // deferred admission, while keeping automatic simulation paused.
  await step(63);
  for (const screen of ["profile", "main", "character"]) {
    await press("Enter", 72);
    expect((await page.evaluate(() => (window as any).game.snapshot())).screen).toBe(screen);
  }
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
  await press("Enter", 72);
  const committed = await page.evaluate(() => (window as any).game.snapshot());
  expect(committed.screen).toBe("character");
  expect(committed.seededRun).toBe(true);
  await press("Enter", 300);
  const before = await page.evaluate(() => (window as any).game.snapshot());
  expect(before.screen).toBe("playing"); expect(before.inventory.bombs).toBe(1);
  expect(before.pressed).not.toContain("KeyE");
  expect(before.bombs).toHaveLength(0);
  await press("KeyE", 0);
  const after = await page.evaluate(() => (window as any).game.snapshot());
  expect(after.inventory.bombs).toBe(0); expect(after.bombs).toHaveLength(1);
  expect(after.pressed).not.toContain("KeyE");
});
