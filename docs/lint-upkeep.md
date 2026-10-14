# Lint and format upkeep

Lint and format checks do not gate CI or `npm run check`. A scheduled upkeep
agent runs them against the latest `main` and opens one short-lived PR to `main`
with whatever is needed to make them pass. Feature agents may run them locally,
but a lint or format failure is never a reason to block, revert or hold a slice.

## Checks

After `source cloud/env.sh`:

- `npm run lint` runs `cargo clippy --all-targets -- -D warnings` on `engine/`.
- `npm run format:check` runs `cargo fmt --check` on `engine/`;
  `npm run format` applies it.
- `npm run upkeep:check` runs both.

Formatting changes are mechanical: apply them without discussion.

## Lint findings

A lint rule can be wrong for a specific line, especially where reconstructed
code mirrors the original binary's arithmetic, field layout or control flow.
So each lint finding is settled with the agent that introduced it:

1. Find the introducing commit with `git log -L` or `git blame` on the flagged
   lines, then its PR (the merge commit or the PR's head branch).
2. If the commit or PR carries a `Claude-Session:` trailer or link, ask that
   session whether the pattern was deliberate before changing it, and follow its
   answer when it gives a reason.
3. Otherwise (for example a `codex/*` branch, whose history is not reachable),
   research the decision directly: read the surrounding module, the commit and
   PR descriptions, `docs/`, `TEST_MATRIX.md` and the reference evidence the code
   cites. Decide whether the flagged form is load-bearing for parity.
4. Fix the code when the lint is right and the fix preserves behavior. When the
   pattern is deliberate, add the narrowest `#[expect(clippy::rule, reason =
   "...")]` on the item, with the reason naming the evidence. Never use a
   crate-wide or module-wide allow, and never weaken the lint configuration to
   get green.

Each upkeep PR lists every finding, its introducing commit or PR, who was
consulted or what was researched, and whether it was fixed or allowed.
