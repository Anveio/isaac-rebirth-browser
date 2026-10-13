import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import { pixelDifference } from "./compare.mjs";
import { observedOriginalSeed } from "./original-seed-contract.mjs";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
export const STAGE_MASK_MANIFEST = join(ROOT, "references/isaac/foundation/stage-masks/manifest.json");
const compact = (value) => value.replaceAll(" ", "");
export function readStageMaskCases(path = STAGE_MASK_MANIFEST) {
  const manifest = JSON.parse(readFileSync(path, "utf8"));
  assert.equal(manifest.schema, 1); assert.equal(manifest.format, "RGBA8-straight"); assert.equal(manifest.origin, "top-left-logical");
  assert.equal(manifest.width, 512); assert.equal(manifest.height, 512);
  assert.ok(Array.isArray(manifest.cases) && manifest.cases.length > 0, "Original stage-mask scenarios cannot be empty");
  assert.equal(new Set(manifest.cases.map((item) => item.id)).size, manifest.cases.length, "Stage-mask scenario IDs must be unique");
  return manifest.cases.map((item) => ({ ...item, manifestPath: resolve(path), format: manifest.format, origin: manifest.origin, width: manifest.width, height: manifest.height }));
}

export function loadStageMaskReference(reference) {
  assert.ok(typeof reference.id === "string" && reference.id.length > 0);
  assert.ok(reference.original?.profile?.fresh && reference.original?.profile?.isolated, "Original stage masks require an isolated fresh profile");
  assert.ok(typeof reference.original.captureId === "string" && reference.original.captureId.length > 0);
  assert.ok(["gameplay-start", "movement"].includes(reference.checkpoint), "Stage-mask inputs require an executable checkpoint");
  const directory = dirname(reference.manifestPath);
  const observed = observedOriginalSeed(readFileSync(resolve(directory, reference.original.log), "utf8"));
  assert.equal(observed.seedValue, reference.initialSeed, "Stage-mask fixture seed must match the actual original log");
  assert.equal(compact(observed.seed), compact(reference.seedText));
  for (const [name, value] of Object.entries(observed)) assert.equal(reference.original.initialState?.[name], value, `Stage-mask ${name} must match the actual original log`);
  if (reference.checkpoint === "movement") {
    assert.equal(reference.browserTape?.movementHoldTicks, 27); assert.equal(reference.browserTape?.movementReleaseTicks, 30);
    assert.equal(reference.original.movementInput?.key, "d"); assert.equal(reference.original.movementInput?.requestedHoldMs, 450);
    assert.ok(reference.original.movementInput.releasedAtMs > reference.original.movementInput.atMs);
  }
  const targets = {};
  for (const name of ["shadow", "lighting"]) {
    assert.ok(typeof reference.targets?.[name] === "string", `Missing original ${name} target`);
    const path = resolve(directory, reference.targets[name]), data = readFileSync(path);
    assert.equal(data.length, reference.width * reference.height * 4, `Original ${name} must contain every RGBA byte`);
    targets[name] = { path, width: reference.width, height: reference.height, data };
  }
  return { ...reference, targets, observed };
}

export function compareStageMaskTargets(actual, reference, output) {
  assert.equal(actual.format, "RGBA8-straight", "Stage masks must preserve straight RGBA, including transparent RGB");
  assert.equal(actual.origin, "top-left-logical", "Stage-mask coordinate origins must agree");
  if (output) mkdirSync(output, { recursive: true });
  const results = {};
  for (const name of ["shadow", "lighting"]) {
    const target = actual[name], expected = reference.targets[name];
    assert.ok(target && expected, `Missing ${name} target cannot be skipped`);
    assert.equal(target.width, 512); assert.equal(target.height, 512);
    assert.equal(expected.width, 512); assert.equal(expected.height, 512);
    const data = Buffer.from(target.rgbaBase64, "base64");
    assert.equal(data.length, 512 * 512 * 4, `Browser ${name} must expose the complete target`);
    assert.equal(expected.data.length, data.length, `Original ${name} must expose the complete target`);
    const { difference, ...comparison } = pixelDifference(expected, { width: target.width, height: target.height, data });
    if (output) {
      writeFileSync(join(output, `${name}-browser.png`), PNG.sync.write({ width: target.width, height: target.height, data }));
      writeFileSync(join(output, `${name}-difference.png`), PNG.sync.write(difference));
    }
    results[name] = { ...comparison, passed: comparison.exactRGBA, budget: { changedPixels: 0, meanAbsoluteChannelError: 0, source: "Complete native GPU target bytes; exact equality required" } };
  }
  const report = { status: Object.values(results).every((item) => item.passed) ? "pass" : "fail", format: actual.format, origin: actual.origin, results, originalStateParity: false, originalSimulationTickDelivery: false, scope: "Complete original-referenced shadow/light target RGBA equality; whole-frame checks remain separate and mandatory" };
  if (output) writeFileSync(join(output, "comparison.json"), JSON.stringify(report, null, 2) + "\n");
  return report;
}
