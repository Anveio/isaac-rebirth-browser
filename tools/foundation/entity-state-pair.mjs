// Original/browser comparison for the authored room entities of one paired
// case, graded from its retained native observations and browser states. Used
// by state-pair.mjs for every spec in tools/e2e/entities/. Usage:
//   node entity-state-pair.mjs [--entity NAME] --case DIR --output FILE
// Each spec the case names in entity.specs (or --entity, for recordings made
// before shared cases) is graded on its own rows and required markers.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { loadSpec } from '../e2e/entities.mjs';
import { entityRows, loadManifest, markers } from './entity-compare.mjs';

const option = (name, required = true) => {
  const at = process.argv.indexOf(name);
  assert.ok(!required || (at >= 0 && process.argv[at + 1]), `Missing ${name}`);
  return at >= 0 ? process.argv[at + 1] : undefined;
};
const directory = resolve(option('--case'));
const caseEntity = JSON.parse(readFileSync(resolve(directory, 'case.json'), 'utf8')).entity ?? {};
const named = caseEntity.specs ?? [option('--entity')];
const specs = await Promise.all(named.map(loadSpec));
const native = loadManifest(directory, 'original'), browser = loadManifest(directory, 'browser');
assert.equal(native.status, 'pass', 'Fresh original capture must pass');
assert.equal(browser.status, 'pass', 'Production browser capture must pass');
const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
const states = new Map(browser.frames.map(frame => [frame.label, frame.state]));
const renders = new Map(browser.frames.map(frame => [frame.label, frame.render]));
const checks = [], graded = {};
for (const spec of specs) {
  const rows = [];
  let started = false;
  for (const [label, observation] of observations) {
    const actual = states.get(label);
    started ||= label === 'gameplay-start';
    if (!started || actual?.screen !== 'playing') continue;
    assert.equal(observation.gameplay?.status, 'pass', `Gameplay observation must pass at ${label}`);
    assert.equal(observation.gameplay.memoryWritten, false);
    const captured = [];
    entityRows(captured, observation.gameplay, actual, spec, renders.get(label));
    for (const row of captured) rows.push({ label, ...row });
  }
  assert.ok(rows.length, `${spec.name}: a case must compare at least one gameplay capture`);
  const execution = markers(directory, spec);
  // A case may require more markers (boss-scenarios.py crossing: both
  // overlapping draw orders).
  const required = [...new Set([...spec.required, ...(caseEntity.required ?? [])])];
  for (const [side, reached] of Object.entries(execution))
    for (const name of required) rows.push({ label: 'execution', field: `${side}.${name}`, expected: true, actual: reached[name],
      status: reached[name] ? 'pass' : 'fail' });
  for (const row of rows) checks.push({ entity: spec.name, ...row });
  graded[spec.name] = { status: rows.every(row => row.status === 'pass') ? 'pass' : 'fail', checks: rows.length,
    failures: rows.filter(row => row.status === 'fail').length, required, execution };
}
const status = checks.every(row => row.status === 'pass') ? 'pass' : 'fail';
writeFileSync(resolve(option('--output')), JSON.stringify({ schema: 2, entities: named, status, specs: graded,
  scope: `Every authored NPC identity in the room, each graded spec's shared and spec fields, its drawn sprite scale and draw order against the player, room/door/manager state and enemy projectile seeds are exact and required at every gameplay capture; both games must reach each spec's required lifecycle markers.`,
  checks }, null, 2) + '\n');
console.log(JSON.stringify({ status, checks: checks.length, failures: checks.filter(row => row.status === 'fail').length, specs: graded }));
process.exitCode = status === 'pass' ? 0 : 1;
