// Diagnostic shared-MT draw comparator for paired runs (`--mt-draws`).
//
// The original side is the record stream of tools/original/foundation-mt-trace.c;
// the port side is the WASM draw log (window.game.mtDraws). Both streams are
// reduced to gameplay draws, each with its update counter and wrapper kind
// (B bounded, U uniform, R raw u32), and walked in order. The first draw whose
// kind differs, or whose counter differs by more than one, is the first
// unported or misordered consumer: everything before it consumed the same
// stream positions in the same way. Native counters are Game+ce9f4, which is
// the port's game update count plus one inside full updates; the run loader's
// extra native update (counter 1) is replayed by the port at count 1, hence
// the tolerance of one. This is a diagnostic: it never changes a case status.
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

// Innermost return addresses of the source draw wrappers 42e8a0/42e850/42e8c0.
const WRAPPERS = new Map([[0x42e8b5n, 'B'], [0x42e859n, 'U'], [0x42e8c9n, 'R']]);
const hex = value => '0x' + value.toString(16);

export function readNativeDraws(directory) {
  const files = readdirSync(directory).filter(name => /^trace-\d+\.bin$/.test(name));
  if (files.length !== 1) throw new Error(`Expected one MT trace in ${directory}, found ${files.length}`);
  const bytes = readFileSync(join(directory, files[0]));
  const draws = [];
  for (let offset = 0; offset + 64 <= bytes.length; offset += 64) {
    const word = index => bytes.readBigUInt64LE(offset + index * 8);
    if (word(0) !== 0x44n) continue;
    const stamp = word(1), counter = Number(stamp >> 32n), index = Number(stamp & 0xffffffffn);
    const frames = [2, 3, 4, 5, 6, 7].map(word).filter(value => value !== 0n);
    const kind = WRAPPERS.get(frames[0]) ?? 'R';
    // Wrapper draws are attributed to their caller; direct draws to themselves.
    const callers = WRAPPERS.has(frames[0]) ? frames.slice(1) : frames;
    draws.push({ counter, index, kind, site: callers.slice(0, 3).map(hex).join('<') });
  }
  return draws;
}

const gameplay = draws => draws.filter(draw => draw.counter >= 1 && draw.counter < 0xffffffff);
// The port's game update count in native Game+ce9f4 terms.
const nativeCounter = count => count + 1;

// `independent` ({ label, gameUpdateCount }) marks where a case's games start
// playing their own keys (its first pairedIndependent scope): later draws are
// per-game and are not compared, so they cannot read as a divergence.
export function compareMtDraws(nativeDirectory, portDraws, context = 6, independent) {
  const native = gameplay(readNativeDraws(nativeDirectory));
  const port = portDraws.filter(draw => draw.counter >= 1).map(draw => ({ ...draw, counter: nativeCounter(draw.counter) }));
  const boundary = independent ? nativeCounter(independent.gameUpdateCount) : Infinity;
  const shared = draws => draws.findIndex(draw => draw.counter > boundary);
  const end = side => { const index = shared(side); return index < 0 ? side.length : index; };
  const length = Math.min(end(native), end(port));
  let divergence = -1;
  for (let i = 0; i < length; i++) {
    if (native[i].kind !== port[i].kind || Math.abs(native[i].counter - port[i].counter) > 1) { divergence = i; break; }
  }
  const result = {
    status: 'diagnostic',
    scope: 'Gameplay shared-MT draws compared by order, wrapper kind and update counter; never gates a case',
    nativeDraws: native.length, portDraws: port.length,
    startIndex: { native: native[0]?.index ?? null, port: port[0]?.index ?? null },
    agreedDraws: divergence < 0 ? length : divergence,
    ...(independent && { independentAfter: { ...independent, nativeCounter: boundary,
      scope: 'Draws after this capture come from per-game inputs and are not compared' } }),
  };
  if (divergence >= 0) {
    result.firstDivergence = {
      draw: divergence, nativeCounter: native[divergence].counter, portCounter: port[divergence].counter,
      context: native.slice(Math.max(0, divergence - context), divergence + context).map((draw, offset) => {
        const at = Math.max(0, divergence - context) + offset;
        return { draw: at, native: draw, port: port[at] ?? null };
      }),
    };
  } else if (!independent && native.length !== port.length) {
    result.unmatchedTail = { side: native.length > port.length ? 'native' : 'port',
      draws: (native.length > port.length ? native : port).slice(length, length + context) };
  }
  return result;
}
