// Source room-clear, natural acquisition and ordinary library key spending.
import { fileURLToPath } from 'node:url';
process.argv.push('--scenario-helper', fileURLToPath(new URL('../original/key-door-scenarios.py', import.meta.url)));
process.argv.push('--state-observer', fileURLToPath(new URL('../original/key-door-observe.py', import.meta.url)));
if (!process.argv.includes('--output'))
  process.argv.push('--output', fileURLToPath(new URL(`../../.cache/e2e/key-doors/${Date.now()}-${process.pid}`, import.meta.url)));
await import('./traversal.mjs');
