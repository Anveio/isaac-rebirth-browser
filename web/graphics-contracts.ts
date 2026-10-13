import type { Assets } from "./animation";
import { GLMenuContext } from "./gl-menu";
import { loadRawPNG } from "./png";

/** Runs the public graphics harness through the bundled production backend. */
export async function graphicsContracts(assets: Assets): Promise<{ mask: number[]; screen: number[] }> {
  const graphics = await GLMenuContext.load(assets, 8, 8);
  const mask = graphics.createTarget(8, 8);
  graphics.beginTarget(mask, 4, 4); graphics.resetTransform(); graphics.clearRect(0, 0, 8, 8);
  graphics.blendFactors("srcAlpha", "oneMinusSrcAlpha");
  graphics.fillStyle = "#ffffff"; graphics.globalAlpha = 0.5; graphics.fillRect(0, 0, 4, 4);
  graphics.blendFactors("one", "zero");
  graphics.fillStyle = "#ff0000"; graphics.globalAlpha = 0; graphics.fillRect(0, 0, 4, 4);
  const maskPixels = [...graphics.readTarget(mask).pixels];

  const target = graphics.createTarget(4, 4);
  graphics.globalAlpha = 1;
  graphics.beginScreen(); graphics.resetTransform(); graphics.blendFactors("one", "zero");
  graphics.fillStyle = "#00ff00"; graphics.fillRect(0, 0, 8, 8); graphics.presentScreen();
  graphics.beginScreen(); graphics.resetTransform(); graphics.clearRect(0, 0, 8, 8);
  graphics.blendFactors("one", "zero");
  graphics.fillStyle = "#0000ff"; graphics.fillRect(0, 0, 7, 4);
  graphics.beginTarget(target); graphics.resetTransform(); graphics.clearRect(0, 0, 4, 4);
  graphics.fillStyle = "#ff0000"; graphics.fillRect(0, 0, 4, 4);
  graphics.beginTarget(null); graphics.resetTransform();
  graphics.drawSurface(target, 0, 0, 4, 4, 0, 4, 7, 4);
  graphics.presentScreen(); graphics.finish();
  const screen = await loadRawPNG(graphics.canvas.toDataURL("image/png"));
  return { mask: maskPixels, screen: [...screen.pixels] };
}
