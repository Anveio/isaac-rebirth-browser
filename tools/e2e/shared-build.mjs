// Bind each CI matrix job to the production bytes checked by its build job.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const { values } = parseArgs({ options: {
  write: { type: 'boolean' }, verify: { type: 'boolean' },
  identity: { type: 'string' }, 'production-directory': { type: 'string' },
} });
assert.ok(values.write !== values.verify, 'Choose exactly one of --write or --verify');
const identityPath = resolve(values.identity || join(ROOT, 'shared/build-identity.json'));
const productionDirectory = resolve(values['production-directory'] || join(ROOT, 'dist'));
const sourceCommit = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: ROOT, encoding: 'utf8' }).trim();

async function productionIdentity(directory) {
  const paths = [];
  async function visit(path) {
    for (const entry of (await readdir(path, { withFileTypes: true })).sort((a, b) => a.name.localeCompare(b.name))) {
      const child = join(path, entry.name);
      assert.ok(!entry.isSymbolicLink(), 'Shared production artifacts cannot contain symlinks');
      if (entry.isDirectory()) await visit(child);
      else if (entry.isFile()) paths.push(child);
      else assert.fail(`Unsupported production artifact entry: ${child}`);
    }
  }
  await visit(directory);
  assert.ok(paths.includes(join(directory, 'index.html')), 'Shared production artifact requires index.html');
  const hash = createHash('sha256');
  let bytes = 0;
  for (const path of paths) {
    const contents = await readFile(path);
    hash.update(relative(directory, path).split(sep).join('/')).update('\0');
    hash.update(String(contents.length)).update('\0').update(contents);
    bytes += contents.length;
  }
  return { sha256: hash.digest('hex'), files: paths.length, bytes };
}

const production = await productionIdentity(productionDirectory);
const identity = { schema: 1, sourceCommit, production };
if (values.write) {
  await mkdir(dirname(identityPath), { recursive: true });
  await writeFile(identityPath, JSON.stringify(identity, null, 2) + '\n');
} else {
  assert.deepEqual(JSON.parse(await readFile(identityPath, 'utf8')), identity,
    'Downloaded production artifact must match the checked build and this exact checkout');
}
console.log(JSON.stringify({ status: 'pass', mode: values.write ? 'write' : 'verify', ...identity }));
