// Preserve the full actual trace without transferring its complete object graph
// into Node. The production snapshot API and retained browser history stay intact.
import { createReadStream, createWriteStream } from 'node:fs';
import { createGunzip, createGzip } from 'node:zlib';
import { Writable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { createHash } from 'node:crypto';
import { once } from 'node:events';

export async function writeBrowserArtifactWithTrace(page, path, fields = {}) {
  if (Object.hasOwn(fields, 'trace')) throw Error('Trace must come from the actual production snapshot');
  const trace = await page.evaluateHandle(() => window.game.trace());
  let output, closed;
  try {
    const projection = await trace.evaluate(value => ({ initialSeed: value.initialSeed, bootEpochSeed: value.bootEpochSeed,
      enabled: value.enabled, inputTraceTruncated: value.inputTraceTruncated, commands: value.commands,
      truncated: value.truncated, stateHistoryTruncated: value.stateHistoryTruncated,
      stateHistoryTruncationReason: value.stateHistoryTruncationReason,
      retainedStates: value.states.length, backendEventCount: value.backendEvents.length,
      backendEventsHaveState: value.backendEvents.every(event => event.state), stateEncodedBytes: value.stateEncodedBytes, maxStateEncodedBytes: value.maxStateEncodedBytes }));
    const keys = await trace.evaluate(value => Object.keys(value));
    const hash = createHash('sha256'); let bytes = 0;
    output = createWriteStream(path, { flags: 'wx' });
    closed = once(output, 'close');
    // The write path also awaits errors; retain a handled completion promise
    // while it is still producing the artifact.
    closed.catch(() => {});
    // Awaiting each write applies the stream's backpressure. Only one serialized
    // actual array element crosses the browser boundary at a time.
    const write = async text => {
      if (typeof text !== 'string') throw Error('Actual trace value has no JSON representation');
      hash.update(text); bytes += Buffer.byteLength(text);
      await new Promise((resolve, reject) => output.write(text, error => error ? reject(error) : resolve()));
    };
    await write('{');
    for (const [key, value] of Object.entries(fields)) await write(JSON.stringify(key) + ':' + JSON.stringify(value) + ',');
    await write('"trace":{');
    for (let k = 0; k < keys.length; k++) {
      const key = keys[k]; if (k) await write(','); await write(JSON.stringify(key) + ':');
      const shape = await trace.evaluate((value, key) => ({ array: Array.isArray(value[key]), length: Array.isArray(value[key]) ? value[key].length : null }), key);
      if (!shape.array) await write(await trace.evaluate((value, key) => JSON.stringify(value[key]), key));
      else {
        await write('[');
        for (let index = 0; index < shape.length; index++) {
          if (index) await write(',');
          await write(await trace.evaluate((value, { key, index }) => JSON.stringify(value[key][index]), { key, index }));
        }
        await write(']');
      }
    }
    await write('}}\n'); output.end(); await closed;
    return { projection, archive: { path, bytes, sha256: hash.digest('hex'), complete: true,
      scope: 'Full actual trace and supplied artifact fields; array order and every retained state/backend event preserved' } };
  } finally {
    if (output && !output.closed) output.destroy();
    if (closed) await closed.catch(() => {});
    await trace.dispose();
  }
}

// Retain all actual API rows, including stereo music PCM, outside the Node graph.
// The semantic consumer receives the same mono Source.start rows it already uses.
export async function archiveBrowserAudioApiWithSfxProjection(page, path) {
  const api = await page.evaluateHandle(async () => {
    await Promise.all(window.__audioApi.pending);
    // Freeze the actual endpoint once. Timed ended events may keep arriving;
    // they must not change an array while its earlier snapshot is serialized.
    return structuredClone({ events: window.__audioApi.events, truncated: window.__audioApi.truncated });
  });
  let output, destination, completion;
  try {
    const projection = await api.evaluate(value => ({
      events: value.events.filter(event => event.type === 'start' && event.channels === 1),
      truncated: value.truncated,
    }));
    const count = await api.evaluate(value => value.events.length);
    const hash = createHash('sha256'); let rawBytes = 0;
    output = createGzip(); destination = createWriteStream(path, { flags: 'wx' });
    completion = pipeline(output, destination); completion.catch(() => {});
    const write = async text => {
      if (typeof text !== 'string') throw Error('Actual audio API value has no JSON representation');
      hash.update(text); rawBytes += Buffer.byteLength(text);
      try {
        await new Promise((resolve, reject) => output.write(text, error => error ? reject(error) : resolve()));
      } catch (writeError) {
        // A downstream filesystem error can destroy gzip before this callback.
        // Its already-created pipeline completion retains the actual cause.
        try { await completion; } catch (deliveryError) { throw deliveryError; }
        throw writeError;
      }
    };
    const chunkCharacters = 65536;
    await write('{"events":[');
    for (let index = 0; index < count; index++) {
      if (index) await write(','); await write('{');
      const keys = await api.evaluate((value, index) => Object.keys(value.events[index]), index);
      let written = 0;
      for (const key of keys) {
        const shape = await api.evaluate((value, { index, key }) => {
          const field = value.events[index][key], type = typeof field;
          return { omitted: type === 'undefined' || type === 'function' || type === 'symbol',
            string: type === 'string', length: type === 'string' ? field.length : null };
        }, { index, key });
        if (shape.omitted) continue; // Same object-property law as JSON.stringify.
        if (written++) await write(','); await write(JSON.stringify(key) + ':');
        if (shape.string) {
          // A music PCM base64 string can be large. Escape bounded actual slices
          // rather than duplicate or transfer that whole string into Node.
          await write('"');
          for (let start = 0; start < shape.length; start += chunkCharacters) {
            await write(await api.evaluate((value, { index, key, start, length }) =>
              JSON.stringify(value.events[index][key].slice(start, start + length)).slice(1, -1),
            { index, key, start, length: chunkCharacters }));
          }
          await write('"');
        } else {
          // Other fields are serialized one field at a time. Even their JSON
          // token crosses the boundary in bounded slices, never an API tree.
          const token = await api.evaluateHandle((value, { index, key }) => JSON.stringify(value.events[index][key]), { index, key });
          try {
            const length = await token.evaluate(value => value.length);
            for (let start = 0; start < length;) {
              const piece = await token.evaluate((value, { start, length }) => {
                let end = Math.min(start + length, value.length);
                // Encode a UTF-16 pair together when its JSON token crosses a
                // chunk boundary. Separate UTF-8 writes would replace halves.
                if (end < value.length && value.charCodeAt(end - 1) >= 0xd800 && value.charCodeAt(end - 1) <= 0xdbff &&
                    value.charCodeAt(end) >= 0xdc00 && value.charCodeAt(end) <= 0xdfff) end--;
                return { text: value.slice(start, end), end };
              }, { start, length: chunkCharacters });
              await write(piece.text); start = piece.end;
            }
          } finally { await token.dispose(); }
        }
      }
      await write('}');
    }
    await write('],"truncated":'); await write(await api.evaluate(value => JSON.stringify(value.truncated)));
    await write('}\n'); output.end(); await completion;
    const rawSha256 = hash.digest('hex');
    const readback = createHash('sha256'); let restoredBytes = 0;
    await pipeline(createReadStream(path), createGunzip(), new Writable({
      write(chunk, _encoding, done) { restoredBytes += chunk.length; readback.update(chunk); done(); }
    }));
    if (restoredBytes !== rawBytes || readback.digest('hex') !== rawSha256)
      throw Error('Full actual browser audio API gzip readback differs');
    return { projection, archive: { path, encoding: 'gzip', rawBytes, rawSha256, complete: true,
      eventCount: count, storageFormat: 'full-browser-audio-api-json-gzip-v1',
      scope: 'Complete actual API events/truncated snapshot, including all stereo music PCM and mono SFX fields. Only the existing mono Source.start semantic projection is materialized in Node; no event or field is discarded.' } };
  } finally {
    if (output && !output.destroyed) output.destroy();
    if (destination && !destination.closed) destination.destroy();
    if (completion) await completion.catch(() => {});
    await api.dispose();
  }
}
