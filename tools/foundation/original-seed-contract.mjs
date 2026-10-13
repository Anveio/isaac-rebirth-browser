import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { decodePng } from "./compare.mjs";

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
