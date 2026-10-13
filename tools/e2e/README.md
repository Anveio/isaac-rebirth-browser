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

## CI

Pull requests run only `npm run check:build` (`.github/workflows/build.yml`).
Live suites run after every push to main, every six hours (fresh fuzz seeds
while main is quiet) and on manual dispatch. These runs are in
`.github/workflows/live-games.yml`, which builds once and runs one job per
suite. A run in progress is never cancelled: the newest main commit queues
behind it. Each job's summary lists the replay command for any failure, and
the job uploads the run directory as an artifact.

## Adding or changing a suite

1. Register it in [`suites.mjs`](suites.mjs): an id, a driver, default args and
   a one-line `about`. That is the whole registry: the runner, CI and
   `--list` all read it.
2. Reuse an existing driver where one fits:
   - [`lib/room-frames.mjs`](lib/room-frames.mjs) for routes compared on
     every frame. The suite supplies only its own `compare` and `finish`
     checks; [`keys.mjs`](keys.mjs) is the smallest example;
   - `traversal.mjs` with a `--scenario-helper` for multi-room routes;
   - `state-pair.mjs` with a helper and a comparator for paired-gate checks;
   - `paired-game.mjs` for menu/title tapes.
3. Drive the page through [`lib/game.mjs`](lib/game.mjs):
   - `stepSnapshot(page)` for per-frame comparisons, which is one round trip
     carrying the engine's JSON;
   - `step(page, n)` when no state is needed.

   Use [`lib/cli.mjs`](lib/cli.mjs) for flags and output directories.
4. Give it contrived cases and a `--count` of fuzz cases. Write
   `scenarios.json` to the output directory and accept `--scenarios` to replay
   it.
5. Record each mismatch as a field failure, with the presentation, actual and
   expected values. Keep comparing after the first mismatch so one run shows
   the whole divergence.

## What not to check in

- Unit tests of any kind: Rust `#[cfg(test)]`/`#[test]`, Node or Python test
  suites, or standalone component harnesses. Behavior is verified at the game
  level against the original.
- Scripts that assert checksums of outputs (golden hashes of states, traces or
  frames). Compare behavior against the live original instead.
- Self-hashing of tooling sources, or copies of the binary pin. The pin lives
  in [`lib/original.mjs`](lib/original.mjs).

Put throwaway probes in the ignored `.cache/` research area.
