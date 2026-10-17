// Small shared helpers for e2e drivers: flags, output directories and JSON.
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export const ROOT = resolve(fileURLToPath(new URL('../../..', import.meta.url)));

// Last occurrence wins, so callers can override a registry default by
// appending the same flag (e.g. `npm run test:e2e -- keys --count 4`).
export function flag(name, fallback, argv = process.argv) {
  const at = argv.lastIndexOf(name);
  if (at < 0) return fallback;
  const value = argv[at + 1];
  assert.ok(value !== undefined && !value.startsWith('--'), `Missing value for ${name}`);
  return value;
}

export const hasFlag = (name, argv = process.argv) => argv.includes(name);

export function outputDirectory(suite) {
  const output = resolve(flag('--output') || join(ROOT, '.cache/e2e', suite, `${Date.now()}-${process.pid}`));
  mkdirSync(output, { recursive: true });
  return output;
}

export function writeJson(path, value) {
  writeFileSync(path, JSON.stringify(value, null, 2) + '\n');
  return path;
}

// `--case PATTERNS` keeps the cases whose id matches one of the comma-separated
// patterns (`*` matches any run of characters); without it every case runs.
// Drivers apply it after validating the full generated bank, so the bank's own
// contrived/OS-random composition checks still hold, and before recording
// scenarios.json, so the printed replay reruns exactly the cases that ran.
export function selectCases(cases, idOf, argv = process.argv) {
  const patterns = flag('--case', undefined, argv);
  if (!patterns) return cases;
  const escape = text => text.replace(/[.+?^${}()|[\]\\]/g, '\\$&');
  const tests = patterns.split(',').map(pattern => new RegExp(`^${pattern.split('*').map(escape).join('.*')}$`));
  const kept = cases.filter(item => tests.some(test => test.test(idOf(item))));
  assert.ok(kept.length, `--case ${patterns} matches none of: ${cases.map(idOf).join(', ')}`);
  console.log(`--case ${patterns}: ${kept.length}/${cases.length} cases (${kept.map(idOf).join(', ')})`);
  return kept;
}
