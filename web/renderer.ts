import { Assets, type SpriteColor } from "./animation";
import { MenuRenderer } from "./menu";
import { GLMenuContext, type GLSurface } from "./gl-menu";
import { StageComposer, type StageTargets } from "./gl-stage";
import { RunPresentation } from "./presentation";
import { PickupRenderer } from "./pickups";
import type { ActorSprite, Room, RoomAppearance, RoomQuad, State } from "./engine";

const PLAYER = "gfx/001.000_player.anm2";
const TEAR = "gfx/002.000_tear.anm2";
// Static champion Colors at 0x8ec700 (built 0x40aa42..0x40ac23): tint RGBA
// and offset RGB. ResetColor 47f430 copies the entry into the sprite color.
const CHAMPION_OFFSET = 200 / 255;
const CHAMPION_COLORS: readonly SpriteColor[] = [
  [1, .1, .1, 1], [.6, .6, 0, 1], [.1, .8, .2, 1], [1, .6, .2, 1], [.2, .2, 1, 1], [0, .15, .15, 1],
  [1, 1, 1, 1, CHAMPION_OFFSET], [.5, .5, .5, 1], [1, 1, 1, .3, CHAMPION_OFFSET], [0, 0, 0, 1],
  [1, 0, 1, 1], [.75, 0, 1, 1], [.5, 0, 0, 1], [.5, .5, 1, 1],
].map(([r, g, b, a, offset = 0]) => ({ tint: [r!, g!, b!, a!].map(Math.fround), offsets: [offset, offset, offset].map(Math.fround) }));
// Rock Init 56ed00 / Render 56bf80: an intact (state 1) rock-family grid
// entity sets frame variant of its type's grid_rock.anm2 animation.
const ROCK_ANIMATIONS: Record<number, string> = { 2: "normal", 3: "black", 4: "tinted", 5: "bombrock", 6: "alt" };
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
  private roomKey = "";
  // Sprite scale of every NPC the latest frame drew, for live render contracts.
  private drawnNpcs: { kind: number; spawnSeed: number; scale: [number, number] }[] = [];
  private drawnGrid: { index: number; kind: number; animation: string; frame: number }[] = [];
  private constructor(private readonly canvas: HTMLCanvasElement, private readonly assets: Assets,
    private readonly graphics: GLMenuContext) {
    this.context = canvas.getContext("2d", { alpha: false })!;
    this.menu = new MenuRenderer(assets);
    this.stage = new StageComposer(graphics, assets);
    this.presentation = new RunPresentation(assets);
    this.pickups = new PickupRenderer(assets);
    this.floor = graphics.createTarget(1024, 512);
    this.walls = graphics.createTarget(1024, 512);
  }
  static async load(canvas: HTMLCanvasElement): Promise<Renderer> {
    const assets = await Assets.load();
    const graphics = await GLMenuContext.load(assets, canvas.width, canvas.height, assets.sources().map(source => source.path));
    return new Renderer(canvas, assets, graphics);
  }
  draw(state: State): void {
    this.drawnNpcs = []; this.drawnGrid = [];
    const gl = this.graphics;
    if (state.screen === "playing") {
      this.prepareRoom(state.room);
      this.stage.prepare(state.stageEffects, state.camera?.scroll);
    }
    gl.beginScreen(); gl.resetTransform(); gl.globalAlpha = 1; gl.imageSmoothingEnabled = false;
    gl.clearRect(0, 0, this.canvas.width, this.canvas.height);
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
    gl.scale(this.canvas.width / 480, this.canvas.height / 270);
    const ctx = gl.asCanvasContext();
    if (state.nightmares.systemActive) {
      if (state.nightmares.definitionLoaded && state.nightmares.currentActor)
        this.assets.draw(ctx, state.nightmares.currentActor, "Scene", 240, 135, state.nightmares.frame);
    }
    else if (state.screen !== "playing") this.menu.draw(ctx, state);
    else {
      ctx.save();
      if (state.camera) ctx.translate(state.camera.scroll.x, state.camera.scroll.y);
      this.drawRoom(state.room.appearance);
      ctx.restore();
      this.stage.ground(state.stageEffects);
      ctx.save();
      if (state.camera) ctx.translate(state.camera.scroll.x, state.camera.scroll.y);
      const grid = state.room.grid;
      if (grid) for (const [index, cell] of grid.cells.entries()) {
        const entity = cell.entity;
        if (!entity) continue;
        const x = (40 + index % grid.width * 40) * 0.65 + 32;
        const y = (120 + Math.floor(index / grid.width) * 40) * 0.65 - 46;
        const rock = entity.state === 1 ? rockSprite(entity.kind, entity.variant) : undefined;
        if (entity.kind === 7) {
          // GridEntity_Pit 5684c0 stores its autotile frame as the variant.
          this.assets.draw(ctx, "gfx/grid/grid_pit.anm2", "pit", x, y, entity.variant);
          this.drawnGrid.push({ index, kind: 7, animation: "pit", frame: entity.variant });
        } else if (rock) {
          this.assets.draw(ctx, "gfx/grid/grid_rock.anm2", rock.animation, x, y, rock.frame);
          this.drawnGrid.push({ index, kind: entity.kind, ...rock });
        } else if (entity.kind === 8 && entity.state === 0)
          this.assets.draw(ctx, "gfx/grid/grid_spikes.anm2", `Spikes${String(entity.variant).padStart(2, "0")}`, x, y);
      }
      this.pickups.draw(ctx, state);
      for (const npc of state.room.npcs) {
        if (npc.removed || (npc.dead && !npc.monstro && !npc.dip) || (!npc.keeper && !npc.visible)) continue;
        // Sprite Scale (+2c4): 1.15 for champions, otherwise the fresh 1.
        const scale = npc.champion?.scale ?? 1;
        this.drawnNpcs.push({ kind: npc.kind, spawnSeed: npc.spawnSeed, scale: [scale, scale] });
        const keeper = npc.keeper;
        if (keeper) {
          this.assets.draw(ctx, "gfx/017.002_secret room keeper.anm2", keeper.animation,
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
            0, 0, npc.animationClock, { scale });
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
          ctx.scale(npc.flipX ? -1 : 1, 1);
          // Champions draw with their table Color and the ×1.15 sprite scale.
          this.assets.draw(ctx, npc.actor, npc.animation, 0, 0, npc.animationClock,
            { scale, color: npc.champion && CHAMPION_COLORS[npc.champion.color] });
          ctx.restore();
        } else {
          this.assets.draw(ctx, npc.kind === 13 ? "gfx/013.000_fly.anm2" : npc.kind === 18 ? "gfx/018.000_attack fly.anm2" : npc.kind === 214 ? "gfx/213.000_level2fly.anm2" : "gfx/244.000_round worm.anm2", npc.animation,
            npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46, npc.animationFrame, { scale });
        }
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
        if (laser.removed) continue;
        // Render479c20's quad builder is not decoded: tile the beam layer from
        // pos+offset along the sprite rotation and cap it with the tip layer.
        const start = { x: laser.presentedPosition.x + laser.positionOffset.x, y: laser.presentedPosition.y + laser.positionOffset.y };
        const body = this.assets.layerFrame(laser.actor, laser.animation, 1, laser.animationClock);
        const tile = body?.Height ? body.Height : 0;
        if (laser.length > 15 && tile > 0) {
          ctx.save();
          ctx.translate(start.x * 0.65 + 32, start.y * 0.65 - 46);
          ctx.rotate(laser.rotation * Math.PI / 180);
          for (let along = 0; along < laser.length; along += tile)
            this.assets.draw(ctx, laser.actor, laser.animation, 0, along * 0.65, laser.animationClock, { scale: 0.65, layers: [1], layerScales: { 1: [laser.scale.x, 1] } });
          this.assets.draw(ctx, laser.actor, laser.animation, 0, laser.length * 0.65, laser.animationClock, { scale: 0.65, layers: [0], layerScales: { 0: [laser.scale.x, 1] } });
          ctx.restore();
        }
        const impact = laser.impact;
        if (impact && !impact.removed) {
          ctx.save();
          ctx.translate((impact.position.x + impact.positionOffset.x) * 0.65 + 32, (impact.position.y + impact.positionOffset.y) * 0.65 - 46);
          ctx.rotate(impact.rotation * Math.PI / 180);
          ctx.scale(impact.scale.x, impact.scale.y);
          this.assets.draw(ctx, impact.actor, impact.animation, 0, 0, impact.animationClock);
          ctx.restore();
        }
      }
      for (const projectile of state.enemyProjectiles)
        if (!projectile.removed && projectile.animation)
          this.assets.draw(ctx, "gfx/009.000_projectile.anm2", projectile.animation,
            projectile.presentedPosition.x * 0.65 + 32, (projectile.presentedPosition.y + projectile.height) * 0.65 - 46, projectile.animationFrame, { scale: projectile.scale });
      for (const particle of state.fireParticles)
        if (!particle.removed && particle.visible)
          this.assets.draw(ctx, "gfx/1000.066_ember particle.anm2", "Idle",
            particle.presentedPosition.x * 0.65 + 32, particle.presentedPosition.y * 0.65 - 46, particle.animationClock);
      this.drawPlayer(ctx, state);
      for (const tear of state.tears)
        this.assets.draw(ctx, TEAR, `RegularTear${tear.animationTier}`, Math.round(tear.x * 2) / 2, Math.round(tear.y * 2) / 2, tear.age);
      for (const impact of state.impacts)
        this.assets.draw(ctx, "gfx/1000.012_tear poof_small.anm2", "Poof", impact.x, impact.y, impact.age);
      for (const bomb of state.bombs)
        if (bomb.motion.lifetime === "alive")
          this.assets.draw(ctx, bomb.variant === 3 ? "gfx/004.003_troll bomb.anm2" : bomb.variant === 4 ? "gfx/004.004_megatroll bomb.anm2" : "gfx/004.000_bomb.anm2", bomb.animation, bomb.x, bomb.y - 1, bomb.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
      for (const explosion of state.explosions) {
        this.assets.draw(ctx, "gfx/1000.001_bomb explosion.anm2", "Explosion", explosion.x, explosion.y - 1, explosion.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
      }
      for (const bomb of state.bombs)
        if (bomb.motion.lifetime === "exploding")
          this.assets.draw(ctx, bomb.variant === 3 ? "gfx/004.003_troll bomb.anm2" : bomb.variant === 4 ? "gfx/004.004_megatroll bomb.anm2" : "gfx/004.000_bomb.anm2", bomb.animation, bomb.x, bomb.y - 1, bomb.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
      for (const effect of state.spriteEffects) {
        // Source31f dead can coexist with31e active until a later Remove.
        if (!effect.visible || !effect.active) continue;
        if (effect.variant === 58 || effect.variant === 43) {
          const sprite = effect.sprite, controller = effect.poopEffect;
          const rotation = controller && "particle58" in controller && typeof controller.particle58 === "object"
            ? controller.particle58.flying.layerRotation : 0;
          ctx.save();
          ctx.translate((effect.presentedPosition.x + effect.renderOffset.x + sprite.offset.x) * 0.65 + 32,
            (effect.presentedPosition.y + effect.renderOffset.y + sprite.offset.y) * 0.65 - 46);
          ctx.scale(sprite.flipX ? -1 : 1, sprite.flipY ? -1 : 1);
          this.assets.draw(ctx, effect.variant === 58 ? "gfx/1000.058_poop particle.anm2" : "gfx/1000.043_poop explosion.anm2",
            sprite.animation, 0, 0, sprite.frame, { layerScales: {0: [sprite.scale.x, sprite.scale.y]}, overrides: {0: {
              Rotation: rotation, RedTint: sprite.tint[0] * 255, GreenTint: sprite.tint[1] * 255,
              BlueTint: sprite.tint[2] * 255, AlphaTint: sprite.tint[3] * 255,
              RedOffset: sprite.offsets[0] * 255, GreenOffset: sprite.offsets[1] * 255, BlueOffset: sprite.offsets[2] * 255,
            }} });
          ctx.restore();
          continue;
        }
        this.drawEffect(ctx, effect.variant, effect.sprite,
          (effect.presentedPosition.x + effect.sprite.offset.x) * 0.65 + 32,
          (effect.presentedPosition.y + effect.sprite.offset.y) * 0.65 - 46);
      }
      ctx.restore();
      this.stage.foreground(state.stageEffects);
      this.presentation.introduction(ctx, state);
      this.hud(ctx, state);
      this.presentation.bossBar(ctx, state);
      const info = state.informationAnimation;
      if (info.state === 2 && info.definitionLoaded && info.currentActor && info.currentClip)
        this.assets.draw(ctx, info.currentActor, info.currentClip, 240, 135, info.frame,
          { spritesheets: info.spritesheet ? { 0: info.spritesheet } : {} });
      const bossScene = state.roomTransition?.bossScene;
      if (bossScene && state.roomTransition?.phase === "bossScene")
        this.assets.draw(ctx, "gfx/ui/boss/versusscreen.anm2", "Scene", 240, 135, bossScene.animationFrame);
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
    }
    gl.presentScreen(); gl.finish();
    this.context.reset(); this.context.imageSmoothingEnabled = false;
    this.context.drawImage(gl.canvas, 0, 0);
  }
  debugTargets(): StageTargets { return this.stage.debugTargets(); }
  drawRecord() { return { npcs: structuredClone(this.drawnNpcs), grid: structuredClone(this.drawnGrid) }; }
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
  private drawEffect(ctx: CanvasRenderingContext2D, variant: number, sprite: ActorSprite, x: number, y: number): void {
    const actors: Record<number, string> = { 7: "gfx/1000.007_blood splat.anm2", 15: "gfx/1000.015_poof01.anm2",
      17: "gfx/1000.017_poof04.anm2", 42: "gfx/1000.042_blood gush.anm2" };
    const actor = actors[variant];
    if (!actor) throw new Error(`Unadmitted effect renderer ${variant}`);
    ctx.save(); ctx.translate(x, y); ctx.rotate(sprite.rotation * Math.PI / 180);
    ctx.scale(sprite.scale.x * (sprite.flipX ? -1 : 1), sprite.scale.y * (sprite.flipY ? -1 : 1));
    const layerScales = Object.fromEntries(Object.entries(sprite.layerFlips).map(([layer, flips]) =>
      [layer, [flips[0] ? -1 : 1, flips[1] ? -1 : 1] as const]));
    this.assets.draw(ctx, actor, sprite.animation, 0, 0, sprite.frame,
      { layerScales, overrides: { 0: {
        RedTint: sprite.tint[0] * 255, GreenTint: sprite.tint[1] * 255,
        BlueTint: sprite.tint[2] * 255, AlphaTint: sprite.tint[3] * 255,
        RedOffset: sprite.offsets[0] * 255, GreenOffset: sprite.offsets[1] * 255,
        BlueOffset: sprite.offsets[2] * 255 } } });
    ctx.restore();
  }
  private prepareRoom(room: Room): void {
    const appearance = room.appearance;
    const key = JSON.stringify({ backdrop: appearance.backdrop, patchSize: appearance.backdropPatchSize,
      patchColumns: appearance.backdropPatchColumns, floor: appearance.floorQuads, walls: appearance.wallQuads,
      columns: appearance.columns, rows: appearance.rows, tileSize: appearance.tileSize, tutorial: appearance.tutorial,
      backgroundStains: room.backgroundStains });
    if (key === this.roomKey) return;
    const gl = this.graphics, width = appearance.columns * appearance.tileSize, height = appearance.rows * appearance.tileSize;
    gl.beginTarget(this.floor, width, height); gl.resetTransform(); gl.globalAlpha = 1; gl.imageSmoothingEnabled = false;
    gl.clearRect(0, 0, this.floor.width, this.floor.height); gl.blendFactors("one", "zero");
    this.drawQuads(appearance, appearance.floorQuads);
    if (appearance.tutorial) {
      const controls = this.assets.image("gfx/backdrop/controls.png");
      gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
      gl.drawImage(controls, Math.round(width / 2 - controls.width / 2), Math.round(height / 2 - controls.height / 2));
    }
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
    const ctx = gl.asCanvasContext();
    for (const stain of room.backgroundStains) {
      const sprite = stain.sprite;
      if (stain.variant === 58 || stain.variant === 43) {
        ctx.save();
        ctx.translate((stain.position.x + stain.renderOffset.x + sprite.offset.x) * 0.65 + 32 - appearance.origin.x,
          (stain.position.y + stain.renderOffset.y + sprite.offset.y) * 0.65 - 46 - appearance.origin.y);
        ctx.scale(sprite.flipX ? -1 : 1, sprite.flipY ? -1 : 1);
        const actor = stain.variant === 58 ? "gfx/1000.058_poop particle.anm2" :
          stain.variant === 43 ? "gfx/1000.043_poop explosion.anm2" : "gfx/1000.007_blood splat.anm2";
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
    gl.beginTarget(this.walls, width, height); gl.resetTransform();
    gl.clearRect(0, 0, this.walls.width, this.walls.height); gl.blendFactors("one", "zero");
    this.drawQuads(appearance, appearance.wallQuads);
    this.roomKey = key;
  }
  private drawRoom(appearance: RoomAppearance): void {
    const gl = this.graphics, ctx = gl.asCanvasContext(), size = appearance.tileSize, border = size * 2;
    const width = appearance.columns * size, height = appearance.rows * size, { x, y } = appearance.origin;
    gl.drawSurface(this.floor, border, border, width - border * 2, height - border - size,
      x + border, y + border, width - border * 2, height - border - size);
    gl.drawSurface(this.walls, 0, 0, border, height, x, y, border, height);
    gl.drawSurface(this.walls, width - border, 0, border, height, x + width - border, y, border, height);
    gl.drawSurface(this.walls, border, 0, width - border * 2, border, x + border, y, width - border * 2, border);
    gl.drawSurface(this.walls, border, height - border, width - border * 2, border,
      x + border, y + height - border, width - border * 2, border);
    const rotation = { up: 0, left: -Math.PI / 2, right: Math.PI / 2, down: Math.PI };
    for (const connection of appearance.doors) {
      ctx.save(); ctx.translate(connection.x, connection.y); ctx.rotate(rotation[connection.side]);
      this.assets.draw(ctx, connection.asset, connection.state, 0, 0, 0,
        connection.spritesheet ? { spritesheets: { 0: connection.spritesheet } } : {}); ctx.restore();
    }
    gl.drawImage(this.assets.image(appearance.fx.shading), x, y, width, height);
  }
  private drawPlayer(ctx: CanvasRenderingContext2D, state: State): void {
    if (!state.player.visible) return;
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
    this.assets.draw(ctx, PLAYER, `Head${directionName(player.facing)}`, x, y, player.headFrame,
      { layers: [4], overrides: { 4: { RedTint, GreenTint, BlueTint } } });
  }
  private hud(ctx: CanvasRenderingContext2D, state: State): void {
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
    for (let pickup = 0; pickup < 3; pickup++) {
      this.assets.draw(ctx, "gfx/ui/hudpickups.anm2", "Idle", 20, 44 + pickup * 12, [0, 2, 1][pickup]!);
      this.assets.font(ctx, "pftempestasevencondensed", String(counts[pickup]).padStart(2, "0"), 36, 45 + pickup * 12);
    }
    if (!state.achievementEligibility) this.assets.draw(ctx, "gfx/ui/hudpickups.anm2", "Idle", 24, 84, 5);
    this.assets.draw(ctx, "gfx/ui/minimap1.anm2", "Frame", 400, 16);
    for (const cell of state.minimap.cells) {
      const animation = { current: "RoomCurrent", visited: "RoomVisited", unvisited: "RoomUnvisited" }[cell.visibility];
      this.assets.draw(ctx, "gfx/ui/minimap1.anm2", animation, 420 + 8 * cell.x, 33 + 8 * cell.y);
    }
  }
}
