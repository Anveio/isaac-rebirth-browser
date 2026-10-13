import { test, expect } from "@playwright/test";

test("GPU masks preserve straight RGBA blending, zero-alpha rejection and unused viewport padding", async ({ page }) => {
  await page.goto("/");
  await page.waitForFunction(() => typeof (window as any).game?.graphicsContracts === "function");
  const pixels = await page.evaluate(async () => (await (window as any).game.graphicsContracts()).mask) as number[];
  for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++)
    expect(pixels.slice((y * 8 + x) * 4, (y * 8 + x + 1) * 4)).toEqual(
      x < 4 && y < 4 ? [128, 128, 128, 64] : [0, 0, 0, 0],
    );
});

test("GPU screen presentation restores nested targets and keeps the logical top row at the top", async ({ page }) => {
  await page.goto("/");
  await page.waitForFunction(() => typeof (window as any).game?.graphicsContracts === "function");
  const pixels = await page.evaluate(async () => (await (window as any).game.graphicsContracts()).screen) as number[];
  for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++)
    expect(pixels.slice((y * 8 + x) * 4, (y * 8 + x + 1) * 4)).toEqual(
      x === 7 ? [0, 0, 0, 255] : y < 4 ? [0, 0, 255, 255] : [255, 0, 0, 255],
    );
});
