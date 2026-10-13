import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { decodePng, exactStateDifference } from "./compare.mjs";
import { assertGameplayInvariants, invariantTape, wasmReplay } from "./seed-invariants.mjs";
import { nativeReplay } from "./verify.mjs";

const ROOT = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const compact = (seed) => seed.replaceAll(" ", "");
export function validateNativeSeedReport(report) {
  assert.equal(report.schema, 1);
  assert.ok(Array.isArray(report.runs) && report.runs.length > 0, "Original seed verification must execute a nonempty seed bank");
  assert.equal(report.tested, report.runs.length, "Every selected original seed must have an executed case");
  assert.equal(report.skipped, 0, "Original seed verification cannot skip cases");
  const values = report.runs.map((run) => run.seedValue);
  assert.ok(values.every((seed) => Number.isInteger(seed) && seed > 0 && seed <= 0xffffffff), "Original seeds must be nonzero u32 values");
  assert.equal(new Set(values).size, values.length, "Original seed coverage must contain independent seeds");
  assert.ok(report.runs.every((run) => typeof run.code === "string" && /^[ABCDEFGHJKLMNPQRSTWXYZ01234V6789]{8}$/.test(compact(run.code))), "Every original case needs its actually entered seed code using the original alphabet");
  return report.runs;
}

export function observedOriginalSeed(log) {
  const seeds = [...log.matchAll(/^RNG Start Seed: ([A-Z0-9]{4} [A-Z0-9]{4}) \((\d+)\)$/gm)];
  const stages = [...log.matchAll(/^Level::Init m_Stage (\d+), m_AltStage (\d+) Seed (\d+)$/gm)];
  const players = [...log.matchAll(/^Initialized player with Variant (\d+) and Subtype (\d+)$/gm)];
  assert.equal(seeds.length, 1, "Fresh original log must contain exactly one observed run seed");
  assert.equal(stages.length, 1, "Fresh original log must contain exactly one observed starting stage");
  assert.equal(players.length, 1, "Fresh original log must contain exactly one initialized player");
  const state = { seed: seeds[0][1], seedValue: Number(seeds[0][2]), stage: Number(stages[0][1]), altStage: Number(stages[0][2]), stageSeed: Number(stages[0][3]), playerVariant: Number(players[0][1]), playerSubtype: Number(players[0][2]) };
  assert.ok(state.seedValue > 0 && state.seedValue <= 0xffffffff);
  assert.ok(Number.isInteger(state.stageSeed) && state.stageSeed >= 0 && state.stageSeed <= 0xffffffff);
  assert.equal(state.stage, 1); assert.equal(state.altStage, 0);
  assert.equal(state.playerVariant, 0); assert.equal(state.playerSubtype, 0);
  return state;
}

export function readOriginalSeedCase(run, reportPath) {
  assert.equal(run.status, "pass", JSON.stringify(run.failures));
  assert.ok(Array.isArray(run.failures) && run.failures.length === 0, "Original case cannot conceal capture failures");
  const manifestPath = resolve(dirname(resolve(reportPath)), run.manifest);
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  assert.equal(manifest.schema, 1); assert.equal(manifest.status, "pass", JSON.stringify(manifest.failures));
  assert.ok(manifest.profile?.fresh && manifest.profile?.isolated, "Original random-seed captures need isolated fresh profiles");
  assert.equal(manifest.seedReadbackMatchesCodec, true, "Original must confirm the requested code was accepted");
  const logPath = join(dirname(manifestPath), "original.log");
  const observed = observedOriginalSeed(readFileSync(logPath, "utf8"));
  assert.equal(compact(observed.seed), compact(run.code), "Actual original log must match the entered seed code");
  assert.equal(observed.seedValue, run.seedValue, "Actual original log must match the selected random seed value");
  for (const [name, value] of Object.entries(observed)) {
    assert.equal(manifest.initialState?.[name], value, `Manifest ${name} must match the real original log`);
    assert.equal(run.initialState?.[name], value, `Report ${name} must match the real original log`);
  }
  const frames = new Map(manifest.frames?.map((frame) => [frame.label, frame]) || []);
  assert.equal(frames.size, manifest.frames?.length, "Native frame labels must be unique");
  for (const label of ["seed-entry", "gameplay-unpaused", "movement", "shooting"]) {
    const frame = frames.get(label);
    assert.ok(frame, `Missing actual original ${label} capture`);
    assert.equal(frame.width, 960); assert.equal(frame.height, 540);
    const image = decodePng(readFileSync(resolve(dirname(manifestPath), frame.path)));
    assert.equal(image.width, 960); assert.equal(image.height, 540);
  }
  return { observed, manifestPath, logPath, manifest };
}

export async function verifyOriginalSeedContract({ nativeReport, output = join(ROOT, ".cache/foundation/original-seed-contract/latest") } = {}) {
  mkdirSync(output, { recursive: true });
  const report = { schema: 1, status: "fail", cases: [], skipped: 0, originalStateParity: false, originalSimulationTickDelivery: false, scope: "Actual original seed-code/value and initial Stage1 RNG readback, plus generic exact native Rust/WASM input replay; room generation, complete hidden state and simulation ticks remain unverified" };
  try {
    assert.ok(nativeReport, "A real original random-seed report is required");
    report.nativeReport = resolve(nativeReport);
    const source = JSON.parse(readFileSync(nativeReport, "utf8"));
    const runs = validateNativeSeedReport(source);
    for (const run of runs) {
      try {
        const original = readOriginalSeedCase(run, nativeReport);
        const commands = invariantTape(original.observed.seed);
        const wasm = await wasmReplay(run.seedValue, commands), native = nativeReplay(run.seedValue, commands);
        assert.equal(native.length, wasm.length);
        for (let index = 0; index < native.length; index++) assert.equal(exactStateDifference(native[index], wasm[index]), null, `Seed ${run.seedValue}: native Rust/WASM diverged after command ${index}`);
        const invariants = assertGameplayInvariants(run.seedValue, commands, wasm);
        const playing = wasm.find((state) => state.screen === "playing");
        assert.equal(compact(playing.runSeed), compact(original.observed.seed));
        assert.equal(playing.runSeedValue, original.observed.seedValue);
        assert.equal(playing.stage, original.observed.stage);
        assert.equal(playing.stageSeed, original.observed.stageSeed, "Generic clone Stage1 RNG must match the actual original random-seed log");
        assert.equal(playing.character, "Isaac"); assert.equal(playing.difficulty, "normal");
        const replay = join(output, `${run.seedValue}.json`);
        writeFileSync(replay, JSON.stringify({ initialSeed: run.seedValue, commands, states: wasm }) + "\n");
        report.cases.push({ initialSeed: run.seedValue, seed: original.observed.seed, status: "pass", manifest: original.manifestPath, originalLog: original.logPath, actualStageSeed: original.observed.stageSeed, ...invariants, replay });
      } catch (error) { report.cases.push({ initialSeed: run.seedValue, seed: run.code, status: "fail", error: error.message }); }
    }
    report.tested = report.cases.length;
    report.status = source.status === "pass" && report.cases.length === runs.length && report.cases.every((item) => item.status === "pass") ? "pass" : "fail";
    if (source.status !== "pass") report.error = "Source original capture report failed";
  } catch (error) { report.error = error.message; }
  writeFileSync(join(output, "report.json"), JSON.stringify(report, null, 2) + "\n");
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const value = (flag) => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
  let nativeReport = value("--native-report"), captureFailed = false;
  if (process.argv.includes("--capture")) {
    const parent = join(ROOT, ".cache/foundation/original-random");
    mkdirSync(parent, { recursive: true });
    const directory = mkdtempSync(join(parent, "capture-"));
    const capture = spawnSync("python3", ["tools/original/foundation-random-seeds.py", "--count", value("--count") || "4", "--jobs", value("--jobs") || "2", "--output", directory], { cwd: ROOT, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
    captureFailed = capture.status !== 0;
    if (captureFailed) console.error(capture.stderr || capture.stdout || capture.error?.message);
    nativeReport = join(directory, "result.json");
  }
  const report = await verifyOriginalSeedContract({ nativeReport: nativeReport ? resolve(nativeReport) : undefined, output: value("--output") ? resolve(value("--output")) : undefined });
  console.log(JSON.stringify(report));
  process.exitCode = !captureFailed && report.status === "pass" ? 0 : 1;
}
