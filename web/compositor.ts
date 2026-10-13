/** Straight-alpha, RGBA8 operations used by OpenGL framebuffer passes. */
export type RGBA = [number, number, number, number];
export type BlendFactor = "zero" | "one" | "source-alpha" | "one-minus-source-alpha" | "destination-alpha" | "one-minus-destination-alpha" | "source-color" | "destination-color";
export interface Pixels { width: number; height: number; data: ArrayLike<number>; }
export type Filter = "nearest" | "linear";
export type Wrap = "clamp" | "repeat";

function index(coordinate: number, length: number, wrap: Wrap): number {
  return wrap === "repeat" ? ((coordinate % length) + length) % length : Math.max(0, Math.min(length - 1, coordinate));
}
function pixel(image: Pixels, x: number, y: number, wrap: Wrap): RGBA {
  const offset = (index(y, image.height, wrap) * image.width + index(x, image.width, wrap)) * 4;
  return [image.data[offset]! / 255, image.data[offset + 1]! / 255, image.data[offset + 2]! / 255, image.data[offset + 3]! / 255];
}

/** x/y are texture pixel coordinates: the first texel's centre is (.5, .5). */
export function sample(image: Pixels, x: number, y: number, filter: Filter = "nearest", wrap: Wrap = "clamp"): RGBA {
  if (filter === "nearest") return pixel(image, Math.floor(x), Math.floor(y), wrap);
  const left = Math.floor(x - .5), top = Math.floor(y - .5), dx = x - .5 - left, dy = y - .5 - top;
  const a = pixel(image, left, top, wrap), b = pixel(image, left + 1, top, wrap), c = pixel(image, left, top + 1, wrap), d = pixel(image, left + 1, top + 1, wrap);
  return a.map((value, channel) => (value * (1 - dx) + b[channel]! * dx) * (1 - dy) + (c[channel]! * (1 - dx) + d[channel]! * dx) * dy) as RGBA;
}

function factor(kind: BlendFactor, source: RGBA, destination: RGBA, channel: number): number {
  switch (kind) {
    case "zero": return 0;
    case "one": return 1;
    case "source-alpha": return source[3];
    case "one-minus-source-alpha": return 1 - source[3];
    case "destination-alpha": return destination[3];
    case "one-minus-destination-alpha": return 1 - destination[3];
    case "source-color": return source[channel]!;
    case "destination-color": return destination[channel]!;
  }
}

/** OpenGL factors apply to alpha as well as RGB, unlike Canvas source-over. */
export function blend(source: RGBA, destination: RGBA, sourceFactor: BlendFactor = "source-alpha", destinationFactor: BlendFactor = "one-minus-source-alpha"): RGBA {
  return source.map((value, channel) => Math.max(0, Math.min(1, value * factor(sourceFactor, source, destination, channel) + destination[channel]! * factor(destinationFactor, source, destination, channel)))) as RGBA;
}

export function tint(source: RGBA, modulation: RGBA): RGBA {
  return source.map((value, channel) => value * modulation[channel]!) as RGBA;
}

export function rgba8(color: RGBA): RGBA {
  return color.map(value => Math.max(0, Math.min(255, Math.round(value * 255)))) as RGBA;
}

export function normalized(color: RGBA): RGBA { return color.map(value => value / 255) as RGBA; }
