import { Assets } from "./animation";
import unlockCatalogue from "./generated/unlock-notifications/catalogue.json";
import { drawBossSplash } from "./boss-splash";
import { MenuRenderer } from "./menu";
import { GLMenuContext, type GLSurface } from "./gl-menu";
import { StageComposer, type StageTargets } from "./gl-stage";
import { RunPresentation } from "./presentation";
import { PickupRenderer } from "./pickups";
import pickupCatalog from "./pickup-data.json";
import {drawPlayerLaser} from "./lasers";
import type { ActorSprite, Room, RoomAppearance, RoomQuad, Slot, State } from "./engine";

const PLAYER = "gfx/001.000_player.anm2";
const TEAR = "gfx/002.000_tear.anm2";
const PROGRESS = "gfx/ui/stage/progress.anm2";
const PARTICLE_ACTORS: Record<number, string> = {
  5: "gfx/1000.005_blood particle.anm2", 27: "gfx/1000.027_wood particle.anm2", 43: "gfx/1000.043_poop explosion.anm2",
  35: "gfx/1000.035_tooth particle.anm2", 58: "gfx/1000.058_poop particle.anm2",
};
// Rock Init 56ed00 / Render 56bf80: an intact (state 1) rock-family grid
// entity sets frame variant of its type's grid_rock.anm2 animation.
// Minimap room-type icons from the jump table at 0x678748; challenge rooms
// (11) use the ambush icon unless the game's boss-challenge flag is set.
const MINIMAP_ICONS: Record<number, string> = {
  2: "IconShop", 4: "IconTreasureRoom", 5: "IconBoss", 6: "IconMiniboss", 7: "IconSecretRoom",
  8: "IconSuperSecretRoom", 9: "IconArcade", 10: "IconCurseRoom", 11: "IconAmbushRoom", 12: "IconLibrary",
  13: "IconSacrificeRoom", 15: "IconAngelRoom", 18: "IconIsaacsRoom", 19: "IconBarrenRoom",
  20: "IconChestRoom", 21: "IconDiceRoom",
};
// Mask 0x3c1204: shown as locked while only shadow-revealed and unvisited.
// Minimap pickup icon 5af6df: saved type5 records set one bit per kind and
// the first set bit in draw order wins. Hearts prefer eternal, then black
// over soul/red, then soul; chests prefer locked (gold), then red.
function minimapPickupIcons(records: readonly { variant: number; subtype: number }[]): string[] {
  let heart = 0, chest = 0;
  const kinds = new Set<string>();
  for (const { variant, subtype } of records) {
    if (variant === 10) {
      kinds.add("heart");
      if (subtype === 4) heart = 4;
      else if (subtype === 6 && (heart === 0 || heart === 3)) heart = 6;
      else if (subtype === 3 && heart === 0) heart = 3;
    } else if (variant === 100 || variant === 350) kinds.add("item");
    else if (variant === 50 || variant === 60 || variant === 360) {
      kinds.add("chest");
      if (variant === 60) chest = 60;
      else if (variant === 360 && chest !== 60) chest = 360;
    } else if (variant === 300) kinds.add("card");
    else if (variant === 70) kinds.add("pill");
    else if (variant === 30) kinds.add("key");
    else if (variant === 40) kinds.add("bomb");
    else if (variant === 20) kinds.add("coin");
    else if (variant === 90) kinds.add("battery");
  }
  const icons: string[] = [];
  if (kinds.has("heart")) icons.push({ 4: "IconWhiteHeart", 6: "IconBlackHeart", 3: "IconBlueHeart" }[heart] ?? "IconHeart");
  if (kinds.has("item")) icons.push("IconItem");
  if (kinds.has("chest")) icons.push(chest === 60 ? "IconGoldChest" : chest === 360 ? "IconRedChest" : "IconChest");
  const order: [string, string][] = [["card", "IconCard"], ["pill", "IconPill"], ["key", "IconKey"],
    ["bomb", "IconBomb"], ["coin", "IconCoin"], ["battery", "IconBattery"]];
  for (const [kind, icon] of order) if (kinds.has(kind)) icons.push(icon);
  return icons;
}
// Base Rebirth's four shapes: bit 1 tall, bit 2 wide (1x1, 1x2, 2x1, 2x2).
function minimapShape(template: { width: number; height: number }): number {
  return (template.width === 26 ? 2 : 0) + (template.height === 14 ? 1 : 0);
}
const LOCKED_MINIMAP_TYPES = new Set([2, 9, 12, 18, 19, 20, 21]);
// GridEntity_Decoration Init 561e60 picks Props_*.anm2 by backdrop type
// (3/4 Caves, 5/6 Depths, 7/8 Womb, 9 Sheol, 10 Cathedral, 12 Chest, else
// Basement). Only the backdrops the port builds are listed.
const BASEMENT_PROPS = { actor: "gfx/grid/props_01_basement.anm2", allowCaves: false };
const DECORATION_PROPS: Record<string, { actor: string; allowCaves: boolean }> = {
  "gfx/backdrop/01_basement.png": BASEMENT_PROPS,
  "gfx/backdrop/02_cellar.png": BASEMENT_PROPS,
  "gfx/backdrop/03_caves.png": { actor: "gfx/grid/props_03_caves.anm2", allowCaves: true },
  "gfx/backdrop/05_depths.png": { actor: "gfx/grid/props_05_depths.anm2", allowCaves: false },
  "gfx/backdrop/09_sheol.png": { actor: "gfx/grid/props_09_sheol.anm2", allowCaves: false },
  ...Object.fromEntries(["0a_library", "0b_shop", "0c_isaacsroom", "0d_barrenroom", "0e_arcade", "0e_diceroom", "0f_secretroom"]
    .map(name => [`gfx/backdrop/${name}.png`, BASEMENT_PROPS])),
};
// 561f07: an RNG seeded (Descriptor+48 + grid index, shift35 = 5/9/7) plays
// clip raw() % count, retrying (20 tries) while a "Caves" clip lands outside
// the Caves backdrop.
function decorationClip(names: readonly string[], seed: number, allowCaves: boolean): string {
  let state = seed >>> 0, name = names[0]!;
  for (let attempt = 0; attempt < 20; attempt++) {
    state = (state ^ (state >>> 5)) >>> 0;
    state = (state ^ (state << 9)) >>> 0;
    state = (state ^ (state >>> 7)) >>> 0;
    name = names[state % names.length]!;
    if (allowCaves || !name.includes("Caves")) break;
  }
  return name;
}
const ROCK_ANIMATIONS: Record<number, string> = { 2: "normal", 3: "black", 4: "tinted", 5: "bombrock", 6: "alt" };
// Rock Init (56ee03..56f076) replaces grid_rock.anm2's layer-0 sheet by
// backdrop (jump table 0x674158: 3 Caves, 5/6 Depths); Basement keeps the
// authored rocks_basement.png.
const ROCK_SHEETS: Record<string, string> = {
  "gfx/backdrop/03_caves.png": "gfx/grid/rocks_caves.png",
  "gfx/backdrop/05_depths.png": "gfx/grid/rocks_depths.png",
};
// GridEntity_Pit Init 568c00..568c29 replaces grid_pit.anm2's sheet on
// backdrops 5 and 9; Basement and Caves keep the authored grid_pit.png.
const PIT_SHEETS: Record<string, string> = {
  "gfx/backdrop/05_depths.png": "gfx/grid/grid_pit_depths.png",
  "gfx/backdrop/09_sheol.png": "gfx/grid/grid_pit_depths.png",
};
function rockSprite(kind: number, variant: number): { animation: string; frame: number } | undefined {
  const animation = ROCK_ANIMATIONS[kind];
  if (!animation) return undefined;
  return kind === 2 && variant >= 1000 ? { animation: "big", frame: variant - 1000 } : { animation, frame: variant };
}
const directionName = (direction: string) => direction[0]!.toUpperCase() + direction.slice(1);

export class Renderer {
  private readonly context: CanvasRenderingContext2D;
  private readonly menu: MenuRenderer;
  private readonly stage: StageComposer;
  private readonly presentation: RunPresentation;
  private readonly pickups: PickupRenderer;
  private readonly floor: GLSurface;
  private readonly walls: GLSurface;
  private readonly smallMap: GLSurface;
  private readonly expandedMap: GLSurface;
  private roomKey = "";
  private paintedStains = { count: 0, last: "" };
  // Room-transition snapshots of the old and new world layers.
  private readonly slideOld: GLSurface;
  private readonly slideNew: GLSurface;
  private slideKey: string | null = null;
  private slideNewReady = false;
  private slideRecord: { npcs: Renderer["drawnNpcs"]; grid: Renderer["drawnGrid"]; order: Renderer["drawnOrder"] } = { npcs: [], grid: [], order: [] };
  // Sprite scale of every NPC the latest frame drew, for live render contracts.
  private drawnNpcs: { kind: number; spawnSeed: number; scale: [number, number] }[] = [];
  private drawnGrid: { index: number; kind: number; animation: string; frame: number }[] = [];
  // [entity type, spawn seed] of the player (seed null) and NPCs, in draw order.
  private drawnOrder: [number, number | null][] = [];
  // Actual submitted map composites, for the existing public capture owner.
  private drawnMaps: {surface: "small-frame" | "small" | "expanded"; alpha: number;
    position: [number, number]; sourceSize?: [number, number]; destinationSize?: [number, number]}[] = [];
  private constructor(private readonly canvas: HTMLCanvasElement, private readonly assets: Assets,
    private readonly graphics: GLMenuContext) {
    this.context = canvas.getContext("2d", { alpha: false })!;
    this.menu = new MenuRenderer(assets);
    this.stage = new StageComposer(graphics, assets);
    this.presentation = new RunPresentation(assets);
    this.pickups = new PickupRenderer(assets);
    this.floor = graphics.createTarget(1024, 512);
    const screen = graphics.screenSurface();
    this.smallMap = graphics.createTarget(screen.width, screen.height);
    this.expandedMap = graphics.createTarget(230, 204);
    this.slideOld = graphics.createTarget(screen.width, screen.height);
    this.slideNew = graphics.createTarget(screen.width, screen.height);
    this.walls = graphics.createTarget(1024, 512);
  }
  static async load(canvas: HTMLCanvasElement): Promise<Renderer> {
    const assets = await Assets.load();
    const graphics = await GLMenuContext.load(assets, canvas.width, canvas.height, assets.sources().map(source => source.path));
    return new Renderer(canvas, assets, graphics);
  }
  draw(state: State): void {
    this.drawnNpcs = []; this.drawnGrid = []; this.drawnOrder = []; this.drawnMaps = [];
    const gl = this.graphics;
    if (state.screen === "playing") {
      this.prepareRoom(state.room);
      this.stage.prepare(state.stageEffects, this.worldOffset(state));
    }
    gl.beginScreen(); gl.resetTransform(); gl.globalAlpha = 1; gl.imageSmoothingEnabled = false;
    gl.clearRect(0, 0, this.canvas.width, this.canvas.height);
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
    gl.scale(this.canvas.width / 480, this.canvas.height / 270);
    const ctx = gl.asCanvasContext();
    if (state.nightmares.systemActive) {
      if (state.nightmares.definitionLoaded && state.nightmares.currentActor) {
        this.assets.draw(ctx, state.nightmares.currentActor, "Scene", 240, 135, state.nightmares.frame);
        if (state.nightmares.progressDefinitionLoaded) this.nightmareProgress(ctx, state);
      }
    }
    else if (state.screen !== "playing") this.menu.draw(ctx, state);
    else {
      this.drawWorldOrSlide(ctx, state);
      this.presentation.introduction(ctx, state);
      this.hud(ctx, state);
      this.presentation.bossBar(ctx, state);
      const info = state.informationAnimation;
      if (info.state === 2 && info.definitionLoaded && info.currentActor && info.currentClip)
        this.assets.draw(ctx, info.currentActor, info.currentClip, 240, 135, info.frame,
          { spritesheets: info.spritesheet ? { 0: info.spritesheet } : {} });
      const bossScene = state.roomTransition?.bossScene;
      if (bossScene && state.roomTransition?.phase === "bossScene")
        drawBossSplash(this.assets, ctx, state);
      // StageTransition::Render5e3bd0: black quad, alpha counter/50 then 1-counter/50.
      const transition = state.stageTransition;
      if (transition.state === 2 || transition.state === 3)
        gl.fillRectRGBA(0, 0, 480, 270, [0, 0, 0, transition.state === 2 ? transition.counter / 50 : 1 - transition.counter / 50]);
      this.presentation.pause(ctx, state);
      if (state.playerDeath.gameOver) {
        gl.fillRectRGBA(0, 0, 480, 270, [0, 0, 0, 128 / 255]);
        this.assets.draw(ctx, "gfx/ui/death screen.anm2", "Diary", 256, 151, 0,
          { layers: [0, 1, 2, 4] });
        if (state.playerDeath.gameOver.damageFlags & 0x80)
          this.assets.draw(ctx, "gfx/ui/death screen.anm2", "Diary", 256, 151, 66, { layers: [5] });
        this.assets.draw(ctx, "gfx/ui/backselectwidget.anm2", "Exit", 10, 215, 0);
        this.assets.draw(ctx, "gfx/ui/backselectwidget.anm2", "Restart", 400, 195, 0);
      }
      //GameRender550350 draws the outgoing black quad after Pause/HUD.
      if (state.runAdmission.initialFade <= 0 && state.runQuit.fadeAlpha > 0)
        gl.fillRectRGBA(0, 0, 480, 270, [0, 0, 0, state.runQuit.fadeAlpha]);
      //GameRender550450..4DE overlays the initial Game+10EB14 fade last,
      //then593580 draws the same loading texture retained at menu confirmation.
      if (state.runAdmission.initialFade > 0) {
        const alpha = Math.min(1, state.runAdmission.initialFade);
        gl.fillRectRGBA(0, 0, 480, 270, [0, 0, 0, alpha]);
        this.menu.drawLoadingImage(ctx, state, alpha);
      }
    }
    //System593CE4/412240 follows Game/Menu draw, before the System output pass.
    //The manager survives Stop/NewRun; its loading byte alone gates this draw.
    const notification = state.unlockNotifications;
    if (notification?.state && notification.definitionLoaded && !state.unlockNotificationsLoading && notification.currentClip) {
      const actor = unlockCatalogue.actor;
      const sheet = this.assets.layerSpritesheet(actor, 3);
      this.assets.draw(ctx, actor, notification.currentClip, 240, 135, notification.frame,
        notification.gfx ? {spritesheets: {[sheet]: notification.gfx}} : {hiddenLayers: [3]});
    }
    gl.presentScreen(state.gammaSettings.gamma); gl.finish();
    this.context.reset(); this.context.imageSmoothingEnabled = false;
    this.context.drawImage(gl.canvas, 0, 0);
  }
  /** Entity_Laser::Render 479c20, two-layer variant path 47a650: the tip
   * layer as an ordinary ANM2 draw at the mouth, then (length > 15) one body
   * quad from 16 px past the mouth to the end, whose V runs 0.65*length
   * texels over the repeating 64-texel layer-1 column (SetWrap GL_REPEAT). */
  private npcLaserView(laser: State["lasers"][number]) {
    return { presentedPosition: laser.position, positionOffset: laser.renderOffset,
      rotation: laser.sprite.rotation, scale: laser.sprite.scale, actor: laser.actor,
      animation: laser.sprite.animation, animationClock: laser.sprite.frame,
      angle: laser.angle, length: laser.beamLength };
  }
  private drawLaser(ctx: CanvasRenderingContext2D, laser: ReturnType<Renderer["npcLaserView"]>): void {
    const origin = { x: laser.presentedPosition.x + laser.positionOffset.x, y: laser.presentedPosition.y + laser.positionOffset.y };
    const screen = (x: number, y: number): [number, number] => [Math.fround(x * 0.65 + 32), Math.fround(y * 0.65 - 46)];
    const s0 = screen(origin.x, origin.y);
    ctx.save();
    ctx.translate(s0[0], s0[1]);
    ctx.rotate(laser.rotation * Math.PI / 180);
    this.assets.draw(ctx, laser.actor, laser.animation, 0, 0, laser.animationClock,
      { layers: [0], layerScales: { 0: [laser.scale.x, laser.scale.y] } });
    ctx.restore();
    if (!(laser.length > 15)) return;
    const tip = this.assets.layerFrame(laser.actor, laser.animation, 0, laser.animationClock);
    const body = this.assets.layerFrame(laser.actor, laser.animation, 1, laser.animationClock);
    const backend = ctx as unknown as { drawTintedImageQuad?: (source: CanvasImageSource, tint: number[], offsets: number[], crop: readonly [number, number, number, number], points: readonly (readonly [number, number])[], pixelSize?: number) => void };
    if (!tip || !body || !backend.drawTintedImageQuad) return;
    const radians = laser.angle * Math.PI / 180, dir = [Math.cos(radians), Math.sin(radians)] as const;
    const along = [dir[0] * 0.5 * tip.Height!, dir[1] * 0.5 * tip.Height!];
    const reach = laser.length - 0.5 * tip.Height! / 0.65;
    const s1 = screen(origin.x + dir[0] * reach, origin.y + dir[1] * reach);
    const half = 0.5 * body.Width! * body.XScale * laser.scale.x, side = [dir[1] * half, -dir[0] * half];
    const a0 = [s0[0] + along[0] - side[0], s0[1] + along[1] - side[1]];
    const u = [2 * side[0], 2 * side[1]];
    const a1 = [s1[0] + along[0] - side[0], s1[1] + along[1] - side[1]];
    const texels = 0.65 * laser.length, v = [(a1[0] - a0[0]) / texels, (a1[1] - a0[1]) / texels];
    const point = (fu: number, tv: number): [number, number] => [a0[0] + u[0] * fu + v[0] * tv, a0[1] + u[1] * fu + v[1] * tv];
    const sheet = this.assets.layerImage(laser.actor, 1);
    const tint = [body.RedTint, body.GreenTint, body.BlueTint, body.AlphaTint];
    const offsets = [body.RedOffset, body.GreenOffset, body.BlueOffset];
    const column = sheet.height;
    for (let start = 0; start < texels; start += column) {
      const end = Math.min(texels, start + column);
      backend.drawTintedImageQuad(sheet, tint, offsets, [body.XCrop!, body.YCrop!, body.Width!, end - start],
        [point(0, start), point(1, start), point(0, end), point(1, end)]);
    }
  }
  /** Source CEA00 is a pixel offset, independent of Room15B8 scrolling. */
  private worldOffset(state: State): { x: number; y: number } | undefined {
    const scroll = state.camera?.scroll, shake = state.gameShake?.offsetCea00;
    if (scroll && shake) return { x: Math.fround(scroll.x + shake.x), y: Math.fround(scroll.y + shake.y) };
    return shake ?? scroll;
  }
  /** Room::Render world layer: backdrop, grid, actors and stage composites. */
  private drawWorld(ctx: CanvasRenderingContext2D, state: State, hidePlayer = false): void {
    ctx.save();
    const offset = this.worldOffset(state);
    if (offset) ctx.translate(offset.x, offset.y);
    this.drawRoom(state.room.appearance, state.room.grid, state);
    ctx.restore();
    this.stage.ground(state.stageEffects);
    ctx.save();
    if (offset) ctx.translate(offset.x, offset.y);
    // Creep's render key (seq & 0x3ff) + 2100 sorts every pool under the
    // y-sorted actors, in spawn order; colour multiplies the clip's pulse.
    for (const effect of state.spriteEffects) {
      if (!effect.creepEffect || !effect.visible || !effect.active) continue;
      const sprite = effect.sprite;
      ctx.save();
      ctx.translate(effect.presentedPosition.x * 0.65 + 32, effect.presentedPosition.y * 0.65 - 46);
      ctx.scale(sprite.scale.x, sprite.scale.y);
      const actor = effect.variant === 23 ? "gfx/1000.023_creep (green).anm2"
        : effect.variant === 24 ? "gfx/1000.024_creep (yellow).anm2" : "gfx/1000.022_creep (red).anm2";
      this.assets.draw(ctx, actor, sprite.animation, 0, 0, sprite.frame,
        { color: { tint: sprite.tint, offsets: sprite.offsets } });
      ctx.restore();
    }
    const grid = state.room.grid;
    if (grid) for (const [index, cell] of grid.cells.entries()) {
      const entity = cell.entity;
      if (!entity) continue;
      const x = (40 + index % grid.width * 40) * 0.65 + 32;
      // Grid Render projects through WorldToScreen 42eb00 (0.65y - 47);
      // the -46 actor anchor sits one logical pixel lower.
      const y = (120 + Math.floor(index / grid.width) * 40) * 0.65 - 47;
      const rock = entity.state === 1 ? rockSprite(entity.kind, entity.variant) : undefined;
      if (entity.kind === 7) {
        // GridEntity_Pit 5684c0 stores its autotile frame as the variant.
        const pitSheet = PIT_SHEETS[state.room.appearance.backdrop];
        this.assets.draw(ctx, "gfx/grid/grid_pit.anm2", "pit", x, y, entity.variant,
          pitSheet ? { spritesheets: { 0: pitSheet } } : {});
        this.drawnGrid.push({ index, kind: 7, animation: "pit", frame: entity.variant });
      } else if (rock) {
        const rockSheet = ROCK_SHEETS[state.room.appearance.backdrop];
        this.assets.draw(ctx, "gfx/grid/grid_rock.anm2", rock.animation, x, y, rock.frame,
          rockSheet ? { spritesheets: { 0: rockSheet } } : {});
        this.drawnGrid.push({ index, kind: entity.kind, ...rock });
      } else if (entity.kind === 12 && entity.sourceOwner?.tnt) {
        const sprite = entity.sourceOwner.tnt.sprite;
        // TNT owns four frame0 grid clips; its callback never advances ANM.
        const crop = { Idle: [0, 0], IdleMedium: [32, 0], ReadyToExplode: [0, 32], Blown: [32, 32] }[sprite.animation]!;
        ctx.drawImage(this.assets.image("gfx/grid/grid_tnt.png"), crop[0]!, crop[1]!, 32, 32, x - 16, y - 16, 32, 32);
        this.drawnGrid.push({ index, kind: 12, animation: sprite.animation, frame: sprite.frame });
      } else if (entity.kind === 14 && entity.variant === 0 && entity.sourceOwner?.poopSheet) {
        // Poop.Init569600 picks grid_poop_{1..3} and plays State{state+1}
        // (State5 once broken); a spawn into a loaded room plays Appear first.
        const sheet = { 0: `gfx/grid/grid_poop_${entity.sourceOwner.poopSheet}.png`, 1: "gfx/effects/effect_poopspawn.png" };
        const birth = entity.sourceOwner.birthGameUpdate, init = state.room.initializationGameUpdate;
        const age = birth !== null && init !== null && birth > init ? state.gameUpdateCount - birth : Infinity;
        const animation = age < 20 ? "Appear" : `State${Math.min(entity.state + 1, 5)}`;
        const frame = age < 20 ? age : entity.state === 0 ? 0 : 4;
        this.assets.draw(ctx, "gfx/grid/grid_poop.anm2", animation, x, y, frame, { spritesheets: sheet });
        this.drawnGrid.push({ index, kind: 14, animation, frame });
      } else if (entity.kind === 8 && entity.state === 0)
        this.assets.draw(ctx, "gfx/grid/grid_spikes.anm2", `Spikes${String(entity.variant).padStart(2, "0")}`, x, y);
    }
    const trapdoor = state.room.trapdoor;
    if (grid && trapdoor)
      this.assets.draw(ctx, "gfx/grid/door_11_trapdoor.anm2", trapdoor.clip === "closed" ? "Closed" : "Open Animation",
        (40 + trapdoor.gridIndex % grid.width * 40) * 0.65 + 32,
        (120 + Math.floor(trapdoor.gridIndex / grid.width) * 40) * 0.65 - 47, Math.floor(trapdoor.frame));
    this.pickups.draw(ctx, state);
    // Room::Render walks the manager render array in order (452850/4528c0),
    // so an entry later in it draws over an earlier one. The engine keeps the
    // Player/NPC part of that array (Scheduler::sort_render); NPCs after the
    // player draw right after him, over his sprite.
    const order = state.actorScheduler.render;
    const slot = (npc: State["room"]["npcs"][number]) => order.findIndex(actor => actor.kind === "npc" && actor.id === npc.id);
    const playerSlot = hidePlayer ? -1 : order.findIndex(actor => actor.kind === "player");
    const npcs = [...state.room.npcs].sort((a, b) => slot(a) - slot(b));
    const overPlayer = playerSlot < 0 ? [] : npcs.filter(npc => slot(npc) > playerSlot);
    // Machines and beggars share the actor render array (441bd0 y key).
    const machineSlot = (machine: Slot) => order.findIndex(actor => actor.kind === "slot" && actor.id === machine.id);
    const machines = [...state.room.slots].sort((a, b) => machineSlot(a) - machineSlot(b));
    const machinesOverPlayer = playerSlot < 0 ? [] : machines.filter(machine => machineSlot(machine) > playerSlot);
    for (const machine of machines) if (!machinesOverPlayer.includes(machine)) this.drawSlot(ctx, machine);
    for (const npc of npcs) if (!overPlayer.includes(npc)) this.drawNpc(ctx, npc);
    for (const knife of state.knives) {
      if (!knife.visible) continue;
      const sprite = knife.sprite, color = knife.currentColor, flips = sprite.layerFlips?.[0] ?? [false, false];
      ctx.save();
      ctx.translate((knife.position.x + knife.renderOffset.x + sprite.offset.x) * 0.65 + 32,
        (knife.position.y + knife.renderOffset.y + sprite.offset.y) * 0.65 - 46);
      ctx.rotate(sprite.rotation * Math.PI / 180);
      ctx.scale(flips[0] ? -1 : 1, flips[1] ? -1 : 1);
      this.assets.draw(ctx, "gfx/008.000_moms knife.anm2", sprite.animation, 0, 0, sprite.frame,
        {scale: 0.65, layerScales: {0: [sprite.scale.x, sprite.scale.y]}, overrides: {0: {
          RedTint: color.tint[0]! * 255, GreenTint: color.tint[1]! * 255,
          BlueTint: color.tint[2]! * 255, AlphaTint: color.tint[3]! * 255,
          RedOffset: color.offsets[0]! * 255, GreenOffset: color.offsets[1]! * 255,
          BlueOffset: color.offsets[2]! * 255,
        }}});
      ctx.restore();
    }
    for (const familiar of state.familiars) {
      if (!familiar.visible || !familiar.active || familiar.removed) continue;
      ctx.save();
      ctx.translate(familiar.presentedPosition.x * 0.65 + 32,
        familiar.presentedPosition.y * 0.65 - 46);
      ctx.scale(familiar.sprite.flipX ? -1 : 1, 1);
      this.assets.draw(ctx, "gfx/003.056_leech.anm2", familiar.sprite.animation,
        familiar.sprite.offset.x * 0.65, familiar.sprite.offset.y * 0.65, familiar.sprite.frame);
      ctx.restore();
    }
    for (const laser of state.lasers) {
      if (laser.origin === "npcShootAngle") {
      if (!laser.active) continue;
      // The impact effect keys 500 below its laser, so it draws under the beam.
      const impact = laser.impact;
      if (impact && !impact.removed) {
        ctx.save();
        ctx.translate((impact.position.x + impact.positionOffset.x) * 0.65 + 32, (impact.position.y + impact.positionOffset.y) * 0.65 - 46);
        ctx.rotate(impact.rotation * Math.PI / 180);
        ctx.scale(impact.scale.x, impact.scale.y);
        this.assets.draw(ctx, impact.actor, impact.animation, 0, 0, impact.animationClock);
        ctx.restore();
      }
      this.drawLaser(ctx, this.npcLaserView(laser));
        continue;
      }
      const impact=laser.impact;
      if (impact && !impact.removed && state.actorScheduler.actors.some(actor => actor.kind === "effect" && actor.id === impact.id)) {
        ctx.save();
        ctx.translate((impact.position.x+impact.positionOffset.x)*.65+32,(impact.position.y+impact.positionOffset.y)*.65-46);
        ctx.rotate(impact.rotation*Math.PI/180);ctx.scale(impact.scale.x,impact.scale.y);
        if (!impact.currentColor) throw new Error("Player Impact color history is absent");
        this.assets.draw(ctx,impact.actor,impact.animation,0,0,impact.animationClock,
          {color:impact.currentColor});
        ctx.restore();
      }
      if (laser.uninitialized !== false || !state.actorScheduler.actors.some(actor => actor.kind === "laser" && actor.id === laser.id)) continue;
      drawPlayerLaser(this.graphics,this.assets,laser,{width:480,height:270,pixelScale:this.canvas.width/480});
    }
    for (const projectile of state.enemyProjectiles)
      if (!projectile.removed && projectile.animation)
        this.assets.draw(ctx, projectile.variant === 3 ? "gfx/009.003_puke projectile.anm2"
          : projectile.variant === 2 ? "gfx/009.002_fire projectile.anm2"
          : projectile.variant === 1 ? "gfx/009.001_bone projectile.anm2" : "gfx/009.000_projectile.anm2", projectile.animation,
          projectile.presentedPosition.x * 0.65 + 32, (projectile.presentedPosition.y + projectile.height) * 0.65 - 46, projectile.animationFrame, { scale: projectile.scale });
    for (const particle of state.fireParticles)
      if (!particle.removed && particle.visible)
        this.assets.draw(ctx, "gfx/1000.066_ember particle.anm2", "Idle",
          particle.presentedPosition.x * 0.65 + 32, particle.presentedPosition.y * 0.65 - 46, particle.animationClock);
    if (!hidePlayer) this.drawPlayer(ctx, state);
    for (const machine of machinesOverPlayer) this.drawSlot(ctx, machine);
    for (const npc of overPlayer) this.drawNpc(ctx, npc);
    for (const tear of state.tears) {
      const color = tear.sourceColor;
      //The selected Regular/Blood authored layer receives its birth-owned ColorMod.
      //Keep all11 components in state; project tint and final RGB offsets.
      const blood = tear.sourceVariant === 1;
      this.assets.draw(ctx, blood ? "gfx/002.001_blood tear.anm2" : TEAR,
        `${blood ? "BloodTear" : "RegularTear"}${tear.animationTier}`, Math.round(tear.x * 2) / 2, Math.round(tear.y * 2) / 2, tear.age,
        { overrides: { 0: {
          RedTint: color.tint[0]! * 255, GreenTint: color.tint[1]! * 255,
          BlueTint: color.tint[2]! * 255, AlphaTint: color.tint[3]! * 255,
          RedOffset: color.offsets[0]! * 255, GreenOffset: color.offsets[1]! * 255,
          BlueOffset: color.offsets[2]! * 255,
        } } });
    }
    for (const bomb of state.bombs)
      if (bomb.motion.lifetime === "alive")
        this.assets.draw(ctx, bomb.variant === 3 ? "gfx/004.003_troll bomb.anm2" : bomb.variant === 4 ? "gfx/004.004_megatroll bomb.anm2" : "gfx/004.000_bomb.anm2", bomb.animation, bomb.x, bomb.y - 1, bomb.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
    for (const explosion of state.explosions) {
      // Owned rows only project the real Effect1 frame/light. Draw it once
      // through spriteEffects; old serialized adapter rows remain explicit.
      if (explosion.sourceEffect !== undefined) continue;
      this.assets.draw(ctx, "gfx/1000.001_bomb explosion.anm2", "Explosion", explosion.x, explosion.y - 1, explosion.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
    }
    for (const bomb of state.bombs)
      if (bomb.motion.lifetime === "exploding")
        this.assets.draw(ctx, bomb.variant === 3 ? "gfx/004.003_troll bomb.anm2" : bomb.variant === 4 ? "gfx/004.004_megatroll bomb.anm2" : "gfx/004.000_bomb.anm2", bomb.animation, bomb.x, bomb.y - 1, bomb.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
    for (const effect of state.spriteEffects) {
      // Source31f dead can coexist with31e active until a later Remove.
      if (!effect.visible || !effect.active || effect.footprint || effect.creepEffect) continue;
      if (effect.variant === 58 || effect.variant === 43 || effect.variant === 27 || effect.variant === 35 || effect.variant === 5) {
        const sprite = effect.sprite, controller = effect.poopEffect;
        const rotation = controller && "particle58" in controller && typeof controller.particle58 === "object"
          ? controller.particle58.flying.layerRotation
          : controller && "particle5" in controller && typeof controller.particle5.particle === "object"
            ? controller.particle5.particle.flying.layerRotation : 0;
        ctx.save();
        ctx.translate((effect.presentedPosition.x + effect.renderOffset.x + sprite.offset.x) * 0.65 + 32,
          (effect.presentedPosition.y + effect.renderOffset.y + sprite.offset.y) * 0.65 - 46);
        ctx.scale(sprite.flipX ? -1 : 1, sprite.flipY ? -1 : 1);
        this.assets.draw(ctx, PARTICLE_ACTORS[effect.variant],
          sprite.animation, 0, 0, sprite.frame, { layerScales: {0: [sprite.scale.x, sprite.scale.y]}, overrides: {0: {
            Rotation: rotation, RedTint: sprite.tint[0] * 255, GreenTint: sprite.tint[1] * 255,
            BlueTint: sprite.tint[2] * 255, AlphaTint: sprite.tint[3] * 255,
            RedOffset: sprite.offsets[0] * 255, GreenOffset: sprite.offsets[1] * 255, BlueOffset: sprite.offsets[2] * 255,
          }} });
        ctx.restore();
        continue;
      }
      if (effect.flyEffect) {
        // Duke effects and a dead fly's 1000.3 puff: the smoke draws 40 above
        // its ground (+458) with its colour multiplying every layer; a dying
        // tiny fly's Main layer falls onto its shadow (LayerState+0x30).
        const sprite = effect.sprite;
        const actor = effect.variant === 1 ? "gfx/1000.001_bomb explosion.anm2"
          : effect.variant === 2 ? "gfx/1000.002_blood explosion.anm2"
          : effect.variant === 3 ? "gfx/1000.003_fly explosion.anm2" : "gfx/1000.033_tiny fly.anm2";
        const fall = sprite.layerOffsets?.[0]?.[1];
        const layerScales = Object.fromEntries(Object.entries(sprite.layerFlips).map(([layer, flips]) =>
          [layer, [flips[0] ? -1 : 1, flips[1] ? -1 : 1] as const]));
        ctx.save();
        ctx.translate((effect.presentedPosition.x + effect.renderOffset.x) * 0.65 + 32,
          (effect.presentedPosition.y + effect.renderOffset.y) * 0.65 - 46);
        ctx.rotate(sprite.rotation * Math.PI / 180);
        ctx.scale(sprite.scale.x * (sprite.flipX ? -1 : 1), sprite.scale.y * (sprite.flipY ? -1 : 1));
        this.assets.draw(ctx, actor, sprite.animation, 0, 0, sprite.frame, {
          layerScales, color: { tint: sprite.tint, offsets: sprite.offsets },
          ...(fall === undefined ? {} : { overrides: { 0: { YPosition: -16 + fall } } }) });
        ctx.restore();
        continue;
      }
      this.drawEffect(ctx, effect.variant, effect.sprite,
        (effect.presentedPosition.x + effect.sprite.offset.x) * 0.65 + 32,
        (effect.presentedPosition.y + effect.sprite.offset.y) * 0.65 - 46);
    }
    ctx.restore();
    this.stage.foreground(state.stageEffects);
    this.drawFxRays(state);
  }

  /** Slot Render 5395b0 → base 442300: the sprite sits at position + PositionOffset (+458). */
  private drawSlot(ctx: CanvasRenderingContext2D, machine: Slot): void {
    this.assets.draw(ctx, machine.actor, machine.animation,
      (machine.presentedPosition.x + machine.positionOffset.x) * 0.65 + 32,
      (machine.presentedPosition.y + machine.positionOffset.y) * 0.65 - 46, machine.animationFrame);
  }

  private drawNpc(ctx: CanvasRenderingContext2D, npc: State["room"]["npcs"][number]): void {
    // A dying boss (state 16) keeps drawing its death clip.
    if (npc.removed || (npc.dead && !npc.monstro && !npc.dip && npc.state !== 16) || (!npc.keeper && !npc.visible)) return;
    // Sprite Scale (+2c4): 1.15 for champions, otherwise the fresh 1.
    const scale = npc.spriteScale ?? npc.champion?.scale ?? 1;
    // Sprite Color (+2d0): champion table colour or a timed hit flash.
    const color = { tint: npc.spriteColor.current.tint, offsets: npc.spriteColor.current.offset };
    this.drawnNpcs.push({ kind: npc.kind, spawnSeed: npc.spawnSeed, scale: [scale, scale] });
    this.drawnOrder.push([npc.kind, npc.spawnSeed]);
    const keeper = npc.keeper;
    if (keeper) {
      this.assets.draw(ctx, npc.variant === 0 ? "gfx/017.001_shopkeeper.anm2" : "gfx/017.002_secret room keeper.anm2", keeper.animation,
        npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46, keeper.animationClock,
        { scale, overrides: { 0: {
          RedTint: keeper.tint[0]! * 255, GreenTint: keeper.tint[1]! * 255, BlueTint: keeper.tint[2]! * 255,
          AlphaTint: keeper.tint[3]! * 255, RedOffset: keeper.offsets[0]! * 255,
          GreenOffset: keeper.offsets[1]! * 255, BlueOffset: keeper.offsets[2]! * 255,
        } } });
    } else if (npc.monstro) {
      ctx.save();
      ctx.translate(npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46);
      ctx.scale(npc.monstro.flipX ? -1 : 1, 1);
      this.assets.draw(ctx, "gfx/020.000_monstro.anm2", npc.animation,
        0, 0, npc.animationClock, { scale, color });
      ctx.restore();
    } else if (npc.dip) {
      ctx.save();
      ctx.translate(npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46);
      ctx.scale(npc.dip.flipX ? -1 : 1, 1);
      this.assets.draw(ctx, "gfx/216.000_dip.anm2", npc.animation, 0, 0, npc.animationClock,
        { scale, overrides: { 0: {
          RedTint: npc.dip.tint[0]! * 255, GreenTint: npc.dip.tint[1]! * 255,
          BlueTint: npc.dip.tint[2]! * 255, AlphaTint: npc.dip.tint[3]! * 255,
          RedOffset: npc.dip.offsets[0]! * 255, GreenOffset: npc.dip.offsets[1]! * 255,
          BlueOffset: npc.dip.offsets[2]! * 255,
        } } });
      ctx.restore();
    } else if (npc.fireplace) {
      this.assets.draw(ctx, "gfx/033.000_fireplace.anm2", npc.animation,
        npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46, npc.animationClock,
        { scale, layerScales: { 1: [npc.fireplace.flameScale, npc.fireplace.flameScale] } });
    } else if (npc.actor) {
      // Catalogue-driven families: the engine names the authored actor.
      ctx.save();
      ctx.translate(npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46);
      // Wall Creep: sprite rotation +2cc onto its wall, and +2fd on a side wall.
      // Mom's door parts turn +2cc to face the room from their doorway.
      const rotation = npc.wallCreep?.rotation ?? npc.mom?.rotation ?? 0;
      if (rotation) ctx.rotate(rotation * Math.PI / 180);
      ctx.scale(npc.flipX ? -1 : 1, npc.wallCreep?.flipY ? -1 : 1);
      this.assets.draw(ctx, npc.actor, npc.animation, 0, 0, npc.animationClock, { scale, color });
      // ANM2 overlay slot (Gemini's head) draws over the main clip.
      if (npc.overlay) this.assets.draw(ctx, npc.actor, npc.overlay.animation, 0, 0, npc.overlay.clock, { scale, color });
      ctx.restore();
    } else {
      this.assets.draw(ctx, npc.kind === 13 ? "gfx/013.000_fly.anm2" : npc.kind === 18 ? "gfx/018.000_attack fly.anm2" : npc.kind === 214 ? "gfx/213.000_level2fly.anm2" : "gfx/244.000_round worm.anm2", npc.animation,
        npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46, npc.animationFrame, { scale, color });
    }
  }

  /** FX ray render 547890 (from room render 5cd00d, after entities and
   * shading). The camera scroll (Room+15B8) moves the rays' anchor C; the
   * player's offset from the room centre P sets parallax and fades them. Each
   * ray draws its whole texture twice on one additive quad: alpha a*s, then
   * mirrored with a*(1-s), s = (sin(phase)+1)/2. Phase advances by
   * animationSpeed per render call; two calls per game update is assumed. */
  private drawFxRays(state: State): void {
    const group = state.room.appearance.fxRays, init = state.room.initializationGameUpdate;
    if (!group || !group.rays.length || init === null) return;
    const grid = state.room.grid; if (!grid) return;
    const scroll = state.camera?.scroll ?? { x: 0, y: 0 }, shake = state.gameShake?.offsetCea00 ?? { x: 0, y: 0 };
    const centre = { x: (40 + 40 * (grid.width - 1) / 2) * 0.65 + 32, y: (120 + 40 * (grid.height - 1) / 2) * 0.65 - 46 };
    const C = { x: centre.x + scroll.x, y: centre.y + scroll.y };
    const P = { x: state.player.x - centre.x, y: state.player.y - centre.y };
    const topY = scroll.y + shake.y, n = group.rays.length;
    const W = (group.xMax - group.xMin - (n - 1) * group.raySpacing) / n, h = W / 2;
    const fade = Math.max(1 - Math.abs(P.x) / C.x, 0) * Math.max(1 - Math.abs(P.y) / C.y, 0);
    const calls = 2 * Math.max(0, state.gameUpdateCount - init);
    const gl = this.graphics, ctx = gl.asCanvasContext();
    const backend = ctx as unknown as { drawTintedImageQuad?: (source: CanvasImageSource, tint: number[], offsets: number[], crop: readonly [number, number, number, number], points: readonly (readonly [number, number])[]) => void };
    if (!backend.drawTintedImageQuad) return;
    gl.blendFactors("srcAlpha", "one");
    for (const [i, ray] of group.rays.entries()) {
      const x = (W + group.raySpacing) * i + group.xMin, xc = x + h;
      const topLeft = [C.x + x + P.x * group.topParallax, topY] as const, topRight = [topLeft[0] + W, topY] as const;
      const bottomY = P.y * group.bottomParallax + group.rayLength;
      const bottomLeft = [group.perspective * xc + C.x - h + group.bottomParallax * P.x, bottomY] as const;
      const bottomRight = [group.perspective * xc + C.x + h + group.bottomParallax * P.x, bottomY] as const;
      const a = fade * ray.opacity, s = (Math.sin(ray.phase + ray.animationSpeed * calls) + 1) * 0.5;
      const image = this.assets.image(ray.texture), crop = [0, 0, image.width, image.height] as const;
      backend.drawTintedImageQuad(image, [1, 1, 1, a * s], [0, 0, 0], crop, [topLeft, topRight, bottomLeft, bottomRight]);
      backend.drawTintedImageQuad(image, [1, 1, 1, a * (1 - s)], [0, 0, 0], crop, [topRight, topLeft, bottomRight, bottomLeft]);
    }
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
  }

  /** Room transition 5de870: the old room (captured without the player) and
   * the new room slide one screen apart by smoothstep(progress), progress =
   * full updates (plus 0.5 on an interpolation frame) over a limit of 7. The
   * HUD and overlays stay fixed on top. */
  private drawWorldOrSlide(ctx: CanvasRenderingContext2D, state: State): void {
    const transition = state.roomTransition, gl = this.graphics;
    const key = transition && `${transition.fromRoom}:${transition.targetRoom}:${transition.fromSlot}`;
    if (transition?.phase === "out") {
      this.drawWorld(ctx, state, true);
      this.copyScreen(this.slideOld);
      this.slideKey = key; this.slideNewReady = false;
      this.drawnNpcs = []; this.drawnGrid = []; this.drawnOrder = [];
      gl.clearRect(0, 0, 480, 270);
    }
    if (!transition || transition.phase !== "in" || this.slideKey !== key || transition.fromSlot < 0) {
      this.drawWorld(ctx, state);
      return;
    }
    if (!this.slideNewReady) {
      this.drawWorld(ctx, state);
      this.copyScreen(this.slideNew);
      this.slideNewReady = true;
      this.slideRecord = { npcs: structuredClone(this.drawnNpcs), grid: structuredClone(this.drawnGrid), order: structuredClone(this.drawnOrder) };
    }
    // The new room shown during the slide is that one snapshot draw.
    this.drawnNpcs = structuredClone(this.slideRecord.npcs); this.drawnGrid = structuredClone(this.slideRecord.grid);
    this.drawnOrder = structuredClone(this.slideRecord.order);
    const limit = 7, k = transition.fullUpdates;
    const interpolated = state.systemClock.interpolationEnabled && state.systemClock.systemCounter % 2 === 0;
    const p = Math.fround((k + (interpolated && k < limit ? 0.5 : 0)) / limit);
    const e = Math.fround(p * p * Math.fround(3 - 2 * p));
    const [dx, dy] = [[1, 0], [0, 1], [-1, 0], [0, -1]][transition.fromSlot % 4]!;
    const old: [number, number] = [dx * e * 480, dy * e * 270];
    const next: [number, number] = [old[0] - dx * 480, old[1] - dy * 270];
    gl.clearRect(0, 0, 480, 270);
    gl.blendFactors("one", "oneMinusSrcAlpha");
    for (const [surface, [x, y]] of [[this.slideOld, old], [this.slideNew, next]] as const)
      gl.drawSurface(surface, 0, 0, surface.width, surface.height, x, y, 480, 270);
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
  }

  /** Copy the screen target (world layer so far) into a slide snapshot. */
  private copyScreen(target: GLSurface): void {
    const gl = this.graphics, screen = gl.screenSurface();
    gl.beginTarget(target); gl.resetTransform(); gl.globalAlpha = 1;
    gl.clearRect(0, 0, target.width, target.height);
    gl.blendFactors("one", "zero");
    gl.drawSurface(screen, 0, 0, screen.width, screen.height, 0, 0, target.width, target.height);
    gl.beginTarget(null); gl.resetTransform(); gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
    gl.scale(this.canvas.width / 480, this.canvas.height / 270);
  }
  debugTargets(): StageTargets { return this.stage.debugTargets(); }
  drawRecord() { return { npcs: structuredClone(this.drawnNpcs), grid: structuredClone(this.drawnGrid), order: structuredClone(this.drawnOrder), maps: structuredClone(this.drawnMaps) }; }
  private drawQuads(appearance: RoomAppearance, quads: RoomQuad[]): void {
    const gl = this.graphics, image = this.assets.image(appearance.backdrop);
    for (const quad of quads) {
      const source = quad.sourceRect, dest = quad.destRect;
      const sourceX = quad.sourcePatchIndex % appearance.backdropPatchColumns * appearance.backdropPatchSize.width + source.x;
      const sourceY = Math.floor(quad.sourcePatchIndex / appearance.backdropPatchColumns) * appearance.backdropPatchSize.height + source.y;
      gl.save(); gl.translate(dest.x + (quad.flipX ? dest.width : 0), dest.y + (quad.flipY ? dest.height : 0));
      gl.scale(quad.flipX ? -1 : 1, quad.flipY ? -1 : 1);
      gl.drawImage(image, sourceX, sourceY, source.width, source.height, 0, 0, dest.width, dest.height);
      gl.restore();
    }
  }
  /** Nightmare Render 5B76C0: the floor progress bar over the scene. The bar
   * sprite takes the scene's FadeGuide null color (5B774E..5B77EA). Icons sit
   * 27 apart at y 24 from x0 = 240 - (23n + 4(n - 1)) / 2 + 11.5. */
  private nightmareProgress(ctx: CanvasRenderingContext2D, state: State): void {
    const night = state.nightmares, actor = night.currentActor!;
    const guide = this.assets.nullFrame(actor, "Scene", 0, Math.floor(night.frame));
    const color = guide
      ? { tint: [guide.RedTint, guide.GreenTint, guide.BlueTint, guide.AlphaTint], offsets: [guide.RedOffset, guide.GreenOffset, guide.BlueOffset] }
      : { tint: [1, 1, 1, 1], offsets: [0, 0, 0] };
    const f = Math.fround, step = 27, y = 24, current = night.stageIndex, floors = night.progressFloors;
    const count = current + 1 < floors ? floors : current + 1;
    const x0 = f(f(240 - f(f(count * 23 + (count - 1) * 4) * 0.5)) + 11.5);
    const draw = (animation: string, x: number, frame = 0) =>
      this.assets.draw(ctx, PROGRESS, animation, x, y, frame, { color });
    const at = (index: number) => f(f(index * step) + x0);
    for (let index = 0; index < count - 1; index++) draw("Connector", f(f(index * step) + f(x0 + 13.5)));
    // Icon i shows resource[i + 1]; the tenth icon reads resource[11].
    for (let index = 0; index <= current; index++)
      draw("Levels", at(index), night.stageResources[index === 9 ? 11 : index + 1] ?? 12);
    for (let index = 0; index < current; index++) draw("ClearFloor", at(index));
    for (let index = current + 1; index < count; index++) draw("NotClearFloor", at(index));
    if (current < floors) {
      draw("BossIndicator", f(x0 + (floors === 6 ? 135 : 189)));
      // Game+CE9F4 against the 36000-frame limit at CE9F8 (Game init 54E95C).
      const limit = 36000, time = state.gameUpdateCount;
      if (current <= 5 && time <= limit) draw("Clock", f(f(f(time / limit) * 135) + x0));
    }
    // The head walks in from the previous icon over scene frames 10..40
    // unless the scene repeats the same stage.
    let x = at(current);
    if (!night.alternate) {
      const frame = Math.min(Math.floor(night.frame), 40);
      x = f(x - (frame > 10 ? f(f(1 - f((frame - 10) / 30)) * step) : step));
    }
    draw("IsaacIndicator", x, Math.floor(night.counter / 2) % 22);
  }
  private drawEffect(ctx: CanvasRenderingContext2D, variant: number, sprite: ActorSprite, x: number, y: number): void {
    const actors: Record<number, string> = { 1: "gfx/1000.001_bomb explosion.anm2",
      2: "gfx/1000.002_blood explosion.anm2", 7: "gfx/1000.007_blood splat.anm2",
      12: "gfx/1000.012_tear poofa.anm2", 13: "gfx/1000.013_tear poofb.anm2", 15: "gfx/1000.015_poof01.anm2",
      17: "gfx/1000.017_poof04.anm2", 18: "gfx/1000.018_bomb crater.anm2",
      21: "gfx/1000.021_tiny bug.anm2", 33: "gfx/1000.033_tiny fly.anm2",
      42: "gfx/1000.042_blood gush.anm2", 48: "gfx/1000.048_battery.anm2", 68: "gfx/1000.021_tiny bug.anm2",
      79: "gfx/1000.012_tear poof_small.anm2", 80: "gfx/1000.012_tear poof_verysmall.anm2" };
    const actor = actors[variant];
    if (!actor) throw new Error(`Unadmitted effect renderer ${variant}`);
    ctx.save(); ctx.translate(x, y); ctx.rotate(sprite.rotation * Math.PI / 180);
    ctx.scale(sprite.scale.x * (sprite.flipX ? -1 : 1), sprite.scale.y * (sprite.flipY ? -1 : 1));
    const layerScales = Object.fromEntries(Object.entries(sprite.layerFlips).map(([layer, flips]) =>
      [layer, [flips[0] ? -1 : 1, flips[1] ? -1 : 1] as const]));
    // Sprite color multiplies each keyframe (Color::operator*= 42f590). Poof01
    // authors a 158 grey tint and an alpha fade from frame 3 to 24; replacing
    // its keyframe color drew the spawn smoke light, opaque and longer-lived.
    const colorOptions = variant === 1 || variant === 15
      ? { color: { tint: sprite.tint, offsets: sprite.offsets } }
      : { overrides: { 0: {
        RedTint: sprite.tint[0] * 255, GreenTint: sprite.tint[1] * 255,
        BlueTint: sprite.tint[2] * 255, AlphaTint: sprite.tint[3] * 255,
        RedOffset: sprite.offsets[0] * 255, GreenOffset: sprite.offsets[1] * 255,
        BlueOffset: sprite.offsets[2] * 255 } } };
    this.assets.draw(ctx, actor, sprite.animation, 0, 0, sprite.frame,
      { layerScales, ...colorOptions });
    ctx.restore();
  }
  private prepareRoom(room: Room): void {
    const appearance = room.appearance;
    const key = JSON.stringify({ backdrop: appearance.backdrop, patchSize: appearance.backdropPatchSize,
      patchColumns: appearance.backdropPatchColumns, floor: appearance.floorQuads, walls: appearance.wallQuads,
      columns: appearance.columns, rows: appearance.rows, tileSize: appearance.tileSize, tutorial: appearance.tutorial,
      origin: appearance.origin });
    // Background stains are append-only within a room (familiars.rs pushes,
    // a new room starts empty), so an unchanged prefix only needs the new
    // stains painted over the existing floor target.
    const stains = room.backgroundStains, painted = this.paintedStains;
    const continues = key === this.roomKey && stains.length >= painted.count
      && (painted.count === 0 || JSON.stringify(stains[painted.count - 1]) === painted.last);
    if (continues && stains.length === painted.count) return;
    const gl = this.graphics, width = appearance.columns * appearance.tileSize, height = appearance.rows * appearance.tileSize;
    gl.beginTarget(this.floor, width, height); gl.resetTransform(); gl.globalAlpha = 1; gl.imageSmoothingEnabled = false;
    if (continues) {
      gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
      this.paintStains(appearance, stains.slice(painted.count));
      this.paintedStains = { count: stains.length, last: JSON.stringify(stains[stains.length - 1]) };
      return;
    }
    gl.clearRect(0, 0, this.floor.width, this.floor.height); gl.blendFactors("one", "zero");
    this.drawQuads(appearance, appearance.floorQuads);
    if (appearance.tutorial) {
      const controls = this.assets.image("gfx/backdrop/controls.png");
      gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
      gl.drawImage(controls, Math.round(width / 2 - controls.width / 2), Math.round(height / 2 - controls.height / 2));
    }
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
    this.paintStains(appearance, stains);
    gl.beginTarget(this.walls, width, height); gl.resetTransform();
    gl.clearRect(0, 0, this.walls.width, this.walls.height); gl.blendFactors("one", "zero");
    this.drawQuads(appearance, appearance.wallQuads);
    this.roomKey = key;
    this.paintedStains = { count: stains.length, last: stains.length ? JSON.stringify(stains[stains.length - 1]) : "" };
  }
  private paintStains(appearance: RoomAppearance, stains: Room["backgroundStains"]): void {
    const ctx = this.graphics.asCanvasContext();
    for (const stain of stains) {
      // Footprints take their color from floor pixels the runtime does not
      // sample yet; painting them with zero offsets would draw black marks.
      if (stain.footprint) continue;
      const sprite = stain.sprite;
      if (stain.variant === 58 || stain.variant === 43 || stain.variant === 27) {
        ctx.save();
        ctx.translate((stain.position.x + stain.renderOffset.x + sprite.offset.x) * 0.65 + 32 - appearance.origin.x,
          (stain.position.y + stain.renderOffset.y + sprite.offset.y) * 0.65 - 46 - appearance.origin.y);
        ctx.scale(sprite.flipX ? -1 : 1, sprite.flipY ? -1 : 1);
        const actor = PARTICLE_ACTORS[stain.variant];
        this.assets.draw(ctx, actor, sprite.animation, 0, 0, sprite.frame,
          { layerScales: {0: [sprite.scale.x, sprite.scale.y]}, overrides: { 0: {
            ...(stain.layerRotation === null ? {} : {Rotation: stain.layerRotation}),
            RedTint: sprite.tint[0] * 255, GreenTint: sprite.tint[1] * 255,
            BlueTint: sprite.tint[2] * 255, AlphaTint: sprite.tint[3] * 255,
            RedOffset: sprite.offsets[0] * 255, GreenOffset: sprite.offsets[1] * 255,
            BlueOffset: sprite.offsets[2] * 255,
          } } });
        ctx.restore();
      } else {
        this.drawEffect(ctx, stain.variant, sprite,
          (stain.position.x + sprite.offset.x) * 0.65 + 32 - appearance.origin.x,
          (stain.position.y + sprite.offset.y) * 0.65 - 46 - appearance.origin.y);
      }
    }
  }
  /** GridEntity_Decoration props draw with the backdrop, under the room's
   * shading quad, as the original's darker props show. */
  private drawDecorations(ctx: CanvasRenderingContext2D, state: State): void {
    const grid = state.room.grid, descriptor = state.room.descriptor;
    const props = DECORATION_PROPS[state.room.appearance.backdrop];
    if (!grid || !descriptor || !props || descriptor.template.type === 16) return;
    for (const [index, cell] of grid.cells.entries()) {
      if (cell.entity?.kind !== 1) continue;
      const x = (40 + index % grid.width * 40) * 0.65 + 32, y = (120 + Math.floor(index / grid.width) * 40) * 0.65 - 47;
      const animation = decorationClip(this.assets.animationNames(props.actor), descriptor.decorationSeed + index, props.allowCaves);
      this.assets.draw(ctx, props.actor, animation, x, y, state.gameUpdateCount);
      this.drawnGrid.push({ index, kind: 1, animation, frame: state.gameUpdateCount });
    }
  }
  private drawRoom(appearance: RoomAppearance, grid: Room["grid"], state: State): void {
    const gl = this.graphics, ctx = gl.asCanvasContext(), size = appearance.tileSize, border = size * 2;
    const width = appearance.columns * size, height = appearance.rows * size, { x, y } = appearance.origin;
    // Backdrop Render427420/427990 copies the composed floor/walls with ONE/ZERO.
    // Alpha blending again would multiply cached tutorial/stain alpha twice.
    gl.blendFactors("one", "zero");
    gl.drawSurface(this.floor, border, border, width - border * 2, height - border - size,
      x + border, y + border, width - border * 2, height - border - size);
    gl.drawSurface(this.walls, 0, 0, border, height, x, y, border, height);
    gl.drawSurface(this.walls, width - border, 0, border, height, x + width - border, y, border, height);
    gl.drawSurface(this.walls, border, 0, width - border * 2, border, x + border, y, width - border * 2, border);
    gl.drawSurface(this.walls, border, height - border, width - border * 2, border,
      x + border, y + height - border, width - border * 2, border);
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
    const rotation = { up: 0, left: -Math.PI / 2, right: Math.PI / 2, down: Math.PI };
    for (const connection of appearance.doors) {
      ctx.save(); ctx.translate(connection.x, connection.y); ctx.rotate(rotation[connection.side]);
      this.assets.draw(ctx, connection.asset, connection.state, 0, 0, 0,
        connection.spritesheet ? { spritesheets: { 0: connection.spritesheet } } : {});
      // Door.LoadGraphics5629c0 overlays door_17_bardoor on barred variant4;
      // AnimState563b70 holds its Idle while closed. The Vanish fade on
      // opening is not yet modelled.
      const index = grid ? Math.floor((connection.worldPosition.x - 40) / 40 + 0.5)
        + Math.floor((connection.worldPosition.y - 120) / 40 + 0.5) * grid.width : -1;
      const entity = grid?.cells[index]?.entity;
      if (entity?.kind === 16 && entity.variant === 4 && entity.state !== 2) {
        this.assets.draw(ctx, "gfx/grid/door_17_bardoor.anm2", "Idle", 0, 0);
        this.drawnGrid.push({ index, kind: 16, animation: "Idle", frame: 0 });
      }
      // A cracked bedroom door (variant3) shows door_18 Idle while intact and
      // Damaged after its first blast (state4); both clips are one frame.
      if (entity?.kind === 16 && entity.variant === 3) {
        const animation = entity.state === 4 ? "Damaged" : "Idle";
        this.assets.draw(ctx, "gfx/grid/door_18_crackeddoor.anm2", animation, 0, 0);
        this.drawnGrid.push({ index, kind: 16, animation, frame: 0 });
      }
      ctx.restore();
    }
    this.drawDecorations(ctx, state);
    gl.drawImage(this.assets.image(appearance.fx.shading), x, y, width, height);
  }
  private drawPlayer(ctx: CanvasRenderingContext2D, state: State): void {
    if (!state.player.visible) return;
    this.drawnOrder.push([1, null]);
    const player = state.player, x = Math.round((player.x + player.renderOffset.x) * 2) / 2,
      y = Math.round((player.y + player.renderOffset.y) * 2) / 2;
    const [RedTint, GreenTint, BlueTint] = state.stageEffects.actorTint;
    if (state.playerAnimation.current) {
      this.assets.draw(ctx, PLAYER, state.playerAnimation.current, x, y, state.playerAnimation.frame,
        { overrides: Object.fromEntries(Array.from({length: 5}, (_, layer) => [layer, {RedTint, GreenTint, BlueTint}])) });
      if (state.playerAnimation.current === "UseItem") this.pickups.drawCarried(ctx, state, x, y);
      return;
    }
    if (state.playerItems?.pickupAnimationGuard) {
      this.assets.draw(ctx, PLAYER, "Pickup", x, y, state.playerItems.pickupAnimationFrame,
        { overrides: Object.fromEntries(Array.from({length: 5}, (_, layer) => [layer, {RedTint, GreenTint, BlueTint}])) });
      this.pickups.drawCarried(ctx, state, x, y);
      return;
    }
    this.assets.draw(ctx, PLAYER, `Walk${directionName(player.bodyFacing)}`, x, y, player.walkFrame,
      { layers: [1], overrides: { 1: { RedTint, GreenTint, BlueTint } } });
    const costume = state.playerItems?.costume229, head = costume?.slots[4];
    if (costume && costume.historyFrontier === null && !costume.mainClipFrontier
      && head?.layer === 0 && !head.bodyGroup) {
      //The actual selected head pose owns its clip/frame. Normal Isaac and
      //this costume both retain skin-1, so50F206 keeps the authored sheet.
      this.assets.draw(ctx, "gfx/characters/229_monstros lung.anm2",
        costume.sprite.clip, x, y, costume.sprite.frame,
        { layers: [0], overrides: { 0: { RedTint, GreenTint, BlueTint } } });
    } else {
      this.assets.draw(ctx, PLAYER, `Head${directionName(player.facing)}`, x, y, player.headFrame,
        { layers: [4], overrides: { 4: { RedTint, GreenTint, BlueTint } } });
    }
  }
  private hud(ctx: CanvasRenderingContext2D, state: State): void {
    this.activeItem(ctx, state);
    this.pocketItems(ctx, state);
    for (let heart = 0; heart < Math.ceil(state.player.maxHp / 2); heart++) {
      const remaining = state.player.hp - heart * 2;
      this.assets.draw(ctx, "gfx/ui/ui_hearts.anm2", remaining >= 2 ? "RedHeartFull" : remaining === 1 ? "RedHeartHalf" : "EmptyHeart", 68 + heart * 12, 24);
    }
    const redSlots = Math.ceil(state.player.maxHp / 2);
    for (let heart = 0; heart < Math.ceil(state.player.soulHp / 2); heart++) {
      const color = state.player.blackHeartMask & (1 << heart) ? "Black" : "Blue";
      const amount = state.player.soulHp - heart * 2 >= 2 ? "Full" : "Half";
      this.assets.draw(ctx, "gfx/ui/ui_hearts.anm2", `${color}Heart${amount}`, 68 + (redSlots + heart) * 12, 24);
    }
    if (state.player.eternalHearts > 0) {
      this.assets.draw(ctx, "gfx/ui/ui_hearts.anm2", redSlots ? "WhiteHeartOverlay" : "WhiteHeartHalf", 68 + Math.max(0, redSlots - 1) * 12, 24);
    }
    const counts = [state.inventory.coins, state.inventory.bombs, state.inventory.keys];
    // HUD5741F9 selects golden frame3 from actual Player+BD0; count stays BCC.
    const pickupFrames = [0, 2, state.player.goldenKey === true ? 3 : 1];
    for (let pickup = 0; pickup < 3; pickup++) {
      this.assets.draw(ctx, "gfx/ui/hudpickups.anm2", "Idle", 20, 44 + pickup * 12, pickupFrames[pickup]!);
      this.assets.font(ctx, "pftempestasevencondensed", String(counts[pickup]).padStart(2, "0"), 36, 45 + pickup * 12);
    }
    // HUD5742BD: one icon sits at (24, 84); the difficulty icon and the
    // seeded icon together sit at (20, 84) and (36, 84).
    const difficultyIcon = state.difficultyPresentation.hudIcon;
    const seededIcon = state.achievementEligibility ? null : 5;
    const icons = [difficultyIcon, seededIcon].filter((frame): frame is number => frame !== null);
    icons.forEach((frame, index) => this.assets.draw(ctx, "gfx/ui/hudpickups.anm2", "Idle",
      icons.length === 1 ? 24 : 20 + index * 16, 84, frame));
    this.drawMaps(ctx, state);
  }

  /** HUD::Render573B8F: the held active's 32x32 art at (22,12) over a
   * 30% black copy at +2,+2, a 1px white cross outline at full charge, then
   * ui_chargebar at (56,29): BarEmpty, BarFull clipped from the top by the
   * charge (red offset for Battery reserve) unless FlashChargeBar's countdown
   * is even, and the BarOverlayN divider for the authored maximum. */
  private activeItem(ctx: CanvasRenderingContext2D, state: State): void {
    const owner = state.playerItems;
    if (!owner?.activeId) return;
    const item = pickupCatalog.items.find(entry => entry.id === owner.activeId && entry.kind === "active");
    if (!item?.sprite) return;
    const maximum = item.maxCharges ?? 0, [x, y] = [22, 12];
    this.assets.drawImage(ctx, item.sprite, x + 2, y + 2, { tint: [0, 0, 0, 0.3], offsets: [0, 0, 0] });
    if (maximum > 0 && owner.activeCharge === maximum) {
      for (const [dx, dy] of [[-1, 0], [0, -1], [0, 1], [1, 0]] as const)
        this.assets.drawImage(ctx, item.sprite, x + dx, y + dy, { tint: [0, 0, 0, 1], offsets: [1, 1, 1] });
    }
    this.assets.drawImage(ctx, item.sprite, x, y);
    if (maximum <= 0) return;
    const bar = "gfx/ui/ui_chargebar.anm2", [barX, barY] = [56, 29];
    const charge = owner.activeSubordinateCharge !== 0 ? owner.activeSubordinateCharge / 450 : owner.activeCharge / maximum;
    const reserve = Math.min(1, owner.reserveCharge / maximum);
    this.assets.draw(ctx, bar, "BarEmpty", barX, barY);
    const flash = state.chargeBarFlash;
    if (flash < 0 || flash % 2 === 1) {
      // AnimationState::Render's top-left clamp removes (1-ratio)*23+3 rows.
      const clipped = (ratio: number) => {
        const rows = (1 - ratio) * 23 + 3;
        return { 0: { YCrop: rows, Height: 32 - rows, YPivot: 16 - rows } };
      };
      this.assets.draw(ctx, bar, "BarFull", barX, barY, 0, { overrides: clipped(charge) });
      if (reserve > 0) this.assets.draw(ctx, bar, "BarFull", barX, barY, 0,
        { overrides: clipped(reserve), color: { tint: [1, 1, 1, 1], offsets: [1, 0, 0] } });
    }
    const overlay = [1, 2, 3, 4, 6, 12].includes(maximum) ? `BarOverlay${maximum}` : null;
    if (overlay) this.assets.draw(ctx, bar, overlay, barX, barY);
  }

  /** HUD::Render 574BA2..5756E9: pocket slot 0 (Player+3044, card tag +3048)
   * at (W-28,H-18) = (452,252). A card with a CardFronts face (its config
   * +18 hud) draws that face 3 to the left; runes, faceless cards and pills
   * draw their ui_cardspills frame. The card name (config +10; the +8
   * description while Map is held; "???" in seed mode 29, and for a card in
   * mode 30, which also drops the face) is drawn right-aligned 16 left of the
   * slot and 10 up, plus the Game+CEA00 shake. Mode 9 rewrites the name
   * (5759E2), not ported: no text. Pill names need the Game+95B0 identified
   * byte, which the port does not own, so a pill draws without text outside
   * mode 29. Slot 1 (+304C/+3050) draws its Small frame at (W-24,H-34). */
  private pocketItems(ctx: CanvasRenderingContext2D, state: State): void {
    const held = state.playerItems?.heldPockets;
    if (!held) return;
    const sheet = "gfx/ui/ui_cardspills.anm2";
    const cardFrame = (id: number, small: string): [string, number] =>
      id <= 22 ? [`Cards${small}`, 0] : id <= 27 ? [`Cards${small}`, 1]
        : id <= 31 ? [`Runes${small}`, 0] : id <= 35 ? [`Runes${small}`, 1] : [`Cards${small}`, 1];
    const mode = state.runSeeds?.sourceEffectMode ?? null;
    const [x, y] = [480 - 28, 270 - 18];
    const slot = held.slots[0]!;
    if (slot.id !== 0) {
      let name: string | null = null;
      if (!slot.card) {
        this.assets.draw(ctx, sheet, "Pills", x, y, slot.id - 1);
        if (mode === 29) name = "???";
      } else {
        const card = pickupCatalog.cards.find(entry => entry.id === slot.id);
        if (card?.hud && mode !== 30) this.assets.draw(ctx, "gfx/ui/ui_cardfronts.anm2", card.hud, x - 3, y, 0);
        else {
          const [animation, frame] = cardFrame(slot.id, "");
          this.assets.draw(ctx, sheet, animation, x, y, frame);
        }
        name = mode === 29 || mode === 30 ? "???"
          : card ? (state.publicActions.held[13] ? card.description : card.name) : null;
      }
      if (name && mode !== 9) {
        const shake = state.gameShake?.offsetCea00 ?? { x: 0, y: 0 };
        this.assets.font(ctx, "pftempestasevencondensed", name, x - 16 + shake.x, y - 10 + shake.y, { align: "right" });
      }
    }
    const second = held.slots[1]!;
    if (second.id !== 0) {
      const [animation, frame] = second.card ? cardFrame(second.id, "Small") : ["PillsSmall", second.id - 1];
      this.assets.draw(ctx, sheet, animation, 480 - 24, 270 - 34, frame);
    }
  }

  private drawMaps(ctx: CanvasRenderingContext2D, state: State): void {
    const gl = this.graphics, {smallAlpha, expandedAlpha} = state.minimapPresentation;
    //5DE820 is1 for the port's admitted ordinary doorway kind0/nonnegative
    //target. Special transition kinds/target-1 need their own source fields;
    //do not guess a fade from the unrelated fullUpdates projection.
    if (smallAlpha > 0) {
      ctx.save(); ctx.globalAlpha *= smallAlpha;
      this.assets.draw(ctx, "gfx/ui/minimap1.anm2", "Frame", 400, 16);
      ctx.restore();
      this.drawnMaps.push({surface: "small-frame", alpha: Math.fround(gl.globalAlpha * smallAlpha), position: [400, 16]});
      //The source applies final alpha once to the completed map texture.
      //Keep the existing small-map geometry and icon scope independent.
      gl.save(); gl.beginTarget(this.smallMap); gl.resetTransform(); gl.globalAlpha = 1;
      gl.clearRect(0, 0, this.smallMap.width, this.smallMap.height);
      gl.scale(this.canvas.width / 480, this.canvas.height / 270);
      this.drawMinimap(gl.asCanvasContext(), state);
      gl.beginScreen(); gl.restore();
      gl.drawSurface(this.smallMap, 0, 0, this.smallMap.width, this.smallMap.height,
        0, 0, 480, 270, [1, 1, 1, smallAlpha]);
      this.drawnMaps.push({surface: "small", alpha: Math.fround(gl.globalAlpha * smallAlpha), position: [0, 0],
        sourceSize: [this.smallMap.width, this.smallMap.height], destinationSize: [480, 270]});
    }
    if (expandedAlpha > 0) {
      gl.save(); gl.beginTarget(this.expandedMap); gl.resetTransform(); gl.globalAlpha = 1;
      gl.clearRect(0, 0, 230, 204);
      const bounds = this.drawExpandedMap(gl.asCanvasContext(), state);
      gl.beginScreen(); gl.restore();
      if (bounds) {
        //5B1ED0: default manager offset/shake0 and scale1; occupied-content
        //bounds align the surface to the top/right. The expanded map has no Frame.
        gl.drawSurface(this.expandedMap, 0, 0, 230, 204,
          480 - bounds.maxX - 24, 14 - bounds.minY, 230, 204, [1, 1, 1, expandedAlpha]);
        this.drawnMaps.push({surface: "expanded", alpha: Math.fround(gl.globalAlpha * expandedAlpha),
          position: [480 - bounds.maxX - 24, 14 - bounds.minY], sourceSize: [230, 204], destinationSize: [230, 204]});
      }
      //5B213A joins5B1811 after the expanded surface: the same alpha
      //also owns the signed Game counter's30Hz Time text. Font5B0E16
      //is TeamMeatFont10;5B18F5 uses logical y30 and exact half-width.
      const counter = state.gameUpdateCount | 0;
      const time = [Math.trunc(counter / 108000), Math.trunc((counter % 108000) / 1800), Math.trunc((counter % 1800) / 30)]
        .map(value => String(value).padStart(2, "0")).join(":");
      ctx.save(); ctx.globalAlpha *= expandedAlpha;
      this.assets.font(ctx, "teammeatfont10", `Time: ${time}`, 240, 30, { align: "center" });
      ctx.restore();
    }
  }

  private drawExpandedMap(ctx: CanvasRenderingContext2D, state: State): {minY: number; maxX: number} | null {
    const path = "gfx/ui/minimap2.anm2", descriptors = state.floor.descriptors;
    const cells: {index: number; flags: number; origin: boolean; frame: number; x: number; y: number}[] = [];
    let minY = Infinity, maxX = -Infinity;
    for (let row = 0; row < 13; row++) for (let column = 0; column < 13; column++) {
      const grid = row * 13 + column, index = state.floor.grid[grid] ?? -1;
      const flags = index < 0 ? 0 : state.roomDisplayFlags[index] ?? 0;
      if (!flags) continue;
      const room = descriptors[index]!;
      const offset = grid - room.gridIndex, down = Math.floor(offset / 13), across = offset - 13 * down;
      const shape = minimapShape(room.template);
      const frame = shape === 1 ? down + 1 : shape === 2 ? across + 3 : shape === 3 ? across + 2 * down + 5 : 0;
      const x = column * 17, y = row * 15;
      cells.push({index, flags, origin: offset === 0, frame, x, y});
      minY = Math.min(minY, y); maxX = Math.max(maxX, x + 26);
    }
    for (const cell of cells) this.assets.draw(ctx, path, "RoomOutline", cell.x, cell.y);
    for (const cell of cells) {
      const type = descriptors[cell.index]!.template.type;
      const animation = cell.index === state.currentRoom ? "RoomCurrent"
        : state.visitedRooms.includes(cell.index) ? "RoomVisited"
        : cell.flags & 1 && type !== 7 && type !== 8 ? "RoomUnvisited" : null;
      if (animation) this.assets.draw(ctx, path, animation, cell.x, cell.y, cell.frame);
    }
    for (const cell of cells) {
      if (!cell.origin) continue;
      const room = descriptors[cell.index]!, type = room.template.type;
      const icon = cell.flags & 4 ? MINIMAP_ICONS[type] : undefined;
      const locked = !icon && cell.flags & 2 && !state.visitedRooms.includes(cell.index) && LOCKED_MINIMAP_TYPES.has(type);
      const icons = icon ? [icon] : locked ? ["IconLockedRoom"] : [];
      //Unlike the small cap1, special/locked rooms may also carry pickups.
      //Only Shop excludes pickup scanning. This preserves the existing admitted
      //regular-pickup record family; generic/nonzero native varData stays open.
      const saved = cell.index === state.currentRoom ? state.room.savedRegularPickups
        : state.persistedRooms[cell.index]?.savedRegularPickups ?? [];
      if (type !== 2) icons.push(...minimapPickupIcons(saved));
      const selected = icons.slice(0, 4);
      const shape = minimapShape(room.template);
      const qx = cell.x + 5 + (shape & 2 ? 8 : 0), qy = cell.y + 5 + (shape & 1 ? 7 : 0);
      const slots = selected.length === 1 ? [[8, 7]] : selected.length === 2 ? [[4, 7], [12, 7]]
        : [[4, 3.5], [12, 3.5], [4, 10.5], [12, 10.5]];
      for (const [index, name] of selected.entries()) {
        const [sx, sy] = slots[index]!;
        this.assets.draw(ctx, path, name, Math.floor(qx + sx! - 6), Math.floor(qy + sy! - 6));
      }
    }
    return cells.length ? {minY, maxX} : null;
  }

  // Minimap render 5aeb10: a 7x7 grid-cell window around the current room's
  // origin, clamped to the 13x13 grid, stepping (7+1, 6+1) from (396, 12).
  // Outlines go under every flagged cell, unvisited secret rooms draw no cell,
  // multi-cell rooms pick per-cell frames by shape, and one icon (the
  // constructor's limit) is centred on the room's origin cell.
  private drawMinimap(ctx: CanvasRenderingContext2D, state: State) {
    const descriptors = state.floor.descriptors;
    const current = descriptors[state.currentRoom]!.gridIndex;
    const left = Math.min(Math.max(current % 13 - 3, 0), 6);
    const top = Math.min(Math.max(Math.floor(current / 13) - 3, 0), 6);
    // A multi-cell current room shifts the view back half a cell, (3.5, 3).
    const currentShape = minimapShape(descriptors[state.currentRoom]!.template);
    const originX = 396 - (currentShape & 2 ? 3.5 : 0), originY = 12 - (currentShape & 1 ? 3 : 0);
    const cells: { index: number; flags: number; origin: boolean; frame: number; x: number; y: number }[] = [];
    for (let row = 0; row < 7; row++) for (let column = 0; column < 7; column++) {
      const cell = (top + row) * 13 + left + column;
      const index = state.floor.grid[cell] ?? -1;
      const flags = index < 0 ? 0 : state.roomDisplayFlags[index] ?? 0;
      if (!flags) continue;
      const room = descriptors[index]!;
      const offset = cell - room.gridIndex, down = Math.floor(offset / 13), across = offset - 13 * down;
      const shape = minimapShape(room.template);
      const frame = shape === 1 ? down + 1 : shape === 2 ? across + 3 : shape === 3 ? across + 2 * down + 5 : 0;
      cells.push({ index, flags, origin: offset === 0, frame, x: originX + 8 * column, y: originY + 7 * row });
    }
    for (const cell of cells) this.assets.draw(ctx, "gfx/ui/minimap1.anm2", "RoomOutline", Math.round(cell.x), cell.y);
    for (const cell of cells) {
      const type = descriptors[cell.index]!.template.type;
      const animation = cell.index === state.currentRoom ? "RoomCurrent"
        : state.visitedRooms.includes(cell.index) ? "RoomVisited"
        : cell.flags & 1 && type !== 7 && type !== 8 ? "RoomUnvisited" : null;
      if (animation) this.assets.draw(ctx, "gfx/ui/minimap1.anm2", animation, Math.round(cell.x), cell.y, cell.frame);
    }
    for (const cell of cells) {
      if (!cell.origin) continue;
      const room = descriptors[cell.index]!, type = room.template.type;
      const icon = cell.flags & 4 ? MINIMAP_ICONS[type] : undefined;
      const locked = !icon && cell.flags & 2 && !state.visitedRooms.includes(cell.index) && LOCKED_MINIMAP_TYPES.has(type);
      // Without a room icon, a non-shop room shows its saved pickups' icon
      // (Descriptor+70..78 records; one icon by the constructor's limit).
      const saved = cell.index === state.currentRoom ? state.room.savedRegularPickups
        : state.persistedRooms[cell.index]?.savedRegularPickups ?? [];
      const pickup = icon || locked || type === 2 ? undefined : minimapPickupIcons(saved)[0];
      if (!icon && !locked && !pickup) continue;
      // Multi-cell rooms shift the icon half a cell (3.5, 3) towards the room's centre.
      const shape = minimapShape(room.template);
      const x = cell.x + (shape & 2 ? 3.5 : 0), y = cell.y + (shape & 1 ? 3 : 0);
      this.assets.draw(ctx, "gfx/ui/minimap1.anm2", icon ?? pickup ?? "IconLockedRoom", Math.round(x), y);
    }
  }
}
