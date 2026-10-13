# Base Rebirth browser port

Work from base Rebirth; keep Afterbirth and Afterbirth+ overlays separate. This
repository and the owner's original assets/captures remain private. The shipped
runtime is reconstructed Rust/WASM; native binaries are reference material.

Keep `TEST_MATRIX.md` synchronized with executable scenarios using
`npm run matrix:update`. Record implementation, original evidence, passing
contracts and unresolved parity separately. Preserve failing comparisons;
never replace a native reference with clone output or label a mismatch as equal.

Keep checked-in verification at the game level. Do not add Rust unit modules,
Node unit suites, Python schema suites or standalone component test harnesses.
Validators belong in runtime and live e2e tools. Put temporary kernel probes,
negative checks and reverse-engineering harnesses in ignored research cache.

Use `source cloud/env.sh` for the installed toolchain. `npm run check` builds and
tests an immutable production artifact against fresh original processes through
public inputs. CI runs contrived interactions and recorded OS-random scenarios;
selected source-observable functional contracts are blocking. Full-frame pixel
comparisons remain explicit diagnostics with their actual pass/fail results.
Use `npm run test:e2e -- --replay-cases PATH` to replay a recorded case. Standalone
browser checks require a fresh build. Native Rust/WASM self-equality proves
replay consistency, not original hidden-state or simulation-tick parity.

Coordinate file ownership across parallel agents. Shared temporary tooling and
RE notes belong in ignored `.cache/coordination/`; persist reusable tools and
the evidence needed to reproduce accepted behavior in the repository.
