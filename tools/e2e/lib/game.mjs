// Drive the production game page from e2e drivers.
//
// State crosses the page boundary as the engine's own JSON text: Playwright's
// structured serializer takes ~4x longer than the step itself for a full
// snapshot, and every per-frame comparison pays that cost. Stepping never
// transfers state unless the caller asks for it.

export async function snapshot(page) {
  return JSON.parse(await page.evaluate(() => window.game.snapshotJson()));
}

export async function step(page, count = 1) {
  await page.evaluate(frames => { window.game.step(frames); }, count);
}

// Advance and read the resulting state in one round trip.
export async function stepSnapshot(page, count = 1) {
  return JSON.parse(await page.evaluate(frames => { window.game.step(frames); return window.game.snapshotJson(); }, count));
}

// Encodes a page value as JSON text when that text parses back to the same
// value, or returns null. -0 is spelled -0 (JSON.parse restores it); undefined,
// non-finite numbers, non-plain objects and the marker string itself are not
// representable and send the value through Playwright's serializer instead.
function exactJson(value) {
  const marker = '\u0000exact-json-negative-zero\u0000';
  let exact = true, negativeZero = false;
  let text;
  try {
    text = JSON.stringify(value, function (key, encoded) {
      const raw = this[key];
      if (raw === undefined || typeof raw === 'function' || typeof raw === 'symbol' || typeof raw === 'bigint') exact = false;
      else if (typeof raw === 'number') {
        if (!Number.isFinite(raw)) exact = false;
        else if (Object.is(raw, -0)) { negativeZero = true; return marker; }
      } else if (typeof raw === 'string') { if (raw.includes(marker)) exact = false; }
      else if (raw !== null && typeof raw === 'object' && !Array.isArray(raw)) {
        const prototype = Object.getPrototypeOf(raw);
        if (prototype !== Object.prototype && prototype !== null) exact = false;
      }
      return encoded;
    });
  } catch { return null; }
  if (!exact || typeof text !== 'string') return null;
  return negativeZero ? text.replaceAll(JSON.stringify(marker), '-0') : text;
}

// Evaluate in the page and bring the result back as JSON text. Playwright's
// object serializer walks the whole graph inside the page: a 34 MB trace
// takes ~25-30 s that way and ~1 s as text. The returned value is the same
// one page.evaluate would return; a value JSON cannot represent exactly
// falls back to the object serializer.
export async function evaluateJson(page, pageFunction, arg) {
  const handle = await page.evaluateHandle(pageFunction, arg);
  try {
    const text = await handle.evaluate(exactJson);
    if (text !== null) {
      try { return JSON.parse(text); } catch { /* not exact JSON; use the serializer */ }
    }
    return await handle.evaluate(value => value);
  } finally { await handle.dispose(); }
}

// The browser's complete replay trace (window.game.trace()).
export const readTrace = page => evaluateJson(page, () => window.game.trace());
