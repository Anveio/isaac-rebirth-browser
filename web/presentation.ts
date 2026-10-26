import { Assets } from "./animation";
import type { SampledFrame } from "./animation-math";
import type { State } from "./engine";

const STREAK = "gfx/ui/ui_streak.anm2";
const BOSS_BAR = "gfx/ui/ui_bosshealthbar.png";
const PAUSE = "gfx/ui/pausescreen.anm2";
const STATS = "gfx/ui/pausescreenstats.anm2";

/** The overlay clocks belong to the simulation snapshot, so replay and redraw agree. */
export class RunPresentation {
  constructor(private readonly assets: Assets) {}

  introduction(ctx: CanvasRenderingContext2D, state: State): void {
    if (!state.runIntroduction.active) return;
    const tick = Math.floor(state.runIntroduction.age / 2);
    this.assets.draw(ctx, STREAK, "Text", 240, 48, tick, { layers: [0] });
    const frame = this.assets.layerFrame(STREAK, "Text", 0, tick);
    if (!frame?.Visible) return;
    // HUD::Render 574EA8 draws the 256x128 text target into a rectangle whose
    // top-left is (240,48) + layer-0 offset - (0.5 * scaled width, 9): the 9 is
    // not scaled, and the target scales from its top edge. ShowItemText 5732C0
    // centres the title at target y 0 and 5734F0 puts the subtitle at y 18.
    ctx.save();
    ctx.translate(240 + frame.XPosition, 48 + frame.YPosition - 9);
    ctx.scale(frame.XScale, frame.YScale);
    ctx.globalAlpha *= frame.AlphaTint;
    this.assets.font(ctx, "upheaval", state.runIntroduction.title, 0, 0, { align: "center", pixelSnap: true });
    const subtitle = state.runIntroduction.subtitle;
    //571F68 loads PFTempestaSevenCondensed into HUD+300 for the second line.
    if (subtitle) this.assets.font(ctx, "pftempestasevencondensed", subtitle, 0, 18, { align: "center", pixelSnap: true });
    ctx.restore();
  }

  /** HUD boss bar (5746a9): ui_bosshealthbar.png background at (166, 8) and
   * a fill of fill*110 pixels from (20, 0), dimmed to 0.6 on even flash ticks. */
  bossBar(ctx: CanvasRenderingContext2D, state: State): void {
    const bar = state.bossBar;
    // A 0/0 fill (bosses present, none summed yet) serializes as null and,
    // like a hidden -1 bar, draws nothing in the original.
    if (bar.fill === null || !(bar.fill >= 0)) return;
    const image = this.assets.image(BOSS_BAR);
    const backend = ctx as unknown as { drawTintedImage?: (source: CanvasImageSource, tint: number[], offsets: number[], ...args: number[]) => void };
    const draw = (tint: number, ...args: number[]) => backend.drawTintedImage
      ? backend.drawTintedImage(image, [tint, tint, tint, 1], [0, 0, 0], ...args)
      : ctx.drawImage(image, ...args as [number, number, number, number, number, number, number, number]);
    draw(1, 0, 32, 148, 32, 166, 8, 148, 32);
    const width = Math.fround(bar.fill * 110);
    if (width > 0) draw(bar.flash >= 0 && bar.flash % 2 === 0 ? 0.6 : 1, 20, 0, width, 32, 186, 8, width, 32);
  }

  pause(ctx: CanvasRenderingContext2D, state: State): void {
    const menu = state.pauseMenu;
    if (menu.phase === "hidden") return;
    const animation = menu.phase === "appearing" ? "Appear" : menu.phase === "disappearing" ? "Dissapear" : "Idle";
    const tick = Math.floor(menu.age / 2);
    // 5BA8D2 draws layer 4 (Blood) only at hard difficulty.
    this.assets.draw(ctx, PAUSE, animation, 288, 135, tick,
      { layers: state.difficultyPresentation.pauseBlood ? [0, 1, 3, 4, 5] : [0, 1, 3, 5] });
    const paper = this.assets.layerFrame(PAUSE, animation, 1, tick);
    if (paper?.Visible) this.attached(ctx, paper, 288, 135, () => {
      this.assets.draw(ctx, STATS, "Idle", 0, 0, 0, { layers: [0] });
      for (const [index, frame] of menu.statFrames.entries()) {
        if (frame !== null)
          this.assets.draw(ctx, STATS, "Idle", 0, 0, frame, { layers: [index + 1] });
      }
    });
    const seedPaper = this.assets.layerFrame(PAUSE, animation, 5, tick);
    if (seedPaper?.Visible) this.attached(ctx, seedPaper, 288, 135, () => {
      const seed = state.runSeed.replaceAll(" ", "");
      this.assets.font(ctx, "teammeatfont12", `${seed.slice(0, 4)}\n${seed.slice(4, 8)}`, -15, -40, { align: "center", color: "#362f2d", pixelSnap: true, measure: "glyph-bounds" });
    });
    if (menu.phase === "idle") this.assets.draw(ctx, PAUSE, "Cursor", 288, 135, menu.selection, { layers: [2] });
  }

  private attached(ctx: CanvasRenderingContext2D, frame: SampledFrame, x: number, y: number, draw: () => void): void {
    ctx.save();
    ctx.translate(x + frame.XPosition, y + frame.YPosition);
    ctx.rotate(frame.Rotation * Math.PI / 180);
    ctx.scale(frame.XScale, frame.YScale);
    ctx.globalAlpha *= frame.AlphaTint;
    draw();
    ctx.restore();
  }
}
