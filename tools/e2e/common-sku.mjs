// Public normal-run resource recipes and actual independent game captures.
import assert from 'node:assert/strict';
import {randomBytes, createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {readFileSync, writeFileSync, mkdirSync, openSync, closeSync} from 'node:fs';
import {join, resolve, sep} from 'node:path';
import {chromium} from 'playwright';
import {ROOT, flag, hasFlag, outputDirectory, selectCases, writeJson} from './lib/cli.mjs';
import {assertOriginalBinary} from './lib/original.mjs';
import {startProductionServer} from '../foundation/production-server.mjs';
import {browserLaunchOptions, inspectBrowserGraphics} from '../foundation/browser.mjs';
import {captureComposition} from '../foundation/composition-capture.mjs';

if (hasFlag('--help')) {
  console.log('common-sku [--count 1 | --scenarios BANK] [--case PATTERN] [--prepare-only] [--production-directory DIST] [--reuse-original ROOT] [--output DIR]');
  console.log('Fixed four SKU branches plus recorded unfiltered OS idle-approach cases on a fixed normal-run seed. Local Base assets are required; generated fixtures remain ignored. Replay draws no entropy.');
  process.exit(0);
}
const output = outputDirectory('common-sku');
assert.ok(resolve(output).startsWith(resolve(ROOT, '.cache') + sep),
  'Private generated resource fixtures and captures must stay below ignored .cache');
const prepareOnly = hasFlag('--prepare-only'), reuse = flag('--reuse-original');
assert.ok(!prepareOnly || !reuse, 'Preparation does not reuse or grade captures');
const digest = file => createHash('sha256').update(readFileSync(file)).digest('hex');
const load = file => JSON.parse(readFileSync(file, 'utf8'));
const save = (name, value) => writeJson(join(output, name), value);
const fixedPath = join(ROOT, 'references/isaac/cases/common-sku/source-fixtures-v1.json');
const plansPath = join(ROOT, 'references/isaac/cases/common-sku/source-plans-v1.json');
const fixed = load(fixedPath);
const originals = ['foundation-clock-run.py', 'foundation-clock-verify.py', 'foundation-clock-gate.c',
  'foundation-paired-scenarios.py', 'foundation-paired-observer.py', 'foundation-menu-observer.py',
  'foundation-combat-observe.py', 'foundation-traversal-observe.py', 'foundation-layout-observer.py',
  'foundation-descriptor-observer.py', 'foundation-pickup-observe.py', 'composition-run.py',
  'item-observer.py', 'audio-observer.py', 'state-observer.py', 'native-identity.py',
  'native-evidence.py', 'native-journal-zstd.py', 'with-display.sh', 'foundation-seeds.py', 'debug-run.py',
  'common-sku-run.py', 'common-sku-observe.py', 'shop-finance-observe.py',
  'shop-finance-law.py', 'shop-finance-layout-observe.py', 'shop-finance-presentation-observe.py'];
const sourceFiles = [join(ROOT, 'tools/e2e/common-sku.mjs'), fixedPath, plansPath,
  ...originals.map(name => join(ROOT, 'tools/original', name)),
  ...['production-server.mjs', 'browser.mjs', 'composition-capture.mjs', 'composition-progress.mjs',
    'browser-evidence.mjs', 'composition-readback.py', 'common-sku-state-pair.py',
    'prepare-shop-sku-fixtures.py', 'decode-rooms.py'].map(name => join(ROOT, 'tools/foundation', name)),
  ...['cli.mjs', 'host-slots.mjs', 'original.mjs'].map(name => join(ROOT, 'tools/e2e/lib', name)),
  ...['challenges.xml', 'sounds.xml', 'rooms/00.special rooms.stb'].map(name => join(ROOT, 'references/isaac/rebirth/resources', name))];
const pins = files => Object.fromEntries([...new Set(files)].map(file => [file, digest(file)]));
function child(command, args, logfile) {
  return new Promise((done, reject) => {
    const fd = openSync(logfile, 'wx');
    const task = spawn(command, args, {cwd: ROOT, env: process.env, stdio: ['ignore', fd, fd]});
    task.once('error', error => {closeSync(fd); reject(error);});
    task.once('exit', (exitCode, signal) => {closeSync(fd); done({exitCode, signal});});
  });
}

function approachCase(draw, ordinal) {
  assert.ok(Number.isInteger(draw) && draw >= 0 && draw <= 0xffffffff);
  const result = structuredClone(fixed.cases.find(row => row.commonSku.branch === 'free-no-active'));
  const extra = draw % 32;
  assert.deepEqual(result.actions[0].keys, []);
  assert.equal(result.actions[0].presentations, 96);
  result.actions[0].presentations += extra;
  for (const alias of result.commonSku.publicCheckpointAliases) {
    assert.ok(alias.gameplayPresentation >= 96);
    alias.gameplayPresentation += extra;
  }
  result.name = `os-free-no-active-${String(ordinal).padStart(4, '0')}`;
  result.origin = 'fresh-os-public-idle-approach';
  result.approachDraw = {ordinal, u32: draw, extraReleasedPresentations: extra,
    scope: 'Unfiltered OS public idle approach on fixed PWTJ run seed; no random-floor or Native-conditioned claim'};
  return result;
}

const report = {schema: 1, status: 'starting', cases: [],
  scope: 'Source-only locally reconstructed normal resource fixtures; public keys/ticks only. Actual Native and headed Chromium with observed graphics backend, typed source/owner coverage and transport remain blocking. PNGs retained; RGBA/PCM not computed.'};
writeFileSync(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n', {flag: 'wx'});
try {
  report.sourcePinsBefore = pins(sourceFiles);
  const replay = flag('--scenarios');
  let bank;
  if (replay !== undefined) {
    assert.ok(replay.trim(), 'Recorded bank path cannot be empty');
    bank = load(resolve(replay));
    report.replayInput = {path: resolve(replay), sha256: digest(resolve(replay))};
    report.inputMode = 'recorded public bank; no entropy';
  } else {
    const count = Number(flag('--count', '1'));
    assert.ok(Number.isInteger(count) && count >= 0 && count <= 8);
    const draws = [];
    for (let ordinal = 0; ordinal < count; ordinal++) {
      draws.push({ordinal, u32: randomBytes(4).readUInt32LE()});
      // Close each actual draw before mapping inputs; every draw remains selected.
      save('approach-draws.json', {schema: 1, kind: 'fresh-os-public-idle-approach', draws});
    }
    bank = {...fixed, cases: [...fixed.cases, ...draws.map(row => approachCase(row.u32, row.ordinal))],
      discovery: {kind: 'fresh-os-public-idle-approach', draws, selectedOrdinals: draws.map(row => row.ordinal)}};
    report.inputMode = 'fixed contracts plus every recorded unfiltered OS idle approach';
  }
  assert.equal(bank.schema, 1); assert.equal(bank.suite, 'common-sku');
  assert.ok(Array.isArray(bank.cases) && bank.cases.length > 0);
  assert.equal(new Set(bank.cases.map(row => row.name)).size, bank.cases.length);
  for (const row of bank.cases) {
    assert.match(row.name, /^[A-Za-z0-9_-]{1,80}$/);
    assert.ok(!Object.hasOwn(row, 'fixture'), 'Public inputs must resolve a named local recipe');
    if (row.origin === 'contrived') {
      const expected = fixed.cases.find(item => item.name === row.name);
      assert.ok(expected, 'Unknown fixed common SKU case'); assert.deepEqual(row, expected);
    } else {
      assert.equal(row.origin, 'fresh-os-public-idle-approach');
      assert.deepEqual(row, approachCase(row.approachDraw.u32, row.approachDraw.ordinal));
      assert.ok(bank.discovery && Array.isArray(bank.discovery.draws));
      assert.deepEqual(bank.discovery.draws[row.approachDraw.ordinal],
        {ordinal: row.approachDraw.ordinal, u32: row.approachDraw.u32});
    }
  }
  const partial = bank.caseFilter !== undefined;
  if (partial) {
    assert.ok(typeof bank.caseFilter === 'string' && bank.caseFilter.trim());
    assert.equal(selectCases(bank.cases, row => row.name,
      ['node', 'common-sku', '--case', bank.caseFilter]).length, bank.cases.length,
    'A stored partial filter must describe every retained case');
  } else {
    for (const required of fixed.cases) assert.ok(bank.cases.some(row => row.name === required.name),
      'A whole bank must retain every fixed common SKU contract');
  }
  if (bank.discovery) {
    const draws = bank.discovery.draws;
    assert.equal(bank.discovery.kind, 'fresh-os-public-idle-approach');
    assert.ok(Array.isArray(draws) && draws.length <= 8 && draws.every((row, ordinal) =>
      row.ordinal === ordinal && Number.isInteger(row.u32) && row.u32 >= 0 && row.u32 <= 0xffffffff));
    assert.deepEqual(bank.discovery.selectedOrdinals, draws.map(row => row.ordinal));
    const retained = bank.cases.filter(row => row.origin !== 'contrived').map(row => row.approachDraw.ordinal);
    if (!partial) assert.deepEqual(retained, bank.discovery.selectedOrdinals);
    else assert.ok(retained.every(ordinal => bank.discovery.selectedOrdinals.includes(ordinal)));

  }
  const declaredFull = [...fixed.cases, ...(bank.discovery?.draws ?? []).map(row => approachCase(row.u32, row.ordinal))];
  const expectedStored = partial ? selectCases(declaredFull, row => row.name,
    ['node', 'common-sku', '--case', bank.caseFilter]) : declaredFull;
  assert.deepEqual(bank.cases, expectedStored, 'Every declared selected input must remain, in source order');
  const selected = selectCases(bank.cases, row => row.name);
  assert.ok(selected.length > 0, 'A case filter must select at least one actual declared case');
  const selectedBank = {...bank, cases: selected, ...(flag('--case') ? {caseFilter: flag('--case')} : {})};
  save('scenarios.json', selectedBank);
  const modes = [...new Set(selected.map(row => row.fixtureRecipe.mode))];
  const prepared = join(output, 'local-fixtures');
  const declaration = await child('python3', [join(ROOT, 'tools/foundation/prepare-shop-sku-fixtures.py'),
    '--output', prepared, ...modes.flatMap(mode => ['--case', mode])], join(output, 'fixtures.log'));
  assert.equal(declaration.exitCode, 0, 'Local source recipe failed; retain its log and no substitution');
  const fixtures = load(join(prepared, 'local-fixtures.json'));
  const resolved = selected.map(row => {
    const matches = fixtures.cases.filter(item => item.name === 'battery90-' + row.fixtureRecipe.mode);
    assert.equal(matches.length, 1, 'Exactly one local source recipe must resolve');
    const fixture = {...matches[0]}; delete fixture.name; delete fixture.recipe;
    fixture.specialRoomsSha256 = digest(fixture.specialRoomsPath);
    return {...row, fixture};
  });
  const resources = [...new Set(resolved.flatMap(row => [row.fixture.authoredResourcePath, row.fixture.specialRoomsPath]))];
  report.resourcePinsBefore = pins(resources);
  save('resolved-source-inputs.json', {schema: 1, cases: resolved, sourcePlans: load(plansPath)});
  report.declaredOrder = resolved.map(row => row.name);
  if (!prepareOnly) await assertOriginalBinary();
  for (const declaration of resolved) {
    const directory = join(output, declaration.name); mkdirSync(directory);
    const caseFile = join(directory, 'case.json'); writeJson(caseFile, declaration);
    const result = {name: declaration.name, directory, status: 'running', errors: [], caseSha256Before: digest(caseFile)};
    report.cases.push(result); save('report.json', report);
    let server, browser;
    const original = reuse ? join(resolve(reuse), declaration.name, 'original') : join(directory, 'original');
    try {
      assert.deepEqual(pins(sourceFiles), report.sourcePinsBefore, 'Source changed before case');
      assert.deepEqual(pins(resources), report.resourcePinsBefore, 'Source resources changed before case');
      const nativeArgs = [join(ROOT, 'tools/original/common-sku-run.py'), '--case', caseFile,
        '--source-plans', plansPath, '--output', original, prepareOnly ? '--prepare-only' : '--capture'];
      if (prepareOnly) result.original = await child('python3', nativeArgs, join(directory, 'original.log'));
      else {
        //The registered runner owns this suite's host slot, as for other drivers.
        //Acquiring another here deadlocks parallel runs on a saturated host.
        server = await startProductionServer({directory: flag('--production-directory')});
        result.production = server.build;
        const launch = browserLaunchOptions();
        browser = await chromium.launch({...launch, headless: false, args: [...launch.args, '--no-sandbox']});
        result.browserGraphics = await inspectBrowserGraphics(browser);
        const captures = await Promise.allSettled([
          reuse ? Promise.resolve({exitCode: 0, reused: true, directory: original,
            scope: 'Canonical retained Original input/clock/resource/provenance validation remains blocking in readback'})
            : child('tools/original/with-display.sh', ['python3', ...nativeArgs], join(directory, 'original.log')),
          captureComposition(browser, server, declaration, join(directory, 'browser')),
        ]);
        for (let index = 0; index < captures.length; index++) {
          const captured = captures[index];
          result[index === 0 ? 'original' : 'browserCapture'] = captured.status === 'fulfilled'
            ? captured.value : {status: 'error', error: String(captured.reason)};
        }
      }
    } catch (error) {result.errors.push(String(error));}
    finally {
      for (const [name, resource] of [['browser', browser], ['server', server]]) {
        if (resource) try {await resource.close();} catch (error) {result.errors.push(name + ' close: ' + String(error));}
      }
    }
    if (!prepareOnly) {
      for (const [stage, script] of [['readback', 'composition-readback.py'], ['comparison', 'common-sku-state-pair.py']]) {
        try {
          result[stage] = await child('python3', [join(ROOT, 'tools/foundation', script), '--case', caseFile,
            '--directory', directory, ...(reuse ? ['--original-directory', original] : []),
            '--output', join(directory, stage + '.json')], join(directory, stage + '.log'));
        } catch (error) {result[stage] = {status: 'error', error: String(error)};}
      }
    }
    result.caseSha256After = digest(caseFile);
    if (result.caseSha256After !== result.caseSha256Before) result.errors.push('Declared case changed during capture/grading');
    result.status = prepareOnly ? result.original?.exitCode === 0 && !result.errors.length ? 'prepared-not-executed' : 'fail'
      : result.original?.exitCode === 0 && result.browserCapture?.status === 'pass'
        && result.readback?.exitCode === 0 && result.comparison?.exitCode === 0 && !result.errors.length ? 'pass' : 'fail';
    writeJson(join(directory, 'case-report.json'), result); save('report.json', report);
  }
  report.allDeclaredCasesRetained = JSON.stringify(report.declaredOrder) === JSON.stringify(report.cases.map(row => row.name));
  const expected = prepareOnly ? 'prepared-not-executed' : 'pass';
  report.status = report.allDeclaredCasesRetained && report.cases.every(row => row.status === expected) ? expected : 'fail';
} catch (error) {report.error = String(error); report.status = 'fail';}
finally {
  if (report.sourcePinsBefore) {
    try {
      report.sourcePinsAfter = pins(sourceFiles);
      report.sourceBeforeEqualsEnd = JSON.stringify(report.sourcePinsBefore) === JSON.stringify(report.sourcePinsAfter);
      assert.ok(report.sourceBeforeEqualsEnd, 'Source changed during attempt');
      if (report.replayInput) assert.equal(digest(report.replayInput.path), report.replayInput.sha256, 'Recorded public bank changed during attempt');
      if (report.resourcePinsBefore) {
        report.resourcePinsAfter = pins(Object.keys(report.resourcePinsBefore));
        assert.deepEqual(report.resourcePinsAfter, report.resourcePinsBefore, 'Resource input changed during attempt');
      }
    } catch (error) {report.endPinError = String(error); report.status = 'fail';}
  }
  save('report.json', report);
}
console.log(JSON.stringify({status: report.status, cases: report.cases.length, output}));
process.exitCode = ['pass', 'prepared-not-executed'].includes(report.status) ? 0 : 1;
