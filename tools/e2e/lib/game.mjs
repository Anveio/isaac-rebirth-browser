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
