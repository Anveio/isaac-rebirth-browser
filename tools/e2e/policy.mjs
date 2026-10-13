// Keep verification at the game level (see tools/e2e/README.md). Fails when a
// change checks in unit tests or hard-codes checksums outside the input pins.
import { globSync, readFileSync } from 'node:fs';
import { ROOT } from './lib/cli.mjs';

// Files that pin the identity of original inputs (binary, authored assets).
// A golden hash of anything the port produces does not belong here.
const INPUT_PINS = new Set([
  'tools/e2e/lib/original.mjs',
  'tools/original/native-identity.py',
  'tools/original/audio-observer.py',
  'tools/original/music-observer.py',
  'tools/original/familiar-observer.py',
  'tools/original/foundation-combat-observe.py',
  'tools/original/foundation-layout-observer.py',
  'tools/original/foundation-pickup-observe.py',
  'tools/original/item-observer.py',
  'tools/foundation/item-catalog.py',
  'tools/foundation/stage-masks.mjs',
]);

const ignore = ['**/node_modules/**', '**/target/**', '**/.cache/**', '**/dist/**', '**/.venv/**'];
const files = pattern => globSync(pattern, { cwd: ROOT, exclude: ignore });
const problems = [];

for (const path of files('**/*.{test,spec}.{js,mjs,cjs,ts,tsx}').concat(files('**/{test_*,*_test}.py'), files('**/__tests__/**'), files('engine/{tests,benches}/**')))
  if (!path.startsWith('tools/tests/')) problems.push(`${path}: unit test file; verify behavior in a live e2e suite instead`);

for (const path of files('engine/src/**/*.rs'))
  if (/#\[(cfg\(test\)|test)\]/.test(readFileSync(`${ROOT}/${path}`, 'utf8'))) problems.push(`${path}: Rust unit test; verify behavior in a live e2e suite instead`);

for (const path of files('tools/**/*.{mjs,js,ts,py,sh}')) {
  if (INPUT_PINS.has(path)) continue;
  if (/['"`][a-f0-9]{64}['"`]/.test(readFileSync(`${ROOT}/${path}`, 'utf8')))
    problems.push(`${path}: hard-coded sha256; compare behavior against the original, or import the pin from tools/e2e/lib/original.mjs`);
}

if (problems.length) {
  console.error(problems.join('\n'));
  process.exit(1);
}
console.log('Test policy: no unit tests or hard-coded output checksums.');
