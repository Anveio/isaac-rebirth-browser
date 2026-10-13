# Supported fresh GridBase records

Fresh normal starting-room admission now constructs actual supported grid
owners before RoomMusic and Save. GridBase `560ff0` consumes two shared MT
words and retains its own default RNG state. Authored raw0/20 selectors become
GridDecoration1/20 and supply the primary seed. Enabled Door16 slots then admit
in ascending order, followed by interleaved top/bottom walls and ordered sides.
Door and Wall callers draw their primary override after construction; the
constructor's secondary seed remains owned by the actual object.

Room.Save generates real seed pairs for null cells and copies seven semantic
fields from each supported occupied object into Level records before Saved
copy. Full records remain internal. Snapshots expose twelve actual cell
records, including four boundary Door/Wall samples, plus the inherited padding
seed pair. `InitRecord` reconstructs before overwriting both saved seeds, but
this API is not yet routed through durable continue or room restoration.
The [source contracts](source-contracts.json) pin sixteen original functions,
field offsets and caller order.

Run `npm run test:e2e:grid-records`. The gate runs two contrived and four
OS-random cases, records entropy, validates complete public command plans and
pins each owned native process executable. Reproduce the
[fresh proof](verification.json) with its [recorded cases](accepted-scenarios.json):

```sh
npm run test:e2e:grid-records -- \
  --scenarios references/isaac/foundation/grid-records/accepted-scenarios.json
```

All six actual original/browser cases ran against source `e7267ba` and immutable
artifact `456b63031b0431f2bbdfc9f3198b4ec6c999c21e3d8beb60f7dbb88f9e5696fc`.
The gate correctly exits nonzero: all six cases retain 254 required current-row,
record-field or whole-occupancy failures. The separate copy-ownership and full
command-provenance checks pass 6/6. Twelve authored primary-seed handoffs match;
353 of 504 paired semantic record fields match. Actual Door16 records are read
in every case, with sixteen Door observations across the cohort.

Native null-cell counts are 88/83/88/85/86/87, versus 89 in production. Variable
GridDecoration1/0 admission remains missing. Actual occupied secondary seeds
are now copied from supported objects, but their numbers and the other shared
seed pairs remain red. Enabled saved Door states also differ: source 1 versus
production 2. Every paired record field, current seed pair and whole occupancy
comparison blocks case status. All six complete postrun MT states differ as
explicit diagnostics; those diagnostics do not decide the gate status.

The strict recorded W0L0LYYM replay on the same artifact passes all 18 functional
checkpoints and both transports. Its prior Saved baseline passed 17/18. This
result does not establish shared-MT or broader tear parity. Exact pixels remain
an explicit diagnostic: seven of eighteen frames match, and eleven differ.

Full build, asset pins, types, matrix and strict Clippy pass. The first Door
coverage rebuild failed with ENOSPC; its log is preserved, followed by a
successful single-job retry. The production WASM ordinary snapshot measures
171,446 JSON bytes, with a 3,434-byte bounded saved summary. A 64-sample Node
measurement under host load took median 2.755 ms and p95 13.383 ms for a tick
plus snapshot; it excludes rendering and proves no original performance parity.

Other derived callbacks, actual weighted room costs, mixed transition admission,
PlayerItems Save/Load and durable continuation remain separate source gaps.
Native observations never supply production seeds, occupancy or cursor changes.
