import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { CHECKPOINTS } from './verify.mjs';
import { TASKS } from './systems.mjs';
import { readStageMaskCases } from './stage-masks.mjs';
import { discoverPairedGameScenarios, discoverTestCases, discoverOriginalClockScenarios, validateOriginalClockScenarios, ORIGINAL_CLOCK_SOURCE } from './scenario-index.mjs';
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
  ['runtime-abi', 'Original source/data establish independent boot epoch, SavedGame/Level/Backdrop construction and menu initialization.', 'Rust JSON ABI2 and shared boot MT execute descriptor initialization, profile preview/reset, Character menu seed and179 Stats poses; historical descriptor origins/mutations remain internal, with bounded snapshot samples.', 'test:e2e:boot-mt compares complete624-word MT, player/menu seeds, nine grid pairs and all179 poses with live originals for zero/fixed plus four OS-random boot epochs and future manual seeds.', 'Post-title navigation/audio, run-entry/room/ambient shared consumers, complete continuation and simulation-tick identity remain unverified.'],
  ['menus', 'Independent title/profile/main/character/seed frames.', 'State-driven menu path, seed dialog, profile selection and authored animation.', 'Reviewed live title/menu/reentry passes 19/19 functional checkpoints. The optional legacy verify:foundation pixel diagnostic matches five complete menu frames while legacy title and four gameplay images still fail.', 'Legacy title pixels, settings, challenges, continue and broader navigation observations.'],
  ['seed-codec', 'Original fresh random-seed acceptance and Stage 1 logs.', 'General checksum codec, crypto boot entropy and Stage 1 startup stream.', 'Live original and browser manually enter fresh OS-random seeds and require actual readback; source-only seed probes remain diagnostic.', 'Special seed effects and all later stream routing.'],
  ['input-clock', 'Consumed native X11 events and source full/intermediate phases; full actor updates stop at entry admission, intermediate traversal sees same-phase additions, and run entry normalizes odd phase parity.', 'Keyboard/text commands, releases, pause/catch-up and normalized systemClock; ordered typed actor admission now dispatches player, tear and bomb phases; source pause completion preserves its frozen clear phase. Tear height/range remains provisional.', 'Original clock repeats, native/WASM tapes, atomic boot-reset and actual DOM key cases.', 'Original phase delivery and full gameplay mixed-rate integration, complete state, controller, mouse and remapping contracts.'],
  ['run-introduction', 'Fresh entry/pause captures and an independent arbitrary-seed native/browser probe match exact banner text, paper/stat/cursor and seed regions.', 'Run-introduction and pause-menu phase/age/selection state; authored banner/paper/seed/stat composition.', 'Reusable native presentation scenario and two exact-region browser cases; complete loading/fade/frame parity remains open.', 'Full transition pixel/timing parity, input enablement holdouts and unsupported pause menu actions.'],
  ['authored-animation', 'Independent original GL submissions provide 75 title quads and 42 smoke quads; ten title/fade/reentry source checkpoints repeat exactly.', 'Generic sequential f32 interpolation/transforms and source floorf cached-frame selection; retained title/f32 fade uses normalized System phases.', 'Live paired title/fader lifecycle and retained strict browser pixel scenarios; earlier math unit contracts are now ignored research. Prior immutable ABI 2 artifact matches seven complete title frames and ten scoped title/fade states.', 'Legacy title whole-frame still differs in 10,538 pixels; menu transition rendering, complete smoke/sprite pixels, arbitrary-angle library edges and other authored consumers.'],
  ['locomotion', 'Native source/trajectory and four wall probes establish motion/reversal laws and authored radius-clearance bounds.', 'Previous-velocity integration, projection, release/turn/reversal behavior, and player walls derived from floor/radius metadata.', 'Native/WASM invariants and held-out native trajectories; original pixel gates remain strict.', 'Original simulation cadence, full typed state, collision velocity response and complete actor pixels.'],
  ['tears', 'Fresh stationary/moving/coasting traces establish inherited velocity; source-located signed cooldown/head fields and full/intermediate shot births are observed in fresh originals.', 'Four-direction regular tears, inherited motion, ordered birth fencing, full-only signed cooldown and head timer; provisional projectile bounds remain separate from measured player walls.', 'Live original/production-browser gate compares two contrived birth phases plus four recorded OS-random seeds, cooldown/head fields, shot admission and horizontal birth projection.', 'Ground/height/gravity separation, range-end drop, initial RNG/eye routing, fixed 45-step expiry, provisional projectile bounds, damage and other weapons.'],
  ['room-assets', 'All 13 packed STB stage sets decode and re-encode byte-identically: 5,314 rooms and 206,690 weighted entries; 790 authored actors have known names.', 'Complete production Special Rooms/Basement STB decoder retains authored doors and every spawn selector; generated descriptors resolve original catalogue entries.', 'Full floor comparisons exercise authored template identities; existing structural asset tooling retains its narrower contract.', 'Source RNG11 weighted groups and local actor/grid seeds are admitted and retained; unsupported selector transformations and actor callbacks remain separate.'],
  ['backdrop-decoration', 'Four independent random-seed originals match all four floor and 44 wall submitted quads when the observed decoration seed is supplied; complete-floor originals validate run-to-decoration seed routing.', 'Recovered decoration xorshift35, repeated/shuffled floor catalogue and conditional wall draws; production uses the generated descriptor decoration seed.', 'Live paired floor gate validates descriptor seeds in production WASM; supplied-decoration GL comparison checks complete XY/UV/crop/mirror geometry separately.', 'FX stream routing and other room/stage variants remain unverified.'],
  ['floor-generation', '17 complete original floors include all shapes, special cropping and three actual two-attempt retries; the live paired gate adds three contrived retries and two OS-random source runs.', 'Run/stage-seed-only fresh normal Basement generator uses authored weights, special fitting/shrink, secrets and retry-continuing streams/weights/flags; starting-room runtime uses its descriptor.', 'test:e2e:floor compares every ordered descriptor, all 169 grid cells, geometry, masks, descriptor seeds and actual attempt counts against live originals through actual DOM keys and production WASM.', 'Labyrinth, other floors/profiles/challenges/curse effects, typed spawning, traversal and shared MT consumers.'],
  ['doors-minimap', 'Live original doorway/return tapes retain every presented phase, public room/SpawnRNG logs and read-only descriptors; source curse-room entry/exit establishes half-heart damage,60-presentation immunity and transition/grid timing.', 'Ordinary style0 13x7 normal/curse corridors use authored positions and reciprocal descriptors. Curse contacts run before player motion; opened-door collision and hurt immunity follow source admission/completion callbacks. Exited Room clones retain selected groups and supported state; visited minimap follows reached descriptors.', 'test:e2e:traversal retains its mandatory hostile Rock-room lock regression. test:e2e:doors compares every phase of contrived opposite-phase and recorded OS-random curse entry/exit, including exact HP/immunity, visible door state/class, player f32 motion and descriptor/spawn/return joins.', 'Secret/key/hostile locks and discovery, other door styles, flight/items, raw NPC behavior, large-room camera/backdrops, complete transition pixels and floor exits.'],
  ['stage-graphics', 'Nine original 512×512 light/shadow pairs, with independently retained depth24/GREATER/clear0/layer-Z state and repeated coasting-blast phases.', 'Authored GPU masks, signed raster corners, real 24-bit depth attachments, per-target layer depth, explicit highp texture sampling and player/explosion lights.', 'Prior production matches all 18 complete RGBA targets exactly; strict source-mask browser diagnostics remain available.', 'Other rooms/stages, sprite corner/rotation numerics, camera behavior and complete original frame pixels.'],
  ['health-inventory', 'Player/entity XML, starting HUD and controlled starting-room bomb HP 6→4/countdown 120 at presentation 121 through zero at 241.', 'Red HP/inventory HUD, bomb consumption, regular blast self-damage and observed 120-step immunity decrement; no pickup collection.', 'Live paired red-HP/countdown and bomb-count checks plus actual-key browser scenarios; consult actual results.', 'Native update/render phase alignment, other health containers/damage sources, hurt visuals, pickups, caps, death and combat damage.'],
  ['bombs', 'Native rising-edge E, post-motion spawn/velocity, full-only placement cooldown, timer59 and birth-fenced full phases establish120/121 fuse presentations.', 'Phase-aware f32 actual/predicted bomb motion, player-before-later-bomb dispatch, strict55.25 blast contact/impulse and HP/countdown; independent authored corpse/explosion clocks.', 'Live original/browser even/odd moving-birth, held-E, pause/release and recorded OS-random tapes; nine existing zero-budget GPU mask cases retain separate exact-pixel results.', 'Recorded random C7YPWS61 exposes nonplayer wall collision; other variants, craters, hurt visuals and browser enemy/grid damage remain open.'],
  ['grid-collision', 'Typed native rocks, held/coasting contacts, blast rays and ordinary Rock frame/link initialization retain raw read-only source proof.', 'Persisted authored ordinary Rock and active Spike descriptors drive player/bomb ground contacts, original static clips and full-grid-before-player spike damage/impulse/immunity; unsupported selectors remain explicit. Boundary/door entries describe geometry without actor RNG claims.', 'Dense C7YPWS61 retains42 paired checkpoints/125 bomb fields. Recorded22PFCSFS plus a phase-offset spike replay and three bomb/pause holdouts pass143 paired checkpoints and5049 grid/phase fields; live traversal retains hostile-return lock failures.', 'Retracting/sacrifice spike rules and exceptions, nonplayer hazard damage, other grid factories, rock destruction/drops, combat locks and full grid graphics parity.'],
  ['enemies-bosses', '495 definitions plus source factory/AI dispatch, native Gaper appearance/pathfinding, AttackFly contact and deferred tear/bomb HP drain.', 'Standalone strict-overlap/mass-contact and deferred-damage kernels from supplied actor inputs; no browser enemy/boss AI.', 'Retained source observations contain eight conditional cases/100 f32 checkpoints; browser enemy AI is not accepted.', 'Live actor routing/update/collision/death callbacks, AI/projectiles, room-clear unlocking, bosses and browser integration.'],
  ['physics-kernels', 'Five independent native tear tapes retain215 exact phase observations; two MT lifecycle checkpoints retain all624 words and descriptor/player seed fields.', 'MT19937, concrete constructor/Clear consumers, full/fast phase and f32 tear height/ground/render kernels; browser shared consumers/tear path remain provisional.', 'Retained source inputs document the kernel; live paired tear/bomb admission is the required gameplay gate.', 'Browser shared MT admission/order, full mixed-rate scheduling, source collision/range termination and complete state parity.'],
  ['pickups', 'Authored XML/ANM2 metadata and live coin rooms recover seeded weighted admission, subtype rolls, full-only animation/contact and Player-before-coin admission order.', 'Typed room selectors retain unsupported entities/grids; natural regular coins use authored rendering, appear/idle/collect clocks, post-actor full contact, coin amounts/cap99 and Room-owned persistence.', 'test:e2e:pickups executes two contrived contact offsets, fresh recorded OS-random coin rooms and mandatory22PFCSFS spike regression; functional failures keep the default cohort red until the owning system is fixed.', 'Key/bomb/heart and other pickup collection, live99-cap boundary and pickup reentry parity, shop/chest/item rules, grid hazards, shared MT effects and full sprite/pixel parity.'],
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
const sources = ['engine/src/pickups.rs', 'engine/src/room_spawns.rs', 'web/pickups.ts', 'web/pickup-data.json', 'tools/e2e/pickups.mjs', 'tools/original/pickup-scenarios.py', 'tools/original/foundation-pickup-observe.py', 'references/isaac/foundation/pickups/source-contracts.json', 'references/isaac/foundation/pickups/known-regressions.json', 'references/isaac/foundation/pickups/scenarios.json', 'references/isaac/foundation/pickups/live-report.json', 'tools/e2e/boot-mt.mjs', 'tools/original/boot-mt-observer.py', 'references/isaac/foundation/boot-mt/source-contracts.json', 'engine/src/actors.rs', 'tools/e2e/weapons.mjs', 'tools/original/weapons-scenarios.py', 'references/isaac/foundation/weapons/source-contracts.json', 'engine/src/floor.rs', 'engine/src/room_assets.rs', 'engine/src/bin/floor.rs', 'tools/foundation/floor-e2e.mjs', 'tools/original/foundation-floor-verify.py', 'engine/build.rs', 'engine/src/content.rs', 'engine/src/lib.rs', 'engine/src/seed.rs', 'engine/src/room.rs', 'engine/src/effects.rs', 'engine/src/bin/replay.rs', 'web/engine.ts', 'web/main.ts', 'web/renderer.ts', 'web/menu.ts', 'web/animation.ts', 'tools/tests/foundation.spec.ts', 'tools/foundation/verify.mjs', 'tools/foundation/compare.mjs', 'tools/foundation/seed-invariants.mjs', 'tools/foundation/seeds.mjs', 'tools/foundation/original-seed-contract.mjs', 'tools/original/foundation-random-seeds.py', 'web/compositor.ts', 'web/png.ts', 'web/gl-menu.ts', 'tools/foundation/stage-masks.mjs', 'references/isaac/foundation/stage-masks/manifest.json', 'tools/original/foundation-capture.py', 'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-clock-verify.py', ORIGINAL_CLOCK_SOURCE];
export function renderMatrix({ originalClockScenarios } = {}) {
  assert.equal(new Set(CHECKPOINTS).size, CHECKPOINTS.length);
  for (const path of sources) assert.ok(existsSync(resolve(root, path)), `Missing matrix source ${path}`);
  assert.equal(new Set(TASKS.map(task => task.id)).size, TASKS.length, 'System task IDs must be unique');
  assert.equal(new Set(SUBSYSTEMS.map(([id]) => id)).size, SUBSYSTEMS.length, 'Subsystem IDs must be unique');
  for (const row of SUBSYSTEMS) assert.ok(row.length === 5 && row.every(value => typeof value === 'string' && value.length), 'Subsystem rows require evidence, implementation, checks and a remaining contract');
  for (const task of TASKS) for (const path of task.sources) assert.ok(existsSync(resolve(root, path)), `Missing system task source ${path}`);
  for (const id of CHECKPOINTS) assert.ok(labels[id], `Document new executable checkpoint ${id}`);
  const browserCases = discoverTestCases({ root, pattern: 'tools/tests/**/*.spec.ts' });
  const pairedCases = discoverPairedGameScenarios({ root, systemIds: TASKS.map(task => task.id) });
  const maskCases = readStageMaskCases();
  const clockCases = originalClockScenarios === undefined
    ? discoverOriginalClockScenarios({ root, systemIds: TASKS.map(task => task.id) })
    : validateOriginalClockScenarios(originalClockScenarios, { systemIds: TASKS.map(task => task.id) });
  return `# Foundation test matrix

Generated from live paired-game declarations, actual browser e2e scenarios and retained strict pixel diagnostics. Run \`npm run matrix:update\` after adding a scenario or checkpoint; \`npm run matrix:check\` rejects drift. The previous port and its scenarios have been removed.

\`npm run check\` builds the Rust/WASM port and runs every live suite registered in [run.mjs](tools/e2e/run.mjs), retaining every exit status before failing. CI discovers that same registry and runs each suite in a separate job with \`fail-fast: false\`; a failed suite cannot cancel the other system checks. Each job checks assets, matrix synchronization, types, build and strict Clippy before its live original/browser gate. The paired gate gives a fresh original process and immutable production browser the same public tape after an explicit first-Title bootstrap boundary. CI gates actual input delivery, seed readback and declared source-observable functional state across contrived and recorded OS-random cases. Every selected case executes; failures remain failures. Checked-in unit suites have been removed by user instruction.

All paired captures retain strict decoded-RGBA diagnostics. Their pixel failures remain failures in the report; tiny visual differences do not determine the default functional CI result. \`--require-exact-pixels\` opts into full-frame equality. The retained \`verify:foundation\` gate remains strict and optional, with its five known legacy full-frame failures. Earlier supplied-input physics/layout/combat diagnostic CLIs are now ignored research; their retained evidence remains separate from live browser gameplay acceptance.

## Required live paired-game scenarios

The definitions below are discovered by executing the actual tape builder. Default CI executes all six contrived cases plus two fresh OS-random seed/action tapes. Random tapes sample three movement/fire direction pairs with replacement; they do not guarantee every direction or birth phase. Contrived moving-bomb tapes cover both birth phases. \`--replay-cases PATH,...\` reruns the exact saved seed **and** all random action choices. Seed-only replay creates new action choices and is discovery.

| Paired scenario | Behavior / systems | Shared bootstrap / captures | Presentation intervals |
| --- | --- | --- | --- |
${pairedCases.map(item => `| \`${item.id}\` | ${item.description}; \`${item.systems.join(', ')}\` | \`${item.boundaryLabel}\`; \`${item.captureLabels.join(', ')}\` | ${item.presentationModel} |`).join('\n')}

The original introductory cutscene is retained in each native tape as a source-only prelude. The browser currently starts at the independently observed first Title frame0/f32 fade .975 boundary. This alignment is explicit; it does not claim cutscene implementation or native simulation-tick identity. Original state observations never become browser inputs. Actual reports preserve the native binary hash, production build digest, consumed X11 phases/focus, original PNGs, real DOM trace, typed source state, pixel differences and failed functional fields.

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
| \`browser-seed-invariants\` | Actual browser DOM runs check movement, every shooting direction, pause and release state; helper assertions are consumed only by these browser scenarios | [browser cases](tools/tests/foundation.spec.ts), [helpers](tools/foundation/seed-invariants.mjs) |
| \`random-seed-browser\` | Actual keys enter ordinary and manual-seed runs; eligibility, movement, shots and generated room images respond to the supplied seeds | [browser tests](tools/tests/foundation.spec.ts) |
| \`production-entropy\` | Browser boot uses crypto entropy; invalid explicit seed inputs reject without changing state | [browser tests](tools/tests/foundation.spec.ts) |
| \`original-boot-menu-mt\` | Drive public native time input and manual seed entry in both originals and immutable production browser; compare all624 MT words, player/menu seeds, nine saved grid pairs and179 stat poses at initialized Title across two contrived and four recorded OS-random boot epochs. Later shared consumers remain separate. | [live gate](tools/e2e/boot-mt.mjs), [bounded readback](tools/original/boot-mt-observer.py), [source proof](references/isaac/foundation/boot-mt/source-contracts.json) |
| \`original-weapons-cadence\` | Live original and immutable production browser receive ordinary menu/seed/fire DOM/X11 input; compare every cooldown/head/phase and shot admission over 54 held and26 released presentations in full/intermediate birth cases and four newly recorded OS-random directions. Native memory is read-only and never enters WASM. | [live gate](tools/e2e/weapons.mjs), [public tapes](tools/original/weapons-scenarios.py), [read-only fields](tools/original/foundation-physics-observer.py), [source proof](references/isaac/foundation/weapons/source-contracts.json) |
| \`original-ordinary-door-return\` | Contrived no-hostile return/hostile Rock-room return-lock regression plus recorded OS-random actual X11/DOM routes compare every source room/player phase, native entity/Rock seed joins and return admission persistence | [live gate](tools/e2e/traversal.mjs), [source tapes/provenance](tools/original/foundation-traversal-scenarios.py) |
| \`original-curse-door-contact\` | Contrived opposite-phase and recorded OS-random curse entry/public wait/exit compare every player/room phase, exact half-heart/immunity dispatch, visible door state/class and return admission | [live gate](tools/e2e/doors.mjs), [public source tapes](tools/original/door-scenarios.py), [source proof](docs/door-mechanics-evidence.md) |
| \`original-finalized-floor\` | Three contrived native retries plus two OS-random runs compare all ordered descriptors, 169 grid cells, seeds and actual attempts with production WASM entered through DOM keys; all selected cases execute | [live gate](tools/foundation/floor-e2e.mjs), [native capture/comparison](tools/original/foundation-floor-verify.py) |
| \`original-random-seed-contract\` | Live paired games enter contrived and recorded OS-random seed codes through public keys and require original/browser seed readback; original log/PNG parsers remain evidence validators | [paired gate](tools/foundation/paired-game.mjs), [evidence parser](tools/foundation/original-seed-contract.mjs) |
| \`original-clock-repeat\` | New random seeds repeat every selected native RGBA checkpoint, public log state and consumed X11 input at identical virtual presentation phases | [clock runner](tools/original/foundation-clock-run.py), [repeat gate](tools/original/foundation-clock-verify.py) |
| \`original-stage-compositing\` | Actual DOM inputs compare full 512×512 straight RGBA light/shadow targets at all ${maskCases.length} selected native checkpoints, including three seeds, controlled movement and coasting bomb presentations 121/123/127/131/133; every RGBA budget is zero | [browser cases](tools/tests/foundation.spec.ts), [mask comparator](tools/foundation/stage-masks.mjs), [native fixtures](references/isaac/foundation/stage-masks/manifest.json) |
| \`authored-mask-raster\` | Authored XML/PNG shadow dimensions feed general effect descriptors; every light/shadow has valid signed integer raster corners, compared in native Rust/WASM state and original GPU masks | [content extraction](engine/build.rs), [content catalogue](engine/src/content.rs), [effects](engine/src/effects.rs), [semantic guards](tools/foundation/seed-invariants.mjs) |
| \`authored-room-catalog\` | Decode all 13 pinned base STB sets, consume every byte, preserve weighted/special selectors and signed doors, reject malformed records and re-encode original bytes exactly; this is resource structure, not runtime spawning or floor parity | [decoder](tools/foundation/decode-rooms.py) |

Actual browser cases below are discovered from the executable browser scenario sources; adding or removing one requires regenerating this matrix.

| Executable browser case | Source |
| --- | --- |
${browserCases.map(({ name, source }) => `| ${name} | [${source.split('/').at(-1)}](${source}) |`).join('\n')}

Selected GPU-mask checkpoints below come directly from the native fixture manifest consumed by the browser gate. Each selected case compares every straight RGBA byte in both complete targets with zero tolerance. Controlled cases require actual X11 delivery, complete paired GL phases, binary/log/target hashes and independently repeated original evidence; this remains a graphics-target contract, not full original state or frame parity.

| Selected native GPU case | Required DOM input / native phase | Targets |
| --- | --- | --- |
${maskCases.map(({ id, browserTape, targets }) => `| \`${id}\` | ${browserTape.phase ?? `D${browserTape.movementHoldTicks}/release${browserTape.movementReleaseTicks}${browserTape.bombPresentation === undefined ? '' : ` → actual E; bomb presentation ${browserTape.bombPresentation}`}`} | |`).join('\n')}

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

Latest integrated immutable artifact \`6b2bdce75556c3fb23f13dfcd49e89067737a1f6c8809bcc811ca290c9f8fff7\`: 134/149 functional checkpoints and all eight original/browser transports pass; room combat and one randomized tear-count checkpoint fail. Finalized floors pass 5/5 and weapon scheduling passes 324/324 presentations. [Compact evidence](references/isaac/foundation/runtime-progress/verification.json) preserves hashes and failures; full live evidence is \`.cache/foundation/paired-integrated-1/report.json\`. Retained legacy state, pixels, differences and phase timings: \`.cache/foundation/verification/latest/report.json\` and checkpoint directories. Declared coverage here is not a cached pass result.
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
