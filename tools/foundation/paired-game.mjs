import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { chromium } from '@playwright/test';
import { PNG } from 'pngjs';
import { browserLaunchOptions, inspectBrowserGraphics } from './browser.mjs';
import { pixelDifference } from './compare.mjs';
import { startProductionServer } from './production-server.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const SOURCE = 'tools/original/foundation-paired-scenarios.py';
export const DEFAULT_PAIRED_SCENARIOS = ['paired-title-menu', 'paired-movement-fire', 'paired-pause-release', 'paired-bomb-moving-even', 'paired-bomb-moving-odd', 'paired-room-combat'];
export const DEFAULT_RANDOM_CASE_COUNT = 2;
const runProcess = promisify(execFile);
const save = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const GATE_SOURCES = [
  'tools/foundation/paired-game.mjs', 'tools/foundation/production-server.mjs',
  'tools/foundation/compare.mjs', 'tools/original/foundation-paired-scenarios.py',
  'tools/original/foundation-paired-observer.py', 'tools/original/foundation-menu-observer.py',
  'tools/original/foundation-combat-observe.py', 'tools/original/foundation-clock-run.py',
  'tools/original/foundation-clock-gate.c', 'tools/original/foundation-clock-verify.py',
  'tools/original/foundation-clock-scenarios.py', 'tools/original/foundation-bomb-scenario.py',
  'tools/original/foundation-presentation-scenario.py', 'tools/original/foundation-title-scenario.py',
  'tools/original/foundation-combat-scenario.py', 'tools/original/foundation-seeds.py',
  'tools/original/foundation-random-seeds.py', 'tools/original/state-observer.py',
  'tools/original/debug-run.py', 'tools/original/with-display.sh', 'tools/foundation/browser.mjs',
];
const sourceHashes = () => Object.fromEntries(GATE_SOURCES.map(path => [path, digest(readFileSync(join(ROOT, path)))]));
const option = (name, fallback) => {
  const at = process.argv.indexOf(name);
  if (at < 0) return fallback;
  assert.ok(process.argv[at + 1] && !process.argv[at + 1].startsWith('--'), `Missing ${name} value`);
  return process.argv[at + 1];
};
const countOption = (name, fallback, minimum = 1) => {
  const value = Number(option(name, fallback));
  assert.ok(Number.isInteger(value) && value >= minimum && value <= 32, `${name} must be ${minimum}..32`);
  return value;
};
async function python(args) {
  const { stdout } = await runProcess('python3', [SOURCE, ...args], { cwd: ROOT, maxBuffer: 8 * 1024 * 1024 });
  return JSON.parse(stdout);
}

export async function pairedScenarioDeclarations() {
  return python(['--list']);
}

const namedKeys = { Return: 'Enter', Escape: 'Escape', Tab: 'Tab', BackSpace: 'Backspace',
  Up: 'ArrowUp', Down: 'ArrowDown', Left: 'ArrowLeft', Right: 'ArrowRight' };
function keyCode(key) {
  if (namedKeys[key]) return namedKeys[key];
  if (/^[a-z]$/i.test(key)) return 'Key' + key.toUpperCase();
  if (/^[0-9]$/.test(key)) return 'Digit' + key;
  throw new Error(`No public browser key transport for ${key}`);
}

async function nativeCapture(caseData, directory) {
  mkdirSync(directory, { recursive: true });
  const args = ['tools/original/with-display.sh', 'python3', 'tools/original/foundation-clock-run.py',
    '--seed', caseData.seed, '--tape', join(dirname(directory), 'tape.json'),
    '--state-observer', 'tools/original/foundation-paired-observer.py', '--output', directory];
  let error;
  try {
    const result = await runProcess('bash', args, { cwd: ROOT, maxBuffer: 8 * 1024 * 1024 });
    writeFileSync(join(directory, 'controller-output.txt'), result.stdout + result.stderr);
  } catch (failure) {
    error = String(failure.message);
    writeFileSync(join(directory, 'controller-output.txt'), String(failure.stdout || '') + String(failure.stderr || '') + error);
  }
  let manifest;
  try { manifest = JSON.parse(readFileSync(join(directory, 'manifest.json'), 'utf8')); }
  catch (failure) { return { status: 'fail', error: error || String(failure.message), directory }; }
  let provenance;
  try { provenance = await python(['--validate-original', join(directory, 'manifest.json')]); }
  catch (failure) {
    try { provenance = JSON.parse(failure.stdout); }
    catch { provenance = { status: 'fail', failures: [String(failure.message)] }; }
  }
  return { status: error || provenance.status !== 'pass' ? 'fail' : 'pass', error, provenance, directory, manifest };
}

async function browserCapture(browser, server, caseData, directory) {
  mkdirSync(directory, { recursive: true });
  const context = await browser.newContext({ viewport: { width: 960, height: 540 } });
  const page = await context.newPage();
  const frames = [], inputs = [], expectedCommands = [];
  const failures = [];
  try {
    await page.goto(server.url + '/?manual=1');
    await page.waitForFunction(() => ['true', 'failed'].includes(document.body.dataset.ready));
    assert.equal(await page.evaluate(() => document.body.dataset.ready), 'true', 'Production browser must boot');
    await page.evaluate(({ initialSeed, bootEpochSeed }) => {
      window.game.pause(); window.game.reset(initialSeed, { bootEpochSeed, trace: true });
    }, caseData);
    let presentations = 0;
    for (const action of caseData.pairedTape) {
      if (action.type === 'step') {
        await page.evaluate(count => window.game.step(count), action.frames);
        presentations += action.frames;
        expectedCommands.push({ type: 'tick', count: action.frames });
      } else if (action.type === 'key') {
        const before = await page.evaluate(() => window.game.snapshot());
        const code = keyCode(action.key);
        const text = before.screen === 'seed' && /^[a-z0-9]$/i.test(action.key);
        await page.keyboard[action.down ? 'down' : 'up'](namedKeys[action.key] || action.key);
        if (text) {
          if (action.down) expectedCommands.push({ type: 'text', text: action.key.toUpperCase() });
          else if (before.pressed.includes(code)) expectedCommands.push({ type: 'key', code, down: false });
        } else expectedCommands.push({ type: 'key', code, down: action.down });
        inputs.push({ ...action, normalizedCode: code, browserPresentation: presentations });
      } else {
        const captured = await page.evaluate(() => window.game.capture());
        const bytes = Buffer.from(captured.png, 'base64');
        const path = join(directory, action.label + '.png');
        writeFileSync(path, bytes);
        frames.push({ label: action.label, path, sha256: digest(bytes), browserPresentation: presentations, state: captured.state });
      }
    }
    const trace = await page.evaluate(() => window.game.trace());
    save(join(directory, 'trace.json'), trace);
    assert.equal(trace.enabled, true); assert.equal(trace.inputTraceTruncated, false);
    assert.equal(trace.bootEpochSeed, caseData.bootEpochSeed);
    assert.equal(trace.initialSeed, caseData.initialSeed);
    assert.deepEqual(trace.commands, expectedCommands, 'Every intended DOM input/interval must reach the actual WASM trace');
    const final = frames.at(-1)?.state;
    assert.equal(final?.runSeedValue, caseData.seedValue, 'Browser must read back the public manually entered run seed');
  } catch (error) { failures.push(String(error.message)); }
  finally { await context.close(); }
  const result = { status: failures.length ? 'fail' : 'pass', failures, frames, inputs };
  save(join(directory, 'manifest.json'), result);
  return result;
}

function scopedState(source, actual, gameplay) {
  assert.equal(source?.status, 'pass', 'Every paired capture requires a successful typed source observation');
  assert.equal(source.memoryWritten, false, 'Source observations must remain read-only');
  const checks = [];
  const check = (field, expected, observed) => checks.push({ field, expected, actual: observed, status: JSON.stringify(expected) === JSON.stringify(observed) ? 'pass' : 'fail' });
  if (!gameplay) {
    assert.equal(source.menu?.status, 'pass', 'Pre-run captures require typed source menu evidence');
    assert.equal(source.menu.menuInitialized, true, 'Pre-run captures require an initialized source menu');
    assert.ok([1, 2, 3, 4].includes(source.menu.menuScreen), 'Pre-run captures require a known source menu family');
    // Numeric menu families are corroborated by this gate's fresh native
    // title/profile/main/character labels; the seed dialog belongs to family4.
    const menuScreens = { 1: 'title', 2: 'profile', 3: 'main' };
    if (menuScreens[source.menu.menuScreen]) check('screen', menuScreens[source.menu.menuScreen], actual.screen);
    else if (source.menu.menuScreen === 4) check('characterMenuFamily', true, ['character', 'seed'].includes(actual.screen));
    check('titleAnimationFrame', source.menu.titleAnimation.frame, actual.titleAnimationFrame);
    check('menuFade.active', source.menu.menuFade.active, actual.menuFade.active);
    check('menuFade.alpha', source.menu.menuFade.alpha, actual.menuFade.alpha);
    check('menuFade.color', source.menu.menuFade.color, actual.menuFade.color);
  } else {
    assert.equal(source.gameplay?.status, 'pass', 'Gameplay captures require typed source gameplay evidence');
    assert.ok(Array.isArray(source.gameplay.players) && source.gameplay.players.length > 0, 'Gameplay captures require an observed source player');
    assert.ok(Array.isArray(source.gameplay.entities), 'Gameplay captures require observed source entities');
    assert.equal(typeof source.pause?.paused, 'boolean', 'Gameplay captures require typed source pause evidence');
    const player = source.gameplay.players[0];
    check('screen', 'playing', actual.screen);
    // The port's public player coordinates use the observed affine room
    // projection; source memory supplies world f32 values independently.
    check('player.x', player.position[0] * 0.65 + 32, actual.player.x);
    check('player.y', player.position[1] * 0.65 - 46, actual.player.y);
    check('player.vx', player.velocity[0] * 0.65, actual.player.vx);
    check('player.vy', player.velocity[1] * 0.65, actual.player.vy);
    check('player.hp', player.redHp, actual.player.hp);
    check('damageCooldown', player.damageCountdown, actual.damageCooldown);
    const admitted = source.gameplay.entities.filter(entity => !entity.removed);
    const activeTears = actual.tears.filter(tear => tear.motion === undefined || tear.motion.lifetime === 'alive');
    check('tears.alive', admitted.filter(entity => entity.type === 2).length, activeTears.length);
    const livingBombs = actual.bombs.filter(bomb => bomb.motion === undefined || bomb.motion.lifetime === 'alive');
    check('bombs.alive', admitted.filter(entity => entity.type === 4).length, livingBombs.length);
    check('paused', source.pause.paused, actual.paused);
    if (source.pause.paused) {
      check('pauseMenu.selection', source.pause.selection, actual.pauseMenu.selection);
      const phases = { Appear: 'appearing', Idle: 'idle', Dissapear: 'disappearing' };
      if (phases[source.pause.animation]) check('pauseMenu.phase', phases[source.pause.animation], actual.pauseMenu.phase);
    }
  }
  assert.ok(checks.length > 0, 'Every paired capture must compare typed functional fields');
  return checks;
}

function comparePair(caseData, native, browser, directory, exactPixels) {
  mkdirSync(directory, { recursive: true });
  const checks = [];
  const originals = new Map((native.manifest?.frames || []).map(frame => [frame.label, frame]));
  const observations = new Map((native.manifest?.debugObservations || []).map(row => [row.context.label, row.observation]));
  const candidates = new Map(browser.frames.map(frame => [frame.label, frame]));
  const labels = caseData.pairedTape.filter(action => action.type === 'capture').map(action => action.label);
  const gameplayStart = labels.indexOf('gameplay-start');
  assert.ok(gameplayStart >= 0, 'Selected paired tape must declare its gameplay boundary');
  const first = originals.get(caseData.bootstrap.boundaryLabel);
  for (const [index, label] of labels.entries()) {
    const check = { label, status: 'fail' };
    try {
      const original = originals.get(label), candidate = candidates.get(label);
      assert.ok(original && candidate, `Missing actual paired checkpoint ${label}`);
      assert.equal(original.presentedFrame - first.presentedFrame, candidate.browserPresentation, 'Both games must receive identical paired presentation intervals');
      const sourceBytes = readFileSync(resolve(dirname(join(native.directory, 'manifest.json')), original.path));
      const difference = pixelDifference(PNG.sync.read(sourceBytes), PNG.sync.read(readFileSync(candidate.path)));
      writeFileSync(join(directory, label + '-difference.png'), PNG.sync.write(difference.difference));
      const { difference: ignoredImage, ...pixels } = difference;
      check.pixels = { ...pixels, status: pixels.exactRGBA ? 'pass' : 'fail', required: exactPixels };
      check.sourcePNGHash = digest(sourceBytes);
      check.state = scopedState(observations.get(label), candidate.state, index >= gameplayStart);
      check.status = (!exactPixels || pixels.exactRGBA) && check.state.every(field => field.status === 'pass') ? 'pass' : 'fail';
    } catch (error) { check.error = String(error.message); }
    checks.push(check);
  }
  assert.ok(checks.length > 0, 'Selected paired scenario must compare actual checkpoints');
  return checks;
}

export async function runPairedGate({ output, selected, randomCount = 2, replaySeeds = [], replayCases = [], exactPixels = false, productionDirectory }) {
  mkdirSync(output, { recursive: true });
  const declarations = await pairedScenarioDeclarations();
  const known = new Set(declarations.map(item => item.id));
  for (const id of selected) assert.ok(known.has(id), `Unknown paired scenario ${id}`);
  const server = await startProductionServer({ directory: productionDirectory });
  const browser = await chromium.launch({ ...browserLaunchOptions(), headless: false });
  const report = { schema: 1, status: 'running', mode: 'fresh original process paired with immutable browser production artifact',
    build: server.build, gateSources: sourceHashes(), browserGraphics: await inspectBrowserGraphics(browser),
    sourceBinarySha256: digest(readFileSync(join(ROOT, '.cache/original-runtime/offline-depot/isaac.x64'))),
    comparisons: { functional: 'Real input/phase/readback and source title, fade, player motion, pause/controller, HP, immunity, active tear/bomb counts',
      pixels: 'Strict full RGBA diagnostic; every difference remains a failure in its own pixel result',
      exactPixelsRequired: exactPixels, completeOriginalStateParity: false },
    cases: [] };
  try {
    const definitions = selected.map(id => ({ id, seed: 'T1PK6Y0W' }));
    for (let index = 0; index < randomCount; index++) definitions.push({ id: 'paired-random-run', seed: replaySeeds[index] });
    for (const path of replayCases) definitions.push({ id: JSON.parse(readFileSync(path, 'utf8')).scenario.id, replayCase: path });
    for (const [index, definition] of definitions.entries()) {
      const directory = join(output, `${String(index).padStart(2, '0')}-${definition.id}`);
      const args = ['--scenario', definition.id, '--output', directory];
      if (definition.seed) args.push('--seed', definition.seed);
      let caseData;
      if (definition.replayCase) {
        caseData = JSON.parse(readFileSync(definition.replayCase, 'utf8'));
        mkdirSync(directory, { recursive: true });
        save(join(directory, 'case.json'), caseData); save(join(directory, 'tape.json'), caseData.tape);
      } else caseData = await python(args);
      process.stdout.write(`Paired ${definition.id} seed ${caseData.seed}\n`);
      // Original and Chromium use independent private displays/profiles.
      const [native, candidate] = await Promise.all([
        nativeCapture(caseData, join(directory, 'original')),
        browserCapture(browser, server, caseData, join(directory, 'browser')),
      ]);
      const checks = comparePair(caseData, native, candidate, join(directory, 'comparison'), exactPixels);
      const result = { id: definition.id, seed: caseData.seed, seedValue: caseData.seedValue, bootEpochSeed: caseData.bootEpochSeed,
        caseSha256: digest(readFileSync(join(directory, 'case.json'))),
        directory, original: { status: native.status, error: native.error, provenance: native.provenance },
        browser: { status: candidate.status, failures: candidate.failures }, checks,
        status: native.status === 'pass' && candidate.status === 'pass' && checks.every(check => check.status === 'pass') ? 'pass' : 'fail' };
      report.cases.push(result); save(join(directory, 'report.json'), result);
      process.stdout.write(`${result.status}: ${checks.filter(check => check.status === 'pass').length}/${checks.length} paired checkpoints\n`);
    }
    report.status = report.cases.length > 0 && report.cases.every(item => item.status === 'pass') ? 'pass' : 'fail';
    report.exactOriginalPixels = report.cases.every(item => item.checks.every(check => check.pixels?.exactRGBA));
    report.gateSourcesStable = JSON.stringify(report.gateSources) === JSON.stringify(sourceHashes());
    if (!report.gateSourcesStable) report.status = 'fail';
  } finally {
    save(join(output, 'report.json'), report);
    await browser.close(); await server.close();
  }
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.includes('--list')) console.log(JSON.stringify(await pairedScenarioDeclarations(), null, 2));
  else {
    const replayCases = [...option('--replay-cases', '').split(',').filter(Boolean),
      ...option('--replay-case', '').split(',').filter(Boolean)].map(path => resolve(path));
    const selected = option('--scenarios', replayCases.length ? '' : DEFAULT_PAIRED_SCENARIOS.join(',')).split(',').filter(Boolean);
    const replaySeeds = option('--random-seeds', '').split(',').filter(Boolean);
    const randomCount = countOption('--random-count', replaySeeds.length || (replayCases.length ? 0 : DEFAULT_RANDOM_CASE_COUNT), 0);
    assert.ok(!replaySeeds.length || replaySeeds.length === randomCount, 'Replay seed count must match randomized case count');
    const output = resolve(option('--output', `.cache/foundation/paired-${Date.now()}`));
    const report = await runPairedGate({ output, selected, randomCount, replaySeeds, replayCases,
      exactPixels: process.argv.includes('--require-exact-pixels'), productionDirectory: option('--production-directory') });
    console.log(JSON.stringify({ status: report.status, report: join(output, 'report.json'), artifact: report.build.sha256 }));
    process.exitCode = report.status === 'pass' ? 0 : 1;
  }
}
