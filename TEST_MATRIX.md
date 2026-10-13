# Foundation test matrix

Generated from the executable checkpoints in [verify.mjs](tools/foundation/verify.mjs). Run `npm run matrix:update` after adding a checkpoint; `npm run matrix:check` rejects drift. The previous port and its scenarios have been removed.

`npm run check` builds the Rust/WASM foundation, runs native unit, real-keyboard browser and fresh random-seed original tests, then compares its screenshots against two independent original captures. Every selected check executes and failures remain failures. There are no skip or expected-failure paths.

| Executable checkpoint | Required behavior | Source |
| --- | --- | --- |
| `title` | Boot to title; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `profile` | Choose a fresh profile; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `main` | New Run menu; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `character` | Isaac, normal mode; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `seed-entry` | Enter observed T1PK 6Y0W seed; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `character-seeded` | Return to character screen with committed seed; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `gameplay-start` | Enter playable tutorial room; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `movement` | Hold D, release, settle; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `shooting` | Hold Right, release, capture live tear; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `after-fire` | Release fire and expire tears; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |
| `command-state` | Every actual DOM key/text/tick command matches native Rust and browser WASM state | [replay.rs](engine/src/bin/replay.rs), [browser tests](tools/tests/foundation.spec.ts) |
| `pause-release` | Pause freezes the run; released keys cannot stick | [browser tests](tools/tests/foundation.spec.ts) |
| `random-seed-invariants` | New OS-random seeds replay identical movement, every shooting direction, pause and release state in native Rust and WASM | [invariants](tools/foundation/seed-invariants.mjs), [seed bank/replay](tools/foundation/seeds.mjs) |
| `random-seed-browser` | Actual keys enter ordinary and manual-seed runs; eligibility, movement, shots and generated room images respond to the supplied seeds | [browser tests](tools/tests/foundation.spec.ts) |
| `production-entropy` | Browser boot uses crypto entropy; invalid explicit seed inputs reject without changing state | [browser tests](tools/tests/foundation.spec.ts) |
| `original-random-seed-contract` | Fresh random native runs accept the code/value and report the Stage 1 seed predicted by Rust/WASM; every selected bank case executes and has real log/PNG evidence | [adapter](tools/foundation/original-seed-contract.mjs), [adapter tests](tools/foundation/original-seed-contract.test.mjs), [native probes](tools/original/foundation-random-seeds.py) |
| `original-clock-repeat` | New random seeds repeat every selected native RGBA checkpoint, public log state and consumed X11 input at identical virtual presentation phases | [clock runner](tools/original/foundation-clock-run.py), [repeat gate](tools/original/foundation-clock-verify.py), [ABI tests](tools/original/foundation-clock-gate-checks.py), [negative tests](tools/original/foundation-clock-verify-checks.py) |
| `original-graphics-frame-contract` | Pair captures with complete public-graphics frames; reject nested/incomplete/reused frames, post-presentation cache draws and unavailable geometry | [observer regression tests](tools/original/foundation-gl-observe-checks.py) |
| `original-stage-compositing` | Actual DOM inputs produce full512×512 straightRGBA light/shadow targets equal to independent original targets, across three seeds and a movement case | [browser cases](tools/tests/foundation.spec.ts), [mask comparator](tools/foundation/stage-masks.mjs), [native fixtures](references/isaac/foundation/stage-masks/manifest.json) |
| `authored-mask-raster` | Authored XML/PNG shadow dimensions feed general effect descriptors; every light/shadow has valid signed integer raster corners, compared in native Rust/WASM state and original GPU masks | [content extraction](engine/build.rs), [content catalogue](engine/src/content.rs), [effects](engine/src/effects.rs), [semantic guards](tools/foundation/seed-invariants.mjs) |
| `comparison-guards` | Wrong/blank graphics, missing references and missing state cannot pass | [compare tests](tools/foundation/compare.test.mjs), [reference tests](tools/foundation/verify.test.mjs) |

Actual browser cases below are discovered from the executable browser scenario sources; adding or removing one requires regenerating this matrix.

| Executable browser case | Source |
| --- | --- |
| actual keyboard opens a seeded run, moves and shoots with exact native Rust/WASM state | [foundation.spec.ts](tools/tests/foundation.spec.ts) |
| game pause freezes movement and release prevents stuck keyboard input | [foundation.spec.ts](tools/tests/foundation.spec.ts) |
| releasing a movement key while Control is held clears the actual DOM input state | [foundation.spec.ts](tools/tests/foundation.spec.ts) |
| automatic simulation catch-up advances ticks without repeatedly presenting the full canvas in one animation frame | [foundation.spec.ts](tools/tests/foundation.spec.ts) |
| OS-random seeds support ordinary and manually seeded browser runs with native/WASM replay | [foundation.spec.ts](tools/tests/foundation.spec.ts) |
| production boot consumes real crypto entropy and invalid explicit reset seeds reject atomically | [foundation.spec.ts](tools/tests/foundation.spec.ts) |
| original GPU shadow and light masks match all complete native targets for independent seeds and movement | [foundation.spec.ts](tools/tests/foundation.spec.ts) |
| GPU masks preserve straight RGBA blending, zero-alpha rejection and unused viewport padding | [graphics.spec.ts](tools/tests/graphics.spec.ts) |
| GPU screen presentation restores nested targets and keeps the logical top row at the top | [graphics.spec.ts](tools/tests/graphics.spec.ts) |

Node cases follow the same source glob as `test:verification`. The index rejects disabled or undocumented dynamic registrations.

| Executable verification case | Source |
| --- | --- |
| browser graphics profiles select one shared backend without a silent profile fallback | [browser.test.mjs](tools/foundation/browser.test.mjs) |
| graphics reports identify actual GPU data and reject flags-only or mismatched backend claims | [browser.test.mjs](tools/foundation/browser.test.mjs) |
| pixel fidelity compares decoded RGBA and reports each changed pixel | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| nearest scaling and exact crops preserve source pixels without filtering | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| reference comparisons require independent calibration samples and critical regions | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| repeatable blank or wrong artwork cannot pass by matching the clone to itself | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| native background variability cannot hide incorrect actor pixels in a stable critical region | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| actor diagnostic compares original foreground RGBA positions, not matching color totals | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| projectile matching rejects missing sprites, wrong shape/color and positions outside native variation | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| nonexact frames within measured native variability remain explicitly nonexact | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| state equality detects hidden fields, missing fields and numeric drift | [compare.test.mjs](tools/foundation/compare.test.mjs) |
| an offscreen light gradient preserves OpenGL's observed alpha-squared channel | [compositor.test.mjs](tools/foundation/compositor.test.mjs) |
| a destination-color light pass modulates RGB while retaining source alpha independently | [compositor.test.mjs](tools/foundation/compositor.test.mjs) |
| linear texture sampling interpolates straight RGBA rather than premultiplied hidden color | [compositor.test.mjs](tools/foundation/compositor.test.mjs) |
| original clock scenario discovery executes its source and rejects deleted, disabled or stale tape metadata | [matrix.test.mjs](tools/foundation/matrix.test.mjs) |
| original scenario index rejects unknown systems and missing executable input or capture contracts | [matrix.test.mjs](tools/foundation/matrix.test.mjs) |
| matrix checking rejects changes in executable native scenarios until the source-derived table is regenerated | [matrix.test.mjs](tools/foundation/matrix.test.mjs) |
| original random-seed contracts reject empty, incomplete, skipped and duplicate banks | [original-seed-contract.test.mjs](tools/foundation/original-seed-contract.test.mjs) |
| actual log parsing requires unique initialized seed, stage and player records | [original-seed-contract.test.mjs](tools/foundation/original-seed-contract.test.mjs) |
| missing/failed original evidence stays red and every selected seed is attempted | [original-seed-contract.test.mjs](tools/foundation/original-seed-contract.test.mjs) |
| PNG decoder preserves straight RGBA and transparent RGB through every standard row filter | [png.test.mjs](tools/foundation/png.test.mjs) |
| Authored texture PNGs match the independent decoder while keeping RGB beneath alpha zero | [png.test.mjs](tools/foundation/png.test.mjs) |
| Malformed PNG dimensions and chunks fail explicitly | [png.test.mjs](tools/foundation/png.test.mjs) |
| source discovery reflects added/deleted scenarios and rejects disabled or undocumented registrations | [scenario-index.test.mjs](tools/foundation/scenario-index.test.mjs) |
| entropy seeds or a saved list replay native Rust/WASM movement, shooting and pause invariants | [seed-invariants.test.mjs](tools/foundation/seed-invariants.test.mjs) |
| semantic assertions reject aliased controls, incoherent world effects and invalid room geometry | [seed-invariants.test.mjs](tools/foundation/seed-invariants.test.mjs) |
| saved random-seed lists replay exactly and invalid or duplicate coverage is rejected | [seed-invariants.test.mjs](tools/foundation/seed-invariants.test.mjs) |
| accepted unsigned seed boundaries execute the full tape without a preset fallback | [seed-invariants.test.mjs](tools/foundation/seed-invariants.test.mjs) |
| native stage-mask fixtures require actual seed logs and complete independent GPU target data | [stage-masks.test.mjs](tools/foundation/stage-masks.test.mjs) |
| raw stage-mask equality detects transparent RGB differences and rejects partial or missing targets | [stage-masks.test.mjs](tools/foundation/stage-masks.test.mjs) |
| missing original evidence is an executed pipeline failure, never a skip or clone-only pass | [verify.test.mjs](tools/foundation/verify.test.mjs) |
| failed pixel regions map to source-indexed tasks without asserting a fabricated cause | [verify.test.mjs](tools/foundation/verify.test.mjs) |
| a corrupt checkpoint remains a failure while later independent pixel assertions still execute | [verify.test.mjs](tools/foundation/verify.test.mjs) |
| reference fixtures reject missing checkpoints, wrong seed and shared run directories | [verify.test.mjs](tools/foundation/verify.test.mjs) |

Original presentation scenarios below are discovered by executing [foundation-clock-scenarios.py --list](tools/original/foundation-clock-scenarios.py). Their captures, input events and interval counts come from the actual validated tapes. The index rejects missing required scenarios, malformed tapes and unknown system tasks. These are original-only reference scenarios; their presence does not assert that the browser already matches them. Replay a seed with `python3 tools/original/foundation-clock-scenarios.py --scenario ID --seed CODE`, then run the emitted capture command.

| Executable original scenario | Required behavior / systems | Tape checkpoints / intervals | Source |
| --- | --- | --- | --- |
| `menu-movement-fire` | Fresh menus and seeded run, movement, shooting and release checkpoints; `input-determinism, menu-navigation, locomotion, projectile-physics` | `intro, title, profile, main, character, seed-entry, character-seeded, gameplay-start, movement, shooting, after-fire`; 1317 presentation intervals, 34 input events | [foundation-clock-scenarios.py](tools/original/foundation-clock-scenarios.py) |
| `menu-run-introduction` | Preserve the stage banner checkpoint, advance its authored introduction without input, then move and shoot; `input-determinism, menu-navigation, run-introduction, entity-animation, locomotion, projectile-physics` | `intro, title-ready, title, profile, main, character, seed-entry, character-seeded, gameplay-start, gameplay-unpaused, movement, shooting, after-fire`; 1374 presentation intervals, 34 input events | [foundation-clock-scenarios.py](tools/original/foundation-clock-scenarios.py) |
| `motion-trajectory` | Every presentation during D30/release30/Right55, with public graphics trajectory measurement; `locomotion, entity-animation, projectile-physics, projectile-height-range, graphics-composition` | `gameplay-start, movement, shooting-release`; 1292 presentation intervals, 34 input events | [foundation-clock-scenarios.py](tools/original/foundation-clock-scenarios.py) |
| `motion-holdout` | Independent direction/length holdout: A40, W20, D30, D+S20 and upward shots; `locomotion, entity-animation, projectile-physics, projectile-height-range, graphics-composition` | `gameplay-start, left-40, up-20, right-30, diagonal-20, up-after-fire`; 1486 presentation intervals, 42 input events | [foundation-clock-scenarios.py](tools/original/foundation-clock-scenarios.py) |
| `projectile-inheritance` | Compare stationary shots, shots during motion, and shots while coasting after release; `locomotion, projectile-physics, projectile-height-range, entity-animation` | `gameplay-start, stationary-shots, moving-shots, coasting-shots, after-fire`; 1405 presentation intervals, 36 input events | [foundation-clock-scenarios.py](tools/original/foundation-clock-scenarios.py) |

The original uses seed **T1PK 6Y0W**, fresh profiles, and a 960×540 window. Captures and observed log/input metadata live in [baseline](references/isaac/foundation/baseline/manifest.json) and [repeat](references/isaac/foundation/repeat/manifest.json). They are comparison evidence; the renderer composes authored sprites and never draws a captured game frame. [Capture](tools/original/foundation-capture.py) and [save-reference.py](tools/foundation/save-reference.py) refresh these fixtures from the runnable original.

Pixel gates compare decoded RGBA over the whole frame, HUD, actor and projectile regions. Any tolerance is measured from the independent native captures; it is not a freely chosen similarity threshold. Reports distinguish exact pixels from native capture variability. Legacy screenshot fixtures use wall-clock keys/SIGSTOP. The newer clock adapter controls presented frames and main-thread time, disables X11 autorepeat and records actual consumed key events; neither adapter proves native simulation ticks or hidden-state parity. Native process CPU/RSS and browser draw/encoding timings remain separate.

## Divergence tasks by game system

Generated from [systems.mjs](tools/foundation/systems.mjs). Each failed image region links to these investigation tasks, state fields and next tests. A region can have several candidate causes; attribution is not proof. Fix the responsible system and rerun its reference scenario before closing a task. Do not substitute a renderer offset, per-seed tile map or screenshot-derived correction image for a missing system.

| Task | Owning system / state | Required next test |
| --- | --- | --- |
| `seed-codec` | Run seed validation and encoding; `runSeed, runSeedValue` | Generate random u32 seeds, round-trip them, and confirm independent native acceptance/log readback. |
| `rng-streams` | Independent random streams; `stageSeed, room.appearance.floorQuads, room.appearance.wallQuads` | Predict stage, level-generation draw consumption, room initialization, decoration and FX outputs for held-out randomly generated seeds. |
| `room-template` | Room geometry and authored template assembly; `room.appearance.floorQuads, room.appearance.wallQuads, room.appearance.origin, room.appearance.tutorial` | Compare geometry/texture boundaries across different original rooms and seeds. |
| `room-decoration` | Seeded room texture variation; `stageSeed, room.appearance.floorQuads, room.appearance.wallQuads, room.appearance.generation` | Validate generic variant selection against held-out native seeds; prohibit captured per-seed tile maps. |
| `room-connectivity` | Door connectivity and exploration; `room.appearance.doors, minimap.cells` | Check doors, neighboring map cells and discovery state against original topology and transitions. |
| `run-introduction` | Run entry, stage banner and tutorial presentation; `screen, runTick, room.appearance.tutorial` | Capture run entry, stage banner timing, gameplay enablement and tutorial visibility across random seeds. |
| `stage-effects` | Biome lighting, overlays and ambient effects; `stageEffects, room.appearance.fx` | Change stage/time and verify authored shading, overlay selection and parallax independently of actor movement. |
| `menu-navigation` | Menu state and input navigation; `screen, profile, mainSelection, characterSelection` | Use real keys to navigate forward/backward and choose profiles and characters across random seeds. |
| `menu-animation` | Authored UI animation playback; `menuAnimationTick` | Verify animation clock reset, interpolation and looping at multiple points during each transition. |
| `character-roster` | Profile unlocks and selectable character definitions; `profiles, characterCatalog` | Derive visible/locked/selectable characters from profile progress and authored player definitions. |
| `character-carousel` | Selection-dependent menu layout and depth; `characterSelection, characterCatalog` | Exercise every visible selection and compare projection, scale and depth ordering. |
| `profile-progress` | Profile progress and menu widgets; `profiles` | Compare fresh and progressed profiles, save drawings, completion marks and win streak. |
| `health-inventory` | Health containers and inventory HUD; `player.hp, inventory` | Modify health and inventory through gameplay and require the corresponding HUD state. |
| `run-eligibility` | Seeded-run achievement eligibility; `achievementEligibility, seededRun` | Compare manual-seed and ordinary-run eligibility and the native no-achievements badge. |
| `world-projection` | World coordinates, camera and raster boundaries; `player.x, player.y, player.vx, player.vy, stageEffects` | Verify world position and velocity against read-only numeric observations, then verify camera projection and signed sprite/mask corner rounding at multiple display scales. |
| `locomotion` | Movement integration and collision; `player.x, player.y, player.vx, player.vy` | Random-seed direction/opposite/diagonal/release/wall tests plus independently held-out native world-coordinate traces. |
| `projectile-physics` | Tear emission, cadence, motion and impact; `tears, fireCooldown, impacts` | Random-seed shots in every direction, alternating eyes, wall impacts and native projectile patches. |
| `projectile-height-range` | Projectile ground motion, height, gravity and range; `tears` | Compare multi-frame tear/shadow trajectories and expiration across directions and flight distances. |
| `entity-animation` | Facing and authored head/body/projectile playback; `player.facing, player.bodyFacing, player.headFrame, player.walkFrame` | Verify animation timelines while moving, firing, releasing and pausing. |
| `graphics-composition` | Texture sampling, tint and alpha compositing; `stageEffects` | Compare generic authored primitive blends and native-resolution nearest/linear sampling against observed output. |
| `input-determinism` | Input scheduling and complete state observation; `pressed, tick, runTick` | Apply the same tape at verified native simulation ticks and compare complete future-affecting state. |

Production behavior is seed-independent: seeds enter a general codec and generators. Seed-specific expected outputs belong only in reference fixtures. Invariants use new OS-random seeds, record those values for replay, and verify the same tape in native Rust and WASM. Randomness does not excuse missing original comparisons or permit seed-specific implementation branches.

## Required extensions

These remain missing original-observed scenarios, not implemented behavior or passing tests. Add a setup, input tape and assertions to the executable verifier before claiming coverage.

| Missing area | Required scenarios |
| --- | --- |
| `original-tick-state` | Original simulation-tick scheduling and complete hidden-state observation |
| `core-gameplay` | Combat rooms, enemies, health changes, bombs, pickups and item effects |
| `all-inputs` | Equipped active items/cards/pills, drops, map/restart, remapping, mouse and controllers |
| `hidden-progression` | RNG streams, room/floor generation, item pools, devil/angel chances and progression |
| `metagame` | Other characters/modes, profiles, unlocks, achievements and save/continue |
| `numerics-performance` | Observed native floating-point edges, long runs and comparable simulation/render performance |

Extend the [input catalogue](tools/verification/input-catalog.json), [Rust commands/state](engine/src/lib.rs), [asset renderer](web/renderer.ts), [authored animation loader](web/animation.ts) and [browser scenario](tools/foundation/verify.mjs) together. Register discoveries before implementing them, keep failing comparisons, and code to green. Do not replace a reference with clone output, weaken a pixel gate, omit a selected checkpoint, or call native Rust/WASM self-equality original-game parity.

Latest actual results, state, pixels, differences and phase timings: `.cache/foundation/verification/latest/report.json` and checkpoint directories. Declared coverage here is not a cached pass result.
