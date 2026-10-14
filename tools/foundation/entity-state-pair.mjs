// Original/browser comparison for one authored room entity, graded from a
// paired case's retained native observations and browser states. Used by
// state-pair.mjs for every spec in tools/e2e/entities/. Usage:
//   node entity-state-pair.mjs --entity NAME --case DIR --output FILE
import assert from 'node:assert/strict';
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { loadSpec } from '../e2e/entities.mjs';
import { entityRows, loadManifest, markers } from './entity-compare.mjs';

const option = name => {
  const at = process.argv.indexOf(name);
  assert.ok(at >= 0 && process.argv[at + 1], `Missing ${name}`);
  return process.argv[at + 1];
};
const spec = await loadSpec(option('--entity'));
const directory = resolve(option('--case'));
const native = loadManifest(directory, 'original'), browser = loadManifest(directory, 'browser');
assert.equal(native.status, 'pass', 'Fresh original capture must pass');
assert.equal(browser.status, 'pass', 'Production browser capture must pass');
const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
const states = new Map(browser.frames.map(frame => [frame.label, frame.state]));
const renders = new Map(browser.frames.map(frame => [frame.label, frame.render]));
const checks = [];
let started = false;
for (const [label, observation] of observations) {
  const actual = states.get(label);
  started ||= label === 'gameplay-start';
  if (!started || actual?.screen !== 'playing') continue;
  assert.equal(observation.gameplay?.status, 'pass', `Gameplay observation must pass at ${label}`);
  assert.equal(observation.gameplay.memoryWritten, false);
  const rows = [];
  entityRows(rows, observation.gameplay, actual, spec, renders.get(label));
  for (const row of rows) checks.push({ label, ...row });
}
assert.ok(checks.length, 'A case must compare at least one gameplay capture');
const execution = markers(directory, spec);
for (const [side, reached] of Object.entries(execution))
  for (const name of spec.required) checks.push({ label: 'execution', field: `${side}.${name}`, expected: true, actual: reached[name],
    status: reached[name] ? 'pass' : 'fail' });
const status = checks.every(row => row.status === 'pass') ? 'pass' : 'fail';
writeFileSync(resolve(option('--output')), JSON.stringify({ schema: 1, entity: spec.name, status, required: spec.required, execution,
  scope: `Every authored NPC identity in the room, ${spec.name}'s shared and spec fields, its drawn sprite scale, room/door/manager state and enemy projectile seeds are exact and required at every gameplay capture; both games must reach the required lifecycle markers.`,
  checks }, null, 2) + '\n');
console.log(JSON.stringify({ status, checks: checks.length, failures: checks.filter(row => row.status === 'fail').length, execution }));
process.exitCode = status === 'pass' ? 0 : 1;
