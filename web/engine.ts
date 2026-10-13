import wasmUrl from "./generated/engine.wasm?url";

export type Screen = "title" | "profile" | "main" | "character" | "seed" | "playing";
export type Direction = "left" | "right" | "up" | "down";
export interface Player {
  x: number; y: number; vx: number; vy: number; hp: number; maxHp: number;
  facing: Direction; bodyFacing: Direction; walkFrame: number; headFrame: number; shooting: boolean;
  renderOffset: { x: number; y: number };
}
export interface Tear { id: number; x: number; y: number; vx: number; vy: number; age: number; }
export interface Impact { id: number; x: number; y: number; age: number; }
export interface Bomb { id: number; x: number; y: number; age: number; }
export interface RunIntroduction { age: number; active: boolean; }
export interface PauseMenu {
  phase: "hidden" | "appearing" | "idle" | "disappearing";
  age: number; selection: number; statFrames: [number, number, number, number, number, number];
}
export interface RoomAppearance {
  backdrop: string; columns: number; rows: number; tileSize: number; origin: { x: number; y: number };
  backdropPatchSize: { width: number; height: number }; backdropPatchColumns: number;
  floorQuads: RoomQuad[]; wallQuads: RoomQuad[]; tutorial: boolean;
  wallVariantCatalog: { sourcePatchIndex: number; name: string; weight: number; eligibility: string }[];
  wallGenerationEvidence: string;
  doors: { side: Direction; state: "Opened" | "Closed"; x: number; y: number; targetRoom: number }[];
  fx: { ambient: [number, number, number]; shading: string; overlay: string; overlayOrigin: { x: number; y: number }; overlayAnchor: { x: number; y: number }; overlayParallax: { x: number; y: number }; overlayMotionEvidence: string; overlayOpacity: number; actorTint: [number, number, number] };
  template: string; generation: string;
}
export interface Room {
  left: number; right: number; top: number; bottom: number; appearance: RoomAppearance;
  projectileBounds: { left: number; right: number; top: number; bottom: number };
}
export interface Rect { x: number; y: number; width: number; height: number; }
export interface RoomQuad { sourcePatchIndex: number; sourceRect: Rect; destRect: Rect; flipX: boolean; flipY: boolean; }
export type BlendFactor = "one" | "zero" | "source-alpha" | "one-minus-source-alpha" | "destination-color";
export interface Blend { source: BlendFactor; destination: BlendFactor; }
export interface StageEffects {
  viewport: { width: number; height: number };
  maskRaster: { pixelSize: number; rounding: "half-away-from-zero" };
  shadowCatalog: { asset: string; sourceSize: { width: number; height: number }; definitions: { entityType: number; variant: number; sizePercent: number }[] };
  ambientColor: [number, number, number, number];
  lightGradient: { asset: string; crop: Rect };
  lights: { ownerId: number; x: number; y: number; radius: number; alpha: number; destinationRect: Rect }[];
  shadowCasters: { ownerId: number; x: number; y: number; width: number; height: number; asset: string; authoredSizePercent: number; sourceSize: { width: number; height: number }; destinationRect: Rect }[];
  groundShadowMaskBlend: Blend; lightingMaskBlend: Blend; groundShadowCompositeBlend: Blend; lightingCompositeBlend: Blend;
  sampler: { filter: "nearest"; wrap: "clamp-to-edge" };
  groundShadowCompositeRGBA: [number, number, number, number];
  lightingCompositeRGBA: [number, number, number, number];
  actorTint: [number, number, number];
  overlay: { asset: string; sourceCrop: Rect; offset: { x: number; y: number }; modulationRGBA: [number, number, number, number]; blend: Blend; simulationTick: number; motionEvidence: string };
}
export interface State {
  schema: 1; tickRate: 60; tick: number; menuAnimationTick: number; runTick: number; screen: Screen;
  titleAnimationFrame: number; menuFade: { active: boolean; alpha: number; color: [number, number, number, number] };
  systemClock: { systemCounter: number; interpolationEnabled: boolean };
  profile: number; mainSelection: number; characterSelection: number; character: string; difficulty: string;
  seedText: string; seedValid: boolean; runSeed: string; initialSeed: number; runSeedValue: number; bootEpochSeed: number;
  stage: number; stageSeed: number; seededRun: boolean; achievementEligibility: boolean;
  profiles: { slot: number; unlockedCharacters: string[]; selectedCharacter: string }[];
  characterCatalog: { name: string; animation: string }[];
  paused: boolean; pressed: string[];
  player: Player; room: Room; stageEffects: StageEffects; inventory: { coins: number; bombs: number; keys: number };
  minimap: { cells: { x: number; y: number; visibility: "unvisited" | "visited" | "current" }[] };
  tears: Tear[]; impacts: Impact[]; fireCooldown: number; nextTear: number;
  bombs: Bomb[]; explosions: Bomb[]; nextBomb: number; damageCooldown: number;
  runIntroduction: RunIntroduction; pauseMenu: PauseMenu;
}
export type Command = { type: "key"; code: string; down: boolean } | { type: "text"; text: string } | { type: "tick"; count: number };
interface Exports extends WebAssembly.Exports {
  memory: WebAssembly.Memory; abi_version: () => number; reset: (initialSeed: number, bootEpochSeed: number) => number; request_ptr: (length: number) => number;
  command: () => number; snapshot_ptr: () => number; snapshot_len: () => number;
  error_ptr: () => number; error_len: () => number;
}
export class Engine {
  private constructor(private readonly wasm: Exports) {}
  static async create(initialSeed: number, bootEpochSeed = 0) {
    Engine.validateSeed(initialSeed);
    Engine.validateBootSeed(bootEpochSeed);
    const response = await fetch(wasmUrl);
    if (!response.ok) throw new Error(`WASM load failed: ${response.status}`);
    const { instance } = await WebAssembly.instantiate(await response.arrayBuffer());
    if (typeof instance.exports.abi_version !== "function" || instance.exports.abi_version() !== 2)
      throw new Error("WASM engine ABI is stale; rebuild with npm run build:wasm");
    const engine = new Engine(instance.exports as Exports); engine.reset(initialSeed, bootEpochSeed); return engine;
  }
  private static validateSeed(initialSeed: number) {
    if (!Number.isInteger(initialSeed) || initialSeed < 1 || initialSeed > 0xffffffff)
      throw new Error("Initial seed must be a nonzero unsigned 32-bit entropy value");
  }
  private static validateBootSeed(bootEpochSeed: number) {
    if (!Number.isInteger(bootEpochSeed) || bootEpochSeed < 0 || bootEpochSeed > 0xffffffff)
      throw new Error("Boot epoch seed must be an unsigned 32-bit value");
  }
  reset(initialSeed: number, bootEpochSeed = 0): State {
    Engine.validateSeed(initialSeed);
    Engine.validateBootSeed(bootEpochSeed);
    if (this.wasm.reset(initialSeed, bootEpochSeed) !== 0) throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()));
    return this.snapshot();
  }
  command(command: Command): State {
    if (command.type === "tick" && (!Number.isSafeInteger(command.count) || command.count < 0)) throw new Error("Tick count must be an unsigned integer");
    const text = JSON.stringify(command); if (text === undefined) throw new Error("Command must be JSON");
    const bytes = new TextEncoder().encode(text);
    if (bytes.length > 65_536) throw new Error("Command exceeds 64 KiB");
    const ptr = this.wasm.request_ptr(bytes.length); if (ptr === 0) throw new Error("WASM request allocation failed");
    new Uint8Array(this.wasm.memory.buffer, ptr, bytes.length).set(bytes);
    if (this.wasm.command() !== 0) throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()));
    return this.snapshot();
  }
  key(code: string, down: boolean): State { return this.command({ type: "key", code, down }); }
  text(text: string): State { return this.command({ type: "text", text }); }
  tick(count = 1): State { return this.command({ type: "tick", count }); }
  snapshot(): State { const ptr = this.wasm.snapshot_ptr(); return JSON.parse(this.read(ptr, this.wasm.snapshot_len())) as State; }
  private read(ptr: number, length: number) { return new TextDecoder().decode(new Uint8Array(this.wasm.memory.buffer, ptr, length)); }
}
