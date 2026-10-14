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
public inputs. Each agent must run relevant available tooling before merging,
check the affected behavior for regressions, and report the actual results and
remaining gaps. CI and automated merge gates are not required. Source/static
checks do not establish live original/browser acceptance. Selected paired runs
retain every predeclared contrived and recorded OS-random case and mismatch;
full-frame pixels remain explicit diagnostics with actual pass/fail results.
`test:e2e` runs every registered live suite, retaining every suite's result
before failing. Use `npm run test:e2e:paired -- --replay-cases PATH` to replay a recorded case. Standalone
browser checks require a fresh build. Native Rust/WASM self-equality proves
replay consistency, not original hidden-state or simulation-tick parity.

Coordinate file ownership across parallel agents. Shared temporary tooling and
RE notes belong in ignored `.cache/coordination/`; persist reusable tools and
the evidence needed to reproduce accepted behavior in the repository.

The user's Claude workflow owns bulk monster, level and character imports and
the associated import/testing refactors. Coordinate compatibility and source
contracts with that workflow and the integration owner; do not duplicate those
imports or start parallel catalogue/framework rewrites.

Keep changes small and send short-lived PRs directly to `main`. The integration
owner reviews exact PR heads, resolves conflicts and merges completed slices
frequently. Retire merged or superseded branches after preserving any unique
superseded commits; do not accumulate long-lived feature or aggregate branches.
