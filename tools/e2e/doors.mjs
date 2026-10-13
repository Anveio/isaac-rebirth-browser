// Share the public-input traversal driver and its original provenance checks.
import { fileURLToPath } from 'node:url';
process.argv.push('--scenario-helper', fileURLToPath(new URL('../original/door-scenarios.py', import.meta.url)));
if (!process.argv.includes('--output'))
  process.argv.push('--output', fileURLToPath(new URL(`../../.cache/e2e/doors/${Date.now()}-${process.pid}`, import.meta.url)));
await import('./traversal.mjs');
