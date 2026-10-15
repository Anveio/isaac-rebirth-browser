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
npm run test:e2e -- --affected              # only the suites your change reaches
```

### Only what your change touches

```sh
npm run test:e2e -- --affected                     # suites the changes since origin/main reach
npm run test:e2e -- --affected --base REF --list   # the selection and the reason for each suite
```

`--affected` diffs against the merge base, including uncommitted and untracked
files. Then:

- **Tool and evidence files** select the suites whose static closure reads them:
  the driver, libraries, scenario helpers, observers, comparators and literal
  `references/` paths.
- **Engine Rust** selects at function level. [`affected-map.json`](affected-map.json)
  records which engine functions each suite's recorded browser traces execute
  when replayed through the native `replay` binary under source-based coverage.
  A coverage build of your tree locates the functions your diff touches.
- **Conservative fallbacks:**
  - a change outside any function, or to a function no suite has executed,
    selects every suite that executes that file;
  - `web/`, build inputs and the runner select every suite;
  - a suite with no recorded traces runs on any engine change.
- **Inert:** docs and Markdown select nothing.

What to expect: tool, spec and evidence changes select precisely, e.g. one
entity spec selects one suite. Engine changes select every suite that executes
a touched function. Over twelve recent engine merges, a leaf module
(`item_pipeline`, `player_stats`) selected 10 of 45 suites. Changes to core
paths such as `Game::start_run` or `advance_room_clear`, which every run
executes, selected 40 to 45.

The first engine query installs `rustfilt` into `.cache/cargo-tools` and needs
`rustup component add llvm-tools`. Use it for the inner loop. A broad change
still runs the full set before merging.

Refresh the map from a full run that kept its evidence when suites or engine
call paths change a lot. Slimmed runs drop the browser-state archives that hold
some suites' traces, and a suite without traces runs on every engine change:

```sh
npm run test:e2e -- --keep-evidence
npm run affected:map                 # or: npm run affected:map -- run-…
```

`--affected` is a developer convenience, not a merge gate.

The runner rebuilds `dist/` when `web/` or `engine/` sources are newer than it
(use `--no-build` to skip). It builds the native `floor` generator, runs each
suite under its own virtual display, and writes `.cache/e2e/run-*/report.json`
and one log per suite. A failing suite prints its replay command:

```sh
npm run test:e2e -- keys --scenarios .cache/e2e/run-…/keys/scenarios.json
npm run test:e2e -- paired --replay-cases .cache/e2e/run-…/paired/<case>/case.json
```

### Keeping a cited case

A case cited in an issue or PR must outlive the container that recorded it:

```sh
npm run case:keep -- paired .cache/e2e/run-…/paired/<case>/case.json cqhr-room-combat
npm run case:keep -- npcs .cache/e2e/run-…/npcs/scenarios.json 162-fly-bomb --case paired-fly-bomb-queue
```

This writes the case to `references/isaac/cases/<suite>/<name>.json` and prints
its replay command. Commit it with the issue or change that cites it.
`--case` keeps only the cited case of a scenarios file. Files over 256 KB need
`--force`.

### Disk retention

A full run captures several GB. When a suite passes, the runner slims its
output ([`lib/retention.mjs`](lib/retention.mjs)):

- **Kept:** `report.json`, `scenarios.json`, `case.json`, logs and every file of
  at most 1 MB. Every recorded case therefore still replays.
- **Gzipped:** recorded traces.
- **Dropped:** frames and other large evidence.

A failing suite keeps everything. At start, the runner slims every suite of the
runner-created `.cache/e2e/run-*` directories older than the newest
`--keep-runs` runs (default 3). It also removes `/tmp/isaac-production-*`
snapshots whose process is gone.

Pass `--keep-evidence` to retain everything, for example to inspect frames of
a passing suite.

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
[`entities/`](entities/) and the `entities` suite grades it. `entity-<name>` runs
that spec alone, which is the per-entity inner loop:

```js
export default { type: 12, variant: 0, about: 'Horf rooms: ...', count: 2, required: ['admission'] };
```

[`entities-scenarios.py`](../original/entities-scenarios.py) gives every spec its
own contrived routes and OS-random rooms next to the start room that admit it
(the rules in [`entity-scenarios.py`](../original/entity-scenarios.py), or the
boss routes of [`boss-scenarios.py`](../original/boss-scenarios.py)). Specs
share one seed stream, and a room that admits several of them is launched once
and graded for each. Both games then walk in and shoot around. [`entity-state-pair.mjs`](../foundation/entity-state-pair.mjs) compares
the shared NPC, room, door and projectile fields exactly, spec by spec. A spec
whose rooms cannot be found fails the suite without hiding the others. The
whole workflow is
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
