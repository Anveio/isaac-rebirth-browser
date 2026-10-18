# Map opacity and map surfaces

The Map slice is partial. It implements the Options opacity value and gauge,
held-key repeat and clamping, changed-value Back persistence, versioned Map
preferences, the configurable Map action, and small/expanded map presentation.
The menu board follows the selected Options row. Filter, Gamma and Popups
callbacks, controllers, natural reload/migration and complete rendering remain
separate work.

The registered `options-map` suite uses actual keyboard input and a declared
public clock. Its default bank contains a fixed route and the first unfiltered
OS run seed with an independent boot epoch. Whole-bank replay preserves both
cases, their inputs and every failure without drawing replacement seeds. Each
case captures 31 Native checkpoints and 30 paired Browser checkpoints; the
source-only intro checkpoint remains outside the paired comparison.

## Bounded current evidence

Frozen artifact `48e1477f` (1,408 files) passes `check:build`, the required
32-function WASM interface checks and all three existing Browser runtime tests.
The unchanged recorded bank contains fixed `1E0W JX06` and OS `RHRE YT48`.
Fresh Native and Browser processes validate each case's input provenance and
public clock. The [public replay bank](../references/isaac/cases/options-map/recorded-fixed-and-rhreyt48-v1.json)
retains only the declared input cases and their plans.

| Check | Result |
| --- | --- |
| Functional fields | 3,122/3,122 pass across both cases |
| Previously passing functional fields | All 2,942 retained; zero missing keys, changed old actual values or lost passes |
| Additional functional fields | 176 pass |
| Required menu-gauge RGBA regions | 26/26 exact |
| Required Playing-map RGBA regions | 7/22 exact: fixed 7/11, OS 0/11 |
| All required RGBA regions | 33/48 exact; 15 failures remain blocking |
| Full-frame RGBA diagnostics | 10/60 exact; 50 failures retained |

Four former functional failures came from an invalid final-opacity expectation,
not a runtime change: each side's own declared release interval and update
counters yield the exact f32 fade value. Those expectations are corrected in
the new report; the earlier reports remain unchanged. This is not four runtime
parity gains. Every other shared functional row retains its expected value,
actual value and status.

The four remaining fixed-case region failures contain stage-banner text
differences tracked by [#600](https://github.com/Anveio/isaac-rebirth-browser/issues/600).
The OS case also retains lighting differences under the existing
[#268](https://github.com/Anveio/isaac-rebirth-browser/issues/268) rendering work.
The comparison rectangles and exact RGBA checks remain unchanged. Passing the
Map manager fields or gauge does not waive a Playing-region failure.

The same artifact passes all 261 checks for the one recorded
`audible-title-floor-pause` Music case, selected from the complete prevalidated
three-case replay bank. This exercises the edited Music caller, including its
existing lifecycle and replay checks; it is not current OS Music coverage.
The same build completes Options Audio V3 on its unchanged fixed plus recorded
`JWMM YY0Y` bank: 948/948 required checks pass, with actual Native provenance and
clock validation. The fixed case runs once; the first OS admission fails its
4 GiB reserve check before either game launches, and that failure remains
recorded beside the separately completed OS-only replay. Both full-history
caps remain failed (396 retained states for 1,112 and 1,142 commands), and
16/32 full-frame RGBA diagnostics remain failed. This is bounded compatibility,
not complete mixed-audio or pixel parity.

Two cached-Native Save/Continue replays preserve every one of the 3,748 ordered
fields from the actual frozen `137b5e4c` Save baseline: expected values, actual
values and statuses are identical, with zero missing checks or lost passes.
The result remains 3,650 passes and 98 failures. All 66 own-save checks, both
natural reloads and the indices plus all 624 words at 48 MT snapshots pass.
These snapshots do not establish a complete draw-by-draw MT trace. Save RGBA
retains 34/62 exact images and 28 failures, with no lost exact result; metrics
change within those 28 failures, so image-byte identity is not claimed. No new
Native capture or entropy was used for these two Save legs.

The unchanged existing presentation suite fails on both the untouched parent
and this artifact. Its first banner setup captures before Playing; later mask
cases resume an earlier case's own save because they share browser storage.
One mask target passes and eight fail before reaching their targets. These
failures remain recorded; correcting the test setup needs its own input/time
contract and fresh-profile isolation, without changing the old pixel references.

This evidence does not establish whole-game parity, complete shared-MT, complete
mixed audio, controller support, full-frame rendering or every Map interaction.
See the [suite commands](../tools/e2e/README.md#map-opacity-and-map-surfaces).
