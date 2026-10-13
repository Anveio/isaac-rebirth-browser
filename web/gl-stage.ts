import type { Assets } from "./animation";
import type { Blend, StageEffects } from "./engine";
import { GLMenuContext, type BlendName, type GLSurface } from "./gl-menu";

const blendNames: Record<Blend["source"], BlendName> = {
  one: "one", zero: "zero", "source-alpha": "srcAlpha",
  "one-minus-source-alpha": "oneMinusSrcAlpha", "destination-color": "dstColor",
};
export interface RawTarget { width: number; height: number; rgbaBase64: string; }
export interface StageTargets {
  format: "RGBA8-straight"; origin: "top-left-logical";
  shadow: RawTarget; lighting: RawTarget;
}

/** Stage masks use the original's observable graphics-API blend contracts. */
export class StageComposer {
  private readonly shadow: GLSurface;
  private readonly lighting: GLSurface;
  constructor(private readonly graphics: GLMenuContext, private readonly assets: Assets) {
    this.shadow = graphics.createTarget(512, 512);
    this.lighting = graphics.createTarget(512, 512);
  }
  private factors(blend: Blend): void {
    this.graphics.blendFactors(blendNames[blend.source], blendNames[blend.destination]);
  }
  prepare(effects: StageEffects, scroll = { x: 0, y: 0 }): void {
    const gl = this.graphics, { width, height } = effects.viewport;
    gl.beginTarget(this.shadow, width, height); gl.resetTransform();
    gl.globalAlpha = 1; gl.imageSmoothingEnabled = false;
    gl.clearRect(0, 0, this.shadow.width, this.shadow.height);
    this.factors(effects.groundShadowMaskBlend);
    gl.translate(scroll.x, scroll.y);
    for (const caster of effects.shadowCasters) {
      const rect = caster.destinationRect;
      gl.drawImage(this.assets.image(caster.asset), rect.x, rect.y, rect.width, rect.height);
    }
    gl.beginTarget(this.lighting, width, height); gl.resetTransform();
    gl.clearRect(0, 0, this.lighting.width, this.lighting.height);
    gl.blendFactors("one", "zero");
    gl.fillStyle = `#${effects.ambientColor.slice(0, 3).map(channel => Math.round(channel * 255).toString(16).padStart(2, "0")).join("")}`;
    gl.globalAlpha = effects.ambientColor[3]; gl.fillRect(0, 0, width, height); gl.globalAlpha = 1;
    this.factors(effects.lightingMaskBlend);
    gl.translate(scroll.x, scroll.y);
    const crop = effects.lightGradient.crop;
    for (const light of effects.lights) {
      const rect = light.destinationRect;
      gl.globalAlpha = light.alpha;
      gl.drawImage(this.assets.image(effects.lightGradient.asset), crop.x, crop.y, crop.width, crop.height,
        rect.x, rect.y, rect.width, rect.height);
    }
    gl.globalAlpha = 1;
    gl.beginTarget(null); gl.resetTransform(); gl.globalAlpha = 1;
  }
  ground(effects: StageEffects): void {
    this.factors(effects.groundShadowCompositeBlend);
    const { width, height } = effects.viewport;
    this.graphics.drawSurface(this.shadow, 0, 0, width, height, 0, 0, width, height, effects.groundShadowCompositeRGBA);
  }
  foreground(effects: StageEffects): void {
    const gl = this.graphics, { width, height } = effects.viewport;
    this.factors(effects.lightingCompositeBlend);
    gl.drawSurface(this.lighting, 0, 0, width, height, 0, 0, width, height, effects.lightingCompositeRGBA);
    const overlay = effects.overlay, crop = overlay.sourceCrop;
    this.factors(overlay.blend);
    gl.drawTintedImage(this.assets.image(overlay.asset), overlay.modulationRGBA, [0, 0, 0],
      crop.x, crop.y, crop.width, crop.height, 0, 0, width, height);
    gl.blendFactors("srcAlpha", "oneMinusSrcAlpha");
  }
  debugTargets(): StageTargets {
    const encode = (surface: GLSurface): RawTarget => {
      const { width, height, pixels } = this.graphics.readTarget(surface);
      let bytes = "";
      for (let offset = 0; offset < pixels.length; offset += 8192)
        bytes += String.fromCharCode(...pixels.subarray(offset, offset + 8192));
      return { width, height, rgbaBase64: btoa(bytes) };
    };
    return { format: "RGBA8-straight", origin: "top-left-logical", shadow: encode(this.shadow), lighting: encode(this.lighting) };
  }
}
