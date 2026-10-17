import { Assets } from "./animation";
import sceneData from "./scene-data.json";
import type { State } from "./engine";

const MENU = "gfx/ui/main menu/";
const PORTRAITS = `${MENU}characterportraits.anm2`;

/** Authored menu layers are composed at the original 480 × 270 logical size. */
export class MenuRenderer {
  constructor(private readonly assets: Assets) {}

  draw(ctx: CanvasRenderingContext2D, state: State): void {
    if (state.screen === "playing") return;
    //59B5E0 draws every page at its board origin plus MenuManager+3C, so a
    //page change pans across the board instead of cutting.
    const [viewX, viewY] = state.menuPresentation.view;
    this.background(ctx, viewX, viewY);
    const visible = (x: number, y: number, height = 270) => x + 480 > 0 && x < 480 && y + height > 0 && y < 270;
    if (visible(viewX, viewY)) {
      this.assets.draw(ctx, `${MENU}titlemenu.anm2`, "Idle", viewX, viewY, state.titleAnimationFrame);
    }
    if (visible(viewX - 480, viewY + 810, 485)) this.challenge(ctx, state, viewX - 480, viewY + 810);
    if (visible(viewX, viewY + 810)) this.main(ctx, state, viewX, viewY + 810);
    if (visible(viewX + 480, viewY + 810)) this.character(ctx, state, viewX + 480, viewY + 810);
    if (visible(viewX + 960, viewY + 810)) this.seed(ctx, state, viewX + 960, viewY + 810);
    if (visible(viewX, viewY + 270)) this.profile(ctx, state, viewX, viewY + 270);
    this.widgets(ctx, state);
    ctx.save();
    ctx.imageSmoothingEnabled = true;
    ctx.drawImage(this.assets.image(`${MENU}menuoverlay.png`), -0.5, -0.5, 481, 271);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(this.assets.image(`${MENU}menushadow.png`), 0, 0, 256, 149, 0, 122, 256, 149);
    //59C890: hard mode draws MenuOverlay a second time at MenuManager+28.
    const { hardOverlay } = state.menuPresentation;
    if (hardOverlay > 0) {
      ctx.imageSmoothingEnabled = true;
      ctx.globalAlpha *= Math.min(1, hardOverlay);
      ctx.drawImage(this.assets.image(`${MENU}menuoverlay.png`), -0.5, -0.5, 481, 271);
    }
    ctx.restore();
    //59C702/59C7F0 draws the outgoing confirmation quad after the menu,
    //using Menu+4 independently of the incoming Menu+8/+C tint below.
    if (state.runStartRequest.fading && state.runStartRequest.fadeAlpha > 0) {
      const alpha = Math.min(1, state.runStartRequest.fadeAlpha);
      const backend = ctx as unknown as { fillRectRGBA?: (x: number, y: number, width: number, height: number, rgba: readonly number[]) => void };
      if (backend.fillRectRGBA) backend.fillRectRGBA(0, 0, 480, 270, [0, 0, 0, alpha]);
      else {
        ctx.save(); ctx.globalAlpha *= alpha;
        ctx.fillStyle = "#000000";
        ctx.fillRect(0, 0, 480, 270); ctx.restore();
      }
      this.drawLoadingImage(ctx, state, alpha);
    }
    if (state.menuFade.active && state.menuFade.alpha > 0) {
      const { color, alpha } = state.menuFade;
      const backend = ctx as unknown as { fillRectRGBA?: (x: number, y: number, width: number, height: number, rgba: readonly number[]) => void };
      if (backend.fillRectRGBA) backend.fillRectRGBA(0, 0, 480, 270, [color[0]!, color[1]!, color[2]!, alpha]);
      else {
        ctx.save(); ctx.globalAlpha *= alpha;
        ctx.fillStyle = `#${color.slice(0, 3).map(value => Math.round(value * 255).toString(16).padStart(2, "0")).join("")}`;
        ctx.fillRect(0, 0, 480, 270); ctx.restore();
      }
    }
  }

  /**593580: same retained texture during menu confirmation and initial Game fade. */
  drawLoadingImage(ctx: CanvasRenderingContext2D, state: State, alpha: number): void {
    const index = state.runStartRequest.loadingImageIndex;
    if (index !== undefined) {
      const image = this.assets.image(`gfx/ui/loadimages/loadimages-${String(index).padStart(3, "0")}.png`);
      ctx.save(); ctx.globalAlpha *= alpha;
      ctx.drawImage(image, (480 - image.width) / 2, (270 - image.height) / 2);
      ctx.restore();
    }
  }

  /**EmptyScreen.png wraps under the pages with the view as its source offset. */
  private background(ctx: CanvasRenderingContext2D, viewX: number, viewY: number): void {
    const image = this.assets.image(`${MENU}emptyscreen.png`);
    const wrap = (value: number, period: number) => ((value % period) + period) % period;
    const left = wrap(viewX, 480) - 480, top = wrap(viewY, 270) - 270;
    for (const x of [left, left + 480]) for (const y of [top, top + 270]) ctx.drawImage(image, x, y, 480, 270);
  }

  /**5A9C60: the SaveSelectMenu clip, each file's drawing at its Drawing null. */
  private profile(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    const { menu, drawings } = state.menuPresentation.save;
    const actor = `${MENU}saveselectmenu.anm2`;
    this.assets.draw(ctx, actor, menu.animation, x, y, menu.frame);
    drawings.forEach((drawing, index) => {
      const at = this.assets.nullFrame(actor, menu.animation, index, menu.frame);
      if (!at) return;
      this.assets.draw(ctx, `${MENU}saveselectmenu_drawing.anm2`, drawing.animation, x + at.XPosition, y + at.YPosition, drawing.frame, {
        layers: [0],
      });
    });
  }

  /**Back and Select drop by MenuManager+2C/+30 below their rest. */
  private widgets(ctx: CanvasRenderingContext2D, state: State): void {
    const [back, select] = state.menuPresentation.widgets;
    if (back >= 90 && select >= 90) return;
    const widget = this.assets.image("gfx/ui/backselectwidget.png");
    ctx.drawImage(widget, 0, 0, 96, 96, -6, 199 + back, 96, 96);
    ctx.drawImage(widget, 96, 0, 112, 128, 384, 179 + select, 112, 128);
    this.assets.font(ctx, "teammeatfont12", "Esc", 20, 242 + back, { color: "#362f2d" });
    this.assets.font(ctx, "teammeatfont12", "Space", 423, 242 + select, { color: "#362f2d" });
  }

  private main(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    this.assets.draw(ctx, `${MENU}gamemenu.anm2`, "Idle", x, y, state.mainSelection, {
      overrides: {
        2: { XCrop: 160, YCrop: 336 },
      },
    });
  }

  private challenge(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    const actor = `${MENU}challengemenu.anm2`;
    this.assets.draw(ctx, actor, "Idle", x, y, 0, {layers: [0, 3, 4, 5, 6, 7, 28, 1]});
    this.assets.draw(ctx, actor, "Idle", x, y, (state.challengeSelection ?? 1) - 1, {layers: [2]});
    for (let index = 0; index < 20; index++) {
      //Fresh menu state implements actual locked/unlocked layers. Completed
      //profile import/presentation is outside this item-focused frontdoor.
      this.assets.draw(ctx, actor, "Idle", x, y, state.challengeUnlocked[index] ? 1 : 0, {layers: [8 + index]});
    }
  }

  private character(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    const profile = state.profiles[state.profile - 1]!;
    const characters = state.characterCatalog;
    const selected = state.characterSelection % characters.length;
    const selection = characters[selected]!;
    if (selection.name === "Random") {
      this.assets.draw(ctx, `${MENU}charactermenu.anm2`, "01_Isaac", x, y, 0, { layers: [0, 1, 2] });
      this.assets.font(ctx, "teammeatfont16bold", "RANDOM", x + 240, y + 165, { align: "center", color: "#362f2d" });
    } else {
      //A locked character shows its name and unlock text without stats.
      this.assets.draw(ctx, `${MENU}charactermenu.anm2`, selection.animation, x, y, 0, {
        layers: profile.unlockedCharacters.includes(selection.name) ? [0, 1, 2, 3, 4, 5, 6, 9, 10, 11] : [0, 1, 2, 3, 14, 15, 16],
      });
    }
    this.assets.draw(ctx, `${MENU}completion_widget.anm2`, "Idle", x + 80, y + 50, 0, { layers: [0] });
    //59FB3E: the BloodStain sprite fades with hard mode at the page origin.
    const { blood } = state.menuPresentation;
    if (blood > 0) {
      this.assets.draw(ctx, `${MENU}charactermenu.anm2`, "BloodStain", x, y, 0, { color: { tint: [1, 1, 1, blood], offsets: [0, 0, 0] } });
    }
    //59F8A0 sorts the carousel by depth; scale and alpha follow the height.
    const { portraits } = state.menuPresentation.carousel;
    const order = characters.map((character, index) => ({ character, position: portraits[index] ?? [240, 135] }))
      .sort((a, b) => a.position[1] - b.position[1]);
    for (const { character, position } of order) {
      const unlocked = profile.unlockedCharacters.includes(character.name) || character.name === "Random";
      const scale = Math.min(1, Math.fround(Math.fround(position[1] - 135) / 50 * 0.5 + 1));
      const frames = sceneData["gfx/ui/main menu/characterportraits.anm2"].Animations.find(animation => animation.Name === character.animation)!.LayerAnimations[0]!.Frames;
      this.assets.draw(ctx, PORTRAITS, character.animation, x + position[0], y + position[1], unlocked ? 0 : Math.min(1, frames.length - 1), {
        scale, color: { tint: [1, 1, 1, scale], offsets: [0, 0, 0] },
      });
    }

    this.assets.draw(ctx, `${MENU}winstreakwidget.anm2`, "Idle", x + 45, y + 140);
    this.assets.font(ctx, "teammeatfont12", "0", x + 85.5, y + 162, { align: "center", color: "#362f2d" });
    this.assets.draw(ctx, `${MENU}difficultywidget.anm2`, "Idle", x + 365, y + 120, state.menuPresentation.hardWidget ? 1 : 0);
    this.assets.draw(ctx, `${MENU}seedwidget.anm2`, "Idle", x + 350, y + 45, 0, {
      layers: state.seedText ? [0] : [0, 1, 2],
    });
    if (state.seedText) {
      this.assets.font(ctx, "teammeatfont12", this.formattedSeed(state.seedText), x + 400, y + 57, { align: "center", color: "#362f2d" });
    }
  }

  /**Seed entry shows only the typed characters; empty slots stay blank. */
  private seed(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    this.assets.draw(ctx, `${MENU}seedentry.anm2`, "Idle", x, y);
    const text = state.seedText;
    this.assets.font(ctx, "teammeatfont20bold", text.slice(0, 4), x + 185, y + 35, { color: "#362f2d" });
    this.assets.font(ctx, "teammeatfont20bold", text.slice(4, 8), x + 185, y + 60, { color: "#362f2d" });
  }

  private formattedSeed(seed: string): string { return `${seed.slice(0, 4)} ${seed.slice(4, 8)}`; }
}
