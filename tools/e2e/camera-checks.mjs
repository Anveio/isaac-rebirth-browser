// Functional source camera checks used by the live large-room public tapes.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { PNG } from 'pngjs';
import { roomArtRows } from '../foundation/entity-compare.mjs';
import { evaluateJson } from './lib/game.mjs';

const vector = value => [Math.fround(value.x), Math.fround(value.y)];
export function compareCamera(phaseAssert, state, contents, label) {
  const room = contents.room, actual = state.camera;
  const art = [];
  roomArtRows(art, contents, state);
  for (const row of art) {
    phaseAssert.ok(row.expected !== undefined && row.actual !== undefined, `${label}:${row.field}:available`);
    phaseAssert.deepEqual(row.actual, row.expected, `${label}:${row.field}`);
  }
  phaseAssert.equal(room.cameraStyle, 1, `${label}:source default CameraStyle1`);
  phaseAssert.ok(actual, `${label}:admitted source camera controller`);
  if (!actual) return null;
  const minimum = [480 - ((room.gridWidth + 2) * 26 + room.renderOrigin[0] * 2),
    270 - ((room.gridHeight + 2) * 26 + room.renderOrigin[1] * 2)];
  const scroll = vector(actual.scroll);
  for (let axis = 0; axis < 2; axis++) {
    phaseAssert.ok(Number.isFinite(scroll[axis]) && scroll[axis] >= minimum[axis] && scroll[axis] <= 0,
      `${label}:camera must stay inside source room extent on axis${axis}`);
    if ([minimum[axis], 0].includes(room.renderScroll[axis]))
      phaseAssert.equal(scroll[axis], room.renderScroll[axis], `${label}:source clamped camera endpoint axis${axis}`);
  }
  const source = room.cameraDiagnostics;
  if (room.gridWidth === 15 && room.gridHeight === 16)
    phaseAssert.ok(source, `${label}:every tall-room phase requires complete native camera histories`);
  if (source) {
    phaseAssert.deepEqual(vector(actual.lastPlayerPosition), source.lastPlayerPosition,
      `${label}:source camera freeze/admission last-player position`);
    phaseAssert.deepEqual(actual.histories.map(history => [history.capacity, history.samples.length, history.head]),
      source.histories.map(history => [history.capacity, history.size, history.head]),
      `${label}:complete source camera histories and both-phase lifecycle`);
    phaseAssert.deepEqual(actual.histories.map(history => history.samples.map(vector)),
      source.histories.map(history => history.samples), `${label}:complete source camera input history`);
    phaseAssert.deepEqual(source.position, room.renderScroll, `${label}:source camera/render-offset join`);
  }
  return {actual: scroll, native: room.renderScroll, completeHistoryCompared: Boolean(source),
    difference: scroll.map((value, axis) => value - room.renderScroll[axis]),
    scope: 'All phases require camera bounds/source endpoints; complete input-history/lifecycle fields are required on the tall-room portion with native readback. Interior scroll floats remain diagnostic'};
}

function skinComponent(image, anchor) {
  // The selected source Isaac has near-white skin. Read the actual canvas at
  // the independent native projected anchor; no renderer metadata substitutes
  // for visible pixels. Frame/outline equality remains a separate pixel gate.
  const left = Math.max(0, Math.floor(anchor[0] - 32));
  const right = Math.min(image.width, Math.ceil(anchor[0] + 32));
  const top = Math.max(0, Math.floor(anchor[1] - 72));
  const bottom = Math.min(image.height, Math.ceil(anchor[1] + 8));
  const pixels = new Set();
  for (let y = top; y < bottom; y++) for (let x = left; x < right; x++) {
    const [r, g, b] = image.data.subarray((y * image.width + x) * 4, (y * image.width + x) * 4 + 3);
    if (Math.min(r, g, b) > 140 && Math.max(r, g, b) - Math.min(r, g, b) < 40) pixels.add(y * image.width + x);
  }
  let largest = {pixels: 0, width: 0, height: 0};
  while (pixels.size) {
    const first = pixels.values().next().value;
    const pending = [first], component = [];
    pixels.delete(first);
    while (pending.length) {
      const index = pending.pop(); component.push(index);
      const x = index % image.width, y = Math.floor(index / image.width);
      for (const [nx, ny] of [[x - 1, y], [x + 1, y], [x, y - 1], [x, y + 1]]) {
        const neighbor = ny * image.width + nx;
        if (nx >= left && nx < right && ny >= top && ny < bottom && pixels.delete(neighbor)) pending.push(neighbor);
      }
    }
    if (component.length > largest.pixels) {
      const xs = component.map(index => index % image.width), ys = component.map(index => Math.floor(index / image.width));
      largest = {pixels: component.length, width: Math.max(...xs) - Math.min(...xs) + 1,
        height: Math.max(...ys) - Math.min(...ys) + 1};
    }
  }
  return largest;
}

export async function captureCameraVisibility(page, native, label, contents, directory) {
  const frame = native.frames.find(frame => frame.label === label);
  assert.ok(frame, 'Functional camera phase must have an actual original frame');
  const sourceBytes = await readFile(frame.path), source = PNG.sync.read(sourceBytes);
  const capture = await evaluateJson(page, () => window.game.capture());
  const browserBytes = Buffer.from(capture.png, 'base64'), candidate = PNG.sync.read(browserBytes);
  assert.deepEqual([candidate.width, candidate.height], [source.width, source.height]);
  const actor = contents.players[0], scroll = contents.room.renderScroll;
  const anchor = [(actor.position[0] * .65 + 32 + scroll[0]) * source.width / 480,
    (actor.position[1] * .65 - 46 + scroll[1]) * source.height / 270];
  assert.ok(anchor[0] > 0 && anchor[0] < source.width && anchor[1] > 0 && anchor[1] < source.height,
    'Independent source camera must keep Isaac inside its actual viewport');
  const sourceSkin = skinComponent(source, anchor), browserSkin = skinComponent(candidate, anchor);
  assert.ok(sourceSkin.pixels > 0, 'Selected original viewport must contain actual Isaac skin pixels');
  assert.ok(browserSkin.pixels >= sourceSkin.width,
    'Actual rendered browser viewport must contain Isaac at the source camera-projected position');
  assert.ok(browserSkin.width >= actor.collisionRadius * source.width / 480
    && browserSkin.height >= actor.collisionRadius * source.height / 270,
    'Functional viewport requires a visible Isaac-sized component, including when native lighting leaves only small bright eye highlights');
  const browserPath = join(directory, label + '-camera.png');
  await writeFile(browserPath, browserBytes);
  return {label, sourcePath: frame.path, browserPath, nativeAnchor: anchor, sourceSkin, browserSkin,
    scope: 'Functional actual-canvas visibility at the independent native camera anchor; full sprite, backdrop and frame pixel equality remain diagnostic'};
}
