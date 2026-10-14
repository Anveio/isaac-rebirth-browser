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
`test:e2e` runs every registered live suite, retaining every suite's result
before failing. CI discovers that same registry and runs the suites in separate
jobs with `fail-fast: false`; each job builds and checks its production artifact. Use `npm run test:e2e:paired -- --replay-cases PATH` to replay a recorded case. Standalone
browser checks require a fresh build. Native Rust/WASM self-equality proves
replay consistency, not original hidden-state or simulation-tick parity.

Lint and format are not CI or `npm run check` gates. A scheduled upkeep agent
runs `npm run upkeep:check` on latest `main` and opens a PR with the fixes,
settling each lint finding with the agent that introduced it; see
`docs/lint-upkeep.md`. Prefer fixing over allowing; a deliberate exception is a
narrow `#[expect(clippy::..., reason = "...")]` naming its evidence.

Coordinate file ownership across parallel agents. Shared temporary tooling and
RE notes belong in ignored `.cache/coordination/`; persist reusable tools and
the evidence needed to reproduce accepted behavior in the repository.

Keep changes small and send short-lived PRs directly to `main`. The integration
owner reviews exact PR heads, resolves conflicts and merges completed slices
frequently. Retire merged or superseded branches after preserving any unique
superseded commits; do not accumulate long-lived feature or aggregate branches.
