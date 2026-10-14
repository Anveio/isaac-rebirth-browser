// Sequential fresh original/production DOM pairs; every declared case remains.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, statSync, unlinkSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { gzipSync, gunzipSync } from 'node:zlib';
import { runPairedGate } from '../foundation/paired-game.mjs';

const option = (name, fallback) => { const at = process.argv.indexOf(name); if (at < 0) return fallback; assert.ok(process.argv[at + 1]); return process.argv[at + 1]; };
const output = resolve(option('--output', `.cache/e2e/blood-particles/${Date.now()}-${process.pid}`));
mkdirSync(output, {recursive: true});
const reportPath = resolve(output, 'report.json');
assert.equal(existsSync(reportPath), false, 'A new execution must preserve every prior report');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n');
const sources = ['tools/e2e/blood-particles.mjs', 'tools/original/blood-particle-scenarios.py',
  'tools/original/roundworm-scenarios.py', 'tools/original/foundation-combat-observe.py',
  'tools/foundation/blood-particle-state-pair.mjs', 'tools/foundation/paired-game.mjs',
  'tools/foundation/verify.mjs', 'tools/foundation/production-server.mjs',
  'references/isaac/foundation/combat/roundworms/random-selection.json',
  'references/isaac/foundation/combat/roundworms/hostile-return-tail.json',
  'engine/build.rs', 'engine/src/gib_effects.rs', 'engine/src/familiars.rs', 'engine/src/roundworm.rs',
  'engine/src/lib.rs', 'engine/src/actors.rs', 'engine/src/items.rs', 'engine/src/mt.rs', 'engine/src/grid.rs',
  'web/main.ts', 'web/engine.ts', 'web/renderer.ts', 'web/animation.ts', 'web/familiar-data.json'];
const pins = () => Object.fromEntries(sources.map(path => [path, sha(readFileSync(path))]));
const before = pins();
const bank = JSON.parse(execFileSync('python3', ['tools/original/blood-particle-scenarios.py'], {encoding: 'utf8', maxBuffer: 4 * 1024 * 1024}));
assert.equal(bank.schema, 1); assert.equal(bank.cases.length, 4);
assert.equal(new Set(bank.cases.map(item => item.scenario.id)).size, 4);
assert.equal(bank.cases.filter(item => item.scenario.randomized).length, 2);
save(resolve(output, 'scenarios.json'), bank);
if (process.argv.includes('--prepare-only')) {
  console.log(JSON.stringify({status: 'prepared', cases: bank.cases.map(item => item.scenario.id), scope: bank.scope, gateSources: before}));
  process.exit(0);
}
const report = {schema: 1, status: 'running', startedAt: new Date().toISOString(), gateSources: before, cases: [],
  scope: 'Fresh original+immutable production DOM/WASM, all four declared public cases. Actual244 lethal5 producer and every matched real5 field required; core paired failures retained. No source state/time/RNG/aim enters production.'};
save(reportPath, report);
const archive = path => {
  if (!existsSync(path)) return null;
  const raw = readFileSync(path); JSON.parse(raw);
  const info = statSync(path), compressed = path + '.gz'; assert.equal(existsSync(compressed), false);
  const bytes = gzipSync(raw); writeFileSync(compressed, bytes, {flag: 'wx'});
  const restored = gunzipSync(readFileSync(compressed));
  assert.equal(restored.length, raw.length); assert.equal(sha(restored), sha(raw));
  const metadata = {originalPath: path, path: compressed, encoding: 'gzip', rawBytes: raw.length,
    rawSha256: sha(raw), archiveBytes: bytes.length, archiveSha256: sha(bytes), mode: info.mode & 0o777, mtimeMs: info.mtimeMs,
    scope: 'Lossless closed-case original/browser bytes; all selected snapshots/input/backend states and failures preserved. Restore by gunzip to originalPath.'};
  save(path + '.archive.json', metadata); unlinkSync(path); return metadata;
};
for (const item of bank.cases) {
  const id = item.scenario.id, directory = resolve(output, id), tape = resolve(directory, 'replay.json');
  mkdirSync(directory, {recursive: true}); save(tape, item);
  const row = {id, status: 'running', randomized: item.scenario.randomized, failures: null};
  report.cases.push(row); save(reportPath, report);
  const pairDirectory = resolve(directory, 'paired'), caseDirectory = resolve(pairDirectory, '00-' + id);
  try {
    const paired = await runPairedGate({output: pairDirectory, selected: [], randomCount: 0, replayCases: [tape],
      productionDirectory: option('--production-directory'), nativeFrameTimeout: 120});
    row.production = paired.build; row.pairedStatus = paired.status; row.pairedSourcesStable = paired.gateSourcesStable;
    const target = resolve(directory, 'blood-particle-fields.json');
    const result = spawnSync('node', ['tools/foundation/blood-particle-state-pair.mjs', '--case', caseDirectory, '--output', target], {encoding: 'utf8'});
    writeFileSync(resolve(directory, 'comparator.log'), result.stdout + result.stderr);
    row.comparatorExitCode = result.status; row.comparatorSignal = result.signal;
    if (existsSync(target)) {
      const fields = JSON.parse(readFileSync(target, 'utf8'));
      Object.assign(row, {presentations: fields.presentations, checks: fields.checks, failures: fields.failures.length,
        execution: fields.execution, fieldReport: target, fieldReportSha256: sha(readFileSync(target))});
      row.status = paired.status === 'pass' && fields.status === 'pass' && result.status === 0 && !result.signal ? 'pass' : 'fail';
    } else { row.status = 'fail'; row.error = result.stderr || 'No actual field comparison report'; }
  } catch (error) { row.status = 'fail'; row.error = String(error); }
  finally {
    // Both actual games and the verifier have closed before any raw path is
    // released. Partial failed captures are preserved by the same exact bytes.
    try { row.archives = ['original/manifest.json', 'browser/manifest.json'].map(path => archive(resolve(caseDirectory, path))).filter(Boolean); }
    catch (error) { row.status = 'fail'; row.archiveError = String(error); }
    save(reportPath, report);
  }
  console.log(JSON.stringify(row));
}
const after = pins();
report.sourceStability = {status: JSON.stringify(before) === JSON.stringify(after) ? 'pass' : 'fail', before, after};
report.status = report.cases.every(row => row.status === 'pass') && report.sourceStability.status === 'pass' ? 'pass' : 'fail';
report.finishedAt = new Date().toISOString(); save(reportPath, report);
console.log(JSON.stringify({status: report.status, report: reportPath, cases: report.cases.length}));
process.exitCode = report.status === 'pass' ? 0 : 1;
