// Run registered live e2e suites (tools/e2e/suites.mjs) and keep every result.
//
//   npm run test:e2e                         every suite, one at a time
//   npm run test:e2e -- keys doors           selected suites
//   npm run test:e2e -- @items @audio        logical groups (tools/e2e/suites.mjs GROUPS)
//   npm run test:e2e -- --groups             groups and their suites
//   npm run test:e2e -- --jobs 2             suites in parallel (logs go to files)
//   npm run test:e2e -- keys --count 4       extra flags override the suite's defaults
//   npm run test:e2e -- keys --scenarios .cache/e2e/<run>/keys/scenarios.json
//   npm run test:e2e -- --list               suite ids as JSON (CI matrix)
//   npm run test:e2e -- --keep-evidence      keep every artifact of passing suites too
//   npm run test:e2e -- --keep-runs 5        slim runs older than the newest 5 (default 3)
//   npm run test:e2e -- --affected           suites the changes since origin/main touch
//   npm run test:e2e -- --affected --base REF --list   the selection and why
//
// The production build is rebuilt first when web/ or engine/ sources are newer
// than dist/ (skip with --no-build; the native floor generator is always kept
// fresh). Failed suites print how to replay them.
import { spawn, spawnSync } from 'node:child_process';
import { appendFileSync, createWriteStream, existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { acquireHostSlot } from './lib/host-slots.mjs';
import { freeBytes, pruneRuns, slimSuite, sweepProductionSnapshots } from './lib/retention.mjs';
import { GROUPS, RUNNABLE, SUITES, suiteById } from './suites.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const writeJson = (path, value) => writeFileSync(path, JSON.stringify(value, null, 2) + '\n');

const argv = process.argv.slice(2);
const ids = [], suiteArgs = [];
let jobs = 1, output, list = false, build = true, keepEvidence = false, keepRuns = 3, affected = false, base = 'origin/main';
for (let index = 0; index < argv.length; index++) {
  const arg = argv[index];
  if (arg === '--jobs') jobs = Number(argv[++index]);
  else if (arg === '--output') output = resolve(argv[++index]);
  else if (arg === '--list') list = true;
  else if (arg === '--no-build') build = false;
  else if (arg === '--keep-evidence') keepEvidence = true;
  else if (arg === '--keep-runs') keepRuns = Number(argv[++index]);
  else if (arg === '--affected') affected = true;
  else if (arg === '--base') base = argv[++index];
  else if (arg === '--suite') ids.push(...argv[++index].split(','));
  else if (arg === '--groups') { console.log(JSON.stringify(GROUPS, null, 2)); process.exit(0); }
  else if (!arg.startsWith('--') && !suiteArgs.length) ids.push(...arg.split(','));
  else suiteArgs.push(arg);
}
if (!Number.isInteger(jobs) || jobs < 1) { console.error('--jobs must be a positive integer'); process.exit(2); }
if (!Number.isInteger(keepRuns) || keepRuns < 1) { console.error('--keep-runs must be a positive integer'); process.exit(2); }
// `@group` expands to its suites; a suite named twice runs once.
for (const id of ids.filter(id => id.startsWith('@'))) if (!GROUPS[id.slice(1)]) {
  console.error(`Unknown group ${id}; known: ${Object.keys(GROUPS).map(group => `@${group}`).join(' ')}`);
  process.exit(2);
}
ids.splice(0, ids.length, ...new Set(ids.flatMap(id => id.startsWith('@') ? GROUPS[id.slice(1)] : [id])));
const unknown = ids.filter(id => !suiteById.has(id));
if (unknown.length) {
  console.error(`Unknown suite ${unknown.join(', ')}; known: ${SUITES.map(suite => suite.id).join(', ')}`);
  process.exit(2);
}
let selection;
if (affected) {
  if (ids.length) { console.error('--affected selects suites itself; name suites or pass --affected, not both'); process.exit(2); }
  selection = (await import('./affected.mjs')).select(base);
  ids.push(...selection.selected.map(item => item.id));
  if (!list) console.log(`--affected: ${ids.length}/${RUNNABLE.length} suites for ${selection.files.length} files changed since ${base} (${selection.mergeBase.slice(0, 10)}): ${ids.join(' ') || 'none'}`);
  if (!ids.length && !list) process.exit(0);
}
if (list) {
  console.log(JSON.stringify(selection ? selection.selected : RUNNABLE.map(suite => suite.id), null, selection ? 2 : 0));
  process.exit(0);
}
const selected = ids.length ? ids.map(id => suiteById.get(id)) : RUNNABLE;
// Drivers take different flags (e.g. paired counts with --random-count), so
// suite flags only make sense for one named suite.
if (suiteArgs.length && selected.length !== 1) {
  console.error(`Suite flags (${suiteArgs.join(' ')}) need exactly one suite: npm run test:e2e -- <suite> ${suiteArgs.join(' ')}`);
  process.exit(2);
}

// Caller flags replace the registry's flag of the same name rather than
// relying on each driver's parser to prefer the last occurrence.
function mergeArgs(defaults, overrides) {
  const names = new Set(overrides.filter(arg => arg.startsWith('--')));
  const merged = [];
  for (let index = 0; index < defaults.length; index++) {
    const arg = defaults[index], takesValue = defaults[index + 1] !== undefined && !defaults[index + 1].startsWith('--');
    if (names.has(arg)) { if (takesValue) index++; continue; }
    merged.push(arg);
  }
  return [...merged, ...overrides];
}

function newestMtime(directory) {
  let newest = 0;
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) { if (entry.name !== 'generated' && entry.name !== 'target') newest = Math.max(newest, newestMtime(path)); }
    else newest = Math.max(newest, statSync(path).mtimeMs);
  }
  return newest;
}

function sh(command, args) {
  console.log(`$ ${command} ${args.join(' ')}`);
  const result = spawnSync(command, args, { cwd: ROOT, stdio: 'inherit' });
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const dist = join(ROOT, 'dist/index.html');
const sourcesChanged = () => ['web', 'engine/src', 'engine/Cargo.toml', 'index.html', 'vite.config.ts']
  .map(path => join(ROOT, path)).filter(existsSync)
  .some(path => (statSync(path).isDirectory() ? newestMtime(path) : statSync(path).mtimeMs) > statSync(dist).mtimeMs);
if (build && (!existsSync(dist) || sourcesChanged())) sh('npm', ['run', 'build']);
// Seed discovery runs the native floor generator; cargo is a no-op when fresh,
// so it is rebuilt even under --no-build (a stale generator fails discovery).
sh('cargo', ['build', '--manifest-path', 'engine/Cargo.toml', '--locked', '--bin', 'floor']);

const runDirectory = output || join(ROOT, '.cache/e2e', `run-${new Date().toISOString().replace(/[:.]/g, '-')}`);
const report = { schema: 2, status: 'running', startedAt: new Date().toISOString(), jobs, suiteArgs, suites: [],
  ...(selection && { affected: { base, mergeBase: selection.mergeBase, files: selection.files, selected: selection.selected } }) };
const reportPath = join(runDirectory, 'report.json');
mkdirSync(runDirectory, { recursive: true });
writeJson(reportPath, report);

// Retention (tools/e2e/lib/retention.mjs): older runs keep their reports,
// scenarios, cases, logs and small files; killed suites' dist copies go.
if (!keepEvidence) {
  for (const directory of sweepProductionSnapshots()) console.log(`swept stale production snapshot ${directory}`);
  // A full entities run alone writes several GB; below LOW_DISK free, every
  // older run is slimmed, not just those beyond --keep-runs.
  const LOW_DISK = 12 * 2 ** 30;
  const low = freeBytes(ROOT) < LOW_DISK;
  if (low) console.log(`low disk (${Math.round(freeBytes(ROOT) / 2 ** 30)} GB free): slimming every older run`);
  for (const run of await pruneRuns(join(ROOT, '.cache/e2e'), low ? 1 : keepRuns, runDirectory))
    console.log(`slimmed ${relative(ROOT, run.directory)}: ${run.files} files, ${Math.round(run.bytes / 2 ** 20)} MB`);
}

// Paired-game suites replay recorded case.json tapes; every other driver
// replays its scenarios.json through the suite's replay flag.
function replayCommand(suite, directory) {
  if (suite.entry === 'tools/foundation/paired-game.mjs') {
    let cases = [];
    try { cases = JSON.parse(readFileSync(join(directory, 'report.json'), 'utf8')).cases ?? []; } catch {}
    const failed = cases.filter(item => item.status !== 'pass' && item.directory).map(item => relative(ROOT, join(item.directory, 'case.json')));
    return failed.length ? `npm run test:e2e -- ${suite.id} --replay-cases ${failed.join(',')}` : undefined;
  }
  if (suite.replay === false) return undefined;
  return existsSync(join(directory, 'scenarios.json'))
    ? `npm run test:e2e -- ${suite.id} ${suite.replay ?? '--scenarios'} ${relative(ROOT, join(directory, 'scenarios.json'))}` : undefined;
}

async function runSuite(suite) {
  const release = await acquireHostSlot(suite.id);
  try { return await runSuiteInSlot(suite); } finally { release(); }
}

function runSuiteInSlot(suite) {
  const directory = join(runDirectory, suite.id);
  const args = [...mergeArgs(suite.args, suiteArgs), '--output', directory];
  const logPath = join(runDirectory, `${suite.id}.log`);
  const log = createWriteStream(logPath);
  const startedAt = Date.now();
  console.log(`▶ ${suite.id}: node ${suite.entry} ${args.join(' ')}`);
  return new Promise(done => {
    const child = spawn('bash', ['tools/original/with-display.sh', 'node', suite.entry, ...args], { cwd: ROOT });
    for (const stream of [child.stdout, child.stderr]) stream.on('data', chunk => {
      log.write(chunk);
      if (jobs === 1) process.stdout.write(chunk);
    });
    const finish = async (exitCode, signal, error) => {
      log.end();
      const result = { id: suite.id, status: exitCode === 0 && !signal ? 'pass' : 'fail', exitCode, signal, error,
        seconds: Math.round((Date.now() - startedAt) / 1000), log: logPath, output: directory };
      if (result.status === 'fail') result.replay = replayCommand(suite, directory);
      // A passing suite keeps what replay and triage need; a failing one keeps everything.
      else if (!keepEvidence) result.retention = await slimSuite(directory);
      report.suites.push(result);
      writeJson(reportPath, report);
      console.log(`${result.status === 'pass' ? '✔' : '✘'} ${suite.id} (${result.seconds}s)${result.status === 'fail' ? ` log: ${relative(ROOT, logPath)}` : ''}`);
      done(result);
    };
    child.once('error', error => finish(null, null, error.message));
    child.once('close', finish);
  });
}

const queue = [...selected];
await Promise.all(Array.from({ length: Math.min(jobs, queue.length) }, async () => {
  while (queue.length) await runSuite(queue.shift());
}));

report.status = report.suites.every(suite => suite.status === 'pass') ? 'pass' : 'fail';
report.finishedAt = new Date().toISOString();
writeJson(reportPath, report);
console.log(`\n${report.suites.filter(suite => suite.status === 'pass').length}/${report.suites.length} suites passed — ${relative(ROOT, reportPath)}`);
for (const suite of report.suites.filter(suite => suite.status === 'fail')) {
  console.log(`  ✘ ${suite.id}: ${relative(ROOT, suite.log)}`);
  if (suite.replay) console.log(`      replay: ${suite.replay}`);
}
if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, [
  '| suite | result | time | replay |', '|---|---|---|---|',
  ...report.suites.map(suite => `| ${suite.id} | ${suite.status === 'pass' ? '✅' : '❌'} | ${suite.seconds}s | ${suite.replay ? `\`${suite.replay}\`` : ''} |`),
  '',
].join('\n'));
process.exitCode = report.status === 'pass' ? 0 : 1;
