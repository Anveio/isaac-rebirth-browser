import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import {
  interpolationProgress, sampleFrame, normalizeAngle, interpolateAngle,
  scalePoint, rotatePoint, frameQuad,
} from '../../web/animation-math.ts';

const data = JSON.parse(readFileSync(new URL('../../web/scene-data.json', import.meta.url)));
const source = JSON.parse(readFileSync(new URL('../../references/isaac/foundation/animation-math/primitives.json', import.meta.url)));
const bits = value => new Uint32Array(new Float32Array([value]).buffer)[0];
const animation = record => data[record.actor].Animations.find(animation => animation.Name === record.animation);
const sourceQuad = (frame, root, position) => {
  let points = frameQuad(frame, root, position);
  // Native aligned quads round only after all actor transforms (ELF 0x5ea8e0).
  if (points[0][0] === points[2][0]) points = points.map(point => point.map(value => Math.sign(value) * Math.floor(Math.abs(value) * 2 + .5) / 2));
  return points;
};

test('authored interpolation accumulates the native f32 reciprocal and normalizes archive units before blending', () => {
  assert.equal(bits(interpolationProgress(3, 13)), 0x3e6c4ec6);
  assert.equal(bits(Math.fround(3 / 13)), 0x3e6c4ec5);
  const smoke = animation(source.smoke).LayerAnimations.find(layer => layer.LayerId === 2);
  const frame = sampleFrame(smoke.Frames, 4, false);
  assert.equal(bits(frame.XScale), 0x3f6c4ec5);
  assert.equal(bits(frame.YScale), 0x3f6c4ec5);
  assert.equal(bits(frame.AlphaTint), 0x3f1a7310);
  assert.equal(frame.AlphaTint, Math.fround(source.smoke.records['130'][0].rgba[3]));
});

test('authored degree interpolation preserves shortest paths and signed 180 endpoints', () => {
  assert.equal(normalizeAngle(360), 0); assert.equal(normalizeAngle(-360), 0);
  assert.equal(normalizeAngle(540), 180); assert.equal(normalizeAngle(-540), -180);
  assert.equal(interpolateAngle(350, 10, .5), 0); assert.equal(interpolateAngle(10, 350, .5), 0);
  assert.equal(interpolateAngle(170, -170, .5), 180); assert.equal(interpolateAngle(-170, 170, .5), -180);
});

test('native identity operations preserve subpixel corners without subtracting an unrelated large anchor', () => {
  const point = [Math.fround(.1), Math.fround(.2)], anchor = [1e20, 1e20];
  assert.deepEqual(scalePoint(point, anchor, 1, 1), point);
  assert.deepEqual(rotatePoint(point, anchor, 0), point);
});

test('all 75 original title primitives match sequential f32 geometry at 25 source-observed phases', () => {
  const title = animation(source.title);
  let quads = 0;
  for (const record of source.title.records) {
    const root = sampleFrame(title.RootAnimation, record.actorFrame, title.Loop);
    for (const expected of record.quads) {
      const layer = title.LayerAnimations.find(layer => layer.LayerId === expected.layer);
      const frame = sampleFrame(layer.Frames, record.actorFrame, title.Loop);
      assert.deepEqual(sourceQuad(frame, root, source.title.position), expected.corners.map(point => point.map(Math.fround)), `title P${record.presentation} layer${expected.layer}`);
      assert.equal(frame.AlphaTint, Math.fround(expected.rgba[3]));
      quads++;
    }
  }
  assert.equal(quads, 75);
});

test('fractional native actor positions select the floored f32 cached frame', () => {
  // ANM2::GetFrame/OverlayFrame call floorf before integer conversion. Use
  // original title frame12 geometry to check the selected interpolated sample.
  const title = animation(source.title), record = source.title.records.find(record => record.actorFrame === 12);
  const root = sampleFrame(title.RootAnimation, 12.875, title.Loop);
  for (const expected of record.quads) {
    const layer = title.LayerAnimations.find(layer => layer.LayerId === expected.layer);
    const frame = sampleFrame(layer.Frames, 12.875, title.Loop);
    assert.deepEqual(sourceQuad(frame, root, source.title.position), expected.corners.map(point => point.map(Math.fround)));
  }
});

test('all 42 original explosion-smoke primitives and alpha values match across independently held-out frames', () => {
  const smoke = animation(source.smoke);
  let quads = 0, heldOut = 0;
  const derived = new Set([130, 140, 141]);
  for (const [age, expected] of Object.entries(source.smoke.records)) {
    const tick = Math.floor((Number(age) - 121) / 2), root = sampleFrame(smoke.RootAnimation, tick, smoke.Loop);
    const frames = smoke.LayerAnimations.filter(layer => layer.LayerId !== 0)
      .map(layer => sampleFrame(layer.Frames, tick, smoke.Loop)).filter(frame => frame.Visible && frame.AlphaTint > 0);
    assert.equal(frames.length, expected.length, `smoke layer count at ${age}`);
    for (const [index, frame] of frames.entries()) {
      assert.deepEqual(sourceQuad(frame, root, source.smoke.position), expected[index].xy.map(point => point.map(Math.fround)), `smoke ${age}:${index}`);
      assert.equal(frame.AlphaTint, Math.fround(expected[index].rgba[3]), `smoke alpha ${age}:${index}`);
      quads++; if (!derived.has(Number(age))) heldOut++;
    }
  }
  assert.equal(quads, 42); assert.equal(heldOut, 38);
});
