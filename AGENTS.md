# Base Rebirth browser port

Work from base Rebirth; keep Afterbirth and Afterbirth+ overlays separate. This
repository and the owner's original assets/captures remain private. The shipped
runtime is reconstructed Rust/WASM; native binaries are reference material.

Keep `TEST_MATRIX.md` synchronized with executable scenarios using
`npm run matrix:update`. Record implementation, original evidence, passing
contracts and unresolved parity separately. Preserve failing comparisons;
never replace a native reference with clone output or label a mismatch as equal.

Keep checked-in verification at the game level: live e2e suites that compare
the fresh original with the production build through public inputs. Do not
check in unit tests (Rust `#[test]`, Node/Python suites, component harnesses)
or scripts that assert checksums of outputs; `npm run test:policy` rejects
them. Put temporary probes and reverse-engineering harnesses in ignored
`.cache/`.

Use `source cloud/env.sh` for the installed toolchain. Every suite is
registered in `tools/e2e/suites.mjs` and mixes contrived cases with OS-random
fuzz cases; read `tools/e2e/README.md` before adding or changing one. Run
`npm run test:e2e -- <suite>` for the suites your change touches before opening
a PR, and replay a failure with the command the runner prints. Pull requests run
only `npm run check:build`; all live suites run after each merge to main and
every six hours. A red main run is a regression to fix next. Full-frame pixel
comparisons remain explicit diagnostics. Native Rust/WASM self-equality proves
replay consistency, not original parity.

Coordinate file ownership across parallel agents. Shared temporary tooling and
RE notes belong in ignored `.cache/coordination/`; persist reusable tools and
the evidence needed to reproduce accepted behavior in the repository.

Keep changes small and send short-lived PRs directly to `main`. The integration
owner reviews exact PR heads, resolves conflicts and merges completed slices
frequently. Retire merged or superseded branches after preserving any unique
superseded commits; do not accumulate long-lived feature or aggregate branches.
