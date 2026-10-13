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

The following first-acquisition slice routes actual contact into the Player's
deferred queue. Contact marks the pool's removed ID, starts `Pickup` at source
speed1, copies the authored pickup presentation, and retains an empty altar.
Source40fef0 sets its stopped main `Empty` frame0; the `Alternates` overlay stays
at its saved frame. Treasure collection admits sound128 at volume0.5.
Ordinary movement continues during the animation; the animation guard suppresses
normal tear emission. Full updates advance the authored42-frame main clip and
36-frame looping copied sprite. The same full update that finishes `Pickup`
clears the guard and later flushes the queued config, charge, and touched
argument before the postactor contact pass. Fast phases advance none of these
animation, queue, or collection-delay lifetimes.

The Player's child streams derive from saved Seeds output13, through local
RNG5 `(1,11,16)`:347 collectible,62 trinket, and41 pocket children initialize
RNG35 owners, followed by the base entity's spawn seed. They consume no shared
MT draws. Grant increments the actual permanent count, then applies the authored
normal-Isaac inventory/health helpers in native order. A first active grant
stores its ID/charge and source30-callback collection delay. Rosary72 changes the
16 Bible override counters. Active swaps and use remain separate callbacks.

Use the same live gate with `--acquisition`. It requires an observed queue and a
later grant in every contrived/recorded case, compares all347 counts, inventory,
health, active fields, actual sprites and collection sound admission, and checks
ordinary motion through the queue. Every gameplay presentation has its own
complete snapshot. Bounded production trace truncation is disclosed separately;
the independent input tape must remain complete, and every actual asynchronous
backend callback must retain its full state. The baseline450 child RNGs are
required. Later item RNG mutations, full shared-MT state, familiar actors and
native cache flags remain explicit diagnostics until their actual callbacks
are implemented. Requested cache bits are retained as `unevaluatedCacheFlags`,
so an OR/clear operation cannot appear to implement native stat or familiar
evaluation. The initial four-case acquisition cohort retained68,362 checks and
3,075 mismatches, with all original transports/observations passing. Queue and
grant boundaries, owned counts, Player/copy sprite phases and sound admission
match, including recorded-random active291 at charge6. The complete paired gate
is red: source40fef0 actually sets the stopped main `Empty` frame0, and later
empty-altar mass contact/home motion is absent in this producer. An integer/bool
comparison also fails on the queued touched byte. All raw failed evidence and
exact source hashes remain preserved. No health clamp or familiar/stat/cache
effect acceptance follows from these cases; the active transition is observed.

The following correction applies the actual retained-altar physics. Successful
collection returns1 and vetoes mass response; a later free subtype0 callback
returns0. The class4 Player/Pickup contact then uses the common445710 mass
kernel, including its Player-only half-share velocity impulse and source grid
displacement check. Each full pickup update sets velocity to anchor minus
physical position, applies current friction, moves, restores configured friction
and multiplies it by the source0.965. Intermediate prediction uses stored
velocity/current friction without advancing the anchor, sprite or contact pass.
The model retains physical and presented positions separately.

`test:e2e:item-acquisition` is required by the suite registry. Its default generator
declares the complete contact and grant intervals for both contrived offsets and
new unfiltered OS cases, while the selection suite still stops before contact.
`--replay-original DIR` is an explicitly labeled recovery mode for a new live
browser replay against the exact earlier current-turn native recordings. It
checks raw bytes, tape hashes, and the complete requested actions; required CI
uses fresh original and browser runs. Original failures and source hashes remain
unchanged. Altar grid impact, health clamp boundaries and later item callbacks
still need independent required cases.

The positive Player collection delay decrements on both full and intermediate
callbacks: the fast branch rejoins at `531e7f` before `531edb`, while only the
later `531f01` test guards queue flushing. The first correction recovery retained
the active291 timer failure at all17 post-grant presentations; its failed raw
report and the earlier full-only mapping bytes remain unchanged.
