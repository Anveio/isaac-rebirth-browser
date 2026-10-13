import { test } from "node:test";
import assert from "node:assert/strict";
import { PNG } from "pngjs";
import { actorForegroundComparison, compareProjectileSprites, crop, scaleNearest, pixelDifference, compareReferenceSamples, exactStateDifference } from "./compare.mjs";

function image(width = 8, height = 8) {
  const result = new PNG({ width, height });
  for (let i = 0; i < result.data.length; i += 4) result.data.set([40, 30, 20, 255], i);
  return result;
}
function paint(result, x, y, rgba) { result.data.set(rgba, (y * result.width + x) * 4); return result; }
const actor = { x: 2, y: 2, width: 4, height: 4 };

test("pixel fidelity compares decoded RGBA and reports each changed pixel", () => {
  const expected = image(), actual = image();
  assert.equal(pixelDifference(expected, actual).exactRGBA, true);
  paint(actual, 3, 3, [40, 30, 20, 0]);
  const report = pixelDifference(expected, actual);
  assert.equal(report.exactRGBA, false);
  assert.equal(report.changedPixels, 1);
  assert.equal(report.maxChannelError, 255);
  assert.throws(() => pixelDifference(expected, image(4, 4)), /Capture widths/);
});

test("nearest scaling and exact crops preserve source pixels without filtering", () => {
  const original = paint(image(2, 2), 0, 0, [1, 2, 3, 4]);
  const scaled = scaleNearest(original, 2);
  const tile = crop(scaled, { x: 0, y: 0, width: 2, height: 2 });
  for (let i = 0; i < tile.data.length; i += 4) assert.deepEqual([...tile.data.subarray(i, i + 4)], [1, 2, 3, 4]);
  assert.throws(() => crop(original, { x: -1, y: 0, width: 2, height: 2 }), /outside/);
  assert.throws(() => scaleNearest(original, 1.5), /positive integer/);
});

test("reference comparisons require independent calibration samples and critical regions", () => {
  assert.throws(() => compareReferenceSamples(image(), [image()], { player: actor }), /Two independent original captures/);
  assert.throws(() => compareReferenceSamples(image(), [image(), image()], {}), /Critical pixel regions/);
  assert.throws(() => compareReferenceSamples(image(), [image(), image(16, 16)], { player: actor }), /Complete original\/candidate capture widths/);
});

test("repeatable blank or wrong artwork cannot pass by matching the clone to itself", () => {
  const originals = [image(), image()];
  const wrong = new PNG({ width: 8, height: 8 });
  const result = compareReferenceSamples(wrong, originals, { player: actor });
  assert.equal(result.passed, false);
  assert.equal(result.regions.wholeWindow.budget.changedPixels, 0);
  assert.equal(result.regions.wholeWindow.changedPixels, 64);
  assert.equal(result.regions.player.passed, false);
});

test("native background variability cannot hide incorrect actor pixels in a stable critical region", () => {
  const first = paint(image(), 3, 3, [200, 180, 170, 255]);
  const second = paint(image(), 3, 3, [200, 180, 170, 255]);
  for (const point of [[0, 0], [0, 1], [1, 0], [1, 1]]) paint(second, ...point, [255, 255, 255, 255]);
  const candidate = paint(image(), 4, 3, [200, 180, 170, 255]);
  const result = compareReferenceSamples(candidate, [first, second], { player: actor });
  assert.equal(result.regions.wholeWindow.passed, true);
  assert.equal(result.regions.player.budget.changedPixels, 0);
  assert.equal(result.regions.player.passed, false);
  assert.equal(result.passed, false);
});

test("actor diagnostic compares original foreground RGBA positions, not matching color totals", () => {
  const native = paint(image(), 3, 3, [200, 180, 170, 255]);
  const displaced = paint(image(), 4, 3, [200, 180, 170, 255]);
  const result = actorForegroundComparison(displaced, [native, native], actor);
  assert.equal(result.comparedPixels, 1);
  assert.equal(result.changedPixels, 1);
  assert.equal(result.passed, false);
  assert.throws(() => pixelDifference(image(), image(), new Uint8Array(64)), /empty mask/);
});

test("projectile matching rejects missing sprites, wrong shape/color and positions outside native variation", () => {
  const projectile = (left, color = [70, 170, 220, 255], ring = false) => {
    const result = image(960, 540);
    const size = ring ? 5 : 4;
    for (let y = 0; y < size; y++) for (let x = 0; x < size; x++)
      if (!ring || x === 0 || y === 0 || x === size - 1 || y === size - 1) paint(result, left + x, 200 + y, color);
    return result;
  };
  const originals = [projectile(600), projectile(604)];
  const anchor = [480, 420];
  assert.equal(compareProjectileSprites(projectile(602), originals, anchor).passed, true);
  const missing = compareProjectileSprites(image(960, 540), originals, anchor);
  assert.equal(missing.passed, false);
  assert.match(missing.error, /No regular-tear pixels/);
  const wrongShape = compareProjectileSprites(projectile(600, undefined, true), originals, anchor);
  assert.equal(wrongShape.component.pixels, 16, "Wrong shape deliberately has the same blue pixel count as the native square");
  assert.equal(wrongShape.passed, false);
  assert.equal(compareProjectileSprites(projectile(600, [80, 180, 230, 255]), originals, anchor).passed, false);
  const displaced = compareProjectileSprites(projectile(620), originals, anchor);
  assert.equal(displaced.exactRGBA, true, "An identical sprite at the wrong position is still a failure");
  assert.equal(displaced.passed, false);
});

test("nonexact frames within measured native variability remain explicitly nonexact", () => {
  const first = image(), second = paint(image(), 0, 0, [80, 30, 20, 255]);
  const candidate = paint(image(), 0, 0, [60, 30, 20, 255]);
  const result = compareReferenceSamples(candidate, [first, second], { player: actor });
  assert.equal(result.passed, true);
  assert.equal(result.exactRGBA, false);
  assert.equal(result.regions.wholeWindow.budget.originalPairs, 1);
  assert.equal(result.regions.wholeWindow.budget.changedPixels, 1);
});

test("state equality detects hidden fields, missing fields and numeric drift", () => {
  assert.equal(exactStateDifference({ player: { x: 1 }, rng: 2 }, { player: { x: 1 }, rng: 2 }), null);
  assert.equal(exactStateDifference({ player: { x: 1 }, rng: 2 }, { player: { x: 1 }, rng: 3 }).path, "$.rng");
  assert.equal(exactStateDifference({ player: { x: 1 }, rng: 2 }, { player: { x: 1 } }).path, "$.keys");
  assert.equal(exactStateDifference({ x: 1 }, { x: 1 + Number.EPSILON }).path, "$.x");
});
