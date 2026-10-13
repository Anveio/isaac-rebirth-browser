import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { crc32 } from 'node:zlib';
import { PNG } from 'pngjs';
import { decodePNG } from '../../web/png.ts';

test('PNG decoder preserves straight RGBA and transparent RGB through every standard row filter', async () => {
  const width = 7, height = 5, pixels = Buffer.alloc(width * height * 4);
  for (let pixel = 0; pixel < width * height; pixel++) {
    pixels.set([(pixel * 41) % 256, (pixel * 73 + 255) % 256, (pixel * 127 + 11) % 256, [0, 1, 128, 255][pixel % 4]], pixel * 4);
  }
  for (const filterType of [0, 1, 2, 3, 4]) {
    const encoded = PNG.sync.write({ width, height, data: pixels }, { colorType: 6, inputColorType: 6, filterType });
    const decoded = await decodePNG(encoded);
    assert.equal(decoded.width, width); assert.equal(decoded.height, height);
    assert.deepEqual(Buffer.from(decoded.pixels), pixels, `Row filter ${filterType}`);
  }
});

test('Authored texture PNGs match the independent decoder while keeping RGB beneath alpha zero', async () => {
  for (const asset of ['menuoverlay', 'menushadow', 'titlemenu', 'charactermenu']) {
    const encoded = await readFile(new URL(`../../references/isaac/rebirth/resources/gfx/ui/main menu/${asset}.png`, import.meta.url));
    let streamEnd = 8;
    while (streamEnd < encoded.length) {
      const type = encoded.toString('ascii', streamEnd + 4, streamEnd + 8);
      streamEnd += encoded.readUInt32BE(streamEnd) + 12;
      if (type === 'IEND') break;
    }
    const decoded = await decodePNG(encoded), reference = PNG.sync.read(encoded.subarray(0, streamEnd));
    assert.equal(decoded.width, reference.width); assert.equal(decoded.height, reference.height);
    assert.deepEqual(Buffer.from(decoded.pixels), reference.data, asset);
    if (asset === 'menuoverlay') {
      const center = (135 * decoded.width + 240) * 4;
      assert.deepEqual([...decoded.pixels.slice(center, center + 4)], [255, 255, 255, 0]);
    }
  }
});

test('Malformed PNG dimensions and chunks fail explicitly', async () => {
  await assert.rejects(decodePNG(new Uint8Array([137, 80, 78])), /PNG signature/);
  const encoded = PNG.sync.write({ width: 1, height: 1, data: Buffer.from([255, 255, 255, 0]) });
  await assert.rejects(decodePNG(encoded.subarray(0, 26)), /Truncated PNG/);
  const oversized = Buffer.from(encoded); oversized.writeUInt32BE(20_000_000, 16);
  oversized.writeUInt32BE(crc32(oversized.subarray(12, 29)), 29);
  await assert.rejects(decodePNG(oversized), /PNG dimensions/);
  const corrupted = Buffer.from(encoded); corrupted[20] ^= 1;
  await assert.rejects(decodePNG(corrupted), /CRC mismatch/);
  await assert.rejects(decodePNG(encoded.subarray(0, encoded.length - 12)), /end chunk is missing/);
});
