// Passive actual WebGL observations; no extra engine step, key or state setter.
import {mapObservedBrowser} from './options-map-browser.mjs';

function installGammaGraphics() {
  const records = [], limit = 4096, ids = new WeakMap(), shaders = new WeakSet();
  const attached = new WeakMap(), gammaPrograms = new WeakSet(), gammaLocations = new WeakMap();
  const attachments = new Map(), dimensions = new Map();
  let next = 0, truncated = false, error = null;
  const id = value => {
    if (value === null) return 0;
    if (!ids.has(value)) ids.set(value, ++next);
    return ids.get(value);
  };
  const wrap = (name, after) => {
    const original = WebGLRenderingContext.prototype[name];
    WebGLRenderingContext.prototype[name] = function(...args) {
      const result = original.apply(this, args);
      try { after.call(this, args, result); }
      catch (failure) { error ??= String(failure.stack ?? failure); }
      return result;
    };
  };
  wrap('shaderSource', ([shader, source]) => {
    if (source.includes('uniform float gamma;') && source.includes('pow(clamp(')) shaders.add(shader);
  });
  wrap('attachShader', ([program, shader]) => {
    if (!attached.has(program)) attached.set(program, []);
    attached.get(program).push(shader);
  });
  wrap('linkProgram', ([program]) => {
    if ((attached.get(program) ?? []).some(shader => shaders.has(shader))) gammaPrograms.add(program);
  });
  wrap('getUniformLocation', ([program, name], location) => {
    if (location && gammaPrograms.has(program) && name === 'gamma') gammaLocations.set(program, location);
  });
  wrap('framebufferTexture2D', function([target, attachment, textarget, texture]) {
    if (target === this.FRAMEBUFFER && attachment === this.COLOR_ATTACHMENT0)
      attachments.set(id(this.getParameter(this.FRAMEBUFFER_BINDING)), id(texture));
  });
  wrap('texImage2D', function(args) {
    const texture = this.getParameter(this.TEXTURE_BINDING_2D);
    if (args.length === 9) dimensions.set(id(texture), [args[3], args[4]]);
    else if (args.length === 6) dimensions.set(id(texture), [args[5].width, args[5].height]);
  });
  wrap('drawArrays', function([mode, first, count]) {
    // The existing final composition presents to the actual default framebuffer.
    if (this.getParameter(this.FRAMEBUFFER_BINDING) !== null) return;
    if (records.length === limit) { truncated = true; return; }
    const program = this.getParameter(this.CURRENT_PROGRAM), texture = this.getParameter(this.TEXTURE_BINDING_2D);
    const gamma = gammaPrograms.has(program), location = gammaLocations.get(program);
    records.push({sequence: records.length, program: id(program), gammaProgram: gamma,
      ...(gamma && location ? {gamma: this.getUniform(program, location)} : {}),
      framebuffer: 0, texture: id(texture), sourceTarget: [...attachments.values()].includes(id(texture)),
      dimensions: dimensions.get(id(texture)), viewport: Array.from(this.getParameter(this.VIEWPORT)),
      mode, first, count, linked: this.getProgramParameter(program, this.LINK_STATUS)});
  });
  window.__gammaGraphics = () => ({records: structuredClone(records), truncated, error});
}

export function gammaObservedBrowser(real, data, contract, observed, retain) {
  const graphics = new Proxy(real, {get(target, key) {
    if (key !== 'newContext') {
      const value = Reflect.get(target, key);
      return typeof value === 'function' ? value.bind(target) : value;
    }
    return async (...args) => {
      const context = await target.newContext(...args);
      await context.addInitScript({content: `(${installGammaGraphics.toString()})();`});
      return context;
    };
  }});
  return mapObservedBrowser(graphics, data, contract, observed, retain);
}
