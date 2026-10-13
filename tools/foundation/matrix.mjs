import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { CHECKPOINTS } from './verify.mjs';
import { TASKS } from './systems.mjs';
import { readStageMaskCases } from './stage-masks.mjs';
import { discoverNodeCases, discoverTestCases, discoverOriginalClockScenarios, validateOriginalClockScenarios, ORIGINAL_CLOCK_SOURCE } from './scenario-index.mjs';
const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const target = resolve(root, 'TEST_MATRIX.md');
const labels = {
  title: 'Boot to title', profile: 'Choose a fresh profile', main: 'New Run menu',
  character: 'Isaac, normal mode', 'seed-entry': 'Enter observed T1PK 6Y0W seed',
  'character-seeded': 'Return to character screen with committed seed',
  'gameplay-start': 'Enter playable tutorial room', movement: 'Hold D, release, settle',
  shooting: 'Hold Right, release, capture live tear', 'after-fire': 'Release fire and expire tears',
};
// These are evidence/implementation contracts, not cached pass results. The
// executable tables below still discover every selected scenario from source.
// Update a row when new original evidence or a runtime contract lands.
export const SUBSYSTEMS = [
  ['runtime-abi', 'Original Linux runtime and independent native fixtures exist; global MT startup uses an independent epoch input.', 'Rust JSON commands/state, ABI 2 with separate bootEpochSeed, browser trace and native replay; invalid resets reject atomically.', 'Native unit, independent boot-input/replay and actual-key browser cases.', 'Shared MT lifecycle/consumer routing, original complete-state continuation and simulation-tick identity remain unverified.'],
  ['menus', 'Independent title/profile/main/character/seed frames.', 'State-driven menu path, seed dialog, profile selection and authored animation.', 'Final immutable-production gate matches five complete menu frames plus command replay; title and four gameplay checkpoints still fail.', 'Title pixels, settings, challenges, continue and broader navigation observations.'],
  ['seed-codec', 'Original fresh random-seed acceptance and Stage 1 logs.', 'General checksum codec, crypto boot entropy and Stage 1 startup stream.', 'Boundary/round-trip tests, random original/Rust/WASM seed contracts.', 'Special seed effects and all later stream routing.'],
  ['input-clock', 'Consumed native X11 events, repeatable frames and concrete full/fast source phases: players update both, nonplayers on full phases; run entry normalizes odd phase parity.', 'Keyboard/text commands, releases, pause/catch-up and normalized systemClock; retained title animation/menu fade consume the clock, while gameplay still uses its legacy update path.', 'Original clock repeats, native/WASM tapes, atomic boot-reset and actual DOM key cases.', 'Original phase delivery and full gameplay mixed-rate integration, complete state, controller, mouse and remapping contracts.'],
  ['run-introduction', 'Fresh entry/pause captures and an independent arbitrary-seed native/browser probe match exact banner text, paper/stat/cursor and seed regions.', 'Run-introduction and pause-menu phase/age/selection state; authored banner/paper/seed/stat composition.', 'Reusable native presentation scenario and two exact-region browser cases; complete loading/fade/frame parity remains open.', 'Full transition pixel/timing parity, input enablement holdouts and unsupported pause menu actions.'],
  ['authored-animation', 'Independent original GL submissions provide 75 title quads and 42 smoke quads; ten title/fade/reentry source checkpoints repeat exactly.', 'Generic sequential f32 interpolation/transforms and source floorf cached-frame selection; retained title/f32 fade uses normalized System phases.', 'Six math Node contracts are declared. Prior immutable ABI 2 artifact matches seven complete title frames at zero error and all ten scoped title/fade states; fresh rebuild remains required after changes.', 'Legacy title whole-frame still differs in 10,538 pixels; menu transition rendering, complete smoke/sprite pixels, arbitrary-angle library edges and other authored consumers.'],
  ['locomotion', 'Native source/trajectory and four wall probes establish motion/reversal laws and authored radius-clearance bounds.', 'Previous-velocity integration, projection, release/turn/reversal behavior, and player walls derived from floor/radius metadata.', 'Native/WASM invariants and held-out native trajectories; original pixel gates remain strict.', 'Original simulation cadence, full typed state, collision velocity response and complete actor pixels.'],
  ['tears', 'Authored metadata and fresh stationary/moving/coasting native traces establish velocity inheritance and emission order.', 'Four-direction regular tears, inherited motion and corrected origins/order; provisional projectile bounds are now separate from measured player walls.', 'Native/WASM shot invariants and independent original projectile pixels.', 'Ground/height/gravity separation, range-end drop, initial RNG/eye routing, fixed 45-step expiry, provisional projectile bounds, damage and other weapons.'],
  ['room-assets', 'All 13 packed STB stage sets decode and re-encode byte-identically: 5,314 rooms and 206,690 weighted entries; 790 authored actors have known names.', 'Complete production Special Rooms/Basement STB decoder retains authored doors and every spawn selector; generated descriptors resolve original catalogue entries.', 'Full floor comparisons exercise authored template identities; existing structural asset tooling retains its narrower contract.', 'Typed spawn selection, placeholder resolution, grids and entity admission.'],
  ['backdrop-decoration', 'Four independent random-seed originals match all four floor and 44 wall submitted quads when the observed decoration seed is supplied; complete-floor originals validate run-to-decoration seed routing.', 'Recovered decoration xorshift35, repeated/shuffled floor catalogue and conditional wall draws; production uses the generated descriptor decoration seed.', 'Live paired floor gate validates descriptor seeds in production WASM; supplied-decoration GL comparison checks complete XY/UV/crop/mirror geometry separately.', 'FX stream routing and other room/stage variants remain unverified.'],
  ['floor-generation', '17 complete original floors include all shapes, special cropping and three actual two-attempt retries; the live paired gate adds three contrived retries and two OS-random source runs.', 'Run/stage-seed-only fresh normal Basement generator uses authored weights, special fitting/shrink, secrets and retry-continuing streams/weights/flags; starting-room runtime uses its descriptor.', 'test:e2e:floor compares every ordered descriptor, all 169 grid cells, geometry, masks, descriptor seeds and actual attempt counts against live originals through actual DOM keys and production WASM.', 'Labyrinth, other floors/profiles/challenges/curse effects, typed spawning, traversal and shared MT consumers.'],
  ['doors-minimap', 'Authored possible doors plus two actual native initial-room-to-neighbor transitions with descriptor/log/seed joins.', 'Starting doors follow generated eight-slot connectivity and authored grid positions; secret targets are hidden and shop/library targets are KeyClosed. Current/visited descriptor indices are retained.', 'Live floor gate checks starting descriptor identity, visited state and visible destination links; source-only transitions do not establish browser traversal parity.', 'Room transitions, exploration/minimap routing, persisted contents, combat locks and floor exits.'],
  ['stage-graphics', 'Nine original 512×512 light/shadow pairs, with independently retained depth24/GREATER/clear0/layer-Z state and repeated coasting-blast phases.', 'Authored GPU masks, signed raster corners, real 24-bit depth attachments, per-target layer depth, explicit highp texture sampling and player/explosion lights.', 'Integrated production matches all 18 complete RGBA targets exactly; unchanged zero budgets, seven mask integrity cases and eight native capture guards.', 'Other rooms/stages, sprite corner/rotation numerics, camera behavior and complete original frame pixels.'],
  ['health-inventory', 'Player/entity XML, starting HUD and controlled starting-room bomb HP 6→4/countdown 120 at presentation 121 through zero at 241.', 'Red HP/inventory HUD, bomb consumption, regular blast self-damage and observed 120-step immunity decrement; no pickup collection.', 'Starting HUD checkpoints, bomb unit/browser replay cases and native captures; consult actual results.', 'Native update/render phase alignment, other health containers/damage sources, hurt visuals, pickups, caps, death and combat damage.'],
  ['bombs', 'Native E/blast/immunity and coasting fixtures; new typed full phases update later-born bombs after the player, with120/121 fuse presentations depending birth phase.', 'E-only placement, strict55.25 contact/f32 impulse, 120-step immunity and authored rendering; legacy blast-before-motion ordering preserves its scoped E121 fixture.', 'Eleven focused Rust bomb cases, real-key E replay and five zero-budget GPU-mask checkpoints; these do not establish full-phase source actor order.', 'Integrate source birth/fuse/actor scheduling; sprite numerics, hurt visuals, craters, push/variants and browser enemy/grid damage.'],
  ['grid-collision', 'Typed native rocks, held/coasting contacts, vertical/horizontal blast rays and former-cell passage are retained with raw read-only source proof.', 'Standalone supplied-grid collision/bisection and blast-candidate kernels; browser still has only outer room bounds.', 'Four grid Rust cases and the required combat verifier cover exact conditional source contacts/blasts; no obstacle gameplay parity is claimed.', 'Authored grid factory/admission, integrated room state, variants/hazards/doors, destruction presentation and persistence.'],
  ['enemies-bosses', '495 definitions plus source factory/AI dispatch, native Gaper appearance/pathfinding, AttackFly contact and deferred tear/bomb HP drain.', 'Standalone strict-overlap/mass-contact and deferred-damage kernels from supplied actor inputs; no browser enemy/boss AI.', 'Four combat Rust cases and seven source integrity cases; eight conditional cases compare100 exact f32 checkpoints.', 'Live actor routing/update/collision/death callbacks, AI/projectiles, room-clear unlocking, bosses and browser integration.'],
  ['physics-kernels', 'Five independent native tear tapes retain215 exact phase observations; two MT lifecycle checkpoints retain all624 words and descriptor/player seed fields.', 'Standalone MT19937, concrete constructor/Clear consumers, full/fast phase and f32 tear height/ground/render kernels through a bounded CLI; browser shared consumers/tear path remain provisional.', 'Required test:physics-kernels verifies explicit source inputs; retained observation pairing and negative guards preserve provenance.', 'Browser shared MT admission/order, full mixed-rate scheduling, source collision/range termination and complete state parity.'],
  ['pickups', 'Heart/coin/key/bomb/collectible XML definitions and authored room spawns.', 'No pickup or chest/shop runtime.', 'No collection/damage scenario yet.', 'Spawn/collection, full-health refusal, inventory mutation, prices and chest rules.'],
  ['items-pools', '343 collectibles, 60 trinkets, 15 pools and concrete Linux config/weighted-selector/depletion/AddCollectible/use boundaries.', 'No item-effect, familiar or item-pool runtime.', 'Resource ingestion and static source map only; no equipped-item/pool parity scenario.', 'Unlock filters, depletion/RNG, stat caches, active charge and effect interactions.'],
  ['pocket-items', '33 card, eight rune and 28 pill-effect definitions.', 'No pocket item, pill assignment or drop/swap runtime.', 'Empty-slot native probes do not establish equipped-item coverage.', 'Equipped use, effect behavior, identification, mapping and swap/drop.'],
  ['characters-modes', '13 authored player definitions, 20 challenges and native character-menu frames.', 'Character carousel/profile unlock list exists; only normal Isaac can start.', 'Locked/unsupported selections reject atomically; selected Isaac browser path.', 'Other characters, initial stats/items, transformations, hard mode and challenges.'],
  ['eligibility-progression', 'Original manual-seed no-achievements badge and 178 achievement definitions.', 'Ordinary/manual seed eligibility state and badge.', 'Ordinary/manual random browser runs and state invariants.', 'Achievement conditions, unlock persistence, completion marks and progression.'],
  ['save-continue', 'Native GameState/PersistentGameData symbol families and profile UI.', 'In-memory profiles and JSON replay; no durable browser save.', 'Serialized port continuation tests establish implementation determinism only.', 'Browser reload, versioned durable profiles, original save/continue observations.'],
  ['audio-cutscenes', '45 music and 169 sound definitions, 17 videos and authored cutscene metadata.', 'No music/sound/video playback runtime.', 'No audio/video behavior gate yet.', 'Event timing, variants/loops, stage transitions, pause and browser audio unlock.'],
  ['co-op', 'Coplayer entity and 203 baby definitions.', 'No co-op or controller runtime.', 'No co-op behavior gate yet.', 'Join/leave, heart allocation, babies, shared room state and device handling.'],
  ['long-run-performance', 'Native process and browser phase timings are separately recorded.', 'Short menu/starting-room runs; production catch-up avoids repeated full presentation.', 'Browser catch-up case plus verification phase timings.', 'Comparable simulation workloads, long-run numerical/state stability and memory behavior.'],
];
const missing = [
  ['original-tick-state', 'Original simulation-tick scheduling and complete hidden-state observation'],
  ['core-gameplay', 'Combat rooms, enemies, complete health/damage rules, pickups, item effects and complete bomb interactions'],
  ['all-inputs', 'Equipped active items/cards/pills, drops, map/restart, remapping, mouse and controllers'],
  ['hidden-progression', 'RNG streams, room/floor generation, item pools, devil/angel chances and progression'],
  ['metagame', 'Other characters/modes, profiles, unlocks, achievements and save/continue'],
  ['numerics-performance', 'Observed native floating-point edges, long runs and comparable simulation/render performance'],
];
const sources = ['engine/src/floor.rs', 'engine/src/room_assets.rs', 'engine/src/bin/floor.rs', 'tools/foundation/floor-e2e.mjs', 'tools/original/foundation-floor-verify.py', 'engine/build.rs', 'engine/src/content.rs', 'engine/src/lib.rs', 'engine/src/seed.rs', 'engine/src/room.rs', 'engine/src/effects.rs', 'engine/src/bin/replay.rs', 'web/engine.ts', 'web/main.ts', 'web/renderer.ts', 'web/menu.ts', 'web/animation.ts', 'web/graphics-contracts.ts', 'tools/tests/foundation.spec.ts', 'tools/foundation/verify.mjs', 'tools/foundation/compare.mjs', 'tools/foundation/compare.test.mjs', 'tools/foundation/verify.test.mjs', 'tools/foundation/seed-invariants.mjs', 'tools/foundation/seed-invariants.test.mjs', 'tools/foundation/seeds.mjs', 'tools/foundation/original-seed-contract.mjs', 'tools/foundation/original-seed-contract.test.mjs', 'tools/original/foundation-random-seeds.py', 'web/compositor.ts', 'tools/foundation/compositor.test.mjs', 'web/png.ts', 'web/gl-menu.ts', 'tools/foundation/png.test.mjs', 'tools/foundation/stage-masks.mjs', 'tools/foundation/stage-masks.test.mjs', 'references/isaac/foundation/stage-masks/manifest.json', 'tools/original/foundation-capture.py', 'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-clock-verify.py', 'tools/original/foundation-clock-gate-checks.py', 'tools/original/foundation-clock-verify-checks.py', 'tools/original/foundation-gl-observe-checks.py', ORIGINAL_CLOCK_SOURCE];
export function renderMatrix({ originalClockScenarios } = {}) {
  assert.equal(new Set(CHECKPOINTS).size, CHECKPOINTS.length);
  for (const path of sources) assert.ok(existsSync(resolve(root, path)), `Missing matrix source ${path}`);
  assert.equal(new Set(TASKS.map(task => task.id)).size, TASKS.length, 'System task IDs must be unique');
  assert.equal(new Set(SUBSYSTEMS.map(([id]) => id)).size, SUBSYSTEMS.length, 'Subsystem IDs must be unique');
  for (const row of SUBSYSTEMS) assert.ok(row.length === 5 && row.every(value => typeof value === 'string' && value.length), 'Subsystem rows require evidence, implementation, checks and a remaining contract');
  for (const task of TASKS) for (const path of task.sources) assert.ok(existsSync(resolve(root, path)), `Missing system task source ${path}`);
  for (const id of CHECKPOINTS) assert.ok(labels[id], `Document new executable checkpoint ${id}`);
  const browserCases = discoverTestCases({ root, pattern: 'tools/tests/**/*.spec.ts' });
  const nodeCases = discoverNodeCases({ root });
  const maskCases = readStageMaskCases();
  const clockCases = originalClockScenarios === undefined
    ? discoverOriginalClockScenarios({ root, systemIds: TASKS.map(task => task.id) })
    : validateOriginalClockScenarios(originalClockScenarios, { systemIds: TASKS.map(task => task.id) });
  return `# Foundation test matrix

Generated from the executable checkpoints in [verify.mjs](tools/foundation/verify.mjs). Run \`npm run matrix:update\` after adding a checkpoint; \`npm run matrix:check\` rejects drift. The previous port and its scenarios have been removed.

\`npm run check\` builds the Rust/WASM foundation, runs native unit, real-keyboard browser and fresh random-seed original tests, then compares its screenshots against two independent original captures. Every selected check executes and failures remain failures. There are no skip or expected-failure paths.

Required component commands \`test:physics-kernels\`, \`test:layout-kernels\` and \`test:combat-kernels\` compare standalone kernels with independently supplied source inputs. They do not establish browser room routing, shared random-consumer scheduling or NPC AI. Finalized generation is a separate required live original/browser gate, \`test:e2e:floor\`; it uses public seeds and authored catalogues without supplied native layouts. Strict supplied-input growth diagnostics remain available with their narrower scope.

## Port subsystem status

The [system map](docs/system-map.md) records original resource/binary evidence, structural asset contracts and parallel implementation acceptance. These rows distinguish independent evidence, current implementation and executable checks. A listed check is a declared contract, not a passing result; consult the actual report. Original presentation repeats and Rust/WASM replay equality do not establish original simulation ticks or complete hidden-state parity. Update this table through [matrix.mjs](tools/foundation/matrix.mjs) as a subsystem changes.

| Subsystem | Original evidence | Port implementation | Executable checks | Remaining contract |
| --- | --- | --- | --- | --- |
${SUBSYSTEMS.map(([id, evidence, implementation, checks, remaining]) => `| \`${id}\` | ${evidence} | ${implementation} | ${checks} | ${remaining} |`).join('\n')}

## Executable coverage

| Executable checkpoint | Required behavior | Source |
| --- | --- | --- |
${CHECKPOINTS.map(id => `| \`${id}\` | ${labels[id]}; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |`).join('\n')}
| \`command-state\` | Every actual DOM key/text/tick command matches native Rust and browser WASM state | [replay.rs](engine/src/bin/replay.rs), [browser tests](tools/tests/foundation.spec.ts) |
| \`pause-release\` | Pause freezes the run; released keys cannot stick | [browser tests](tools/tests/foundation.spec.ts) |
| \`random-seed-invariants\` | New OS-random seeds replay identical movement, every shooting direction, pause and release state in native Rust and WASM | [invariants](tools/foundation/seed-invariants.mjs), [seed bank/replay](tools/foundation/seeds.mjs) |
| \`random-seed-browser\` | Actual keys enter ordinary and manual-seed runs; eligibility, movement, shots and generated room images respond to the supplied seeds | [browser tests](tools/tests/foundation.spec.ts) |
| \`production-entropy\` | Browser boot uses crypto entropy; invalid explicit seed inputs reject without changing state | [browser tests](tools/tests/foundation.spec.ts) |
| \`original-finalized-floor\` | Three contrived native retries plus two OS-random runs compare all ordered descriptors, 169 grid cells, seeds and actual attempts with production WASM entered through DOM keys; all selected cases execute | [live gate](tools/foundation/floor-e2e.mjs), [native capture/comparison](tools/original/foundation-floor-verify.py) |
| \`original-random-seed-contract\` | Fresh random native runs accept the code/value and report the Stage 1 seed predicted by Rust/WASM; every selected bank case executes and has real log/PNG evidence | [adapter](tools/foundation/original-seed-contract.mjs), [adapter tests](tools/foundation/original-seed-contract.test.mjs), [native probes](tools/original/foundation-random-seeds.py) |
| \`original-clock-repeat\` | New random seeds repeat every selected native RGBA checkpoint, public log state and consumed X11 input at identical virtual presentation phases | [clock runner](tools/original/foundation-clock-run.py), [repeat gate](tools/original/foundation-clock-verify.py), [ABI tests](tools/original/foundation-clock-gate-checks.py), [negative tests](tools/original/foundation-clock-verify-checks.py) |
| \`original-graphics-frame-contract\` | Pair captures with complete public-graphics frames; reject nested/incomplete/reused frames, post-presentation cache draws and unavailable geometry | [observer regression tests](tools/original/foundation-gl-observe-checks.py) |
| \`original-stage-compositing\` | Actual DOM inputs compare full 512×512 straight RGBA light/shadow targets at all ${maskCases.length} selected native checkpoints, including three seeds, controlled movement and coasting bomb presentations 121/123/127/131/133; every RGBA budget is zero | [browser cases](tools/tests/foundation.spec.ts), [mask comparator](tools/foundation/stage-masks.mjs), [native fixtures](references/isaac/foundation/stage-masks/manifest.json), [capture guards](tools/original/foundation-stage-mask-capture-checks.py) |
| \`authored-mask-raster\` | Authored XML/PNG shadow dimensions feed general effect descriptors; every light/shadow has valid signed integer raster corners, compared in native Rust/WASM state and original GPU masks | [content extraction](engine/build.rs), [content catalogue](engine/src/content.rs), [effects](engine/src/effects.rs), [semantic guards](tools/foundation/seed-invariants.mjs) |
| \`original-backdrop-contract\` | Given an independently observed room-decoration seed, compare every floor/wall destination and texture coordinate, crop and mirror against complete original GL cache submissions; this does not prove production run→room seed routing | [native verifier](tools/original/foundation-backdrop-verify.py), [integrity cases](tools/original/foundation-backdrop-verify-checks.py), [Rust diagnostic](engine/src/bin/backdrop.rs) |
| \`authored-room-catalog\` | Decode all 13 pinned base STB sets, consume every byte, preserve weighted/special selectors and signed doors, reject malformed records and re-encode original bytes exactly; this is resource structure, not runtime spawning or floor parity | [decoder](tools/foundation/decode-rooms.py), [corruption/reference checks](tools/foundation/decode-rooms-checks.py) |
| \`original-authored-quads\` | Compare generic sequential f32 interpolation and transforms against all 75 original title and 42 smoke quads/alpha values, retaining independent held-out smoke phases; this does not establish full title/sprite pixels | [math contracts](tools/foundation/animation-math.test.mjs), [generic animation math](web/animation-math.ts), [native primitives](references/isaac/foundation/animation-math/primitives.json) |
| \`original-room-template-selector\` | Given independently observed selector seed, authored shape and door mask, compare every chosen STB identity and mutable weight decay across all eight retained original floors; full run→room routing is separate | [template kernel](engine/src/room_templates.rs), [bounded CLI](engine/src/bin/layout.rs), [source verifier](tools/original/foundation-layout-verify.py), [integrity cases](tools/original/foundation-layout-verify-checks.py) |
| \`original-room-growth\` | Given an independent generator seed, request and shape mask, compare all finalized native rectangles using \`--contract growth\` or \`both\`; selected special fitting/resizing mismatches remain nonzero failures | [growth kernel](engine/src/layout.rs), [source verifier](tools/original/foundation-layout-verify.py), [scope and retained failures](docs/room-layout-evidence.md) |
| \`original-physics-kernels\` | Given explicit source inputs, compare215 tear phases across five native tapes plus two complete624-word MT lifecycle states and actual descriptor/player seed fields; browser consumer routing remains separate | [required verifier](tools/foundation/physics-kernels.py), [bounded CLI](engine/src/bin/physics.rs), [source observer](tools/original/foundation-physics-observer.py) |
| \`original-combat-kernels\` | Compare eight conditional grid/contact/damage cases and100 exact f32 checkpoints with hash-bound raw original observations, public keys and phases; supplied actor motion does not establish live NPC AI | [strict verifier](tools/original/foundation-combat-verify.py), [integrity cases](tools/original/foundation-combat-verify-checks.py), [bounded CLI](engine/src/bin/combat.rs), [source proofs](references/isaac/foundation/combat/native-observations.json) |
| \`comparison-guards\` | Wrong/blank graphics, missing references and missing state cannot pass | [compare tests](tools/foundation/compare.test.mjs), [reference tests](tools/foundation/verify.test.mjs) |

Actual browser cases below are discovered from the executable browser scenario sources; adding or removing one requires regenerating this matrix.

| Executable browser case | Source |
| --- | --- |
${browserCases.map(({ name, source }) => `| ${name} | [${source.split('/').at(-1)}](${source}) |`).join('\n')}

Selected GPU-mask checkpoints below come directly from the native fixture manifest consumed by the browser gate. Each selected case compares every straight RGBA byte in both complete targets with zero tolerance. Controlled cases require actual X11 delivery, complete paired GL phases, binary/log/target hashes and independently repeated original evidence; this remains a graphics-target contract, not full original state or frame parity.

| Selected native GPU case | Required DOM input / native phase | Targets |
| --- | --- | --- |
${maskCases.map(({ id, browserTape, targets }) => `| \`${id}\` | ${browserTape.phase ?? `D${browserTape.movementHoldTicks}/release${browserTape.movementReleaseTicks}${browserTape.bombPresentation === undefined ? '' : ` → actual E; bomb presentation ${browserTape.bombPresentation}`}`} | [shadow](references/isaac/foundation/stage-masks/${targets.shadow}), [lighting](references/isaac/foundation/stage-masks/${targets.lighting}) |`).join('\n')}

Node cases follow the same source glob as \`test:verification\`. The index rejects disabled or undocumented dynamic registrations.

| Executable verification case | Source |
| --- | --- |
${nodeCases.map(({ name, source }) => `| ${name} | [${source.split('/').at(-1)}](${source}) |`).join('\n')}

Original presentation scenarios below are discovered by executing [foundation-clock-scenarios.py --list](tools/original/foundation-clock-scenarios.py). Their captures, input events and interval counts come from the actual validated tapes. The index rejects missing required scenarios, malformed tapes and unknown system tasks. These are original-only reference scenarios; their presence does not assert that the browser already matches them. Replay a seed with \`python3 tools/original/foundation-clock-scenarios.py --scenario ID --seed CODE\`, then run the emitted capture command.

| Executable original scenario | Required behavior / systems | Tape checkpoints / intervals | Source |
| --- | --- | --- | --- |
${clockCases.map(scenario => `| \`${scenario.id}\` | ${scenario.description}; \`${scenario.systems.join(', ')}\` | \`${scenario.captureLabels.join(', ')}\`; ${scenario.presentedFrames} presentation intervals, ${scenario.inputEvents} input events | [foundation-clock-scenarios.py](${scenario.source}) |`).join('\n')}

Combat routes replay their retained source seed; arbitrary-seed overrides are discovery and do not assert the same neighboring room or actors. Their source observations/conditional assertions are separate from browser gameplay coverage.

The original uses seed **T1PK 6Y0W**, fresh profiles, and a 960×540 window. Captures and observed log/input metadata live in [baseline](references/isaac/foundation/baseline/manifest.json) and [repeat](references/isaac/foundation/repeat/manifest.json). They are comparison evidence; the renderer composes authored sprites and never draws a captured game frame. [Capture](tools/original/foundation-capture.py) and [save-reference.py](tools/foundation/save-reference.py) refresh these fixtures from the runnable original.

Pixel gates compare decoded RGBA over the whole frame, HUD, actor and projectile regions. Any tolerance is measured from the independent native captures; it is not a freely chosen similarity threshold. Reports distinguish exact pixels from native capture variability. Legacy screenshot fixtures use wall-clock keys/SIGSTOP. The newer clock adapter controls presented frames and main-thread time, disables X11 autorepeat and records actual consumed key events; neither adapter proves native simulation ticks or hidden-state parity. Native process CPU/RSS and browser draw/encoding timings remain separate.

## Divergence tasks by game system

Generated from [systems.mjs](tools/foundation/systems.mjs). Each failed image region links to these investigation tasks, state fields and next tests. A region can have several candidate causes; attribution is not proof. Fix the responsible system and rerun its reference scenario before closing a task. Do not substitute a renderer offset, per-seed tile map or screenshot-derived correction image for a missing system.

| Task | Owning system / state | Required next test |
| --- | --- | --- |
${TASKS.map(task => `| \`${task.id}\` | ${task.system}; \`${task.fields.join(', ')}\` | ${task.nextTest} |`).join('\n')}

Production behavior is seed-independent: seeds enter a general codec and generators. Seed-specific expected outputs belong only in reference fixtures. Invariants use new OS-random seeds, record those values for replay, and verify the same tape in native Rust and WASM. Randomness does not excuse missing original comparisons or permit seed-specific implementation branches.

## Required extensions

These contracts remain incomplete beyond the scoped systems above. Add the missing original-observed setup, input tape and assertions to the executable verifier before claiming coverage; a partial implemented interaction does not complete its subsystem.

| Missing area | Required scenarios |
| --- | --- |
${missing.map(([id, text]) => `| \`${id}\` | ${text} |`).join('\n')}

Extend the [input catalogue](tools/verification/input-catalog.json), [Rust commands/state](engine/src/lib.rs), [asset renderer](web/renderer.ts), [authored animation loader](web/animation.ts) and [browser scenario](tools/foundation/verify.mjs) together. Register discoveries before implementing them, keep failing comparisons, and code to green. Do not replace a reference with clone output, weaken a pixel gate, omit a selected checkpoint, or call native Rust/WASM self-equality original-game parity.

Latest actual results, state, pixels, differences and phase timings: \`.cache/foundation/verification/latest/report.json\` and checkpoint directories. Declared coverage here is not a cached pass result.
`;
}
export function checkMatrix({ path = target, rendered = renderMatrix() } = {}) {
  assert.equal(readFileSync(path, 'utf8'), rendered, 'TEST_MATRIX.md is stale; run npm run matrix:update');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.includes('--write')) {
    writeFileSync(target, renderMatrix());
    console.log('Updated TEST_MATRIX.md from executable foundation checkpoints and scenarios');
  } else {
    checkMatrix();
    console.log('Foundation checkpoint matrix and source agree');
  }
}
