// Independent room-context checks on actual paired capture bytes.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { gunzipSync } from 'node:zlib';
import { resolve, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { manualSeedCommands } from '../foundation/verify.mjs';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const identity = descriptor => descriptor ? {
  index: descriptor.index, generatedIndex: descriptor.generatedIndex, gridIndex: descriptor.gridIndex,
  assetStage: descriptor.assetStage, catalogIndex: descriptor.catalogIndex,
  template: descriptor.template, roomInitSeed: descriptor.roomInitSeed,
  decorationSeed: descriptor.decorationSeed, spawnSeed: descriptor.spawnSeed,
  followingSeed: descriptor.followingSeed,
} : null;

export function compareRoomContext(state, row) {
  const source = row.observation.world.room;
  const publicState = row.context.publicLogState;
  const parsed = /^(\d+)\.(\d+)\((.*)\)$/.exec(publicState.room ?? '');
  const descriptor = state.room.descriptor;
  const template = descriptor?.template;
  const checks = [];
  const check = (field, actual, expected) => checks.push({field, actual, expected});
  check('current descriptor exists', Boolean(descriptor), true);
  check('native direct room type', template?.type, source.type);
  check('native padded grid width', state.room.grid?.width, source.gridWidth);
  check('native padded grid height', state.room.grid?.height, source.gridHeight);
  check('native interior width', template?.width, source.gridWidth - 2);
  check('native interior height', template?.height, source.gridHeight - 2);
  check('native public room identity exists', Boolean(parsed), true);
  check('native direct/public type join', parsed ? Number(parsed[1]) : null, source.type);
  check('native public descriptor type', template?.type, parsed ? Number(parsed[1]) : null);
  check('native public descriptor variant', template?.variant, parsed ? Number(parsed[2]) : null);
  check('native public descriptor name', template?.name, parsed?.[3]);
  check('native current spawn seed', descriptor?.spawnSeed, publicState.spawnSeed);
  check('port current room/descriptor index', state.currentRoom, descriptor?.index);
  check('port current descriptor/floor identity', identity(descriptor), identity(state.floor.descriptors[state.currentRoom]));
  return checks;
}

export async function verifyRoomEvidence(reportPath, outputPath) {
  const reportBytes = await readFile(reportPath);
  const report = JSON.parse(reportBytes);
  assert.equal(report.status, 'pass');
  assert.equal(report.sourceStability.status, 'pass');
  assert.deepEqual(report.sourceStability.before, report.sourceStability.after);
  assert.ok(report.cases.length >= 4, 'Acceptance requires the complete contrived/recorded-random cohort');
  const result = {schema: 1, status: 'running', capturedReportSha256: digest(reportBytes),
    capturedGateSources: report.gateSources,
    verifierSha256: digest(await readFile(fileURLToPath(import.meta.url))),
    scope: 'Every captured original/browser phase requires actual native room type/padded dimensions, public type.variant(name)/spawn-seed joined to the browser current descriptor, and port descriptor/floor consistency. Native appearance origin/backdrop and native descriptor index were not observed and remain unaccepted.',
    checks: 0, comparedPresentations: 0, cases: []};
  for (const item of report.cases) {
    const directory = join(dirname(reportPath), item.name);
    const nativeBytes = await readFile(join(directory, 'original/manifest.json'));
    assert.equal(digest(nativeBytes), item.nativeManifestSha256);
    const native = JSON.parse(nativeBytes);
    assert.equal(native.status, 'pass'); assert.deepEqual(native.failures, []);
    const compressed = await readFile(item.browserEvidence.path);
    const browserBytes = gunzipSync(compressed);
    assert.equal(item.browserEvidence.encoding, 'gzip');
    assert.equal(browserBytes.length, item.browserEvidence.rawBytes);
    assert.equal(digest(browserBytes), item.browserEvidence.rawSha256);
    const browser = JSON.parse(browserBytes);
    assert.equal(browser.trace.inputTraceTruncated, false);
    assert.equal(browser.trace.stateHistoryTruncated, false);
    const baseline = native.debugObservations.find(row => row.context.label === 'gameplay-start');
    const rows = new Map(native.debugObservations.map(row => [row.context.presentedFrame, row]));
    const states = [browser.trace.states[manualSeedCommands(item.code).length], ...browser.states];
    assert.equal(states.length, item.comparedPresentations);
    const caseResult = {name: item.name, status: 'running', comparedPresentations: states.length,
      nativeManifestSha256: item.nativeManifestSha256, browserRawSha256: digest(browserBytes), checks: 0, failures: []};
    for (let presentation = 0; presentation < states.length; presentation++) {
      const row = rows.get(baseline.context.presentedFrame + presentation);
      assert.equal(row?.observation.status, 'pass');
      assert.equal(row.observation.memoryWritten, false);
      for (const check of compareRoomContext(states[presentation], row)) {
        caseResult.checks++;
        try { assert.deepEqual(check.actual, check.expected, `${item.name}:${presentation}:${check.field}`); }
        catch (error) { caseResult.failures.push({...check, presentation, error: String(error.message)}); }
      }
    }
    caseResult.status = caseResult.failures.length ? 'fail' : 'pass';
    result.checks += caseResult.checks;
    result.comparedPresentations += caseResult.comparedPresentations;
    result.cases.push(caseResult);
  }
  result.status = result.cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
  await writeFile(outputPath, JSON.stringify(result, null, 2) + '\n');
  return result;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
  assert.ok(value('--report'), 'Provide the immutable complete live capture report');
  const report = resolve(value('--report'));
  const result = await verifyRoomEvidence(report, resolve(value('--output') || join(dirname(report), 'room-context-verification.json')));
  console.log(JSON.stringify({status: result.status, checks: result.checks,
    comparedPresentations: result.comparedPresentations, verifierSha256: result.verifierSha256}));
  process.exitCode = result.status === 'pass' ? 0 : 1;
}
