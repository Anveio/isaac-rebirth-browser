import { type Assets, type Frame, type DrawOptions } from "./animation";
import { sampleFrame } from "./animation-math";
import bossData from "./boss-splash-data.json";
import npcData from "./npc-data.json";
import type { State } from "./engine";

const path = "gfx/ui/boss/versusscreen.anm2";
const actor = npcData[path];
const playerPortraits = [
  "playerportrait_01_isaac.png", "playerportrait_02_magdalene.png", "playerportrait_03_cain.png",
  "playerportrait_04_judas.png", "playerportrait_06_bluebaby.png", "playerportrait_05_eve.png",
  "playerportrait_07_samson.png", "playerportrait_08_azazel.png", "playerportrait_09_lazarus.png",
  "playerportrait_09_eden.png", "playerportrait_12_thelost.png", "playerportrait_10_lazarus2.png",
  "playerportrait_blackjudas.png",
];
const f = Math.fround;

/** 5dd1b0 selects actual presentation/config records before Scene playback. */
export function drawBossSplash(assets: Assets, ctx: CanvasRenderingContext2D, state: State): void {
  const scene = state.roomTransition?.bossScene;
  if (!scene) return;
  const presentation = bossData.bosses.find(record => record.id === scene.presentationIndex);
  const playerType = state.weaponAdmission.sourcePlayerType;
  const playerPortrait = playerType !== null && Number.isInteger(playerType) ? playerPortraits[playerType] : undefined;
  const sheets: Record<number, string> = {};
  if (presentation) sheets[4] = presentation.portrait;
  if (playerPortrait) sheets[5] = `gfx/ui/boss/${playerPortrait}`;
  // Current admitted levels are normal Basement I/II. The actual 5d49b0
  // stage selector chooses the same authored record1 on both floors.
  if (state.stage === 1 || state.stage === 2) {
    sheets[2] = "gfx/ui/boss/bossspot_01_basement.png";
    sheets[3] = "gfx/ui/boss/playerspot_01_basement.png";
  }
  const animation = actor.Animations.find(animation => animation.Name === "Scene")!;
  const portraitLayer = animation.LayerAnimations.find(layer => layer.LayerId === 4)!;
  const portrait = sampleFrame(portraitLayer.Frames as Frame[], scene.animationFrame, animation.Loop);
  const overrides: NonNullable<DrawOptions["overrides"]> = {};
  if (presentation && portrait) {
    // 5df160 stores (96-pivotX,132-pivotY) as Layer4's offset;
    // 40eeb0 adds it to the selected frame position before Sprite motion.
    overrides[4] = { XPosition: f(portrait.XPosition + f(96 - f(presentation.pivotX))),
      YPosition: f(portrait.YPosition + f(132 - f(presentation.pivotY))) };
  }
  assets.draw(ctx, path, "Scene", 240, 135, scene.animationFrame,
    { spritesheets: sheets, overrides,
      ...(scene.presentationIndex === 55 ? { layers: [0, 1, 3, 4, 5, 6, 7, 8, 9, 10] } : {}) });
  // 5de870 draws names after the sprite. Scene layers6/7 are hidden guide
  // tracks: 40fa30 samples their geometry regardless of Layer.Visible.
  for (const [layerId, name, index] of [[7, "BossNames", scene.presentationIndex], [6, "PlayerNames", playerType]] as const) {
    if (index === null || !Number.isInteger(index) || index < 0 || (layerId === 6 && !playerPortrait) || (layerId === 7 && !presentation)) continue;
    const track = animation.LayerAnimations.find(layer => layer.LayerId === layerId)!;
    const guide = sampleFrame(track.Frames as Frame[], scene.animationFrame, animation.Loop);
    if (!guide) continue;
    // Native uses the selected name crop but the guide's position/pivot/scale,
    // with white ColorMod; guide visibility, alpha and rotation are not read.
    assets.draw(ctx, path, name, 240, 135, index, { layers: [layerId],
      layerScales: { [layerId]: [guide.XScale, guide.YScale] },
      overrides: { [layerId]: { XPosition: guide.XPosition, YPosition: guide.YPosition,
        XPivot: guide.XPivot ?? 0, YPivot: guide.YPivot ?? 0, XScale: 100, YScale: 100,
        Rotation: 0, Visible: true, RedTint: 255, GreenTint: 255, BlueTint: 255, AlphaTint: 255,
        RedOffset: 0, GreenOffset: 0, BlueOffset: 0 } } });
  }
}
