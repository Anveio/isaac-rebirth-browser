import sceneData from "./scene-data.json";
import fontData from "./font-data.json";
import completionUrl from "./assets/completion_widget.png?url";
import postitOverlapUrl from "./assets/gamemenu_postitoverlap.png?url";

export interface Frame {
  XCrop?: number; YCrop?: number; Width?: number; Height?: number;
  XPosition: number; YPosition: number; XScale: number; YScale: number;
  XPivot?: number; YPivot?: number; Delay: number; Visible: boolean;
  RedTint: number; GreenTint: number; BlueTint: number; AlphaTint: number;
  RedOffset: number; GreenOffset: number; BlueOffset: number; Rotation: number; Interpolated: boolean;
}
interface Animation { Name: string; FrameNum: number; Loop: boolean; RootAnimation: Frame[]; LayerAnimations: { LayerId: number; Visible: boolean; Frames: Frame[] }[]; }
interface Actor { BaseSpritesheetPath: string; Spritesheets: { Id: number; Path: string }[]; Layers: { Id: number; SpritesheetId: number; Name: string }[]; Animations: Animation[]; }
export interface DrawOptions { scale?: number; layers?: number[]; overrides?: Record<number, Partial<Frame>>; pixelSnap?: number; }
interface Glyph { x: number; y: number; width: number; height: number; xoffset: number; yoffset: number; xadvance: number; page: number; }
interface Font { lineHeight: number; pages: string[]; chars: Record<string, Glyph>; kernings: { first: number; second: number; amount: number }[]; }
const normalize = (path: string) => path.replaceAll("\\", "/").replace(/\/\.\//g, "/").replace(/\/\.$/, "").replace(/^resources\//i, "").toLowerCase();
const urls = import.meta.glob([
  "../references/isaac/rebirth/resources/gfx/ui/main menu/**/*.png",
  "../references/isaac/rebirth/resources/gfx/ui/backselectwidget.png",
  "../references/isaac/rebirth/resources/gfx/ui/ui_hearts.png",
  "../references/isaac/rebirth/resources/gfx/ui/hudpickups.png",
  "../references/isaac/rebirth/resources/gfx/ui/minimap1.png",
  "../references/isaac/rebirth/resources/gfx/ui/minimap2.png",
  "../references/isaac/rebirth/resources/gfx/ui/bgblack.png",
  "../references/isaac/rebirth/resources/gfx/ui/pausescreen.png",
  "../references/isaac/rebirth/resources/gfx/ui/menu_mystuff.png",
  "../references/isaac/rebirth/resources/gfx/ui/effect_024_streak.png",
  "../references/isaac/rebirth/resources/gfx/ui/effect_cursepaper.png",
  "../references/isaac/rebirth/resources/gfx/characters/costumes/character_001_isaac.png",
  "../references/isaac/rebirth/resources/gfx/characters/costumes/ghost.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/01_basement.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/controls.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/shading.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/light_gradient.png",
  "../references/isaac/rebirth/resources/gfx/overlays/basement/1x1_*.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_01_normaldoor.png",
  "../references/isaac/rebirth/resources/gfx/shadow.png",
  "../references/isaac/rebirth/resources/gfx/tears.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_015_tearpoof*.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_029_explosion.png",
  "../references/isaac/rebirth/resources/gfx/items/pick ups/pickup_016_bomb.png",
  "../references/isaac/rebirth/resources/font/*_0.png",
], { eager: true, query: "?url", import: "default" }) as Record<string, string>;

function frameAt(frames: Frame[], index: number, loop: boolean): Frame | undefined {
  if (!frames.length) return undefined;
  const length = frames.reduce((total, frame) => total + frame.Delay, 0);
  let cursor = loop ? index % length : Math.min(index, length - 1);
  for (const [i, frame] of frames.entries()) {
    if (cursor < frame.Delay) {
      const next = frames[i + 1];
      if (!frame.Interpolated || !next || frame.Delay === 0) return frame;
      const result = { ...frame }; const ratio = cursor / frame.Delay;
      for (const key of ["XPosition", "YPosition", "XScale", "YScale", "RedTint", "GreenTint", "BlueTint", "AlphaTint", "Rotation"] as const) result[key] = frame[key] + (next[key] - frame[key]) * ratio;
      return result;
    }
    cursor -= frame.Delay;
  }
  return frames.at(-1);
}

function fontLineWidth(font: Font, line: string, glyphBounds: boolean): number {
  let cursor = 0, extent = 0, previous: number | undefined;
  for (const character of line) {
    const code = character.codePointAt(0)!, glyph = font.chars[code];
    if (!glyph) continue;
    cursor += font.kernings.find(kerning => kerning.first === previous && kerning.second === code)?.amount ?? 0;
    extent = Math.max(extent, cursor + glyph.xoffset + glyph.width);
    cursor += glyph.xadvance;
    previous = code;
  }
  return glyphBounds ? Math.max(cursor, extent) : cursor;
}

export class Assets {
  private readonly tinted = new Map<string, HTMLCanvasElement>();
  private constructor(private readonly images: Map<string, HTMLImageElement>) {}
  static async load(): Promise<Assets> {
    const entries = Object.entries(urls).map(([path, url]) => [normalize(path.split("/resources/")[1]!), url] as const);
    entries.push(["gfx/ui/main menu/completion_widget.png", completionUrl], ["gfx/ui/main menu/gamemenu_postitoverlap.png", postitOverlapUrl]);
    const loaded = await Promise.all(entries.map(async ([path, url]) => {
      const image = new Image(); image.src = url; await image.decode();
      return [path, image] as const;
    }));
    return new Assets(new Map(loaded));
  }
  image(path: string): HTMLImageElement {
    const image = this.images.get(normalize(path)); if (!image) throw new Error(`Missing original asset: ${path}`); return image;
  }
  sources(): readonly { path: string; image: HTMLImageElement }[] { return [...this.images.entries()].map(([path, image]) => ({ path, image })); }
  /** Sample an authored layer so attached text follows the same animation as its paper or streak. */
  layerFrame(path: string, name: string, layerId: number, tick: number): Frame | undefined {
    const actor = (sceneData as unknown as Record<string, Actor>)[normalize(path)];
    if (!actor) throw new Error(`Missing authored animation: ${path}`);
    const animation = actor.Animations.find(animation => animation.Name === name);
    if (!animation) throw new Error(`Missing animation ${path}:${name}`);
    const layer = animation.LayerAnimations.find(layer => layer.LayerId === layerId);
    return layer?.Visible ? frameAt(layer.Frames, tick, animation.Loop) : undefined;
  }
  private colorImage(path: string, tint: number[], offsets = [0, 0, 0]): CanvasImageSource {
    if (tint.every(value => Math.abs(value - 255) < .001) && offsets.every(value => value === 0)) return this.image(path);
    const key = `${path}:${tint.map(Math.round)}:${offsets.map(Math.round)}`;
    const cached = this.tinted.get(key); if (cached) return cached;
    const source = this.image(path); const canvas = document.createElement("canvas"); canvas.width = source.width; canvas.height = source.height;
    const ctx = canvas.getContext("2d")!; ctx.drawImage(source, 0, 0); const pixels = ctx.getImageData(0, 0, canvas.width, canvas.height);
    for (let i = 0; i < pixels.data.length; i += 4) for (let c = 0; c < 3; c++) pixels.data[i + c] = Math.round(pixels.data[i + c]! * tint[c]! / 255 + offsets[c]!);
    ctx.putImageData(pixels, 0, 0); this.tinted.set(key, canvas); return canvas;
  }
  draw(ctx: CanvasRenderingContext2D, path: string, name: string, x = 0, y = 0, tick = 0, options: DrawOptions = {}): void {
    const actor = (sceneData as unknown as Record<string, Actor>)[normalize(path)]; if (!actor) throw new Error(`Missing authored animation: ${path}`);
    const animation = actor.Animations.find(animation => animation.Name === name); if (!animation) throw new Error(`Missing animation ${path}:${name}`);
    const root = frameAt(animation.RootAnimation, tick, animation.Loop); if (!root || !root.Visible || root.AlphaTint <= 0) return;
    ctx.save(); ctx.translate(x + root.XPosition, y + root.YPosition); ctx.rotate(root.Rotation * Math.PI / 180); ctx.scale(root.XScale / 100 * (options.scale ?? 1), root.YScale / 100 * (options.scale ?? 1));
    for (const layer of animation.LayerAnimations) {
      if (!layer.Visible || options.layers && !options.layers.includes(layer.LayerId)) continue;
      const found = frameAt(layer.Frames, tick, animation.Loop); if (!found) continue;
      const frame = { ...found, ...options.overrides?.[layer.LayerId] }; if (!frame.Visible || frame.AlphaTint <= 0) continue;
      const definition = actor.Layers.find(definition => definition.Id === layer.LayerId)!;
      const sheet = actor.Spritesheets.find(sheet => sheet.Id === definition.SpritesheetId)!;
      const imagePath = normalize(`${actor.BaseSpritesheetPath}/${sheet.Path}`);
      ctx.save(); ctx.globalAlpha *= root.AlphaTint / 255 * frame.AlphaTint / 255;
      ctx.translate(frame.XPosition, frame.YPosition); ctx.rotate(frame.Rotation * Math.PI / 180); ctx.scale(frame.XScale / 100, frame.YScale / 100);
      const backend = ctx as unknown as {
        drawTintedImage?: (source: CanvasImageSource, tint: number[], offsets: number[], ...args: number[]) => void;
        drawTintedImageAligned?: (source: CanvasImageSource, tint: number[], offsets: number[], pixelSize: number, ...args: number[]) => void;
      };
      const tint = [frame.RedTint / 255, frame.GreenTint / 255, frame.BlueTint / 255, 1];
      const offsets = [frame.RedOffset / 255, frame.GreenOffset / 255, frame.BlueOffset / 255];
      const rectangle = [frame.XCrop!, frame.YCrop!, frame.Width!, frame.Height!, -frame.XPivot!, -frame.YPivot!, frame.Width!, frame.Height!];
      if (options.pixelSnap && backend.drawTintedImageAligned)
        backend.drawTintedImageAligned(this.image(imagePath), tint, offsets, options.pixelSnap, ...rectangle);
      else if (backend.drawTintedImage) backend.drawTintedImage(this.image(imagePath), tint, offsets, ...rectangle);
      else {
        const image = this.colorImage(imagePath, [frame.RedTint, frame.GreenTint, frame.BlueTint], [frame.RedOffset, frame.GreenOffset, frame.BlueOffset]);
        ctx.drawImage(image, frame.XCrop!, frame.YCrop!, frame.Width!, frame.Height!, -frame.XPivot!, -frame.YPivot!, frame.Width!, frame.Height!);
      }
      ctx.restore();
    }
    ctx.restore();
  }
  font(ctx: CanvasRenderingContext2D, name: string, text: string, x: number, y: number, options: { scale?: number; color?: string; align?: "left" | "center" | "right"; pixelSnap?: boolean; measure?: "advance" | "glyph-bounds" } = {}): void {
    const font = (fontData as unknown as Record<string, Font>)[name]; if (!font) throw new Error(`Missing authored font: ${name}`);
    const scale = options.scale ?? 1; const codes = [...text].map(character => character.codePointAt(0)!);
    const width = Math.max(0, ...text.split("\n").map(line => fontLineWidth(font, line, options.measure === "glyph-bounds"))) * scale;
    const offset = options.align === "center" ? width / 2 : options.align === "right" ? width : 0;
    ctx.save(); ctx.translate(options.pixelSnap ? Math.round(x - offset) : x - offset, options.pixelSnap ? Math.round(y) : y); ctx.scale(scale, scale); let cursor = 0, baseline = 0;
    const color = options.color ?? "#ffffff"; const rgb = /^#([0-9a-f]{6})$/i.exec(color); const tint = rgb ? [0, 2, 4].map(i => parseInt(rgb[1]!.slice(i, i + 2), 16)) : [255, 255, 255];
    for (const [i, code] of codes.entries()) {
      if (code === 10) { cursor = 0; baseline += font.lineHeight; continue; }
      const glyph = font.chars[code]; if (!glyph) continue;
      const previous = codes[i - 1]; cursor += font.kernings.find(kerning => kerning.first === previous && kerning.second === code)?.amount ?? 0;
      if (glyph.width && glyph.height) {
        const backend = ctx as unknown as { drawTintedImage?: (source: CanvasImageSource, tint: number[], offsets: number[], ...args: number[]) => void };
        if (backend.drawTintedImage) backend.drawTintedImage(this.image(`font/${font.pages[glyph.page]}`), [tint[0]! / 255, tint[1]! / 255, tint[2]! / 255, 1], [0, 0, 0], glyph.x, glyph.y, glyph.width, glyph.height, cursor + glyph.xoffset, baseline + glyph.yoffset, glyph.width, glyph.height);
        else ctx.drawImage(this.colorImage(`font/${font.pages[glyph.page]}`, tint), glyph.x, glyph.y, glyph.width, glyph.height, cursor + glyph.xoffset, baseline + glyph.yoffset, glyph.width, glyph.height);
      }
      cursor += glyph.xadvance;
    }
    ctx.restore();
  }
}
