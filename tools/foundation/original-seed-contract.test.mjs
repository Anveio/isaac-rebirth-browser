import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { observedOriginalSeed, validateNativeSeedReport, verifyOriginalSeedContract } from "./original-seed-contract.mjs";

test("original random-seed contracts reject empty, incomplete, skipped and duplicate banks", () => {
  const run = { code: "T1PK 6Y0W", seedValue: 2184569263 };
  for (const report of [
    { schema: 1, tested: 0, skipped: 0, runs: [] },
    { schema: 1, tested: 2, skipped: 0, runs: [run] },
    { schema: 1, tested: 1, skipped: 1, runs: [run] },
    { schema: 1, tested: 2, skipped: 0, runs: [run, run] },
    { schema: 1, tested: 1, skipped: 0, runs: [{ ...run, seedValue: 0 }] },
  ]) assert.throws(() => validateNativeSeedReport(report));
  assert.equal(validateNativeSeedReport({ schema: 1, tested: 1, skipped: 0, runs: [{ code: "B911 9VKA", seedValue: 16 }] }).length, 1, "Original seed alphabet includes V");
  assert.throws(() => validateNativeSeedReport({ schema: 1, tested: 1, skipped: 0, runs: [{ code: "B911 95KA", seedValue: 16 }] }), /original alphabet/, "Digit 5 is not in the original seed alphabet");
});

test("actual log parsing requires unique initialized seed, stage and player records", () => {
  const log = "RNG Start Seed: T1PK 6Y0W (2184569263)\nInitialized player with Variant 0 and Subtype 0\nLevel::Init m_Stage 1, m_AltStage 0 Seed 1427977769\n";
  assert.equal(observedOriginalSeed(log).stageSeed, 1427977769);
  assert.throws(() => observedOriginalSeed("seedReadbackMatchesCodec: true"));
  assert.throws(() => observedOriginalSeed(log + log));
  assert.throws(() => observedOriginalSeed(log.replace("m_Stage 1", "m_Stage 2")));
  assert.throws(() => observedOriginalSeed(log.replace("Subtype 0", "Subtype 1")));
});

test("missing/failed original evidence stays red and every selected seed is attempted", async () => {
  const directory = mkdtempSync(join(tmpdir(), "isaac-original-seed-negative-"));
  const path = join(directory, "result.json");
  writeFileSync(path, JSON.stringify({ schema: 1, status: "pass", tested: 2, skipped: 0, runs: [
    { code: "T1PK 6Y0W", seedValue: 2184569263, status: "pass", failures: [], manifest: "missing.json" },
    { code: "FXSW JZBM", seedValue: 1, status: "fail", failures: ["Original launch failed"] },
  ] }));
  const report = await verifyOriginalSeedContract({ nativeReport: path, output: join(directory, "verification") });
  assert.equal(report.status, "fail");
  assert.equal(report.cases.length, 2);
  assert.ok(report.cases.every((item) => item.status === "fail"));
  assert.equal(report.skipped, 0);
  assert.equal(report.originalStateParity, false);
  const absent = await verifyOriginalSeedContract({ output: join(directory, "missing-report") });
  assert.equal(absent.status, "fail");
  assert.match(absent.error, /real original/);
});
