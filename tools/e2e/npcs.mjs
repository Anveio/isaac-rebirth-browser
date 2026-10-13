// Fresh original + production browser/WASM gate; no entity/state injection.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { runPairedGate } from '../foundation/paired-game.mjs';

const option = (name, fallback) => {
  const at = process.argv.indexOf(name);
  if (at < 0) return fallback;
  assert.ok(process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
const output = resolve(option('--output', '.cache/e2e/npcs'));
mkdirSync(output, { recursive: true });
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n');
const sources = ['tools/e2e/npcs.mjs', 'tools/original/npc-scenarios.py', 'tools/foundation/npc-state-pair.mjs',
  'references/isaac/foundation/combat/npcs/random-selection.json'];
const hashes = () => Object.fromEntries(sources.map(path => [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const before = hashes();
const scenarios = JSON.parse(execFileSync('python3', ['tools/original/npc-scenarios.py'], { encoding: 'utf8' }));
save(resolve(output, 'scenarios.json'), scenarios);
const tapes = scenarios.cases.map((scenario, index) => {
  const path = resolve(output, `replay-${index}.json`);
  save(path, scenario);
  return path;
});
const paired = await runPairedGate({ output: resolve(output, 'paired'), selected: [], randomCount: 0, replayCases: tapes,
  productionDirectory: option('--production-directory', undefined) });
const cases = scenarios.cases.map((scenario, index) => {
  const directory = resolve(output, 'paired', `${String(index).padStart(2, '0')}-${scenario.scenario.id}`);
  const target = resolve(directory, 'npc-state.json');
  const process = spawnSync('node', ['tools/foundation/npc-state-pair.mjs', '--case', directory,
    '--require', scenario.npcRequired.join(','), '--output', target], { encoding: 'utf8' });
  writeFileSync(resolve(directory, 'npc-state-output.txt'), process.stdout + process.stderr);
  let report;
  try { report = JSON.parse(readFileSync(target, 'utf8')); }
  catch { report = { status: 'fail', error: process.stderr || 'NPC comparator did not produce its required report' }; }
  return { id: scenario.scenario.id, required: scenario.npcRequired, status: report.status,
    fields: report.checks?.length || 0, failures: report.checks?.filter(check => check.status === 'fail').length || 0,
    diagnosticFields: report.diagnostics?.length || 0,
    diagnosticFailures: report.diagnostics?.filter(check => check.status === 'fail').length || 0,
    execution: report.execution, error: report.error, report: target };
});
assert.deepEqual(hashes(), before, 'NPC cohort and comparison sources must remain stable');
const report = { schema: 1, status: paired.status === 'pass' && cases.every(row => row.status === 'pass') ? 'pass' : 'fail',
  pairedStatus: paired.status, sourcesStable: true, sources: before, cases,
  scope: 'Every selected public original/browser checkpoint is retained. Exact player/active-actor fields and supported NPC fields remain required. AI/counter/door diagnostics are exact and retained separately; known center-grid and random trajectory failures remain mandatory.' };
save(resolve(output, 'report.json'), report);
console.log(JSON.stringify(report));
process.exitCode = report.status === 'pass' ? 0 : 1;
