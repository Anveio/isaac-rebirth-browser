# Save and Continue

The browser stores three origin-local save slots and reconstructs a supported
initial normal Isaac room through public Pause Quit, Main Continue and a real
page reload. Continue also resumes any visited stage-1 room (see the #896
section below). This is a partial implementation: complete Native
restoration, broader shared-MT consumers, later stages and saved live bombs
remain open.

The versioned, bounded Save packet contains explicit Player, ItemPool,
Level/Room/GridRecord and profile owners. It does not replace the running
Game/State. Quit reports the actual own-storage write result; fresh menu
construction reads the actual stored bytes before selecting Main Continue.
The complete encoded profile document is bounded at 4 MiB. Walking paint has
no separate 4,096-record cutoff and is never truncated to fit. An actual
capacity export failure produces no partial packet, receives exactly one
failed-write acknowledgement, preserves every older stored byte and resumes
the remaining public Tick iterations. Other export errors remain errors.
An older valid own save may still be admitted after that failed latest write.
Each IO request suspends the current public Tick until its own response,
including the remaining iterations of a batched Tick. Keyboard/InputConfigs, audio/display
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
walking footprints and other Blood7 paint are admitted: they use a transient
backdrop and are not saved Blood7 bodies. Continue constructs the normal
backdrop again; the earlier stationary restored endpoints require actual empty
paint arrays. Other stain families, room travel, special characters/modes,
nonzero item callbacks, unsupported grid factories and progression remain explicit unsupported load domains. The packet
validator and this narrower load admission are separate; a durable packet does
not by itself establish an implemented Continue caller for every saved state.

## Continue after leaving the initial room (#896)

Game.Save (`54F2C8`) saves the ItemPool (`581ED0`), runs Room.Save (`5CE2D0`)
on the current room and then saves the whole Level (`58CD90`). The Level save
holds every descriptor with its seeds, visited and cleared flags, persistent
spawn set and saved entity records. It also holds the current and previous
rooms (`Level+8188` and `+818C`) and the entry door slot (`+8198`), followed by
the Player. Room.Save keeps only the persistent families of `5C57C0`: bombs,
pickups, slots, shopkeepers, fireplaces and the listed effects. NPCs are not
saved.

On Continue, the current room takes Room.Init's (`5CECC0`) revisit branch,
because its descriptor is visited:

- The saved records come back (`5C5860`).
- `5CE6B0` respawns the layout's other entities unless the room is cleared.

The behaviour below was observed in the original:

- **Room and position.** The run resumes in the same room. Isaac stands
  40 units inside the entry door. A start room that was never left has no
  entry door, so a door is drawn at random.
- **Controls.** They are disabled through the room fade and come back by
  300 frames after Continue.
- **Uncleared fights.** Flies that moved before Exit come back on their
  layout cells at full health. The fight restarts.
- **No re-rolls.** The pools, their RNGs and every descriptor seed are
  restored. A treasure pedestal holds the same collectible whether the run
  was quit before or after entering its room. For example, 3YSTBQPG shows
  item 71 in both orders.

The port now admits a second Continue domain, `ContinueDomain::VisitedRoom`
(`RunRecord::admit_continue`). It covers normal Isaac in challenge 0 and seed
effect mode 0, alive and on any stage-1 room reached so far.

- **Saved state.** ChangeRoom now records the previous room and the entry
  slot in the saved entry, where it used to clear them.
- **Restore.** `restore_continued_room` rebuilds the saved current room in
  the revisit order: it clears the transient NPCs and combat doors, respawns
  the layout NPCs unless the room is cleared, and readmits the saved pickups,
  slots and ambient records.

The initial-room caller retains its guards, but #908's visited-room fallback
can bypass them. A public Browser probe on unchanged main (`3BMBMFTE`, E,
Pause Quit, Main Continue) saves an actual own packet with listed Bomb count1,
then resumes gameplay with zero Bombs. The packet was never edited. A stricter
source-only repair's actual Browser replay keeps the Bomb and stored bytes,
but its refusal currently marks the host runtime failed. Both failures remain
preserved, and the narrow admission/host refusal repair is in progress.
This is an actual Browser storage/admission contract observation; complete
Native Bomb Save/Continue restoration remains unqualified. Persistent
shopkeeper/fireplace records and broader saved actor/player ownership also
remain open.

The `continue-run` suite (`tools/original/continue-run-scenarios.py`) exits
and continues in both games, with these cases:

- KRAYFRWA, with flies that moved before Exit.
- 3YSTBQPG, quitting in the treasure room. This case Continues, collects the
  item, walks back and quits and Continues again in the start room.
- 3YSTBQPG, quitting before the treasure room was ever entered.
- One OS-random neighbour room, with a random idle before Exit.

Every gameplay capture compares the room, previous room, exact player
motion, controls, hearts, consumables, owned collectibles, visited set and
clear flag. The room's pickups are compared as a multiset. NPCs are compared
as exact type, variant, subtype, position and HP tuples at 60 frames after
each Continue, and as identities afterwards, once their AI randomness
diverges. Every actual Native NPC is required, including families absent
from the Browser; missing actors fail the count and multiset comparisons.
The complete declared bank is validated by the existing route owner before
selection, and the full input bank and selected IDs are retained separately.
NPC status, friction, color and complete shared MT remain unqualified.

## Required NPC audit after #908

Fresh Original and actual Chromium on source `c6265d54`, with unchanged main
runtime/WASM `5304d150`, run all four cases of the bank declared before either
game (three fixed routes plus recorded OS neighbour `7JRW6L28`). All 71 paired
checkpoints capture successfully. The required grade is **551 pass / 16 fail
of 567 checks**. Both treasure routes and the recorded neighbour pass their
selected contracts. The uncleared `KRAYFRWA` room has six actual Native NPCs
and four Browser NPCs: two Ring Fly222.0 bodies are absent at eight captures,
including the first post-Continue layout/health tuples. Their implementation
remains a separate NPC import gap.

Replaying the preceding comparator over those same immutable captures reports
567/567, because it excludes both Native Ring Flies after learning families
from Browser output. The correction preserves 551 rows exactly and makes the
16 affected count/multiset rows required failures. This is a corrected test
contract, with no runtime improvement. The old comparison and its diagnostic
rows remain retained beside the corrected report. NPC status/friction/color,
complete shared MT and full pixels are not qualified by these checks.

Fresh main build and the seven runtime Browser checks pass. The grader's fresh
build and affected preview pass and produce exactly the same WASM. These
results do not qualify the pending Continue admission/record reconstruction
repair or the missing saved pedestal and acquired-item owners.

## Current-room regular records at Save

Game.Save now captures the supported regular pickups in actual manager order at
the Room.Save boundary, before PlayerManager Save flushes queued grants. The
RoomOut writer uses the same pure capture loop. The captured list is installed
in the copied SavedRun room; it does not replace live bodies or widen Continue
admission. This follows the mapped Original Game.Save `54F347` to Room.Save
`5CE2D0` call chain.

The fresh candidate on PR #711 base `248f2d09` has tree `46affff7` and frozen
artifact `2a6cbdf4` (1,417 files; WASM `f415dc46`). Build, interface checks and
the canonical affected preview pass; the preview selects 52 suites without
running them. Its registered **Pause Quit saves live regular pickups in the
current room** Browser regression passes all 28 checks, 18 capture labels and
217 commands on the fixed public `PBW9YNC8` route. At paused room 7 / GameUpdate 91,
four uncollected coins remain frozen through Quit. The actual revision 2 packet
contains their exact ordered handles, positions, spawn/RNG seeds, charge,
touched and saved-byte fields; the successful write, stored bytes and menu
readmission identify the same packet.

The two existing initial-room capacity contracts also pass on this artifact:
35 checks for a complete packet, successful acknowledgement and Continue from
its own store; 4,062 checks for rejecting the oversized write and Continuing
from the unchanged older store. These remain Browser storage contracts.

The retained Bomb-only baseline remains 27 pass / 1 fail: its packet omitted
those same four live records. Earlier launch/observation failures and its
separate retention hold remain preserved. This is a Browser storage contract
witness, not Original actor-save packet parity. The visited-room packet does
not establish later-room Continue, NPC restoration or reload support, and this
PR #711 base qualification does not grade later public-main imports.

## Saved ambient restoration

The successor `137b5e4c` restores Tiny Bug1000/21, Tiny Fly1000/33 and
Wall Bug1000/68, subtype0, from owned saved records. Records retain manager
order, position, spawn seed and RNG30 state. Continue runs fresh Init and its
source callbacks before restoring the saved RNG; it does not copy animation
state or rerun the first-visit spawn callback. Unknown saved families remain
unsupported. A legacy packet without ambient ownership is unknown, and its
Continue request is refused while its stored bytes remain intact.

The ordinary scheduler restores physical positions before the Full/Fast split,
and critters use the already-incremented Game Full counter. Stop reconstructs
a Room whose Backdrop constructor owns one raw shared seed; the same callback
runs before Quit, Death Exit and Restart destination admission. The generated
seed is retained in the Backdrop owner, including zero.

A fresh current-main build and runtime2/2 pass. Both retained input cases reuse
their unchanged independent Original captures. All 1,714 inherited ordered rows
and Native expected values remain: **1,644 pass / 70 fail**, with 52 gains and
zero lost passes versus `0af25692`. This fixes the 24 fixed-case post-Continue
Player/tear failures and 28 unused-row127 seed failures. All 14 OS player-Y
regressions found on the intermediate `adfb14de` candidate are recovered; that
failed candidate and its reports remain preserved.

The additional 2,034 shared-MT/ambient rows pass 2,006 and fail 28, with 854
gains and zero losses against the same projection of closed parent captures.
All 48 measured shared-MT checkpoints match the original cursor and all 624
words. The 28 remaining additive failures are exact binary32 velocity-word
differences in three fixed-case ambient bodies before Quit; none occurs after
Quit or Continue. The OS case passes all 507 additional rows. No tolerance or
original expected value is changed.

Actual own-save guards pass 66/66 and both natural reloads pass. A real
same-origin upgrade from `0af25692` confirms that missing legacy ambient
ownership produces visible refusal before any world request, retaining the
actual own packet. This witness performs no storage injection. Strict RGBA
remains 34 pass/28 fail, with no changed verdicts. The 70 inherited failures
remain 14 typed menu/gameplay-scope errors, 28 literal seed-text separator
differences and 28 last-penalty-stage differences. Complete gameplay, broader
saved actors/items/rooms and PCM remain unqualified.

The generic Browser seed-provenance check now uses the unique actual
`gameplay-start` snapshot when Exit finishes on Main, whose reconstructed
Character dialog has cleared its transient text. Final committed run-code
checks remain required. The retained seven-capture Exit/audio case passes all
95 canonical and 14 audio comparisons on both current-main `253f6a3b` and
`137b5e4c`, with every expected value, actual value and verdict identical.
Actual death/Exit, public clocks and three earlier SFX8 positive owners pass;
pixels remain 4 pass / 3 fail. The earlier rejected capture and its ungraded
audio remain preserved. This covers Exit, without a Restart or full PCM claim.

## Save capacity and current integration scope

PR696's six-file Save foundation is merged at `9f952efd`. Its fresh Browser
qualification remains bound to exact main715.
Fresh baseline `7ad918af` and Save candidate `92dc82b9` pass `check:build`,
ABI/constructor checks, the canonical affected preview and all four public
Browser runtime tests each. Four actual same-Native Save controls replay the
unchanged fixed moving case and recorded `DL17GHC7` on both artifacts. Each
role preserves all 3,748 ordered expected/actual/status rows: **3,650 pass /
98 fail**, with zero lost passes, changed rows or pixel-metric changes.
All 33 own-save guards pass on each control. The 98 failures comprise 28
fixed-case ambient velocity-word rows, 14 canonical menu/gameplay-scope rows,
28 seed-text separator rows and 28 last-penalty-stage rows. The original closed
baseline leg was reused after its wrapper mirror failed; that failed wrapper
remains failed. No new Original process or random draw was used.

The merged current-main composition separately passes an isolated native port library check:
`cargo check --locked --lib`. Its outer wrapper's failed label is preserved:
the wrapper expected a literal `Checking` line, while Cargo compiled the fresh
engine and exited zero. This compiler result adds no current-main Browser or
gameplay qualification.

Earlier frozen Source C capacity evidence retains its own artifact scope.
The native port Rust/C ABI replay observes a complete 2,970,551-byte packet
with 6,368 paint records, literal complete-owner/wire/store agreement and successful write
continuation. The over-bound replay observes 13,334 footprint records, one
failed capacity acknowledgement, unchanged older stored bytes and continued
Tick progress. Existing shipped-WASM differential replay passes; it does not
observe complete direct WASM SavedRun/Profiles owner bytes. Production Browser
over-bound capacity passes separately. The under-bound Browser wrapper stops
with `KeyError('fatal')`; its failed status and raw output remain preserved.
These C results do not become main715 capacity gameplay evidence.

The later H composition has its own fresh public-main134 artifact `7ddc18cc`
(WASM `a3f2c0bb`). It passes `check:build`, ABI/constructor checks and all four
Browser runtime controls. One unchanged fixed case reuses the full retained
Original input bank: all 2,384 ordered expected/actual/status rows equal the
older dbfab H candidate, **2,349 pass / 35 fail**, with zero lost passes,
missing/new rows or changed Native expectations. All 33 own-save guards pass;
all pixel metrics and source PNG hashes remain identical (17 pass / 14 fail).
The 35 required failures remain 28 ambient velocity words and seven canonical
scope rows, so the actual canonical invocation still exits failed. The earlier
archive-parent wrapper preflight failure remains preserved separately.

That main134 capture also passes real natural same-origin reload and
own-storage Continue: the stored seed is exactly `T1PK 6Y0W`, the restored live
seed stays `T1PK6Y0W`, and the owned penalty stage is zero. After public Continue, the player
resumes at the valid x396 door inset; the public movement then takes the
reciprocal room5-to-room6 transition. This is the captured valid door branch,
not a same-room movement or Original-process reload claim. No current134
baseline artifact, OS Save replay or ordinary-run replay is claimed; the
ordinary consumer's actual dbfab and historical715 results remain below.
The actual canonical main134 affected preview selects 52 suites. The H publication
composes onto later main `3b08711d`, preserving Mega Fatty and the two unrelated
quality-tool changes; these main134 tests do not become a gameplay grade for
that later source tree.

Persistent seed and penalty owners (H) have a separate fresh qualification on
snapshot `dbfab9b0`: baseline `9e3c05e5` and shared H/tooling artifact `70cb3285`
pass `check:build`, ABI/constructor checks and all four Browser runtime controls
each. The candidate's actual affected preview selects 52 suites. The shared artifact's portable test files
are separate from the seven H implementation/consumer paths. These results do
not qualify the later main134 integration.

On the same retained fixed Original case, the 2,384 ordered comparisons improve
from **2,321 pass / 63 fail** to **2,349 pass / 35 fail**. Exactly 14 saved-seed
separator rows and 14 last-penalty-stage rows turn green, with zero lost passes,
missing/new rows or changed Native expectations. All 33 own-save guards pass
per role. Every pixel metric and source PNG hash is unchanged. The 35 required
failures remain 28 pre-Quit ambient velocity words and seven canonical scope
rows; both suite exits remain failed. The recorded OS case was validated in the
whole input bank but was not selected for this dbfab pair.

The edited ordinary-run driver also runs on both fresh artifacts against the
same retained `zero-epoch` Native case. Both pass all 801 saved-Player checks,
the observed shared-MT cursor/624 words and required grid/owner comparisons.
The candidate exercises the literal nine-character saved-seed guard; the
baseline retains its older eight-character normalization. No new Native
process or used input was drawn; the unchanged ordinary driver still makes
four unused OS draws before loading the recorded scenario.

Both fixed-case natural same-origin reloads and own-storage Continues pass.
The stored documents differ in exactly three values: the run and summary seed
strings gain the canonical space, and `lastPenaltyStage` changes from null to
zero. Both resume with compact eight-character live seeds. The saved entry is
identical, but fresh constructor seeds, epochs and complete shared-MT states
differ. Observed positions match different valid door placements allowed by
unchanged placement code: baseline starts at x396 and subsequently crosses to
room6; H starts at x84 and moves within room5. The local selected slot and draw
are not directly observed. This is not an identical-boot counterfactual or
Original-process reload parity.

H formats the persistent seed from validated numeric ownership while keeping
the live codec compact. Continue accepts only the canonical legacy eight-character
form or its exact nine-character form with one ASCII space after character4,
both matching the saved numeric context. Fresh admitted penalty state starts
at zero; missing or explicitly unknown legacy state stays unknown. Damage,
cross-stage and Restart lifetime coverage beyond these initial-room controls
remains separate.

Historical H artifact `65178aba`, on frozen main715 plus Save6, passes build
and runtime 4/4. Its unchanged fixed and recorded OS comparisons retain all
3,748 rows and improve **3,650 pass / 98 fail** to **3,706 pass / 42 fail**:
28 seed-text and 28 penalty-stage gains, with zero lost passes, missing/new rows
or changed Native expectations. All 66 own-save guards pass; pixels remain
17 pass / 14 fail per case. The 42 required failures remain 28 fixed-case
ambient velocity words and 14 canonical scope rows; both suite exits remain
failed. Its ordinary-run driver subsequently passed the retained `zero-epoch`
case. A later audit of both complete raw reload reports, checkpoints and durable
archives also proves both historical natural reloads passed. This explicitly
corrects the earlier H readout and PR697 notes that labeled ordinary-run and
natural reload unqualified; those earlier records remain preserved. Neither
historical artifact establishes later-main, Factory/Get or whole-game parity.

## Earlier live scope

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

The `1b30a0fb`/`0af25692` strict paired result remains preserved as
**FAIL: 1,592/1,714 rows pass, 122 fail**.
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

The earlier captures exposed seven continued Native ambient bodies in the
fixed case and two in the OS case, while `1b30a0fb`/`0af25692` restored none.
The successor above implements that saved-ambient owner; these earlier reports
retain their original artifact scope and failures.

Private readouts and journals remain local. The public retained bank contains
only the declared inputs and original OS draw receipt. It carries no Native
readback, captures, disassembly, copied room resources or storage payload.

## Reproduction

```sh
npm run test:e2e -- save-continue --count 1 --no-build --production-directory FROZEN
npm run test:e2e -- save-continue --scenarios references/isaac/cases/save-continue/recovery-initial-footprints-fixed-dl17.json --no-build --production-directory FROZEN
npm run test:e2e -- continue-run --no-build
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
