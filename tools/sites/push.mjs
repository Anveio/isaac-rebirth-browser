import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, realpathSync } from 'node:fs';
import { resolve } from 'node:path';

// The Sites plugin's source helper is absent in this executor. Preserve its
// stdin-only credential and exact pushed-source/archive contract here.
const checkout = realpathSync(process.argv[2]);
const archive = resolve(process.argv[3]);
const manifest = JSON.parse(readFileSync(resolve(checkout, '.openai/hosting.json'), 'utf8'));
if (!manifest.project_id || manifest.static?.directory !== 'dist') throw new Error('Expected a registered static Site');
if (!existsSync(resolve(checkout, 'dist/index.html'))) throw new Error('Prepare the Site before pushing');
if (!process.stdin.isTTY) throw new Error('Use a retained terminal for hidden credential input');
execFileSync('stty', ['-echo'], { stdio: ['inherit', 'pipe', 'pipe'] });
console.log('Ready for Site workflow JSON on stdin (input is hidden).');
let payload = '';
for await (const chunk of process.stdin) {
  payload += chunk;
  if (payload.includes('\n')) break;
  if (payload.length > 65_536) throw new Error('Credential input exceeds limit');
}
execFileSync('stty', ['echo'], { stdio: ['inherit', 'pipe', 'pipe'] });
const { credential } = JSON.parse(payload);
payload = '';
if (credential.auth_mode !== 'http_extra_header' || credential.branch !== 'main') throw new Error('Unsupported Sites Git contract');
const endpoint = new URL(credential.remote_url);
if (endpoint.protocol !== 'https:' || endpoint.username || endpoint.password || !credential.token) throw new Error('Invalid credential endpoint');
const authEnv = {
  ...process.env,
  GIT_TERMINAL_PROMPT: '0', GIT_CONFIG_COUNT: '1',
  GIT_CONFIG_KEY_0: `http.${endpoint.href}.extraheader`,
  GIT_CONFIG_VALUE_0: `Authorization: Bearer ${credential.token}`,
};
const run = (command, args, env = process.env) => {
  try { return execFileSync(command, args, { cwd: checkout, env, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim(); }
  catch (error) { throw new Error(`${command} failed: ${String(error.stderr ?? error.message).replaceAll(credential.token, '[redacted]')}`); }
};
if (!existsSync(resolve(checkout, '.git'))) {
  run('git', ['init', '-b', credential.branch]);
  run('git', ['config', 'user.name', 'Codex']);
  run('git', ['config', 'user.email', 'codex@localhost']);
  run('git', ['remote', 'add', 'origin', endpoint.href]);
}
if (run('git', ['remote', 'get-url', 'origin']) !== endpoint.href) throw new Error('Site source remote mismatch');
run('git', ['add', '.openai/hosting.json', 'dist']);
if (run('git', ['status', '--porcelain'])) run('git', ['commit', '-m', 'Update playable WASM build and verification status']);
const commit_sha = run('git', ['rev-parse', 'HEAD']);
run('git', ['push', 'origin', `HEAD:${credential.branch}`], authEnv);
const pushed = run('git', ['ls-remote', 'origin', `refs/heads/${credential.branch}`], authEnv).split(/\s+/)[0];
if (pushed !== commit_sha) throw new Error('Pushed Site source does not match local HEAD');
if (run('git', ['status', '--porcelain'])) throw new Error('Site source changed after push');
run('tar', ['-czf', archive, '.openai/hosting.json', 'dist']);
console.log(JSON.stringify({ project_id: manifest.project_id, commit_sha, archive, checkout_path: checkout }));
