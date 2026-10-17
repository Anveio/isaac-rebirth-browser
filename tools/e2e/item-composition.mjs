// Public Challenge resources and recorded key tapes. Native outputs never enter WASM.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { mkdir, open, readFile, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { chromium } from 'playwright';
import { ROOT, flag, hasFlag, outputDirectory, selectCases } from './lib/cli.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { captureComposition } from '../foundation/composition-capture.mjs';

const pocketMode = flag('--pockets');
const pillUseMode = hasFlag('--pill-use');
assert.ok(!pillUseMode || pocketMode === 'collection',
  'Positive Pill use requires the public collection mode');
const wasmNode = flag('--wasm-node');
const wasmSha = flag('--wasm-sha');
const reusedOriginal = flag('--reuse-original');
if (wasmNode) {
  assert.ok(pocketMode, 'Explicit WASM Node fallback is scoped to pocket cases');
  assert.match(wasmSha ?? '', /^[a-f0-9]{64}$/);
} else assert.ok(!wasmSha && !reusedOriginal, 'WASM/reuse declarations need --wasm-node');
if (pocketMode) assert.ok(['init', 'collection', 'all'].includes(pocketMode), 'Unknown pocket mode');
const recordedBankDirectory = pocketMode ? 'references/isaac/foundation/pockets/history'
  : 'references/isaac/foundation/item-composition/history';
if (hasFlag('--help')) {
  console.log('item-composition [--pockets all|init|collection] [--pill-use] [--count 2 | --scenarios BANK] [--production DIST | --wasm-node WASM --wasm-sha SHA [--reuse-original DIR]] [--prepare-only] [--output DIR]');
  console.log('Fixed and recorded OS cases; exact --scenarios replay never draws entropy. Preparation runs neither game.');
  console.log(pillUseMode ? 'Replay with --scenarios PATH from the recorded Pill-use run'
    : `Recorded replay: --scenarios ${recordedBankDirectory}/scenarios.json`);
  process.exit(0);
}
const output = outputDirectory(pocketMode ? 'pockets-' + pocketMode : 'item-composition');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const save = (path, value) => writeFile(path, JSON.stringify(value, null, 2) + '\n');
const sourceFiles = [
  'tools/e2e/item-composition.mjs', 'tools/e2e/lib/original.mjs',
  'tools/original/composition-scenarios.py', 'tools/original/composition-run.py',
  'tools/original/composition-observe.py', 'tools/original/foundation-clock-run.py',
  'tools/original/native-evidence.py', 'tools/foundation/native-evidence.mjs',
  'tools/foundation/composition-capture.mjs', 'tools/foundation/composition-progress.mjs',
  'tools/foundation/composition-readback.py', 'tools/foundation/composition-state-pair.py',
  'tools/foundation/composition-history.py', 'tools/original/item-observer.py',
  'tools/original/native-identity.py', 'tools/original/foundation-pickup-observe.py',
  'tools/original/foundation-combat-observe.py', 'tools/original/debug-run.py',
  'tools/original/foundation-clock-gate.c', 'tools/original/foundation-seeds.py',
  'tools/foundation/decode-rooms.py',
  'tools/foundation/browser-evidence.mjs', 'tools/foundation/production-server.mjs',
  'tools/foundation/browser.mjs',
];
if (pocketMode) sourceFiles.push('tools/original/pocket-scenarios.py',
  'tools/original/pocket-collection-observe.py', 'tools/original/audio-observer.py',
  'tools/foundation/pocket-state-pair.py', 'tools/original/native-journal-zstd.py',
  'tools/original/state-observer.py', 'references/isaac/foundation/pockets/history/scenarios.json');
const filePins = async paths => Object.fromEntries(await Promise.all(paths.map(async path => [path, hash(await readFile(path))])));
if (pillUseMode) sourceFiles.push('tools/foundation/pill-range-use-state-pair.py',
  'tools/original/pill-range-use-scenarios.py',
  'references/isaac/rebirth/resources/rooms/00.special rooms.stb',
  'references/isaac/rebirth/resources/challenges.xml');
if (wasmNode) sourceFiles.push('tools/foundation/composition-wasm-public.mjs');
const sourcePins = () => filePins(sourceFiles.map(path => join(ROOT, path)));

async function child(command, args, log) {
  const logFile = await open(log, 'wx');
  try {
    const process = spawn(command, args, {
      cwd: ROOT, env: { ...globalThis.process.env, LP_NUM_THREADS: '2', ISAAC_ORIGINAL_SHA: ORIGINAL_SHA },
      stdio: ['ignore', logFile.fd, logFile.fd],
    });
    return await new Promise((done, reject) => {
      process.once('error', reject);
      process.once('close', (exitCode, signal) => done({ exitCode, signal }));
    });
  } finally { await logFile.close(); }
}

const report = {
  schema: 1, status: 'starting', startedUtc: new Date().toISOString(), cases: [],
  scope: 'Fresh original and immutable production snapshot receive independently declared resources and public inputs. Capture validity and functional differences are separate. PNGs are retained; full-frame pixel parity is a separate diagnostic.',
};
// Claim this attempt before any preparation; repeated --output never overwrites a prior report.
await writeFile(join(output, 'report.json'), JSON.stringify(report, null, 2) + '\n', { flag: 'wx' });
let browser, server, bankPath, bankBytes, resources;
try {
  await assertOriginalBinary();
  report.sourceBefore = await sourcePins();
  const replay = flag('--scenarios');
  if (replay) {
    bankPath = resolve(replay); // Exact replay never calls the entropy producer.
    report.inputMode = 'recorded bank; no new entropy';
  } else {
    const generated = join(output, 'declared-inputs');
    const result = await child('python3', [join(ROOT, pillUseMode ? 'tools/original/pill-range-use-scenarios.py' : pocketMode
      ? 'tools/original/pocket-scenarios.py' : 'tools/original/composition-scenarios.py'),
      '--output', generated, '--count', flag('--count', '2'),
      ...(pocketMode && !pillUseMode ? ['--mode', pocketMode] : [])], join(output, 'declaration.log'));
    assert.equal(result.exitCode, 0, 'Case declaration failed; its log and any recorded entropy remain retained');
    bankPath = join(generated, 'scenarios.json');
    report.inputMode = 'fixed cases plus every freshly journaled unfiltered OS projection';
  }
  bankBytes = await readFile(bankPath);
  const bank = JSON.parse(bankBytes);
  assert.equal(bank.schema, 1);
  assert.ok(Array.isArray(bank.cases) && bank.cases.length > 0);
  assert.equal(new Set(bank.cases.map(item => item.name)).size, bank.cases.length);
  let cases = bank.cases.map(declaration => {
    assert.match(declaration.name, /^[A-Za-z0-9_-]{1,80}$/);
    const fixture = { ...declaration.fixture };
    for (const key of ['authoredResourcePath', 'specialRoomsPath']) {
      if (fixture[key]) fixture[key] = resolve(dirname(bankPath), fixture[key]);
    }
    if (pocketMode) {
      const kind = declaration.pocketInit ? 'init' : declaration.pocketCollection ? 'collection' : null;
      assert.ok(kind && (pocketMode === 'all' || pocketMode === kind), 'Every declared replay case must match the requested pocket mode');
      assert.equal(Number.isFinite(declaration.sfxVolume), true);
      assert.ok(declaration.sfxVolume >= 0 && declaration.sfxVolume <= 1);
    }
    assert.equal(Boolean(declaration.pillUse), pillUseMode,
      'Real consumption requires its separate use reader');
    if (pillUseMode) assert.equal(declaration.pillUse.schema, 'source-public-pill-range-use-v1');
    return { ...declaration, fixture };
  });
  if (cases.some(item => item.requires?.spoonBender3Functional !== undefined)) {
    for (const item of cases.filter(value => value.requires?.spoonBender3Functional !== undefined)) {
      const spoon = item.requires.spoonBender3Functional;
      const contract = item.requires.sourceExpectedActorContract;
      assert.equal(spoon.schema, 'ordinary-spoon-bender3-registered-homing-v1');
      assert.equal(spoon.marker, 'spoonBender3Tears');
      assert.deepEqual([contract.primaryC08, contract.family, contract.variant, contract.subtype,
        contract.bornAttackFlags, contract.controlledBy329], [1, 2, 0, 0, '4', false]);
      assert.ok(item.requires.compositionSourceFields.includes('spoonBender3Tears'));
      assert.ok(item.requires.actorStages.every(stage => Array.isArray(stage.requiredWitnesses)
        && stage.requiredWitnesses.every(value => typeof value === 'string')));
    }
    sourceFiles.push('tools/foundation/composition-spoon-bender3.py');
    report.sourceBefore = await sourcePins();
  }
  if (cases.some(item => item.requires?.controlledKnifeSatellites !== undefined)) {
    sourceFiles.push('tools/foundation/composition-controlled-knife-satellites.py');
    report.sourceBefore = await sourcePins();
  }
  cases = selectCases(cases, item => item.name);
  report.caseFilter = flag('--case') ?? bank.caseFilter;
  const scenarios = join(output, 'scenarios.json');
  await save(scenarios, { ...bank, caseFilter: report.caseFilter, cases });
  report.recordedBank = { path: bankPath, sha256: hash(bankBytes), replayPath: scenarios };
  resources = [...new Set(cases.flatMap(item => [item.fixture.authoredResourcePath, item.fixture.specialRoomsPath].filter(Boolean)))];
  report.resourcePinsBefore = await filePins(resources);
  report.declaredOrder = cases.map(item => item.name);
  await save(join(output, 'report.json'), report);
  if (wasmNode) {
    assert.equal(hash(await readFile(resolve(wasmNode))), wasmSha, 'Actual qualified WASM bytes');
    assert.ok(!reusedOriginal || cases.length === 1, 'A closed Original reuse names exactly one identical case');
    report.captureBackend = 'wasm-node';
    report.wasmArtifact = {path:resolve(wasmNode),sha256:wasmSha};
    report.unavailable = ['Browser DOM inputs', 'Renderer and PNGs', 'Audio backend and PCM'];
    report.scope = 'Independent Original evidence and explicit qualified public-WASM ABI replay. Actual game fields remain assertions; DOM/render/backend evidence is unavailable.';
  }
  if (!hasFlag('--prepare-only') && !wasmNode) {
    const qualifiedSha = flag('--qualified-production-sha');
    const qualifiedFiles = flag('--qualified-production-files');
    assert.equal(Boolean(qualifiedSha), Boolean(qualifiedFiles), 'Both qualified production declarations required');
    server = await startProductionServer({ directory: resolve(flag('--production', join(ROOT, 'dist'))),
      ...(qualifiedSha ? {qualifiedBuildIdentity:{sha256:qualifiedSha,files:Number(qualifiedFiles)}} : {}) });
    if (qualifiedSha) assert.deepEqual({sha256:server.build.sha256,files:server.build.files},
      {sha256:qualifiedSha,files:Number(qualifiedFiles)}, 'Actual production identity matches declaration');
    report.production = server.build;
    const launch = browserLaunchOptions();
    browser = await chromium.launch({ ...launch, headless: false, args: [...launch.args, '--no-sandbox'] });
    report.browserGraphics = await inspectBrowserGraphics(browser);
  }
  for (const declaration of cases) {
    const directory = join(output, declaration.name);
    await mkdir(directory); // A prior attempt is never overwritten.
    await save(join(directory, 'case.json'), declaration);
    const result = { name: declaration.name, directory, status: 'running', errors: [] };
    report.cases.push(result);
    await save(join(output, 'report.json'), report);
    console.log(JSON.stringify({ case: declaration.name, status: 'started' }));
    const originalDirectory = reusedOriginal ? resolve(reusedOriginal) : join(directory, 'original');
    try {
      if (reusedOriginal) {
        assert.deepEqual(JSON.parse(await readFile(join(originalDirectory,'case.json'),'utf8')),declaration,
          'Reused independent Original must have the exact same declaration');
        result.original = {exitCode:0,scope:'Reused closed independent Original capture; no new Native process.',directory:originalDirectory};
      } else {
      const args = [join(ROOT, 'tools/original/composition-run.py'), '--scenarios', scenarios,
        '--case', declaration.name, '--output', join(directory, 'original')];
      if (pocketMode) args.push('--pockets', declaration.pocketInit ? 'init' : 'collection');
      if (hasFlag('--prepare-only')) args.push('--prepare-only');
      else args.push('--archive-evidence', ...(pocketMode ? ['--journal-codec', 'zstd'] : []), '--observe-every-step', '--state-observer',
        join(ROOT, pocketMode ? 'tools/original/pocket-collection-observe.py'
          : 'tools/original/composition-observe.py'), '--frame-timeout', flag('--native-frame-timeout', '60'));
      result.original = await child('python3', args, join(directory, 'original.log'));
      }
    } catch (error) { result.errors.push('Original: ' + String(error)); }
    // Every declared browser case is attempted even when its native capture fails.
    if (!hasFlag('--prepare-only')) {
      const portDirectory = join(directory,wasmNode?'wasm':'browser');
      try {
        if (wasmNode) {
          await mkdir(portDirectory);
          const node = await child(process.execPath,[join(ROOT,'tools/foundation/composition-wasm-public.mjs'),
            portDirectory,resolve(wasmNode),wasmSha,join(directory,'case.json'),hash(await readFile(join(directory,'case.json')))],
            join(directory,'wasm.log'));
          result.wasmCapture = node;
        } else result.browserCapture = await captureComposition(browser, server, declaration, portDirectory);
      } catch (error) { result.errors.push((wasmNode?'WASM: ':'Browser: ') + String(error)); }
      for (const [key, tool, args] of [
        ['captureReadback', 'composition-readback.py', ['--directory', directory, ...(wasmNode?['--wasm-node','--wasm-artifact',resolve(wasmNode),'--wasm-sha',wasmSha]:[]), ...(reusedOriginal?['--original-directory',originalDirectory]:[])]],
        ['comparison', pillUseMode ? 'pill-range-use-state-pair.py' : pocketMode ? 'pocket-state-pair.py' : 'composition-state-pair.py', ['--original', join(originalDirectory, 'manifest.json.gz'),
          '--browser', join(portDirectory,wasmNode?'actual-replay-result.json':'browser-evidence.json.gz'),
          ...(pillUseMode?['--capture-readback',join(directory,'capture-readback.json')]:[]), ...(wasmNode?['--wasm-node']:[])]],
      ]) {
        try {
          result[key] = await child('python3', [join(ROOT, 'tools/foundation', tool),
            '--case', join(directory, 'case.json'), ...args, '--output',
            join(directory, key === 'comparison' ? 'comparison.json.gz' : 'capture-readback.json')], join(directory, key + '.log'));
        } catch (error) { result.errors.push(key + ': ' + String(error)); }
      }
    }
    result.status = hasFlag('--prepare-only')
      ? result.original?.exitCode === 0 && !result.errors.length ? 'prepared-not-executed' : 'fail'
      : result.original?.exitCode === 0 && (wasmNode ? result.wasmCapture?.exitCode === 0 : result.browserCapture?.status === 'pass')
        && result.captureReadback?.exitCode === 0 && result.comparison?.exitCode === 0 && !result.errors.length ? 'pass' : 'fail';
    await save(join(directory, 'case-report.json'), result);
    await save(join(output, 'report.json'), report);
    console.log(JSON.stringify({ case: declaration.name, status: result.status }));
  }
  report.allDeclaredCasesRetained = JSON.stringify(report.cases.map(item => item.name)) === JSON.stringify(report.declaredOrder);
  const expectedStatus = hasFlag('--prepare-only') ? 'prepared-not-executed' : 'pass';
  report.status = report.allDeclaredCasesRetained
    && report.cases.every(item => item.status === expectedStatus) ? expectedStatus : 'fail';
} catch (error) { report.error = String(error); report.status = 'fail'; }
finally {
  for (const [name, resource] of [['browser', browser], ['server', server]]) {
    if (resource) try { await resource.close(); }
    catch (error) { report[name + 'CloseError'] = String(error); report.status = 'fail'; }
  }
  if (report.sourceBefore) {
    try {
      report.sourceAfter = await sourcePins();
      report.sourceBeforeEqualsEnd = JSON.stringify(report.sourceBefore) === JSON.stringify(report.sourceAfter);
      assert.ok(report.sourceBeforeEqualsEnd, 'Source changed during the attempt');
      if (report.resourcePinsBefore) {
        report.resourcePinsAfter = await filePins(resources);
        report.protectedBeforeEqualsEnd = report.sourceBeforeEqualsEnd
          && JSON.stringify(report.resourcePinsBefore) === JSON.stringify(report.resourcePinsAfter)
          && hash(await readFile(bankPath)) === hash(bankBytes);
        assert.ok(report.protectedBeforeEqualsEnd, 'Bank or resources changed during the attempt');
      }
      await assertOriginalBinary();
    } catch (error) { report.endPinError = String(error); report.status = 'fail'; }
  }
  report.closedUtc = new Date().toISOString();
  await save(join(output, 'report.json'), report);
}
console.log(JSON.stringify({ status: report.status, cases: report.cases.length, output }));
process.exitCode = ['pass', 'prepared-not-executed'].includes(report.status) ? 0 : 1;
