// Exact admitted grid fields from a fresh original/public-input production pair.
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
const kinds = option('--kinds').split(',').map(Number);
assert.ok(kinds.length && kinds.every(kind => [2, 8].includes(kind)), 'Only admitted ordinary Rock and Spike contracts are supported');
const load = path => JSON.parse(readFileSync(path, 'utf8'));
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const originalPath = resolve(directory, 'original/manifest.json');
const browserPath = resolve(directory, 'browser/manifest.json');
const native = load(originalPath), browser = load(browserPath);
const gate = load(resolve(dirname(directory), 'report.json'));
const pair = load(resolve(directory, 'report.json'));
const replay = load(resolve(directory, 'case.json'));
assert.equal(pair.original.status, 'pass', 'Fresh original provenance must pass');
assert.equal(pair.browser.status, 'pass', 'Actual production DOM/WASM capture must pass');
assert.equal(browser.status, 'pass');
assert.equal(gate.gateSourcesStable, true, 'Paired gate source identity must remain fixed');
const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
const labels = replay.pairedTape.filter(action => action.type === 'capture').map(action => action.label);
const gameplayStart = labels.indexOf('gameplay-start');
assert.ok(gameplayStart >= 0, 'The tape must declare its gameplay boundary');
assert.equal(observations.size, native.debugObservations.length, 'Native observations must have unique labels');
const checks = [];
let admitted = 0;
const check = (label, field, expected, actual) => checks.push({ label, field, expected, actual,
  status: JSON.stringify(expected) === JSON.stringify(actual) ? 'pass' : 'fail' });
for (const [index, label] of labels.entries()) {
  const observation = observations.get(label), candidate = frames.get(label);
  assert.ok(candidate && observation, `Missing selected original/production checkpoint ${label}`);
  assert.equal(observation.status, 'pass', `Source observation failed at ${label}`);
  assert.equal(observation.memoryWritten, false);
  const gameplay = observation.gameplay;
  if (index < gameplayStart) {
    assert.notEqual(candidate.state.screen, 'playing', 'Only explicit pre-run phases may skip grid comparison');
    assert.equal(gameplay?.status, 'skip');
    assert.equal(observation.menu?.status, 'pass');
    continue;
  }
  assert.equal(candidate.state.screen, 'playing');
  assert.equal(gameplay?.status, 'pass', `Every selected gameplay grid observation must pass at ${label}`);
  assert.equal(gameplay.memoryWritten, false);
  assert.equal(gameplay.executableMappingsRead, false);
  assert.equal(gameplay.nativeBinarySha256, gate.sourceBinarySha256);
  const grid = candidate.state.room.grid;
  assert.ok(grid, 'The live room must expose its admitted grid');
  assert.ok(Number.isSafeInteger(gameplay.nativePhase?.systemIterations));
  assert.ok(Number.isSafeInteger(gameplay.nativePhase?.fullGameUpdates));
  check(label, 'systemClock.phase', gameplay.nativePhase.systemIterations % 2,
    candidate.state.systemClock.systemCounter % 2);
  check(label, 'grid.dimensions', [gameplay.room.gridWidth, gameplay.room.gridHeight], [grid.width, grid.height]);
  const sourceCells = gameplay.grid.filter(cell => kinds.includes(cell.type));
  const actualCells = grid.cells.flatMap((cell, index) => kinds.includes(cell.entity?.kind) ? [{ index, ...cell.entity }] : []);
  check(label, 'grid.admittedIndices', sourceCells.map(cell => cell.index), actualCells.map(cell => cell.index));
  admitted += sourceCells.length;
  for (const source of sourceCells) {
    const actual = grid.cells[source.index]?.entity;
    if (!actual) continue; // The index comparison retains this missing actor failure.
    for (const [field, expected, observed] of [
      ['kind', source.type, actual.kind], ['variant', source.variant, actual.variant],
      ['state', source.state, actual.state], ['collisionClass', source.collisionClass, actual.collisionClass],
      ['spawnSeed', source.spawnSeed, actual.spawnSeed],
      ['position', source.position, [40 + source.index % grid.width * 40, 120 + Math.floor(source.index / grid.width) * 40]],
    ]) {
      assert.ok(expected !== undefined && observed !== undefined, `Missing bounded native grid field ${field}`);
      check(label, `grid${source.index}.${field}`, expected, observed);
    }
  }
}
assert.ok(admitted > 0, 'At least one independently observed supported grid actor must execute');
const report = { schema: 1, status: checks.every(check => check.status === 'pass') ? 'pass' : 'fail', kinds,
  scope: 'Exact actual admitted grid indices, kind, variant, state, collision class and spawn seed; world centers are derived from the source index/room lattice. Fresh original and production DOM/WASM inputs; boundary actor RNG is excluded',
  buildSha256: gate.build.sha256, sourceBinarySha256: gate.sourceBinarySha256,
  originalManifestSha256: hash(originalPath), browserManifestSha256: hash(browserPath),
  caseSha256: hash(resolve(directory, 'case.json')), checks };
writeFileSync(resolve(option('--output')), JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify({ status: report.status, fields: checks.length, failures: checks.filter(check => check.status === 'fail') }));
process.exitCode = report.status === 'pass' ? 0 : 1;
