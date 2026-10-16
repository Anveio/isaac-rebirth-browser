// Machine-wide cap on concurrently running suites. Each suite drives a fresh
// original process and a Chromium page with the production build; three
// runners at once on a 4-core, 15 GB container crashed renderers ("Target
// crashed") and timed out page loads. Every run.mjs on the host shares these
// slot files, so separate runs queue instead of starving each other.
import { mkdirSync, openSync, closeSync, readFileSync, unlinkSync, writeSync } from 'node:fs';
import { totalmem, tmpdir } from 'node:os';
import { join } from 'node:path';

const DIRECTORY = join(tmpdir(), 'isaac-e2e-slots');
const SUITE_MEMORY = 6 * 2 ** 30;

export function hostSlotCount() {
  const configured = Number(process.env.E2E_HOST_SLOTS);
  return configured >= 1 ? Math.floor(configured) : Math.max(1, Math.floor(totalmem() / SUITE_MEMORY));
}

// An owner is its pid plus its start time (/proc stat field 22): /tmp outlives
// container restarts, after which a dead run's pid can name a new process.
const started = pid => { try { return readFileSync(`/proc/${pid}/stat`, 'utf8').split(') ')[1].split(' ')[19]; } catch { return ''; } };
const OWNER = `${process.pid} ${started(process.pid)}`;
const alive = owner => {
  const [pid, start] = owner.split(' ');
  if (!(Number(pid) > 0)) return false;
  try { process.kill(Number(pid), 0); } catch (error) { if (error.code !== 'EPERM') return false; }
  return !start || started(pid) === start;
};

function tryAcquire(path, retry = true) {
  try {
    const fd = openSync(path, 'wx');
    writeSync(fd, OWNER); closeSync(fd);
    return true;
  } catch (error) {
    if (error.code !== 'EEXIST') throw error;
  }
  // A slot whose owner died (killed run, container restart) is reclaimed.
  let owner = '';
  try { owner = readFileSync(path, 'utf8'); } catch {}
  if (alive(owner)) return false;
  try { unlinkSync(path); } catch {}
  return retry && tryAcquire(path, false);
}

const held = new Set();
process.once('exit', () => { for (const path of held) try { unlinkSync(path); } catch {} });

// Resolves with a release function once a slot is free.
export async function acquireHostSlot(label) {
  mkdirSync(DIRECTORY, { recursive: true });
  const count = hostSlotCount();
  let waited = false;
  for (;;) {
    for (let index = 0; index < count; index++) {
      const path = join(DIRECTORY, `slot-${index}`);
      if (tryAcquire(path)) {
        held.add(path);
        return () => { if (held.delete(path)) try { unlinkSync(path); } catch {} };
      }
    }
    if (!waited) console.log(`… ${label} waits for a host slot (${count} in use; set E2E_HOST_SLOTS to change)`);
    waited = true;
    await new Promise(done => setTimeout(done, 5000));
  }
}
