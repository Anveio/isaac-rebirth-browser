import { Assets } from "./animation";
import { MenuRenderer } from "./menu";
import { GLMenuContext, type GLSurface } from "./gl-menu";
import { StageComposer, type StageTargets } from "./gl-stage";
import { RunPresentation } from "./presentation";
import { PickupRenderer } from "./pickups";
import type { RoomAppearance, RoomQuad, State } from "./engine";

const PLAYER = "gfx/001.000_player.anm2";
const TEAR = "gfx/002.000_tear.anm2";
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
    const gl = this.graphics;
    if (state.screen === "playing") {
      this.prepareRoom(state.room.appearance);
      this.stage.prepare(state.stageEffects);
    }
    gl.beginScreen(); gl.resetTransform(); gl.globalAlpha = 1; gl.imageSmoothingEnabled = false;
    gl.clearRect(0, 0, this.canvas.width, this.canvas.height);
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
    gl.scale(this.canvas.width / 480, this.canvas.height / 270);
    const ctx = gl.asCanvasContext();
    if (state.screen !== "playing") this.menu.draw(ctx, state);
    else {
      this.drawRoom(state.room.appearance);
      this.stage.ground(state.stageEffects);
      const grid = state.room.grid;
      if (grid) for (const [index, cell] of grid.cells.entries()) {
        const entity = cell.entity;
        if (!entity) continue;
        const x = (40 + index % grid.width * 40) * 0.65 + 32;
        const y = (120 + Math.floor(index / grid.width) * 40) * 0.65 - 46;
        if (entity.kind === 2 && entity.state === 1)
          this.assets.draw(ctx, "gfx/grid/grid_rock.anm2", entity.variant >= 1000 ? "big" : "normal",
            x, y, entity.variant >= 1000 ? entity.variant - 1000 : entity.variant);
        else if (entity.kind === 8 && entity.state === 0)
          this.assets.draw(ctx, "gfx/grid/grid_spikes.anm2", `Spikes${String(entity.variant).padStart(2, "0")}`, x, y);
      }
      this.pickups.draw(ctx, state);
      for (const npc of state.room.npcs)
        if (!npc.dead && !npc.removed)
          this.assets.draw(ctx, npc.kind === 13 ? "gfx/013.000_fly.anm2" : "gfx/018.000_attack fly.anm2", "Fly",
            npc.presentedPosition.x * 0.65 + 32, npc.presentedPosition.y * 0.65 - 46, npc.animationFrame, { scale: 0.65 });
      this.drawPlayer(ctx, state);
      for (const tear of state.tears)
        this.assets.draw(ctx, TEAR, "RegularTear6", Math.round(tear.x * 2) / 2, Math.round(tear.y * 2) / 2, tear.age);
      for (const impact of state.impacts)
        this.assets.draw(ctx, "gfx/1000.012_tear poof_small.anm2", "Poof", impact.x, impact.y, impact.age);
      for (const bomb of state.bombs)
        if (bomb.motion.lifetime === "alive")
          this.assets.draw(ctx, "gfx/004.000_bomb.anm2", "Pulse", bomb.x, bomb.y - 1, bomb.motion.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
      for (const explosion of state.explosions) {
        this.assets.draw(ctx, "gfx/1000.001_bomb explosion.anm2", "Explosion", explosion.x, explosion.y - 1, explosion.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
      }
      for (const bomb of state.bombs)
        if (bomb.motion.lifetime === "exploding")
          this.assets.draw(ctx, "gfx/004.000_bomb.anm2", "Explode", bomb.x, bomb.y - 1, bomb.motion.animationFrame, { pixelSnap: this.canvas.width / 480 / 2 });
      this.stage.foreground(state.stageEffects);
      this.presentation.introduction(ctx, state);
      this.hud(ctx, state);
      this.presentation.pause(ctx, state);
    }
    gl.presentScreen(); gl.finish();
    this.context.reset(); this.context.imageSmoothingEnabled = false;
    this.context.drawImage(gl.canvas, 0, 0);
  }
  debugTargets(): StageTargets { return this.stage.debugTargets(); }
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
  private prepareRoom(appearance: RoomAppearance): void {
    const key = JSON.stringify({ backdrop: appearance.backdrop, patchSize: appearance.backdropPatchSize,
      patchColumns: appearance.backdropPatchColumns, floor: appearance.floorQuads, walls: appearance.wallQuads,
      columns: appearance.columns, rows: appearance.rows, tileSize: appearance.tileSize, tutorial: appearance.tutorial });
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
      this.assets.draw(ctx, connection.asset, connection.state, 0, 0); ctx.restore();
    }
    gl.drawImage(this.assets.image(appearance.fx.shading), x, y, width, height);
  }
  private drawPlayer(ctx: CanvasRenderingContext2D, state: State): void {
    const player = state.player, x = Math.round((player.x + player.renderOffset.x) * 2) / 2,
      y = Math.round((player.y + player.renderOffset.y) * 2) / 2;
    const [RedTint, GreenTint, BlueTint] = state.stageEffects.actorTint;
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
