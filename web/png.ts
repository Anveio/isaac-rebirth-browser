export interface RawImage { width: number; height: number; pixels: Uint8Array; }

const CRC_TABLE = Uint32Array.from({ length: 256 }, (_, index) => {
  let value = index;
  for (let bit = 0; bit < 8; bit++) value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
  return value >>> 0;
});
function chunkCRC(bytes: Uint8Array): number {
  let value = 0xffffffff;
  for (const byte of bytes) value = CRC_TABLE[(value ^ byte) & 255]! ^ (value >>> 8);
  return (value ^ 0xffffffff) >>> 0;
}

function paeth(left: number, above: number, corner: number): number {
  const predictor = left + above - corner;
  const a = Math.abs(predictor - left), b = Math.abs(predictor - above), c = Math.abs(predictor - corner);
  return a <= b && a <= c ? left : b <= c ? above : corner;
}

/** Standard PNG decoding retains RGB beneath alpha zero for straight-alpha GL sampling. */
export async function decodePNG(bytes: Uint8Array): Promise<RawImage> {
  const signature = [137, 80, 78, 71, 13, 10, 26, 10];
  if (!signature.every((byte, index) => bytes[index] === byte)) throw new Error("Invalid PNG signature");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let width = 0, height = 0, depth = 0, color = 0, interlace = 0;
  let header = false, ended = false, dataEnded = false;
  let palette: Uint8Array | undefined, transparency: Uint8Array | undefined;
  const compressed: Uint8Array[] = [];
  for (let offset = 8; offset < bytes.length;) {
    if (offset + 12 > bytes.length) throw new Error("Truncated PNG chunk");
    const length = view.getUint32(offset); const end = offset + 12 + length;
    if (end > bytes.length) throw new Error("Truncated PNG payload");
    const name = String.fromCharCode(...bytes.subarray(offset + 4, offset + 8));
    const data = bytes.subarray(offset + 8, offset + 8 + length);
    if (chunkCRC(bytes.subarray(offset + 4, offset + 8 + length)) !== view.getUint32(offset + 8 + length)) throw new Error("PNG chunk CRC mismatch");
    if (!header && name !== "IHDR") throw new Error("PNG header must be the first chunk");
    if (name === "IHDR") {
      if (header || length !== 13) throw new Error("Invalid PNG header");
      header = true;
      width = view.getUint32(offset + 8); height = view.getUint32(offset + 12);
      depth = data[8]!; color = data[9]!; interlace = data[12]!;
      if (data[10] !== 0 || data[11] !== 0) throw new Error("Unsupported PNG compression/filter method");
    } else if (name === "PLTE") {
      if (palette || compressed.length || !length || length > 768 || length % 3) throw new Error("Invalid PNG palette chunk");
      palette = data;
    } else if (name === "tRNS") {
      if (transparency || compressed.length) throw new Error("Invalid PNG transparency chunk");
      transparency = data;
    } else if (name === "IDAT") {
      if (dataEnded) throw new Error("PNG image chunks must be consecutive");
      compressed.push(data);
    } else if (name === "IEND") {
      if (length || !compressed.length) throw new Error("Invalid PNG end chunk");
      ended = true;
    } else if ((bytes[offset + 4]! & 32) === 0) throw new Error(`Unsupported critical PNG chunk ${name}`);
    if (compressed.length && name !== "IDAT") dataEnded = true;
    offset = end;
    if (ended) {
      break;
    }
  }
  if (!header || !ended) throw new Error("PNG header or end chunk is missing");
  if (!width || !height || width * height > 16_777_216 || !compressed.length) throw new Error("Invalid PNG dimensions or image payload");
  if (interlace !== 0) throw new Error("Interlaced PNG assets are unsupported");
  if (![1, 2, 4, 8].includes(depth) || depth !== 8 && color !== 0 && color !== 3) throw new Error("Unsupported PNG bit depth");
  const channels = ({ 0: 1, 2: 3, 3: 1, 4: 2, 6: 4 } as Record<number, number>)[color];
  if (!channels) throw new Error("Unsupported PNG color type");
  const stride = Math.ceil(width * channels * depth / 8), bpp = Math.max(1, Math.ceil(channels * depth / 8));
  const packed = new Uint8Array(compressed.reduce((sum, part) => sum + part.length, 0));
  let at = 0; for (const part of compressed) { packed.set(part, at); at += part.length; }
  const compressedBuffer = new Uint8Array(packed).buffer;
  const stream = new Blob([compressedBuffer]).stream().pipeThrough(new DecompressionStream("deflate"));
  const inflated = new Uint8Array(await new Response(stream).arrayBuffer());
  if (inflated.length !== (stride + 1) * height) throw new Error("Invalid PNG scanline length");
  const scanlines = new Uint8Array(stride * height);
  for (let y = 0; y < height; y++) {
    const mode = inflated[y * (stride + 1)]!;
    if (mode > 4) throw new Error("Invalid PNG scanline filter");
    for (let x = 0; x < stride; x++) {
      const index = y * stride + x, left = x >= bpp ? scanlines[index - bpp]! : 0;
      const above = y ? scanlines[index - stride]! : 0, corner = y && x >= bpp ? scanlines[index - stride - bpp]! : 0;
      const prediction = mode === 0 ? 0 : mode === 1 ? left : mode === 2 ? above : mode === 3 ? Math.floor((left + above) / 2) : paeth(left, above, corner);
      scanlines[index] = inflated[y * (stride + 1) + x + 1]! + prediction;
    }
  }
  const pixels = new Uint8Array(width * height * 4), maximum = (1 << depth) - 1;
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const target = (y * width + x) * 4;
    const offset = y * stride + Math.floor(x * channels * depth / 8);
    const sample = depth === 8 ? scanlines[offset]! : (scanlines[offset]! >>> (8 - depth - (x * depth % 8))) & maximum;
    if (color === 6) pixels.set(scanlines.subarray(offset, offset + 4), target);
    else if (color === 2) {
      pixels.set(scanlines.subarray(offset, offset + 3), target); pixels[target + 3] = 255;
      if (transparency?.length === 6 && pixels[target] === transparency[1] && pixels[target + 1] === transparency[3] && pixels[target + 2] === transparency[5]) pixels[target + 3] = 0;
    } else if (color === 3) {
      if (!palette || sample * 3 + 2 >= palette.length) throw new Error("PNG palette index is invalid");
      pixels.set(palette.subarray(sample * 3, sample * 3 + 3), target); pixels[target + 3] = transparency?.[sample] ?? 255;
    } else {
      const gray = Math.round(sample * 255 / maximum);
      pixels[target] = gray; pixels[target + 1] = gray; pixels[target + 2] = gray;
      pixels[target + 3] = color === 4 ? scanlines[offset + 1]! : transparency?.length === 2 && sample === ((transparency[0]! << 8) | transparency[1]!) ? 0 : 255;
    }
  }
  return { width, height, pixels };
}

export async function loadRawPNG(url: string): Promise<RawImage> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`PNG load failed: ${response.status}`);
  try { return await decodePNG(new Uint8Array(await response.arrayBuffer())); }
  catch (error) { throw new Error(`PNG ${url}: ${error instanceof Error ? error.message : String(error)}`); }
}
