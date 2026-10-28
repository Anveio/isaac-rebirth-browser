// Disk retention for live e2e runs. A full run writes several GB of frames,
// traces and raw states; what replay and triage need is small:
//   - every suite keeps report.json, scenarios.json, case.json, logs and every
//     file of at most 1 MB except images (PNG frames and the raw RGBA texture
//     dumps of a --gl-trace), so a recorded case can always be replayed;
//   - recorded browser traces are gzipped, not dropped, and the input traces
//     embedded in large browser evidence archives are kept beside them (the
//     --affected map is rebuilt from them);
//   - a failing suite keeps everything until its run falls out of the newest
//     --keep-runs runs, or at once when the disk runs low mid-run (run.mjs).
// Only runner-created run directories under .cache/e2e are ever pruned.
import { createReadStream, createWriteStream, existsSync, readdirSync, readFileSync, rmSync, statfsSync, statSync, unlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pipeline } from 'node:stream/promises';
import { createGzip, gunzipSync, gzipSync } from 'node:zlib';

const ALWAYS = new Set(['report.json', 'scenarios.json', 'case.json']);
const SMALL = 1 << 20;

function* files(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* files(path);
    else if (entry.isFile()) yield [path, entry.name];
  }
}

// Slim one suite output directory in place; returns what was removed.
export async function slimSuite(directory) {
  const removed = { files: 0, bytes: 0 }, gzipped = { files: 0, bytes: 0 };
  if (!existsSync(directory)) return { removed, gzipped };
  for (const [path, name] of [...files(directory)]) {
    const size = statSync(path).size;
    if (ALWAYS.has(name) || name.endsWith('.log') || (size <= SMALL && !/\.(png|rgba)$/.test(name))) continue;
    if (/trace.*\.json$/.test(name)) {
      await pipeline(createReadStream(path), createGzip({ level: 6 }), createWriteStream(`${path}.gz`));
      unlinkSync(path);
      gzipped.files++; gzipped.bytes += size - statSync(`${path}.gz`).size;
      continue;
    }
    if (name.endsWith('.gz') && /trace/.test(name)) continue;
    if (/^browser.*\.json(\.gz)?$/.test(name)) keepInputTraces(path);
    unlinkSync(path);
    removed.files++; removed.bytes += size;
  }
  return { removed, gzipped };
}

// The public input part (seed, commands, backend callbacks) of every trace in
// a browser evidence file, without its state history: <name>.traces.json.gz.
function keepInputTraces(path) {
  let document;
  try { document = JSON.parse((path.endsWith('.gz') ? gunzipSync(readFileSync(path)) : readFileSync(path)).toString()); } catch { return; }
  const found = [];
  (function walk(value) {
    if (!value || typeof value !== 'object') return;
    if (Number.isInteger(value.initialSeed) && Array.isArray(value.commands)) {
      found.push({ initialSeed: value.initialSeed, bootEpochSeed: value.bootEpochSeed ?? 0, commands: value.commands, backendEvents: (value.backendEvents ?? []).map(({ afterCommandCount, command }) => ({ afterCommandCount, command })) });
      return;
    }
    for (const child of Array.isArray(value) ? value : Object.values(value)) walk(child);
  })(document);
  if (found.length) writeFileSync(path.replace(/\.json(\.gz)?$/, '.traces.json.gz'), gzipSync(JSON.stringify({ schema: 1, traces: found })));
}

// Slim every suite of the runner-created runs older than the newest `keep`.
// A runner-created run is any directory directly under `root` holding the
// runner's schema-2 report, whether named run-* or chosen with --output.
// Runs still marked running are left alone unless their report is a day old.
export async function pruneRuns(root, keep, current) {
  if (!existsSync(root)) return [];
  const runs = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const directory = join(root, entry.name), reportPath = join(directory, 'report.json');
    if (!entry.isDirectory() || directory === current || !existsSync(reportPath)) continue;
    let report;
    try { report = JSON.parse(readFileSync(reportPath, 'utf8')); } catch { continue; }
    if (report.schema !== 2 || !Array.isArray(report.suites) || report.retention?.slimmed) continue;
    const stale = Date.now() - statSync(reportPath).mtimeMs > 24 * 3600 * 1000;
    if (report.status === 'running' && !stale) continue;
    runs.push({ directory, reportPath, report, startedAt: Date.parse(report.startedAt) || 0 });
  }
  runs.sort((a, b) => b.startedAt - a.startedAt);
  const pruned = [];
  for (const run of runs.slice(Math.max(0, keep - 1))) {
    const totals = { files: 0, bytes: 0 };
    for (const entry of readdirSync(run.directory, { withFileTypes: true })) if (entry.isDirectory()) {
      const { removed } = await slimSuite(join(run.directory, entry.name));
      totals.files += removed.files; totals.bytes += removed.bytes;
    }
    run.report.retention = { ...run.report.retention, slimmed: new Date().toISOString(), removed: totals };
    writeFileSync(run.reportPath, JSON.stringify(run.report, null, 2) + '\n');
    pruned.push({ directory: run.directory, ...totals });
  }
  return pruned;
}

// Production snapshots whose owning process is gone (e.g. a killed suite).
export function sweepProductionSnapshots() {
  const swept = [];
  for (const name of readdirSync(tmpdir())) {
    if (!name.startsWith('isaac-production-')) continue;
    const directory = join(tmpdir(), name);
    let alive;
    try { alive = isAlive(Number(readFileSync(join(directory, 'owner.pid'), 'utf8'))); }
    catch { alive = Date.now() - statSync(directory).mtimeMs < 24 * 3600 * 1000; }
    if (!alive) { rmSync(directory, { recursive: true, force: true }); swept.push(directory); }
  }
  return swept;
}

function isAlive(pid) {
  if (!Number.isInteger(pid) || pid <= 0) return false;
  try { process.kill(pid, 0); } catch (error) { return error.code === 'EPERM'; }
  if (process.platform === 'linux') {
    try {
      const stat = readFileSync(`/proc/${pid}/stat`, 'utf8');
      // The command name may itself contain spaces or parentheses.
      const commandEnd = stat.lastIndexOf(')');
      if (stat.startsWith(`${pid} (`) && stat.slice(commandEnd + 1).startsWith(' Z ')) return false;
    } catch { /* Keep the kill(pid, 0) result when procfs is unavailable. */ }
  }
  return true;
}

// Free bytes on the filesystem holding `path` (statfs; Infinity if unknown).
export function freeBytes(path) {
  try { const stats = statfsSync(path); return stats.bavail * stats.bsize; } catch { return Infinity; }
}
