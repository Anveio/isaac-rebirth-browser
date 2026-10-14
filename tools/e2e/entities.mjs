// Entity specs: one small module per authored room entity in ./entities/.
// suites.mjs turns each into a live state-pair suite `entity-<name>`, graded
// by tools/foundation/entity-state-pair.mjs over rooms that
// tools/original/entity-scenarios.py finds for it.
//
//   export default {
//     type: 12, variant: 0,                 // entities2.xml identity
//     about: 'one line for the suite list',
//     contrived: ['CODE:SLOT'],             // fixed seed + starting-room door slot (optional)
//     count: 2,                             // OS-random rooms discovered per run
//     required: ['admission', ...],         // lifecycle markers both games must reach
//     fields: (native, actor, phase) => [], // extra [field, native, port] rows (optional)
//   };
import { readdirSync } from 'node:fs';

const directory = new URL('./entities/', import.meta.url);

export const specNames = () => readdirSync(directory).filter(file => file.endsWith('.mjs')).map(file => file.slice(0, -4)).sort();

export async function loadSpec(name) {
  if (!specNames().includes(name)) throw new Error(`Unknown entity spec ${name}`);
  const spec = (await import(new URL(`${name}.mjs`, directory))).default;
  return { contrived: [], count: 2, required: ['admission'], ...spec, name,
    matches: (type, variant) => type === spec.type && variant === spec.variant };
}

export const entitySpecs = () => Promise.all(specNames().map(loadSpec));
