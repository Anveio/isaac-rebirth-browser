// Full-frame pixel diagnostics of paired checkpoints on a small worker pool.
// Each job decodes both PNGs, diffs them, writes the difference PNG and hashes
// the source bytes exactly as comparePair does inline; comparePair recomputes
// any label without a result here (no job, or a worker that died).
// E2E_PIXEL_WORKERS sets the pool size (default 2; 0 compares inline).
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { isMainThread, parentPort, Worker } from 'node:worker_threads';
import { PNG } from 'pngjs';
import { pixelDifference } from './compare.mjs';

export function comparePixelPair({ sourcePath, candidatePath, differencePath }) {
  try {
    const sourceBytes = readFileSync(sourcePath);
    const difference = pixelDifference(PNG.sync.read(sourceBytes), PNG.sync.read(readFileSync(candidatePath)));
    writeFileSync(differencePath, PNG.sync.write(difference.difference));
    const { difference: ignoredImage, ...pixels } = difference;
    return { pixels, sourcePNGHash: createHash('sha256').update(sourceBytes).digest('hex') };
  } catch (error) { return { error: String(error.message) }; }
}

export async function comparePixelPairs(jobs, workers = Number(process.env.E2E_PIXEL_WORKERS ?? 2)) {
  const results = new Map();
  const queue = [...new Map(jobs.map(job => [job.label, job])).values()];
  const size = Math.min(Number.isInteger(workers) && workers > 0 ? workers : 0, queue.length);
  const run = () => new Promise(done => {
    const worker = new Worker(new URL(import.meta.url));
    let current;
    const next = () => {
      current = queue.shift();
      if (current) worker.postMessage(current); else worker.terminate();
    };
    worker.on('message', result => { results.set(current.label, result); next(); });
    // A failed worker leaves its labels to comparePair's inline path.
    worker.on('error', () => {});
    worker.on('exit', done);
    next();
  });
  await Promise.all(Array.from({ length: size }, run));
  return results;
}

if (!isMainThread) parentPort.on('message', job => parentPort.postMessage(comparePixelPair(job)));
