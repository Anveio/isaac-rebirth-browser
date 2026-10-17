import baseSceneData from "./scene-data.json";
import informationData from "./information-data.json";
import doorData from "./door-data.json";
import gridData from "./grid-data.json";
import npcData from "./npc-data.json";
import familiarData from "./familiar-data.json";
import gameOverData from "./gameover-data.json";
import stageData from "./stage-data.json";
import attackData from "./attack-data.json";
import bloodTearData from "./blood-tear-data.json";
import costume229Data from "./costume229-data.json";
import bossSplashData from "./boss-splash-data.json";
import pocketData from "./pocket-data.json";
const sceneData = { ...informationData, ...baseSceneData, ...doorData, ...gridData, ...npcData, ...familiarData, ...gameOverData, ...stageData, ...attackData, ...bloodTearData, ...costume229Data, ...pocketData };
import fontData from "./font-data.json";
import completionUrl from "./assets/completion_widget.png?url";
import postitOverlapUrl from "./assets/gamemenu_postitoverlap.png?url";
import { sampleFrame, overrideFrame, frameQuad, type SampledFrame, type Quad } from "./animation-math";

export interface Frame {
  XCrop?: number; YCrop?: number; Width?: number; Height?: number;
  XPosition: number; YPosition: number; XScale: number; YScale: number;
  XPivot?: number; YPivot?: number; Delay: number; Visible: boolean;
  RedTint: number; GreenTint: number; BlueTint: number; AlphaTint: number;
  RedOffset: number; GreenOffset: number; BlueOffset: number; Rotation: number; Interpolated: boolean;
}
interface Animation { Name: string; FrameNum: number; Loop: boolean; RootAnimation: Frame[]; LayerAnimations: { LayerId: number; Visible: boolean; Frames: Frame[] }[]; NullAnimations?: { NullId: number; Visible: boolean; Frames: Frame[] }[]; }
interface Actor { BaseSpritesheetPath: string; Spritesheets: { Id: number; Path: string }[]; Layers: { Id: number; SpritesheetId: number; Name: string }[]; Animations: Animation[]; }
/** Sprite color (tint RGBA, offset RGB in 0..1), combined with each keyframe as Color::operator*= 42f590 does. */
export interface SpriteColor { tint: readonly number[]; offsets: readonly number[]; }
export interface DrawOptions { color?: SpriteColor; scale?: number; layers?: number[]; overrides?: Record<number, Partial<Frame>>; layerScales?: Record<number, readonly [number, number]>; spritesheets?: Record<number, string>; pixelSnap?: number; }
interface Glyph { x: number; y: number; width: number; height: number; xoffset: number; yoffset: number; xadvance: number; page: number; }
interface Font { lineHeight: number; pages: string[]; chars: Record<string, Glyph>; kernings: { first: number; second: number; amount: number }[]; }
// Authored sheets are relative to the anm2's root path and may climb out of
// it (ui_cardspills.anm2 names "..\Effects\Effect_023_StarFlash.png").
export const normalizeAssetPath = (path: string) => path.replaceAll("\\", "/").split("/").reduce<string[]>((parts, part) => {
  if (part === "..") parts.pop(); else if (part !== "." && part !== "") parts.push(part);
  return parts;
}, []).join("/").replace(/^resources\//i, "").toLowerCase();
const normalize = normalizeAssetPath;
const urls = import.meta.glob([
  "../references/isaac/rebirth/resources/gfx/ui/main menu/**/*.png",
  "../references/isaac/rebirth/resources/gfx/ui/backselectwidget.png",
  "../references/isaac/rebirth/resources/gfx/ui/loadimages/loadimages-*.png",
  "../references/isaac/rebirth/resources/gfx/ui/death *.png",
  "../references/isaac/rebirth/resources/gfx/ui/ui_hearts.png",
  "../references/isaac/rebirth/resources/gfx/ui/hudpickups.png",
  "../references/isaac/rebirth/resources/gfx/ui/ui_cardspills.png",
  "../references/isaac/rebirth/resources/gfx/ui/ui_cardfronts.png",
  "../references/isaac/rebirth/resources/gfx/ui/minimap1.png",
  "../references/isaac/rebirth/resources/gfx/ui/minimap2.png",
  "../references/isaac/rebirth/resources/gfx/ui/bgblack.png",
  "../references/isaac/rebirth/resources/gfx/ui/giantbook/**/*.png",
  "../references/isaac/rebirth/resources/gfx/ui/stage/**/*.png",
  "../references/isaac/rebirth/resources/gfx/ui/pausescreen.png",
  "../references/isaac/rebirth/resources/gfx/ui/menu_mystuff.png",
  "../references/isaac/rebirth/resources/gfx/ui/effect_024_streak.png",
  "../references/isaac/rebirth/resources/gfx/ui/effect_cursepaper.png",
  "../references/isaac/rebirth/resources/gfx/characters/costumes/character_001_isaac.png",
  "../references/isaac/rebirth/resources/gfx/characters/costumes/ghost.png",
  "../references/isaac/rebirth/resources/gfx/characters/costumes/costume_monstros lung.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/01_basement.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/03_caves.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/05_depths.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/09_sheol.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/0a_library.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/0b_shop.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/0c_isaacsroom.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/0d_barrenroom.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/0e_arcade.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/0e_diceroom.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/0f_secretroom.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/controls.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/shading.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/shading_1x2.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/shading_2x1.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/shading_2x2.png",
  "../references/isaac/rebirth/resources/gfx/backdrop/light_gradient.png",
  "../references/isaac/rebirth/resources/gfx/overlays/basement/1x1_*.png",
  "../references/isaac/rebirth/resources/gfx/overlays/caves/1x1_*.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_01_normaldoor.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_02_treasureroomdoor.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_04_selfsacrificeroomdoor.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_08_holeinwall.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_17_bardoor.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_18_crackeddoor.png",
  "../references/isaac/rebirth/resources/gfx/grid/grid_tnt.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_13_librarydoor.png",
  "../references/isaac/rebirth/resources/gfx/grid/rocks_basement.png",
  "../references/isaac/rebirth/resources/gfx/grid/rocks_caves.png",
  "../references/isaac/rebirth/resources/gfx/grid/door_11_trapdoor.png",
  "../references/isaac/rebirth/resources/gfx/grid/grid_spikes.png",
  "../references/isaac/rebirth/resources/gfx/grid/grid_poop_[123].png",
  "../references/isaac/rebirth/resources/gfx/backdrop/fx_ray*.png",
  "../references/isaac/rebirth/resources/gfx/grid/props_01_basement.png",
  "../references/isaac/rebirth/resources/gfx/grid/props_03_caves.png",
  "../references/isaac/rebirth/resources/gfx/grid/props_05_depths.png",
  "../references/isaac/rebirth/resources/gfx/grid/props_09_sheol.png",
  "../references/isaac/rebirth/resources/gfx/grid/grid_pit.png",
  "../references/isaac/rebirth/resources/gfx/grid/grid_fireplace.png",
  "../references/isaac/rebirth/resources/gfx/grid/effect_005_fire.png",
  "../references/isaac/rebirth/resources/gfx/grid/effect_ember.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_poopspawn.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_002b_shitpoof.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_poopgibs.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_030_bloodgibs.png",
  "../references/isaac/rebirth/resources/gfx/monsters/**/*.png",
  "../references/isaac/rebirth/resources/gfx/bosses/**/*.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_032_bloodstains.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_016_bloodpool.png",
  "../references/isaac/rebirth/resources/gfx/effects/hangingguys.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_000_shopkeepers.png",
  "../references/isaac/rebirth/resources/gfx/familiar/familiar_270_leech.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_009_poof01*.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_012_poof04.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_bloodgush.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_018_lasereffects.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_brimstoneimpact.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_momsknife.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_006_hearteffect.png",
  "../references/isaac/rebirth/resources/gfx/shadow.png",
  "../references/isaac/rebirth/resources/gfx/tears.png",
  "../references/isaac/rebirth/resources/gfx/enemybullets.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_015_tearpoof*.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_002_bloodpoof*.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_021_tiny bug.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_033_tiny fly.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_029_explosion.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_002_bloodpoof*.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_033_tiny fly.png",
  "../references/isaac/rebirth/resources/gfx/items/pick ups/pickup_016_bomb.png",
  "../references/isaac/rebirth/resources/gfx/items/**/*.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_023_starflash.png",
  "../references/isaac/rebirth/resources/gfx/effects/effect_052_dust cloud.png",
  "../references/isaac/rebirth/resources/gfx/ui/ui_chargebar.png",
  "../references/isaac/rebirth/resources/gfx/ui/ui_bosshealthbar.png",
  "../references/isaac/rebirth/resources/gfx/ui/boss/*.png",
  "../references/isaac/rebirth/resources/font/*_0.png",
], { eager: true, query: "?url", import: "default" }) as Record<string, string>;

function loaded(image: HTMLImageElement, error: unknown): Promise<void> {
  return new Promise((resolve, reject) => {
    const settle = () => image.naturalWidth > 0 ? resolve() : reject(error);
    if (image.complete) settle();
    else {
      image.addEventListener("load", settle, { once: true });
      image.addEventListener("error", () => reject(error), { once: true });
    }
  });
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
    // Monster and boss sheets are globbed whole; decode only the ones an
    // authored actor in the loaded animation data actually references.
    const referenced = new Set(Object.values(sceneData as Record<string, Actor>).flatMap(actor =>
      actor.Spritesheets.map(sheet => normalize(`${actor.BaseSpritesheetPath}/${sheet.Path}`))));
    const entries = Object.entries(urls).map(([path, url]) => [normalize(path.split("/resources/")[1]!), url] as const)
      .filter(([path]) => !/^gfx\/(monsters|bosses)\//.test(path) || referenced.has(path));
    entries.push(["gfx/ui/main menu/completion_widget.png", completionUrl], ["gfx/ui/main menu/gamemenu_postitoverlap.png", postitOverlapUrl]);
    const available = new Set(entries.map(([path]) => path));
    for (const path of bossSplashData.assets) {
      if (!available.has(normalize(path))) throw new Error(`Missing original boss presentation asset: ${path}`);
    }
    // Chromium rejects decode() with EncodingError when many decodes are in
    // flight, even for intact images. Keep a bounded number busy, and on a
    // rejection fall back to the load event: an image that loaded with real
    // dimensions is usable (drawing decodes it); only a failed load is fatal.
    const images: HTMLImageElement[] = [];
    let next = 0;
    await Promise.all(Array.from({ length: 16 }, async () => {
      while (next < entries.length) {
        const index = next++, image = new Image();
        image.src = entries[index]![1];
        await image.decode().catch(error => loaded(image, error));
        images[index] = image;
      }
    }));
    return new Assets(new Map(entries.map(([path], index) => [path, images[index]!])));
  }
  image(path: string): HTMLImageElement {
    const image = this.images.get(normalize(path)); if (!image) throw new Error(`Missing original asset: ${path}`); return image;
  }
  sources(): readonly { path: string; image: HTMLImageElement }[] { return [...this.images.entries()].map(([path, image]) => ({ path, image })); }
  /** Sample an authored layer so attached text follows the same animation as its paper or streak. */
  layerFrame(path: string, name: string, layerId: number, tick: number): SampledFrame | undefined {
    const actor = (sceneData as unknown as Record<string, Actor>)[normalize(path)];
    if (!actor) throw new Error(`Missing authored animation: ${path}`);
    const animation = actor.Animations.find(animation => animation.Name === name);
    if (!animation) throw new Error(`Missing animation ${path}:${name}`);
    const layer = animation.LayerAnimations.find(layer => layer.LayerId === layerId);
    return layer?.Visible ? sampleFrame(layer.Frames, tick, animation.Loop) : undefined;
  }
  /** An actor's clip names in authored order (Sprite play-by-index order). */
  animationNames(path: string): string[] {
    const actor = (sceneData as unknown as Record<string, Actor>)[normalize(path)];
    if (!actor) throw new Error(`Missing authored animation: ${path}`);
    return actor.Animations.map(animation => animation.Name);
  }
  /** The spritesheet image an authored layer samples. */
  layerImage(path: string, layerId: number): HTMLImageElement {
    const actor = (sceneData as unknown as Record<string, Actor>)[normalize(path)];
    if (!actor) throw new Error(`Missing authored animation: ${path}`);
    const definition = actor.Layers.find(layer => layer.Id === layerId);
    const sheet = definition && actor.Spritesheets.find(candidate => candidate.Id === definition.SpritesheetId);
    if (!sheet) throw new Error(`Missing authored layer ${path}:${layerId}`);
    return this.image(`${actor.BaseSpritesheetPath}/${sheet.Path}`);
  }
  nullFrame(path: string, name: string, nullId: number, tick: number): SampledFrame | undefined {
    const actor = (sceneData as unknown as Record<string, Actor>)[normalize(path)];
    if (!actor) throw new Error(`Missing authored animation: ${path}`);
    const animation = actor.Animations.find(animation => animation.Name === name);
    if (!animation) throw new Error(`Missing animation ${path}:${name}`);
    const attachment = animation.NullAnimations?.find(attachment => attachment.NullId === nullId);
    return attachment?.Visible ? sampleFrame(attachment.Frames, tick, animation.Loop) : undefined;
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
    const root = sampleFrame(animation.RootAnimation, tick, animation.Loop); if (!root || !root.Visible || root.AlphaTint <= 0) return;
    for (const layer of animation.LayerAnimations) {
      if (!layer.Visible || options.layers && !options.layers.includes(layer.LayerId)) continue;
      const found = sampleFrame(layer.Frames, tick, animation.Loop); if (!found) continue;
      const frame = overrideFrame(found, options.overrides?.[layer.LayerId]);
      // The root keyframe colour combines into every layer (tints multiply,
      // offsets add); only the brimstone impact authors a non-white root.
      if (root.RedTint !== 1 || root.GreenTint !== 1 || root.BlueTint !== 1 || root.RedOffset || root.GreenOffset || root.BlueOffset) {
        frame.RedTint = Math.fround(frame.RedTint * root.RedTint); frame.GreenTint = Math.fround(frame.GreenTint * root.GreenTint);
        frame.BlueTint = Math.fround(frame.BlueTint * root.BlueTint);
        frame.RedOffset = Math.fround(frame.RedOffset + root.RedOffset); frame.GreenOffset = Math.fround(frame.GreenOffset + root.GreenOffset);
        frame.BlueOffset = Math.fround(frame.BlueOffset + root.BlueOffset);
      }
      const color = options.color;
      if (color) {
        // Tints multiply and offsets add; colorize is zero for every caller.
        frame.RedTint = Math.fround(frame.RedTint * color.tint[0]!); frame.GreenTint = Math.fround(frame.GreenTint * color.tint[1]!);
        frame.BlueTint = Math.fround(frame.BlueTint * color.tint[2]!); frame.AlphaTint = Math.fround(frame.AlphaTint * color.tint[3]!);
        frame.RedOffset = Math.fround(frame.RedOffset + color.offsets[0]!); frame.GreenOffset = Math.fround(frame.GreenOffset + color.offsets[1]!);
        frame.BlueOffset = Math.fround(frame.BlueOffset + color.offsets[2]!);
      }
      if (!frame.Visible || frame.AlphaTint <= 0) continue;
      const layerScale = options.layerScales?.[layer.LayerId];
      if (layerScale) {
        frame.XScale = Math.fround(frame.XScale * layerScale[0]);
        frame.YScale = Math.fround(frame.YScale * layerScale[1]);
      }
      const definition = actor.Layers.find(definition => definition.Id === layer.LayerId)!;
      const sheet = actor.Spritesheets.find(sheet => sheet.Id === definition.SpritesheetId)!;
      const imagePath = normalize(options.spritesheets?.[sheet.Id] ?? `${actor.BaseSpritesheetPath}/${sheet.Path}`);
      ctx.save(); ctx.globalAlpha *= Math.fround(root.AlphaTint * frame.AlphaTint);
      const backend = ctx as unknown as {
        drawTintedImage?: (source: CanvasImageSource, tint: number[], offsets: number[], ...args: number[]) => void;
        drawTintedImageQuad?: (source: CanvasImageSource, tint: number[], offsets: number[], crop: readonly [number, number, number, number], points: Quad, pixelSize?: number) => void;
      };
      const tint = [frame.RedTint, frame.GreenTint, frame.BlueTint, 1];
      const offsets = [frame.RedOffset, frame.GreenOffset, frame.BlueOffset];
      if (backend.drawTintedImageQuad) {
        // The native actor rounds aligned quads on the final display-pixel grid.
        // Rotated layers retain the sequential f32 coordinates from GetDestQuad.
        backend.drawTintedImageQuad(this.image(imagePath), tint, offsets,
          [frame.XCrop!, frame.YCrop!, frame.Width!, frame.Height!], frameQuad(frame, root, [x, y], options.scale), options.pixelSnap ?? 1);
      } else {
        ctx.translate(Math.fround(x + root.XPosition), Math.fround(y + root.YPosition));
        ctx.rotate(root.Rotation * Math.PI / 180); ctx.scale(root.XScale * (options.scale ?? 1), root.YScale * (options.scale ?? 1));
        ctx.translate(frame.XPosition, frame.YPosition); ctx.rotate(frame.Rotation * Math.PI / 180); ctx.scale(frame.XScale, frame.YScale);
        const rectangle = [frame.XCrop!, frame.YCrop!, frame.Width!, frame.Height!, -frame.XPivot!, -frame.YPivot!, frame.Width!, frame.Height!];
        if (backend.drawTintedImage) backend.drawTintedImage(this.image(imagePath), tint, offsets, ...rectangle);
        else ctx.drawImage(this.colorImage(imagePath, tint.slice(0, 3).map(value => value * 255), offsets.map(value => value * 255)), ...rectangle as [number, number, number, number, number, number, number, number]);
      }
      ctx.restore();
    }
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
