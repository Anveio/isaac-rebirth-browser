// Public Challenge resources and recorded key tapes. Native outputs never enter WASM.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { mkdir, open, readFile, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { chromium } from 'playwright';
import { ROOT, flag, hasFlag, outputDirectory } from './lib/cli.mjs';
import { ORIGINAL_SHA, assertOriginalBinary } from './lib/original.mjs';
import { startProductionServer } from '../foundation/production-server.mjs';
import { browserLaunchOptions, inspectBrowserGraphics } from '../foundation/browser.mjs';
import { captureComposition } from '../foundation/composition-capture.mjs';

const recordedBankDirectory = 'references/isaac/foundation/item-composition/history';
if (hasFlag('--help')) {
  console.log('item-composition [--count 2 | --scenarios BANK] [--production DIST] [--prepare-only] [--output DIR]');
  console.log('Fixed and recorded OS cases; exact --scenarios replay never draws entropy. Preparation runs neither game.');
  console.log(`Recorded replay: --scenarios ${recordedBankDirectory}/scenarios.json`);
  process.exit(0);
}
const output = outputDirectory('item-composition');
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
const filePins = async paths => Object.fromEntries(await Promise.all(paths.map(async path => [path, hash(await readFile(path))])));
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
    const result = await child('python3', [join(ROOT, 'tools/original/composition-scenarios.py'),
      '--output', generated, '--count', flag('--count', '2')], join(output, 'declaration.log'));
    assert.equal(result.exitCode, 0, 'Case declaration failed; its log and any recorded entropy remain retained');
    bankPath = join(generated, 'scenarios.json');
    report.inputMode = 'fixed cases plus every freshly journaled unfiltered OS projection';
  }
  bankBytes = await readFile(bankPath);
  const bank = JSON.parse(bankBytes);
  assert.equal(bank.schema, 1);
  assert.ok(Array.isArray(bank.cases) && bank.cases.length > 0);
  assert.equal(new Set(bank.cases.map(item => item.name)).size, bank.cases.length);
  const cases = bank.cases.map(declaration => {
    assert.match(declaration.name, /^[A-Za-z0-9_-]{1,80}$/);
    const fixture = { ...declaration.fixture };
    for (const key of ['authoredResourcePath', 'specialRoomsPath']) {
      if (fixture[key]) fixture[key] = resolve(dirname(bankPath), fixture[key]);
    }
    return { ...declaration, fixture };
  });
  const scenarios = join(output, 'scenarios.json');
  await save(scenarios, { ...bank, cases });
  report.recordedBank = { path: bankPath, sha256: hash(bankBytes), replayPath: scenarios };
  resources = [...new Set(cases.flatMap(item => [item.fixture.authoredResourcePath, item.fixture.specialRoomsPath].filter(Boolean)))];
  report.resourcePinsBefore = await filePins(resources);
  report.declaredOrder = cases.map(item => item.name);
  await save(join(output, 'report.json'), report);
  if (!hasFlag('--prepare-only')) {
    server = await startProductionServer({ directory: resolve(flag('--production', join(ROOT, 'dist'))) });
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
    try {
      const args = [join(ROOT, 'tools/original/composition-run.py'), '--scenarios', scenarios,
        '--case', declaration.name, '--output', join(directory, 'original')];
      if (hasFlag('--prepare-only')) args.push('--prepare-only');
      else args.push('--archive-evidence', '--observe-every-step', '--state-observer',
        join(ROOT, 'tools/original/composition-observe.py'), '--frame-timeout', flag('--native-frame-timeout', '60'));
      result.original = await child('python3', args, join(directory, 'original.log'));
    } catch (error) { result.errors.push('Original: ' + String(error)); }
    // Every declared browser case is attempted even when its native capture fails.
    if (!hasFlag('--prepare-only')) {
      try { result.browserCapture = await captureComposition(browser, server, declaration, join(directory, 'browser')); }
      catch (error) { result.errors.push('Browser: ' + String(error)); }
      for (const [key, tool, args] of [
        ['captureReadback', 'composition-readback.py', ['--directory', directory]],
        ['comparison', 'composition-state-pair.py', ['--original', join(directory, 'original/manifest.json.gz'),
          '--browser', join(directory, 'browser/browser-evidence.json.gz')]],
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
      : result.original?.exitCode === 0 && result.browserCapture?.status === 'pass'
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
