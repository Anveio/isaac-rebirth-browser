import { Assets } from "./animation";
import sceneData from "./scene-data.json";
import type { State } from "./engine";

const MENU = "gfx/ui/main menu/";
const PORTRAITS = `${MENU}characterportraits.anm2`;

/** Authored menu layers are composed at the original 480 × 270 logical size. */
export class MenuRenderer {
  constructor(private readonly assets: Assets) {}

  draw(ctx: CanvasRenderingContext2D, state: State): void {
    switch (state.screen) {
      case "title":
        this.assets.draw(ctx, `${MENU}titlemenu.anm2`, "Idle", 0, 0, state.titleAnimationFrame);
        break;
      case "profile": this.profile(ctx, state); break;
      case "main": this.main(ctx, state); break;
      case "character": this.character(ctx, state); break;
      case "challenge": this.challenge(ctx, state); break;
      case "seed": this.seed(ctx, state); break;
      case "playing": return;
    }
    ctx.save();
    ctx.imageSmoothingEnabled = true;
    ctx.drawImage(this.assets.image(`${MENU}menuoverlay.png`), -0.5, -0.5, 481, 271);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(this.assets.image(`${MENU}menushadow.png`), 0, 0, 256, 149, 0, 122, 256, 149);
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
      //59C872 ->593580 draws the retained loading texture in white/current
      //alpha, centered at its authored size, before the incoming menu tint.
      const index = state.runStartRequest.loadingImageIndex;
      if (index !== undefined) {
        const image = this.assets.image(`gfx/ui/loadimages/loadimages-${String(index).padStart(3, "0")}.png`);
        ctx.save(); ctx.globalAlpha *= alpha;
        ctx.drawImage(image, (480 - image.width) / 2, (270 - image.height) / 2);
        ctx.restore();
      }
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

  private profile(ctx: CanvasRenderingContext2D, state: State): void {
    const selected = Math.max(0, Math.min(2, state.profile - 1));
    const drawingPositions = [32, 176, 321];
    this.assets.draw(ctx, `${MENU}saveselectmenu.anm2`, `File${selected + 1}Select`, 0, 0, 4);
    for (let i = 0; i < 3; i++) {
      this.assets.draw(ctx, `${MENU}saveselectmenu_drawing.anm2`, "Idle", drawingPositions[i], i === selected ? 20 : 30, 0, {
        layers: [0],
      });
    }
    const widget = this.assets.image("gfx/ui/backselectwidget.png");
    ctx.drawImage(widget, 0, 0, 96, 96, -6, 199, 96, 96);
    ctx.drawImage(widget, 96, 0, 112, 128, 384, 179, 112, 128);
    this.assets.font(ctx, "teammeatfont12", "Esc", 20, 242, { color: "#362f2d" });
    this.assets.font(ctx, "teammeatfont12", "Space", 423, 242, { color: "#362f2d" });
  }

  private main(ctx: CanvasRenderingContext2D, state: State): void {
    this.assets.draw(ctx, `${MENU}gamemenu.anm2`, "Idle", 0, 0, state.mainSelection, {
      overrides: {
        2: { XCrop: 160, YCrop: 336 },
      },
    });
  }

  private challenge(ctx: CanvasRenderingContext2D, state: State): void {
    const actor = `${MENU}challengemenu.anm2`;
    this.assets.draw(ctx, actor, "Idle", 0, 0, 0, {layers: [0, 3, 4, 5, 6, 7, 28, 1]});
    this.assets.draw(ctx, actor, "Idle", 0, 0, (state.challengeSelection ?? 1) - 1, {layers: [2]});
    for (let index = 0; index < 20; index++) {
      //Fresh menu state implements actual locked/unlocked layers. Completed
      //profile import/presentation is outside this item-focused frontdoor.
      this.assets.draw(ctx, actor, "Idle", 0, 0, state.challengeUnlocked[index] ? 1 : 0, {layers: [8 + index]});
    }
  }

  private character(ctx: CanvasRenderingContext2D, state: State): void {
    const profile = state.profiles[state.profile - 1]!;
    const characters = state.characterCatalog;
    const selected = state.characterSelection % characters.length;
    const selection = characters[selected]!;
    if (selection.name === "Random") {
      this.assets.draw(ctx, `${MENU}charactermenu.anm2`, "01_Isaac", 0, 0, 0, { layers: [0, 1, 2] });
      this.assets.font(ctx, "teammeatfont16bold", "RANDOM", 240, 158, { align: "center", color: "#362f2d" });
    } else {
      this.assets.draw(ctx, `${MENU}charactermenu.anm2`, selection.animation, 0, 0, 0, {
        layers: profile.unlockedCharacters.includes(selection.name) ? [0, 1, 2, 3, 4, 5, 6, 9, 10, 11] : [0, 1, 2, 3, 4, 5, 6, 9, 10, 11, 14, 15, 16],
      });
    }
    this.assets.draw(ctx, `${MENU}completion_widget.anm2`, "Idle", 80, 50, 0, { layers: [0] });
    const portraits = characters.map(({ name: character, animation }, index) => {
      const angle = (index - selected) * Math.PI * 2 / characters.length;
      const depth = Math.cos(angle);
      const scale = Math.min(1, 0.55 + (1 + depth) * 7 / 30);
      return { character, animation, depth, x: 240 + 84 * Math.sin(angle), y: 90 + (1 + depth) * 70 / 3, scale };
    }).sort((a, b) => a.depth - b.depth);
    for (const portrait of portraits) {
      const unlocked = profile.unlockedCharacters.includes(portrait.character) || portrait.character === "Random";
      const actor = sceneData["gfx/ui/main menu/characterportraits.anm2"];
      const frames = actor.Animations.find(animation => animation.Name === portrait.animation)!.LayerAnimations[0]!.Frames;
      const frame = frames[unlocked ? 0 : Math.min(1, frames.length - 1)]!;
      const snap = (position: number) => Math.round(position * 2) / 2;
      const left = snap(portrait.x - frame.Width * portrait.scale / 2), right = snap(portrait.x + frame.Width * portrait.scale / 2);
      const top = snap(portrait.y - frame.Height * portrait.scale / 2), bottom = snap(portrait.y + frame.Height * portrait.scale / 2);
      this.assets.draw(ctx, PORTRAITS, portrait.animation, (left + right) / 2, (top + bottom) / 2, unlocked ? 0 : 1, {
        overrides: { 0: { AlphaTint: portrait.scale * 255, XScale: (right - left) / frame.Width * 100, YScale: (bottom - top) / frame.Height * 100 } },
      });
    }

    this.assets.draw(ctx, `${MENU}winstreakwidget.anm2`, "Idle", 45, 140);
    this.assets.font(ctx, "teammeatfont12", "0", 85.5, 162, { align: "center", color: "#362f2d" });
    this.assets.draw(ctx, `${MENU}difficultywidget.anm2`, "Idle", 365, 120, state.difficulty === "hard" ? 1 : 0);
    this.assets.draw(ctx, `${MENU}seedwidget.anm2`, "Idle", 350, 45, 0, {
      layers: state.seedText ? [0] : [0, 1, 2],
    });
    if (state.seedText) {
      this.assets.font(ctx, "teammeatfont12", this.formattedSeed(state.seedText), 400, 57, { align: "center", color: "#362f2d" });
    }
  }

  private seed(ctx: CanvasRenderingContext2D, state: State): void {
    this.assets.draw(ctx, `${MENU}seedentry.anm2`, "Idle");
    const text = state.seedText.padEnd(8, "_");
    this.assets.font(ctx, "teammeatfont20bold", text.slice(0, 4), 185, 35, { color: "#362f2d" });
    this.assets.font(ctx, "teammeatfont20bold", text.slice(4, 8), 185, 60, { color: "#362f2d" });
  }

  private formattedSeed(seed: string): string { return `${seed.slice(0, 4)} ${seed.slice(4, 8)}`; }
}
