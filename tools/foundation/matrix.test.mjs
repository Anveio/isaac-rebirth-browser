import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { discoverOriginalClockScenarios, ORIGINAL_CLOCK_SOURCE, REQUIRED_ORIGINAL_CLOCK_SCENARIOS, validateOriginalClockScenarios } from "./scenario-index.mjs";
import { renderMatrix, checkMatrix } from "./matrix.mjs";

const registry = () => REQUIRED_ORIGINAL_CLOCK_SCENARIOS.map((id, index) => ({
  id, description: `Execute ${id}`, systems: ["locomotion"], source: ORIGINAL_CLOCK_SOURCE,
  captureLabels: ["playing", `observed-${index}`], presentedFrames: 60 + index, inputEvents: 4,
}));

test("original clock scenario discovery executes its source and rejects deleted, disabled or stale tape metadata", (t) => {
  const root = mkdtempSync(join(tmpdir(), "isaac-original-scenario-index-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const source = join(root, ORIGINAL_CLOCK_SOURCE);
  mkdirSync(dirname(source), { recursive: true });
  const writeRegistry = (cases) => writeFileSync(source, `import json\nprint(json.dumps(json.loads(${JSON.stringify(JSON.stringify(cases))})))\n`);
  const cases = registry();
  writeRegistry(cases);
  assert.deepEqual(discoverOriginalClockScenarios({ root, systemIds: ["locomotion"] }), cases);
  const added = [...cases, { ...cases[0], id: "diagonal-wall", captureLabels: ["at-wall"], presentedFrames: 120 }];
  writeRegistry(added);
  assert.equal(discoverOriginalClockScenarios({ root, systemIds: ["locomotion"] }).at(-1).id, "diagonal-wall");
  writeRegistry(cases.slice(1));
  assert.throws(() => discoverOriginalClockScenarios({ root, systemIds: ["locomotion"] }), /Missing required original clock scenario menu-movement-fire/);
  writeRegistry([]);
  assert.throws(() => discoverOriginalClockScenarios({ root, systemIds: ["locomotion"] }), /registry cannot be empty/);
  writeFileSync(source, "raise RuntimeError('tape generator is broken')\n");
  assert.throws(() => discoverOriginalClockScenarios({ root, systemIds: ["locomotion"] }), /Original clock scenario source failed/);
  rmSync(source);
  assert.throws(() => discoverOriginalClockScenarios({ root, systemIds: ["locomotion"] }), /Original clock scenario source failed/);
});

test("original scenario index rejects unknown systems and missing executable input or capture contracts", () => {
  const invalid = (change, pattern) => {
    const cases = registry(); change(cases);
    assert.throws(() => validateOriginalClockScenarios(cases, { systemIds: ["locomotion"] }), pattern);
  };
  invalid(cases => { cases[0].systems = ["imaginary-rng"]; }, /Unknown system task imaginary-rng/);
  invalid(cases => { cases[0].captureLabels = []; }, /unique executable capture labels/);
  invalid(cases => { cases[0].captureLabels = ["playing", "playing"]; }, /unique executable capture labels/);
  invalid(cases => { cases[0].presentedFrames = 0; }, /must execute presentation intervals/);
  invalid(cases => { cases[0].inputEvents = 0; }, /must execute input events/);
  invalid(cases => { cases[0].source = "unused-script.py"; }, /executable tape source/);
  invalid(cases => { cases[1].id = cases[0].id; }, /nonempty and unique/);
  for (const id of REQUIRED_ORIGINAL_CLOCK_SCENARIOS) {
    const cases = registry().filter(scenario => scenario.id !== id);
    assert.throws(() => validateOriginalClockScenarios(cases, { systemIds: ["locomotion"] }),
      new RegExp(`Missing required original clock scenario ${id}`));
  }
});

test("matrix checking rejects changes in executable native scenarios until the source-derived table is regenerated", (t) => {
  const root = mkdtempSync(join(tmpdir(), "isaac-matrix-drift-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const path = join(root, "TEST_MATRIX.md"), cases = registry();
  const original = renderMatrix({ originalClockScenarios: cases });
  assert.ok(original.includes("`motion-holdout`"));
  writeFileSync(path, original);
  checkMatrix({ path, rendered: original });
  cases[2].captureLabels.push("diagonal-wall"); cases[2].presentedFrames += 30;
  const changed = renderMatrix({ originalClockScenarios: cases });
  assert.notEqual(changed, original, "Tape capture and timing edits must change their source-derived matrix rows");
  assert.throws(() => checkMatrix({ path, rendered: changed }), /TEST_MATRIX.md is stale/);
  writeFileSync(path, changed);
  checkMatrix({ path, rendered: changed });
});
