export type Screen = "title" | "profile" | "main" | "options" | "controls" | "keyboard" | "character" | "challenge" | "seed" | "playing";
/** Game+10ECD8: the run's difficulty (engine/src/difficulty.rs). */
export type Difficulty = "normal" | "hard";
export type Direction = "left" | "right" | "up" | "down";
export interface Player {
  pocketActions?: {sourceD98: number | null; heldCallbacks3054: number | null; source30d0: number | null; frontier: PocketActionFrontier | null; playerBe8StoreReached: boolean; gnawedLeafFrontier: boolean};
  entitySource?: { entityType: number; variant: number; spawnerType: number; spawnerVariant: number };
  canShoot: boolean; initialized3074: boolean;
  visible: boolean; inputCooldown: number; dead: boolean;
  x: number; y: number; vx: number; vy: number; hp: number; maxHp: number;
  soulHp: number; blackHeartMask: number; eternalHearts: number; goldenKey?: boolean | null;
  facing: Direction; bodyFacing: Direction; walkFrame: number; headFrame: number; shooting: boolean; shootingLock: number;
  renderOffset: { x: number; y: number }; controlsEnabled: boolean;
  transformationCounts: [number, number] | null;
  flying: boolean | null; gridCollisionClass: number | null;
  slowing?: { remaining: number; factor: number; friction: number };
  eye: "negative" | "positive";
  shotsFired: number;
}
export type PocketActionFrontier = {kind: "playerVariant" | "inputDevice" | "heldCounter" | "heldRecords" | "capacity" | "trinketRemoval" | "queuedFlush" | "pendingCache" | "missingGrid" | "pocketInit"} | {kind: "placement"; frontier: OrdinaryPlacementFrontier};
export interface AttackColor { tint: [number,number,number,number]; sourceMiddle: [number,number,number,number]; offsets: [number,number,number]; }
export interface Costume229 {
  slots: ({entry: number; layer: number; bodyGroup: boolean} | null)[];
  sprite: {clip: string; frame: number; playing: boolean; overrideActive: boolean};
  priority: number; historyFrontier: string | null; mainClipFrontier: boolean;
  lastShootMainClipBa8: string | null;
}
export interface Tear {
  id: number; x: number; y: number; vx: number; vy: number; age: number;
  spawnSeed: number; sourceVariant: number | null; collisionRadius: number; gridContact: boolean;
  shotOrdinal: number | null; collisionDamage: number; baseDamage: number; scale: number; animationTier: number;
  attackFlags: [number,number]; sourceColor: AttackColor;
  controlled?: { parent: ActorRef | null; active: boolean; dead: boolean; visible: boolean; entityFlags: number; configuredFriction: number; currentFriction: number; savedPosition: {x:number;y:number}; velocityDeltaB28: {x:number;y:number}; currentColor: AttackColor; birthGameUpdate: number; hitSequences: number[]; registryKnown: boolean; currentParamsKnown: boolean; childChainEmpty: boolean | null };
  motion: {
    position: {x: number; y: number}; presentedPosition: {x: number; y: number};
    velocity: {x: number; y: number}; height: number; fallingSpeed: number;
    fallingAcceleration: number; renderOffsetY: number; lifetime: "alive" | "dying" | "removed";
  };
}
export interface ActorSprite {
  animation: string; frame: number; playing: boolean; speed: number; scale: {x: number; y: number};
  offset: {x: number; y: number}; flipX: boolean; flipY: boolean;
  rotation: number; layerFlips: Record<string, [boolean, boolean]>; layerOffsets?: Record<string, [number, number]>;
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
type ParticleFlight = "initializing" | {flying: {height: number; verticalVelocity: number; gravity: number; layerRotation: number}};
export type PoopEffect = { particle58: ParticleFlight } | { particle27: ParticleFlight } | { explosion43: {frameTarget: number} }
  | { particle5: { particle: ParticleFlight; subtype: number; stage: number } };
export interface SpriteEffect {
  id: number; variant: number; subtype: number; spawnSeed: number;
  spawnerType: number; spawnerVariant: number;
  parent: {actor: ActorRef; entityType: number; variant: number} | null;
  birthGameUpdate: number; actorRng: ItemRng; position: {x: number; y: number}; presentedPosition: {x: number; y: number};
  renderOffset: {x: number; y: number};
  velocity: {x: number; y: number}; configuredFriction: number; currentFriction: number;
  collisionRadius: number; collisionMass: number; collisionDamage: number; hp: number; maxHp: number;
  gridCollisionClass: number; entityCollisionClass: number; flags: number; visible: boolean; active: boolean; removed: boolean;
  renderOrderOffset: number; lifetimeDivisor: number; timeToLive: number; raw7f4: number; raw7f8: number;
  raw7d8: boolean; poopEffect?: PoopEffect; flyEffect?: { kind: string }; creepEffect?: { counter: number; size: number };
  sprite: ActorSprite; fullUpdates: number;
  bloodEffect?: {kind: "poof04" | "bloodGush"};
  footprint?: boolean;
}
export interface ActorRef { kind: "player" | "familiar" | "tear" | "knife" | "laser" | "bomb" | "pickup" | "slot" | "npc" | "projectile" | "effect"; id: number; }
/** Entity_Slot (type 6): an idle machine or beggar (engine/src/slots.rs). */
export interface Slot {
  id: number; variant: number; subtype: number; spawnSeed: number;
  position: {x: number; y: number}; presentedPosition: {x: number; y: number}; velocity: {x: number; y: number};
  targetPosition: {x: number; y: number}; positionOffset: {x: number; y: number}; sizeMulti: {x: number; y: number};
  collisionRadius: number; collisionMass: number; configuredFriction: number; currentFriction: number;
  state: number; actor: string; animation: string; animationFrame: number;
}
export type OrdinaryPlacementFrontier =
  {kind: "playerUpdate"; field: string} |
  {kind: "factory"; entityType: number; variant: number; subtype: number} |
  {kind: "actorEffects" | "missingBody" | "nonfinitePosition"; actor: ActorRef} |
  {kind: "collectibleEffects"; collectibleId: number} |
  {kind: "pocketEffects" | "activeEffects" | "playerDamageEffects" | "gridEffects" | "chestEffects" | "restore" | "missingHistory"};
export type PickupGrantFrontier = {kind: "missingHistory"} |
  {kind: "input"; field: string} | {kind: "placement"; frontier: OrdinaryPlacementFrontier} |
  {kind: "operator"; message: string};
export interface PickupGrantHistory {
  firstUnownedEffect: {collectibleId: number; frontier: PickupGrantFrontier} | null;
  itemRngFrontiers: Record<number, PickupGrantFrontier>;
}
export interface Knife {
  id: number; entityType: number; variant: number; subtype: number; spawnSeed: number; birthGameUpdate: number; actorRng: ItemRng;
  spawnerType: number; spawnerVariant: number; parent: ActorRef | null; parentNullKnown?: boolean;
  childChain4a0: "unknown" | "empty" | { linked: ActorRef };
  active: boolean; dead: boolean; visible: boolean;
  position: {x: number; y: number}; savedPosition: {x: number; y: number};
  homePosition: {x: number; y: number}; homingTarget: ActorRef | null; homingTargetKnown?: boolean; sourceMotionKnown?: boolean;
  renderOffset: {x: number; y: number}; velocity: {x: number; y: number};
  configuredFriction: number; currentFriction: number;
  collisionRadius: number; collisionMass: number; collisionDamage: number;
  hp: number; maxHp: number; raw494: boolean;
  entityFlags: number; gridCollisionClass: number; entityCollisionClass: number;
  attackFlags: [number, number]; sourceColor: AttackColor; currentColor: AttackColor; fast: boolean; sprite: ActorSprite;
  raw7e0: number; raw7e4: number; raw7e8: number; raw7ec: number;
  raw7f0: number; raw7f4: number; raw7f8: number; raw7fc: {x: number; y: number};
  raw804: {x: number; y: number}; raw80c: number; raw810: number; raw814: boolean;
  line: {origin: {x: number; y: number}; offset: {x: number; y: number}; subdivisions: number | null; mode: number | null; width: number | null;
    controls: {position: {x: number; y: number}; incoming: {x: number; y: number}; outgoing: {x: number; y: number}}[];
    history: {x: number; y: number}[]; points: {x: number; y: number}[]; priorPoints: {x: number; y: number}[]; length: number | null};
  lineRaw18: [number, number] | null;
  raw8a0: number; raw8a4: number; raw8a8: number; hitSequences: number[];
}
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
export interface AcquisitionHistory {
  firstRecords: Record<string, {sourceUpdateWord: number; sourceStageWord: number;
    sourceStageType: number; sourceRoomTypeWord: number; sourcePoolWord488: number} | null>;
}
export interface ActiveChargeHistory {
  unowned: Partial<Record<"activeId" | "charge" | "subordinate" | "reserve",
    {kind: string; [field: string]: unknown}>>;
  unownedEffects: {kind: string; [field: string]: unknown}[];
}
export interface SavedRunSummary {
  acquisitionHistory?: AcquisitionHistory;
  runSeed: string; seededRun: boolean; stage: number; gameUpdateCount: number;
  descriptorCount: number; copiedDescriptorCount: number; redHp: number; maxRedHp: number;
  inventory: {coins: number; bombs: number; keys: number};
  playerItems: {
    pickupGrantHistory?: PickupGrantHistory | null;
    activeChargeHistory?: ActiveChargeHistory | null;
    ordinaryProducerHistory?: {frontier: OrdinaryPlacementFrontier | null} | null;
    ownedCounts: number[]; itemRngStates: number[];
    queryHistory?: { permanentCounts: boolean; temporaryRecords: boolean; temporarySourceCache?: {maskCf8: number | null; pendingD18: boolean | null} | null; temporaryUnknownInputs?: { configKind: number; itemId: number }[]; trinketSlots: boolean; primaryC08: boolean; damageHealthKnown?: boolean; cachedCallbacks?: number; temporaryEvaluation?: { possibleCacheMask: number | null; writerPcs: number[] }; unresolvedEffects?: Record<string, string> };
    trinketRngStates: number[]; pocketRngStates: number[]; activeId: number;
    activeCharge: number; saveWordD88: number; reserveCharge: number;
    shotSpeedInputs?: {sourceD38: number};
    damageInputs?: {sourceD30: number};
    fireDelayInputs?: {sourceD2c: number};
    rangeInputs?: {sourceD34: number}};
  emptyGridCellsSaved: number; occupiedGridSeedsPending: number[];
  currentRoomRecord: number; currentRoomSeedSamples: {gridIndex: number;
    levelSeeds: [number, number]; savedSeeds: [number, number]; occupancy: "empty" | "occupied" | "padding";
    record: GridRecord | null}[];
  doorCellRecords: {gridIndex: number; record: GridRecord | null}[];
  itemPools: {poolSeed: number; rootRngSeed: number; secondaryRngSeed?: number | null;
    specialFactor: number; specialCounters: number[]; removedSamples: {itemId: number; removed: boolean}[];
    pillEffects: number[]; weightSamples: {poolId: number; itemId: number; weight: number}[]};
}
export interface SoundEvent {
  sequence: number; id: number; sampleIndex: number; systemCounter: number; halfTick: number;
  looping: boolean; pitch: number; gain: number;
}
export interface VolumeCallback {
  sequence: number; beforeSoundSequence: number; volume: number;
  loadedSlotVolumes: Record<string, number>;
}
export interface StopLoopingCallback {
  sequence: number; beforeSoundSequence: number; beforeVolumeSequence: number;
  loadedSlots: number[];
}
export interface AudioState {
  sfxVolume: number; nextSequence: number; events: SoundEvent[];
  nextVolumeSequence: number; volumeCallbacks: VolumeCallback[];
  nextStopSequence: number; stopLoopingCallbacks: StopLoopingCallback[];
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
  animation: string; animationFrame: number; animationPlaying: boolean; x: number; y: number; playerContactCountdown?: number;
  shot?: {actorRng: ItemRng; spawnerType: number; spawnerVariant: number; subtype: number; parent: ActorRef | null;
    attackFlags: [number,number]; sourceColor: AttackColor; currentColor: AttackColor; explosionDamage: number;
    gridCollisionClass: number; gridCollided: boolean; shotMarker: boolean; playerContactCountdown: number; ordinal7ec: number | null};
  motion: {
    position: { x: number; y: number }; presentedPosition: { x: number; y: number };
    velocity: { x: number; y: number }; fuseUpdatesRemaining: number; initialFuseUpdates?: number;
    fullUpdates: number; animationFrame: number; lifetime: "alive" | "exploding" | "removed";
  };
}
export interface Explosion { id: number; x: number; y: number; animationFrame: number; sourceEffect?: number; }
export interface Pickup {
  sourceSpawner?: {entityType: number; variant: number} | null;
  pocketContactDelay?: number | null;
  shopPrice?: {value: number; slot: number; purchaseFrontier?: string; sprite: {actor: string; animation: string; animationFrame: number; animationPlaying: boolean; layerFrames: (number | null)[]} | null};
  id: number; spawnSeed: number; variant: number; subtype: number; position: { x: number; y: number };
  x: number; y: number; actor: string; animation: string; animationFrame: number; animationPlaying: boolean;
  canCollect: boolean; collected: boolean; removed: boolean; collisionRadius: number; shadowSize: number; collectionDelay: number;
  overlay: string | null; overlayFrame: number; overlayPlaying: boolean;
  age: number; birthGameUpdate: number; charge: number; touched: boolean; saveByteAbd?: number;
  chest: {rng: ItemRng; gridCollisionClass: number; entityCollisionClass: number; price: number; cooldown: number; openFrontier?: 'playerContext' | 'collectibleMorph' | 'explosionGeometry' | 'regularChildInit';
    lootBirths: {variant: number; spawnSeed: number; position: {x: number; y: number}; velocity: {x: number; y: number}; spriteScaleMultiplier: number | null}[]} | null;
  morphLinks?: {parent498: ActorRef | null; link4a0: ActorRef | null; link4a8: ActorRef | null};
  regularCollision: {gridCollisionClass: number; entityCollisionClass: number} | null;
  regularRng: ItemRng | null;
  spritesheetItemId: number;
  presentedPosition: {x: number; y: number}; anchorPosition: {x: number; y: number};
  velocity: {x: number; y: number}; configuredFriction: number; currentFriction: number; collisionMass: number;
}
export interface SpriteColorValue { tint: number[]; colorize: number[]; offset: number[]; }
export interface RunIntroduction { age: number; active: boolean; title: string; subtitle?: string; stage: boolean; }
export interface BossBar { fill: number | null; flash: number; maximum: number; }
export interface PauseMenu {
  phase: "hidden" | "appearing" | "idle" | "disappearing";
  age: number; selection: number; statFrames: [number | null, number | null, number | null, number | null, number | null, number | null];
}
export interface RoomAppearance {
  backdrop: string; backdropId: number; columns: number; rows: number; tileSize: number; origin: { x: number; y: number };
  fxRays?: { index: number; xMin: number; xMax: number; topParallax: number; bottomParallax: number; rayLength: number;
    raySpacing: number; perspective: number; rays: { opacity: number; animationSpeed: number; phase: number; texture: string }[] };
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
  shop?: {categories: number[] | null; cursor: number; saleSlot: number;
    unadmitted: {spawnSeed: number; slot: number; variant: number; subtype: number; reason: string}[]};
  /** Owned map: absent cells outside both sets are empty; missing restore history is unknown. */
  persistentSpawnCells?: {present: number[]; unknown: number[]} | null;
}
export interface Floor {
  challengeCurse: {initialLocalState: number; finalLocalState: number; threshold: number; thresholdResult: number | null; selectionResult: number | null; selectedMask: number; filteredMask: number; roomAdjustmentResult: number; localDraws: number} | null;
  contract: string; stage: number; stageSeed: number; additionalRooms: number; attempts: number;
  /** Level+8 curse mask; absent when zero. */
  curses?: number;
  descriptors: RoomDescriptor[]; grid: number[]; startingDescriptor: number;
  primaryState: number; primaryDraws: number; generatorState: number; gameFlags: number;
  specialWeights: number[]; basementWeights: number[];
  attemptReports: { attempt: number; initialGeneratorState: number; finalGeneratorState: number;
    initialPrimaryState: number; finalPrimaryState: number; outcome: string }[];
}
/** Room ambience loops 5c6980: eased volume, pitch and whether the loop plays. */
export interface AmbienceLoop { volume: number; pitch: number; playing: boolean; }
export interface Ambience { fire: AmbienceLoop; swarm: AmbienceLoop; updateSequence: number; }
export interface Room {
  left: number; right: number; top: number; bottom: number; appearance: RoomAppearance;
  descriptor: RoomDescriptor | null;
  initializationGameUpdate: number | null; loaded: boolean; sourceTearSlowdown2494: number | null;
  pathCosts: number[] | null;
  trapdoor: {gridIndex: number; state: number; spawnSeed: number; clip: "closed" | "openAnimation"; frame: number; playing: boolean} | null;
  spawns: {tintedRockIndex: number; finalSpawnState: number; selectors: {groupIndex: number; gridIndex: number; category: "entity" | "grid"; type: number; variant: number; subtype: number; position: {x: number; y: number}; groupSeed: number; spawnSeed: number}[]} | null;
  pickups: Pickup[];
  backgroundStains: {sourceEffectId: number; spawnSeed: number; birthGameUpdate: number;
    position: {x: number; y: number}; variant: number; renderOffset: {x: number; y: number}; layerRotation: number | null; sprite: ActorSprite; footprint?: boolean}[];
  savedRegularPickups: {handle: number; variant: number; subtype: number; position: {x: number; y: number};
    spawnSeed: number; rngSeed: number; charge: number; touched: boolean; saveByteAbd?: number; shopPrice?: {value: number; slot: number}}[];
  npcs: { id: number; kind: number; variant: number; subtype: number; spawnSeed: number;
    position: {x: number; y: number}; presentedPosition: {x: number; y: number}; velocity: {x: number; y: number};
    hp: number; maxHp: number; collisionRadius: number; collisionMass: number; collisionDamage: number;
    baseFriction: number; friction: number;
    gridCollisionClass: number; entityCollisionClass: number; canShutDoors: boolean; boss: boolean;
    state: number; frameAge: number; animationFrame: number; animation: string; animationClock: number; animationPlaying: boolean; visible: boolean; savedInitializedState: number; shadowSize: number; dead: boolean; removed: boolean; raw494: boolean; spriteScale?: number; overlay?: { animation: string; frame: number; clock: number; playing: boolean }; wallCreep?: { rotation: number; flipY: boolean }; mom?: { part: number; rotation: number };
    damage: {pendingDamage: number; pendingFlags: number};
    keeper: {anchor: {x: number; y: number}; entityFlags: number; stateFrame: number; countdown: number;
      animation: string; animationClock: number; animationPlaying: boolean; tint: number[]; offsets: number[]} | null;
    actor?: string;
    champion?: {color: number; scale: number; speed: number; timer: number | null};
    spriteColor: {current: SpriteColorValue; stack: {priority: number; color: SpriteColorValue; duration: number; remaining: number}[]};
    status: {currentFriction: number | null; freezeTimer: number; fearTimer: number; applicationLockout: number; frozen: boolean; feared: boolean; baseUpdateAdmitted: boolean;
      frontier: "frozenDeath445c60" | "unsupportedActor" | null};
    parent?: number; child?: number;
    shooter?: {cooldown: number; delay: number};
    flipX?: boolean;
    dip: {entityFlags: number; birthGameUpdate: number; currentFriction: number; flipX: boolean;
      tint: number[]; offsets: number[]; renderOffset: {x: number; y: number}; statusCountdown: number} | null;
    level2Fly: {entityFlags: number; birthGameUpdate: number; feedbackVelocity: {x: number; y: number};
      gridContact: boolean; gridNormal: {x: number; y: number}; previousGridVelocity: {x: number; y: number}} | null;
    monstro: {stateFrame: number; choice: number; entityFlags: number; flipX: boolean; birthGameUpdate: number;
      projectileBirthCount: number; lastProjectileBatch: {count: number; target: {x: number; y: number}; parameterAngle: number} | null;
      deathBloodStart: boolean; deathExplosion: boolean; finalBossCallbackPending: boolean} | null;
    fireplace: {anchor: {x: number; y: number}; authoredGridIndex: number; entityFlags: number; countdown: number;
      actorRngState: number; renderOrderOffset: number; flameScale: number; flameRenderMode: number;
      particleBirthCount: number; lastParticleBirth: {parentId: number; spawnSeed: number; position: {x: number; y: number}; velocity: {x: number; y: number}} | null;
      lootBirth: {parentId: number; variant: number; spawnSeed: number; position: {x: number; y: number}} | null} | null }[];
  slots: Slot[];
  authoredActorOrder: {kind: "npc" | "pickup" | "slot"; id: number}[];
  managerCounts: { countablePending: number; countableCurrent: number; bossPending: number; bossCurrent: number };
  ordinaryPlacement?: {frontier: OrdinaryPlacementFrontier | null} | null;
  bossCount: number; bossPresentationIndex: number;
  combatClear: boolean; combatClearCountdown: number; combatShutDoorSlots: number[]; combatShutUpdate?: number;
  grid: { width: number; height: number; cells: {selector: number; entity: {kind: number; variant: number; state: number; collisionClass: number; spawnSeed: number; sourceOwner?: {birthGameUpdate: number | null; poopSheet?: number; tnt?: {countdown: number; sprite: {animation: "Idle" | "IdleMedium" | "ReadyToExplode" | "Blown"; frame: number; playing: boolean}; rewardFrontier?: {nextTrinketPoolQuery: number}}}} | null}[] } | null;
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
export type MenuPage = "title" | "save" | "game" | "options" | "controls" | "keyboard" | "character" | "challenge";
export interface MenuClip { animation: string; frame: number; playing: boolean }
/** MenuManager view, Back/Select widgets, File select and Character carousel. */
export interface MenuPresentation {
  view: [number, number]; moving: boolean; page: MenuPage; seedView: boolean; challengeScroll: number; keyboardScroll: number; optionsScroll: number;
  widgetsShown: boolean; widgets: [number, number];
  save: { file: number; menu: MenuClip; drawings: [MenuClip, MenuClip, MenuClip] };
  carousel: { angle: number; portraits: [number, number][] };
  blood: number; hardOverlay: number; hardWidget: boolean;
}
export interface State {
  saveRevision: number; saveReadmissionPending: boolean; saveWritePending: boolean; savedLoadValid: boolean | null;
  knifeQueryPartition?: { gameCounter: number; actors: ActorRef[]; buckets: number[][] };
  weaponCache: {primary: number; flagsC98: [number,number]; colorCa0: AttackColor; colorCcc: AttackColor};
  attackRuntime: {chargeC0c: number | null; aimRotationC70?: number | null; headDirectionDa0: number | null; shootingAimDb4: {x: number; y: number} | null; movementDirectionD9c: number | null; playerDc4: {x: number; y: number} | null; persistent: ActorRef | null; secondary: (ActorRef | null)[]; technology2C40?: {pointerKnown: boolean; actor: ActorRef | null; firstDiagnostic: string | null}; nextKnife: number};
  knives: Knife[];
  stageShootingGuard: {state: number; counter: number};
  weaponAdmission: {sourcePlayerType: number|null; c18IsKnownEmpty: boolean; c18ControllerHistoryKnown?: boolean; pendingPrimaryCleanup: {fromPrimary:number;toPrimary:number}|null; unadmittedController: {primary:number;channel:string}|null};
  lasers: {id:number; origin:"npcShootAngle"|"playerCharged" | "playerControlled" | "playerTechnology" | "playerTechnology2"; uninitialized:boolean|null; entityType:number; variant:number; subtype:number;
    spawnSeed:number; parent:ActorRef|null; child:ActorRef|null; active:boolean; dead:boolean; visible:boolean;
    position:{x:number;y:number}; savedPosition:{x:number;y:number}; renderOffset:{x:number;y:number};
    parentOffset:{x:number;y:number}; angle:number; renderSortOffset:number; remainingUpdates:number;
    beamLength:number; lineEnabled:boolean; line:{points:{x:number;y:number}[]}; fast:boolean; sourceColor:AttackColor; currentColor:AttackColor; sprite:ActorSprite;
    impact: {sourceSubtype:number|null; spriteFlipX:boolean|null; raw814:number|null; id:number; parent:ActorRef|null; parentKnown:boolean; sourceColor:AttackColor|null; currentColor:AttackColor|null; position:{x:number;y:number}; positionOffset:{x:number;y:number}; scale:{x:number;y:number}; rotation:number; actor:string; animation:string; animationClock:number; removed:boolean}|null; actor:string}[];
  enemyProjectiles: {id:number; variant?:number; spawnSeed:number; position:{x:number;y:number}; presentedPosition:{x:number;y:number}; velocity:{x:number;y:number}; height:number; scale:number; animation:string|null; animationFrame:number; dead:boolean; removed:boolean}[];
  schema: 1; tickRate: 60; tick: number; menuAnimationTick: number; runTick: number; gameUpdateCount: number; screen: Screen;
  titleAnimationFrame: number; menuFade: { active: boolean; alpha: number; color: [number, number, number, number] };
  menuPresentation: MenuPresentation;
  runStartRequest: { inputPending: boolean; fading: boolean; fadeAlpha: number; systemPending: boolean; challengeId: number | null; loadingImageIndex?: number };
  runQuit: { inputPending: boolean; fadeAlpha: number; fadeRate: number; systemPending: boolean; saveEnabled: boolean | null; saveOutcome: "supportedRunCopied" | "deadRunCleared" | null };
  systemClock: { systemCounter: number; interpolationEnabled: boolean };
  runAdmission: { clock: { milliseconds: number; subMillisecondNanoseconds: number; nanosecondsPerStep: number };
    requestedTimeAnchorMs: number | null; initialFade: number; fadeRate: number;
    completionCallbacksPending: boolean };
  audio: AudioState;
  unlockPopups: boolean | null; unlockNotificationsLoading: boolean;
  unlockNotifications: {state: number; currentClip: string | null; frame: number; gfx: string | null; definitionLoaded: boolean} | null;
  music: MusicState;
  profile: number; challengeSelection: number | null; challengeUnlocked: boolean[]; challengeId: number; challengeRunParameters: {sourceR8: number; sourceR9: boolean} | null; mainSelection: number; optionsMenu: {selection: number; leftHeld: number; rightHeld: number; verticalHeld: number; nextUnavailableSequence: number; unavailableRequests: {sequence: number; row: number}[]; nextSaveSequence: number; saveRequests: {sequence: number; sfxVolume: string; musicVolume: string; fullscreen: boolean; mapOpacity: string; gamma: string}[]}; fullscreen: {requested: boolean; nextSequence: number; requests: {sequence: number; requested: boolean; origin: 'startup' | 'options'}[]}; characterSelection: number; character: string; difficulty: Difficulty; menuDifficulty: Difficulty;
  /** engine/src/difficulty.rs rows the renderer draws, from the run's difficulty. */
  difficultyPresentation: {hudIcon: number | null; pauseBlood: boolean};
  seedText: string; seedValid: boolean; runSeed: string; initialSeed: number; runSeedValue: number; bootEpochSeed: number; bootLifecycle: BootLifecycle;
  runSeeds: {startSeed: number; streamState: number; initializedFields: number[]; sourceEffectMode?: number | null} | null;
  acquisitionHistory?: AcquisitionHistory;
  itemPools: {seed: number; rootRng: ItemRng; secondaryRng?: ItemRng | null; pools: ItemPool[];
    specialFactor: number; removed: boolean[]; selected: boolean[]; roomRoutes: number[]; pillEffects: number[]; pillIdentified?: boolean[] | null;
    trinkets?: {available: boolean[]; count: number; eligible?: boolean[] | null} | null} | null;
  /** HUD 575FDF held Pill text: the name when identified, "???" when not; absent when unknown. */
  heldPillName?: string | null;
  playerItems: {initializerSeed: number; managerRng: ItemRng; initializerRng: ItemRng;
    itemRngs: ItemRng[]; trinketRngs: ItemRng[]; pocketRngs: ItemRng[]; entitySpawnSeed: number;
    heldPockets?: {slots: {id: number; card: boolean}[];
      frontier: {kind: "cardEffect"; cardId: number} | {kind: "pillEffect"; pillColor: number}
        | {kind: "modifierConversion"; collectibleId: number} | {kind: "dropAction"} | null} | null;
    ordinaryProducerHistory?: {frontier: OrdinaryPlacementFrontier | null} | null;
    pickupGrantHistory?: PickupGrantHistory | null;
    activeChargeHistory?: ActiveChargeHistory | null;
    technologyEyeRegistered: boolean | null;
    costume229?: Costume229 | null;
    costumeRegistrations?: {configurationIds: number[]} | null;
    ownedCounts: number[]; temporaryCollectibles: {itemId: number; count: number; cooldown: number}[]; queuedId: number | null; queuedConfigKind?: 1 | 2 | 3 | 4 | null; queuedCharge: number; queuedTouched: boolean;
    queryHistory?: { permanentCounts: boolean; temporaryRecords: boolean; temporarySourceCache?: {maskCf8: number | null; pendingD18: boolean | null} | null; temporaryUnknownInputs?: { configKind: number; itemId: number }[]; trinketSlots: boolean; primaryC08: boolean; damageHealthKnown?: boolean; cachedCallbacks?: number; temporaryEvaluation?: { possibleCacheMask: number | null; writerPcs: number[] }; unresolvedEffects?: Record<string, string> };
    pickupAnimationFrame: number; pickupAnimationPlaying: boolean; pickupAnimationGuard: boolean;
    pickupPresentation: {itemId: number; animationFrame: number; animationPlaying: boolean;
      regular?: {variant: number; subtype: number; actor: string; animation: string; animationFrame: number;
        animationPlaying: boolean; overlay: string | null; overlayFrame: number; overlayPlaying: boolean; attachedToPlayer: boolean};
      pocket?: {overlay: "Pills" | "CardFronts" | "Runes"; overlayFrame: number}} | null;
    collectionDelay: number; activeId: number; activeCharge: number; activeSubordinateCharge: number; saveWordD88: number;
    activeStateD80: number; activeStateD84: number; reserveCharge: number; activeFlag3076: boolean;
    unevaluatedCacheFlags: number;
    treatment240Frontiers?: {healthSourceUnresolved: boolean;
      achievementUnlocks: {achievementId: number}[]};
    damageInputs?: {sourceD30: number};
    fireDelayInputs?: {sourceD2c: number};
    rangeInputs?: {sourceD34: number};
    shotSpeedInputs?: {sourceD38: number};
    pillEffectHistory: {kind: "current"; version: 1; history: {words: (number | null)[]} | null};
    moveSpeedTerms?: {sourceD28: number; sourceD44: number};
    luckTerms: {sourceD3c: number; sourceD40: number; sourceD58: number} | null;
    moveSpeedInputs?: {sourceD28: number; sourceD44: number; source13bc: number; source13c0: number};
    luckInputs: {sourceD3c: number; sourceD40: number; sourceD58: number; source13cc: number; source13d0: number} | null} | null;
  informationAnimation: {state: number; delay: number; currentActor: string | null;
    currentClip: string | null; spritesheet: string | null; definitionLoaded: boolean; frame: number; playing: boolean};
  savedRunSummary: SavedRunSummary | null;
  pendingMenuInitializations: { afterAudioRequests: number; initialization: { type: "loadFreshProfile"; profile: number } | { type: "resetCharacterSeed" } | { type: "commitCharacterSeed"; seed: number } }[];
  stage: number; stageSeed: number; seededRun: boolean; achievementEligibility: boolean;
  levelProgress: {flags: number; heartCollected: boolean; lastPenaltyStage: number | null};
  nightmares: {order: number[]; pathIndex: number; currentActor: string | null; requestPending: boolean; systemActive: boolean;
    active: boolean; definitionLoaded: boolean; progressDefinitionLoaded: boolean; stageResources: number[];
    counter: number; frame: number; playing: boolean; stageIndex: number; alternate: boolean; progressFloors: number};
  profiles: { slot: number; unlockedCharacters: string[]; selectedCharacter: string; achievements: boolean[]; counters: number[] }[];
  characterCatalog: { name: string; animation: string }[];
  paused: boolean; pressed: string[];
  floor: Floor; currentRoom: number; visitedRooms: number[];
  roomDisplayFlags: number[]; revealedSecretConnections: [number, number][];
  persistedRooms: Record<string, Room>;
  stageTransition: {state: number; counter: number; sameStage: boolean; animation: number};
  ambience: Ambience;
  roomTransition: {fromRoom: number; targetRoom: number; fromSlot: number; targetSlot: number; phase: "out" | "in" | "bossScene" | "bossCleanup"; fullUpdates: number; bossScene: {presentationIndex: number; animationFrame: number; animationPlaying: boolean} | null} | null;
  departurePosition?: {x: number; y: number};
  sourceCacheRunFlags: number | null;
  sourceShotSpeedMinimum?: boolean;
  sourceDamageMaximum?: boolean;
  sourceFireDelayMinimum?: boolean;
  playerStats: {shotSpeed: number; damage: number; tearHeight: number; tearFallingSpeed: number; tearFallingAcceleration: number; moveSpeed: number; luck: number | null};
  player: Player; room: Room; stageEffects: StageEffects; inventory: { coins: number; bombs: number; keys: number };
  publicActions: { menuConfirm: boolean; menuBack: boolean; activePressed: boolean; activeTriggered: boolean; pocketPressed: boolean; pocketTriggered: boolean; held: boolean[]; triggered: boolean[]; rawKey: number | null; physicalWasDown: string[] };
  keyboard: {bindings: number[][]};
  selectedInputDevice?: number;
  inputConfigs: InputConfigs;
  controlsMenu: {deviceSelection: number; deviceHeld: number; selectedDevice: number | null; selection: number; column: number; waiting: boolean; nextSaveSequence: number; saveRequests: {sequence: number; configs: InputConfigs}[]; unavailableActions: number[]};
  playerAnimation: { current: "Trapdoor" | "Appear" | "Death" | "UseItem" | "Hit" | "Sad" | "Happy" | "Jump" | null; frame: number; playing: boolean; guard: boolean };

  gameShake?: {durationCe9fc: number | null; offsetCea00: {x: number; y: number} | null; frontier: string | null} | null;
  camera: {scroll: {x: number; y: number}; lastPlayerPosition: {x: number; y: number}; shakeHistoryKnown?: boolean | null;
    histories: {capacity: number; head: number; samples: {x: number; y: number}[]}[]} | null;
  nextPickup: number;
  mapSettings: MapPreferences;
  gammaSettings: GammaPreferences;
  minimapPresentation: {mode: number; expandedLatched: boolean; heldFrames: number; smallAlpha: number; expandedAlpha: number};
  minimap: { cells: { x: number; y: number; visibility: "unvisited" | "visited" | "current" }[] };
  tears: Tear[]; fireCooldown: number; nextTear: number;
  fireParticles: {id: number; parentNpc: number; parentType: number; parentVariant: number; spawnSeed: number; birthGameUpdate: number;
    position: {x: number; y: number}; presentedPosition: {x: number; y: number}; velocity: {x: number; y: number};
    hp: number; maxHp: number; collisionRadius: number; collisionMass: number; collisionDamage: number; baseFriction: number;
    gridCollisionClass: number; entityCollisionClass: number; visible: boolean;
    lifetimeDivisor: number; timeToLive: number; fullUpdates: number; animationClock: number; animationPlaying: boolean;
    dead: boolean; removed: boolean}[];
  familiars: Familiar[]; nextFamiliar: number; spriteEffects: SpriteEffect[]; nextEffect: number;
  maxFireDelay: number; headTimer: number; actorScheduler: { actors: ActorRef[]; identities: {actor: ActorRef; sequenceId: number}[]; roomNextSequence: number; render: ActorRef[] };
  bombs: Bomb[]; explosions: Explosion[]; nextBomb: number; damageCooldown: number;
  bombPlacementCooldown: { remainingUpdates: number }; bombActionWasDown: boolean;
  runIntroduction: RunIntroduction; bossBar: BossBar; chargeBarFlash: number; pauseMenu: PauseMenu;
  playerDeath: { lastDamageFlags: number; gameOver: { state: number; damageFlags: number; diaryFrame: number; diaryPlaying: boolean } | null; systemPending: "restart" | "exit" | null };
}
export interface InputConfigs { devices: {name: string; bindings: number[][]}[]; }
export interface AudioPreferences { sfxVolume: number; musicVolume: number; }
export interface FullscreenPreferences { fullscreen: boolean; }
export interface MapPreferences { opacity: number; }
export interface GammaPreferences { gamma: number; }
export interface ResourceInputs { challengesXml: string; specialRooms?: number[]; }
/** A controlled harness's declared time between the boot epoch and this reset. */
export interface BootElapsed { milliseconds: number; subMillisecondNanoseconds: number; }
export type Command = { type: "key"; code: string; down: boolean } | { type: "text"; text: string } | { type: "tick"; count: number; timestampMillis?: number } | { type: "audioVolume"; sfx_volume: number } | { type: 'musicVolume'; volume: number } | MusicBackendCommand;
interface Exports extends WebAssembly.Exports {
  reset_with_input_configs: (initialSeed: number, bootEpochSeed: number) => number;
  memory: WebAssembly.Memory; abi_version: () => number; reset: (initialSeed: number, bootEpochSeed: number) => number; request_ptr: (length: number) => number;
  reset_with_settings_preferences: (initialSeed: number, bootEpochSeed: number) => number;
  reset_with_audio_preferences: (initialSeed: number, bootEpochSeed: number, sfxVolume: number, musicVolume: number) => number;
  reset_with_fullscreen_preferences: (initialSeed: number, bootEpochSeed: number, fullscreen: number, hasAudio: number, sfxVolume: number, musicVolume: number) => number;
  reset_with_boot_resources_and_fullscreen_preferences: (initialSeed: number, bootEpochSeed: number, fullscreen: number, hasAudio: number, sfxVolume: number, musicVolume: number) => number;
  reset_with_boot_resources_and_audio_preferences: (initialSeed: number, bootEpochSeed: number, sfxVolume: number, musicVolume: number) => number;
  reset_with_map_preferences: (initialSeed: number, bootEpochSeed: number, fullscreen: number, hasAudio: number, sfxVolume: number, musicVolume: number, opacity: number) => number;
  reset_with_boot_resources_and_map_preferences: (initialSeed: number, bootEpochSeed: number, fullscreen: number, hasAudio: number, sfxVolume: number, musicVolume: number, opacity: number) => number;
  reset_with_map_preferences_v2: (initialSeed: number, bootEpochSeed: number, hasFullscreen: number, fullscreen: number, hasAudio: number, sfxVolume: number, musicVolume: number, opacity: number) => number;
  reset_with_boot_resources_and_map_preferences_v2: (initialSeed: number, bootEpochSeed: number, hasFullscreen: number, fullscreen: number, hasAudio: number, sfxVolume: number, musicVolume: number, opacity: number) => number;
  reset_with_resources: (initialSeed: number, bootEpochSeed: number) => number;
  reset_with_boot_resources: (initialSeed: number, bootEpochSeed: number) => number;
  boot_resource_ptr: (length: number) => number;
  save_input_ptr: (length: number) => number; load_save_data: () => number; save_data_ptr: () => number;
  rejected_profile_document: () => number;
  enable_saved_game_io: (protocol: number) => number;
  complete_saved_game_write: (low: number, high: number, succeeded: number) => number;
  complete_save_menu_readmission: (revisionLow: number, revisionHigh: number, loaded: number) => number;
  set_boot_elapsed: (milliseconds: number, subMillisecondNanoseconds: number) => number;
  command: () => number; snapshot_ptr: () => number; snapshot_len: () => number;
  frame_snapshot_ptr: () => number; background_stains_ptr: (from: number) => number;
  error_ptr: () => number; error_len: () => number; public_keys_ptr: () => number; public_keys_len: () => number;
  mt_draws_enable: (enabled: number) => void; mt_draws_ptr: () => number;
}
/** Present only in a `debug-sandbox` cargo build (npm run dev:sandbox). */
interface DebugSandboxExports {
  debug_sandbox_slots: () => number; debug_sandbox_clear: () => number;
  debug_sandbox_place: (subtype: number, slot: number) => number;
  debug_sandbox_enemy: (entityType: number, variant: number) => number;
}
/** The port-only debug sandbox floor's engine controls (web/debug-sandbox.ts). */
export interface DebugSandbox {
  slots(): number; clear(): void; place(collectible: number, slot: number): void; enemy(entityType: number, variant: number): void;
}
/** One shared-MT draw from the opt-in diagnostic log (e2e `--mt-draws`). */
export interface MtDraw { counter: number; index: number; kind: "B" | "U" | "R"; site: string; }
/** An actual atomic run-confirmation refusal, not a failed simulation Tick. */
export class SavedProfileAdmissionRefused extends Error {
  constructor(message: string) { super(message); this.name = 'SavedProfileAdmissionRefused'; }
}
const REJECTED_PROFILE_ADMISSION = 'unknown rejected own-profile document must be recovered or explicitly reset';

export class Engine {
  /** The engine-owned key surface: the only codes `key` commands accept. */
  readonly publicKeys: ReadonlySet<string>;
  private readonly wasm: Exports;
  private constructor(wasm: Exports) {
    this.wasm = wasm;
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
  /** `wasm` is the engine module's bytes: the page fetches them (main.ts),
   * the headless e2e page reads the production snapshot's file. */
  static async create(wasm: BufferSource, initialSeed: number, bootEpochSeed = 0, resources?: ResourceInputs, audioPreferences?: AudioPreferences, fullscreenPreferences?: FullscreenPreferences, inputConfigs?: InputConfigs, mapPreferences?: MapPreferences, gammaPreferences?: GammaPreferences) {
    Engine.validateSeed(initialSeed);
    Engine.validateBootSeed(bootEpochSeed);
    const { instance } = await WebAssembly.instantiate(wasm);
    if (typeof instance.exports.abi_version !== "function" || instance.exports.abi_version() !== 2
      || typeof instance.exports.public_keys_ptr !== "function" || typeof instance.exports.public_keys_len !== "function"
      || typeof instance.exports.frame_snapshot_ptr !== "function" || typeof instance.exports.background_stains_ptr !== "function"
      || typeof instance.exports.rejected_profile_document !== "function")
      throw new Error("WASM engine ABI is stale; rebuild with npm run build:wasm");
    const engine = new Engine(instance.exports as Exports); engine.reset(initialSeed, bootEpochSeed, resources, audioPreferences, fullscreenPreferences, inputConfigs, mapPreferences, gammaPreferences); return engine;
  }
  private static validateSeed(initialSeed: number) {
    if (!Number.isInteger(initialSeed) || initialSeed < 1 || initialSeed > 0xffffffff)
      throw new Error("Initial seed must be a nonzero unsigned 32-bit entropy value");
  }
  private static validateBootSeed(bootEpochSeed: number) {
    if (!Number.isInteger(bootEpochSeed) || bootEpochSeed < 0 || bootEpochSeed > 0xffffffff)
      throw new Error("Boot epoch seed must be an unsigned 32-bit value");
  }
  reset(initialSeed: number, bootEpochSeed = 0, resources?: ResourceInputs, audioPreferences?: AudioPreferences, fullscreenPreferences?: FullscreenPreferences, inputConfigs?: InputConfigs, mapPreferences?: MapPreferences, gammaPreferences?: GammaPreferences, bootElapsed?: BootElapsed): State {
    Engine.validateSeed(initialSeed);
    Engine.validateBootSeed(bootEpochSeed);
    if (audioPreferences !== undefined && (!audioPreferences ||
      Object.keys(audioPreferences).sort().join(',') !== 'musicVolume,sfxVolume' ||
      ![audioPreferences.sfxVolume, audioPreferences.musicVolume].every(value => Number.isFinite(value) && value >= 0 && value <= 1)))
      throw new Error('Initial audio preferences must contain finite SFX/Music volumes within 0..1');
    if (audioPreferences !== undefined && (typeof this.wasm.reset_with_audio_preferences !== 'function' ||
      typeof this.wasm.reset_with_boot_resources_and_audio_preferences !== 'function'))
      throw new Error('WASM audio preference ingress is stale; rebuild with npm run build:wasm');
    if (fullscreenPreferences !== undefined && (!fullscreenPreferences || typeof fullscreenPreferences !== 'object' || Array.isArray(fullscreenPreferences) ||
      Object.keys(fullscreenPreferences).join(',') !== 'fullscreen' || typeof fullscreenPreferences.fullscreen !== 'boolean'))
      throw new Error('Initial fullscreen preference requires exactly one boolean');
    if (typeof this.wasm.reset_with_fullscreen_preferences !== 'function' ||
      typeof this.wasm.reset_with_boot_resources_and_fullscreen_preferences !== 'function')
      throw new Error('WASM fullscreen preference owner is stale; rebuild with npm run build:wasm');
    if (mapPreferences !== undefined && (!mapPreferences || typeof mapPreferences !== 'object' || Array.isArray(mapPreferences) ||
      Object.keys(mapPreferences).join(',') !== 'opacity' || !Number.isFinite(mapPreferences.opacity) || mapPreferences.opacity<0 || mapPreferences.opacity>1))
      throw new Error('Initial map preference requires exactly one finite opacity within 0..1');
    if (mapPreferences !== undefined && (typeof this.wasm.reset_with_map_preferences_v2 !== 'function' ||
      typeof this.wasm.reset_with_boot_resources_and_map_preferences_v2 !== 'function'))
      throw new Error('WASM map preference owner is stale; rebuild with npm run build:wasm');
    if (gammaPreferences !== undefined && (!gammaPreferences || typeof gammaPreferences !== 'object' || Array.isArray(gammaPreferences) ||
      Object.keys(gammaPreferences).join(',') !== 'gamma' || !Number.isFinite(gammaPreferences.gamma) ||
      gammaPreferences.gamma < 0.5 || gammaPreferences.gamma > 1.5))
      throw new Error('Initial Gamma requires exactly one finite value within .5..1.5');
    if (gammaPreferences !== undefined && typeof this.wasm.reset_with_settings_preferences !== 'function')
      throw new Error('WASM Gamma preference ingress is stale; rebuild');
    const mapReset = (withResources: boolean) => {
      const reset = withResources ? this.wasm.reset_with_boot_resources_and_map_preferences_v2 : this.wasm.reset_with_map_preferences_v2;
      return reset(initialSeed, bootEpochSeed, Number(fullscreenPreferences !== undefined),
        Number(fullscreenPreferences?.fullscreen ?? false), Number(audioPreferences !== undefined),
        audioPreferences?.sfxVolume ?? 0, audioPreferences?.musicVolume ?? 0, mapPreferences!.opacity);
    };
    const displayReset = (withResources: boolean) => {
      const reset = withResources ? this.wasm.reset_with_boot_resources_and_fullscreen_preferences : this.wasm.reset_with_fullscreen_preferences;
      return reset(initialSeed, bootEpochSeed, Number(fullscreenPreferences!.fullscreen), Number(audioPreferences !== undefined),
        audioPreferences?.sfxVolume ?? 0, audioPreferences?.musicVolume ?? 0);
    };
    let result: number;
    if (inputConfigs !== undefined || gammaPreferences !== undefined) {
      if (typeof this.wasm.reset_with_input_configs !== 'function') throw new Error('WASM keyboard boot ingress is stale; rebuild');
      if (resources !== undefined) {
        const resourceBytes = new TextEncoder().encode(JSON.stringify(resources));
        if (resourceBytes.length > 512*1024) throw new Error('Boot resource input exceeds512KiB');
        const resourcePointer = this.wasm.boot_resource_ptr(resourceBytes.length);
        if (resourcePointer === 0) throw new Error('WASM resource allocation failed');
        new Uint8Array(this.wasm.memory.buffer,resourcePointer,resourceBytes.length).set(resourceBytes);
      }
      const bytes = new TextEncoder().encode(JSON.stringify({withResources:resources !== undefined, audioPreferences, fullscreenPreferences, inputConfigs, mapPreferences, gammaPreferences}));
      if (bytes.length > 65536) throw new Error('Keyboard startup inputs exceed64KiB');
      const pointer = this.wasm.request_ptr(bytes.length);
      if (pointer === 0) throw new Error('Keyboard startup input allocation failed');
      new Uint8Array(this.wasm.memory.buffer,pointer,bytes.length).set(bytes);
      result = gammaPreferences === undefined
        ? this.wasm.reset_with_input_configs(initialSeed,bootEpochSeed)
        : this.wasm.reset_with_settings_preferences(initialSeed,bootEpochSeed);
    } else if (resources === undefined) result = mapPreferences !== undefined ? mapReset(false)
      : fullscreenPreferences !== undefined ? displayReset(false)
      : audioPreferences === undefined ? this.wasm.reset(initialSeed, bootEpochSeed)
      : this.wasm.reset_with_audio_preferences(initialSeed, bootEpochSeed, audioPreferences.sfxVolume, audioPreferences.musicVolume);
    else {
      if (typeof this.wasm.reset_with_boot_resources !== "function" || typeof this.wasm.boot_resource_ptr !== "function")
        throw new Error("WASM resource ingress is stale; rebuild with npm run build:wasm");
      const bytes = new TextEncoder().encode(JSON.stringify(resources));
      if (bytes.length > 512 * 1024) throw new Error("Boot resource input exceeds 512 KiB");
      const ptr = this.wasm.boot_resource_ptr(bytes.length);
      if (ptr === 0) throw new Error("WASM resource allocation failed");
      new Uint8Array(this.wasm.memory.buffer, ptr, bytes.length).set(bytes);
      result = mapPreferences !== undefined ? mapReset(true)
        : fullscreenPreferences !== undefined ? displayReset(true)
        : audioPreferences === undefined ? this.wasm.reset_with_boot_resources(initialSeed, bootEpochSeed)
        : this.wasm.reset_with_boot_resources_and_audio_preferences(initialSeed, bootEpochSeed, audioPreferences.sfxVolume, audioPreferences.musicVolume);
    }
    if (result !== 0) throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()));
    if (bootElapsed !== undefined) {
      const { milliseconds, subMillisecondNanoseconds } = bootElapsed;
      if (![milliseconds, subMillisecondNanoseconds].every(value => Number.isInteger(value) && value >= 0 && value <= 0xffffffff))
        throw new Error("Boot elapsed time must be unsigned 32-bit milliseconds and nanoseconds");
      if (typeof this.wasm.set_boot_elapsed !== "function") throw new Error("WASM boot clock ingress is stale; rebuild with npm run build:wasm");
      if (this.wasm.set_boot_elapsed(milliseconds, subMillisecondNanoseconds) !== 0)
        throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()));
    }
    this.stains = { room: null, initialization: null, list: [] };
    return this.snapshot();
  }
  command(command: Command): State {
    if (command.type === "tick" && (!Number.isSafeInteger(command.count) || command.count < 0)) throw new Error("Tick count must be an unsigned integer");
    const text = JSON.stringify(command); if (text === undefined) throw new Error("Command must be JSON");
    const bytes = new TextEncoder().encode(text);
    if (bytes.length > 65_536) throw new Error("Command exceeds 64 KiB");
    const ptr = this.wasm.request_ptr(bytes.length); if (ptr === 0) throw new Error("WASM request allocation failed");
    new Uint8Array(this.wasm.memory.buffer, ptr, bytes.length).set(bytes);
    if (this.wasm.command() !== 0) {
      const message = this.read(this.wasm.error_ptr(), this.wasm.error_len());
      if (message === REJECTED_PROFILE_ADMISSION && command.type === 'key'
        && command.code === 'Enter' && command.down && this.wasm.rejected_profile_document() === 1) {
        const state = this.snapshot();
        if (!state.pressed.includes('Enter') && (state.screen === 'character' || state.screen === 'challenge'
          || state.screen === 'main' && state.mainSelection === 1))
          throw new SavedProfileAdmissionRefused(message);
      }
      throw new Error(message);
    }
    return this.snapshot();
  }
  importSaveData(text: string): void {
    const bytes = new TextEncoder().encode(text);
    if (bytes.length > 4 * 1024 * 1024) throw new Error('Saved profiles exceed the supported 4 MiB format');
    if (typeof this.wasm.save_input_ptr !== 'function' || typeof this.wasm.load_save_data !== 'function')
      throw new Error('WASM save transport is stale; rebuild the engine');
    const ptr = this.wasm.save_input_ptr(bytes.length);
    if (!ptr) throw new Error('WASM save input allocation failed');
    new Uint8Array(this.wasm.memory.buffer, ptr, bytes.length).set(bytes);
    if (this.wasm.load_save_data() !== 0) throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()));
  }
  enableSaveIO(): void {
    if (typeof this.wasm.enable_saved_game_io !== 'function' || this.wasm.enable_saved_game_io(1) !== 0)
      throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()) || 'Stale Save IO ABI');
  }
  completeSaveWrite(revision: number, succeeded: boolean): void {
    if (!Number.isSafeInteger(revision) || revision < 0) throw new Error('Invalid save write revision');
    if (typeof this.wasm.complete_saved_game_write !== 'function' ||
      this.wasm.complete_saved_game_write(revision >>> 0, Math.floor(revision / 0x100000000), succeeded ? 1 : 0) !== 0)
      throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()) || 'Stale Save IO ABI');
  }
  completeMenuReadmission(revision: number, text: string | null): boolean {
    if (!Number.isSafeInteger(revision) || revision < 0) throw new Error('Invalid menu IO request revision');
    const bytes = new TextEncoder().encode(text ?? '');
    if (bytes.length > 4 * 1024 * 1024) throw new Error('Saved profiles exceed4MiB');
    if (text !== null) {
      const ptr = this.wasm.save_input_ptr(bytes.length);
      if (!ptr) throw new Error('WASM save allocation failed');
      new Uint8Array(this.wasm.memory.buffer, ptr, bytes.length).set(bytes);
    }
    if (typeof this.wasm.complete_save_menu_readmission !== 'function') throw new Error('Stale menu IO ABI');
    const result = this.wasm.complete_save_menu_readmission(revision >>> 0, Math.floor(revision / 0x100000000), text === null ? 0 : 1);
    if (result !== 0 && result !== 2) throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()));
    return result === 0;
  }
  exportSaveData(): string {
    if (typeof this.wasm.save_data_ptr !== 'function') throw new Error('WASM save transport is stale; rebuild the engine');
    const ptr = this.wasm.save_data_ptr();
    if (!ptr) throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len()));
    return this.read(ptr, this.wasm.snapshot_len());
  }
  key(code: string, down: boolean): State { return this.command({ type: "key", code, down }); }
  text(text: string): State { return this.command({ type: "text", text }); }
  tick(count = 1): State { return this.command({ type: "tick", count }); }
  /** Per-tick state. Background stains are append-only within a room, so the
   * engine elides them from this snapshot and only new ones cross the
   * boundary; each snapshot still carries the room's complete list. */
  /** Null unless the WASM was built with the `debug-sandbox` cargo feature. */
  debugSandbox(): DebugSandbox | null {
    if (!__DEBUG_SANDBOX__) return null;
    const { debug_sandbox_slots: slots, debug_sandbox_clear: clear, debug_sandbox_place: place, debug_sandbox_enemy: enemy } =
      this.wasm as unknown as Partial<DebugSandboxExports>;
    if (!slots || !clear || !place || !enemy) return null;
    const check = (result: number) => { if (result !== 0) throw new Error(this.read(this.wasm.error_ptr(), this.wasm.error_len())); };
    return {
      slots: () => slots(), clear: () => check(clear()),
      place: (collectible, slot) => check(place(collectible, slot)),
      enemy: (entityType, variant) => check(enemy(entityType, variant)),
    };
  }
  snapshot(): State {
    const state = JSON.parse(this.read(this.wasm.frame_snapshot_ptr(), this.wasm.snapshot_len())) as State | null;
    if (state === null) return state as unknown as State;
    const cache = this.stains;
    let delta = this.stainsSince(cache.list.length);
    if (delta.room !== cache.room || delta.initialization !== cache.initialization || delta.total < cache.list.length) {
      delta = this.stainsSince(0);
      this.stains = { room: delta.room, initialization: delta.initialization, list: delta.stains };
    } else if (delta.stains.length) this.stains = { ...cache, list: cache.list.concat(delta.stains) };
    if (this.stains.list.length !== delta.total) throw new Error("Background stain channel lost entries");
    state.room.backgroundStains = this.stains.list;
    return state;
  }
  /** The complete engine state, stains included, as the engine's JSON text. */
  snapshotJson(): string { const ptr = this.wasm.snapshot_ptr(); return this.read(ptr, this.wasm.snapshot_len()); }
  /** Diagnostic only: turn the shared-MT draw log on (cleared) or off. */
  setMtDrawLog(enabled: boolean): void { this.wasm.mt_draws_enable(enabled ? 1 : 0); }
  /** Diagnostic only: the draws logged since the previous call. */
  takeMtDraws(): MtDraw[] { const ptr = this.wasm.mt_draws_ptr(); return JSON.parse(this.read(ptr, this.wasm.snapshot_len())) as MtDraw[]; }
  private stains: { room: number | null; initialization: number | null; list: Room["backgroundStains"] } =
    { room: null, initialization: null, list: [] };
  private stainsSince(from: number): { room: number; initialization: number | null; total: number; stains: Room["backgroundStains"] } {
    return JSON.parse(this.read(this.wasm.background_stains_ptr(from), this.wasm.snapshot_len()));
  }
  private read(ptr: number, length: number) { return new TextDecoder().decode(new Uint8Array(this.wasm.memory.buffer, ptr, length)); }
}
