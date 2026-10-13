// Natural free Chest opening through actual source and production public input.
import { fileURLToPath } from 'node:url';
process.argv.push('--scenario-helper', fileURLToPath(new URL('../original/chest-scenarios.py', import.meta.url)));
process.argv.push('--state-observer', fileURLToPath(new URL('../original/chest-observe.py', import.meta.url)));
// This is the independently declared public time(NULL) input to both games,
// matching the native runner's clock environment, not observed MT state.
process.argv.push('--boot-epoch-seed', '1700000000');
if (!process.argv.includes('--count')) process.argv.push('--count', '2');
if (!process.argv.includes('--output'))
  process.argv.push('--output', fileURLToPath(new URL(`../../.cache/e2e/chest-opening/${Date.now()}-${process.pid}`, import.meta.url)));
await import('./traversal.mjs');
