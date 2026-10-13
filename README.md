# Browser foundation

Fresh Rust/WASM implementation: title → profile → New Run → character/seed → a
playable starting room. WASD moves; arrow keys shoot; Escape pauses.

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
npm run matrix:check
```

The old port has been removed. The verifier compares independently captured
original frames, including movement and shooting. Failed pixel checks
stay red. Rust/WASM self-equality does not prove original-game state parity.

| Extend here | Source |
| --- | --- |
| Game commands, menu, movement and projectiles | [Rust systems](engine/src/lib.rs), [WASM bridge](web/engine.ts) |
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
