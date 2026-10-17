// Every declared case runs real original plus immutable production WASM.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream, createWriteStream, mkdirSync, readFileSync, unlinkSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { createGzip, createGunzip } from 'node:zlib';
import { runPairedGate } from '../foundation/paired-game.mjs';
import { bloodStartComparator } from '../foundation/blood-start-state-pair.mjs';
import { flag, selectCases } from './lib/cli.mjs';

const option = (name, fallback) => { const at = process.argv.indexOf(name); if (at < 0) return fallback;
  assert.ok(process.argv[at + 1]); return process.argv[at + 1]; };
const output = resolve(option('--output', '.cache/e2e/blood-start'));
mkdirSync(output, { recursive: true });
const save = (path, document) => writeFileSync(path, JSON.stringify(document, null, 2) + '\n');
const bank = JSON.parse(execFileSync('python3', ['tools/original/blood-start-scenarios.py'], { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }));
assert.equal(bank.cases.length, 4);
assert.deepEqual(bank.cases.map(row => row.bloodStartInputPhaseOffset), [0, 1, 0, 1]);
const cases = bank.cases = selectCases(bank.cases, row => row.scenario.id);
if (flag('--case')) bank.caseFilter = flag('--case');
save(resolve(output, 'scenarios.json'), bank);
if (process.argv.includes('--list')) {
  console.log(JSON.stringify(bank.cases.map(row => ({ id: row.scenario.id, seed: row.seed,
    randomized: row.scenario.randomized, captures: row.pairedTape.filter(action => action.type === 'capture').length }))));
} else {
  const report = { schema: 1, status: 'running', startedAt: new Date().toISOString(), caseFilter: bank.caseFilter, cases: [],
    scope: 'Partial actual Monstro BloodStart17/SFX72/42. Every selected outcome and existing core mismatch remains blocking; no Explosion/gib/final-heart/Grid17 or whole-state claim.' };
  save(resolve(output, 'report.json'), report);
  for (const [index, definition] of cases.entries()) {
    const directory = resolve(output, `${String(index).padStart(2, '0')}-${definition.scenario.id}`);
    mkdirSync(directory, { recursive: true });
    const tape = resolve(directory, 'replay.json'); save(tape, definition);
    const comparator = bloodStartComparator({ scenario: definition.scenario.id, bossIdentity: definition.bloodStartBossIdentity });
    const row = { id: definition.scenario.id, seed: definition.seed, status: 'fail' };
    try {
      const paired = await runPairedGate({ output: resolve(directory, 'paired'), selected: [], randomCount: 0,
        replayCases: [tape], archiveBrowserEvidence: true, originalObserver: 'tools/original/blood-start-observer.py',
        stateComparator: comparator.compare, nativeFrameTimeout: 120,
        productionDirectory: option('--production-directory') });
      const direct = comparator.finish(definition.bloodStartRequired);
      save(resolve(directory, 'body-report.json'), direct);
      row.pairedStatus = paired.status; row.pairedReport = resolve(directory, 'paired', 'report.json');
      row.functionalChecks = paired.cases.reduce((sum, item) => sum + item.checks.length, 0);
      row.functionalFailures = paired.cases.reduce((sum, item) => sum + item.checks.filter(check => check.status === 'fail').length, 0);
      row.execution = direct.execution; row.counts = direct.counts;
      row.requiredFailures = direct.checks.filter(check => check.status === 'fail').length;
      row.status = paired.status === 'pass' && direct.status === 'pass' ? 'pass' : 'fail';
      // Native runner and direct comparator are fully closed. Archive their
      // exact original bytes with independent decompression verification,
      // before the next actual pair starts; no silent evidence deletion.
      const original = resolve(directory, 'paired', `00-${definition.scenario.id}`, 'original');
      row.archives = [];
      for (const name of ['manifest.json', 'trace.jsonl']) {
        const path = resolve(original, name);
        const rawHash = createHash('sha256'); let rawBytes = 0;
        try {
          await pipeline(createReadStream(path), new Transform({transform(chunk, _encoding, done) {
            rawBytes += chunk.length; rawHash.update(chunk); done(); }}));
        } catch (error) { if (error.code === 'ENOENT') continue; throw error; }
        const rawSha256 = rawHash.digest('hex');
        await pipeline(createReadStream(path), createGzip({level: 1}), createWriteStream(path + '.gz', {flags: 'wx'}));
        const hash = createHash('sha256'); let restoredBytes = 0;
        await pipeline(createReadStream(path + '.gz'), createGunzip(), new Transform({transform(chunk, _encoding, done) {
          restoredBytes += chunk.length; hash.update(chunk); done(); }}));
        assert.equal(restoredBytes, rawBytes); assert.equal(hash.digest('hex'), rawSha256);
        row.archives.push({ path: path + '.gz', rawBytes, rawSha256,
          archiveSha256: createHash('sha256').update(readFileSync(path + '.gz')).digest('hex') });
        save(resolve(directory, 'native-archive-journal.json'), row.archives);
        unlinkSync(path);
      }
    } catch (error) { row.error = String(error); }
    report.cases.push(row); save(resolve(output, 'report.json'), report);
  }
  report.status = report.cases.length === cases.length && report.cases.every(row => row.status === 'pass') ? 'pass' : 'fail';
  report.finishedAt = new Date().toISOString(); save(resolve(output, 'report.json'), report);
  console.log(JSON.stringify(report)); process.exitCode = report.status === 'pass' ? 0 : 1;
}
