// Build a self-contained playtest copy of the production game that fits
// Claude Artifact limits (page <=16MB, file <=15MB, <=511 files, <=64MB per
// publish call). It is a playtest convenience, not a verification artifact.
//
// Trade-offs, all confined to this bundle (dist/ and the gates are untouched):
// - Small assets (SFX WAVs, PNGs, the WASM engine) are concatenated into a few
//   pack files. An inline loader fetches them, then serves the original bytes
//   through fetch() and HTMLImageElement.src, so the runtime's pinned SFX
//   checks still see the authored bytes.
// - Music stays the authored Vorbis files (~183MB). Re-encoding was rejected:
//   decoders disagree on Vorbis end trimming, and the runtime pins each track's
//   bytes and decoded length. The whole bundle fits one Artifact version
//   (<=256MB) but needs several publish calls. Names are stable, so with
//   --previous a later publish resends only files whose hash changed.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises';
import { basename, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';
import { build } from 'vite';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const { values } = parseArgs({ options: {
  out: { type: 'string', default: join(ROOT, '.cache/playtest/bundle') },
  'pack-bytes': { type: 'string', default: String(12 * 1024 * 1024) },
  'skip-wasm': { type: 'boolean', default: false },
  previous: { type: 'string' },
  label: { type: 'string' },
} });
const OUT = resolve(values.out);
const PACK_BYTES = Number(values['pack-bytes']);
const RESOURCES = join(ROOT, 'references/isaac/rebirth/resources');
const LIMITS = { page: 16e6, file: 15e6, files: 511, publish: 60e6 };
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const git = (...args) => execFileSync('git', args, { cwd: ROOT, encoding: 'utf8' }).trim();
const commit = git('rev-parse', 'HEAD');
const shortSha = commit.slice(0, 7);
const dirty = git('status', '--porcelain', '--untracked-files=no') !== '';

if (!values['skip-wasm']) execFileSync('bash', ['tools/build-wasm.sh'], { cwd: ROOT, stdio: 'inherit' });

// Ship the authored music bytes under stable names so later publishes can
// leave unchanged tracks in place.
const catalogue = JSON.parse(await readFile(join(ROOT, 'web/audio-data.json'), 'utf8'));
const music = new Map();
for (const path of Object.keys(catalogue.assets).filter(path => path.endsWith('.ogg'))) {
  const name = `music-${basename(path, '.ogg').toLowerCase().replace(/[^a-z0-9]+/g, '-')}.ogg`;
  music.set(path, { name, bytes: await readFile(join(RESOURCES, path)) });
}
assert.equal(new Set([...music.values()].map(file => file.name)).size, music.size, 'Music names must stay unique');

function playtestMusic() {
  return {
    name: 'playtest-music',
    enforce: 'pre',
    resolveId(id) { if (id === 'virtual:original-music') return '\0playtest-music'; },
    load(id) {
      if (id !== '\0playtest-music') return;
      const entries = [...music].map(([path, file]) =>
        // The music runtime re-encodes each URL segment, so hand it an
        // origin-relative path rather than an absolute URL.
        `${JSON.stringify(path)}: new URL(import.meta.ROLLUP_FILE_URL_${this.emitFile({ type: 'asset', fileName: `assets/${file.name}`, source: file.bytes })}).pathname`);
      return `export default {${entries.join(',')}};`;
    },
  };
}

const viteOut = join(ROOT, '.cache/playtest/vite');
await build({
  configFile: join(ROOT, 'vite.config.ts'),
  base: './',
  logLevel: 'error',
  plugins: [playtestMusic()],
  build: { outDir: viteOut, emptyOutDir: true, chunkSizeWarningLimit: 1e5 },
});

// Classify the build: entry script and stylesheet, music files, packed assets.
const html = await readFile(join(viteOut, 'index.html'), 'utf8');
const entry = html.match(/<script type="module"[^>]*src="\.\/assets\/([^"]+\.js)"/)?.[1];
const css = html.match(/<link rel="stylesheet"[^>]*href="\.\/assets\/([^"]+\.css)"/)?.[1];
assert.ok(entry && css, 'Built index.html must reference one module entry and one stylesheet');
const assets = (await readdir(join(viteOut, 'assets'))).sort();
assert.deepEqual(assets.filter(name => name.endsWith('.js')), [entry], 'Playtest bundle expects a single JS chunk');
const musicNames = new Set([...music.values()].map(file => file.name));
const packed = assets.filter(name => name !== entry && name !== css && !musicNames.has(name));
const TYPES = { png: 'image/png', wav: 'audio/wav', wasm: 'application/wasm', ogg: 'audio/ogg', json: 'application/json' };

await rm(OUT, { recursive: true, force: true });
await mkdir(join(OUT, 'assets'), { recursive: true });
const packs = [];
const index = {};
let current = null;
for (const name of packed) {
  const bytes = await readFile(join(viteOut, 'assets', name));
  if (!current || current.size + bytes.length > PACK_BYTES) {
    // Artifacts serve only web media types; .wasm carries opaque bytes.
    current = { name: `assets/pack-${packs.length}.wasm`, parts: [], size: 0 };
    packs.push(current);
  }
  const type = TYPES[name.split('.').pop()];
  assert.ok(type, `No media type for packed asset ${name}`);
  index[name] = [packs.length - 1, current.size, bytes.length, type];
  current.parts.push(bytes);
  current.size += bytes.length;
}
for (const pack of packs) {
  const bytes = Buffer.concat(pack.parts);
  pack.sha256 = sha256(bytes);
  await writeFile(join(OUT, pack.name), bytes);
}
for (const file of music.values()) await writeFile(join(OUT, 'assets', file.name), file.bytes);
const script = await readFile(join(viteOut, 'assets', entry));
await writeFile(join(OUT, 'assets/game.js'), script);
const version = sha256(Buffer.concat([script, ...packs.map(pack => Buffer.from(pack.sha256))])).slice(0, 12);

const loader = await readFile(join(ROOT, 'tools/playtest/loader.js'), 'utf8');
const style = await readFile(join(viteOut, 'assets', css), 'utf8');
const built = new Date().toISOString();
const label = values.label || `${shortSha}${dirty ? '+dirty' : ''}`;
const page = `<title>Isaac Rebirth Playtest</title>
<style>${style}
:root{color-scheme:dark;background:#000}
body{background:#000;color:#d7ceba}
#playtest-bar{position:fixed;left:0;right:0;bottom:0;display:flex;flex-wrap:wrap;justify-content:center;gap:4px 16px;padding:6px max(16px,env(safe-area-inset-right,0px)) calc(6px + env(safe-area-inset-bottom,0px)) max(16px,env(safe-area-inset-left,0px));font:12px/1.4 ui-monospace,Menlo,Consolas,monospace;color:#a59d8c;background:rgba(0,0,0,.72);pointer-events:none}
#playtest-bar b{color:#e8dfc9;font-weight:600}
#playtest-start{position:absolute;inset:0;display:grid;place-items:center;background:rgba(0,0,0,.55);color:#efe6cf;font:600 18px/1.4 ui-monospace,Menlo,Consolas,monospace;border:0;cursor:pointer;width:100%}
#playtest-start:focus-visible{outline:2px solid #e8dfc9;outline-offset:-6px}
</style>
<main>
  <canvas id="game" width="960" height="540" tabindex="0" aria-label="Isaac game; Enter selects, WASD moves, arrow keys shoot"></canvas>
  <p id="status" role="status">Loading…</p>
  <button id="playtest-start" type="button" hidden>Click to play</button>
</main>
<div id="playtest-bar"><span><b>Enter</b> select</span><span><b>WASD</b> move</span><span><b>Arrows</b> shoot</span><span><b>E</b> bomb</span><span><b>Esc</b> pause</span><span>build ${label} · ${built.slice(0, 10)}</span></div>
<script>
window.__PLAYTEST__ = ${JSON.stringify({ commit, label, built, version, entry: `assets/game.js?v=${version}`, packs: packs.map(pack => `${pack.name}?v=${pack.sha256.slice(0, 12)}`), index })};
${loader}
</script>
`;
await writeFile(join(OUT, 'index.html'), page);

// Enforce the Artifact limits and plan publish batches: the page goes in the
// first call with every changed file that fits, unchanged files are skipped.
const files = [];
for (const name of (await readdir(join(OUT, 'assets'))).sort()) {
  const bytes = await readFile(join(OUT, 'assets', name));
  files.push({ path: `assets/${name}`, bytes: bytes.length, sha256: sha256(bytes) });
}
const pageBytes = Buffer.byteLength(page);
assert.ok(pageBytes <= LIMITS.page, `Page is ${pageBytes} bytes`);
for (const file of files) assert.ok(file.bytes <= LIMITS.file, `${file.path} is ${file.bytes} bytes`);
assert.ok(files.length + 1 <= LIMITS.files, `${files.length + 1} files exceed the Artifact file limit`);
const previous = values.previous ? JSON.parse(await readFile(resolve(values.previous), 'utf8')) : { assets: [] };
const published = new Map(previous.assets.map(file => [file.path, file.sha256]));
const changed = files.filter(file => published.get(file.path) !== file.sha256);
const batches = [];
let batch = { bytes: pageBytes, files: [] };
// Music the title screen prefetches at boot goes up with the page.
const rank = file => !file.path.startsWith('assets/music-') ? 0 : file.path.startsWith('assets/music-title-screen') ? 1 : 2;
for (const file of changed.sort((a, b) => rank(a) - rank(b) || a.path.localeCompare(b.path))) {
  if (batch.bytes + file.bytes > LIMITS.publish) { batches.push(batch); batch = { bytes: 0, files: [] }; }
  batch.files.push(file.path); batch.bytes += file.bytes;
}
batches.push(batch);
const manifest = {
  commit, label, dirty, built, version,
  page: { path: 'index.html', bytes: pageBytes },
  files: files.length + 1,
  bytes: pageBytes + files.reduce((sum, file) => sum + file.bytes, 0),
  packs: packs.map(pack => ({ path: pack.name, bytes: pack.size, entries: pack.parts.length })),
  music: { files: music.size, bytes: [...music.values()].reduce((sum, file) => sum + file.bytes.length, 0) },
  assets: files,
  publish: { batches, remove: [...published.keys()].filter(path => !files.some(file => file.path === path)) },
};
await writeFile(join(OUT, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
console.log(JSON.stringify({ out: OUT, commit, label, version, files: manifest.files, bytes: manifest.bytes, packs: manifest.packs,
  music: manifest.music, publishBatches: batches.map(item => ({ bytes: item.bytes, files: item.files.length })), remove: manifest.publish.remove }, null, 2));
