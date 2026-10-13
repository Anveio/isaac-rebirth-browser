import assert from "node:assert/strict";
import { globSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseSync } from "rolldown/utils";
import { spawnSync } from "node:child_process";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
export const NODE_TEST_GLOB = "tools/foundation/*.test.mjs";
export const ORIGINAL_CLOCK_SOURCE = "tools/original/foundation-clock-scenarios.py";
export const REQUIRED_ORIGINAL_CLOCK_SCENARIOS = [
  "menu-movement-fire", "menu-run-introduction", "motion-trajectory", "motion-holdout", "projectile-inheritance",
  "bomb-stationary", "bomb-escape", "bomb-pause", "run-pause-presentation",
];

// This indexes declarations, not cached test outcomes. Dynamic registrations
// require an explicit index adapter so they cannot silently vanish from docs.
export function discoverTestCases({ root = ROOT, pattern = NODE_TEST_GLOB } = {}) {
  const files = [...globSync(pattern, { cwd: root })].sort();
  assert.ok(files.length > 0, `No executable scenario sources match ${pattern}`);
  const cases = [];
  for (const source of files) {
    const text = readFileSync(resolve(root, source), "utf8");
    const parsed = parseSync(source, text);
    assert.equal(parsed.errors.length, 0, `Scenario source must parse: ${source}`);
    const declarations = [], testNames = new Set(["test"]);
    for (const node of parsed.program.body) if (node.type === "ImportDeclaration" && ["node:test", "@playwright/test"].includes(node.source.value)) {
      for (const specifier of node.specifiers) if (specifier.imported?.name === "test" || specifier.type === "ImportDefaultSpecifier") testNames.add(specifier.local.name);
    }
    const visit = (node) => {
      if (!node || typeof node !== "object") return;
      if (node.type === "CallExpression") {
        const callee = node.callee;
        if (callee.type === "Identifier" && testNames.has(callee.name)) declarations.push(node);
        if (callee.type === "MemberExpression" && callee.object.type === "Identifier" && testNames.has(callee.object.name)) {
          const method = callee.property.name ?? callee.property.value;
          assert.ok(["before", "after", "beforeAll", "afterAll", "beforeEach", "afterEach", "setTimeout"].includes(method), `Disabled or unsupported test registration in ${source}; every scenario must execute`);
        }
      }
      for (const value of Object.values(node)) if (Array.isArray(value)) value.forEach(visit); else if (value && typeof value === "object") visit(value);
    };
    visit(parsed.program);
    assert.ok(declarations.length > 0, `No test declarations indexed in ${source}`);
    const names = new Set();
    for (const declaration of declarations) {
      const title = declaration.arguments[0], options = declaration.arguments[1];
      assert.ok(title?.type === "Literal" && typeof title.value === "string", `Undocumented dynamic scenario in ${source}; register a source-index adapter`);
      const callback = options?.type === "ObjectExpression" ? declaration.arguments[2] : options;
      assert.ok(["ArrowFunctionExpression", "FunctionExpression"].includes(callback?.type), `Missing or dynamic scenario callback in ${source}; register a source-index adapter`);
      if (options?.type === "ObjectExpression") for (const property of options.properties) {
        assert.equal(property.type, "Property", `Dynamic test options in ${source} cannot establish complete execution`);
        const key = property.key.name ?? property.key.value;
        if (["skip", "todo", "only"].includes(key)) assert.ok(property.value.type === "Literal" && property.value.value === false, `Disabled test options in ${source}; every scenario must execute`);
      }
      const name = title.value;
      assert.ok(name.length > 0 && !names.has(name), `Empty or duplicated scenario title in ${source}`);
      names.add(name);
      cases.push({ source, name, line: text.slice(0, declaration.start).split("\n").length });
    }
  }
  return cases;
}

export function discoverNodeCases({ root = ROOT } = {}) {
  const packageJSON = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8"));
  const patterns = packageJSON.scripts?.["test:verification"]?.match(/\S+\.test\.mjs/g);
  assert.deepEqual(patterns, [NODE_TEST_GLOB], "The Node matrix index must follow every test:verification source glob");
  return discoverTestCases({ root, pattern: NODE_TEST_GLOB });
}

export function validateOriginalClockScenarios(scenarios, { systemIds, requiredIds = REQUIRED_ORIGINAL_CLOCK_SCENARIOS } = {}) {
  assert.ok(Array.isArray(scenarios) && scenarios.length > 0, "Original clock scenario registry cannot be empty");
  assert.ok(Array.isArray(systemIds) && systemIds.length > 0, "Original clock scenarios require a known system task index");
  const ids = new Set(), knownSystems = new Set(systemIds);
  for (const scenario of scenarios) {
    assert.ok(typeof scenario.id === "string" && /^[a-z][a-z0-9-]+$/.test(scenario.id) && !ids.has(scenario.id), "Original clock scenario IDs must be nonempty and unique");
    ids.add(scenario.id);
    assert.ok(typeof scenario.description === "string" && scenario.description.trim(), `Missing description for original clock scenario ${scenario.id}`);
    assert.ok(Array.isArray(scenario.systems) && scenario.systems.length > 0 && new Set(scenario.systems).size === scenario.systems.length, `Missing or duplicate systems for ${scenario.id}`);
    for (const id of scenario.systems) assert.ok(knownSystems.has(id), `Unknown system task ${id} in original clock scenario ${scenario.id}`);
    assert.ok(Array.isArray(scenario.captureLabels) && scenario.captureLabels.length > 0 && scenario.captureLabels.every(label => typeof label === "string" && label.length) && new Set(scenario.captureLabels).size === scenario.captureLabels.length, `Original clock scenario ${scenario.id} must expose unique executable capture labels`);
    assert.ok(Number.isInteger(scenario.presentedFrames) && scenario.presentedFrames > 0, `Original clock scenario ${scenario.id} must execute presentation intervals`);
    assert.ok(Number.isInteger(scenario.inputEvents) && scenario.inputEvents > 0, `Original clock scenario ${scenario.id} must execute input events`);
    assert.equal(scenario.source, ORIGINAL_CLOCK_SOURCE, `Original clock scenario ${scenario.id} must identify its executable tape source`);
  }
  for (const id of requiredIds) assert.ok(ids.has(id), `Missing required original clock scenario ${id}`);
  return scenarios;
}

export function discoverOriginalClockScenarios({ root = ROOT, systemIds } = {}) {
  const source = resolve(root, ORIGINAL_CLOCK_SOURCE);
  const listed = spawnSync("python3", [source, "--list"], { encoding: "utf8", env: { ...process.env, PYTHONDONTWRITEBYTECODE: "1" }, maxBuffer: 2 * 1024 * 1024 });
  assert.equal(listed.status, 0, `Original clock scenario source failed: ${listed.error?.message || listed.stderr}`);
  let scenarios;
  try { scenarios = JSON.parse(listed.stdout); } catch { assert.fail("Original clock scenario source must emit its complete JSON registry"); }
  return validateOriginalClockScenarios(scenarios, { systemIds });
}
