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
    if (visible(viewX + 480, viewY + 1080, 380)) this.options(ctx, state, viewX + 480, viewY + 1080);
    if (visible(viewX + 480, viewY + 1620)) this.controls(ctx, state, viewX + 480, viewY + 1620);
    if (visible(viewX + 480, viewY + 1890, 810)) this.keyboard(ctx, state, viewX + 480, viewY + 1890);
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

  private options(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    const path = `${MENU}optionsmenu.anm2`;
    // Main-family constructor5A6950 uses the authored normal paper.
    const owned = new Set([3, 4, 5, 6, 7, 8, 20]);
    const actor = sceneData[path as keyof typeof sceneData];
    const layers = actor.Layers.map(layer => layer.Id).filter(id => !owned.has(id));
    this.assets.draw(ctx, path, "Idle", x, y, 0, {layers});
    const gauge = (value: number) => Math.floor(Math.fround(Math.fround(value * 10) + 0.5));
    this.assets.draw(ctx, path, "Idle", x, y, gauge(state.audio.sfxVolume), {layers: [3]});
    this.assets.draw(ctx, path, "Idle", x, y, gauge(state.music.musicEnabled ? state.music.volume : 0), {layers: [4]});
    this.assets.draw(ctx, path, "Idle", x, y, gauge(state.mapSettings.opacity), {layers: [5]});
    // The native numeric display uses its GammaMenu Counter actor, not a slider.
    const percent = Math.floor(Math.fround(Math.fround(Math.fround(state.gammaSettings.gamma) * 100) + 0.5));
    for (const [index, digit] of [...String(percent)].reverse().entries())
      this.assets.draw(ctx, `${MENU}gammamenu.anm2`, "Counter", x + 300 - index * 11, y + 210, Number(digit));
    //5A79D9/5A7F70: source Settings fullscreen uses inverted layer frames.
    this.assets.draw(ctx, path, "Idle", x, y, state.fullscreen.requested ? 0 : 1, {layers: [6]});
    //5A79B1/5A7990: Filter (layer 7) and Popups (layer 20) also draw frame
    // !Settings+15 and !Settings+29. The port has no Filter setting, so it
    // keeps the Settings constructor5B8CEB default 0 (OFF).
    this.assets.draw(ctx, path, "Idle", x, y, 1, {layers: [7]});
    if (state.unlockPopups !== null)
      this.assets.draw(ctx, path, "Idle", x, y, state.unlockPopups ? 0 : 1, {layers: [20]});
    this.assets.draw(ctx, path, "Idle", x, y, state.optionsMenu.selection, {layers: [8]});
  }

  private controls(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    this.assets.draw(ctx, `${MENU}controllerselectmenu.anm2`, 'Idle', x, y, 0);
    //Only the actual admitted Keyboard is connected. Retained file devices
    //cannot become invented connected controller rows.
    this.assets.font(ctx, 'teammeatfont12', 'Keyboard', x + 280, y + 63, {color:'#362f2d'});
  }

  private keyboard(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    const path = `${MENU}keyconfigmenu.anm2`;
    this.assets.draw(ctx,path,'Idle',x,y,0,{layers:[0,9,8,7,2,1,3,4]});
    const names = ['Move up','Move down','Move left','Move right','Shoot up','Shoot down','Shoot left','Shoot right',
      'Bomb','Item','Pill / card','Drop (unavailable)','Map','Fullscreen (unavailable)','Restart (unavailable)','Mute (unavailable)','Pause (unavailable)'];
    const actions = [2,3,0,1,6,7,4,5,8,9,10,11,13,17,16,18,12];
    const labels: Record<number,string> = {'-1':'---',32:'Space',256:'Esc',257:'Enter',258:'Tab',259:'Backspace',
      260:'Insert',261:'Delete',262:'Right',263:'Left',264:'Down',265:'Up',266:'Page up',267:'Page down',
      268:'Home',269:'End',340:'Left Shift',341:'Left Ctrl',342:'Left Alt',343:'Left Super',
      344:'Right Shift',345:'Right Ctrl',346:'Right Alt',347:'Right Super',348:'Menu'};
    const label = (key: number) => labels[key] ?? (key >= 65 && key <= 90 || key >= 48 && key <= 57
      ? String.fromCharCode(key) : key >= 290 && key <= 313 ? `F${key-289}` : `Key ${key}`);
    for (let row=0;row<17;row++) {
      const at = y + 54 + row*21;
      this.assets.font(ctx,'teammeatfont12',names[row]!,x+92,at,{color:'#362f2d'});
      for (let column=0;column<2;column++) {
        const selected = state.controlsMenu.selection === row && state.controlsMenu.column === column;
        const text = selected && state.controlsMenu.waiting ? 'Press key' : label(state.keyboard.bindings[actions[row]!]![column]!);
        //Native column1 is selected by Left; column0 by Right.
        this.assets.font(ctx,'teammeatfont12',`${selected ? '> ' : ''}${text}`,x+(column === 1 ? 235 : 335),at,
          {color:selected ? '#8c2a24' : '#362f2d'});
      }
    }
    const row = state.controlsMenu.selection;
    this.assets.draw(ctx,path,'Idle',x,y,0,{layers:[6],overrides:{6:{YPosition:row === 17 ? 452 : 70+row*21}}});
  }

  private main(ctx: CanvasRenderingContext2D, state: State, x: number, y: number): void {
    this.assets.draw(ctx, `${MENU}gamemenu.anm2`, "Idle", x, y, state.mainSelection, {
      //5A464A..4692 sets layer2 independently of cursor layer6:
      //saved-valid selects its actual frame0, absent selects frame1.
      overrides: {
        2: { XCrop: state.savedRunSummary === null ? 160 : 32, YCrop: 336 },
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
