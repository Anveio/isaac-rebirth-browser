// Public bomb inputs share the traversal driver and source provenance checks.
import { fileURLToPath } from 'node:url';
process.argv.push('--scenario-helper', fileURLToPath(new URL('../original/secret-door-scenarios.py', import.meta.url)));
if (!process.argv.includes('--output'))
  process.argv.push('--output', fileURLToPath(new URL(`../../.cache/e2e/secrets/${Date.now()}-${process.pid}`, import.meta.url)));
await import('./traversal.mjs');
