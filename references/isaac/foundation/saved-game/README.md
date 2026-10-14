# Supported fresh-run save ownership

At actual normal Level admission, `58c411` invokes System.Save. Game.Save
`54f230` clears the real SavedGame destination, copies the modeled ItemPool,
calls Room.Save, copies ordinary Level descriptors, then copies PlayerManager
state. The production route retains actual Level/Room/Player/inventory and
ItemPool data internally. Snapshots expose bounded samples and a summary.

The [Player item owner](player-item-owner-source.json) now runs
`54f373 → 5bf400 → 518350` at that save boundary: it flushes the real queued
grant and stores 347 counts, 450 current child RNG states, and four active or
reserve fields. The restore store helper targets an already constructed Player
owner, skips dummy count ID 0, and restores current RNG35 states without drawing
new seeds. It does not enable or execute public Continue. Temporary effects,
active cancellation and costume normalization are explicitly bounded; wider
owners fail until their actual callbacks are implemented.

The folded `ordinary-run --ambient-grid` comparison requires all 801 item-owner
fields against read-only original SavedState observations. The
[current six-case proof](player-owner-current-pair.json) retains 4,806 passing
saved-field comparisons, including 450 nonzero current RNG states per case.
The baseline lacks this owner and retains six missing-owner assertions; both
artifacts retain the same 228 grid differences. All owned counts and active
words were zero in this cohort. The captured sources are 3fbd27f and ba0ae5e;
the current source projection is separately reviewed. Public Continue, restore,
nonzero count/active bindings and independent backend callback coverage remain
unaccepted. The historical failures below remain preserved.

Clear generates the actual 135×448 seed-pair table plus the saved manager seed.
Room.Save consumes two shared MT words for each null cell in the supported live
room grid. Level copy excludes only descriptor index −3; ordinary unused −1
records copy too. The fresh normal scope therefore copies all 135 rows.
PlayerManager's second RNG is separately default-constructed with
`0xaa17414f`; the incoming 13th startup seed belongs to the first manager RNG.
The [source contracts](source-contracts.json) pin each function's original bytes
and the copy order.

Run `npm run test:e2e -- ordinary-run`. The gate executes two contrived and four
OS-random cases, records entropy for replay, and checks complete public command
plans and the owned native process executable SHA. The
[fresh evidence](verification.json) uses source `98c4ba6` and immutable browser
artifact `0454fdd751e17452e09b8b9162d4312747e84437d4782050a5d6580f77e6e8e8`.
All six scoped ownership checks pass: saved seed/code/flags, supported Player
fields, actual modeled ItemPool fields, all 135 copied rows, 54 legacy saved
pairs, and independent active-descriptor identity. The 54 active-row reads
verify Level-to-Saved equality within both games. Menu checks compare 14,976 MT
words, 216 saved pairs, 270 Level pairs, 4,296 stat poses and 96 DOM commands.

All six paired current-row seed comparisons remain red. Actual native null-cell
counts are 88/83/88/85/86/87, versus 91 in production. The required cells include
56/85 (actual GridDecoration1/20) and 103, whose occupancy varies. Production
does not yet admit the actual occupied grid constructors or their secondary
seeds; its summary lists 44 pending occupied slots. Native values never supply
these fields or change production occupancy. All six complete postrun MT states
also differ as disclosed diagnostics. Neither untouched legacy rows nor within-game copy equality prove
current-room numerical parity.

The first gate recorded active-row and occupancy failures without making them
block its case status. That policy error is corrected: actual paired
`levelSeeds`, `savedSeeds`, per-sample occupancy and the complete live-grid
occupancy now determine case status. A
[required-comparison regrade](required-grid-regrade.json) preserves all six
unchanged native manifests, browser checkpoints and canonical traces by hash.
All six selected cases execute and fail, retaining 115 required field failures;
the independent copy-ownership checks remain 6/6. The original scoped report
and runtime artifact are preserved. The regrade invokes no game, supplies no
observed state to production and exits nonzero. Reproduce it with:

```sh
node tools/e2e/regrade-saved-game.mjs \
  --bank .cache/e2e/saved-game/current-row-final-1 \
  --output .cache/e2e/saved-game/current-row-required-regrade.json
```

The strict W0L0LYYM replay remains a failure at 17/18 checkpoints: at
`random-resumed`, the original has zero live tears and production has one. All
other functional fields and both transports pass. The failed comparison is
retained in the evidence and remains required. The first six-case cohort's
[rejected allocated-row inference](rejected-copy-predicate-report.json) is also
preserved; it was corrected from actual source stores and observations.

The historical WASM ordinary snapshot was 135,792 JSON bytes, including a 1,901-byte
saved summary with nine legacy and nine current-row samples. The full tables
and ItemPool copy stay internal. In a 256-sample Node measurement under host
load, a tick plus snapshot took median 1.151 ms and p95 1.810 ms. This measures
production cost and makes no original performance-parity claim. Continue/load,
occupied-grid record reconstruction, broader room/effect consumers, and
unmodeled ItemPool trinket/identification fields remain outside this leaf.
