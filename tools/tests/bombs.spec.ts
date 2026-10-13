import { test, expect } from "@playwright/test";
import { nativeReplay, waitForReady } from "../foundation/verify.mjs";
import type { State } from "../../web/engine";

test("actual bomb key consumes inventory once, preserves paused fuse, and replays blast health in native Rust and WASM", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/"); await waitForReady(page);
  await page.evaluate(() => {
    const game = (window as any).game;
    game.pause(); game.reset(game.snapshot().initialSeed, { trace: true });
  });
  for (let index = 0; index < 4; index++) await page.keyboard.press("Enter");
  await page.keyboard.down("KeyE"); await page.keyboard.down("KeyE");
  const planted = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(planted.inventory.bombs).toBe(0); expect(planted.bombs).toHaveLength(1);
  expect(planted.bombs[0]!.age).toBe(0);
  await page.evaluate(() => (window as any).game.step(40));
  await page.keyboard.press("Escape");
  const paused = await page.evaluate(() => (window as any).game.snapshot()) as State;
  await page.evaluate(() => (window as any).game.step(300));
  const frozen = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(frozen.bombs).toEqual(paused.bombs); expect(frozen.player.hp).toBe(6);
  await page.keyboard.up("KeyE"); await page.keyboard.press("Escape");
  await page.evaluate(() => (window as any).game.step(80));
  const beforeBlast = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(beforeBlast.bombs).toHaveLength(1); expect(beforeBlast.player.hp).toBe(6);
  await page.evaluate(() => (window as any).game.step(1));
  const blast = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(blast.bombs).toHaveLength(0); expect(blast.explosions).toHaveLength(1);
  expect(blast.player.hp).toBe(4);
  await page.evaluate(() => (window as any).game.step(40));
  const ended = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(ended.explosions).toHaveLength(0); expect(ended.player.hp).toBe(4);
  const trace = await page.evaluate(() => (window as any).game.trace());
  expect(trace.enabled).toBe(true); expect(trace.truncated).toBe(false);
  expect(nativeReplay(trace.initialSeed, trace.commands)).toEqual(trace.states);
  expect(errors).toEqual([]);
});
