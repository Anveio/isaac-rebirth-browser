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
Do not check in scripts that assert checksums of outputs or fixtures; the
original binary pin lives in `tools/e2e/lib/original.mjs`. `npm run
test:policy` (part of `check:build`) rejects unit tests and stray checksums.

The only exception, authorized by the repository owner, is the Antithesis
deterministic simulation testing (DST) harness in `engine/dst/`
(`npm run test:dst`, `docs/dst.md`). Only that crate may hold per-unit engine
workloads and Antithesis properties; it never adds test attributes to
`engine/src` and admits no other unit suite or component harness anywhere. DST
checks engine safety, determinism and documented contracts. It is not
original-game evidence, never replaces live e2e parity and is an inner-loop
regression tool, not a merge gate: `check:build` does not run it. Fix or
record every DST violation; never weaken a property or its workload to pass.

Every live suite is registered in `tools/e2e/suites.mjs` and mixes contrived
cases with OS-random fuzz cases; read `tools/e2e/README.md` before adding or
changing one. Run `npm run test:e2e -- <suite>` for the suites your change
touches; a failed suite prints the command that replays its recorded cases.

Use `source cloud/env.sh` for the installed toolchain. `npm run check` builds and
tests an immutable production artifact against fresh original processes through
public inputs. Live suites run contrived interactions and recorded OS-random scenarios;
selected source-observable functional contracts are blocking. Full-frame pixel
comparisons remain explicit diagnostics with their actual pass/fail results.
`test:e2e` runs every registered live suite, retaining every suite's result
before failing.

This repository deliberately has no CI. Nothing runs on pull requests or
after merges, so every agent is responsible for making sure its own change
does not regress the codebase, using the tooling available here: run
`npm run check:build`, then the live suites your change touches (and any
suite whose driver, comparator or shared code you edited), and compare their
results against main before opening or merging a PR. Never merge a change
that turns a passing comparison red or breaks the build, and fix any
regression you find on main before starting new work. A dedicated agent
workstream keeps main playable and fixes regressions, but that is a backstop,
not a substitute for checking your own change. Use `npm run test:e2e:paired -- --replay-cases PATH` to replay a recorded case. Standalone
browser checks require a fresh build. Native Rust/WASM self-equality proves
replay consistency, not original hidden-state or simulation-tick parity.

Coordinate file ownership across parallel agents. Shared temporary tooling and
RE notes belong in ignored `.cache/coordination/`; persist reusable tools and
the evidence needed to reproduce accepted behavior in the repository.

Keep changes small and send short-lived PRs directly to `main`. The integration
owner reviews exact PR heads, resolves conflicts and merges completed slices
frequently. Retire merged or superseded branches after preserving any unique
superseded commits; do not accumulate long-lived feature or aggregate branches.
