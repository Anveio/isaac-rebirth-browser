// Playtest sandbox: a run that starts with chosen items in a chosen Start Room.
//
// Generates a pickup-interactions preset (tools/original/pickup-interaction-
// scenarios.py: Challenge 1 rewritten as "Sandbox" with its starting items,
// starting card/rune and Start Room spawns, from the local original resources)
// and serves the production build with it. Open the printed URL and pick
// Challenges -> Sandbox.
//
//   npm run playtest:sandbox                         # sandbox-2x2: Blank Card + The Battery + Jera, 2x2 room
//   npm run playtest:sandbox -- --preset sandbox     # the same combo in the 1x1 Start Room
//   npm run playtest:sandbox -- --out FILE.json      # only write the resources JSON
//
// The page hook is `/?sandbox=URL` (web/main.ts): any JSON {challengesXml,
// specialRooms} works, e.g. next to a published playtest bundle. The same
// presets replay side by side against the original:
//   npm run playtest:video -- --case .cache/e2e/RUN/pickup-interactions/scenarios.json#sandbox-2x2-fixed
// The JSON derives from original resources: keep it local or in private artifacts.
import { execFileSync } from 'node:child_process';
import { createReadStream, existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const args = process.argv.slice(2);
const flag = (name, fallback) => { const at = args.indexOf(name); return at < 0 ? fallback : args[at + 1]; };
const preset = flag('--preset', 'sandbox-2x2');

const directory = join(ROOT, '.cache/playtest/sandbox', `${preset}-${Date.now()}`);
mkdirSync(join(directory, '..'), { recursive: true });
execFileSync('python3', ['tools/original/pickup-interaction-scenarios.py', '--output', directory, '--count', '0', '--only', preset],
  { cwd: ROOT, stdio: ['ignore', 'ignore', 'inherit'] });
const bank = JSON.parse(readFileSync(join(directory, 'scenarios.json'), 'utf8'));
const fixture = bank.cases[0].fixture;
const resources = {
  challengesXml: readFileSync(join(directory, fixture.authoredResourcePath), 'utf8'),
  specialRooms: Array.from(readFileSync(join(directory, fixture.specialRoomsPath))),
};
const json = join(directory, 'sandbox.json');
writeFileSync(json, JSON.stringify(resources));
const out = flag('--out');
if (out) {
  writeFileSync(resolve(out), JSON.stringify(resources));
  console.log(JSON.stringify({ preset, about: bank.cases[0].about, resources: resolve(out) }));
  process.exit(0);
}

const dist = resolve(flag('--production-directory', join(ROOT, 'dist')));
if (!existsSync(join(dist, 'index.html'))) throw new Error(`Missing ${dist}/index.html; run npm run build`);
const types = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8',
  '.json': 'application/json', '.wasm': 'application/wasm', '.png': 'image/png', '.ogg': 'audio/ogg', '.wav': 'audio/wav' };
const server = createServer((request, response) => {
  const path = decodeURIComponent(new URL(request.url, 'http://local').pathname);
  const file = path === '/sandbox.json' ? json : join(dist, normalize(path === '/' ? '/index.html' : path));
  if (!file.startsWith(dist) && file !== json) { response.writeHead(403).end(); return; }
  if (!existsSync(file) || !statSync(file).isFile()) { response.writeHead(404).end(); return; }
  response.writeHead(200, { 'Content-Type': types[extname(file)] ?? 'application/octet-stream' });
  createReadStream(file).pipe(response);
});
const port = Number(flag('--port', '5180'));
server.listen(port, '0.0.0.0', () => {
  console.log(`${bank.cases[0].about}`);
  console.log(`Open http://localhost:${port}/?sandbox=/sandbox.json and choose Challenges -> Sandbox (Ctrl-C stops).`);
});
