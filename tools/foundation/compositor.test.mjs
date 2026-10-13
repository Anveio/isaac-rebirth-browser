import assert from "node:assert/strict";
import test from "node:test";
import { blend, normalized, rgba8, sample, tint } from "../../web/compositor.ts";

test("an offscreen light gradient preserves OpenGL's observed alpha-squared channel", () => {
  // A half-transparent white gradient over the opaque blue ambient pass has
  // alpha .75 in the native framebuffer. Canvas source-over would return 1.
  assert.deepEqual(rgba8(blend([1, 1, 1, .5], normalized([26, 38, 64, 255]))), [141, 147, 160, 191]);
});

test("a destination-color light pass modulates RGB while retaining source alpha independently", () => {
  const output = rgba8(blend(tint(normalized([122, 129, 144, 193]), [.3, .3, .3, .3]), normalized([227, 198, 197, 255]), "destination-color", "one-minus-source-alpha"));
  assert.deepEqual(output, [208, 183, 186, 255]);
});

test("linear texture sampling interpolates straight RGBA rather than premultiplied hidden color", () => {
  const image = { width: 2, height: 1, data: [255, 255, 255, 0, 0, 0, 0, 255] };
  assert.deepEqual(sample(image, 1, .5, "linear"), [.5, .5, .5, .5]);
  assert.deepEqual(sample(image, 0, .5, "linear", "clamp"), [1, 1, 1, 0]);
  assert.deepEqual(sample(image, 0, .5, "linear", "repeat"), [.5, .5, .5, .5]);
});
