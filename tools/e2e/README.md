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
(use `--no-build` to skip). It always builds the native `floor` generator,
which seed discovery runs, so `--no-build` never leaves it stale. It runs each
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

A failing suite keeps everything. State-pair suites (`entities`, `npcs`,
`monstro`, …) write each case's browser manifest and trace gzipped
(`manifest.json.gz`, `trace.json.gz`, losslessly, with raw and archive
hashes). A case then takes about 120 MB instead of 350 MB, and the comparators
read either form. At start, the runner slims every suite of the
runner-created `.cache/e2e/run-*` directories older than the newest
`--keep-runs` runs (default 3). It also removes `/tmp/isaac-production-*`
snapshots whose process is gone.

Pass `--keep-evidence` to retain everything, for example to inspect frames of
a passing suite.

### Shared-MT draw diagnostic

Original and port consume one shared MT19937 stream, so a missing or misordered
consumer shifts every later random choice. Paired suites (those built on
[`runPairedGate`](../foundation/paired-game.mjs)) accept `--mt-draws`, or
`E2E_MT_DRAWS=1` for a multi-suite run:

```sh
npm run test:e2e -- roundworms --mt-draws --replay-cases .cache/e2e/run-…/roundworms/paired/<case>/case.json
```

The original runs with the draw observer
[`foundation-mt-trace.c`](../original/foundation-mt-trace.c) chained, and the
browser enables the engine's opt-in draw log (`window.game.mtDraws`). Each
case report gains an `mtDraws` entry from
[`mt-draws.mjs`](../foundation/mt-draws.mjs). It compares gameplay draws in
order by wrapper kind and update counter, and reports the first divergence
with native return addresses and port source sites. Everything before that
point used the same stream positions. The diagnostic never changes a case
status. Drivers that launch the original themselves (traversal, items and
similar) do not support it yet.

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

## Public item composition

`item-composition` gives both games the same declared raw Challenge XML and,
when present, full Special room catalogue. It enters Challenge 1 through real
menu keys and collects authored pedestals through normal movement. Native run
seeds, caches, actors and RNG observations are comparison evidence only.

```sh
npm run test:e2e -- item-composition --keep-evidence
npm run test:e2e -- item-composition --scenarios references/isaac/foundation/item-composition/history/scenarios.json --keep-evidence
```

The default case generator records every OS token before projecting cases.
`--scenarios` replays the recorded bank without calling the generator or drawing
entropy. Resource paths in a bank resolve relative to that bank. `--prepare-only`
writes launch inputs without running either game and reports `prepared-not-executed`;
it is not parity evidence. For preparation without a build or display, use
`node tools/e2e/item-composition.mjs --prepare-only --scenarios BANK`.

The capture retains every admission and gameplay presentation, actual public
commands, complete backend trace, committed browser journal, native journal and
declared PNG. A partial prefix cannot pass as a complete capture. The separate
readback checks transport/resources; the functional comparator retains each
mismatch and required missing history witness. Recorded histories do not claim
that their required contact, cleanup or actor stages succeeded. See the
[source coverage and remaining requirements](../../docs/item-composition-coverage.md).
The checked-in replay keeps both fixed phases and both previously recorded OS
projections; none is filtered by its outcome. The first fixed phase has actual
native and a320 browser captures with functional differences and missing child
coverage. That result does not qualify every recorded case or the current main
runtime. A coherent current producer must admit the declared whole Special
catalogue, ordinary Brimstone/Knife and controlled Knife. Missing support fails
explicitly; private grants and alternate WASM are never substituted.
Full-frame pixels remain a separate diagnostic.

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

Native evidence journals default to the historical per-record gzip format. For a
new capture, `foundation-clock-run.py --archive-evidence --journal-codec zstd`
(or the same extra arguments to `composition-run.py`) explicitly selects a
persistent level-1 Zstd stream in `native-progress.jsonl.zst`. It retains the
canonical JSONL records and flushes every completed record at the existing gate.
The complete manifest remains gzip. This opt-in requires installed Python
`zstandard`, system `libzstd`, and Node 24 with built-in Zstd support for JavaScript
readers. `cloud/install.sh` installs the pinned Python dependency through
`cloud/requirements.txt` and the system library through apt; non-root portable
hosts must provide system `libzstd` before opting in. Legacy gzip runs do not
load those codec helpers. Codec metadata and
source/tool identities must describe the new capture. Existing evidence and
archival restoration recipes keep their original formats and identities.

A running Zstd journal is an unfinished frame. Atomic metadata pins only its
committed encoded prefix; recovery must verify both encoded and decoded hashes,
bytes and record count and retain incomplete status. Only a successful final
frame/checksum write, file close and metadata commit certify a closed journal.
Readers reject incomplete frames, uncommitted tails and ambiguous mixed formats.
No game field, public input, clock, RNG stream or capacity floor changes.
