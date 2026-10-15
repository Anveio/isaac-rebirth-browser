import sceneData from "./scene-data.json";
import fontData from "./font-data.json";
import type { Assets } from "./animation";
import { loadRawPNG, type RawImage } from "./png";

interface DrawState { transform: number[]; alpha: number; smoothing: boolean; fillStyle: string; }
interface Texture { handle: WebGLTexture; width: number; height: number; textureWidth: number; textureHeight: number; }
export interface GLSurface { texture: WebGLTexture; framebuffer: WebGLFramebuffer; depth: WebGLRenderbuffer; width: number; height: number; }
export type BlendName = "zero" | "one" | "srcAlpha" | "oneMinusSrcAlpha" | "dstAlpha" | "oneMinusDstAlpha" | "srcColor" | "oneMinusSrcColor" | "dstColor" | "oneMinusDstColor";
export type AlphaComparison = "never" | "less" | "equal" | "lessEqual" | "greater" | "notEqual" | "greaterEqual" | "always";
const normalize = (path: string) => path.replaceAll("\\", "/").replace(/\/\.\//g, "/").replace(/\/\.$/, "").replace(/^resources\//i, "").toLowerCase();

/** A textured-quad context using straight RGBA uploads and standard GL blending. */
export class GLMenuContext {
  readonly canvas: HTMLCanvasElement;
  private readonly gl: WebGLRenderingContext;
  private readonly textures = new Map<CanvasImageSource, Texture>();
  private readonly states: DrawState[] = [];
  private state: DrawState = { transform: [1, 0, 0, 1, 0, 0], alpha: 1, smoothing: false, fillStyle: "#000000" };
  private readonly program: WebGLProgram;
  private readonly vertices: WebGLBuffer;
  private readonly color: WebGLUniformLocation;
  private readonly offsets: WebGLUniformLocation;
  private readonly alphaFunction: WebGLUniformLocation;
  private readonly alphaReference: WebGLUniformLocation;
  private readonly projection: WebGLUniformLocation;
  private readonly white: Texture;
  private readonly screen: GLSurface;
  private screenActive = false;
  private currentTarget: GLSurface | null = null;
  private readonly drawLayers = new Map<GLSurface | null, number>();

  private constructor(width: number, height: number) {
    this.canvas = document.createElement("canvas"); this.canvas.width = width; this.canvas.height = height;
    const gl = this.canvas.getContext("webgl", { alpha: false, antialias: false, depth: true, stencil: false, preserveDrawingBuffer: true, premultipliedAlpha: false });
    if (!gl) throw new Error("A WebGL graphics context is required"); this.gl = gl;
    const vertex = this.shader(gl.VERTEX_SHADER, `attribute vec3 position; attribute vec2 textureUV; varying vec2 uv; uniform mat4 projection;
      void main() { uv = textureUV; gl_Position = projection * vec4(position, 1.0); }`);
    const fragment = this.shader(gl.FRAGMENT_SHADER, `precision highp float; varying vec2 uv; uniform highp sampler2D image; uniform vec4 tint; uniform vec3 offsets;
      uniform int alphaFunction; uniform float alphaReference;
      void main() { vec4 sample = texture2D(image, uv); vec4 color = vec4(sample.rgb * tint.rgb + offsets, sample.a * tint.a);
        bool accepted = alphaFunction == 0 || alphaFunction == 8 ||
          (alphaFunction == 2 && color.a < alphaReference) || (alphaFunction == 3 && color.a == alphaReference) ||
          (alphaFunction == 4 && color.a <= alphaReference) || (alphaFunction == 5 && color.a > alphaReference) ||
          (alphaFunction == 6 && color.a != alphaReference) || (alphaFunction == 7 && color.a >= alphaReference);
        if (!accepted) discard; gl_FragColor = color; }`);
    this.program = gl.createProgram()!; gl.attachShader(this.program, vertex); gl.attachShader(this.program, fragment); gl.linkProgram(this.program);
    if (!gl.getProgramParameter(this.program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(this.program) ?? "Graphics program link failed");
    gl.useProgram(this.program);
    this.projection = gl.getUniformLocation(this.program, "projection")!;
    this.setProjection(width, height, -1);
    gl.uniform1i(gl.getUniformLocation(this.program, "image"), 0); this.color = gl.getUniformLocation(this.program, "tint")!;
    this.offsets = gl.getUniformLocation(this.program, "offsets")!;
    this.alphaFunction = gl.getUniformLocation(this.program, "alphaFunction")!; this.alphaReference = gl.getUniformLocation(this.program, "alphaReference")!;
    this.alphaTest("greater", 0);
    this.vertices = gl.createBuffer()!; gl.bindBuffer(gl.ARRAY_BUFFER, this.vertices); gl.bufferData(gl.ARRAY_BUFFER, 30 * 4, gl.DYNAMIC_DRAW);
    const position = gl.getAttribLocation(this.program, "position"), uv = gl.getAttribLocation(this.program, "textureUV");
    gl.enableVertexAttribArray(position); gl.vertexAttribPointer(position, 3, gl.FLOAT, false, 20, 0);
    gl.enableVertexAttribArray(uv); gl.vertexAttribPointer(uv, 2, gl.FLOAT, false, 20, 12);
    gl.enable(gl.BLEND); gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA); gl.disable(gl.CULL_FACE);
    // Native targets retain a real depth attachment with GREATER and clear 0.
    // This also controls Mesa's interpolation path at nearest-sampled texel ties.
    gl.enable(gl.DEPTH_TEST); gl.depthFunc(gl.GREATER); gl.depthMask(true); gl.clearDepth(0); gl.depthRange(0, 1);
    gl.disable(gl.STENCIL_TEST);
    this.white = this.upload({ width: 1, height: 1, pixels: new Uint8Array([255, 255, 255, 255]) });
    this.screen = this.createTarget(width, height);
    gl.viewport(0, 0, width, height); gl.clearColor(0, 0, 0, 1); gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  }

  static async load(assets: Assets, width = 960, height = 540, extraPaths: string[] = []): Promise<GLMenuContext> {
    const context = new GLMenuContext(width, height), paths = new Set<string>([
      "gfx/ui/main menu/menuoverlay.png", "gfx/ui/main menu/menushadow.png", "gfx/ui/backselectwidget.png",
    ]);
    const menuActors = new Set(["titlemenu", "saveselectmenu", "saveselectmenu_drawing", "gamemenu", "charactermenu", "characterportraits", "completion_widget", "difficultywidget", "seedwidget", "seedentry", "winstreakwidget", "menushadow"]);
    for (const [path, actor] of Object.entries(sceneData)) if (menuActors.has(path.replace("gfx/ui/main menu/", "").replace(".anm2", ""))) {
      for (const sheet of actor.Spritesheets) paths.add(normalize(`${actor.BaseSpritesheetPath}/${sheet.Path}`));
    }
    for (const font of Object.values(fontData)) for (const page of font.pages) paths.add(`font/${page}`);
    for (const path of extraPaths) paths.add(normalize(path));
    await Promise.all([...paths].map(async path => {
      const image = assets.image(path); const raw = await loadRawPNG(image.src); context.textures.set(image, context.upload(raw));
    }));
    return context;
  }

  private shader(kind: number, source: string): WebGLShader {
    const shader = this.gl.createShader(kind)!; this.gl.shaderSource(shader, source); this.gl.compileShader(shader);
    if (!this.gl.getShaderParameter(shader, this.gl.COMPILE_STATUS)) throw new Error(this.gl.getShaderInfoLog(shader) ?? "Graphics shader compilation failed");
    return shader;
  }
  private setProjection(width: number, height: number, orientation: number): void {
    this.gl.uniformMatrix4fv(this.projection, false, new Float32Array([
      2 / width, 0, 0, 0, 0, orientation * 2 / height, 0, 0, 0, 0, -0.001, 0, -1, -orientation, 0, 1,
    ]));
  }
  private upload(raw: RawImage): Texture {
    const gl = this.gl, handle = gl.createTexture()!; gl.bindTexture(gl.TEXTURE_2D, handle);
    const textureWidth = 2 ** Math.ceil(Math.log2(raw.width)), textureHeight = 2 ** Math.ceil(Math.log2(raw.height));
    const padded = new Uint8Array(textureWidth * textureHeight * 4);
    for (let y = 0; y < raw.height; y++) padded.set(raw.pixels.subarray(y * raw.width * 4, (y + 1) * raw.width * 4), y * textureWidth * 4);
    gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, 0); gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 0);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, textureWidth, textureHeight, 0, gl.RGBA, gl.UNSIGNED_BYTE, padded);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    return { handle, width: raw.width, height: raw.height, textureWidth, textureHeight };
  }
  private texture(source: CanvasImageSource): Texture {
    const existing = this.textures.get(source); if (existing) return existing;
    if (source instanceof HTMLCanvasElement) {
      const pixels = source.getContext("2d")!.getImageData(0, 0, source.width, source.height).data;
      const texture = this.upload({ width: source.width, height: source.height, pixels: new Uint8Array(pixels) }); this.textures.set(source, texture); return texture;
    }
    throw new Error("Graphics source was not loaded before drawing");
  }

  createTarget(width: number, height: number): GLSurface {
    const gl = this.gl, previous = gl.getParameter(gl.FRAMEBUFFER_BINDING) as WebGLFramebuffer | null;
    const previousDepth = gl.getParameter(gl.RENDERBUFFER_BINDING) as WebGLRenderbuffer | null;
    const texture = gl.createTexture()!, framebuffer = gl.createFramebuffer()!;
    const depth = gl.createRenderbuffer();
    if (!depth) { gl.deleteFramebuffer(framebuffer); gl.deleteTexture(texture); throw new Error("Graphics target depth buffer could not be created"); }
    gl.bindTexture(gl.TEXTURE_2D, texture); gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST); gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer); gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
    // WebGL requires matching attachment dimensions. Packed depth-stencil
    // supplies the native 24 depth bits; stencil testing stays disabled.
    gl.bindRenderbuffer(gl.RENDERBUFFER, depth); gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_STENCIL, width, height);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.RENDERBUFFER, depth);
    const status = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    gl.bindRenderbuffer(gl.RENDERBUFFER, previousDepth); gl.bindFramebuffer(gl.FRAMEBUFFER, previous);
    if (status !== gl.FRAMEBUFFER_COMPLETE) {
      gl.deleteRenderbuffer(depth); gl.deleteFramebuffer(framebuffer); gl.deleteTexture(texture);
      throw new Error(`Graphics target framebuffer is incomplete: ${status}`);
    }
    return { texture, framebuffer, depth, width, height };
  }
  readTarget(surface: GLSurface): RawImage {
    const gl = this.gl, previous = gl.getParameter(gl.FRAMEBUFFER_BINDING) as WebGLFramebuffer | null;
    const pixels = new Uint8Array(surface.width * surface.height * 4);
    gl.bindFramebuffer(gl.FRAMEBUFFER, surface.framebuffer); gl.readPixels(0, 0, surface.width, surface.height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
    gl.bindFramebuffer(gl.FRAMEBUFFER, previous); return { width: surface.width, height: surface.height, pixels };
  }
  beginTarget(surface: GLSurface | null, viewportWidth?: number, viewportHeight?: number): void {
    surface ??= this.screenActive ? this.screen : null;
    this.currentTarget = surface;
    const gl = this.gl, width = viewportWidth ?? surface?.width ?? this.canvas.width, height = viewportHeight ?? surface?.height ?? this.canvas.height;
    gl.bindFramebuffer(gl.FRAMEBUFFER, surface?.framebuffer ?? null); gl.viewport(0, 0, width, height);
    // Target row zero stores logical row zero, matching uploaded source texture rows.
    this.setProjection(width, height, surface ? 1 : -1);
    gl.clearColor(0, 0, 0, surface && surface !== this.screen ? 0 : 1);
  }
  screenSurface(): GLSurface { return this.screen; }
  beginScreen(): void {
    this.screenActive = true;
    this.beginTarget(null);
  }
  presentScreen(): void {
    this.screenActive = false;
    this.beginTarget(null); this.resetTransform(); this.globalAlpha = 1; this.imageSmoothingEnabled = false;
    this.clearRect(0, 0, this.canvas.width, this.canvas.height);
    this.blendFactors("one", "zero");
    this.drawSurface(this.screen, 0, 0, this.screen.width, this.screen.height,
      0, 0, this.canvas.width, this.canvas.height);
  }
  blendFactors(source: BlendName, destination: BlendName): void {
    const gl = this.gl, factors: Record<BlendName, number> = {
      zero: gl.ZERO, one: gl.ONE, srcAlpha: gl.SRC_ALPHA, oneMinusSrcAlpha: gl.ONE_MINUS_SRC_ALPHA,
      dstAlpha: gl.DST_ALPHA, oneMinusDstAlpha: gl.ONE_MINUS_DST_ALPHA, srcColor: gl.SRC_COLOR,
      oneMinusSrcColor: gl.ONE_MINUS_SRC_COLOR, dstColor: gl.DST_COLOR, oneMinusDstColor: gl.ONE_MINUS_DST_COLOR,
    };
    gl.blendFunc(factors[source], factors[destination]);
  }
  alphaTest(comparison: AlphaComparison | null, reference = 0): void {
    const operations: AlphaComparison[] = ["never", "less", "equal", "lessEqual", "greater", "notEqual", "greaterEqual", "always"];
    this.gl.uniform1i(this.alphaFunction, comparison === null ? 0 : operations.indexOf(comparison) + 1);
    this.gl.uniform1f(this.alphaReference, reference);
  }
  drawSurface(surface: GLSurface, sx: number, sy: number, sw: number, sh: number, dx: number, dy: number, dw: number, dh: number, rgbaTint: readonly number[] = [1, 1, 1, 1]): void {
    this.quad({ handle: surface.texture, width: surface.width, height: surface.height, textureWidth: surface.width, textureHeight: surface.height },
      sx, sy, sw, sh, dx, dy, dw, dh, [rgbaTint[0]!, rgbaTint[1]!, rgbaTint[2]!, rgbaTint[3]! * this.state.alpha]);
  }
  drawTintedImage(source: CanvasImageSource, rgbaTint: readonly number[], offsets: readonly number[], ...arguments_: number[]): void {
    this.drawTexture(this.texture(source), arguments_, [rgbaTint[0]!, rgbaTint[1]!, rgbaTint[2]!, rgbaTint[3]! * this.state.alpha], offsets);
  }
  drawTintedImageAligned(source: CanvasImageSource, rgbaTint: readonly number[], offsets: readonly number[], pixelSize: number, ...arguments_: number[]): void {
    if (!Number.isFinite(pixelSize) || pixelSize <= 0) throw new Error("Sprite raster pixel size must be finite and positive");
    this.drawTexture(this.texture(source), arguments_, [rgbaTint[0]!, rgbaTint[1]!, rgbaTint[2]!, rgbaTint[3]! * this.state.alpha], offsets, pixelSize);
  }
  /** Authored actors supply their sequential f32 geometry before outer transforms. */
  drawTintedImageQuadRepeated(source: CanvasImageSource, rgbaTint: readonly number[], offsets: readonly number[], crop: readonly [number, number, number, number], points: readonly (readonly [number, number])[]): void {
    if (points.length !== 4) throw new Error("Laser beam must have four source corners");
    const texture = this.texture(source), gl = this.gl;
    //The selected448x64 authored Laser sheet has unpadded64-pixel height.
    //Its bounded x crop never crosses the logical448-pixel source boundary.
    if (texture.height !== texture.textureHeight || crop[0] < 0 || crop[0] + crop[2] > texture.width)
      throw new Error("Laser repeat requires actual unpadded height and bounded authored x crop");
    gl.bindTexture(gl.TEXTURE_2D, texture.handle);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
    try {
      this.submitQuad(texture, crop, points, [rgbaTint[0]!, rgbaTint[1]!, rgbaTint[2]!, rgbaTint[3]! * this.state.alpha], offsets);
    } finally {
      gl.bindTexture(gl.TEXTURE_2D, texture.handle);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    }
  }

  drawTintedImageQuad(source: CanvasImageSource, rgbaTint: readonly number[], offsets: readonly number[], crop: readonly [number, number, number, number], points: readonly (readonly [number, number])[], pixelSize?: number): void {
    if (points.length !== 4) throw new Error("An authored sprite must have four corners");
    if (pixelSize !== undefined && (!Number.isFinite(pixelSize) || pixelSize <= 0)) throw new Error("Sprite raster pixel size must be finite and positive");
    this.submitQuad(this.texture(source), crop, points, [rgbaTint[0]!, rgbaTint[1]!, rgbaTint[2]!, rgbaTint[3]! * this.state.alpha], offsets, pixelSize);
  }

  get globalAlpha(): number { return this.state.alpha; }
  set globalAlpha(alpha: number) { this.state.alpha = alpha; }
  get imageSmoothingEnabled(): boolean { return this.state.smoothing; }
  set imageSmoothingEnabled(value: boolean) { this.state.smoothing = value; }
  get fillStyle(): string { return this.state.fillStyle; }
  set fillStyle(value: string) { this.state.fillStyle = value; }
  save(): void { this.states.push({ ...this.state, transform: [...this.state.transform] }); }
  restore(): void { const state = this.states.pop(); if (state) this.state = state; }
  setTransform(a: number, b: number, c: number, d: number, e: number, f: number): void { this.state.transform = [a, b, c, d, e, f]; }
  resetTransform(): void { this.setTransform(1, 0, 0, 1, 0, 0); }
  translate(x: number, y: number): void {
    const m = this.state.transform; m[4] = m[0]! * x + m[2]! * y + m[4]!; m[5] = m[1]! * x + m[3]! * y + m[5]!;
  }
  scale(x: number, y: number): void { const m = this.state.transform; m[0]! *= x; m[1]! *= x; m[2]! *= y; m[3]! *= y; }
  rotate(angle: number): void {
    const m = this.state.transform, cos = Math.cos(angle), sin = Math.sin(angle), [a, b, c, d] = m;
    m[0] = a! * cos + c! * sin; m[1] = b! * cos + d! * sin; m[2] = c! * cos - a! * sin; m[3] = d! * cos - b! * sin;
  }
  clearRect(_x: number, _y: number, _width: number, _height: number): void {
    this.gl.clear(this.gl.COLOR_BUFFER_BIT | this.gl.DEPTH_BUFFER_BIT);
    this.drawLayers.set(this.currentTarget, 0);
  }
  fillRect(x: number, y: number, width: number, height: number): void {
    const match = /^#([0-9a-f]{6})$/i.exec(this.state.fillStyle); if (!match) throw new Error("Graphics fill color must use six-digit RGB");
    const rgb = [0, 2, 4].map(offset => parseInt(match[1]!.slice(offset, offset + 2), 16) / 255);
    this.fillRectRGBA(x, y, width, height, [rgb[0]!, rgb[1]!, rgb[2]!, 1]);
  }
  /** Source menu fades retain float color rather than an eight-bit CSS color. */
  fillRectRGBA(x: number, y: number, width: number, height: number, rgba: readonly number[]): void {
    this.quad(this.white, 0, 0, 1, 1, x, y, width, height, [rgba[0]!, rgba[1]!, rgba[2]!, rgba[3]! * this.state.alpha]);
  }
  drawImage(source: CanvasImageSource, ...arguments_: number[]): void {
    this.drawTexture(this.texture(source), arguments_, [1, 1, 1, this.state.alpha]);
  }
  private drawTexture(texture: Texture, arguments_: number[], tint: number[], offsets: readonly number[] = [0, 0, 0], pixelSize?: number): void {
    let sx = 0, sy = 0, sw = texture.width, sh = texture.height, dx = 0, dy = 0, dw = sw, dh = sh;
    if (arguments_.length === 2) [dx, dy] = arguments_ as [number, number];
    else if (arguments_.length === 4) [dx, dy, dw, dh] = arguments_ as [number, number, number, number];
    else if (arguments_.length === 8) [sx, sy, sw, sh, dx, dy, dw, dh] = arguments_ as [number, number, number, number, number, number, number, number];
    else throw new Error("Invalid graphics image draw arguments");
    this.quad(texture, sx, sy, sw, sh, dx, dy, dw, dh, tint, offsets, pixelSize);
  }
  private quad(texture: Texture, sx: number, sy: number, sw: number, sh: number, dx: number, dy: number, dw: number, dh: number, tint: number[], offsets: readonly number[] = [0, 0, 0], pixelSize?: number): void {
    this.submitQuad(texture, [sx, sy, sw, sh], [[dx, dy], [dx + dw, dy], [dx, dy + dh], [dx + dw, dy + dh]], tint, offsets, pixelSize);
  }
  private submitQuad(texture: Texture, crop: readonly [number, number, number, number], points: readonly (readonly [number, number])[], tint: number[], offsets: readonly number[], pixelSize?: number): void {
    const [sx, sy, sw, sh] = crop;
    const m = this.state.transform, xy = (x: number, y: number) => [m[0]! * x + m[2]! * y + m[4]!, m[1]! * x + m[3]! * y + m[5]!];
    // Source pixels are normalized before the padded-upload texture scale is applied.
    // Keeping both operations in f32 matches the authored renderer's public UV inputs.
    const uv = (pixel: number, sourceExtent: number, uploadExtent: number) =>
      Math.fround(Math.fround(pixel * Math.fround(1 / sourceExtent)) * Math.fround(sourceExtent / uploadExtent));
    const left = uv(sx, texture.width, texture.textureWidth), right = uv(sx + sw, texture.width, texture.textureWidth);
    const top = uv(sy, texture.height, texture.textureHeight), bottom = uv(sy + sh, texture.height, texture.textureHeight);
    const coordinates = [[left, top], [right, top], [left, bottom], [right, bottom]];
    const corners = points.map((point, index) => [...xy(point[0], point[1]), ...coordinates[index]!]);
    if (pixelSize !== undefined) {
      // Apply the observed grid only after the complete sprite transform.
      // Rotation 180 can leave sin(pi) roundoff; actual rotated quads retain
      // their submitted geometry instead of snapping their bounding box.
      const roundoff = Number.EPSILON * Math.max(1, ...corners.flatMap((corner) => corner.slice(0, 2).map(Math.abs))) * 32;
      if (Math.abs(corners[0]![1]! - corners[1]![1]!) <= roundoff
          && Math.abs(corners[0]![0]! - corners[2]![0]!) <= roundoff) {
        for (const corner of corners) {
          corner[0] = Math.sign(corner[0]!) * Math.floor(Math.abs(corner[0]!) / pixelSize + 0.5) * pixelSize;
          corner[1] = Math.sign(corner[1]!) * Math.floor(Math.abs(corner[1]!) / pixelSize + 0.5) * pixelSize;
        }
      }
    }
    const layer = this.drawLayers.get(this.currentTarget) ?? 0;
    this.drawLayers.set(this.currentTarget, layer + 1);
    const depth = Math.fround(-1 - Math.fround(0.01) * layer);
    const buffer = new Float32Array([0, 2, 1, 1, 2, 3].flatMap(index => {
      const corner = corners[index]!;
      return [corner[0]!, corner[1]!, depth, corner[2]!, corner[3]!];
    }));
    const gl = this.gl; gl.bindTexture(gl.TEXTURE_2D, texture.handle);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, this.state.smoothing ? gl.LINEAR : gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, this.state.smoothing ? gl.LINEAR : gl.NEAREST);
    gl.bindBuffer(gl.ARRAY_BUFFER, this.vertices); gl.bufferSubData(gl.ARRAY_BUFFER, 0, buffer); gl.uniform4fv(this.color, tint); gl.uniform3fv(this.offsets, offsets); gl.drawArrays(gl.TRIANGLES, 0, 6);
  }
  asCanvasContext(): CanvasRenderingContext2D { return this as unknown as CanvasRenderingContext2D; }
  finish(): void { this.gl.finish(); }
}
