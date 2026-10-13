import { test, expect } from "@playwright/test";
import { waitForReady } from "../foundation/verify.mjs";

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
