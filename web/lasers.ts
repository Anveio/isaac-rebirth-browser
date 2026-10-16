import type { Assets } from "./animation";
import type { ActorSprite, AttackColor } from "./engine";
import type { GLMenuContext } from "./gl-menu";
import { sampleFrame, type Point, type Quad } from "./animation-math";
import laserData from "./npc-data.json";

type Vector = { x: number; y: number };
export interface LaserView {
  id: number; origin: "npcShootAngle" | "playerCharged" | "playerControlled" | "playerTechnology" | "playerTechnology2"; uninitialized: boolean | null;
  active: boolean; actor: string; beamLength: number;
  impact: {id: number; position: Vector; positionOffset: Vector; scale: Vector;
    rotation: number; actor: string; animation: string; animationClock: number; removed: boolean;
    sourceColor: AttackColor | null; currentColor: AttackColor | null} | null;
  entityType: number; variant: number; subtype: number;
  visible: boolean; position: Vector; renderOffset: Vector;
  lineEnabled: boolean; sourceColor: AttackColor; currentColor: AttackColor; sprite: ActorSprite;
  line: { points: Vector[] };
}

const ACTOR = laserData["gfx/007.001_thick red laser.anm2"];
const THIN_ACTOR = laserData["gfx/007.002_thin red laser.anm2"];
const SHEET = "gfx/effects/effect_018_lasereffects.png";
const f = Math.fround;
const add = (a: Point, b: Point): Point => [f(a[0] + b[0]), f(a[1] + b[1])];
const sub = (a: Point, b: Point): Point => [f(a[0] - b[0]), f(a[1] - b[1])];
const scale = (a: Point, multiplier: number): Point => [f(a[0] * multiplier), f(a[1] * multiplier)];
const midpoint = (a: Point, b: Point): Point => scale(add(a, b), .5);
const length = (a: Point) => f(Math.sqrt(f(f(a[0] * a[0]) + f(a[1] * a[1]))));

export interface LaserProjection {
  /** Actual logical source viewport; the caller shares its room projection. */
  width: number; height: number; pixelScale: number;
}

/**42EB00 snaps projected center points before479C20 adds beam widths. */
function project(point: Vector, viewport: LaserProjection): Point {
  const half = f(.5), ratio = f(.65);
  const x = f(f(f(point.x - 60) * ratio) + f(f(viewport.width - 338) * half));
  const y = f(f(f(point.y - 140) * ratio) + f(f(viewport.height - 182) * half));
  const snap = (value: number) => f(Math.floor(f(f(value * viewport.pixelScale) + half)) / viewport.pixelScale);
  return [snap(x), snap(y)];
}

interface BeamQuad { points: Quad; from: number; to: number; }

/**479C20 line-enabled/subtype1 branch: four corners per source segment,
 * closed endpoint replacement, then shared adjacent joins in source order. */
function beamQuads(laser: LaserView, width: number, xScale: number, yScale: number,
  viewport: LaserProjection): BeamQuad[] {
  const vertices: Point[] = [], points = laser.line.points;
  const offset: Point = scale([laser.renderOffset.x, laser.renderOffset.y], f(.65));
  const halfWidth = f(width * .5);
  for (let index = 1; index < points.length; index++) {
    const from = points[index - 1]!, to = points[index]!;
    const difference: Point = [f(to.x - from.x), f(to.y - from.y)];
    const magnitude = length(difference);
    const reciprocal = magnitude > 0 ? f(1 / magnitude) : 1;
    const normal: Point = [f(difference[1] * reciprocal), f(-difference[0] * reciprocal)];
    const wide = scale(normal, halfWidth);
    const side: Point = [f(wide[0] * xScale), f(wide[1] * yScale)];
    const start = add(project(from, viewport), offset), end = add(project(to, viewport), offset);
    vertices.push(add(start, side), sub(start, side), add(end, side), sub(end, side));
  }
  if (vertices.length === 0) return [];
  //47BDBC erases the final pair;47E700 inserts the first pair AT THE END.
  if (laser.subtype === 1)
    vertices.splice(vertices.length - 2, 2, vertices[0]!, vertices[1]!);
  const quads: BeamQuad[] = [];
  let distance = 0;
  for (let index = 0; index < vertices.length; index += 4) {
    if (index < vertices.length - 4) {
      vertices[index + 2] = midpoint(vertices[index + 2]!, vertices[index + 4]!);
      vertices[index + 3] = midpoint(vertices[index + 3]!, vertices[index + 5]!);
    }
    if (index > 0) {
      vertices[index] = vertices[index - 2]!;
      vertices[index + 1] = vertices[index - 1]!;
    }
    const start = midpoint(vertices[index]!, vertices[index + 1]!);
    const end = midpoint(vertices[index + 2]!, vertices[index + 3]!);
    const next = f(distance + length(sub(end, start)));
    quads.push({ points: [vertices[index]!, vertices[index + 1]!, vertices[index + 2]!, vertices[index + 3]!],
      from: distance, to: next });
    distance = next;
  }
  return quads;
}

/**The actual subtype1 has no endpoint layer0 draw. Remove does not hide it;
 * the owning manager's Uninit removes retired actors from this render query. */
export function drawPlayerLaser(context: GLMenuContext, assets: Assets,
  laser: LaserView, viewport: LaserProjection): void {
  if (!laser.visible) return;
  if (laser.entityType !== 7 || ![1, 2].includes(laser.variant) || ![0, 1].includes(laser.subtype) || !laser.lineEnabled)
    throw new Error("Laser renderer requires admitted Player7/1 or7/2 line ownership");
  if (!Number.isFinite(viewport.pixelScale) || viewport.pixelScale <= 0)
    throw new Error("Laser projection requires the actual positive display scale");
  if (laser.currentColor.sourceMiddle.some(value => value !== 0))
    throw new Error("Laser renderer's additional source color operators are not admitted");
  const actor = laser.variant === 2 ? THIN_ACTOR : ACTOR;
  const animation = actor.Animations.find(clip => clip.Name === laser.sprite.animation);
  if (!animation) throw new Error("Laser clip is missing from its authored actor");
  const beamLayer = actor.Layers.length > 1 ? 1 : 0;
  const layer = animation.LayerAnimations.find(layer => layer.LayerId === beamLayer);
  const frame = layer?.Visible ? sampleFrame(layer.Frames, laser.sprite.frame, animation.Loop) : undefined;
  if (!frame) throw new Error("Laser beam requires its actual cached authored beam layer");
  //sampleFrame already converts the authored percentage to its native f32
  //ratio. Consume that value once, independently of Sprite2C4 current scale.
  const xScale = f(frame.XScale * laser.sprite.scale.x);
  const yScale = f(frame.YScale * laser.sprite.scale.y);
  const tint = [f(frame.RedTint * laser.currentColor.tint[0]), f(frame.GreenTint * laser.currentColor.tint[1]),
    f(frame.BlueTint * laser.currentColor.tint[2]), f(frame.AlphaTint * laser.currentColor.tint[3])];
  const offsets = [f(frame.RedOffset + laser.currentColor.offsets[0]), f(frame.GreenOffset + laser.currentColor.offsets[1]),
    f(frame.BlueOffset + laser.currentColor.offsets[2])];
  for (const quad of beamQuads(laser, frame.Width!, xScale, yScale, viewport)) {
    //Native texture.virtual88(0,0) selectsGL_REPEAT. UV y is cumulative
    //screen-space arc length, not a crop-height stretch or padded stripe.
    context.drawTintedImageQuadRepeated(assets.image(SHEET), tint, offsets,
      [frame.XCrop!, quad.from, frame.Width!, f(quad.to - quad.from)], quad.points);
  }
  //47B654 admits the authored endpoint layer0 for subtype0 only. Keep its
  //source current line direction and original ANM frame separate from layer1.
  if (actor.Layers.length > 1 && laser.subtype === 0 && laser.line.points.length >= 2) {
    const end = laser.line.points.at(-1)!, before = laser.line.points.at(-2)!;
    const point = add(project(end, viewport), scale([laser.renderOffset.x, laser.renderOffset.y], f(.65)));
    context.save();
    context.translate(point[0], point[1]);
    context.rotate(Math.atan2(end.y - before.y, end.x - before.x) - Math.PI / 2);
    assets.draw(context.asCanvasContext(), laser.actor, laser.sprite.animation, 0, 0, laser.sprite.frame,
      { scale: .65, layers: [0], layerScales: { 0: [laser.sprite.scale.x, laser.sprite.scale.y] } });
    context.restore();
  }
}
