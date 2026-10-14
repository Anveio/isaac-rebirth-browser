// Paired original/browser runs of a scenario helper's tapes, each graded by a
// state comparator over the retained native observations and browser states.
// Used by npcs, roundworms, fireplaces and monstro (see suites.mjs).
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, rmSync, writeFileSync } from 'node:fs';
import { basename, resolve } from 'node:path';
import { runPairedGate } from '../foundation/paired-game.mjs';
import { flag, outputDirectory, writeJson } from './lib/cli.mjs';

const helper = flag('--scenario-helper');
// Repeatable pass-through arguments, e.g. `--helper-arg --type --helper-arg 10`,
// let one registry line drive a generic helper and comparator per entity.
const repeated = name => process.argv.flatMap((arg, index) => arg === name ? [process.argv[index + 1]] : []);
const helperArgs = repeated('--helper-arg'), comparatorArgs = repeated('--comparator-arg');
const comparator = flag('--comparator');
assert.ok(helper && comparator, 'state-pair requires --scenario-helper and --comparator');
const name = basename(comparator, '.mjs');
const output = outputDirectory(name);

const recorded = flag('--scenarios');
const scenarios = recorded ? JSON.parse(readFileSync(resolve(recorded), 'utf8'))
  : JSON.parse(execFileSync('python3', [helper, ...helperArgs], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }));
const selected = flag('--case');
if (selected) {
  scenarios.cases = scenarios.cases.filter(item => item.scenario.id === selected);
  assert.equal(scenarios.cases.length, 1, `Unknown case ${selected}`);
}
writeJson(resolve(output, 'scenarios.json'), scenarios);
const tapes = scenarios.cases.map((item, index) => writeJson(resolve(output, `replay-${index}.json`), item));

const paired = await runPairedGate({ output: resolve(output, 'paired'), selected: [], randomCount: 0, replayCases: tapes,
  productionDirectory: flag('--production-directory'), nativeFrameTimeout: Number(flag('--native-frame-timeout', '60')) });

const requireKey = flag('--require-from-case');
const cases = scenarios.cases.map((scenario, index) => {
  const directory = resolve(output, 'paired', `${String(index).padStart(2, '0')}-${scenario.scenario.id}`);
  const target = resolve(directory, `${name}.json`);
  rmSync(target, { force: true });
  const args = [comparator, ...comparatorArgs, '--case', directory, '--output', target];
  if (requireKey) args.push('--require', scenario[requireKey].join(','));
  const child = spawnSync('node', args, { encoding: 'utf8' });
  writeFileSync(resolve(directory, `${name}-output.txt`), child.stdout + child.stderr);
  let report;
  try { report = JSON.parse(readFileSync(target, 'utf8')); }
  catch { report = { status: 'fail', error: child.stderr || 'Comparator produced no report' }; }
  const failed = list => list?.filter(check => check.status === 'fail').length || 0;
  return {
    id: scenario.scenario.id,
    status: child.status === 0 && !child.signal && report.status === 'pass' ? 'pass' : 'fail',
    checks: report.checks?.length || 0, failures: failed(report.checks),
    diagnostics: report.diagnostics?.length || 0, diagnosticFailures: failed(report.diagnostics),
    error: report.error || child.error?.message || (child.status ? child.stderr.trim().split('\n').at(-1) : undefined),
    report: target,
  };
});

const status = paired.status === 'pass' && cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
writeJson(resolve(output, 'report.json'), { schema: 1, status, pairedStatus: paired.status, cases });
for (const item of cases) console.log(`${item.status}: ${item.id} (${item.failures}/${item.checks} checks failed)${item.error ? ` ${item.error}` : ''}`);
console.log(JSON.stringify({ status, report: resolve(output, 'report.json') }));
process.exitCode = status === 'pass' ? 0 : 1;
