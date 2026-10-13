# Browser foundation

Rust/WASM implementation: title → profile → New Run → character/seed → a
playable starting room. WASD moves, arrow keys shoot, E plants a bomb, and
Escape opens the animated pause screen. This remains an incomplete port.

Movement and tear inheritance now follow independently observed native f32
trajectories. Regular bombs have source-backed fuse, blast contact, damage,
knockback and immunity countdowns. Stage-banner text and the tested opaque pause
regions match independent original captures exactly. These scoped checks do
not establish whole-game state or pixel parity.

All 13 base room archives can be decoded into 5,314 authored rooms. Backdrop
selection matches independent original runs given an observed decoration seed;
production run-to-room seed routing and floor topology remain unresolved.
Enemies, traversal, item effects and progression still need runtime work. See
[the system map](docs/system-map.md) and [the test matrix](TEST_MATRIX.md).

Play the latest deployed build on the private [progress Site](https://isaac-browser-progress.anveio.chatgpt.site).
It shows the actual verification status. [Prepare](tools/sites/prepare.mjs) refreshes its
playable build and status; [publish source](tools/sites/push.mjs) preserves its [Site identity](tools/sites/site.json).

```sh
bash cloud/install.sh
bash cloud/start.sh                    # http://localhost:5173
npm run check                         # Includes original screenshot comparisons
npm run verify:foundation             # PNG/state/performance reports in .cache/
npm run test:original-seeds           # Fresh random-seed original/Rust/WASM probes
npm run test:original-clock           # Repeat original inputs/pixels on random seeds
npm run test:original-backdrop        # Fresh original GL geometry vs recovered generator
npm run test:rooms                    # All base room records and corruption guards
npm run matrix:check
```

Browser tests and the default verifier serve an immutable copy of the built
production artifact on an isolated loopback server. Run `npm run build` before
standalone browser tests. Reports identify the artifact by SHA256. Full-frame
original comparison failures remain failures in `npm run check`.

Ordinary play retains no command/state history. To capture diagnostic evidence,
pause automatic stepping and call `game.reset(seed, { trace: true })` in the
browser console. `game.trace()` exports a complete prefix and explicitly reports
any command or encoded-byte budget exhaustion; truncated traces cannot pass
replay verification.

The old port has been removed. The verifier compares independently captured
original frames, including movement and shooting. Failed pixel checks
stay red. Rust/WASM self-equality does not prove original-game state parity.

| Extend here | Source |
| --- | --- |
| Game commands, menu, movement and projectiles | [Rust systems](engine/src/lib.rs), [WASM bridge](web/engine.ts) |
| Native motion, tear inheritance and regular bombs | [Motion](engine/src/motion.rs), [bombs](engine/src/bombs.rs), [bomb evidence](docs/bomb-evidence.md) |
| Run introduction, pause and bomb animation | [Presentation](web/presentation.ts), [authored asset selection](tools/foundation/port-animations.txt) |
| Authored room catalogue and recovered backdrop | [STB decoder](tools/foundation/decode-rooms.py), [backdrop evidence](docs/backdrop-observations.md), [live verifier](tools/original/foundation-backdrop-verify.py) |
| Controlled original clock and consumed inputs | [Clock runner](tools/original/foundation-clock-run.py), [exact original repeats](tools/original/foundation-clock-verify.py), [scenario tapes](tools/original/foundation-clock-scenarios.py) |
| Actual input and command/state capture | [Browser entry](web/main.ts), [browser scenarios](tools/foundation/verify.mjs) |
| Room, actor, HUD and menu composition | [Renderer](web/renderer.ts), [menus](web/menu.ts), [authored animation playback](web/animation.ts), [GPU harness](web/graphics-contracts.ts) |
| Every divergence → owning system and fix tasks | [System/task registry](tools/foundation/systems.mjs) |
| Cases, gaps and source synchronization | [TEST_MATRIX.md](TEST_MATRIX.md), [generator](tools/foundation/matrix.mjs) |
| Exact original light/shadow buffer comparisons | [Stage mask cases](tools/foundation/stage-masks.mjs), [native fixtures](references/isaac/foundation/stage-masks/manifest.json) |
| Independent screenshot gates and differences | [Comparator](tools/foundation/compare.mjs) |
| Read-only original numeric-state probes | [Seed relationships](tools/original/state-observer-analyze.py), [position observation](tools/original/position-observer.py) |
| Original captures and observed state | [Native capture](tools/original/foundation-capture.py), [analysis](tools/original/foundation-analyze.py), [fixtures](references/isaac/foundation/baseline/manifest.json) |
| Random-seed invariants and replay | [Seed bank](tools/foundation/seeds.mjs), [native/WASM invariants](tools/foundation/seed-invariants.mjs), [browser tests](tools/tests/foundation.spec.ts) |
| Original seed and input coverage | [Original/Rust/WASM seed contract](tools/foundation/original-seed-contract.mjs), [native probes](tools/original/foundation-random-seeds.py), [input catalogue](tools/verification/input-catalog.json) |
| Authored asset extraction | [Extractor](tools/foundation/extract-animations.py) |

For each difference, identify the responsible system, record its task and failing
scenario, implement the general rule, then rerun the comparison. Do not patch a
screenshot, use per-seed behavior/maps, relax a gate, or skip a selected test.
Production seeds are ordinary inputs; invariants use random seeds recorded for
replay. Rust property tests print `FOUNDATION_TEST_SEED` for replay; browser/WASM
banks use [the seed runner](tools/foundation/seeds.mjs). Fixed seeds belong in reference fixtures.

Update the matrix with `npm run matrix:update` when checkpoints or system tasks
change. Pixel reports distinguish exact equality from measured native capture
variability. The original clock adapter records consumed X11 keys and repeats pixels exactly
at controlled presentation phases. A simulation-tick/full-state adapter is still
needed for full original parity. Keep reference files private.
