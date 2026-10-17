# Save and Continue

The browser stores three origin-local save slots and reconstructs a supported
initial normal Isaac room through public Pause Quit, Main Continue and a real
page reload. This is a partial implementation. Complete Native restoration,
shared-MT alignment and later-room/item/actor saves remain open.

The versioned, bounded Save packet contains explicit Player, ItemPool,
Level/Room/GridRecord and profile owners. It does not replace the running
Game/State. Quit reports the actual own-storage write result; fresh menu
construction reads the actual stored bytes before selecting Main Continue.
A failed latest write may leave an older valid own save. Each IO request
suspends the current public Tick until its own response, including the
remaining iterations of a batched Tick. Keyboard/InputConfigs, audio/display
preferences and the global clock retain their current owners. Constructor,
cache and Room callbacks reconstruct the saved world; host data never supplies
Native observations to WASM.

The first admitted Continue domain is stage1, normal Isaac, challenge0 and
seed-effect mode0 in the initial rectangular room, with neutral owned items,
no saved NPC/pickup/familiar actors and supported actual grid records. The
actual listed Bomb vector must have a known zero count at Save. A listed Bomb
is unadmitted because this load caller cannot reconstruct it; the count is a
conservative port list owner, not an inferred Native live/dead count. A legacy
packet without this owner remains unknown and its Continue request is refused,
while its own stored bytes remain intact. Ordinary
walking footprints are admitted: they paint a transient backdrop and are not
saved Blood7 bodies. Continue constructs the normal backdrop again, and both
stationary restored endpoints require actual empty paint arrays. Other stains,
room travel, special characters/modes, nonzero item callbacks, unsupported grid
factories and progression remain explicit unsupported load domains. The packet
validator and this narrower load admission are separate; a durable packet does
not by itself establish an implemented Continue caller for every saved state.

## Closed live scope

The first current404a producer with the explicit Save opt-in ran immutable
`1b30a0fb` (1,407 files). The registered `save-continue` gate captured one
fixed moving-before-Save case (`T1PK6Y0W`) and the first unfiltered OS draw,
`DL17GHC7` / `356560097` (raw bytes `e1ac4015`). Every case and input was
recorded before either game; there was no redraw or Native-conditioned
selection.

The admission-guard successor `0af25692` (1,407 files) changes only the
listed-Bomb ownership/refusal code. Its isolated build passes the required
affected-suite preview and `check:build`, forces an actual fresh top-crate
compile, checks Save/Controls exports and the actual new transport key, and
proves changed WASM/artifact bytes. It replays both exact recorded cases
against those same validated Native captures without drawing inputs or
launching another Native process. All 1,714 ordered field rows (expected,
actual and status), 62 Browser PNG byte sequences and 62 complete shared-MT
snapshots equal `1b30a0fb`, with zero changed rows, gains or losses. The two
existing Browser runtime tests also pass (2/2) on `0af25692`. These are slot1
live witnesses; three slots are implemented, without a three-slot live claim.

Both original fresh Native processes pass provenance and independent public-clock
validation (158 acknowledgements and 32 captures each). Browser capture passes
with no reported Browser errors. Actual own Save/Continue guards pass 66/66;
both natural same-origin reloads pass their own storage/world/movement guards.
Those checks require the actual Quit packet revision, write/read events, actual
pre-Save footprints, empty restored paint arrays, the saved world and later
public controls. Reload does not reset or inject storage, and Browser inputs
contain no Native data. Source and production bytes are unchanged before/end;
both later reuses also verify all 56 actual Native files per case and the
unchanged recorded bank.

Two additional Browser-only public witnesses pass on `0af25692`. The first
places a real bomb, leaves it listed through Pause Quit and records an actual
stored count of one; Continue produces visible unsupported-load feedback
before requesting another world. The second produces an actual own save on
`1b30a0fb`, switches the production app at the same origin/port, and naturally
reloads `0af25692` without resetting or injecting storage. Its actual legacy
packet lacks the count and Continue is refused. Both refusals retain their
actual failure feedback and own stored bytes; they do not claim resumed
unsupported gameplay or fault-injected storage coverage.

The strict paired result remains **FAIL: 1,592/1,714 rows pass, 122 fail**.
The fixed case passes 784/857; the OS case passes 808/857. Each retains seven
canonical one-way gameplay-family scope failures, fourteen unused-row127 seed
failures, fourteen literal saved-seed text spacing differences and fourteen
last-penalty-stage differences. The fixed moving context additionally retains
24 real post-Continue Player X/velocity and live-tear failures, including the
independent current-System-owner rows. Native resumes at the left door while
the Browser selects the right door. The recorded shared-MT cursor already
differs before Save; continuation consumption also differs. The local chooser
must not be fitted to the observed side. Own-storage/world guards do not imply
Native placement, complete actor or complete RNG parity.

Strict decoded RGBA diagnostics pass 34/62 frames and fail 28/62. These are
actual diagnostics, not a full rendering or PCM qualification. The earlier
`538c921c`/archived051b stationary V7 result keeps its historical producer,
input and artifact scope; its 49 strict failures and earlier failed movement
reports remain intact. Its chance matching-door result is not an exact-parent
comparison for the current moving case.

The same fixed public prefix through `save-before-quit` was also captured on
parent `2a706930` and candidate `1b30a0fb` against the same validated Native.
Both pass 89/89 canonical rows and 12/12 checkpoints. All ordered expected,
actual and status rows, all 12 Browser PNG byte sequences and all 12 complete
shared-MT states are identical, with zero gains, losses or changed rows. The
361 Source/Native pins and both artifacts remain unchanged before/end. Each
side retains seven Native RGBA matches and five differences; the supplementary
Native MT readout retains ten word-array passes and ten cursor failures. This
is a pre-Quit no-loss proof, not a regrade of the full 122 Save failures.

Initial-room ambient effects remain a concrete restoration frontier: Native
saves and readmits Tiny Bug1000/21, Tiny Fly1000/33 and Wall Bug1000/68 through
its saved-entity vector; this initial Browser load does not restore them. The
fixed case has seven such continued Native bodies and the OS case has two,
while Browser has none. Their constructor/Init/update and shared-MT effects
need a separately owned saved-ambient implementation; the first-visit spawn
callback must not be rerun as a substitute.

Private readouts and journals remain local. The public retained bank contains
only the declared inputs and original OS draw receipt. It carries no Native
readback, captures, disassembly, copied room resources or storage payload.

## Reproduction

```sh
npm run test:e2e -- save-continue --count 1 --no-build --production-directory FROZEN
npm run test:e2e -- save-continue --scenarios references/isaac/cases/save-continue/recovery-initial-footprints-fixed-dl17.json --no-build --production-directory FROZEN
```

The default draws a new unfiltered OS case. The recorded bank replays the
closed fixed/OS inputs. `--case` selects recorded ids after full-bank validation;
a partial bank must retain a valid nonempty filter. Native reuse requires a
single recorded case and its canonical validated Original directory. Every
selected field, own-storage guard and natural reload remains required; pixels
retain their strict diagnostic result (`--require-exact-pixels` makes them
blocking). The canonical runner owns host-slot admission. Actual Native
admission requires 4 GiB on the evidence filesystem after the production copy.

Complete saved actors/items, broader rooms/stages/characters, arbitrary paint,
profile unlock callbacks, fault-injected durable IO, cross-process Original
file reload, full shared-MT consumers, pixels and PCM require further evidence.
