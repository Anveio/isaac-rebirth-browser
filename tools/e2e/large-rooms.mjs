// Authored tall-room geometry through actual original and production DOM input.
import { fileURLToPath } from 'node:url';
process.argv.push('--scenario-helper', fileURLToPath(new URL('../original/large-room-scenarios.py', import.meta.url)));
if (!process.argv.includes('--count')) process.argv.push('--count', '2');
if (!process.argv.includes('--output'))
  process.argv.push('--output', fileURLToPath(new URL(`../../.cache/e2e/large-rooms/${Date.now()}-${process.pid}`, import.meta.url)));
await import('./traversal.mjs');
