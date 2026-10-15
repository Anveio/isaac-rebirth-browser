// Keep a recorded case cited as evidence (an issue, a PR) beyond the container
// that recorded it:
//
//   npm run case:keep -- <suite> <recorded case.json or scenarios.json> <name> [--case ID] [--force]
//
// The case is written to references/isaac/cases/<suite>/<name>.json and the
// command that replays it is printed. Commit it with the change or issue that
// cites it. Paired-game suites replay one case.json. Other drivers replay a
// scenarios.json: --case keeps only the cited case (by scenario.id or name),
// which drivers that require their whole case set may refuse; then keep the
// whole file. Files over 256 KB need --force, so kept cases stay small.
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { suiteById } from './suites.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const argv = process.argv.slice(2), at = flag => argv.indexOf(flag);
const caseId = at('--case') >= 0 ? argv.splice(at('--case'), 2)[1] : undefined;
const force = at('--force') >= 0 && Boolean(argv.splice(at('--force'), 1));
const [id, source, name] = argv;
const suite = suiteById.get(id);
if (!suite || !source || !/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(name ?? '')) {
  console.error('usage: npm run case:keep -- <suite> <case.json|scenarios.json> <name>   (name: letters, digits, . _ -)');
  process.exit(2);
}
const document = JSON.parse(readFileSync(resolve(source), 'utf8'));
const pairedGame = suite.entry === 'tools/foundation/paired-game.mjs';
if (pairedGame !== !Array.isArray(document.cases)) {
  console.error(`${id} replays ${pairedGame ? 'one recorded case.json' : 'its scenarios.json (a document with cases[])'}`);
  process.exit(2);
}
let text = readFileSync(resolve(source), 'utf8');
if (caseId !== undefined) {
  const cases = (document.cases ?? []).filter(item => (item.scenario?.id ?? item.name ?? item.id) === caseId);
  if (cases.length !== 1) { console.error(`No single case ${caseId} in ${source}`); process.exit(2); }
  text = JSON.stringify({ ...document, cases }, null, 2) + '\n';
}
if (Buffer.byteLength(text) > 256 * 1024 && !force) {
  console.error(`${Math.round(Buffer.byteLength(text) / 1024)} KB: keep only the cited case with --case ID, or pass --force`);
  process.exit(2);
}
const target = join(ROOT, 'references/isaac/cases', id, `${name}.json`);
if (existsSync(target)) { console.error(`${relative(ROOT, target)} already exists`); process.exit(2); }
mkdirSync(join(target, '..'), { recursive: true });
writeFileSync(target, text);
console.log(`kept ${relative(ROOT, target)}; replay with:`);
console.log(`  npm run test:e2e -- ${id} ${pairedGame ? '--replay-cases' : suite.replay || '--scenarios'} ${relative(ROOT, target)}`);
