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
