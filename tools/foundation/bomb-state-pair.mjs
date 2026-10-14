import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const option = name => {
  const at = process.argv.indexOf(name);
  assert.ok(at >= 0 && process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
const directory = resolve(option('--case'));
const originalPath = resolve(directory, 'original/manifest.json');
const browserPath = resolve(directory, 'browser/manifest.json');
const load = path => JSON.parse(readFileSync(path, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const native = load(originalPath), browser = load(browserPath);
const gate = load(resolve(dirname(directory), 'report.json'));
const pair = load(resolve(directory, 'report.json'));
assert.equal(pair.original.status, 'pass', 'Fresh original provenance must pass');
assert.equal(pair.browser.status, 'pass', 'Actual production DOM/WASM capture must pass');
assert.equal(browser.status, 'pass');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const checks = [];
for (const row of native.debugObservations) {
  const gameplay = row.observation.gameplay;
  if (gameplay?.status !== 'pass') continue;
  assert.equal(gameplay.memoryWritten, false);
  assert.equal(gameplay.executableMappingsRead, false);
  assert.equal(gameplay.nativeBinarySha256, gate.sourceBinarySha256);
  const sourceBombs = gameplay.entities.filter(actor => actor.type === 4);
  if (!sourceBombs.length) continue;
  const candidate = frames.get(row.context.label);
  assert.ok(candidate, `Missing production checkpoint ${row.context.label}`);
  // This fresh-Isaac admission tape owns one inventory bomb. A future
  // multi-bomb gate needs the shared actor-ID mapping before pairing actors.
  assert.equal(sourceBombs.length, 1, 'Single-bomb source contract');
  assert.equal(candidate.state.bombs.length, 1, 'Single-bomb production contract');
  const source = sourceBombs[0], motion = candidate.state.bombs[0].motion;
  const fields = [
    ['presentedPosition.x', source.position[0], Math.fround(motion.presentedPosition.x)],
    ['presentedPosition.y', source.position[1], Math.fround(motion.presentedPosition.y)],
    ['velocity.x', source.velocity[0], Math.fround(motion.velocity.x)],
    ['velocity.y', source.velocity[1], Math.fround(motion.velocity.y)],
    ['fuseUpdatesRemaining', source.bombFuseUpdatesRemaining, motion.fuseUpdatesRemaining],
  ];
  for (const [field, expected, actual] of fields) {
    assert.ok(Number.isFinite(expected) && Number.isFinite(actual), `Missing finite ${field}`);
    checks.push({ label: row.context.label, field, expected, actual,
      status: expected === actual ? 'pass' : 'fail' });
  }
}
assert.ok(checks.length > 0, 'At least one independently observed bomb must execute');
const report = {
  schema: 1, status: checks.every(check => check.status === 'pass') ? 'pass' : 'fail',
  scope: 'Exact native f32 presented position, stored velocity and full-phase fuse from fresh original/public DOM production inputs; no state injection',
  buildSha256: gate.build.sha256, sourceBinarySha256: gate.sourceBinarySha256,
  originalManifestSha256: hash(originalPath), browserManifestSha256: hash(browserPath),
  caseSha256: hash(resolve(directory, 'case.json')), checks,
};
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, fields: checks.length,
  failures: checks.filter(check => check.status === 'fail') }));
process.exitCode = report.status === 'pass' ? 0 : 1;
