// Renders docs/items.md: one row per base Rebirth collectible and trinket,
// generated from the game's own items.xml/itempools.xml, the active-use
// inventory in docs/active-items.md and the curated tools/foundation/item-status.json.
//
//   npm run items:update   # rewrite docs/items.md
//   npm run items:check    # fail when docs/items.md is stale
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const resources = join(root, 'references/isaac/rebirth/resources');
const target = join(root, 'docs/items.md');
const STATUSES = ['missing', 'partial', 'complete'];
const SET_STATUSES = ['match', 'mismatch', 'untested'];

function attributes(text) {
  return Object.fromEntries([...text.matchAll(/([a-zA-Z]+)="([^"]*)"/g)].map(match => [match[1], match[2]]));
}

const unescape = text => text.replaceAll('&apos;', "'").replaceAll('&quot;', '"').replaceAll('&amp;', '&');
const cell = text => String(text ?? '').replaceAll('|', '\\|').replaceAll('\n', ' ');

function readItems() {
  const xml = readFileSync(join(resources, 'items.xml'), 'utf8').replace(/<!--[\s\S]*?-->/g, '');
  return [...xml.matchAll(/<(passive|active|familiar|trinket)\s([^>]*?)\/?>/g)].map(match => {
    const values = attributes(match[2]);
    return { kind: match[1], id: Number(values.id), name: unescape(values.name), cache: values.cache ?? '' };
  });
}

function readPools() {
  const xml = readFileSync(join(resources, 'itempools.xml'), 'utf8').replace(/<!--[\s\S]*?-->/g, '');
  const pools = new Map();
  for (const pool of xml.matchAll(/<Pool Name="([^"]+)">([\s\S]*?)<\/Pool>/g)) {
    for (const item of pool[2].matchAll(/<Item\s([^>]*?)\/>/g)) {
      const values = attributes(item[1]);
      const id = Number(values.Id);
      const entry = pools.get(id) ?? { weight: 0, pools: [] };
      entry.weight += Number(values.Weight);
      entry.pools.push(pool[1]);
      pools.set(id, entry);
    }
  }
  return pools;
}

// docs/active-items.md owns active-use status: | Id | Item | Max | Cooldown | Status | Notes |
function readActiveInventory() {
  const rows = new Map();
  for (const line of readFileSync(join(root, 'docs/active-items.md'), 'utf8').split('\n')) {
    const match = line.match(/^\| (\d+) \| [^|]+ \| [^|]+ \| [^|]+ \| (Complete|Partial|Missing) \| (.*) \|$/);
    if (!match) continue;
    const cases = [...match[3].matchAll(/`([a-z][a-z0-9]*(?:-[a-z0-9]+)+)`/g)].map(found => found[1]);
    rows.set(Number(match[1]), { status: match[2].toLowerCase(), case: cases.length ? cases.map(name => `\`${name}\``).join(', ') : '' });
  }
  // The active-items suite declares one fixed case per ACTIVES entry, phase by sorted index.
  const scenarios = readFileSync(join(root, 'tools/original/active-items-scenarios.py'), 'utf8');
  const declared = [...scenarios.match(/ACTIVES = \{([\s\S]*?)\}/)[1].matchAll(/(\d+): '([a-z0-9-]+)'/g)]
    .map(match => [Number(match[1]), match[2]]).sort((a, b) => a[0] - b[0]);
  declared.forEach(([id, slug], index) => {
    const row = rows.get(id);
    if (row && !row.case) row.case = `active-items \`${slug}-${id}-fixed-phase${index & 1}\``;
  });
  return rows;
}

// Literal ids the engine passes to its collectible queries. A reference is a
// formula read or a recorded frontier, never proof of behaviour by itself.
function readEngineReferences() {
  const collectibles = new Set();
  const trinkets = new Set();
  const directory = join(root, 'engine/src');
  const query = /\b(?:raw_count|temporary_count|have|temporary_present|owned_count|positive_owned_count|primary_permanent_presence)\(\s*(\d+)\s*\)|owned_counts\[(\d+)\]/g;
  for (const file of readdirSync(directory).filter(name => name.endsWith('.rs'))) {
    const source = readFileSync(join(directory, file), 'utf8');
    for (const match of source.matchAll(query)) collectibles.add(Number(match[1] ?? match[2]));
    for (const match of source.matchAll(/\bhas_trinket\(\s*(\d+)\s*\)/g)) trinkets.add(Number(match[1]));
  }
  return { collectibles, trinkets };
}

function issueLink(issue) {
  return issue ? `#${issue}` : '';
}

export function renderItems() {
  const items = readItems();
  const pools = readPools();
  const actives = readActiveInventory();
  const curated = JSON.parse(readFileSync(join(root, 'tools/foundation/item-status.json'), 'utf8'));
  const references = readEngineReferences();
  const rows = items.map(item => {
    const trinket = item.kind === 'trinket';
    const entry = (trinket ? curated.trinkets : curated.collectibles)[item.id] ?? {};
    const active = item.kind === 'active' ? actives.get(item.id) : undefined;
    if (item.kind === 'active' && !active) throw new Error(`active ${item.id} has no docs/active-items.md row`);
    const status = entry.status ?? active?.status ?? 'missing';
    if (!STATUSES.includes(status)) throw new Error(`item ${item.kind} ${item.id}: unknown status ${status}`);
    if (status === 'complete' && !(entry.case || active?.case)) throw new Error(`item ${item.id} is complete without a live case`);
    const referenced = (trinket ? references.trinkets : references.collectibles).has(item.id);
    const pool = trinket ? undefined : pools.get(item.id);
    let note = entry.note ?? (active ? 'See [active inventory](active-items.md).' : '');
    if (entry.docs) note = `${note} [Notes](${entry.docs}).`.trim();
    return { ...item, status, referenced, weight: pool?.weight ?? 0, pools: pool?.pools ?? [],
      case: entry.case ?? active?.case ?? '', issue: issueLink(entry.issue), note };
  });
  for (const [group, table] of [['collectible', curated.collectibles], ['trinket', curated.trinkets]]) {
    for (const id of Object.keys(table)) {
      if (!rows.some(row => (row.kind === 'trinket') === (group === 'trinket') && row.id === Number(id))) {
        throw new Error(`item-status.json names unknown ${group} ${id}`);
      }
    }
  }
  const kinds = ['passive', 'active', 'familiar', 'trinket'];
  const count = (list, status) => list.filter(row => row.status === status).length;
  const summary = kinds.map(kind => {
    const list = rows.filter(row => row.kind === kind);
    return `| ${kind} | ${list.length} | ${count(list, 'complete')} | ${count(list, 'partial')} | ${count(list, 'missing')} | ${list.filter(row => row.status === 'missing' && row.referenced).length} |`;
  });
  const totalReferenced = rows.filter(row => row.status === 'missing' && row.referenced).length;
  summary.push(`| **all** | ${rows.length} | ${count(rows, 'complete')} | ${count(rows, 'partial')} | ${count(rows, 'missing')} | ${totalReferenced} |`);

  const mechanisms = curated.mechanisms.map(mechanism => {
    if (!STATUSES.includes(mechanism.status)) throw new Error(`mechanism ${mechanism.id}: unknown status`);
    return `| \`${mechanism.id}\` | ${cell(mechanism.name)} | ${cell(mechanism.original)} | ${mechanism.status} | ${cell(mechanism.case)} | ${issueLink(mechanism.issue)} | ${cell(mechanism.note)} |`;
  });
  const byId = new Map(rows.filter(row => row.kind !== 'trinket').map(row => [row.id, row]));
  const sets = curated.sets.map(set => {
    if (!SET_STATUSES.includes(set.status)) throw new Error(`set ${set.items}: unknown status`);
    const names = set.items.map(id => {
      const row = byId.get(id);
      if (!row) throw new Error(`set names unknown collectible ${id}`);
      return `${id} ${row.name}`;
    });
    return `| ${cell(names.join(' + '))} | ${set.status} | ${cell(set.case)} | ${issueLink(set.issue)} | ${cell(set.note)} |`;
  });
  const setCounts = SET_STATUSES.map(status => `${curated.sets.filter(set => set.status === status).length} ${status}`).join(', ');
  const mechanismCounts = STATUSES.slice().reverse().map(status => `${curated.mechanisms.filter(mechanism => mechanism.status === status).length} ${status}`).join(', ');

  const table = kind => [
    '| Id | Name | Status | Live case | Issue | Pool weight | Engine refs | Notes |',
    '| --- | --- | --- | --- | --- | --- | --- | --- |',
    ...rows.filter(row => row.kind === kind).map(row => `| ${row.id} | ${cell(row.name)} | ${row.status} | ${cell(row.case)} | ${row.issue} | ${row.weight ? `${row.weight} (${row.pools.join(', ')})` : ''} | ${row.referenced ? 'yes' : ''} | ${cell(row.note)} |`),
  ].join('\n');

  return `# Items, trinkets and synergies

Generated by \`npm run items:update\` from the game's own \`items.xml\` and
\`itempools.xml\`, the [active inventory](active-items.md) and the curated
status in [item-status.json](../tools/foundation/item-status.json). Edit that
file (or \`active-items.md\` for actives), never this page; \`npm run
items:check\` (part of \`check:build\`) rejects a stale page.

Status: **missing** means no item-specific behaviour (pickup, inventory and
pool only); **partial** means some behaviour is ported or no live case passes
yet; **complete** means every behaviour is ported and the named live
original-vs-port case passes. A failing or unrun comparison is never complete.
*Engine refs* marks an id the engine passes to a collectible or trinket query
(a formula read or a recorded frontier); it is a lead, not evidence.
*Pool weight* sums the item's weight over every pool it appears in. The design
for porting composition will live in \`docs/item-synergies.md\`.

## Summary

| Kind | Total | Complete | Partial | Missing | Missing but engine refs |
| --- | --- | --- | --- | --- | --- |
${summary.join('\n')}

Synergy mechanisms: ${mechanismCounts}. Tested item sets: ${setCounts}.

## Synergy mechanisms

Synergies come from the original's composition rules, so parity is tracked per
mechanism first. A mechanism is complete only when it runs for every
inventory, matching the original in live cases.

| Mechanism | Name | Original | Status | Live case | Issue | Notes |
| --- | --- | --- | --- | --- | --- | --- |
${mechanisms.join('\n')}

## Tested item sets

Each set is granted in both games and graded per field. *match* requires a
passing live comparison.

| Items | Result | Live case | Issue | Notes |
| --- | --- | --- | --- | --- |
${sets.join('\n')}

## Passive collectibles

${table('passive')}

## Active collectibles

${table('active')}

## Familiars

${table('familiar')}

## Trinkets

${table('trinket')}
`;
}

if (process.argv.includes('--write')) {
  writeFileSync(target, renderItems());
  console.log('Updated docs/items.md from items.xml, itempools.xml, active-items.md and item-status.json');
} else if (process.argv.includes('--check')) {
  if (readFileSync(target, 'utf8') !== renderItems()) {
    console.error('docs/items.md is stale; run npm run items:update');
    process.exit(1);
  }
  console.log('docs/items.md agrees with its sources');
} else {
  console.log('items-coverage --write | --check');
}
