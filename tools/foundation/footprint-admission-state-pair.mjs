// Admission-only regression composed from existing exact entity contracts.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { entityRows, loadManifest, markers, npcFields } from './entity-compare.mjs';
import { scopedState } from './paired-game.mjs';

function flag(name) {
  const index = process.argv.lastIndexOf(name);
  assert.ok(index >= 0 && process.argv[index + 1], `Missing ${name}`);
  return process.argv[index + 1];
}

const output = resolve(flag('--output'));
const directory = resolve(flag('--case'));
const checks = [], errors = [];
let execution;

// Sibling actors can share a spawn seed; retain their actual list occurrence.
function keyedActors(actors, ported) {
  const seen = new Map();
  return actors.map(actor => {
    const key = JSON.stringify([ported ? actor.kind : actor.type, actor.variant, actor.spawnSeed]);
    const occurrence = seen.get(key) ?? 0;
    seen.set(key, occurrence + 1);
    return [JSON.stringify([key, occurrence]), actor];
  });
}

const spec = {
  name: 'footprint-admission', roomType: 6, matches: () => false,
  roomFields(source, actual, phase) {
    const native = source.entities.filter(actor => actor.type >= 10 && actor.type < 1000);
    const port = actual.room.npcs.filter(actor => !actor.removed);
    const candidates = new Map(keyedActors(port, true));
    return keyedActors(native, false).flatMap(([key, nativeActor]) => {
      const actor = candidates.get(key), name = `admission.npc${key}`;
      return [
        [name + '.managerMember', true, actor ? true : undefined],
        ...(actor ? npcFields(nativeActor, actor, phase).map(([field, expected, actual]) =>
          [name + '.' + field, expected, actual]) : []),
      ];
    });
  },
};

try {
  const c = JSON.parse(readFileSync(resolve(directory, 'case.json'), 'utf8'));
  assert.ok(c.entity?.required === undefined
    || (Array.isArray(c.entity.required) && c.entity.required.length === 0),
  'Admission-only scope does not accept extra lifecycle or draw-order markers');
  const endpoint = c.pairedTape.at(-1);
  assert.ok(endpoint?.type === 'capture' && ['entry-1', 'entry-40'].includes(endpoint.label),
    'Require a declared pre-shoot admission endpoint');
  assert.equal(c.pairedTape.filter(action => action.type === 'capture' && action.label === endpoint.label).length,
    1, 'Require one literal admission endpoint');
  assert.deepEqual(endpoint, { type: 'capture', label: endpoint.label });
  if (c.entity?.stopAfterCapture !== undefined) assert.equal(c.entity.stopAfterCapture, endpoint.label);
  assert.deepEqual(c.tape, [...c.bootstrap.sourceOnlyPrelude, ...c.pairedTape]);

  const native = loadManifest(directory, 'original'), browser = loadManifest(directory, 'browser');
  assert.equal(native.status, 'pass');
  assert.equal(browser.status, 'pass');
  assert.deepEqual(native.tape, c.tape, 'Require the actual Native public prefix');
  assert.deepEqual(browser.frames.map(frame => frame.label),
    c.pairedTape.filter(action => action.type === 'capture').map(action => action.label));
  const observations = new Map(native.debugObservations.map(row => [row.context.label, row.observation]));
  const frames = new Map(browser.frames.map(frame => [frame.label, frame]));
  let started = false, nativeNpcAdmission = false, browserNpcAdmission = false;

  for (const action of c.pairedTape) {
    if (action.type !== 'capture') continue;
    started ||= action.label === 'gameplay-start';
    if (!started) continue;
    const rows = [];
    try {
      const observation = observations.get(action.label);
      const source = observation?.gameplay, frame = frames.get(action.label);
      assert.equal(source?.status, 'pass', `Missing actual Native gameplay owner ${action.label}`);
      assert.equal(source.memoryWritten, false);
      assert.equal(frame?.state?.screen, 'playing', `Missing actual Browser gameplay owner ${action.label}`);
      const actual = frame.state;
      rows.push(...scopedState(observation, actual, true));
      entityRows(rows, source, actual, spec, frame.render);
      nativeNpcAdmission ||= source.room.type === 6 && source.entities.some(actor =>
        actor.type >= 10 && actor.type < 1000 && !actor.removed && actor.active && actor.hp > 0);
      browserNpcAdmission ||= actual.room.descriptor?.template?.type === 6
        && actual.room.npcs.some(actor => !actor.removed && !actor.dead && actor.hp > 0);
    } catch (error) {
      errors.push({ label: action.label, message: error.message, stack: error.stack });
      checks.push({ label: action.label, field: 'admission.actualOwners',
        expected: 'complete actual owners', actual: error.message, status: 'fail' });
    } finally {
      // Keep completed fields even when a later owner in this capture is absent.
      checks.push(...rows.map(row => ({ label: action.label, ...row })));
    }
  }

  assert.ok(checks.length, 'Every case must compare actual gameplay fields');
  execution = markers(directory, spec);
  for (const side of ['original', 'browser']) {
    const admission = side === 'original' ? nativeNpcAdmission : browserNpcAdmission;
    for (const [field, actual] of [['roomType', execution[side].roomType], ['npcAdmission', admission]]) {
      checks.push({ label: 'execution', field: side + '.' + field, expected: true, actual,
        status: actual ? 'pass' : 'fail' });
    }
  }
} catch (error) {
  errors.push({ message: error.message, stack: error.stack });
}

const status = !errors.length && checks.length > 0 && checks.every(row => row.status === 'pass') ? 'pass' : 'fail';
writeFileSync(output, JSON.stringify({ schema: 1, status, checks, errors, execution,
  scope: 'Admission prefix through its declared entry-1 or entry-40 endpoint only. '
    + 'Existing exact generic/player/room/render contracts and every initial NPC field by actual identity/seed/occurrence; '
    + 'both games actually enter miniboss room6 with a living NPC. '
    + 'No shooting, lifecycle, clear or complete shared-MT acceptance.',
}, null, 2) + '\n');
console.log(JSON.stringify({ status, checks: checks.length,
  failures: checks.filter(row => row.status !== 'pass').length, errors: errors.length }));
process.exitCode = status === 'pass' ? 0 : 1;
