// Complete opt-in native manifest/journal reads without a giant JSON string.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createReadStream, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { createInterface } from 'node:readline';
import { Transform, Writable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { createGunzip } from 'node:zlib';

const counter = () => {
  const hash = createHash('sha256'); let bytes = 0;
  return { write(chunk) { hash.update(chunk); bytes += chunk.length; },
    result() { return [bytes, hash.digest('hex')]; } };
};
const counted = value => new Transform({ transform(chunk, _encoding, done) {
  value.write(chunk); done(null, chunk);
} });

export async function readNativeEvidence(manifestPath) {
  const evidence = JSON.parse(readFileSync(manifestPath + '.meta.json', 'utf8'));
  assert.equal(evidence.schema, 1); assert.equal(evidence.encoding, 'gzip');
  assert.equal(evidence.status, 'closed');
  const archive = counter(), raw = counter();
  await pipeline(createReadStream(manifestPath), counted(archive), createGunzip(),
    new Writable({ write(chunk, _encoding, done) { raw.write(chunk); done(); } }));
  assert.deepEqual(archive.result(), [evidence.archiveBytes, evidence.archiveSha256]);
  assert.deepEqual(raw.result(), [evidence.rawBytes, evidence.rawSha256]);
  const path = join(dirname(manifestPath), 'native-progress.jsonl.gz');
  const metadata = JSON.parse(readFileSync(path + '.meta.json', 'utf8'));
  assert.equal(metadata.schema, 1); assert.equal(metadata.encoding, 'gzip-members-jsonl');
  assert.equal(metadata.status, 'closed');
  const progressArchive = counter(), progressRaw = counter(), decoded = counted(progressRaw);
  let manifest, finished = false, records = 0;
  const parse = (async () => {
    for await (const line of createInterface({ input: decoded, crlfDelay: Infinity })) {
      records++; assert.equal(finished, false, 'Native progress after finish');
      const { kind, value } = JSON.parse(line);
      if (kind === 'start' && !manifest && records === 1) { manifest = value; continue; }
      assert.ok(manifest, 'Native progress requires its initial report');
      if (kind === 'acknowledgement') manifest.presentations.push(value);
      else if (kind === 'advanceComplete') {
        assert.equal(manifest.presentations.at(-1).presentedFrame, value.presentedFrame);
        manifest.presentations[manifest.presentations.length - 1] = value;
      } else if (kind === 'field' && ['bootstrapPresentedFrames', 'declaredOptionsKept', 'initialState'].includes(value.name)) manifest[value.name] = value.value;
      else if (['debugObservation', 'input', 'capture', 'audioHold', 'focusHandoff'].includes(kind)) {
        const field = { debugObservation: 'debugObservations', input: 'inputs', capture: 'frames',
          audioHold: 'hostAudioHolds', focusHandoff: 'focusHandoffs' }[kind];
        (manifest[field] ??= []).push(value);
      } else if (kind === 'finish') { Object.assign(manifest, value); finished = true; }
      else throw new Error('Unknown native progress record: ' + kind);
    }
  })();
  await Promise.all([parse, pipeline(createReadStream(path), counted(progressArchive), createGunzip(), decoded)]);
  assert.equal(finished, true); assert.equal(records, metadata.records);
  assert.deepEqual(progressArchive.result(), [metadata.committedArchiveBytes, metadata.archiveSha256]);
  assert.deepEqual(progressRaw.result(), [metadata.rawBytes, metadata.rawSha256]);
  // The independent Python native validator also reconstructs this journal
  // and requires its canonical bytes to equal the complete manifest's hash.
  return { manifest, evidence: { ...evidence, progress: metadata } };
}
