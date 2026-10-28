// Paired original/browser runs of a scenario helper's tapes, each graded by a
// state comparator over the retained native observations and browser states.
// Used by npcs, roundworms, fireplaces, monstro and entities (see suites.mjs).
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { basename, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runPairedGate } from '../foundation/paired-game.mjs';
import { flag, outputDirectory, selectCases, writeJson } from './lib/cli.mjs';

const helper = flag('--scenario-helper');
// Repeatable pass-through arguments, e.g. `--helper-arg --type --helper-arg 10`,
// let one registry line drive a generic helper and comparator per entity.
const repeated = name => process.argv.flatMap((arg, index) => arg === name ? [process.argv[index + 1]] : []);
const helperArgs = repeated('--helper-arg'), comparatorArgs = repeated('--comparator-arg');
// Pass-through values can themselves name --production-directory. Resolve
// this outer option before authoring, excluding only the two forwarded lists.
const outerArgv = [];
for (let index = 0; index < process.argv.length; index++) {
  const arg = process.argv[index];
  if (arg === '--helper-arg' || arg === '--comparator-arg') {
    assert.ok(index + 1 < process.argv.length, `Missing value for ${arg}`);
    index++;
  } else outerArgv.push(arg);
}
const productionDirectory = flag('--production-directory', undefined, outerArgv);
const comparator = flag('--comparator');
assert.ok(helper && comparator, 'state-pair requires --scenario-helper and --comparator');
const name = basename(comparator, '.mjs');
const output = outputDirectory(name);

// `--scenarios` replays a recorded run; `--replay-cases a,b` replays single
// cases from their paired `case.json`, as the other paired suites accept.
const recorded = flag('--scenarios'), replayCases = flag('--replay-cases');
const readJson = path => JSON.parse(readFileSync(resolve(path), 'utf8'));
// --browser-recorder reaches the helpers' route recorder (boss-route-record.mjs)
// through the environment they pass on.
const helperEnv = { ...process.env, ...(outerArgv.includes('--browser-recorder') && { E2E_BROWSER_RECORDER: '1' }) };
const scenarios = replayCases ? { schema: 1, cases: replayCases.split(',').map(readJson) }
  : recorded ? readJson(recorded)
  : JSON.parse(execFileSync('python3', [helper, ...helperArgs], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, env: helperEnv }));
// Continue's route owner validates every declaration before selection can
// hide a malformed case. Retain the whole input bank and its selected scope.
const continueRun = resolve(comparator) === fileURLToPath(new URL('../foundation/continue-run-state-pair.mjs', import.meta.url));
const bankValidation = continueRun
  ? (await import('../foundation/continue-run-state-pair.mjs')).validateContinueRunScenarios(scenarios, { allowSubset: Boolean(replayCases) })
  : undefined;
if (continueRun) writeJson(resolve(output, 'generated-scenarios.json'), scenarios);
scenarios.cases = selectCases(scenarios.cases, item => item.scenario.id);
if (flag('--case')) scenarios.caseFilter = flag('--case');
writeJson(resolve(output, 'scenarios.json'), scenarios);
if (continueRun) writeJson(resolve(output, 'selection.json'), { schema: 1, caseFilter: scenarios.caseFilter,
  bankValidation, cases: scenarios.cases.map(item => item.scenario.id) });
const tapes = scenarios.cases.map((item, index) => writeJson(resolve(output, `replay-${index}.json`), item));

// Browser manifests and traces are gzipped losslessly as they are written
// (~250 MB raw per case); the comparators read either form.
const paired = tapes.length ? await runPairedGate({ output: resolve(output, 'paired'), selected: [], randomCount: 0, replayCases: tapes,
  productionDirectory,
  publicClockSchedule: outerArgv.includes('--public-clock-schedule'),
  originalObserver: flag('--state-observer') || undefined,
  reuseOriginalDirectory: flag('--reuse-original'),
  nativeFrameTimeout: Number(flag('--native-frame-timeout', '60')),
  archiveNativeEvidence: outerArgv.includes('--archive-native-evidence'),
  archiveBrowserEvidence: true,
  concurrentCaptures: outerArgv.includes('--concurrent-captures') })
  : { status: 'fail', cases: [] };

// Cases whose port keys were recorded closed-loop (stage-exit fights) only
// replay the recorded game on the build that recorded them.
const build = paired.build?.sha256;
const stale = scenario => scenario.portRecordedBuild && build && scenario.portRecordedBuild !== build
  ? { recordedBuild: scenario.portRecordedBuild, build } : undefined;

const requireKey = flag('--require-from-case');
const cases = scenarios.cases.map((scenario, index) => {
  const directory = resolve(output, 'paired', `${String(index).padStart(2, '0')}-${scenario.scenario.id}`);
  const target = resolve(directory, `${name}.json`);
  rmSync(target, { force: true });
  const args = [comparator, ...comparatorArgs, '--case', directory, '--output', target];
  if (requireKey) args.push('--require', scenario[requireKey].join(','));
  const child = spawnSync('node', args, { encoding: 'utf8' });
  // A case that aborted before capture (e.g. on a full disk) may have no
  // directory; it is graded as failed without losing the other cases' reports.
  try {
    mkdirSync(directory, { recursive: true });
    writeFileSync(resolve(directory, `${name}-output.txt`), child.stdout + child.stderr);
  } catch {}
  let report;
  try { report = JSON.parse(readFileSync(target, 'utf8')); }
  catch { report = { status: 'fail', error: child.stderr || 'Comparator produced no report' }; }
  const failed = list => list?.filter(check => check.status === 'fail').length || 0;
  // A browser replay that left its recording's path (paired-game.mjs
  // recorderDivergence) fails the case whatever the comparator found.
  const divergence = paired.cases?.find(item => item.directory === directory)?.recorderReplay;
  return {
    id: scenario.scenario.id,
    status: child.status === 0 && !child.signal && report.status === 'pass' && divergence?.status !== 'fail' ? 'pass' : 'fail',
    checks: report.checks?.length || 0, failures: failed(report.checks),
    diagnostics: report.diagnostics?.length || 0, diagnosticFailures: failed(report.diagnostics),
    ...(continueRun && { scope: report.scope }),
    error: (divergence?.status === 'fail' ? divergence.error : undefined) || report.error || child.error?.message || (child.status ? child.stderr.trim().split('\n').at(-1) : undefined),
    report: target, staleRecording: stale(scenario),
  };
});

// A helper may report items it could not build (e.g. one entity spec's room
// discovery); they fail the suite without hiding the cases that did build.
const helperFailures = scenarios.failures ?? [];
const status = paired.status === 'pass' && !helperFailures.length && cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
writeJson(resolve(output, 'report.json'), { schema: 1, status, caseFilter: scenarios.caseFilter, pairedStatus: paired.status, helperFailures,
  ...(continueRun && { bankValidation, selection: resolve(output, 'selection.json') }), cases });
for (const item of cases) console.log(`${item.status}: ${item.id} (${item.failures}/${item.checks} checks failed)${item.error ? ` ${item.error}` : ''}`);
for (const failure of helperFailures) console.log(`fail: ${JSON.stringify(failure)}`);
for (const item of cases.filter(item => item.staleRecording)) console.log(`warning: ${item.id} replays port keys recorded on build ${item.staleRecording.recordedBuild.slice(0, 12)} against build ${build.slice(0, 12)}; its result does not compare the two builds — rerun without --scenarios`);
console.log(JSON.stringify({ status, report: resolve(output, 'report.json') }));
process.exitCode = status === 'pass' ? 0 : 1;
