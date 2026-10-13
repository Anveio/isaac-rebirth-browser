import { test, expect, type Page } from "@playwright/test";
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { decodePNG, type RawImage } from "../../web/png";
import { waitForReady } from "../foundation/verify.mjs";
import type { State } from "../../web/engine";

const ARTIFACTS = process.env.FOUNDATION_BROWSER_ARTIFACTS ?? "test-results/presentation";

async function seededRun(page: Page): Promise<void> {
  await page.goto("/");
  await waitForReady(page);
  await page.evaluate(() => { const game = (window as any).game; game.pause(); game.reset(2184569263); });
  for (let index = 0; index < 3; index++) await page.keyboard.press("Enter");
  await page.keyboard.press("Tab");
  await page.keyboard.type("T1PK6Y0W");
  await page.keyboard.press("Enter");
  await page.keyboard.press("Enter");
}

async function capture(page: Page, name: string): Promise<RawImage> {
  const { png } = await page.evaluate(() => (window as any).game.capture());
  const bytes = Buffer.from(png, "base64");
  const actual = await decodePNG(bytes);
  mkdirSync(ARTIFACTS, { recursive: true });
  writeFileSync(`${ARTIFACTS}/${name}.png`, bytes);
  return actual;
}

async function compareRegion(actual: RawImage, reference: string, rect: [number, number, number, number]): Promise<number> {
  const expected = await decodePNG(readFileSync(reference));
  expect(actual.width).toBe(expected.width);
  expect(actual.height).toBe(expected.height);
  let changed = 0;
  const [left, top, width, height] = rect;
  for (let y = top; y < top + height; y++) for (let x = left; x < left + width; x++) {
    const at = (y * actual.width + x) * 4;
    if (actual.pixels.subarray(at, at + 4).some((value, channel) => value !== expected.pixels[at + channel])) changed++;
  }
  return changed;
}

test("authored stage banner matches the original text region and leaves gameplay enabled", async ({ page }) => {
  await seededRun(page);
  await page.evaluate(() => (window as any).game.step(16));
  const banner = await capture(page, "stage-banner");
  expect(await compareRegion(banner, "references/isaac/foundation/clock-baseline/frames/gameplay-start.png", [360, 86, 240, 24])).toBe(0);
  const before = await page.evaluate(() => (window as any).game.snapshot()) as State;
  await page.keyboard.down("KeyD");
  await page.evaluate(() => (window as any).game.step(8));
  await page.keyboard.up("KeyD");
  const moving = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(moving.runIntroduction.active).toBe(true);
  expect(moving.player.x).toBeGreaterThan(before.player.x);
  await page.evaluate(() => (window as any).game.step(116));
  expect((await page.evaluate(() => (window as any).game.snapshot()) as State).runIntroduction.active).toBe(false);
});

test("pause composes original paper, actual seed and stats while freezing the run", async ({ page }) => {
  await seededRun(page);
  await page.evaluate(() => (window as any).game.step(140));
  await page.keyboard.press("Escape");
  const before = await page.evaluate(() => (window as any).game.snapshot()) as State;
  await page.evaluate(() => (window as any).game.step(40));
  const paused = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(paused.runTick).toBe(before.runTick);
  expect(paused.player).toEqual(before.player);
  expect(paused.pauseMenu.phase).toBe("idle");
  expect(paused.pauseMenu.selection).toBe(1);
  const image = await capture(page, "pause-idle");
  const reference = "references/isaac/foundation/baseline/frames/pause-seed.png";
  expect(await compareRegion(image, reference, [390, 140, 370, 324])).toBe(0);
  expect(await compareRegion(image, reference, [150, 118, 112, 64])).toBe(0);
  await page.keyboard.press("ArrowDown");
  expect((await page.evaluate(() => (window as any).game.snapshot()) as State).pauseMenu.selection).toBe(2);
  await page.keyboard.press("Escape");
  await page.evaluate(() => (window as any).game.step(22));
  const resumed = await page.evaluate(() => (window as any).game.snapshot()) as State;
  expect(resumed.paused).toBe(false);
  expect(resumed.pauseMenu.phase).toBe("hidden");
});
