// Live original and production-browser ordinary doorway/return contracts.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { promisify } from 'node:util';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';
import { chromium } from '@playwright/test';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { assertInputProvenance, assertSeededRun, enterSeededRun, manualSeedCommands, waitForReady } from '../foundation/verify.mjs';
const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const run = promisify(execFile);
const value = flag => { const index = process.argv.indexOf(flag); return index < 0 ? undefined : process.argv[index + 1]; };
const nativeProcessTimeoutSeconds = Number(value('--native-process-timeout') || '600');
assert.ok(Number.isInteger(nativeProcessTimeoutSeconds) && nativeProcessTimeoutSeconds > 0
  && nativeProcessTimeoutSeconds <= 3600, 'Native process timeout must be 1..3600 host seconds');
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
  scope: bank.contract === 'large-geometry'
    ? 'Fresh Isaac enters an authored ordinary13x14 room through public movement, follows both radius-inset vertical boundary planes and returns. Exact full/intermediate player motion, native dimensions/bounds, complete authored door indices/positions/endpoints, source descriptor/spawn/visited/minimap joins and supported Rock state are required. Actual Chest5/50 admission remains a mandatory unsupported-actor regression. Source camera offsets are retained as diagnostics; camera/backdrop rendering and chest callbacks remain separate.'
    : bank.contract === 'keeper-idle'
    ? 'Fresh Isaac enters real13x7 secret rooms through public bomb/movement inputs. Actual NPC17/1 constructor and anchored idle compare complete identity/seed/subtype, authored collision/health/countability, source state/flags/anchor/color, static and animated Guy clips, every full/intermediate/frozen phase and room/player/door/minimap joins. Damage/drop callbacks and other Keeper variants remain separate.'
    : bank.contract === 'curse'
    ? 'Fresh ordinary Isaac, authored 13x7 no-hostile curse-room entry and exit; exact native half-heart charges, immunity/door collision/admission phases, descriptor/spawn identity and returned room persistence. Flight/items, other door styles, hostile callbacks and complete pixels remain separate.'
    : bank.contract === 'authored-doors'
      ? 'Fresh ordinary Isaac, authored13x7 curse and first-floor free treasure doors; recorded W0L0LYYM bomb-driven admission and raw1400 fire admission remain mandatory. Exact room/player/HP/immunity/visible door phases, complete authored groups and return metadata; actor/item callbacks and pixels remain separate.'
      : bank.contract === 'secret'
        ? 'Fresh Isaac, no-hostile13x7 Basement secret bomb revelation/entry/return: complete hidden/open door objects, reciprocal persistence, source display flags and minimap routing, exact room/player/HP/immunity and authored admissions. Keeper damage/drop and item callbacks, hostile secret closure, other shapes/items and complete pixels remain separate.'
        : bank.contract === 'key'
          ? 'Fresh Isaac, no-hostile13x7 natural secret keys and ordinary library lock: denied zero-key approach, actual collection/spending, full-only unlock/collision/SFX156, admission/return/revisit, exact player/HP/immunity, complete door/display/minimap/descriptor joins. Actual Keeper17 constructor admission remains required; golden/double locks, library items and pixels remain separate.'
        : 'Ordinary style0 13x7 doorway and no-hostile return; mandatory hostile-room lock/return regression; exact f32 player/descriptor phases, complete authored group accounting, supported actor admission and ordinary Rock metadata, visited-room restoration. NPC behavior, other grids/doors, large-room camera and complete pixels remain separate.',
  exclusions: bank.contract === 'large-geometry'
    ? ['Default Style1 camera controller, large-room backdrop generation and rendered pixel parity', 'Chest5/50 constructor/animation/contact/drop behavior remains a required admission failure', 'Boss/shop and other special-door art/policy/traversal, hostile large-room lifecycle, other large shapes, later floors and exits']
    : bank.contract === 'keeper-idle'
    ? ['Keeper damage/death/drop callbacks and actor mutation persistence', 'Other Keeper variants, hostile secret policy, other room shapes and floor exits', 'Complete sprite/backdrop/pixel parity and total shared MT consumer alignment']
    : bank.contract === 'secret'
    ? ['Keeper17 constructor and regular keys are concrete actors; Keeper damage/drop callbacks remain separate', 'Later-stage surprise substitution and failed-open saved transitions', 'Actor/item/curse policies and door FX/animation pixels', 'Other room shapes and floor exits']
    : bank.contract === 'key' ? ['Keeper17 damage/death/drop callbacks remain separate', 'Golden keys, double-chain locks, item overrides and hostile special-door policy', 'Library collectibles/sacks/batteries and complete backdrop/door animation pixels', 'Other room shapes, later floors and exits']
    : ['curse', 'authored-doors'].includes(bank.contract) ? ['Undiscovered native door variant6', 'Actor behavior and item callbacks; NPC244 metadata only, raw1400 fire admission remains a required regression', 'Flight/items and other door policies', 'Complete rendered pixels'] : [],
  nativeProcessTimeoutSeconds, production: production.build, graphics: await inspectBrowserGraphics(browser), cases: [] };
try {
  for (const scenario of bank.cases) {
    const row = { seed: scenario.code, status: 'fail', comparedPhases: 0 };
    let page;
    try {
      const directory = join(output, scenario.code); await mkdir(directory, { recursive: true });
      // Saved scenarios replay public inputs, including when an older research
      // bank carries a sourceManifest path. Every gate launches a fresh original.
      const tape = join(directory, 'tape.json');
      await writeFile(tape, JSON.stringify(scenario.tape, null, 2) + '\n');
      row.originalExecution = { mode: 'fresh-process', timeoutSeconds: nativeProcessTimeoutSeconds,
        ignoredCachedReference: Boolean(scenario.sourceManifest) };
      try {
        await run('bash', [join(ROOT, 'tools/original/with-display.sh'), 'python3',
          join(ROOT, 'tools/original/foundation-clock-run.py'), '--seed', scenario.code, '--tape', tape,
          '--state-observer', value('--state-observer') || join(ROOT, 'tools/original/foundation-traversal-observe.py'),
          '--output', join(directory, 'original')], { cwd: ROOT, timeout: nativeProcessTimeoutSeconds * 1000, maxBuffer: 8 * 1024 * 1024 });
      } catch (error) {
        row.originalExecution.failure = { code: error.code ?? null, signal: error.signal ?? null,
          killed: Boolean(error.killed), message: String(error.message) };
        throw error;
      }
      const sourceManifest = join(directory, 'original/manifest.json');
      row.originalExecution.fresh = true;
      const provenance = await python(['--validate-original', sourceManifest]);
      assert.equal(provenance.status, 'pass');
      const native = JSON.parse(await readFile(sourceManifest, 'utf8'));
      assert.equal(native.seedValue, scenario.runSeed);
      const originals = new Map(native.debugObservations.map(entry => [entry.context.label, entry]));
      const setup = scenario.setup || [{ type: 'step', frames: 60 }, ...(scenario.slot % 2 === 0
        ? [{ type: 'key', key: 'w', down: true }, { type: 'step', frames: 27 }, { type: 'key', key: 'w', down: false }, { type: 'step', frames: 30 }] : [])];
      const keyCodes = { a: 'KeyA', w: 'KeyW', d: 'KeyD', s: 'KeyS', e: 'KeyE', Up: 'ArrowUp',
        Down: 'ArrowDown', Left: 'ArrowLeft', Right: 'ArrowRight', Escape: 'Escape' };
      const expectedCommands = [...manualSeedCommands(scenario.code),
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
      const seeded = await enterSeededRun(page, scenario.code);
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
        if (scenario.doorContract === 'large-geometry') {
          const nativeRoom = observed.contents.room;
          phaseAssert.deepEqual([state.room.grid?.width, state.room.grid?.height, state.room.grid?.cells.length],
            [nativeRoom.gridWidth, nativeRoom.gridHeight, nativeRoom.gridWidth * nativeRoom.gridHeight], `${label}:complete native padded room dimensions`);
          phaseAssert.deepEqual([state.room.left, state.room.right, state.room.top, state.room.bottom],
            [(nativeRoom.boundsMin[0] + actor.collisionRadius) * .65 + 32,
              (nativeRoom.boundsMax[0] - actor.collisionRadius) * .65 + 32,
              (nativeRoom.boundsMin[1] + actor.collisionRadius) * .65 - 46,
              (nativeRoom.boundsMax[1] - actor.collisionRadius) * .65 - 46], `${label}:authored radius-inset room bounds`);
          phaseAssert.equal(state.player.hp, actor.redHp, `${label}:native tall-room route health`);
          phaseAssert.equal(state.damageCooldown, actor.damageCountdown, `${label}:native tall-room transition dispatch`);
          const sourceDoors = observed.contents.grid.filter(cell => cell.type === 16);
          const actualDoorIndices = state.room.grid?.cells.flatMap((cell, index) => cell.entity?.kind === 16 ? [index] : []) || [];
          phaseAssert.deepEqual(actualDoorIndices, sourceDoors.map(door => door.index).sort((a, b) => a - b), `${label}:complete authored tall-room door grid objects`);
          const authoredDoors = provenance.authoredDoors[source.currentDescriptorIndex];
          const connected = authoredDoors.filter(door => descriptor.doorMask & (1 << door.slot));
          phaseAssert.deepEqual(sourceDoors.map(door => door.index).sort((a, b) => a - b),
            connected.map(door => door.index).sort((a, b) => a - b), `${label}:independent complete source-connected authored door set`);
          phaseAssert.deepEqual(state.room.appearance.doors.map(door => door.slot).sort((a, b) => a - b),
            connected.map(door => door.slot).sort((a, b) => a - b), `${label}:complete authored tall-room door appearances`);
          const width = descriptor.template.width / 13, height = descriptor.template.height / 7;
          const neighborOffsets = [-1, -13, width, 13 * height, 12, -12, width + 13, 13 * height + 1];
          for (const door of sourceDoors) {
            const authored = authoredDoors.find(candidate => candidate.index === door.index);
            phaseAssert.ok(authored, `${label}:native door must have one independently authored boundary coordinate`);
            if (!authored) continue;
            const actual = state.room.grid?.cells[door.index]?.entity;
            phaseAssert.deepEqual([actual?.kind, actual?.variant, actual?.state, actual?.collisionClass],
              [door.type, door.variant, door.state, door.collisionClass], `${label}:native tall-room door variant/state/class`);
            const target = source.grid[descriptor.gridIndex + neighborOffsets[authored.slot]];
            phaseAssert.ok(Number.isInteger(target), `${label}:authored extended slot has a source floor neighbor`);
            const types = [descriptor.template.type, source.descriptors[target]?.template.type];
            const asset = types.some(type => [7, 8].includes(type)) ? 'gfx/grid/door_08_holeinwall.anm2'
              : types.includes(10) ? 'gfx/grid/door_04_selfsacrificeroomdoor.anm2'
              : types.includes(4) ? 'gfx/grid/door_02_treasureroomdoor.anm2' : 'gfx/grid/door_01_normaldoor.anm2';
            const appearance = state.room.appearance.doors.find(candidate => candidate.slot === authored.slot);
            phaseAssert.deepEqual([appearance?.targetRoom, appearance?.worldPosition],
              [target, {x: authored.position[0], y: authored.position[1]}], `${label}:independent door endpoint/world position`);
            if (types.every(type => [1, 4, 7, 8, 10, 12].includes(type)))
              phaseAssert.equal(appearance?.asset, asset, `${label}:source-supported door endpoint asset identity`);
          }
          phaseAssert.deepEqual(state.roomDisplayFlags, source.descriptors.map(room => room.displayFlags), `${label}:complete tall-room source display flags`);
          const cells = source.descriptors.filter(room => room.displayFlags & 1).map(room => ({
            x: room.gridIndex % 13 - 6, y: Math.floor(room.gridIndex / 13) - 6,
            visibility: room.index === source.currentDescriptorIndex ? 'current' : room.visited ? 'visited' : 'unvisited',
          }));
          phaseAssert.deepEqual(state.minimap.cells, cells, `${label}:complete tall-room descriptor minimap routing`);
        }
        if (['curse', 'free-treasure', 'secret', 'key'].includes(scenario.doorContract)) {
          const includeHidden = ['secret', 'key'].includes(scenario.doorContract);
          phaseAssert.equal(state.player.hp, actor.redHp, `${label}:native door-route health state`);
          phaseAssert.equal(state.damageCooldown, actor.damageCountdown, `${label}:door-route immunity and transition dispatch`);
          const sourceDoors = observed.contents.grid.filter(cell => cell.type === 16 && (includeHidden || cell.variant !== 6));
          const actualDoors = state.room.grid?.cells.flatMap((cell, index) => cell.entity?.kind === 16 && (includeHidden || cell.entity.variant !== 6) ? [index] : []) || [];
          phaseAssert.deepEqual(actualDoors, sourceDoors.map(door => door.index), `${label}:complete authored visible door objects`);
          phaseAssert.deepEqual(state.room.appearance.doors.filter(door => includeHidden || door.state !== 'Hidden').map(door => door.slot).sort((a, b) => a - b),
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
            const types = [descriptor.template.type, targetType];
            const asset = types.some(type => [7, 8].includes(type))
              ? 'gfx/grid/door_08_holeinwall.anm2'
              : types.includes(10)
              ? 'gfx/grid/door_04_selfsacrificeroomdoor.anm2'
              : [descriptor.template.type, targetType].includes(4)
                ? 'gfx/grid/door_02_treasureroomdoor.anm2'
                : 'gfx/grid/door_01_normaldoor.anm2';
            const appearance = state.room.appearance.doors.find(candidate => candidate.slot === slot);
            phaseAssert.deepEqual([appearance?.targetRoom, appearance?.asset], [target, asset], `${label}:source door endpoint/asset identity`);
            if (includeHidden) phaseAssert.equal(appearance?.blownOpen, door.blownOpen, `${label}:actual source blown-open door flag`);
          }
          if (includeHidden) {
            phaseAssert.deepEqual(state.floor.descriptors.map(room => room.surpriseMiniboss),
              source.descriptors.map(room => room.doorPolicyFlag ?? room.hasEnemies), `${label}:generated surprise-room policy flag`);
            phaseAssert.deepEqual(state.roomDisplayFlags, source.descriptors.map(room => room.displayFlags), `${label}:complete source descriptor display flags`);
            const cells = source.descriptors.filter(room => room.displayFlags & 1).map(room => ({
              x: room.gridIndex % 13 - 6, y: Math.floor(room.gridIndex / 13) - 6,
              visibility: room.index === source.currentDescriptorIndex ? 'current' : room.visited ? 'visited' : 'unvisited',
            }));
            phaseAssert.deepEqual(state.minimap.cells, cells, `${label}:complete source descriptor-derived minimap routing`);
          }
          if (scenario.doorContract === 'key') {
            phaseAssert.equal(state.inventory.keys, actor.keyInventory, `${label}:natural source key acquisition/spending`);
            phaseAssert.equal(state.room.combatClear, descriptor.combatClear, `${label}:source cleared-room unlock policy`);
            for (const door of state.room.appearance.doors) {
              const expected = [14, 15].includes(observed.contents.room.backdropType) && door.asset === 'gfx/grid/door_01_normaldoor.anm2'
                ? 'gfx/grid/door_13_librarydoor.png' : null;
              phaseAssert.equal(door.spritesheet, expected, `${label}:source current-room door palette replacement`);
            }
            const baseline = native.debugObservations[0].observation;
            if (observed.doorSound.lastTick !== baseline.doorSound.lastTick) {
              const sound = state.audio.slots[156], halfTick = baseline.audioSystemCounter >> 1;
              const browserHalfTick = seeded.systemClock.systemCounter >> 1;
              phaseAssert.equal(sound.lastTick - browserHalfTick, observed.doorSound.lastTick - halfTick, `${label}:native unlock sound admission phase`);
              phaseAssert.equal(sound.nextAllowed - browserHalfTick, observed.doorSound.nextAllowed - halfTick, `${label}:native unlock sound cooldown`);
              phaseAssert.equal(Math.fround(sound.lastVolume), observed.doorSound.lastVolume, `${label}:native unlock sound volume`);
              phaseAssert.equal(sound.loaded, observed.doorSound.loaded, `${label}:native unlock sound loaded state`);
            }
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
          const rawMetadata = entity => entity.type === 244
            || (['secret', 'key'].includes(scenario.doorContract) && entity.type === 17 && entity.variant !== 1)
            || (scenario.doorContract === 'free-treasure' && entity.type === 33);
          const supported = entity => rawMetadata(entity)
            || (['secret', 'key'].includes(scenario.doorContract) && entity.type === 17 && entity.variant === 1)
            || (scenario.doorContract === 'large-geometry' && entity.type === 5 && entity.variant === 50)
            || (entity.type === 5 && ({10: [1, 2, 5], 20: [1, 2, 3, 4], 30: [1, 3], 40: [1, 2]})[entity.variant]?.includes(entity.subtype));
          const tuple = entity => [entity.type, entity.variant, entity.subtype, entity.spawnSeed,
            ...(Array.isArray(entity.position) ? entity.position : [entity.position.x, entity.position.y])];
          const multiset = entities => entities.map(tuple).map(JSON.stringify).sort();
          const actualEntities = [...selectors.filter(selector => selector.category === 'entity' && rawMetadata(selector)
            && !(state.room.npcs || []).some(npc => npc.kind === selector.type && npc.variant === selector.variant && npc.spawnSeed === selector.spawnSeed)),
            ...(state.room.npcs || []).map(npc => ({...npc, type: npc.kind})),
            ...(state.room.pickups || []).map(pickup => ({type: 5, ...pickup}))].filter(supported);
          phaseAssert.deepEqual(multiset(actualEntities), multiset(observed.contents.entities.filter(supported)),
            `${label}:complete supported native admission multiset with subtype/seed/position`);
        } else phaseAssert.deepEqual(admission, admissions.get(state.currentRoom), `${label}:return must retain the first selected authored groups`);
        if (scenario.npcContract === 'keeper-idle') {
          const actualKeepers = state.room.npcs.filter(npc => npc.kind === 17 && npc.variant === 1);
          const nativeKeepers = observed.contents.entities.filter(npc => npc.type === 17 && npc.variant === 1);
          const actual = npc => [npc.variant, npc.subtype, npc.spawnSeed, npc.position, npc.presentedPosition,
            npc.velocity, npc.hp, npc.maxHp, npc.collisionRadius, npc.collisionMass, npc.collisionDamage,
            npc.baseFriction, npc.gridCollisionClass, npc.entityCollisionClass, npc.canShutDoors, npc.boss,
            npc.state, npc.frameAge, npc.keeper?.entityFlags, npc.keeper?.stateFrame, npc.keeper?.countdown,
            npc.keeper?.anchor, npc.keeper?.animation, npc.keeper?.animationClock, npc.keeper?.animationPlaying,
            npc.keeper?.tint.map(Math.fround), npc.keeper?.offsets.map(Math.fround), npc.damage.pendingDamage, npc.removed];
          const vector = pair => ({x: pair[0], y: pair[1]});
          const native = npc => [npc.variant, npc.subtype, npc.spawnSeed, vector(npc.position), vector(npc.position),
            vector(npc.velocity), npc.hp, npc.maxHp, npc.collisionRadius, npc.collisionMass, npc.collisionDamage,
            npc.friction, npc.gridCollisionClass, npc.entityCollisionClass, npc.canShutDoors, npc.boss,
            npc.npcState, npc.npcFrameAge, npc.npcEntityFlags, npc.npcStateFrame, npc.npcCountdown,
            vector(npc.npcAnchor), npc.animation, npc.animationFrame, npc.animationPlaying,
            npc.npcColor.slice(0, 4), npc.npcColor.slice(8, 11), npc.pendingDamage, npc.removed];
          phaseAssert.deepEqual(actualKeepers.map(actual), nativeKeepers.map(native), `${label}:complete actual Keeper constructor and anchored idle fields`);
          phaseAssert.ok(actualKeepers.every(npc => npc.appearanceEffectSeed === null && npc.deathEffectSeed === null),
            `${label}:Keeper initialization never uses Fly appearance/blood shared draws`);
        }
        states.push({ label, state, currentRoom: state.currentRoom, descriptor: state.room.descriptor, spawns: admission,
          keepers: state.room.npcs.filter(npc => npc.kind === 17),
          nativeKeepers: observed.contents.entities.filter(npc => npc.type === 17),
          player: state.player, inventory: state.inventory, nativeKeys: actor.keyInventory,
          combatClear: state.room.combatClear, nativeCombatClear: descriptor.combatClear,
          damageCooldown: state.damageCooldown, nativePosition: actor.position,
          nativeVelocity: actor.velocity, nativeHp: actor.redHp, nativeDamageCountdown: actor.damageCountdown,
          roomGeometry: scenario.doorContract === 'large-geometry' ? {
            actualBounds: [state.room.left, state.room.right, state.room.top, state.room.bottom],
            actualGridDimensions: [state.room.grid?.width, state.room.grid?.height], source: observed.contents.room,
            nativeChests: observed.contents.entities.filter(entity => entity.type === 5 && entity.variant === 50),
          } : undefined,
          roomDisplayFlags: state.roomDisplayFlags, minimap: state.minimap,
          nativeRoomDisplayFlags: source.descriptors.map(room => room.displayFlags),
          doors: state.room.appearance.doors,
          doorMetadata: state.room.grid?.cells.flatMap((cell, index) => cell.entity?.kind === 16 ? [{index, ...cell.entity}] : []) || [],
          nativeDoorMetadata: observed.contents.grid.filter(cell => cell.type === 16),
          nativeBackdropType: observed.contents.room.backdropType,
          unlockSound: scenario.doorContract === 'key' ? state.audio.slots[156] : undefined,
          nativeUnlockSound: observed.doorSound,
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
      assert.equal(observedRooms.size, scenario.expectedVisitedRooms || 2, 'A complete declared route must reach every intended authored room');
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
      if (['secret', 'key'].includes(scenario.doorContract)) {
        const start = native.debugObservations[0].observation;
        assert.equal(start.contents.grid.find(door => door.index === 127)?.variant, 6, 'Source must begin with an undiscovered secret doorway');
        const blast = originals.get('blast-133').observation;
        assert.deepEqual(blast.contents.grid.filter(door => door.index === 127).map(door => [door.variant, door.state, door.blownOpen]), [[7, 2, true]], 'Actual public bomb must reveal the source doorway');
        const entered = originals.get('secret-entry').observation.floor;
        assert.ok([7, 8].includes(entered.descriptors[entered.currentDescriptorIndex].template.type), 'Public source route must enter an actual secret room');
        assert.ok(native.debugObservations.every(entry => entry.observation.contents.players[0].redHp === 6), 'Public retreat must avoid source blast damage');
      }
      if (scenario.doorContract === 'key') {
        const keys = native.debugObservations.map(entry => entry.observation.contents.players[0].keyInventory);
        assert.equal(keys[0], 0, 'Source fresh Isaac must start without supplied keys');
        assert.equal(Math.max(...keys), 2, 'Public source movement must collect both natural regular keys');
        assert.equal(keys.at(-1), 1, 'Exactly one key must be spent despite library revisit');
        assert.ok(native.debugObservations.some(entry => {
          const floor = entry.observation.floor;
          return floor.descriptors[floor.currentDescriptorIndex].template.type === 12;
        }), 'Public source route must enter the actual library');
        if (scenario.actions.some(action => action.label?.startsWith('no-key-approach')))
          for (const entry of native.debugObservations.filter(entry => entry.context.label.startsWith('no-key-approach'))) {
            assert.equal(entry.observation.contents.players[0].keyInventory, 0, 'Denied route must have no source key');
            const door = entry.observation.contents.grid.find(cell => cell.index === 7);
            assert.deepEqual([door.variant, door.state, door.collisionClass], [1, 1, 4], 'Zero-key public approach must retain source key lock');
          }
      }
      const trace = await page.evaluate(() => window.game.trace());
      const browserBytes = Buffer.from(JSON.stringify({ schema: 1, states, trace, expectedCommands }, null, 2) + '\n');
      const archive = gzipSync(browserBytes, { level: 1 });
      const archivePath = join(directory, 'browser-states.json.gz');
      await writeFile(archivePath, archive);
      row.browserEvidence = {
        path: archivePath, encoding: 'gzip', rawBytes: browserBytes.length,
        rawSha256: createHash('sha256').update(browserBytes).digest('hex'),
        archiveBytes: archive.length, archiveSha256: createHash('sha256').update(archive).digest('hex'),
        scope: 'Every selected phase retains its full actual browser snapshot, alongside selected native comparisons, complete public commands and backend callbacks. The separate trace history caps are disclosed in traceHistory; decompress before replay or inspection',
      };
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
