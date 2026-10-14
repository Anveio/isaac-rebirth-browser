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

The only exception is the Antithesis deterministic simulation testing (DST)
harness in `engine/dst/` (`npm run test:dst`, `docs/dst.md`). Only that crate
may hold per-unit engine workloads and Antithesis properties; it never adds
`#[cfg(test)]` modules to `engine/src` and admits no other unit suite or
component harness anywhere. DST checks engine safety, determinism, snapshot
completeness, native/WASM agreement and documented contracts. It is not
original-game evidence and never replaces live e2e parity. It is an inner-loop
regression tool, not a merge gate. Fix or record every DST violation; never
weaken a property or its workload to pass.

Use `source cloud/env.sh` for the installed toolchain. `npm run check` builds and
tests an immutable production artifact against fresh original processes through
public inputs. CI runs contrived interactions and recorded OS-random scenarios;
selected source-observable functional contracts are blocking. Full-frame pixel
comparisons remain explicit diagnostics with their actual pass/fail results.
`test:e2e` runs every registered live suite, retaining every suite's result
before failing. CI discovers that same registry and runs the suites in separate
jobs with `fail-fast: false`; each job builds and checks its production artifact. Use `npm run test:e2e:paired -- --replay-cases PATH` to replay a recorded case. Standalone
browser checks require a fresh build. Native Rust/WASM self-equality proves
replay consistency, not original hidden-state or simulation-tick parity.

Coordinate file ownership across parallel agents. Shared temporary tooling and
RE notes belong in ignored `.cache/coordination/`; persist reusable tools and
the evidence needed to reproduce accepted behavior in the repository.

Keep changes small and send short-lived PRs directly to `main`. The integration
owner reviews exact PR heads, resolves conflicts and merges completed slices
frequently. Retire merged or superseded branches after preserving any unique
superseded commits; do not accumulate long-lived feature or aggregate branches.
