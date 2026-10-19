The browser now initializes the run-owned item pools and admits natural free
collectible pedestals from their authored selectors. Configuration comes from
the pinned base `items.xml` and `itempools.xml`: 343 sparse collectible IDs,
497 ordered entries, and 16 native pool slots, including the empty challenge
slot absent from the 15 authored XML pools.

`Seeds::SetStartSeed` owns the startup xorshift stream. It initializes thirteen
run fields; the next actual caller initializes ItemPool. The manager seeds its
two root streams, all 32 child streams, and its pill mapping through their
source consumers. This path consumes no shared MT draws. Natural pedestal Init
uses its spawn seed in local RNG35 and passes its first output to local RNG4
selection. Fresh-profile eligibility uses 179 achievement bytes, independently
observed as zero in the original, rather than treating every item as unlocked.

Selection preserves immutable weight intervals and f32 arithmetic. Depletion,
both ID maps, special-item probability, eligibility redraws, treasure fallback,
and the local Bible override follow the recovered source branches. The selector
does not normalize current weights, compensate seeds, or consume shared audio
draws. Exceptional retry/fallback boundaries and a nonzero Bible modifier still
need dedicated live scenarios.

Pedestal age uses the ordinary Game-update clock minus its creation clock.
Room loading can call actor animation updates without incrementing that Game
clock. The live gate compares actual birth ages and retains each game's absolute
counter values separately; the native cutscene/run-admission counter difference
is a diagnostic and never an input to WASM.

Run the actual pair with `cargo build --manifest-path engine/Cargo.toml --locked
--bin floor`, then `bash tools/original/with-display.sh node tools/e2e/items.mjs`.
The default cohort includes two contrived phase offsets and two newly recorded
OS-random public seeds. Discovery selects the complete authored treasure-room
layout, with no filtering of selected item IDs or unlock/redraw outcomes. Every
presentation compares all pool entries/maps and the natural pedestal. The gate
also requires the independent complete DOM command plan and actual manual-seed
readback. Original state never supplies browser behavior.

The retained complete cohort passed all four pairs across 542 presentations and
10,296 captured checks. A separately hashed verifier replays those exact raw
native manifests and full browser state archives with 7,588 additional required
room-context checks. It joins actual native room type/dimensions and public
variant/name/spawn seed to the current browser descriptor, and checks the port's
descriptor/floor consistency. The live gate now requires those same checks.
The original comparator and source hashes remain in the preserved report; no
recapture or runtime modification was needed for the stronger assertions.
Native descriptor index and appearance origin/backdrop were absent from these
observations and remain unaccepted. The earlier rejected 3/4 cohort retains its
inactive-queue observer failure and its actual captured bytes.

This selection leaf stops before acquisition. The mapped normal acquisition
plays the authored 42-frame `Pickup` animation and queues AddCollectible; it is
not a fitted nine-frame `LiftItem` wait. Player acquisition, cache/stat effects,
familiars, active use, trinkets, saved unlock progression, paid contexts, room
overrides, and pixels remain unaccepted in this leaf.

The first-acquisition slice routes actual free contact into the Player's deferred
config queue. It copies `PlayerPickupSparkle`, plays the authored nonlooping
42-frame `Pickup` clip, then grants permanent counts after the real full-update
completion boundary. Ordinary motion continues and the animation guard suppresses
normal shooting. Authored inventory and normal Isaac health helper calls retain
native ordering; the observed first active item 291 retains charge 6. Health caps,
parity and nonzero health grants remain unaccepted in this cohort.

The stopped main `Empty` frame 0 preserves the old `Alternates` overlay. Successful
collection returns 1 and vetoes mass response; later free subtype 0 returns 0 and
uses the common class 4 Player/Pickup mass kernel. The retained altar has separate
physical and presented positions. Full updates return it toward its authored
anchor using current friction, restore configured friction, then multiply by
native f32(0.965). Intermediate prediction uses stored velocity/current friction
without advancing physical position, sprites or contacts. Positive Player
collection delay decrements on both callback paths: the fast branch rejoins at
`531e7f` before `531edb`; only the later `531f01` test guards queue flushing.

[The corrected four-case evidence](../references/isaac/foundation/items/acquisition-correction-live.json)
passes all 70,696 required checks across 1,070 source/browser checkpoints. Both W0
phase offsets and recorded OS cases `R74QYMYS` and `VRQPT844` observe an actual
queue followed by grant. This is a new live production-browser recovery against
the exact complete original recordings from the current-turn initial cohort,
not a fresh native recapture. Raw SHA/byte counts, public tape/X11 provenance,
complete intended browser commands and every asynchronous backend event's full
state are required. Independent per-presentation snapshots remain complete even
when bounded production state history is truthfully truncated.

A separately hashed verifier recomputes 3,112 pedestal f32 comparisons and 5,350
queue/count/guard/timer comparisons. Only native/Rust f32 position, velocity,
friction and collision mass recover exact binary32 values from serde's shortest
JSON decimals. Integer IDs, seeds, counts, counters and ages remain unchanged.
The initial 0/4 actual paired report retains 3,075 failures; the following recovery
retains 902 raw decimal-spelling differences and 17 real active-item timer errors.
Those failed reports and original mapping bytes remain intact.

e2e suite `item-acquisition` is required by the live suite registry and runs both
real games freshly. Its default bank declares complete contact/grant intervals
for both contrived offsets, both retained unfiltered OS outcomes, and two newly
recorded unfiltered OS treasure routes. Selection remains a separate gate that
stops before contact. `--replay-original DIR` discloses frozen-original recovery
and checks exact raw bytes and requested tape/action hashes; queue, grant,
provenance and backend-state requirements are identical in both modes.

All 450 initialized Player RNG children are checked before acquisition. Later
item RNG mutations, complete shared-MT state, stat/familiar/cache callbacks and
post-grant item effects remain explicit raw diagnostics. Requested cache bits
stay `unevaluatedCacheFlags`; an OR/clear cannot stand in for actual evaluation.
Rosary and authored helper code have mapped source boundaries, while exceptional
selector/Bible, health clamp/parity, active swap/use, altar grid-impact/prediction
correction, native-origin/backdrop and pixel acceptance still need their own
required cases.

The separate trinket manager implements Get582180: one manager shift2 draw
seeds the local shift35 retry stream. It tests31 candidates, consumes the
unused last retry, then takes the fallback draw and increments before scanning
all62 cells. Depleted signed counts reset from the actual current profile;
unknown legacy availability or RNG words remain unknown. A real zero return
stays zero and belongs to Pickup.Init's separate card-conversion owner.

An immutable whole-batch admission checks every possible Kind2 selection before
any Get, including reached resets. Explicit nonzero Init350 selectors bypass
ItemPool. Unsupported subtype0 births invalidate both availability and manager
RNG knownness; explicit nonzero births do not. This adds selection ownership,
not natural350 body admission or retail Original parity. The process type5
allocator increments and masks its cursor before inspecting active cells;
reused cell bytes survive across allocations and room/run boundaries. Its
complete allocation, active-skip and reclamation history remains a prerequisite
for new350 body admission. DST properties check selection conservation,
profile readsets, true0 and unknown history; live Original Get remains open.
