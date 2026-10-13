import assert from "node:assert/strict";
import { PNG } from "pngjs";

export function decodePng(value) {
  return Buffer.isBuffer(value) ? PNG.sync.read(value) : PNG.sync.read(Buffer.from(value, "base64"));
}

export function scaleNearest(image, scale) {
  assert.ok(Number.isInteger(scale) && scale > 0, "Capture scale must be a positive integer");
  const output = new PNG({ width: image.width * scale, height: image.height * scale });
  for (let y = 0; y < output.height; y++) for (let x = 0; x < output.width; x++) {
    const source = (Math.floor(y / scale) * image.width + Math.floor(x / scale)) * 4;
    output.data.set(image.data.subarray(source, source + 4), (y * output.width + x) * 4);
  }
  return output;
}

export function crop(image, region) {
  const { x, y, width, height } = region;
  assert.ok([x, y, width, height].every(Number.isInteger) && width > 0 && height > 0, "Invalid integer pixel region");
  assert.ok(x >= 0 && y >= 0 && x + width <= image.width && y + height <= image.height, "Pixel region lies outside capture");
  const result = new PNG({ width, height });
  for (let row = 0; row < height; row++) {
    const offset = ((y + row) * image.width + x) * 4;
    result.data.set(image.data.subarray(offset, offset + width * 4), row * width * 4);
  }
  return result;
}

export function pixelDifference(expected, actual, mask) {
  assert.equal(actual.width, expected.width, "Capture widths must match");
  assert.equal(actual.height, expected.height, "Capture heights must match");
  if (mask) assert.equal(mask.length, expected.width * expected.height, "Mask must describe every candidate pixel");
  let changedPixels = 0, absoluteChannelError = 0, maxChannelError = 0, comparedPixels = 0;
  const difference = new PNG({ width: expected.width, height: expected.height });
  for (let i = 0; i < expected.data.length; i += 4) {
    if (mask && !mask[i / 4]) { difference.data.set([0, 0, 0, 255], i); continue; }
    comparedPixels++;
    let changed = false;
    for (let channel = 0; channel < 4; channel++) {
      const delta = Math.abs(expected.data[i + channel] - actual.data[i + channel]);
      absoluteChannelError += delta;
      maxChannelError = Math.max(maxChannelError, delta);
      changed ||= delta !== 0;
    }
    if (changed) {
      changedPixels++;
      difference.data.set([255, 40, 120, 255], i);
    } else difference.data.set([0, 0, 0, 255], i);
  }
  assert.ok(comparedPixels > 0, "Pixel comparison cannot have an empty mask");
  return {
    exactRGBA: changedPixels === 0, changedPixels, comparedPixels,
    changedFraction: changedPixels / comparedPixels,
    meanAbsoluteChannelError: absoluteChannelError / (comparedPixels * 4),
    maxChannelError, difference,
  };
}

export function actorForegroundComparison(candidate, originals, region) {
  assert.ok(originals.length >= 2, "Actor appearance requires independent original samples");
  const samples = originals.map((image) => crop(image, region));
  const current = crop(candidate, region);
  const mask = new Uint8Array(region.width * region.height);
  for (const sample of samples) for (let pixel = 0; pixel < mask.length; pixel++) {
    const [r, g, b] = sample.data.subarray(pixel * 4, pixel * 4 + 3);
    // Skin and near-black sprite outlines observed in these native room images.
    // We compare their actual RGBA at each coordinate, not their color counts.
    if ((Math.min(r, g, b) > 140 && Math.max(r, g, b) - Math.min(r, g, b) < 40) || Math.max(r, g, b) <= 25) mask[pixel] = 1;
  }
  const variation = [];
  for (let i = 0; i < samples.length; i++) for (let j = i + 1; j < samples.length; j++) variation.push(pixelDifference(samples[i], samples[j], mask));
  const budget = { changedPixels: Math.max(...variation.map((result) => result.changedPixels)), meanAbsoluteChannelError: Math.max(...variation.map((result) => result.meanAbsoluteChannelError)), source: "Native actor foreground sample variation; original-derived skin/outline mask" };
  const differences = samples.map((sample, originalIndex) => ({ originalIndex, ...pixelDifference(sample, current, mask) }));
  const accepted = differences.filter((result) => result.changedPixels <= budget.changedPixels && result.meanAbsoluteChannelError <= budget.meanAbsoluteChannelError);
  const best = (accepted.length ? accepted : differences).toSorted((a, b) => a.meanAbsoluteChannelError - b.meanAbsoluteChannelError)[0];
  return { ...best, budget, passed: accepted.length > 0, scope: "Original actor skin/outline pixels at native coordinates; complementary diagnostic that does not replace full-frame and full actor-region gates" };
}

export function projectileComponents(image, exclude) {
  const pixels = new Set();
  for (let y = 120; y < Math.min(image.height, 450); y++) for (let x = 140; x < Math.min(image.width, 820); x++) {
    if (exclude && x >= exclude.x && x < exclude.x + exclude.width && y >= exclude.y && y < exclude.y + exclude.height) continue;
    const [r, g, b] = image.data.subarray((y * image.width + x) * 4, (y * image.width + x) * 4 + 3);
    if (b - r > 25 && g - r > 15 && b > 100) pixels.add(y * image.width + x);
  }
  const components = [];
  while (pixels.size) {
    const first = pixels.values().next().value;
    pixels.delete(first);
    const pending = [first], component = [];
    while (pending.length) {
      const current = pending.pop(); component.push(current);
      const x = current % image.width, y = Math.floor(current / image.width);
      for (const [nx, ny] of [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]]) {
        const next = ny * image.width + nx;
        if (nx >= 0 && nx < image.width && ny >= 0 && ny < image.height && pixels.delete(next)) pending.push(next);
      }
    }
    const xs = component.map((pixel) => pixel % image.width), ys = component.map((pixel) => Math.floor(pixel / image.width));
    const x = Math.min(...xs), y = Math.min(...ys), width = Math.max(...xs) - x + 1, height = Math.max(...ys) - y + 1;
    // The selected fixture's regular tear is smaller than 30 rendered pixels;
    // large wall splashes are outside the playable-region observer.
    if (width <= 30 && height <= 30) components.push({ x, y, width, height, pixels: component.length, center: [x + width / 2, y + height / 2] });
  }
  return components.toSorted((a, b) => b.pixels - a.pixels);
}

export function compareProjectileSprites(candidate, originals, playerAnchor) {
  assert.ok(originals.length >= 2, "Projectile comparison requires independent original samples");
  const exclude = { x: playerAnchor[0] - 40, y: playerAnchor[1] - 90, width: 80, height: 110 };
  const observed = originals.map((image) => projectileComponents(image, exclude)[0]);
  if (observed.some((component) => !component)) return { passed: false, error: "Original regular-tear sprite was not observed; missing evidence cannot be skipped" };
  const candidates = projectileComponents(candidate, exclude);
  if (!candidates.length) return { passed: false, error: "No regular-tear pixels rendered despite original projectile evidence" };
  const size = Math.max(...observed.flatMap((component) => [component.width, component.height])) + 4;
  const sprite = (image, component) => crop(image, { x: Math.floor(component.center[0] - size / 2), y: Math.floor(component.center[1] - size / 2), width: size, height: size });
  const templates = originals.map((image, index) => sprite(image, observed[index]));
  const mask = new Uint8Array(size * size);
  for (const template of templates) for (let pixel = 0; pixel < mask.length; pixel++) {
    const [r, g, b] = template.data.subarray(pixel * 4, pixel * 4 + 3);
    if ((b - r > 25 && g - r > 15 && b > 100) || Math.max(r, g, b) <= 25) mask[pixel] = 1;
  }
  const variation = [], distances = [];
  const distance = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1]);
  for (let i = 0; i < templates.length; i++) for (let j = i + 1; j < templates.length; j++) {
    variation.push(pixelDifference(templates[i], templates[j], mask));
    distances.push(distance(observed[i].center, observed[j].center));
  }
  const budget = {
    changedPixels: Math.max(...variation.map((result) => result.changedPixels)),
    meanAbsoluteChannelError: Math.max(...variation.map((result) => result.meanAbsoluteChannelError)),
    positionDistancePixels: Math.max(...distances),
    source: "Original normalized sprite RGBA differences and actual native checkpoint position spread",
  };
  const comparisons = [];
  for (const component of candidates) for (let originalIndex = 0; originalIndex < templates.length; originalIndex++) {
    const difference = pixelDifference(templates[originalIndex], sprite(candidate, component), mask);
    const positionDistancePixels = distance(component.center, observed[originalIndex].center);
    comparisons.push({ ...difference, originalIndex, component, positionDistancePixels,
      passed: difference.changedPixels <= budget.changedPixels && difference.meanAbsoluteChannelError <= budget.meanAbsoluteChannelError && positionDistancePixels <= budget.positionDistancePixels });
  }
  const accepted = comparisons.filter((comparison) => comparison.passed);
  const best = (accepted.length ? accepted : comparisons).toSorted((a, b) => a.meanAbsoluteChannelError - b.meanAbsoluteChannelError || a.positionDistancePixels - b.positionDistancePixels)[0];
  return { ...best, budget, passed: accepted.length > 0, originals: observed, scope: "Detected live-tear shape/color RGBA plus original-derived spatial spread; blue pixels only locate components, never establish a match" };
}

// Budgets come only from independent original captures of this same checkpoint.
// A zero-variation original region therefore requires exact decoded pixels.
export function compareReferenceSamples(candidate, originals, regions) {
  assert.ok(Array.isArray(originals) && originals.length >= 2, "Two independent original captures are required for variability calibration");
  assert.ok(regions && Object.keys(regions).length > 0, "Critical pixel regions must be explicit");
  for (const original of originals) {
    assert.equal(original.width, candidate.width, "Complete original/candidate capture widths must match");
    assert.equal(original.height, candidate.height, "Complete original/candidate capture heights must match");
  }
  const views = { wholeWindow: { x: 0, y: 0, width: candidate.width, height: candidate.height }, ...regions };
  const results = {};
  for (const [name, region] of Object.entries(views)) {
    const samples = originals.map((image) => crop(image, region));
    const current = crop(candidate, region);
    const variability = [];
    for (let i = 0; i < samples.length; i++) for (let j = i + 1; j < samples.length; j++) variability.push(pixelDifference(samples[i], samples[j]));
    const budget = {
      changedPixels: Math.max(...variability.map((result) => result.changedPixels)),
      meanAbsoluteChannelError: Math.max(...variability.map((result) => result.meanAbsoluteChannelError)),
      source: "maximum pairwise difference between independent original captures at this checkpoint",
      originalPairs: variability.length,
    };
    const comparisons = samples.map((sample, originalIndex) => ({ originalIndex, ...pixelDifference(sample, current) }));
    const accepted = comparisons.filter((comparison) => comparison.changedPixels <= budget.changedPixels && comparison.meanAbsoluteChannelError <= budget.meanAbsoluteChannelError);
    const closest = (accepted.length ? accepted : comparisons).toSorted((a, b) => a.meanAbsoluteChannelError - b.meanAbsoluteChannelError || a.changedPixels - b.changedPixels)[0];
    results[name] = {
      passed: accepted.length > 0, exactRGBA: closest.exactRGBA,
      originalIndex: closest.originalIndex, changedPixels: closest.changedPixels,
      changedFraction: closest.changedFraction, meanAbsoluteChannelError: closest.meanAbsoluteChannelError,
      maxChannelError: closest.maxChannelError, budget, difference: closest.difference,
    };
  }
  return {
    passed: Object.values(results).every((result) => result.passed),
    exactRGBA: results.wholeWindow.exactRGBA,
    scope: "Original-reference pixel fidelity within measured native checkpoint variability; not simulation-state parity",
    regions: results,
  };
}

export function exactStateDifference(expected, actual, path = "$") {
  if (expected === null || typeof expected !== "object")
    return Object.is(expected, actual) ? null : { path, expected, actual };
  if (actual === null || typeof actual !== "object" || Array.isArray(expected) !== Array.isArray(actual))
    return { path, expected, actual };
  const expectedKeys = Object.keys(expected).sort(), actualKeys = Object.keys(actual).sort();
  if (JSON.stringify(expectedKeys) !== JSON.stringify(actualKeys)) return { path: `${path}.keys`, expected: expectedKeys, actual: actualKeys };
  for (const key of expectedKeys) {
    const difference = exactStateDifference(expected[key], actual[key], `${path}.${key}`);
    if (difference) return difference;
  }
  return null;
}
