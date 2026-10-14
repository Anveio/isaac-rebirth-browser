# Live e2e suites

Each suite runs the fresh original game and the immutable production build with
the same public inputs (keys, text, ticks, seeds) and compares what each game
observably does. Native observations never enter WASM.

Every suite mixes two kinds of cases:

- **Contrived**: fixed seeds, phases and routes that pin known edge cases and
  regressions.
- **Fuzz**: OS-random seeds (and sometimes random actions or boot epochs)
  drawn on every run. These catch code that only works for the seeds it was
  developed against. A fuzz failure is a real bug until shown otherwise. The
  run records the case, so it can be replayed exactly.

## Running

```sh
npm run test:e2e -- --list                  # suite ids
npm run test:e2e -- keys                    # one suite
npm run test:e2e -- keys doors --jobs 2     # several, in parallel (logs go to files)
npm run test:e2e -- keys --count 4          # extra flags override the suite's defaults
npm run test:e2e                            # everything
```

The runner rebuilds `dist/` when `web/` or `engine/` sources are newer than it
(use `--no-build` to skip). It builds the native `floor` generator, runs each
suite under its own virtual display, and writes `.cache/e2e/run-*/report.json`
and one log per suite. A failing suite prints its replay command:

```sh
npm run test:e2e -- keys --scenarios .cache/e2e/run-…/keys/scenarios.json
npm run test:e2e -- paired --replay-cases .cache/e2e/run-…/paired/<case>/case.json
```

## No CI

There is deliberately no CI (see [AGENTS.md](../../AGENTS.md)). Before you open
or merge a change, run `npm run check:build` and the suites it touches, and
compare their results against main. A dedicated agent workstream runs the
suites on main and fixes regressions, but it is a backstop, not a substitute
for checking your own change.

## Adding or changing a suite

1. Register it in [`suites.mjs`](suites.mjs): an id, a driver, default args and
   a one-line `about`. That is the whole registry: the runner, CI and
   `--list` all read it.
2. Reuse an existing driver where one fits:
   - [`lib/room-frames.mjs`](lib/room-frames.mjs) for routes compared on
     every frame. The suite supplies only its own `compare` and `finish`
     checks; [`keys.mjs`](keys.mjs) is the smallest example;
   - `traversal.mjs` (or a thin wrapper such as `doors.mjs`) with a scenario
     helper for multi-room routes;
   - `runPairedGate` from `tools/foundation/paired-game.mjs` for paired tapes,
     or a mode flag on an existing driver (`items.mjs --stat-cache`).
3. Drive the page through [`lib/game.mjs`](lib/game.mjs): `stepSnapshot(page)`
   for per-frame comparisons (one round trip carrying the engine's JSON),
   `step(page, n)` when no state is needed. Use [`lib/cli.mjs`](lib/cli.mjs)
   for flags and output directories.
4. Give it contrived cases and a `--count` of fuzz cases. Write
   `scenarios.json` to the output directory and accept `--scenarios` to replay
   it.
5. Record each mismatch as a field failure, with the presentation, actual and
   expected values. Keep comparing after the first mismatch so one run shows
   the whole divergence.

## Adding an entity

Most room entities need no new driver. Add a spec in
[`entities/`](entities/) and it becomes the `entity-<name>` suite:

```js
export default { type: 12, variant: 0, about: 'Horf rooms: ...', count: 2, required: ['admission'] };
```

[`entity-scenarios.py`](../original/entity-scenarios.py) finds OS-random rooms
next to the start room that admit the entity. Both games then walk in and shoot
around. [`entity-state-pair.mjs`](../foundation/entity-state-pair.mjs) compares
the shared NPC, room, door and projectile fields exactly. The whole workflow is
in [docs/content-pipeline.md](../../docs/content-pipeline.md).

## Large native captures

`npm run test:e2e:paired -- --archive-native-evidence` retains the complete
native manifest as `manifest.json.gz` with decompressed and archive hashes. Its
separate progress journal commits each actual input, acknowledgement,
observation and capture in a closed gzip member. Interrupted journals retain
their uncommitted tail and identify the verified byte prefix; they cannot pass
as complete runs.

## What not to check in

- Unit tests of any kind: Rust `#[cfg(test)]`/`#[test]`, Node or Python test
  suites, or standalone component harnesses. Behavior is verified at the game
  level against the original.
- Scripts that assert checksums of outputs (golden hashes of states, traces or
  frames). Compare behavior against the live original instead.
- Copies of the original binary pin. Import `ORIGINAL_SHA` from
  [`lib/original.mjs`](lib/original.mjs).

Put throwaway probes in the ignored `.cache/` research area.
