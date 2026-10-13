// Lossless complete selected snapshots without one unbounded JSON string.
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { rename } from 'node:fs/promises';
import { Readable, Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { createGzip } from 'node:zlib';

export async function writeBrowserEvidence(path, {states, trace, expectedCommands}) {
  const rawHash = createHash('sha256'), archiveHash = createHash('sha256');
  let rawBytes = 0, archiveBytes = 0;
  async function* chunks() {
    yield '{"schema":1,"states":[';
    for (let index = 0; index < states.length; index++) {
      if (index) yield ',';
      yield JSON.stringify(states[index]);
    }
    yield '],"trace":'; yield JSON.stringify(trace);
    yield ',"expectedCommands":'; yield JSON.stringify(expectedCommands);
    yield '}\n';
  }
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
