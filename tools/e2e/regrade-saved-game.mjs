// Regrade immutable completed original/browser captures; do not replay or mutate either game.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { compareSavedRoomGrid } from '../foundation/saved-grid-compare.mjs';
import { assertInputProvenance } from '../foundation/verify.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const option = name => { const index = process.argv.indexOf(name); return index < 0 ? undefined : process.argv[index + 1]; };
assert.ok(option('--bank') && option('--output'), 'Use --bank completed-cohort-directory --output report.json');
const bank = resolve(option('--bank')), output = resolve(option('--output'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const load = async path => { const bytes = await readFile(path); return { value: JSON.parse(bytes), sha256: hash(bytes) }; };
const source = await load(join(bank, 'report.json'));
const proof = await load(join(ROOT, 'references/isaac/foundation/saved-game/verification.json'));
assert.equal(source.sha256, proof.value.reportSha256, 'Completed source report must match preserved proof');
assert.equal(source.value.gateSourcesStable, true);
assert.equal(source.value.originalBinarySha256, proof.value.originalBinarySha256);
assert.equal(source.value.production.sha256, proof.value.production.sha256);
const commands = [];
for (let index = 0; index < 4; index++) commands.push({ type: 'key', code: 'Enter', down: true }, { type: 'tick', count: 12 }, { type: 'key', code: 'Enter', down: false }, { type: 'tick', count: index === 3 ? 300 : 72 });
const nativeTape = [{ type: 'step', frames: 180 }, { type: 'capture', label: 'intro' }, { type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Return', down: false }, { type: 'step', frames: 72 }, { type: 'capture', label: 'title' }];
for (const [index, label] of ['profile', 'main', 'character', 'gameplay-start'].entries()) nativeTape.push({ type: 'key', key: 'Return', down: true }, { type: 'step', frames: 12 }, { type: 'key', key: 'Return', down: false }, { type: 'step', frames: index === 3 ? 300 : 72 }, { type: 'capture', label });
const report = { schema: 1, status: 'running',
  mode: 'Offline regrade of unchanged complete original manifests, browser checkpoints and canonical traces; no native recapture or browser execution',
  runtimeSourceHead: proof.value.sourceHead, production: source.value.production,
  originalBinarySha256: source.value.originalBinarySha256,
  preservedSourceReportSha256: source.sha256, preservedProofSha256: proof.sha256,
  sourceGateHashes: source.value.gateSources,
  scope: 'Paired current-row levelSeeds/savedSeeds/occupancy and whole live-grid occupancy are blocking. Within-game copy ownership remains a separate result. Complete postrun shared MT remains a disclosed diagnostic.',
  graderSources: {}, cases: [] };
for (const path of ['tools/e2e/regrade-saved-game.mjs', 'tools/foundation/saved-grid-compare.mjs', 'tools/foundation/verify.mjs']) report.graderSources[path] = hash(await readFile(join(ROOT, path)));
assert.equal(source.value.cases.length, proof.value.cases.length);
for (const selected of source.value.cases) {
  const result = { name: selected.name, status: 'running', bootEpochSeed: selected.bootEpochSeed, initialSeed: selected.initialSeed };
  report.cases.push(result);
  try {
    const retained = proof.value.cases.find(c => c.name === selected.name); assert.ok(retained);
    const directory = join(bank, selected.name);
    const native = await load(join(directory, 'original/manifest.json'));
    const browser = await load(join(directory, 'browser-checkpoints.json'));
    const trace = await load(join(directory, 'trace.json'));
    assert.equal(native.sha256, selected.nativeManifestSha256);
    assert.equal(native.sha256, retained.nativeManifestSha256);
    assert.equal(browser.sha256, retained.browserCheckpointsSha256);
    assert.equal(trace.sha256, retained.canonicalTraceSha256);
    assert.equal(native.value.status, 'pass'); assert.deepEqual(native.value.tape, nativeTape);
    assertInputProvenance(trace.value, commands, selected.initialSeed, selected.bootEpochSeed);
    const observed = native.value.debugObservations.find(c => c.context.label === 'gameplay-start').observation;
    for (const capture of native.value.debugObservations) {
      assert.equal(capture.observation.nativeBinarySha256, report.originalBinarySha256);
      assert.equal(capture.observation.memoryWritten, false);
    }
    const state = browser.value.find(c => c.label === 'gameplay-start').state;
    assert.ok(state.savedRunSummary && observed.savedRun);
    Object.assign(result, compareSavedRoomGrid(state.savedRunSummary, state.room.grid, observed));
    const expectedLegacy = observed.gridSeedSamples.map(c => ({ roomRecord: c.roomRecord, gridIndex: c.gridIndex, seeds: [c.seed10, c.seed18] }));
    assert.deepEqual(state.bootLifecycle.savedGridSeedSamples, expectedLegacy);
    assert.equal(selected.savedGameOwnership.status, 'pass');
    result.savedGameOwnership = { ...selected.savedGameOwnership,
      currentRowCopyStatus: 'pass', currentRowComparedPairs: 9 };
    result.inputProvenance = selected.inputProvenance;
    result.captureHashes = { nativeManifest: native.sha256, browserCheckpoints: browser.sha256, canonicalTrace: trace.sha256 };
    result.runMtDiagnostic = selected.runMtDiagnostic;
    result.savedCounterDiagnostic = selected.savedCounterDiagnostic;
    result.status = result.savedGridRequiredFailures.length ? 'fail' : 'pass';
    if (result.status === 'fail') result.error = `${result.savedGridRequiredFailures.length} required SavedGame current-row/grid comparisons failed`;
  } catch (error) { result.status = 'fail'; result.error = String(error); }
}
report.status = report.cases.every(c => c.status === 'pass') ? 'pass' : 'fail';
report.graderSourcesEnd = {};
for (const path of Object.keys(report.graderSources)) report.graderSourcesEnd[path] = hash(await readFile(join(ROOT, path)));
report.graderSourcesStable = JSON.stringify(report.graderSources) === JSON.stringify(report.graderSourcesEnd);
if (!report.graderSourcesStable) { report.status = 'fail'; report.error = 'Grader sources changed during regrade'; }
report.counts = { selected: report.cases.length, attempted: report.cases.length,
  passed: report.cases.filter(c => c.status === 'pass').length,
  failed: report.cases.filter(c => c.status === 'fail').length,
  ownedCopyPasses: report.cases.filter(c => c.savedGameOwnership?.status === 'pass').length,
  requiredFieldFailures: report.cases.reduce((sum, c) => sum + (c.savedGridRequiredFailures?.length || 0), 0) };
await writeFile(output, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, ...report.counts, report: output }));
if (report.status !== 'pass') process.exitCode = 1;
