// Entity specs: one small module per authored room entity in ./entities/.
// The `entities` suite (suites.mjs) grades every spec with
// tools/foundation/entity-state-pair.mjs over rooms and routes that
// tools/original/entities-scenarios.py finds; `entity-<name>` runs one alone.
//
//   export default {
//     type: 12, variant: 0,                 // entities2.xml identity (omit for a room-shape spec)
//     shape: [26, 7],                       // authored room size to enter (default 13x7)
//     roomType: 13,                         // enter a room of this authored type (typeless specs)
//     gridType: 1940,                       // enter a room that authors this grid id (typeless specs)
//     boss: true,                           // walk a recorded route into the floor's boss room
//                                           // (tools/original/boss-scenarios.py; contrived entries are CODE or CODE:FIGHT)
//     shop: true,                           // bomb a secret room for keys and walk into the shop
//                                           // (tools/original/shop-scenarios.py; contrived entries are CODE)
//     about: 'one line for the suite list',
//     contrived: ['CODE:SLOT'],             // fixed seed + starting-room door slot (optional)
//     count: 2,                             // OS-random rooms discovered per run
//     required: ['admission', ...],         // lifecycle markers both games must reach
//     fields: (native, actor, phase) => [], // extra [field, native, port] rows (optional)
//     roomFields: (source, actual, phase) => [], // extra room-level rows (optional)
//   };
import { readdirSync } from 'node:fs';

const directory = new URL('./entities/', import.meta.url);

export const specNames = () => readdirSync(directory).filter(file => file.endsWith('.mjs')).map(file => file.slice(0, -4)).sort();

export async function loadSpec(name) {
  if (!specNames().includes(name)) throw new Error(`Unknown entity spec ${name}`);
  const spec = (await import(new URL(`${name}.mjs`, directory))).default;
  return { contrived: [], count: spec.boss ? 1 : 2, required: spec.boss ? ['bossRoom', 'admission'] : ['admission'], ...spec, name,
    matches: (type, variant) => spec.type !== undefined && type === spec.type && variant === spec.variant };
}

export const entitySpecs = () => Promise.all(specNames().map(loadSpec));
