// Machine-wide suite slots. Linux flock serializes updated clients; the slot-N
// files retain the PID/start-time format understood by older frozen runners.
import { mkdirSync, openSync, closeSync, readFileSync, unlinkSync, writeSync, fstatSync, statSync, linkSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { totalmem, tmpdir } from 'node:os';
import { join } from 'node:path';

const DIRECTORY = join(tmpdir(), 'isaac-e2e-slots');
const SUITE_MEMORY = 6 * 2 ** 30;

export function hostSlotCount() {
  const configured = Number(process.env.E2E_HOST_SLOTS);
  return configured >= 1 ? Math.floor(configured) : Math.max(1, Math.floor(totalmem() / SUITE_MEMORY));
}

const started = pid => { try { return readFileSync(`/proc/${pid}/stat`, 'utf8').split(') ')[1].split(' ')[19]; } catch { return ''; } };
const OWNER = `${process.pid} ${started(process.pid)}`;
const alive = owner => {
  const [pid, start] = owner.split(' ');
  if (!(Number(pid) > 0)) return false;
  try {
    process.kill(Number(pid), 0);
    if (process.platform === 'linux') {
      try {
        const stat = readFileSync(`/proc/${pid}/stat`, 'utf8');
        // The command name may itself contain spaces or parentheses.
        const commandEnd = stat.lastIndexOf(')');
        if (stat.startsWith(`${pid} (`) && stat.slice(commandEnd + 1).startsWith(' Z ')) return false;
      } catch { /* Keep the existing kill and start-time checks without procfs. */ }
    }
  } catch (error) { if (error.code !== 'EPERM') return false; }
  return !start || started(pid) === start;
};
const sameInode = (a, b) => a.dev === b.dev && a.ino === b.ino;

// This persistent mutex is not a legacy slot. Never unlink it: flock locks the
// inode, while all clients, including older ones, still contend on slot-N.
function acquireMutex(path) {
  return new Promise((resolve, reject) => {
    const child = spawn('flock', ['--exclusive', '--nonblock', '--no-fork', path,
      'sh', '-c', 'printf "ready\\n"; read -r release'], { stdio: ['pipe', 'pipe', 'pipe'] });
    let ready = false, output = '', errorText = '';
    child.on('error', error => { if (!ready) reject(new Error('Host slots require Linux flock', { cause: error })); });
    child.stderr.on('data', data => { errorText = (errorText + data).slice(0, 2048); });
    child.stdout.on('data', data => {
      output += data;
      if (ready || !output.includes('\n')) return;
      if (output !== 'ready\n') { child.stdin.destroy(); reject(new Error('Invalid host-slot mutex handshake')); return; }
      ready = true;
      child.unref(); child.stdin.unref?.(); child.stdout.destroy(); child.stderr.destroy();
      resolve({ release() { child.stdin.destroy(); } });
    });
    child.once('exit', (code, signal) => {
      if (ready) return;
      child.stdin.destroy();
      if (code === 1) resolve(null);
      else reject(new Error(`Host-slot flock failed (${signal ?? code}): ${errorText}`));
    });
  });
}

function removeMatching(path, identity, owner) {
  let fd;
  try {
    fd = openSync(path, 'r');
    const actual = fstatSync(fd);
    if (!sameInode(actual, identity) || readFileSync(fd, 'utf8') !== owner) return;
    if (sameInode(statSync(path), actual)) unlinkSync(path);
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  } finally { if (fd !== undefined) closeSync(fd); }
}

async function tryAcquire(path) {
  const mutex = await acquireMutex(path + '.mutex');
  if (!mutex) return null;
  let acquired = false;
  try {
    let fd;
    try {
      fd = openSync(path, 'r');
      const identity = fstatSync(fd), owner = readFileSync(fd, 'utf8');
      if (alive(owner)) return null;
      removeMatching(path, identity, owner);
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    finally { if (fd !== undefined) closeSync(fd); }

    // Hard-link only after the complete legacy owner record is visible.
    const candidate = join(DIRECTORY, `.candidate-${process.pid}-${randomUUID()}`);
    let identity;
    try {
      fd = openSync(candidate, 'wx', 0o600);
      try { writeSync(fd, OWNER); identity = fstatSync(fd); }
      finally { closeSync(fd); }
      try { linkSync(candidate, path); }
      catch (error) { if (error.code === 'EEXIST') return null; throw error; }
    } finally { try { unlinkSync(candidate); } catch (error) { if (error.code !== 'ENOENT') throw error; } }
    acquired = true;
    return { release() { try { removeMatching(path, identity, OWNER); } finally { mutex.release(); } } };
  } finally { if (!acquired) mutex.release(); }
}

const held = new Set();
process.once('exit', () => { for (const slot of held) try { slot.release(); } catch {} });

export async function acquireHostSlot(label) {
  mkdirSync(DIRECTORY, { recursive: true });
  const count = hostSlotCount();
  let waited = false;
  for (;;) {
    for (let index = 0; index < count; index++) {
      const slot = await tryAcquire(join(DIRECTORY, `slot-${index}`));
      if (slot) {
        held.add(slot);
        return () => { if (held.delete(slot)) slot.release(); };
      }
    }
    if (!waited) console.log(`… ${label} waits for a host slot (${count} in use; set E2E_HOST_SLOTS to change)`);
    waited = true;
    await new Promise(done => setTimeout(done, 5000));
  }
}
