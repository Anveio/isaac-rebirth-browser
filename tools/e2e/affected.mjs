// Map changed files to the live suites that must run for them.
//
//   npm run test:e2e -- --affected [--base REF]      run only the affected suites
//   node tools/e2e/affected.mjs [--base REF]         print the selection and why
//   npm run affected:map [-- RUN_DIR]                regenerate affected-map.json from a run
//                                                    (default: the newest .cache/e2e/run-*)
//
// Tool and evidence files select the suites whose driver, libraries, scenario
// helpers, observers or comparators read them (the static import closure).
// Engine Rust selects at function level: affected-map.json records which engine
// functions each suite's recorded browser traces execute when replayed natively
// under coverage, and a coverage build of the current tree locates the functions
// a diff touches. Anything the map cannot place (web/, build inputs, a new or
// unexecuted engine function, a suite with no recorded traces) selects
// conservatively. The selection is a fast inner loop, not a replacement for the
// full run before merging a broad change.
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, lstatSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gunzipSync } from 'node:zlib';
import { SUITES as ALL_SUITES, suiteById } from './suites.mjs';

// Suites a full run covers; an alias (e.g. entity-<name>) is credited to the
// suite it runs a subset of.
const SUITES = ALL_SUITES.filter(suite => !suite.alias);
const owner = id => suiteById.get(id)?.alias ?? id;

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const MAP = join(ROOT, 'tools/e2e/affected-map.json');
const COVERAGE = join(ROOT, '.cache/coverage');
const git = (...args) => execFileSync('git', args, { cwd: ROOT, encoding: 'utf8', maxBuffer: 64 << 20 });

// Files that never change what a live suite observes.
const INERT = [/\.md$/, /^docs\//, /^engine\/dst\//, /^tools\/dst\//, /^\.claude\//, /^\.github\//, /^cloud\//,
  /^tools\/e2e\/(affected\.mjs|affected-map\.json|policy\.mjs)$/, /^tools\/foundation\/(matrix|dst-index)\.mjs$/, /^tools\/playtest\//];
// Files every suite reads: the runner, the build and the production page.
const SHARED = [/^tools\/e2e\/(run|suites)\.mjs$/, /^tools\/original\/(with-display\.sh|setup-runtime\.py|runtime-packages\.json)$/,
  /^engine\/(build\.rs|Cargo\.(toml|lock))$/, /^engine\/src\/bin\/replay\.rs$/, /^package(-lock)?\.json$/, /^(index\.html|vite\.config\.ts|tsconfig\.json)$/,
  /^tools\/build-wasm\.sh$/, /^web\//];

// ---- static closure of each suite's tool files ----

function closure(path, seen) {
  const absolute = resolve(ROOT, path);
  if (seen.has(absolute) || !existsSync(absolute) || statSync(absolute).isDirectory()) return;
  seen.add(absolute);
  if (!/\.(mjs|js|py|sh|c)$/.test(absolute)) return;
  const source = readFileSync(absolute, 'utf8');
  // Scenario helpers that run the native floor generator read its source.
  if (source.includes('engine/target/debug/floor') || source.includes("'--bin', 'floor'")) seen.add(join(ROOT, 'engine/src/bin/floor.rs'));
  const follow = relativePath => closure(relativePath, seen);
  for (const match of source.matchAll(/(?:from|import)\s*\(?\s*['"](\.{1,2}\/[^'"]+)['"]/g)) follow(resolve(dirname(absolute), match[1]));
  for (const match of source.matchAll(/['"`]((?:tools|references)\/[A-Za-z0-9_./-]+)['"`]/g)) {
    // A literal directory (e.g. a fixture bank) brings every file in it.
    const target = resolve(ROOT, match[1]);
    if (existsSync(target) && statSync(target).isDirectory()) for (const file of git('ls-files', match[1]).split('\n').filter(Boolean)) follow(file);
    else follow(match[1]);
  }
  // Sibling scripts named bare, e.g. load('x', 'foundation-paired-scenarios.py') or helper('npc-scenarios.py').
  for (const match of source.matchAll(/['"]([A-Za-z0-9_-]+\.(?:py|c|sh|json))['"]/g)) follow(join(dirname(absolute), match[1]));
  if (absolute.endsWith('.py')) for (const match of source.matchAll(/^\s*(?:from|import)\s+([A-Za-z0-9_]+)/gm)) follow(join(dirname(absolute), `${match[1]}.py`));
}

export function suiteFiles(suite) {
  const seen = new Set();
  closure(suite.entry, seen);
  for (const arg of suite.args) for (const path of arg.split(',')) if (/^(tools|references)\//.test(path)) closure(path, seen);
  // Entity specs are loaded dynamically: entity-<name> reads its own spec, and
  // a suite whose helper or comparator reads the spec list reads them all.
  if (suite.id.startsWith('entity-')) closure(`tools/e2e/entities/${suite.id.slice(7)}.mjs`, seen);
  else if ([...seen].some(path => path.endsWith('tools/original/entities-scenarios.py'))) {
    closure('tools/e2e/entities.mjs', seen);
    for (const file of git('ls-files', 'tools/e2e/entities').split('\n').filter(Boolean)) closure(file, seen);
  }
  return new Set([...seen].map(path => relative(ROOT, path)));
}

// ---- engine coverage ----

// Demangled names (without crate hashes) stay stable across unrelated edits.
function demangler() {
  const path = join(ROOT, '.cache/cargo-tools/bin/rustfilt');
  if (!existsSync(path)) execFileSync('cargo', ['install', '--root', join(ROOT, '.cache/cargo-tools'), 'rustfilt', '--version', '0.2.1', '--locked'],
    { stdio: ['ignore', 'ignore', 'inherit'] });
  return path;
}
function demangle(functions) {
  const names = execFileSync(demangler(), [], { input: functions.map(fn => fn.name).join('\n'), encoding: 'utf8', maxBuffer: 1 << 30 }).split('\n');
  return functions.map((fn, index) => ({ ...fn, name: names[index] }));
}

const sysroot = () => execFileSync('rustc', ['--print', 'sysroot'], { encoding: 'utf8' }).trim();
function llvmTool(name) {
  const base = join(sysroot(), 'lib/rustlib');
  for (const host of existsSync(base) ? readdirSync(base) : []) {
    const path = join(base, host, 'bin', name);
    if (existsSync(path)) return path;
  }
  throw new Error(`${name} not found; run: rustup component add llvm-tools`);
}

// The native replay binary built with source-based coverage for the current tree.
function coverageBinary() {
  const target = join(ROOT, '.cache/coverage-target');
  const result = spawnSync('cargo', ['build', '--manifest-path', 'engine/Cargo.toml', '--locked', '--release', '--bin', 'replay', '--target-dir', target],
    { cwd: ROOT, encoding: 'utf8', env: { ...process.env, RUSTFLAGS: '-C instrument-coverage', LLVM_PROFILE_FILE: join(COVERAGE, 'build/%p-%m.profraw') } });
  if (result.status !== 0) throw new Error(`coverage build failed:\n${result.stderr}`);
  return join(target, 'release/replay');
}

function runReplays(binary, directory, inputs) {
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  const failures = [];
  for (const input of inputs) {
    const result = spawnSync(binary, [], { input, env: { ...process.env, LLVM_PROFILE_FILE: join(directory, '%p-%m.profraw') },
      maxBuffer: 1 << 30, stdio: ['pipe', 'ignore', 'pipe'] });
    if (result.status !== 0) failures.push(result.stderr.toString().trim().split('\n').at(-1));
  }
  const raws = readdirSync(directory).filter(name => name.endsWith('.profraw')).map(name => join(directory, name));
  execFileSync(llvmTool('llvm-profdata'), ['merge', '-sparse', '-o', join(directory, 'merged.profdata'), ...raws]);
  const exported = JSON.parse(execFileSync(llvmTool('llvm-cov'), ['export', binary, `-instr-profile=${join(directory, 'merged.profdata')}`,
    '-skip-expansions'], { maxBuffer: 1 << 30, encoding: 'utf8' }));
  return { functions: demangle(exported.data[0].functions.filter(fn => engineFile(fn.filenames[0]))), failures };
}

const engineFile = path => {
  const at = path.indexOf('/engine/src/');
  return at < 0 ? undefined : path.slice(at + 1);
};

// Name and line span of every engine function in the current tree.
export function functionSpans() {
  const binary = coverageBinary();
  const { functions } = runReplays(binary, join(COVERAGE, 'spans'), ['']);
  const spans = [];
  for (const fn of functions) {
    const file = engineFile(fn.filenames[0]);
    const lines = fn.regions.filter(region => region[5] === 0).flatMap(region => [region[0], region[2]]);
    spans.push({ name: fn.name, file, start: Math.min(...lines), end: Math.max(...lines) });
  }
  return spans;
}

// Every recorded browser trace below a suite's output directory.
function* traces(value) {
  if (!value || typeof value !== 'object') return;
  if (Number.isInteger(value.initialSeed) && Array.isArray(value.commands) && value.commands.every(item => item?.type)) { yield value; return; }
  for (const child of Array.isArray(value) ? value : Object.values(value)) yield* traces(child);
}
function* jsonFiles(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* jsonFiles(path);
    // Browser traces live in browser-side evidence; native manifests never hold one.
    else if (/(trace|browser).*\.json(\.gz)?$/.test(entry.name) && statSync(path).size < 512 << 20) yield path;
  }
}
function replayInputs(directory) {
  const seen = new Set();
  for (const path of jsonFiles(directory)) {
    let value;
    try { value = JSON.parse((path.endsWith('.gz') ? gunzipSync(readFileSync(path)) : readFileSync(path)).toString()); } catch { continue; }
    for (const trace of traces(value)) {
      // Backend callbacks are commands the page dispatched between inputs.
      const commands = [...trace.commands];
      for (const event of [...(trace.backendEvents ?? [])].reverse()) commands.splice(event.afterCommandCount, 0, event.command);
      seen.add(JSON.stringify({ initialSeed: trace.initialSeed, bootEpochSeed: trace.bootEpochSeed ?? 0, commands }));
    }
  }
  return [...seen];
}

export function updateMap(runDirectory) {
  const binary = coverageBinary();
  const suites = {}, functions = {};
  for (const suite of SUITES) {
    const directory = join(runDirectory, suite.id);
    const inputs = existsSync(directory) ? replayInputs(directory) : [];
    if (!inputs.length) { console.error(`${suite.id}: no recorded traces`); continue; }
    const { functions: covered, failures } = runReplays(binary, join(COVERAGE, suite.id), inputs);
    let executed = 0;
    for (const fn of covered) if (fn.count > 0) {
      const name = fn.name;
      (functions[name] ??= new Set()).add(suite.id);
      executed++;
    }
    suites[suite.id] = { traces: inputs.length, rejected: failures.length };
    console.error(`${suite.id}: ${inputs.length} traces, ${failures.length} rejected, ${executed} engine function instances executed`);
  }
  // Functions executed by the same suites share one suite set.
  const sets = [], setIndex = new Map(), byName = {};
  for (const [name, ids] of Object.entries(functions).sort()) {
    const key = [...ids].sort().join(' ');
    if (!setIndex.has(key)) { setIndex.set(key, sets.length); sets.push(key); }
    byName[name] = setIndex.get(key);
  }
  const map = { schema: 2, generatedAt: new Date().toISOString().slice(0, 10), suites, sets, functions: byName };
  // One entry per line keeps regenerated maps diffable.
  writeFileSync(MAP, JSON.stringify(map).replace(/,"(?=[^"]+":)/g, ',\n"').replace(/\{"/g, '{\n"') + '\n');
  return map;
}

// ---- selection ----

export function changedFiles(base) {
  const mergeBase = git('merge-base', base, 'HEAD').trim();
  const names = [git('diff', '--name-only', mergeBase), git('ls-files', '--others', '--exclude-standard')].join('\n');
  // Untracked symlinks (a shared node_modules, a linked depot) are not source.
  const files = names.split('\n').filter(name => name && !(existsSync(join(ROOT, name)) && lstatSync(join(ROOT, name)).isSymbolicLink()));
  return { mergeBase, files: [...new Set(files)].sort() };
}

// Changed line ranges on the working-tree side of each engine file.
function changedLines(mergeBase, file) {
  const ranges = [];
  for (const match of git('diff', '-U0', mergeBase, '--', file).matchAll(/^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@/gm)) {
    const start = Number(match[1]), count = match[2] === undefined ? 1 : Number(match[2]);
    // A pure deletion touches the line it was removed before.
    ranges.push([Math.max(1, start), start + Math.max(count, 1) - 1]);
  }
  return ranges;
}

export function select(base = 'origin/main') {
  const { mergeBase, files } = changedFiles(base);
  const stored = existsSync(MAP) ? JSON.parse(readFileSync(MAP, 'utf8')) : undefined;
  const map = stored && { ...stored, functions: Object.fromEntries(Object.entries(stored.functions).map(([name, set]) => [name, stored.sets[set].split(' ')])) };
  const reasons = new Map(SUITES.map(suite => [suite.id, []]));
  const all = reason => SUITES.forEach(suite => reasons.get(suite.id).push(reason));
  const closures = new Map(SUITES.map(suite => [suite.id, suiteFiles(suite)]));
  const ignored = [];
  let spans;
  for (const file of files) {
    if (INERT.some(pattern => pattern.test(file))) { ignored.push(file); continue; }
    if (SHARED.some(pattern => pattern.test(file))) { all(file); continue; }
    if (/^engine\/src\/.*\.rs$/.test(file) && file !== 'engine/src/bin/floor.rs') {
      if (!map) { all(`${file} (no affected-map.json)`); continue; }
      spans ??= functionSpans();
      const ranges = existsSync(join(ROOT, file)) ? changedLines(mergeBase, file) : [[1, Infinity]];
      const touched = spans.filter(span => span.file === file && ranges.some(([start, end]) => start <= span.end && end >= span.start));
      const outside = ranges.some(([start, end]) => !spans.some(span => span.file === file && start >= span.start && end <= span.end));
      // Items, types and constants outside any function, or a function no suite
      // has executed yet, select every suite that executes anything in the file.
      const unknown = touched.filter(span => !map.functions[span.name]);
      if (outside || unknown.length) {
        const users = new Set(spans.filter(span => span.file === file).flatMap(span => map.functions[span.name] ?? []).map(owner));
        const why = outside ? 'items outside functions' : `unexecuted ${unknown.map(span => span.name).join(', ')}`;
        if (users.size) SUITES.forEach(suite => users.has(suite.id) && reasons.get(suite.id).push(`${file} (${why})`));
        else all(`${file} (${why}; no suite executes this file)`);
      }
      for (const span of touched) for (const id of map.functions[span.name] ?? []) reasons.get(owner(id))?.push(`${file}: ${span.name}`);
      // Suites without recorded traces have no coverage; any engine change selects them.
      const mapped = new Set(Object.keys(map.suites).map(owner));
      for (const suite of SUITES) if (!mapped.has(suite.id)) reasons.get(suite.id).push(`${file} (suite has no engine coverage map)`);
      continue;
    }
    let hit = false;
    for (const [id, closure] of closures) if (closure.has(file)) { reasons.get(id).push(file); hit = true; }
    if (!hit) {
      if (/^(tools|references)\//.test(file)) ignored.push(`${file} (read by no registered suite)`);
      else all(`${file} (unclassified)`);
    }
  }
  const selected = SUITES.filter(suite => reasons.get(suite.id).length).map(suite => ({ id: suite.id, reasons: [...new Set(reasons.get(suite.id))] }));
  return { base, mergeBase, files, ignored, selected, map: map && { generatedAt: map.generatedAt } };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const argv = process.argv.slice(2), at = name => argv.indexOf(name);
  if (at('--update-map') >= 0) {
    const runs = join(ROOT, '.cache/e2e');
    const given = argv[at('--update-map') + 1];
    const newest = existsSync(runs) ? readdirSync(runs).filter(name => name.startsWith('run-')).sort().at(-1) : undefined;
    if (!given && !newest) throw new Error('No run directory: pass one, or run npm run test:e2e first');
    updateMap(given ? resolve(given) : join(runs, newest));
  }
  else {
    const result = select(at('--base') >= 0 ? argv[at('--base') + 1] : 'origin/main');
    for (const item of result.selected) console.log(`${item.id}\n  ${item.reasons.slice(0, 6).join('\n  ')}${item.reasons.length > 6 ? `\n  … ${item.reasons.length - 6} more` : ''}`);
    if (result.ignored.length) console.log(`no suite: ${result.ignored.join(', ')}`);
    console.log(`${result.selected.length}/${SUITES.length} suites for ${result.files.length} changed files since ${result.mergeBase.slice(0, 10)}`);
  }
}
