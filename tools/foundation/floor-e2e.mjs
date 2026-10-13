import assert from "node:assert/strict";
import { chromium } from "@playwright/test";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { browserLaunchOptions, inspectBrowserGraphics } from "./browser.mjs";
import { startProductionServer } from "./production-server.mjs";
import { assertSeededRun, browserScenario } from "./verify.mjs";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const fields = ["gridIndex", "roomInitSeed", "decorationSeed", "spawnSeed", "followingSeed", "doorMask"];
const templateFields = ["type", "variant", "name", "width", "height"];
const descriptors = (values) => values.map((room) => ({
  ...Object.fromEntries(fields.map((key) => [key, room[key]])), assetStage: room.assetStage,
  template: Object.fromEntries(templateFields.map((key) => [key, room.template[key]])),
}));

export async function floorE2e({ sourceReport, output = join(ROOT, ".cache/foundation/floor-e2e.json") }) {
  const native = JSON.parse(readFileSync(sourceReport, "utf8"));
  assert.equal(native.schema, 1); assert.equal(native.skipped, 0);
  assert.ok(native.cases?.length > 0, "Requires independent original floor cases");
  assert.equal(native.selected, native.cases.length, "All selected source cases must be accounted for");
  const identities = native.cases.map((entry) => entry.runSeed);
  assert.equal(new Set(identities).size, identities.length, "Original identities must be independent");
  const report = { schema: 1, status: "running", sourceReport: resolve(sourceReport), selected: native.cases.length, skipped: 0, cases: [] };
  let server, browser;
  try {
    server = await startProductionServer();
    report.browserArtifact = server.build;
    browser = await chromium.launch({ headless: true, ...browserLaunchOptions() });
    report.graphics = await inspectBrowserGraphics(browser);
    const page = await browser.newPage({ viewport: { width: 960, height: 540 }, deviceScaleFactor: 1 });
    await page.goto(server.url);
    for (const original of native.cases) {
      const result = { seed: original.seed, status: "fail" };
      try {
        assert.equal(original.status, "pass", JSON.stringify(original.error ?? original.differences));
        assert.match(original.nativeBinarySha256, /^[a-f0-9]{64}$/);
        assert.ok(Number.isInteger(original.bootEpochSeed) && original.bootEpochSeed >= 0 && original.bootEpochSeed <= 0xffffffff,
          "Live floor comparison requires the original's recorded public boot input");
        let starting;
        const run = await browserScenario(page, { initialSeed: original.runSeed, bootEpochSeed: original.bootEpochSeed, manualSeedText: original.seed,
          onCheckpoint: async (frame) => { if (frame.label === "gameplay-start") starting = frame.state; } });
        const trace = await page.evaluate(() => window.game.trace());
        const caseDirectory = join(dirname(output), original.seed.replaceAll(" ", ""));
        mkdirSync(caseDirectory, { recursive: true });
        writeFileSync(join(caseDirectory, "browser-input.json"), JSON.stringify({ schema: 1, trace,
          expectedCommands: run.commands, starting }, null, 2) + "\n");
        assert.equal(starting.screen, "playing");
        assert.equal(starting.runSeedValue, original.runSeed);
        assertSeededRun(starting, original.seed, original.runSeed);
        assert.equal(starting.stageSeed, original.stageSeed);
        assert.deepEqual(descriptors(starting.floor.descriptors), original.expected.descriptors,
          "Live browser generated room identities, geometry, masks and all descriptor seeds must match the original");
        assert.deepEqual(starting.floor.grid, original.expected.grid, "Full floor topology must match the original");
        assert.equal(starting.floor.attempts, original.expected.attempts, "Retry count must match the actual native log");
        assert.equal(starting.currentRoom, original.expected.startingDescriptor);
        assert.deepEqual(starting.visitedRooms, [starting.currentRoom]);
        const current = starting.floor.descriptors[starting.currentRoom];
        assert.deepEqual(starting.room.descriptor, current);
        const sourceCurrent = original.expected.descriptors[original.expected.startingDescriptor];
        const expectedDoors = ["left", "up", "right", "down"].flatMap((side, slot) => {
          if ((sourceCurrent.doorMask & (1 << slot)) === 0) return [];
          const targetRoom = original.expected.grid[sourceCurrent.gridIndex + [-1, -13, 1, 13][slot]];
          assert.ok(targetRoom >= 0, "A source connected slot must identify a destination");
          const type = original.expected.descriptors[targetRoom].template.type;
          if ([7, 8].includes(type)) return [];
          return [{ side, targetRoom, state: [2, 12].includes(type) ? "KeyClosed" : "Opened",
            asset: type === 4 ? "gfx/grid/door_02_treasureroomdoor.anm2" : type === 10
              ? "gfx/grid/door_04_selfsacrificeroomdoor.anm2" : "gfx/grid/door_01_normaldoor.anm2" }];
        });
        const visibleDoors = starting.room.appearance.doors.filter(door => door.state !== "Hidden");
        assert.deepEqual(visibleDoors.map(({side, targetRoom, state, asset}) =>
          ({side, targetRoom, state, asset})), expectedDoors,
          "The complete visible doorway set and authored room-type states must match source connections");
        for (const connection of visibleDoors) {
          const slot = ["left", "up", "right", "down"].indexOf(connection.side);
          assert.equal(connection.targetRoom, current.neighbors[slot]);
          const destination = starting.floor.descriptors[connection.targetRoom];
          assert.ok(![7, 8].includes(destination.template.type), "Unopened secret entrance must remain hidden");
        }
        Object.assign(result, { status: "pass", descriptorCount: starting.floor.descriptors.length,
          attempts: starting.floor.attempts, decorationSeed: current.decorationSeed,
          inputProvenance: { status: "pass", commands: trace.commands.length, initialSeed: trace.initialSeed,
            bootEpochSeed: trace.bootEpochSeed, originalBootEpochSeed: original.bootEpochSeed, manualSeed: original.seed },
          inputTransport: "actual DOM keyboard; production WASM; original public X11 tape" });
      } catch (error) { result.error = error.message; }
      report.cases.push(result);
    }
    report.status = report.cases.every((entry) => entry.status === "pass") ? "pass" : "fail";
  } finally {
    await browser?.close(); await server?.close();
    mkdirSync(dirname(output), { recursive: true });
    writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
  }
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const value = (name) => { const at = process.argv.indexOf(name); return at < 0 ? undefined : process.argv[at + 1]; };
  let sourceReport = value("--source-report");
  if (!sourceReport) {
    sourceReport = join(ROOT, ".cache/foundation/floor-e2e-native.json");
    const args = ["tools/original/foundation-floor-verify.py", "--report", sourceReport, "--capture", "--count", value("--count") ?? "2"];
    // These public cases exercise two failed boss fits and a failed treasure fit.
    for (const code of ["QNXPTVT1", "0YAZCZ7N", "BHX1EF8A"]) args.push("--seed", code);
    const captured = spawnSync("python3", args, { cwd: ROOT, encoding: "utf8", timeout: 600_000 });
    assert.equal(captured.status, 0, captured.stderr + captured.stdout);
  }
  const report = await floorE2e({ sourceReport, output: value("--output") });
  console.log(JSON.stringify({ status: report.status, originals: report.cases.length, cases: report.cases }));
  process.exitCode = report.status === "pass" ? 0 : 1;
}
