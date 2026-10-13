// Live original and production-browser ordinary doorway/return contracts.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, assertSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';
const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const output = resolve(value('--output') || join(ROOT, '.cache/e2e/traversal', String(Date.now())));
const helper = value('--scenario-helper') || join(ROOT, 'tools/original/foundation-traversal-scenarios.py');
const python = async args => JSON.parse((await run('python3', [helper, ...args], { cwd: ROOT, maxBuffer: 8 * 1024 * 1024 })).stdout);
await mkdir(output, { recursive: true });
const bank = value('--scenarios') ? JSON.parse(await readFile(resolve(value('--scenarios')), 'utf8')) : await python(['--count', value('--count') || '1']);
assert.equal(bank.schema, 1);
assert.ok(bank.cases.length >= 2 && bank.cases.length <= 6, 'Require contrived and independently recorded source routes');
assert.equal(new Set(bank.cases.map(entry => entry.runSeed)).size, bank.cases.length);
await writeFile(join(output, 'scenarios.json'), JSON.stringify(bank, null, 2) + '\n');
const production = await startProductionServer();
const browser = await chromium.launch({ headless: true, ...browserLaunchOptions() });
const report = { schema: 1, status: 'running', skipped: 0, selected: bank.cases.length,
  scope: bank.contract === 'curse'
    ? 'Fresh ordinary Isaac, authored 13x7 no-hostile curse-room entry and exit; exact native half-heart charges, immunity/door collision/admission phases, descriptor/spawn identity and returned room persistence. Flight/items, other door styles, hostile callbacks and complete pixels remain separate.'
    : bank.contract === 'authored-doors'
      ? 'Fresh ordinary Isaac, authored13x7 curse and first-floor free treasure doors; recorded W0L0LYYM bomb-driven admission and raw1400 fire admission remain mandatory. Exact room/player/HP/immunity/visible door phases, complete authored groups and return metadata; actor/item callbacks and pixels remain separate.'
      : 'Ordinary style0 13x7 doorway and no-hostile return; mandatory hostile-room lock/return regression; exact f32 player/descriptor phases, complete authored group accounting, supported actor admission and ordinary Rock metadata, visited-room restoration. NPC behavior, other grids/doors, large-room camera and complete pixels remain separate.',
  exclusions: ['curse', 'authored-doors'].includes(bank.contract) ? ['Undiscovered native door variant6', 'Actor behavior and item callbacks; NPC244 metadata only, raw1400 fire admission remains a required regression', 'Flight/items and other door policies', 'Complete rendered pixels'] : [],
  production: production.build, graphics: await inspectBrowserGraphics(browser), cases: [] };
try {
  for (const scenario of bank.cases) {
    const row = { seed: scenario.code, status: 'fail', comparedPhases: 0 };
    let page;
    try {
      const directory = join(output, scenario.code); await mkdir(directory, { recursive: true });
      let sourceManifest = scenario.sourceManifest && resolve(scenario.sourceManifest);
      if (!sourceManifest) {
        const tape = join(directory, 'tape.json');
        await writeFile(tape, JSON.stringify(scenario.tape, null, 2) + '\n');
        await run('bash', [join(ROOT, 'tools/original/with-display.sh'), 'python3',
          join(ROOT, 'tools/original/foundation-clock-run.py'), '--seed', scenario.code, '--tape', tape,
          '--state-observer', join(ROOT, 'tools/original/foundation-traversal-observe.py'),
          '--output', join(directory, 'original')], { cwd: ROOT, timeout: 300_000, maxBuffer: 8 * 1024 * 1024 });
        sourceManifest = join(directory, 'original/manifest.json');
      }
      const provenance = await python(['--validate-original', sourceManifest]);
      assert.equal(provenance.status, 'pass');
      const native = JSON.parse(await readFile(sourceManifest, 'utf8'));
      assert.equal(native.seedValue, scenario.runSeed);
      const originals = new Map(native.debugObservations.map(entry => [entry.context.label, entry]));
      const setup = scenario.setup || [{ type: 'step', frames: 60 }, ...(scenario.slot % 2 === 0
        ? [{ type: 'key', key: 'w', down: true }, { type: 'step', frames: 27 }, { type: 'key', key: 'w', down: false }, { type: 'step', frames: 30 }] : [])];
      const keyCodes = { a: 'KeyA', w: 'KeyW', d: 'KeyD', s: 'KeyS', e: 'KeyE', Up: 'ArrowUp',
        Down: 'ArrowDown', Left: 'ArrowLeft', Right: 'ArrowRight', Escape: 'Escape' };
      const expectedCommands = [...manualSeedCommands(scenario.code), { type: 'tick', count: 140 },
        ...[...setup, ...scenario.actions].flatMap(action => {
          if (action.type === 'step') return [{ type: 'tick', count: action.frames }];
          if (action.type === 'capture') return [];
          assert.ok(keyCodes[action.key], 'Every declared route key must have a DOM mapping');
          return [{ type: 'key', code: keyCodes[action.key], down: action.down }];
        })];
      const nativeStart = native.tape.findIndex(action => action.type === 'capture' && action.label === 'gameplay-start');
      assert.deepEqual(native.tape.slice(nativeStart + 1), [...setup, ...scenario.actions],
        'The declared browser route must match the actual recorded public X11 route');
      const labels = ['gameplay-start', ...scenario.actions.filter(action => action.type === 'capture').map(action => action.label)];
      assert.deepEqual([...originals.keys()], labels, 'Every selected source phase must execute exactly once');
      page = await browser.newPage({ viewport: { width: 960, height: 540 } });
      const errors = []; page.on('pageerror', error => errors.push(String(error)));
      await page.goto(production.url + '/?manual=1'); await waitForReady(page);
      await page.evaluate(seed => { window.game.pause(); window.game.reset(seed, { trace: true }); }, scenario.runSeed);
      for (let index = 0; index < 3; index++) await page.keyboard.press('Enter');
      await page.keyboard.press('Tab'); await page.keyboard.type(scenario.code);
      await page.keyboard.press('Enter'); await page.keyboard.press('Enter');
      await page.evaluate(() => window.game.step(140));
      const seeded = await page.evaluate(() => window.game.snapshot());
      await writeFile(join(directory, 'browser-input.json'), JSON.stringify({ schema: 1, seeded,
        trace: await page.evaluate(() => window.game.trace()), expectedCommands }, null, 2) + '\n');
      assertSeededRun(seeded, scenario.code, scenario.runSeed);
      const observedRooms = new Set(), admissions = new Map(), states = [], comparisonFailures = [];
      const compare = async label => {
        const phaseAssert = new Proxy(assert, { get(target, name) {
          if (typeof target[name] !== 'function') return target[name];
          return (...args) => { try { target[name](...args); }
            catch (error) { comparisonFailures.push({label, error: String(error)}); } };
        }});
        const state = await page.evaluate(() => window.game.snapshot());
        const original = originals.get(label), observed = original.observation;
        const source = observed.floor, actor = observed.contents.players[0];
        phaseAssert.equal(state.screen, 'playing', `${label}:playable browser state`);
        phaseAssert.equal(state.runSeedValue, scenario.runSeed);
        phaseAssert.equal(state.systemClock.systemCounter % 2, observed.contents.nativePhase.systemIterations % 2, `${label}:full/intermediate phase`);
        phaseAssert.equal(state.currentRoom, source.currentDescriptorIndex, `${label}:current source descriptor`);
        const descriptor = source.descriptors[source.currentDescriptorIndex];
        for (const field of ['gridIndex', 'roomInitSeed', 'decorationSeed', 'spawnSeed', 'followingSeed', 'doorMask'])
          phaseAssert.equal(state.room.descriptor[field], descriptor[field], `${label}:room descriptor ${field}`);
        for (const field of ['type', 'variant', 'name', 'width', 'height'])
          phaseAssert.equal(state.room.descriptor.template[field], descriptor.template[field], `${label}:authored identity ${field}`);
        phaseAssert.equal(state.room.descriptor.spawnSeed, original.context.publicLogState.spawnSeed, `${label}:actual room SpawnRNG log join`);
        phaseAssert.deepEqual(state.floor.grid, source.grid, `${label}:floor topology`);
        phaseAssert.deepEqual([state.player.x, state.player.y, state.player.vx, state.player.vy],
          [actor.position[0] * .65 + 32, actor.position[1] * .65 - 46, actor.velocity[0] * .65, actor.velocity[1] * .65], `${label}:source f32 motion projection`);
        if (['curse', 'free-treasure'].includes(scenario.doorContract)) {
          phaseAssert.equal(state.player.hp, actor.redHp, `${label}:native door-route health state`);
          phaseAssert.equal(state.damageCooldown, actor.damageCountdown, `${label}:door-route immunity and transition dispatch`);
          const sourceDoors = observed.contents.grid.filter(cell => cell.type === 16 && cell.variant !== 6);
          const actualDoors = state.room.grid?.cells.flatMap((cell, index) => cell.entity?.kind === 16 && cell.entity.variant !== 6 ? [index] : []) || [];
          phaseAssert.deepEqual(actualDoors, sourceDoors.map(door => door.index), `${label}:complete authored visible door objects`);
          phaseAssert.deepEqual(state.room.appearance.doors.map(door => door.slot).sort((a, b) => a - b),
            sourceDoors.map(door => [60, 7, 74, 127].indexOf(door.index)).sort((a, b) => a - b), `${label}:complete visible door appearances`);
          for (const door of sourceDoors) {
            const actual = state.room.grid?.cells[door.index]?.entity;
            phaseAssert.deepEqual([actual?.kind, actual?.variant, actual?.state, actual?.collisionClass],
              [door.type, door.variant, door.state, door.collisionClass], `${label}:door variant/collision phase`);
            const slot = [60, 7, 74, 127].indexOf(door.index);
            phaseAssert.ok(slot >= 0, `${label}:supported 13x7 source doorway slot`);
            const target = source.grid[descriptor.gridIndex + [-1, -13, 1, 13][slot]];
            phaseAssert.ok(Number.isInteger(target), `${label}:source door has a descriptor neighbor`);
            const targetType = source.descriptors[target]?.template.type;
            // Source LoadAnimation5629c0 chooses curse before treasure and
            // checks both independently observed native endpoint room types.
            const asset = [descriptor.template.type, targetType].includes(10)
              ? 'gfx/grid/door_04_selfsacrificeroomdoor.anm2'
              : [descriptor.template.type, targetType].includes(4)
                ? 'gfx/grid/door_02_treasureroomdoor.anm2'
                : 'gfx/grid/door_01_normaldoor.anm2';
            const appearance = state.room.appearance.doors.find(candidate => candidate.slot === slot);
            phaseAssert.deepEqual([appearance?.targetRoom, appearance?.asset], [target, asset], `${label}:source door endpoint/asset identity`);
          }
        }
        observedRooms.add(source.currentDescriptorIndex);
        phaseAssert.deepEqual(state.visitedRooms, [...observedRooms].sort((a, b) => a - b), `${label}:visited source room identities`);
        phaseAssert.deepEqual(Object.keys(state.persistedRooms).map(Number).sort((a, b) => a - b),
          [...observedRooms].filter(index => index !== source.currentDescriptorIndex).sort((a, b) => a - b), `${label}:retained exited rooms`);
        if (scenario.mode === 'blocked-return' && source.currentDescriptorIndex !== source.grid[84]) {
          for (const door of observed.contents.grid.filter(cell => cell.type === 16)) {
            const actual = state.room.grid?.cells[door.index]?.entity;
            phaseAssert.ok(actual, `${label}:source doorway collision object`);
            phaseAssert.deepEqual([actual?.kind, actual?.state, actual?.collisionClass],
              [door.type, door.state, door.collisionClass], `${label}:mandatory hostile-room door lock`);
          }
        }
        const sourceRocks = observed.contents.grid.filter(cell => cell.type === 2);
        const actualRocks = state.room.grid?.cells.flatMap((cell, index) => cell.entity?.kind === 2 ? [index] : []) || [];
        phaseAssert.deepEqual(actualRocks,
          sourceRocks.map(cell => cell.index).sort((a, b) => a - b), `${label}:complete supported ordinary Rock index set`);
        if (scenario.mode === 'blocked-return' && source.currentDescriptorIndex !== source.grid[84])
          phaseAssert.ok(sourceRocks.length > 0, 'Contrived source entry must exercise actual supported Rock objects');
        for (const rock of sourceRocks) {
          const actual = state.room.grid?.cells[rock.index]?.entity;
          phaseAssert.ok(actual, `${label}:supported native rock must have a runtime grid object`);
          phaseAssert.deepEqual([actual?.kind, actual?.variant, actual?.state, actual?.collisionClass, actual?.spawnSeed],
            [rock.type, rock.variant, rock.state, rock.collisionClass, rock.spawnSeed], `${label}:source rock factory fields`);
        }
        const admission = state.room.spawns;
        phaseAssert.ok(admission, 'A reached authored room must retain its selected contents');
        const selectors = admission?.selectors || [];
        const groups = provenance.authoredGroups[source.currentDescriptorIndex];
        phaseAssert.deepEqual(selectors.map(selector => selector.groupIndex), groups.map(group => group.groupIndex),
          `${label}:exact independently authored nonempty group accounting`);
        for (const group of groups) {
          const selector = selectors.find(selected => selected.groupIndex === group.groupIndex);
          phaseAssert.ok(selector, `${label}:every authored group has one retained selector`);
          if (!selector) continue;
          phaseAssert.ok(group.entries.some(entry => entry.type === selector.type && entry.variant === selector.variant
            && entry.subtype === selector.subtype), `${label}:selector raw type/variant/subtype must be an authored choice`);
          phaseAssert.equal(selector.category, selector.type >= 1 && selector.type < 1000 ? 'entity' : 'grid', `${label}:authored selector category`);
          phaseAssert.equal(selector.gridIndex, group.x + 1 + (group.y + 1) * (descriptor.template.width + 2), `${label}:authored selector grid index`);
          phaseAssert.deepEqual(selector.position, {x: 80 + group.x * 40, y: 160 + group.y * 40}, `${label}:authored selector position`);
        }
        if (!admissions.has(state.currentRoom)) {
          admissions.set(state.currentRoom, admission);
          // Stationary NPC244 and regular pickup admission join both ways.
          // Treasure routes also retain the native NPC33 fire admission
          // regression: raw1400 selectors have not yet resolved into actors.
          const rawMetadata = entity => entity.type === 244 || (scenario.doorContract === 'free-treasure' && entity.type === 33);
          const supported = entity => rawMetadata(entity) || (entity.type === 5 && ({10: [1, 2, 5], 20: [1, 2, 3, 4], 30: [1, 3], 40: [1, 2]})[entity.variant]?.includes(entity.subtype));
          const tuple = entity => [entity.type, entity.variant, entity.subtype, entity.spawnSeed,
            ...(Array.isArray(entity.position) ? entity.position : [entity.position.x, entity.position.y])];
          const multiset = entities => entities.map(tuple).map(JSON.stringify).sort();
          const actualEntities = [...selectors.filter(selector => selector.category === 'entity' && rawMetadata(selector)),
            ...(state.room.pickups || []).map(pickup => ({type: 5, ...pickup}))].filter(supported);
          phaseAssert.deepEqual(multiset(actualEntities), multiset(observed.contents.entities.filter(supported)),
            `${label}:complete supported native admission multiset with subtype/seed/position`);
        } else phaseAssert.deepEqual(admission, admissions.get(state.currentRoom), `${label}:return must retain the first selected authored groups`);
        states.push({ label, currentRoom: state.currentRoom, descriptor: state.room.descriptor, spawns: admission,
          player: state.player, damageCooldown: state.damageCooldown, nativePosition: actor.position,
          nativeVelocity: actor.velocity, nativeHp: actor.redHp, nativeDamageCountdown: actor.damageCountdown,
          doors: state.room.appearance.doors,
          rockMetadata: state.room.grid?.cells.flatMap((cell, index) => [2, 5, 6].includes(cell.entity?.kind)
            ? [{index, ...cell.entity}] : []) || [],
          visitedRooms: state.visitedRooms, transition: state.roomTransition });
        row.comparedPhases++;
      };
      const deliver = async action => {
        if (action.type === 'key') {
          const code = keyCodes[action.key];
          assert.ok(code); await page.keyboard[action.down ? 'down' : 'up'](code);
        } else if (action.type === 'step') {
          await page.evaluate(count => window.game.step(count), action.frames);
        } else {
          try { await compare(action.label); } catch (error) {
            const state = await page.evaluate(() => window.game.snapshot());
            comparisonFailures.push({label: action.label, error: String(error)});
            states.push({label: action.label, currentRoom: state.currentRoom, descriptor: state.room.descriptor,
              position: [state.player.x, state.player.y], velocity: [state.player.vx, state.player.vy], transition: state.roomTransition, status: 'fail'});
            row.comparedPhases++;
          }
        }
      };
      await compare('gameplay-start');
      for (const action of setup) await deliver(action);
      for (const action of scenario.actions) await deliver(action);
      assert.equal(observedRooms.size, 2, 'A complete declared route must reach exactly one authored neighbor');
      const expectedFinal = originals.get(scenario.actions.filter(action => action.type === 'capture').at(-1).label).observation.floor;
      if (scenario.mode === 'blocked-return') assert.notEqual(expectedFinal.currentDescriptorIndex, native.debugObservations[0].observation.floor.currentDescriptorIndex,
        'Mandatory source hostile-room regression must actually keep its attempted return locked');
      else if (scenario.mode === 'entry-only') assert.equal(expectedFinal.descriptors[expectedFinal.currentDescriptorIndex].template.type, scenario.goalRoomType,
        'The recorded public route must actually reach its declared native room kind');
      else assert.equal(expectedFinal.currentDescriptorIndex, native.debugObservations[0].observation.floor.currentDescriptorIndex,
        'Independent ordinary return route must actually reach the starting descriptor');
      if (scenario.doorContract === 'curse') {
        assert.equal(native.debugObservations[0].observation.contents.players[0].redHp, 6, 'Source fresh Isaac starts with three red hearts');
        assert.equal(originals.get('neighbor').observation.contents.players[0].redHp, 5, 'Source curse entrance must charge half a heart');
        assert.equal(originals.get('cooled').observation.contents.players[0].damageCountdown, 0, 'Public wait must clear source entry immunity');
        assert.equal(originals.get('returned').observation.contents.players[0].redHp, 4, 'Source curse exit must charge another half heart');
      }
      const trace = await page.evaluate(() => window.game.trace());
      await writeFile(join(directory, 'browser-states.json'), JSON.stringify({ schema: 1, states, trace, expectedCommands }, null, 2) + '\n');
      assertInputProvenance(trace, expectedCommands, scenario.runSeed);
      assertSeededRun(await page.evaluate(() => window.game.snapshot()), scenario.code, scenario.runSeed);
      assert.deepEqual(errors, []);
      Object.assign(row, { status: comparisonFailures.length ? 'fail' : 'pass', sourceManifest, provenance, visitedRooms: [...observedRooms], comparisonFailures,
        inputProvenance: { status: 'pass', commands: trace.commands.length, initialSeed: trace.initialSeed,
          bootEpochSeed: trace.bootEpochSeed, manualSeed: scenario.code },
        traceHistory: { truncated: trace.truncated, inputTraceTruncated: trace.inputTraceTruncated, stateHistoryTruncated: trace.stateHistoryTruncated,
          retainedStates: trace.states.length, stateEncodedBytes: trace.stateEncodedBytes, maxStateEncodedBytes: trace.maxStateEncodedBytes } });
    } catch (error) { row.error = String(error); }
    finally { await page?.close(); report.cases.push(row); }
  }
  report.status = report.cases.every(row => row.status === 'pass') ? 'pass' : 'fail';
} finally {
  await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  await browser.close(); await production.close();
}
console.log(JSON.stringify({ status: report.status, cases: report.cases.map(({seed, status, comparedPhases, comparisonFailures, error}) => ({seed, status, comparedPhases, failures: comparisonFailures?.length || 0, error})), report: join(output, 'report.json') }));
process.exitCode = report.status === 'pass' ? 0 : 1;
