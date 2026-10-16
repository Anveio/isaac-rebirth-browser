import assert from 'node:assert/strict';
import { globSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
export const DST_REGISTRY = 'engine/dst/src/units/mod.rs';
const KINDS = { always: 'Always', sometimes: 'Sometimes', reachable: 'Reachable', unreachable: 'Unreachable' };

// Indexes the Antithesis DST registry and every declared property from
// source. These are declarations, not results; a run's report is the outcome.
export function discoverDstUnits({ root = ROOT } = {}) {
  const registry = readFileSync(resolve(root, DST_REGISTRY), 'utf8');
  const units = [...registry.matchAll(/name: "([a-z_]+)",\s*summary:\s*"([^"]+)"/g)].map(([, name, summary]) => ({ name, summary, properties: [] }));
  assert.ok(units.length > 0, `No DST units registered in ${DST_REGISTRY}`);
  const byName = new Map(units.map(unit => [unit.name, unit]));
  const seen = new Set();
  for (const source of [...globSync('engine/dst/src/**/*.rs', { cwd: root })].sort()) {
    const text = readFileSync(resolve(root, source), 'utf8');
    for (const match of text.matchAll(/props::(always|sometimes|reachable|unreachable)!\([\s\S]*?"([a-z_]+): ([^"]+)"/g)) {
      const [, macro, owner, claim] = match;
      const key = `${owner}: ${claim}`;
      if (seen.has(key)) continue;
      seen.add(key);
      if (owner === 'harness') continue;
      // The shipped-WASM differential checks game tapes.
      const unit = byName.get(owner === 'wasm' ? 'game' : owner);
      assert.ok(unit, `DST property "${key}" in ${source} names no registered unit`);
      unit.properties.push({ kind: KINDS[macro], message: key, source });
    }
  }
  for (const unit of units) assert.ok(unit.properties.length > 0, `DST unit ${unit.name} declares no properties`);
  return units;
}
