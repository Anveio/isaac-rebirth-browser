// Lossless complete selected snapshots without one unbounded JSON string.
import { createHash } from 'node:crypto';
import { createWriteStream, existsSync, readFileSync } from 'node:fs';
import { rename } from 'node:fs/promises';
import { resolve } from 'node:path';
import { Readable, Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { createGzip, gunzipSync } from 'node:zlib';

// One side's manifest of a paired case: the raw manifest.json, or the
// manifest.json.gz written when that side's evidence is archived.
export function manifestPath(directory, side) {
  const raw = resolve(directory, side, 'manifest.json');
  return existsSync(raw) ? raw : `${raw}.gz`;
}

export const readJsonEvidence = path =>
  JSON.parse((path.endsWith('.gz') ? gunzipSync(readFileSync(path)) : readFileSync(path)).toString('utf8'));

export async function writeJsonEvidence(path, document) {
  const rawHash = createHash('sha256'), archiveHash = createHash('sha256');
  let rawBytes = 0, archiveBytes = 0;
  async function* valueChunks(value) {
    if (value?.[Symbol.asyncIterator] || Array.isArray(value)) {
      yield '[';
      let first = true;
      for await (const row of value) {
        if (!first) yield ',';
        first = false;
        yield JSON.stringify(row);
      }
      yield ']';
    } else if (value && typeof value === 'object' &&
               Object.values(value).some(child => child?.[Symbol.asyncIterator])) {
      yield* objectChunks(value);
    } else yield JSON.stringify(value);
  }
  async function* objectChunks(value) {
    yield '{';
    let first = true;
    for (const [key, field] of Object.entries(value)) {
      if (field === undefined) continue;
      if (!first) yield ',';
      first = false;
      yield JSON.stringify(key) + ':';
      yield* valueChunks(field);
    }
    yield '}';
  }
  async function* chunks() {yield* objectChunks(document);yield '\n';}
  const raw = new Transform({transform(chunk, _encoding, done) {
    rawBytes += chunk.length; rawHash.update(chunk); done(null, chunk);
  }});
  const compressed = new Transform({transform(chunk, _encoding, done) {
    archiveBytes += chunk.length; archiveHash.update(chunk); done(null, chunk);
  }});
  const temporary = path + '.part';
  await pipeline(Readable.from(chunks(), {objectMode: false}), raw, createGzip({level: 1}),
    compressed, createWriteStream(temporary, {flags: 'wx'}));
  await rename(temporary, path);
  return {path, encoding: 'gzip', rawBytes, rawSha256: rawHash.digest('hex'),
    archiveBytes, archiveSha256: archiveHash.digest('hex'),
    scope: 'Every selected phase retains its full actual snapshot, native comparisons, complete public commands and backend callbacks. JSON is written in lossless chunks without truncating snapshots or creating one whole-cohort string. Separate trace caps are disclosed; decompress before replay or inspection'};
}

export async function writeBrowserEvidence(path, {states, trace, expectedCommands}) {
  return writeJsonEvidence(path, {schema: 1, states, trace, expectedCommands});
}
