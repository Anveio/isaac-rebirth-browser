// The release bundle must not contain the port-only debug sandbox floor
// (web/debug-sandbox.ts, engine/src/debug_sandbox.rs). npm run build leaves
// __DEBUG_SANDBOX__ false and the cargo feature off; this scan of dist/ fails
// the build if either ever leaks into shipped JavaScript, HTML or WASM.
//
//   node tools/foundation/check-bundle.mjs [DIST]
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const dist = resolve(process.argv[2] ?? join(ROOT, 'dist'));
if (!existsSync(join(dist, 'index.html'))) {
  console.error(`${relative(ROOT, dist)}/index.html is missing; run npm run build first`);
  process.exit(1);
}
// The panel's marker, its engine exports and its module name.
const MARKERS = ['debug-sandbox-floor', 'debug_sandbox_', 'mountDebugSandbox'].map(text => Buffer.from(text));
const leaks = [];
let scanned = 0;
const walk = directory => {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) walk(path);
    else if (/\.(js|mjs|html|wasm)$/.test(entry.name)) {
      scanned++;
      const bytes = readFileSync(path);
      for (const marker of MARKERS) if (bytes.includes(marker)) leaks.push(`${relative(ROOT, path)}: ${marker}`);
    }
  }
};
walk(dist);
if (leaks.length) {
  for (const leak of leaks) console.error(`debug sandbox leaked into the release bundle: ${leak}`);
  process.exit(1);
}
console.log(`Bundle check: no debug sandbox code in ${scanned} shipped scripts, pages and modules`);
