import type { Frame } from "./animation";

/** ANM2 stores percentages/bytes; the native actor stores normalized f32 values. */
export type SampledFrame = Frame;
export type Point = readonly [number, number];
export type Quad = readonly [Point, Point, Point, Point];

const f = Math.fround;
const fields = ["XPosition", "YPosition", "XScale", "YScale", "RedTint", "GreenTint", "BlueTint", "AlphaTint", "RedOffset", "GreenOffset", "BlueOffset", "Rotation"] as const;
const divisor = (key: typeof fields[number]) => key.endsWith("Scale") ? 100 : key.endsWith("Tint") || key.endsWith("Offset") ? 255 : 1;

/** Interpolate<Frame> (0x42331b) advances a rounded reciprocal, once per frame. */
export function interpolationProgress(cursor: number, delay: number): number {
  const step = f(1 / delay);
  let progress = 0;
  for (let index = 0; index < cursor; index++) progress = f(progress + step);
  return progress;
}

export function normalizeAngle(angle: number): number {
  angle = f(angle);
  if (Math.abs(angle) > 360) angle = f(angle % 360);
  if (angle > 180) return f(angle - 360);
  if (angle < -180) return f(angle + 360);
  return angle;
}

/** InterpolateAngle (0x42eeb0) preserves the signed endpoint at 180 degrees. */
export function interpolateAngle(a: number, b: number, progress: number): number {
  a = normalizeAngle(a); b = normalizeAngle(b);
  let delta = f(b - a);
  if (Math.abs(delta) > 180) delta = b > a ? f(b - f(a + 360)) : f(f(b + 360) - a);
  return normalizeAngle(f(a + f(delta * progress)));
}

export function normalizeFrame(frame: Frame, overrides: Partial<Frame> = {}): SampledFrame {
  const result = { ...frame, ...overrides };
  for (const key of fields) result[key] = f(result[key] / divisor(key));
  return result;
}

export function overrideFrame(frame: SampledFrame, overrides: Partial<Frame> = {}): SampledFrame {
  const result = { ...frame, ...overrides };
  for (const key of fields) if (overrides[key] !== undefined) result[key] = f(overrides[key]! / divisor(key));
  return result;
}

export function sampleFrame(frames: readonly Frame[], index: number, loop: boolean): SampledFrame | undefined {
  if (!frames.length) return undefined;
  // ANM2::GetFrame (0x40ff70) floors the stored f32 animation position.
  // Fractional playback speed selects cached integer frames, rather than
  // introducing another interpolation between already authored samples.
  index = Math.floor(f(index));
  const length = frames.reduce((total, frame) => total + frame.Delay, 0);
  if (length <= 0) return normalizeFrame(frames[0]!);
  let cursor = loop ? ((index % length) + length) % length : Math.max(0, Math.min(index, length - 1));
  for (const [position, authored] of frames.entries()) {
    if (cursor >= authored.Delay) { cursor -= authored.Delay; continue; }
    const result = normalizeFrame(authored), next = frames[position + 1];
    if (!authored.Interpolated || !next) return result;
    const progress = interpolationProgress(cursor, authored.Delay), target = normalizeFrame(next);
    for (const key of fields) {
      const a = result[key], b = target[key];
      // ColorMod::Lerp (0x42f6b0) uses weighted endpoints; vectors use a delta.
      result[key] = key === "Rotation" ? interpolateAngle(a, b, progress)
        : key.endsWith("Tint") || key.endsWith("Offset") ? f(f(a * f(1 - progress)) + f(b * progress))
        : f(a + f(f(b - a) * progress));
    }
    return result;
  }
  return normalizeFrame(frames.at(-1)!);
}

export function scalePoint(point: Point, anchor: Point, x: number, y: number): Point {
  if (x === 1 && y === 1) return point;
  return [f(f(f(point[0] - anchor[0]) * x) + anchor[0]), f(f(f(point[1] - anchor[1]) * y) + anchor[1])];
}

export function rotatePoint(point: Point, anchor: Point, degrees: number): Point {
  if (degrees === 0) return point;
  const radians = f(f(degrees) * f(Math.PI / 180));
  const cos = f(Math.cos(radians)), sin = f(Math.sin(radians));
  const x = f(point[0] - anchor[0]), y = f(point[1] - anchor[1]);
  return [f(f(f(x * cos) - f(y * sin)) + anchor[0]), f(f(f(x * sin) + f(y * cos)) + anchor[1])];
}

/** GetDestQuad/Scale/Rotate preserve each native intermediate rounding. */
export function frameQuad(frame: SampledFrame, root: SampledFrame, position: Point, scale = 1): Quad {
  const center: Point = [f(position[0] + root.XPosition), f(position[1] + root.YPosition)];
  const anchor: Point = [f(center[0] + frame.XPosition), f(center[1] + frame.YPosition)];
  const left = f(anchor[0] - (frame.XPivot ?? 0)), top = f(anchor[1] - (frame.YPivot ?? 0));
  const right = f((frame.Width ?? 0) + left), bottom = f((frame.Height ?? 0) + top);
  const points: Quad = [[left, top], [right, top], [left, bottom], [right, bottom]];
  return points.map(point => {
    point = scalePoint(point, anchor, frame.XScale, frame.YScale);
    point = scalePoint(point, center, f(root.XScale * scale), f(root.YScale * scale));
    point = rotatePoint(point, anchor, frame.Rotation);
    return rotatePoint(point, center, root.Rotation);
  }) as unknown as Quad;
}
