import { test } from "node:test";
import assert from "node:assert/strict";
import { existsSync, mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { CHECKPOINTS, compareCapturedFrames, investigationForRegion, readReferences, verify } from "./verify.mjs";
import { PNG } from "pngjs";

const scratch = (t) => {
  const directory = mkdtempSync(join(tmpdir(), "isaac-foundation-test-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
};

test("missing original evidence is an executed pipeline failure, never a skip or clone-only pass", async (t) => {
  const directory = scratch(t);
  const report = await verify({ references: [join(directory, "missing-0.json"), join(directory, "missing-1.json")], output: join(directory, "results") });
  assert.equal(report.status, "fail");
  assert.equal(report.checks.length, 1);
  assert.equal(report.checks[0].status, "fail");
  assert.match(report.checks[0].error, /ENOENT/);
  assert.equal(report.originalStateParity, false);
  assert.equal(report.exactOriginalPixels, false);
});

test("failed pixel regions map to source-indexed tasks without asserting a fabricated cause", () => {
  const result = investigationForRegion("playerAppearanceAndPosition", { passed: false, changedPixels: 12, meanAbsoluteChannelError: 0.5 }, { native: "original.png", browser: "browser.png" });
  assert.ok(result.taskIds.includes("locomotion"));
  assert.ok(result.taskIds.includes("entity-animation"));
  assert.equal(result.status, "requires-reference-verification");
  assert.match(result.interpretation, /does not establish a root cause/);
  for (const task of result.tasks) {
    assert.ok(task.sources.length > 0 && task.fields.length > 0 && task.nextTest);
    assert.deepEqual(task.observedEvidence, { native: "original.png", browser: "browser.png" });
  }
  assert.throws(() => investigationForRegion("undocumented-region", { passed: false }, {}), /Unmapped screenshot difference/);
});

test("a corrupt checkpoint remains a failure while later independent pixel assertions still execute", (t) => {
  const directory = scratch(t), output = join(directory, "results");
  const image = new PNG({ width: 960, height: 540 });
  image.data.fill(255);
  const valid = PNG.sync.write(image), originalPNG = join(directory, "original.png"), corruptPNG = join(directory, "corrupt.png");
  writeFileSync(originalPNG, valid); writeFileSync(corruptPNG, "intentionally corrupt reference fixture");
  const original = [0, 1].map(() => ({ frames: new Map([["title", { path: corruptPNG }], ["profile", { path: originalPNG }]]) }));
  const frames = ["title", "profile"].map((label) => ({ label, png: valid.toString("base64"), state: { screen: label } }));
  const result = compareCapturedFrames(frames, original, output);
  assert.deepEqual(result.checks.map((check) => [check.id, check.status]), [["title", "fail"], ["profile", "pass"]]);
  assert.equal(result.exactOriginalPixels, false);
  assert.ok(existsSync(join(output, "profile/comparison.json")), "The later checkpoint must write actual decoded-RGBA comparison evidence");
  assert.throws(() => compareCapturedFrames([], original, output), /omit every checkpoint/);
  assert.throws(() => compareCapturedFrames([frames[0], frames[0]], original, output), /unique/);
});

test("reference fixtures reject missing checkpoints, wrong seed and shared run directories", (t) => {
  const directory = scratch(t);
  const frame = join(directory, "dummy.png");
  writeFileSync(frame, "schema validation fixture; pixels are not used in this test");
  const fixture = (output) => ({ schema: 1, status: "pass", output, initialState: { seed: "T1PK 6Y0W" }, profile: { fresh: true, isolated: true }, frames: [...CHECKPOINTS, "gameplay-unpaused"].map((label) => ({ label, path: frame, width: 960, height: 540 })) });
  const files = [join(directory, "run-0.json"), join(directory, "run-1.json")];
  const write = (first, second) => { writeFileSync(files[0], JSON.stringify(first)); writeFileSync(files[1], JSON.stringify(second)); };
  const first = fixture("original-0"), second = fixture("original-1");
  write(first, second);
  assert.equal(readReferences(files).length, 2);
  write(
    { ...first, captureId: "frozen-original-baseline", output: undefined, frames: first.frames.map((item) => ({ ...item, path: "dummy.png" })) },
    { ...second, captureId: "frozen-original-repeat", output: undefined, frames: second.frames.map((item) => ({ ...item, path: "dummy.png" })) },
  );
  assert.equal(readReferences(files)[0].frames.get("title").path, frame, "Saved native references resolve frame paths beside their manifest");
  write({ ...first, frames: first.frames.filter((item) => item.label !== "shooting") }, second);
  assert.throws(() => readReferences(files), /Missing original checkpoint shooting/);
  write({ ...first, initialState: { seed: "FXSW JZBM" } }, second);
  assert.throws(() => readReferences(files), /Actual original log seed/);
  write(first, { ...second, output: first.output });
  assert.throws(() => readReferences(files), /independent output\/profile directories/);
  write({ ...first, frames: [...first.frames, first.frames[0]] }, second);
  assert.throws(() => readReferences(files), /duplicated/);
});
