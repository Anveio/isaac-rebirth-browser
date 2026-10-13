# Browser foundation

Rust/WASM implementation: title → profile → New Run → character/seed → a
playable starting room. WASD moves, arrow keys shoot, E plants a bomb, and
Escape opens the animated pause screen. This remains an incomplete port.

Movement and tear inheritance now follow independently observed native f32
trajectories. Regular bombs have source-backed fuse, blast contact, damage,
knockback and immunity countdowns. Stage-banner text and the tested opaque pause
regions match independent original captures exactly. These scoped checks do
not establish whole-game state or pixel parity.

Title animation, the initial f32 fade, looping and retained reentry clock now
match seven complete original frames and ten observed phase checkpoints.
Authored geometry also matches 75 native title and 42 explosion-smoke quads.
The legacy title comparison remains a separate failing gate.

All 13 base room archives can be decoded into 5,314 authored rooms. Backdrop
selection matches independent original runs given an observed decoration seed;
normal Basement run-to-room routing, finalized floor topology and ordinary
room traversal now have live original/browser coverage. NPCs, items, later floors
and progression still need runtime work. See
[the system map](docs/system-map.md) and [the test matrix](TEST_MATRIX.md).
Room-growth and weighted-template kernels now have bounded diagnostic tools:
44 template IDs match ten original runs with supplied native initialization
seeds. Special-room resizing and production seed routing remain open.

Play the latest deployed build on the private [progress Site](https://isaac-browser-progress.anveio.chatgpt.site).
It shows the actual verification status. [Prepare](tools/sites/prepare.mjs) refreshes its
playable build and status; [publish source](tools/sites/push.mjs) preserves its [Site identity](tools/sites/site.json).

```sh
bash cloud/install.sh
bash cloud/start.sh                    # http://localhost:5173
npm run check                         # Build and gate fresh original/browser games
npm run test:e2e -- --list            # Registered live suites (tools/e2e/suites.mjs)
npm run test:e2e -- keys doors        # Run selected suites; omit names to run all
npm run test:e2e -- keys --scenarios PATH # Replay a recorded scenarios.json
npm run verify:foundation             # Strict legacy PNG/state diagnostics; mismatches exit nonzero
npm run assets:rooms                  # Decode all base room records
npm run matrix:check
```

Browser tests and the default verifier serve an immutable copy of the built
production artifact on an isolated loopback server. Run `npm run build` before
standalone browser tests. Reports identify the artifact by SHA256.
Source-observable functional failures block `npm run check`. Every paired
checkpoint also retains a complete pixel comparison and difference image;
pixel mismatches stay reported as failures without blocking the default
functional gate. `verify:foundation` remains a strict pixel diagnostic.

Live suites execute both games with public inputs, fresh original profiles and
a copied production build. Each mixes contrived cases with OS-random fuzz cases
recorded for exact replay; see [the e2e guide](tools/e2e/README.md). Pull
requests run the fast [build gate](.github/workflows/build.yml); every suite runs
in its own job after each push to main and every six hours
([live e2e](.github/workflows/live-games.yml)). Unit suites and output-checksum
scripts are rejected by `npm run test:policy`.

Ordinary play retains no command/state history. To capture diagnostic evidence,
pause automatic stepping and call `game.reset(seed, { trace: true })` in the
browser console. `game.trace()` exports a complete prefix and explicitly reports
any command or encoded-byte budget exhaustion; truncated traces cannot pass
replay verification.
ABI 2 also carries an independent `bootEpochSeed` through state and traces.
Production uses epoch seconds; diagnostic resets accept it explicitly and
default to zero. It is separate from the run seed. Shared MT and projectile
kernels are source-verified diagnostics awaiting complete runtime routing.

The old port has been removed. The verifier compares independently captured
original frames, including movement and shooting. Failed pixel checks
stay red. Rust/WASM self-equality does not prove original-game state parity.

| Extend here | Source |
| --- | --- |
| Game commands, menu, movement and projectiles | [Rust systems](engine/src/lib.rs), [WASM bridge](web/engine.ts) |
| Native motion, tear inheritance and regular bombs | [Motion](engine/src/motion.rs), [bombs](engine/src/bombs.rs), [bomb evidence](docs/bomb-evidence.md) |
| Run introduction, pause and bomb animation | [Presentation](web/presentation.ts), [authored asset selection](tools/foundation/port-animations.txt) |
| Native title clock, fade and f32 geometry | [Animation evidence](docs/animation-evidence.md), [title browser gate](tools/tests/title.spec.ts), [geometry kernel](web/animation-math.ts) |
| Supplied-seed floor growth and weighted templates | [Layout evidence](docs/room-layout-evidence.md), [growth](engine/src/layout.rs), [templates](engine/src/room_templates.rs), [source observer](tools/original/foundation-layout-observer.py) |
| Authored room catalogue and recovered backdrop | [STB decoder](tools/foundation/decode-rooms.py), [backdrop evidence](docs/backdrop-observations.md), [live paired gate](tools/foundation/paired-game.mjs) |
| Controlled original clock and consumed inputs | [Clock runner](tools/original/foundation-clock-run.py), [exact original repeats](tools/original/foundation-clock-verify.py), [scenario tapes](tools/original/foundation-clock-scenarios.py) |
| Actual input and command/state capture | [Browser entry](web/main.ts), [browser scenarios](tools/foundation/verify.mjs) |
| Room, actor, HUD and menu composition | [Renderer](web/renderer.ts), [menus](web/menu.ts), [authored animation playback](web/animation.ts) |
| Every divergence → owning system and fix tasks | [System/task registry](tools/foundation/systems.mjs) |
| Cases, gaps and source synchronization | [TEST_MATRIX.md](TEST_MATRIX.md), [generator](tools/foundation/matrix.mjs) |
| Exact original light/shadow buffer comparisons | [Stage mask cases](tools/foundation/stage-masks.mjs), [native fixtures](references/isaac/foundation/stage-masks/manifest.json) |
| Independent screenshot gates and differences | [Comparator](tools/foundation/compare.mjs) |
| Read-only original numeric-state probes | [Seed relationships](tools/original/state-observer-analyze.py), [position observation](tools/original/position-observer.py) |
| Original captures and observed state | [Native capture](tools/original/foundation-capture.py), [analysis](tools/original/foundation-analyze.py), [fixtures](references/isaac/foundation/baseline/manifest.json) |
| Random-seed invariants and replay | [Seed bank](tools/foundation/seeds.mjs), [browser scenario helpers](tools/foundation/seed-invariants.mjs), [browser tests](tools/tests/foundation.spec.ts) |
| Original seed and input coverage | [Original seed evidence parser](tools/foundation/original-seed-contract.mjs), [native probes](tools/original/foundation-random-seeds.py), [input catalogue](tools/verification/input-catalog.json) |
| Authored asset extraction | [Extractor](tools/foundation/extract-animations.py) |

For each difference, identify the responsible system, record its task and failing
scenario, implement the general rule, then rerun the comparison. Do not patch a
screenshot, use per-seed behavior/maps, relax a gate, or skip a selected test.
Production seeds are ordinary inputs; invariants use random seeds recorded for
replay. Live paired cases retain their OS-random seed and complete action tape in
`case.json` and `tape.json`; `--replay-cases` uses the recorded choices. Fixed
seeds belong in contrived interactions and reference fixtures.

Update the matrix with `npm run matrix:update` when checkpoints or system tasks
change. Pixel reports distinguish exact equality from measured native capture
variability. The original clock adapter records consumed X11 keys and repeats pixels exactly
at controlled presentation phases. A simulation-tick/full-state adapter is still
needed for full original parity. Keep reference files private.
