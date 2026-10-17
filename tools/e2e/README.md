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

### Running a slice

A small change rarely needs every suite or every case. These narrow a run:

```sh
npm run test:e2e -- --groups                     # the logical groups and their suites
npm run test:e2e -- @items @audio                # every suite in those groups
npm run test:e2e -- @combat --list               # what a selection would run
npm run test:e2e -- keys --case 'random-*'       # only matching cases (comma-separated, * wildcard)
npm run test:e2e -- --rerun-failed               # failed suites of the newest run, on the same cases
npm run test:e2e -- --rerun-failed .cache/e2e/run-…
```

- **Groups** (`GROUPS` in [`suites.mjs`](suites.mjs)) are `@boot`, `@rooms`,
  `@stage`, `@pickups`, `@items`, `@combat`, `@monsters` and `@audio`. A suite
  may sit in several; the registry refuses a suite in none.
- **`--case PATTERNS`** matches the id each suite prints for a case and names its
  directory by. Names differ by driver: `scenario.id` for paired-game and
  state-pair suites (`health-soul-phase0`, `gaper-random-0`), `name` elsewhere
  (`random-0`, `fixed-epoch`), the seed code for traversal-family and floor
  suites. A pattern matching nothing fails with the known ids. The suite
  validates its whole generated bank first, so contrived and OS-random
  composition checks still hold. It then records only the cases that ran,
  marked `caseFilter`, so the printed replay reruns exactly those. A bank marked
  `caseFilter` skips whole-bank count and composition checks on replay; every
  per-case check still applies. Fresh `paired` cases all share the id
  `paired-random-run`, and a filtered `floor` run draws no fresh seeds (replay
  one with `--scenarios … --case`).
- **`--rerun-failed`** reruns each failed suite with the replay flags its run
  recorded. A suite that died before recording its bank runs fresh. A failed run
  prints this command.

A slice is an inner-loop tool. Merging still needs the suites your change
touches, compared against main.

The `save-continue` suite runs public living Pause Quit, actual own-storage
readmission/Continue and natural same-origin reload. Its default is the fixed
moving-before-Save route plus one unfiltered OS seed; all inputs and draw bytes
are retained before either game. No Native state enters Browser inputs.

```sh
npm run test:e2e -- save-continue --count 1 --no-build --production-directory FROZEN
npm run test:e2e -- save-continue --scenarios references/isaac/cases/save-continue/recovery-initial-footprints-fixed-dl17.json --no-build --production-directory FROZEN
```

The current404a/1b30 first bank passes66 own Save guards and two natural reloads,
while strict paired state remains1592/1714 (122 failures) and RGBA34 pass/28 fail.
Original/browser placement/shared-MT differences and historical538c failures stay required;
the own-world reload result is a separate scope. The listed-Bomb guard on0af
reuses both recorded Native cases with all1714 rows,62 Browser PNGs and62 MT
states unchanged; runtime2/2 and real Bomb/legacy-own-save refusal witnesses
pass. The legacy witness uses a real same-origin app upgrade, without storage
injection. [Save evidence and supported
load domain](../../docs/save-continue.md) describes both. Reuse requires exactly
one recorded case; the canonical runner owns its host slot. Fresh Native needs
4 GiB after the production snapshot on the actual evidence filesystem.

The existing death suite has a bounded actual-audio replay:

```sh
npm run test:e2e -- player-death --exit-audio --scenario-file references/isaac/cases/player-death/recovery-death-exit-audio-prefix.json
```

`--exit-audio` requires that explicitly marked one-case seven-capture Exit
prefix; it rejects a full death tape or other selected routes before launching
either game. It preserves the canonical public clock, field checks and actual
death/popup coverage, requires actual deferred Exit to Main, adds all 14 bounded Music/SFX8 checks and requires three
earlier actual unique-owner SFX8 starts in each runtime. Existing Native
forwarding OpenAL instrumentation and same-call Browser API observations remain
blocking, including zero observed Browser forward SFX8 events, complete traces,
unmuted volume, destination and actual buffer hashes. Missing owner evidence cannot imply zero starts. Full pixels
remain retained diagnostics; these checks do not claim audible/PCM equality,
complete music lifecycle or full Shared-MT parity. Default death-bank/OS-random
admission and the separate `audio` suite's menu/PCM schema are unchanged.

The `options-audio` suite uses the canonical paired capture and public clock
with actual Options audio/Back owner observations. Its default bank contains
the fixed audio-arrow regression plus one unfiltered OS seed, boot epoch and
recorded navigation hold. The critical 237 burst stays unchanged; held menu
navigation varies below the source repeat threshold. Every case completes the
public Tab/seed/normal-start route and requires actual final run-seed readback.

```sh
npm run test:e2e -- options-audio --no-build --production-directory FROZEN
npm run test:e2e -- options-audio --scenarios SAVED_BANK --case audio-arrows-back-v1 --no-build --production-directory FROZEN
npm run test:e2e -- options-audio --scenarios references/isaac/cases/options-audio/recorded-fixed-jwmm-epoch-holds-v1.json --no-build --production-directory FROZEN
```

Requests, configured-wrapper identity, actual IsPlaying/gain writes, class
stops, pool resets, submitted sample PCM and Browser API witnesses remain
separate mandatory evidence. Native physical stops and Browser old-voice
stops have independent source-derived stage counts. Missing owner evidence,
wrong labels, an incomplete input/backend journal or a missed playing window
fails the case. The bounded actual step-return journal supplies Back witnesses; the selected
request windows still require every actual command state, including keys, to
survive the unchanged 32 MiB history cap. Pixel
comparisons and the full-history cap retain their actual diagnostic results;
there is no full audible mix, complete MT, other-row or reload acceptance.

The registered Options suite keeps full Native rendering and window queries,
as the music suite does, to preserve the real-clock music upload schedule.

`--reuse-original PATH/TO/CASE/original` requires one selected exact case and
the matching archived observer identity, library, clock, public preferences
and prior validated Native status. Legacy captures without this identity are
rejected before Browser launch. Fresh Native capture requires 4 GiB free after
the production snapshot; fixed replay validation does not establish OS coverage.
The first registered producer on a716 with frozen 99f passes the fixed case
and one unfiltered recorded seed/epoch/hold case: 866/866 functional checks.
Both full-history caps and 16 of 32 pixel comparisons remain failed diagnostics.
Fresh main634-based Controls-source integration on frozen 4ca replays that same
fixed plus recorded JWMM YY0Y bank, without new draws or filtering: 866/866 pass.
All 32 Browser PNG bytes equal 99f; fresh Native object identities and journal
sequence labels differ, with zero verdict changes. The current full-history caps
remain failed at 400/1112 and 400/1142 states; 16/32 pixel diagnostics remain failed.
The kept input-only bank above reproduces both cases. Historical results retain
their original producer/artifact scope.

The bounded `footprint-admission` suite mixes a fixed regression route with one
Source-selected OS-random adjacent miniboss room. It stops at `entry-40` before
shooting and requires actual room6/living-NPC admission plus every initial NPC
field through the shared exact entity comparator. Replay the retained fixed
and failing OS draw without replacement:

```sh
npm run test:e2e -- footprint-admission --scenarios references/isaac/cases/footprint-admission/recorded-fixed-and-hh936e8z-entry40-v1.json
```

The saved `HH936E8Z` case retains its 78 initial Envy position/velocity failures
on counter artifact `40295031` and fresh PR629 artifact `f0eb9e70`; the latter
adds passing mandatory draw-order observations without changing an earlier row.
The mixed suite remains red. This
is short admission coverage, without fight/death/clear acceptance. See
[the artifact-bound results](../../docs/player-footprints.md).

The keyboard Controls gate keeps four fixed routes plus one unfiltered OS run
seed/boot epoch by default. It records every public input before either game;
a failure never selects a replacement draw. Executing producer/observer Source is
archived before generation or replay, its dynamic identity is retained, and
unchanged Source is required at completion. Empty replay paths fail before any
generation; this registered producer requires fresh Native evidence.
Replaying the retained public bank:

```sh
npm run test:e2e -- controls --scenarios references/isaac/cases/controls/recorded-four-fixed-and-9xzmzpr0-v1.json
npm run test:e2e -- controls --scenarios references/isaac/cases/controls/recorded-os-m3mjey2z-v1.json
```

The required owners are actual menu9/10, all24x2 keyboard bindings, physical
held/edge state, Back save and independent native-file/origin-storage readback.
Movement/coasting is mandatory on W→I, both-column and Defaults routes; Escape
binding owns only its declared menu and final normal seeded-run suffix. The
duplicate refusal requires actual SFX187 slot/cooldown and measured Menu clock
origins; it does not rewrite the original three failed clock rows. Every pixel
mismatch and history-cap failure remains visible. Reload, alternate physical
confirm/raw-priority interactions, controller/hotplug, disabled-interpolation/
Skipped behavior and complete pixels/PCM remain separate coverage.

The first registered run on main0ad-based artifact `2a706930` passes all five
cases: 3,688/3,688 fields and 109 checkpoints. The recorded OS input is
`M3MJ EY2Z`/boot epoch `670687023`, retained as the one-case filtered replay above; the complete five-case generated
bank remains in private evidence and the prior 9XZM bank stays unchanged. All 302 actual physical edges are complete and
trusted, with no Browser errors. The four fixed routes preserve every ordered
field, expectation, actual and verdict, and all 83 Browser PNG bytes, against
their historical witnesses. Movement aggregate checkpoint names differ on two
routes. Strict RGBA remains 45 pass/60 fail and all five history caps fail.
Fresh fixed/JWMM Options regression passes 866/866 and preserves 32 Browser PNG
bytes; 64 raw Native pointer/sequence rows differ with zero verdict changes.
Both Options history caps and 16/32 pixel failures remain.

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
and one log per suite. Every runner on the host shares a cap on concurrent suites (one per 6 GB
of memory, at least one; override with `E2E_HOST_SLOTS`), so overlapping runs
queue rather than crash Chromium renderers.

Updated runners require Linux `flock` (util-linux). They atomically publish a
complete PID/start-time record in the existing `slot-N` namespace, so older
frozen runners recognize active updated runners. Persistent `slot-N.mutex`
sidecars serialize updated clients; do not remove them while a runner is active.

Quiesce old runners when upgrading concurrent runs. An unmodified old stale-file
reaper can read a dead record and later unlink an updated runner's live
replacement. The new mutex cannot serialize that old cleanup code; ordinary
mixed-client ownership is compatible, but this stale-reaper interleaving remains
unsafe until the old runners have stopped.

A failing suite prints its replay command:

```sh
npm run test:e2e -- keys --scenarios .cache/e2e/run-…/keys/scenarios.json
npm run test:e2e -- paired --replay-cases .cache/e2e/run-…/paired/<case>/case.json
```

A replay compares builds only when both games get the same keys. Stage-exit
cases also carry port fight keys recorded closed-loop against one production
build (`portRecordedBuild`); replayed on another build they feed stale keys to
a different game, and the suite prints a warning. To compare such builds, run
the suite fresh on each.

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
runner-created run directories under `.cache/e2e` (`run-*` and any `--output`
there) older than the newest `--keep-runs` runs (default 3). Below 12 GB of
free disk it slims every older run. It also removes `/tmp/isaac-production-*`
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
status. A case whose games play their own keys from a capture on
(its first `pairedIndependent` scope, e.g. stage-exit's fight) is compared only
up to that capture; the report's `independentAfter` names it. Drivers that launch the original themselves (traversal, items and
similar) do not support it yet.

### Original wall cache draws

The original pre-renders each room's backdrop walls into a cache texture when
the room loads (`Backdrop::pre_render_walls` 424440) and keeps no wall rects
afterwards. Paired suites built on `runPairedGate` accept `--gl-trace`: the
gate compiles the public GL observer
[`foundation-gl-trace.c`](../original/foundation-gl-trace.c), the original
preloads it, and the manifest's `graphicsTrace` names the recorded draws. A
comparator then grades the port's walls with
[`wall-quads.mjs`](../foundation/wall-quads.mjs): every room the port entered
must equal one of the original's 44-quad wall cache draws (patch source rects,
destinations and flips), in entry order. It is off by default and enabled per
suite in [`suites.mjs`](suites.mjs); `stage-exit` grades it.

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
   `step(page, n)` when no state is needed, `readTrace(page)` for the replay
   trace and `evaluateJson(page, fn)` for any other large page value
   (Playwright's object serializer takes ~25 s for a full trace; JSON text
   about 1 s, with the same value). Use [`lib/cli.mjs`](lib/cli.mjs)
   for flags and output directories. Paired pixel diagnostics run on
   `E2E_PIXEL_WORKERS` worker threads (default 2; 0 compares inline).
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

### Functional public-WASM room-frame backend

Existing room-frame suites also accept `--wasm-node WASM --wasm-sha SHA`.
They replay the same declared menu/key/tick commands through the public ABI
with the same seed and boot epoch, then use the existing per-frame comparer
and suite minima. They add no resource or state override.
`--reuse-original DIR` requires this backend and validates the retained
Original case and public tape against the requested declaration before using
its observations. A matching port replay is never its own reference.

This route reports game-functional evidence only. Browser DOM input, renderer
output and backend audio/PCM remain unavailable and require their separate
Browser runs. Native and Browser resource-partial reports remain unchanged;
a complete public-WASM journal alone does not establish a functional pass.

### Functional public-WASM natural coins

The existing `pickups` driver accepts `--wasm-node WASM --wasm-sha SHA` and
replays its same coin input route/comparer/coverage minima through the public
ABI. `--case NAME --scenarios BANK` selects one declared case without changing
the retained whole bank. The default Browser route remains available.

`--reuse-original DIR` additionally requires the explicit functional backend
and case selection. The prior root must retain its original `scenarios.json`,
public case tape and successful report with the complete Native manifest hash.
The driver checks all three, then runs the existing canonical Native
provenance validator before WASM. It never creates a missing declaration or
uses Native output as a runtime input.

Node journals are actual reset/key/tick/snapshot observations. Browser DOM,
rendered output and audio playback/PCM remain unavailable; source/schema gaps,
comparison failures and missing pickup witnesses remain failures. Historical
passes do not become current-build acceptance without the real replay.

### Common SKU resource fixtures

`common-sku` runs the four retained normal-run Battery/collectible contracts
plus one unfiltered recorded OS idle-approach case by default. The approach
case uses the declared PWTJ run seed; it does not claim a fresh random floor.
`--scenarios` replays recorded public recipes without drawing entropy, and
`--case` selects a mandatory recorded subset. A zero-case filter fails.

The suite reconstructs its private STB inputs locally through
`prepare-shop-sku-fixtures.py` and the existing Base decoder. Local canonical
Base assets are required. Public cases contain source recipes, keys/ticks and
obligations, without original resource payloads or private cache paths.
Generated resources/captures remain below ignored `.cache`; unresolved recipes
fail and cannot silently use a free or base-room substitute. `--prepare-only`
launches neither game. The suite uses headed Chromium with its observed graphics backend (default Mesa) under the existing
runner's display and accepts `--production-directory` for immutable runtimes.
`--reuse-original ROOT` adds a fresh actual Browser capture while the existing
canonical transport reader validates every retained Native input, clock,
resource and archive; it adds no fresh Native evidence.

Owner fields and mandatory contact/branch/grant/debit/presentation/retirement
checks are blocking. Pickup P/V compare exact binary32 words; Native448 maps
to the actual Browser presented owner, with integration values also retained.
The canonical reader reports active queuedTouched as boolean; historical
validated0/1 captures retain their wire and receive an explicit typed decode.
The historical first four cases on frozen build `5c33` retain 41,481 strict
semantic checks: 41,212 pass and 269 fail on health or pickup guard fields. The
new comparer preserves those values and statuses. An explicit mapping renames
1,042 Native field labels from the legacy ChallengeID alias to seed effect mode;
ChallengeID remains a separate frontdoor and state contract. All 16 retained
input and transport checks also pass. The promoted suite's registered replay
on frozen build `9841` passed the free-no-active case through the canonical
runner: 1,749 functional checks and all four input and transport checks passed,
with unchanged source pins. This qualifies that recorded case, rather than
a whole bank or the default OS approach. A separate actual Chromium/Mesa
Browser replay with the reviewed isolated suite on frozen build `cc38` reused
the closed Native free-no-active case. All 1,749 existing functional checks
passed with unchanged actual values, expectations and results, and 29 separate
comparisons of actual Native and Browser seed effect mode fields passed. All
1,770 protected inputs stayed unchanged. This adds no fresh Native evidence
and does not replay or waive the other three cases' historical failures.

All other failures remain strict. PNGs are retained; RGBA/backend audio/PCM
are not graded by this suite. See [the closed first-four scope and retained
failures](../../docs/shop-economy.md#first-common-sku-controls). Fresh Native
capture through the promoted wrapper, the default OS approach case and the
three remaining Battery branch controls remain unqualified. These recorded
fixture results do not establish complete current-main or retail-floor parity.
