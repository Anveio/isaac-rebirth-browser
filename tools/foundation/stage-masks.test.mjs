import { test } from "node:test";
import assert from "node:assert/strict";
import { compareStageMaskTargets, loadStageMaskReference, readStageMaskCases } from "./stage-masks.mjs";

test("native stage-mask fixtures require actual seed logs and complete independent GPU target data", () => {
  const cases = readStageMaskCases(), references = cases.map(loadStageMaskReference);
  assert.ok(new Set(references.map((item) => item.original.captureId)).size >= 3, "Neutral masks need independently captured original seeds");
  assert.ok(references.some((item) => item.checkpoint === "movement"), "A movement checkpoint must remain selected");
  assert.throws(() => loadStageMaskReference({ ...cases[0], initialSeed: 1 }), /actual original log/);
  assert.throws(() => loadStageMaskReference({ ...cases[0], original: { ...cases[0].original, profile: { fresh: false, isolated: true } } }), /isolated fresh profile/);
});

test("raw stage-mask equality detects transparent RGB differences and rejects partial or missing targets", () => {
  const bytes = Buffer.alloc(512 * 512 * 4), expected = { targets: { shadow: { width: 512, height: 512, data: bytes }, lighting: { width: 512, height: 512, data: bytes } } };
  const raw = { width: 512, height: 512, rgbaBase64: bytes.toString("base64") };
  const actual = { format: "RGBA8-straight", origin: "top-left-logical", shadow: raw, lighting: raw };
  assert.equal(compareStageMaskTargets(actual, expected).status, "pass");
  const wrongRGB = Buffer.from(bytes); wrongRGB[wrongRGB.length - 4] = 17;
  const mismatch = compareStageMaskTargets({ ...actual, shadow: { ...raw, rgbaBase64: wrongRGB.toString("base64") } }, expected);
  assert.equal(mismatch.status, "fail"); assert.equal(mismatch.results.shadow.changedPixels, 1);
  assert.equal(mismatch.originalStateParity, false);
  assert.throws(() => compareStageMaskTargets({ ...actual, lighting: undefined }, expected), /cannot be skipped/);
  assert.throws(() => compareStageMaskTargets({ ...actual, shadow: { ...raw, rgbaBase64: bytes.subarray(0, bytes.length - 4).toString("base64") } }, expected), /complete target/);
  assert.throws(() => compareStageMaskTargets({ ...actual, format: "premultiplied-RGBA8" }, expected), /straight RGBA/);
});
