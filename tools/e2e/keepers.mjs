// Actual authored Keeper actors; fresh public original and production DOM.
import { fileURLToPath } from 'node:url';
process.argv.push('--scenario-helper', fileURLToPath(new URL('../original/keeper-scenarios.py', import.meta.url)));
if (!process.argv.includes('--output'))
  process.argv.push('--output', fileURLToPath(new URL(`../../.cache/e2e/keepers/${Date.now()}-${process.pid}`, import.meta.url)));
await import('./traversal.mjs');
