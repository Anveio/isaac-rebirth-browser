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

This selection leaf stops before acquisition. The mapped normal acquisition
plays the authored 42-frame `Pickup` animation and queues AddCollectible; it is
not a fitted nine-frame `LiftItem` wait. Player acquisition, cache/stat effects,
familiars, active use, trinkets, saved unlock progression, paid contexts, room
overrides, and pixels remain unaccepted in this leaf.
