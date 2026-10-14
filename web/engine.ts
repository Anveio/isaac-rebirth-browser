import wasmUrl from "./generated/engine.wasm?url";

export type Screen = "title" | "profile" | "main" | "character" | "seed" | "playing";
export type Direction = "left" | "right" | "up" | "down";
export interface Player {
  visible: boolean; inputCooldown: number; dead: boolean;
  x: number; y: number; vx: number; vy: number; hp: number; maxHp: number;
  soulHp: number; blackHeartMask: number; eternalHearts: number;
  facing: Direction; bodyFacing: Direction; walkFrame: number; headFrame: number; shooting: boolean; shootingLock: number;
  renderOffset: { x: number; y: number }; controlsEnabled: boolean;
  eye: "negative" | "positive";
}
export interface Tear {
  id: number; x: number; y: number; vx: number; vy: number; age: number;
  spawnSeed: number; collisionRadius: number; gridContact: boolean;
  motion: {
    position: {x: number; y: number}; presentedPosition: {x: number; y: number};
    velocity: {x: number; y: number}; height: number; fallingSpeed: number;
    fallingAcceleration: number; renderOffsetY: number; lifetime: "alive" | "dying" | "removed";
  };
}
export interface ActorSprite {
  animation: string; frame: number; playing: boolean; speed: number; scale: {x: number; y: number};
  offset: {x: number; y: number}; flipX: boolean; flipY: boolean;
  tint: [number, number, number, number]; offsets: [number, number, number];
}
export interface Familiar {
  id: number; variant: number; subtype: number; spawnSeed: number; owner: ActorRef; parent: ActorRef;
  parentType: number; parentVariant: number; birthGameUpdate: number; actorRng: ItemRng;
  position: {x: number; y: number}; presentedPosition: {x: number; y: number}; velocity: {x: number; y: number};
  configuredFriction: number; currentFriction: number; collisionRadius: number; collisionMass: number;
  collisionDamage: number; hp: number; maxHp: number; shadowSize: number; gridCollisionClass: number; entityCollisionClass: number;
  flags: number; visible: boolean; active: boolean; removed: boolean; followerIndex: number; appearanceCounter: number;
  sprite: ActorSprite; fullUpdates: number; unresolvedAiUpdates: number;
}
export interface SpriteEffect {
  id: number; variant: number; subtype: number; spawnSeed: number;
  spawnerType: number; spawnerVariant: number;
  parent: {actor: ActorRef; entityType: number; variant: number} | null;
  birthGameUpdate: number; actorRng: ItemRng; position: {x: number; y: number}; presentedPosition: {x: number; y: number};
  velocity: {x: number; y: number}; configuredFriction: number; currentFriction: number;
  collisionRadius: number; collisionMass: number; collisionDamage: number; hp: number; maxHp: number;
  gridCollisionClass: number; entityCollisionClass: number; flags: number; visible: boolean; active: boolean; removed: boolean;
  renderOrderOffset: number; lifetimeDivisor: number; timeToLive: number; raw7f4: number; raw7f8: number;
  sprite: ActorSprite; fullUpdates: number;
}
export interface Impact { id: number; x: number; y: number; age: number; }
export interface ActorRef { kind: "player" | "familiar" | "tear" | "bomb" | "pickup" | "npc" | "projectile" | "effect"; id: number; }
export interface MtState { words: number[]; index: number; }
export interface GridSeedSample { roomRecord: number; gridIndex: number; seeds: [number, number]; }
export interface GridRecord {
  kind: number; variant: number; state: number; sourceWord14: number;
  primarySeed: number; initialized: boolean; secondarySeed: number;
}
export interface BootLifecycle {
  shared: MtState; savedPlayerSeed: number; savedGridSeedSamples: GridSeedSample[];
  levelGridSeedSamples: GridSeedSample[]; backdropSeed: number; menuRunSeed: number; menuSeedText: string; secretPoses: number[];
}
export interface SavedRunSummary {
  runSeed: string; seededRun: boolean; stage: number; gameUpdateCount: number;
  descriptorCount: number; copiedDescriptorCount: number; redHp: number; maxRedHp: number;
  inventory: {coins: number; bombs: number; keys: number};
  emptyGridCellsSaved: number; occupiedGridSeedsPending: number[];
  currentRoomRecord: number; currentRoomSeedSamples: {gridIndex: number;
    levelSeeds: [number, number]; savedSeeds: [number, number]; occupancy: "empty" | "occupied" | "padding";
    record: GridRecord | null}[];
  doorCellRecords: {gridIndex: number; record: GridRecord | null}[];
  itemPools: {poolSeed: number; rootRngSeed: number; secondaryRngSeed: number;
    specialFactor: number; specialCounters: number[]; removedSamples: {itemId: number; removed: boolean}[];
    pillEffects: number[]; weightSamples: {poolId: number; itemId: number; weight: number}[]};
}
export interface SoundEvent {
  sequence: number; id: number; sampleIndex: number; systemCounter: number; halfTick: number;
  looping: boolean; pitch: number; gain: number;
}
export interface AudioState {
  sfxVolume: number; nextSequence: number; events: SoundEvent[];
  pending: { id: number; volume: number; delay: number; looping: boolean; pitch: number }[];
  slots: Record<string, { lastTick: number | null; nextAllowed: number; lastVolume: number; loaded: boolean;
    samples: { looping: boolean; gain: number; pitch: number }[] }>;
}
export interface MusicChannel {
  generation: number | null; backendPath: string | null; queuedPath: string | null;
  queuedLooping: boolean; wrapperLooping: boolean; backendLooping: boolean; backendPlaying: boolean;
  playing: boolean; updatesSincePrepare: number; volume: number; targetVolume: number; fadeRate: number; gain: number; pitch: number;
}
export type MusicEvent = { type: 'start'; channel: number; generation: number; path: string; looping: boolean;
  queuedPath: string | null; queuedLooping: boolean; gain: number; pitch: number } |
  { type: 'stop'; channel: number; generation: number } |
  { type: 'gain'; channel: number; generation: number; gain: number } |
  { type: 'pitch'; channel: number; generation: number; pitch: number } |
  { type: 'loop'; channel: number; generation: number; looping: boolean };
export interface MusicState {
  volume: number; musicEnabled: boolean; activeChannel: number; currentId: number; queuedId: number;
  layerEnabled: boolean; pitch: number; targetPitch: number; channels: [MusicChannel, MusicChannel, MusicChannel];
  nextGeneration: number; events: MusicEvent[]; pending: { type: 'crossfade' | 'room'; id: number }[];
}
export interface ItemRng { state: number; rightA: number; left: number; rightB: number; }
export interface ItemPool {
  id: number; name: string; baseWeightTotal: number; rng: [ItemRng, ItemRng]; specialCounter: number;
  entries: {id: number; baseWeight: number; currentWeight: number; decreaseBy: number; removeOn: number;
    eligible: boolean; special: boolean}[];
}
export type MusicBackendCommand = { type: 'musicBackend'; channel: number; generation: number; event: 'ended' | 'loop' };
export interface Bomb {
  id: number; variant: number; spawnSeed: number | null; birthGameUpdate: number; friction: number;
  animation: string; animationFrame: number; animationPlaying: boolean; x: number; y: number;
  motion: {
    position: { x: number; y: number }; presentedPosition: { x: number; y: number };
    velocity: { x: number; y: number }; fuseUpdatesRemaining: number;
    fullUpdates: number; animationFrame: number; lifetime: "alive" | "exploding" | "removed";
  };
}
export interface Explosion { id: number; x: number; y: number; animationFrame: number; }
export interface Pickup {
  id: number; spawnSeed: number; variant: number; subtype: number; position: { x: number; y: number };
  x: number; y: number; actor: string; animation: string; animationFrame: number; animationPlaying: boolean;
  canCollect: boolean; collected: boolean; removed: boolean; collisionRadius: number; shadowSize: number; collectionDelay: number;
  overlay: string | null; overlayFrame: number; overlayPlaying: boolean;
  age: number; birthGameUpdate: number; charge: number; touched: boolean;
  chest: {rng: ItemRng; gridCollisionClass: number; entityCollisionClass: number; price: number; cooldown: number;
    lootBirths: {variant: number; spawnSeed: number; position: {x: number; y: number}; velocity: {x: number; y: number}; spriteScaleMultiplier: number | null}[]} | null;
  regularCollision: {gridCollisionClass: number; entityCollisionClass: number} | null;
  regularRng: ItemRng | null;
  spritesheetItemId: number;
  presentedPosition: {x: number; y: number}; anchorPosition: {x: number; y: number};
  velocity: {x: number; y: number}; configuredFriction: number; currentFriction: number; collisionMass: number;
}
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
  doors: { side: Direction; state: string; asset: string; spritesheet: string | null; x: number; y: number; targetRoom: number; slot: number; worldPosition: {x: number; y: number}; blownOpen: boolean }[];
  fx: { ambient: [number, number, number]; shading: string; overlay: string; overlayOrigin: { x: number; y: number }; overlayAnchor: { x: number; y: number }; overlayParallax: { x: number; y: number }; overlayMotionEvidence: string; overlayOpacity: number; actorTint: [number, number, number] };
  template: string; generation: string;
}
export interface RoomTemplate {
  type: number; variant: number; name: string; width: number; height: number;
  difficulty: number; weight: number; doorMask: number;
}
export interface RoomDescriptor {
  index: number; generatedIndex: number; gridIndex: number; assetStage: number; catalogIndex: number;
  template: RoomTemplate; doorMask: number; neighbors: (number | null)[];
  surpriseMiniboss: boolean;
  roomInitSeed: number; decorationSeed: number; spawnSeed: number; followingSeed: number;
}
export interface Floor {
  contract: string; stage: number; stageSeed: number; additionalRooms: number; attempts: number;
  descriptors: RoomDescriptor[]; grid: number[]; startingDescriptor: number;
  primaryState: number; primaryDraws: number; generatorState: number; gameFlags: number;
  specialWeights: number[]; basementWeights: number[];
  attemptReports: { attempt: number; initialGeneratorState: number; finalGeneratorState: number;
    initialPrimaryState: number; finalPrimaryState: number; outcome: string }[];
}
export interface Room {
  left: number; right: number; top: number; bottom: number; appearance: RoomAppearance;
  descriptor: RoomDescriptor | null;
  pathCosts: number[] | null;
  spawns: {tintedRockIndex: number; finalSpawnState: number; selectors: {groupIndex: number; gridIndex: number; category: "entity" | "grid"; type: number; variant: number; subtype: number; position: {x: number; y: number}; groupSeed: number; spawnSeed: number}[]} | null;
  pickups: Pickup[];
  backgroundStains: {sourceEffectId: number; spawnSeed: number; birthGameUpdate: number;
    position: {x: number; y: number}; sprite: ActorSprite}[];
  savedRegularPickups: {handle: number; variant: number; subtype: number; position: {x: number; y: number};
    spawnSeed: number; rngSeed: number; charge: number; touched: boolean}[];
  npcs: { id: number; kind: number; variant: number; subtype: number; spawnSeed: number;
    position: {x: number; y: number}; presentedPosition: {x: number; y: number}; velocity: {x: number; y: number};
    hp: number; maxHp: number; collisionRadius: number; collisionMass: number; collisionDamage: number;
    gridCollisionClass: number; entityCollisionClass: number; canShutDoors: boolean; boss: boolean;
    state: number; frameAge: number; animationFrame: number; animation: string; animationClock: number; animationPlaying: boolean; visible: boolean; savedInitializedState: number; shadowSize: number; dead: boolean; removed: boolean;
    damage: {pendingDamage: number; pendingFlags: number};
    keeper: {anchor: {x: number; y: number}; entityFlags: number; stateFrame: number; countdown: number;
      animation: string; animationClock: number; animationPlaying: boolean; tint: number[]; offsets: number[]} | null;
    level2Fly: {entityFlags: number; birthGameUpdate: number; feedbackVelocity: {x: number; y: number};
      gridContact: boolean; gridNormal: {x: number; y: number}; previousGridVelocity: {x: number; y: number}} | null;
    monstro: {stateFrame: number; choice: number; entityFlags: number; flipX: boolean; birthGameUpdate: number;
      projectileBirthCount: number; lastProjectileBatch: {count: number; target: {x: number; y: number}; parameterAngle: number} | null;
      deathBloodStart: boolean; deathExplosion: boolean; finalBossCallbackPending: boolean} | null;
    fireplace: {anchor: {x: number; y: number}; authoredGridIndex: number; entityFlags: number; countdown: number;
      actorRngState: number; renderOrderOffset: number; flameScale: number; flameRenderMode: number;
      particleBirthCount: number; lastParticleBirth: {parentId: number; spawnSeed: number; position: {x: number; y: number}; velocity: {x: number; y: number}} | null;
      lootBirth: {parentId: number; variant: number; spawnSeed: number; position: {x: number; y: number}} | null} | null }[];
  authoredActorOrder: {kind: "npc" | "pickup"; id: number}[];
  managerCounts: { countablePending: number; countableCurrent: number; bossPending: number; bossCurrent: number };
  bossCount: number; bossPresentationIndex: number;
  combatClear: boolean; combatClearCountdown: number; combatShutDoorSlots: number[];
  grid: { width: number; height: number; cells: {selector: number; entity: {kind: number; variant: number; state: number; collisionClass: number; spawnSeed: number} | null}[] } | null;
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
  enemyProjectiles: {id:number; spawnSeed:number; position:{x:number;y:number}; presentedPosition:{x:number;y:number}; velocity:{x:number;y:number}; height:number; scale:number; animation:string|null; animationFrame:number; dead:boolean; removed:boolean}[];
  schema: 1; tickRate: 60; tick: number; menuAnimationTick: number; runTick: number; gameUpdateCount: number; screen: Screen;
  titleAnimationFrame: number; menuFade: { active: boolean; alpha: number; color: [number, number, number, number] };
  runStartRequest: { inputPending: boolean; fading: boolean; fadeAlpha: number; systemPending: boolean };
  systemClock: { systemCounter: number; interpolationEnabled: boolean };
  runAdmission: { clock: { milliseconds: number; subMillisecondNanoseconds: number; nanosecondsPerStep: number };
    requestedTimeAnchorMs: number | null; initialFade: number; fadeRate: number;
    completionCallbacksPending: boolean };
  audio: AudioState;
  music: MusicState;
  profile: number; mainSelection: number; characterSelection: number; character: string; difficulty: string;
  seedText: string; seedValid: boolean; runSeed: string; initialSeed: number; runSeedValue: number; bootEpochSeed: number; bootLifecycle: BootLifecycle;
  runSeeds: {startSeed: number; streamState: number; initializedFields: number[]} | null;
  itemPools: {seed: number; rootRng: ItemRng; secondaryRng: ItemRng; pools: ItemPool[];
    specialFactor: number; removed: boolean[]; selected: boolean[]; roomRoutes: number[]; pillEffects: number[]} | null;
  playerItems: {initializerSeed: number; managerRng: ItemRng; initializerRng: ItemRng;
    itemRngs: ItemRng[]; trinketRngs: ItemRng[]; pocketRngs: ItemRng[]; entitySpawnSeed: number;
    ownedCounts: number[]; temporaryCollectibles: {itemId: number; count: number; cooldown: number}[]; queuedId: number | null; queuedCharge: number; queuedTouched: boolean;
    pickupAnimationFrame: number; pickupAnimationPlaying: boolean; pickupAnimationGuard: boolean;
    pickupPresentation: {itemId: number; animationFrame: number; animationPlaying: boolean} | null;
    collectionDelay: number; activeId: number; activeCharge: number; activeSubordinateCharge: number;
    activeStateD80: number; activeStateD84: number; reserveCharge: number; activeFlag3076: boolean;
    unevaluatedCacheFlags: number} | null;
  informationAnimation: {state: number; delay: number; currentActor: string | null;
    currentClip: string | null; spritesheet: string | null; definitionLoaded: boolean; frame: number; playing: boolean};
  savedRunSummary: SavedRunSummary | null;
  pendingMenuInitializations: { afterAudioRequests: number; initialization: { type: "loadFreshProfile"; profile: number } | { type: "resetCharacterSeed" } | { type: "commitCharacterSeed"; seed: number } }[];
  stage: number; stageSeed: number; seededRun: boolean; achievementEligibility: boolean;
  levelProgress: {flags: number; heartCollected: boolean; lastPenaltyStage: number | null};
  nightmares: {order: number[]; pathIndex: number; currentActor: string | null; requestPending: boolean; systemActive: boolean;
    active: boolean; definitionLoaded: boolean; progressDefinitionLoaded: boolean; stageResources: number[];
    counter: number; frame: number; playing: boolean; stageIndex: number; alternate: boolean};
  profiles: { slot: number; unlockedCharacters: string[]; selectedCharacter: string; achievements: boolean[] }[];
  characterCatalog: { name: string; animation: string }[];
  paused: boolean; pressed: string[];
  floor: Floor; currentRoom: number; visitedRooms: number[];
  roomDisplayFlags: number[]; revealedSecretConnections: [number, number][];
  persistedRooms: Record<string, Room>;
  roomTransition: {fromRoom: number; targetRoom: number; fromSlot: number; targetSlot: number; phase: "out" | "in" | "bossScene" | "bossCleanup"; fullUpdates: number; bossScene: {presentationIndex: number; animationFrame: number; animationPlaying: boolean} | null} | null;
  playerStats: {shotSpeed: number; damage: number; tearHeight: number; tearFallingSpeed: number; tearFallingAcceleration: number; moveSpeed: number};
  player: Player; room: Room; stageEffects: StageEffects; inventory: { coins: number; bombs: number; keys: number };
  publicActions: { menuConfirm: boolean; menuBack: boolean; activePressed: boolean; activeTriggered: boolean; pocketPressed: boolean; pocketTriggered: boolean; enterWasDown: boolean; spaceWasDown: boolean; escapeWasDown: boolean; pocketWasDown: boolean };
  playerAnimation: { current: "Trapdoor" | "Appear" | "Death" | "UseItem" | "Hit" | null; frame: number; playing: boolean; guard: boolean };

  camera: {scroll: {x: number; y: number}; lastPlayerPosition: {x: number; y: number};
    histories: {capacity: number; head: number; samples: {x: number; y: number}[]}[]} | null;
  nextPickup: number;
  minimap: { cells: { x: number; y: number; visibility: "unvisited" | "visited" | "current" }[] };
  tears: Tear[]; impacts: Impact[]; fireCooldown: number; nextTear: number;
  fireParticles: {id: number; parentNpc: number; parentType: number; parentVariant: number; spawnSeed: number; birthGameUpdate: number;
    position: {x: number; y: number}; presentedPosition: {x: number; y: number}; velocity: {x: number; y: number};
    hp: number; maxHp: number; collisionRadius: number; collisionMass: number; collisionDamage: number; baseFriction: number;
    gridCollisionClass: number; entityCollisionClass: number; visible: boolean;
    lifetimeDivisor: number; timeToLive: number; fullUpdates: number; animationClock: number; animationPlaying: boolean;
    dead: boolean; removed: boolean}[];
  familiars: Familiar[]; nextFamiliar: number; spriteEffects: SpriteEffect[]; nextEffect: number;
  maxFireDelay: number; headTimer: number; actorScheduler: { actors: ActorRef[]; identities: {actor: ActorRef; sequenceId: number}[]; roomNextSequence: number };
  bombs: Bomb[]; explosions: Explosion[]; nextBomb: number; damageCooldown: number;
  bombPlacementCooldown: { remainingUpdates: number }; bombActionWasDown: boolean;
  runIntroduction: RunIntroduction; pauseMenu: PauseMenu;
  playerDeath: { lastDamageFlags: number; gameOver: { state: number; damageFlags: number; diaryFrame: number; diaryPlaying: boolean } | null; systemPending: "restart" | "exit" | null };
}
export type Command = { type: "key"; code: string; down: boolean } | { type: "text"; text: string } | { type: "tick"; count: number; timestampMillis?: number } | { type: "audioVolume"; sfx_volume: number } | { type: 'musicVolume'; volume: number } | MusicBackendCommand;
interface Exports extends WebAssembly.Exports {
  memory: WebAssembly.Memory; abi_version: () => number; reset: (initialSeed: number, bootEpochSeed: number) => number; request_ptr: (length: number) => number;
  command: () => number; snapshot_ptr: () => number; snapshot_len: () => number;
  error_ptr: () => number; error_len: () => number; public_keys_ptr: () => number; public_keys_len: () => number;
}
export class Engine {
  /** The engine-owned key surface: the only codes `key` commands accept. */
  readonly publicKeys: ReadonlySet<string>;
  private constructor(private readonly wasm: Exports) {
    const ptr = wasm.public_keys_ptr(), length = wasm.public_keys_len();
    const size = wasm.memory.buffer.byteLength;
    if (!Number.isSafeInteger(ptr) || ptr < 0 || !Number.isSafeInteger(length) || length <= 0
      || ptr > size || length > size - ptr)
      throw new Error("WASM public key surface has invalid bounds");
    const keys: unknown = JSON.parse(this.read(ptr, length));
    if (!Array.isArray(keys) || keys.length === 0
      || !keys.every((key: unknown) => typeof key === "string" && key.length > 0)
      || new Set(keys).size !== keys.length)
      throw new Error("WASM public key surface must be a nonempty array of unique strings");
    this.publicKeys = new Set(keys as string[]);
  }
  static async create(initialSeed: number, bootEpochSeed = 0) {
    Engine.validateSeed(initialSeed);
    Engine.validateBootSeed(bootEpochSeed);
    const response = await fetch(wasmUrl);
    if (!response.ok) throw new Error(`WASM load failed: ${response.status}`);
    const { instance } = await WebAssembly.instantiate(await response.arrayBuffer());
    if (typeof instance.exports.abi_version !== "function" || instance.exports.abi_version() !== 2
      || typeof instance.exports.public_keys_ptr !== "function" || typeof instance.exports.public_keys_len !== "function")
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
  snapshot(): State { return JSON.parse(this.snapshotJson()) as State; }
  snapshotJson(): string { const ptr = this.wasm.snapshot_ptr(); return this.read(ptr, this.wasm.snapshot_len()); }
  private read(ptr: number, length: number) { return new TextDecoder().decode(new Uint8Array(this.wasm.memory.buffer, ptr, length)); }
}
