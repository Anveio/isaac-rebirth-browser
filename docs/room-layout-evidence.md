# Recovered floor-growth and room-template contracts

The Rust kernels in `engine/src/layout.rs` and `engine/src/room_templates.rs`
accept independently supplied generator or selector seeds. The earlier bounded
`layout` diagnostic binary is now ignored research. Gameplay still uses its
explicitly provisional room routing: these kernels do not yet generate a
complete floor from a run seed or enable traversal.

Evidence comes from the pinned Linux original, eight independently captured
floors, its actual seed/room/SpawnRNG logs, and a bounded read-only descriptor
observer. `references/isaac/foundation/room-layout/native-observations.json`
retains logical observations; the adjacent original logs retain their actual
bytes and SHA256 provenance. Each adjacent controller JSON retains its actual
tape, public X11 inputs, consumed event acknowledgments, clock and paired
gameplay-start observation, with local paths omitted. The verifier checks its
byte hash and pairs seed/frame/time/context against the actual gate ack and
consumed tape. Original addresses and private cache paths are excluded from
these portable observations. The reusable read-only observer remains checked
in; the earlier component verifier is now ignored research.

The source executable SHA256 is
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`,
after the repository's existing Steam-wrapper restoration. The wrapped depot
executable SHA256 is
`c1b2ca9375d60d4354f9086871913901428af41b82a5892a0d235c5d8ae2f768`.
Instructions were decoded from this Linux executable; the wrapped Mach-O text
is not used as formula evidence.

The descriptor observer validates the native binary hash, direct-child
ownership and blocked presentation phase before opening only writable data
mappings through `/proc/PID/mem` with `O_RDONLY`. It stops the child only for
the bounded read and resumes it afterward. No process memory is written.
The current descriptor must match the original's actual SpawnRNG log.

The Level table starts at member `+0x10`, descriptor stride `0xf0`, count
`+0x8144`. Its 13×13 grid is `+0x7ea0`; starting/current/previous grid indices
are `+0x8148/+0x8188/+0x818c`. Descriptor `+0x10` is the required door mask,
`+0x18..+0x34` its eight destinations, and `+0x48/+0x4c/+0x50` decoration,
spawn and following RNG12 words. `place_room` at `0x584bc0` seeds a local RNG12
with the supplied init seed and draws exactly these three words. RoomConfig
headers are asset-stage/type/variant at `+0/+4/+8`; the Special Rooms starting
record is `(0,1,2)`, not an ordinary Basement asset.

For normal Basement I without curses, the recovered caller prefix is:
descriptor reset `0x587ee2` discards one primary RNG35 draw; backdrop
`0x58beca` consumes `Random(1)`; count `0x58bf06` consumes `Random(2)` and
requests `8 + result` additional rooms; constructor call `0x58c0ba` consumes
the generator's seed. The generator owns an independent RNG35 stream. The
reset also consumes 120,960 words from the separate global MT stream while
initializing 135 descriptor slots × 448 grid descriptions × two words.

Rectangular growth follows native `0x5910b0`, candidate builder `0x5907c0`,
shuffle `0x5923d0`, dead-end classification `0x58ee80`, and expansion
`0x5918c0`. It starts with a 1×1 room at grid cell84. Shape availability bits
cover 1×1, 1×2, 2×1 and 2×2. Each viable nominal neighbor consumes five
`Random(20)` large-shape gates before availability and fit checks; candidates
then undergo descending Fisher–Yates. Center child count is `2+Random(2)` for
requests up to15; other parents consume four `Random(2)` draws. The desired
limit is checked after insertion, so a zero sum still permits the first fit.
Successful children consume `Random(3)`, pushing the room origin to the front
for nonzero and the back for zero. Dead-end flags persist and incoming parents
are cleared. Up to five expansion attempts insert the first fitting candidate
in creation order, with no shuffle/child-count/queue draws.

Weighted selection follows filter `0x5d54d0` and picker `0x5d5600`. It preserves
authored order and filters by type, shape (4 means any), inclusive variant and
difficulty bounds, and required-door subset. A local RNG7 draws float A and,
when an exact-mask subset exists, float B. The exact subset is selected when
`B < (exactWeight / compatibleWeight) * f32(9.99)`; all sums, ratios and
products use native f32 arithmetic. Strict cumulative weight `> A*poolWeight`
selects the record. Native `0x5d5878` then multiplies that record's current
weight by `f32(0.1)`, preserving authored weights separately. The constructor
of a new picker initializes its mutable weights; cross-floor weight-reset
routing remains a separate caller contract. Those current weights must also
be validated and serialized in saved Level continuation before gameplay wiring.

`basement-templates.json` contains all572 authored Basement record metadata
entries in exact STB order, joined to the complete pinned `rooms.a` archive.
The verifier reconstructs it from that original archive and compares every
field. Selected catalogue indices remain available to resolve complete spawn
groups; the kernel does not drop grid selectors or manufacture generic NPCs.

| Original seed | Ordinary templates | Finalized rectangles vs growth | Native expansions |
| --- | ---: | --- | ---: |
| N4Z8 GWV3 | 3 exact | exact | 0 |
| W2K0 3EEH | 4 exact | exact | 0 |
| ZY99 7RMD | 4 exact | exact | 0 |
| M9QM YR3N | 5 exact | exact | 2 |
| GTJ6 17ZW | 4 exact | exact | 0 |
| MZ69 9MM1 | 4 exact | exact | 1 |
| MPCM QQN6 | 4 exact | unresolved shop resizing | 0 |
| BLH3 34RQ | 4 exact | exact | 1 |

All32 ordinary template identities match independently supplied native seeds,
shapes and door masks across all four room shapes. Seven finalized rectangle
sets match. MPCM QQN6 retains the explicit failure: growth yields a 1×2 room
at49 with connector62, while native special assignment creates a 1×1 shop
at62. Its other nine nonsecret rectangles match. The source assignment helper
`0x58f480` can resize/reposition generated rectangles; that behavior is still
unimplemented. The verifier reports this mismatch and exits nonzero for
`--contract growth` or `--contract both`, without an exception for this seed.

A further live run of the checked-in observer and Rust diagnostic, using fresh
OS-random seeds7X7B GA4Y andGS9E 16DG, matched six ordinary templates each:
44 total identities across ten independent originals. 7X7B also matches its
finalized rectangles and two expansions. GS9E preserves another finalized
growth failure: a generated 2×2 at56 becomes an ordinary 1×1 at70 after special
room fitting. Resizing can therefore affect a room that remains ordinary;
admission and rejected-placement restoration are part of the open contract.

The earlier base-only growth prototype failed M9QM YR3N by omitting its two
dead-end expansion rooms. That failed comparison remains in ignored RE notes;
the native general expansion rule closes it, and later random-seed holds
exercise zero, one and two expansions. Omitting native weight decay likewise
produced later-template mismatches; the actual source mutation closes them.

The component-only selector/growth verifier and diagnostic CLI are now ignored
research. The production kernels, authored tables, retained original evidence
and read-only layout observer remain checked in. Their earlier conditional
results do not establish browser floor routing. `npm run test:e2e` runs both
games live; room traversal failures remain visible until runtime hooks land.

Remaining work includes full primary stage draw consumption with real profile
and challenge predicates, special assignment/resizing, outer generation
retries, curses and other stages, secret placement, descriptor save/load,
doors/traversal, typed spawn instantiation and room-clear behavior. No
per-seed map or fixed stage-to-room draw offset is used by either Rust kernel.

## Caves II, the surprise-miniboss tail and non-1x1 boss rooms

Decoded from the Linux x64 original; the port's inputs stay the stage seed,
the retained game flags, health and coins.

- **Post-room tail (589feb..58a9d1).** Every successful placement attempt, on
  every stage, draws two RandomFloats (5c2db0, `next() * 2.3283064e-10`)
  before the 58c337 word. The chance is `base * 0.33333334`: `base` is 1.0
  (1.05 above 19 coins), plus 0.1 / 0.02 for flags 0x40000000 / 0x80000000,
  which are then cleared. Game+0x10eafc subtracts 0.05 per count (at most
  0.25); only payouts the port does not admit raise it (552650). The roll needs
  that no miniboss room was placed this attempt and that flags 0x18000 are not
  both set. The first float flags the secret room from stage 5 (with a local
  SetSeed shift-0xc R1000), the second the shop from stage 4. Descriptor+40 is
  the flag. The five off-grid descriptors after it (58a137..58a433) are not
  generated.
- **Entering a flagged room (586c90..586cc3).** The original loads a type-6
  template from the special set instead: Greed 2040..2049 while flag 0x8000 is
  clear (then sets it), otherwise Super Greed 2140..2149 (sets 0x10000); a
  flagged devil room takes Krampus 2300..2309. The pick seeds SetSeed(desc+0x4c,
  1), one next(), then 5d5600's SetSeed(s, 7) and one RandomFloat, and decays
  the picked weight by 0.1. Greed (50.0 / 50.1) spawns from the template at
  grid 67. The port does not perform this swap yet, so stage 4 stays
  unreachable (`next_stage`).
- **Boss-room fit (591b80 -> 58fad0 -> 58f480).** The boss room takes the
  first dead end farther than one step from the start for which one of six
  candidates around its connector `C` (incoming direction `d`, `o = (d+2)%4`)
  fits: c0 `(C, 1x1, o)`, c1 `(C+off, long, o)`, c2 `(C, cross, o+4)`, c3
  `(C+s, cross, o)`, c4 `(C+off, 2x2, o+4)`, c5 `(C+off+s, 2x2, unset)`.
  `off` is (-1,0) for d=0, (0,-1) for d=1, else (0,0); horizontal entries use
  long 2x1, cross 1x2 and `s = (0,-1)`, vertical ones the swapped axes and
  `s = (-1,0)`. A candidate needs the template's cell dims (8ed500: shapes
  0..3 -> 1x1, 1x2, 2x1, 2x2), its door in the template mask, only free or own
  cells, and at most one other room touching it, each touching slot's door in
  the mask. The dead end takes the candidate's rectangle; a failure changes
  nothing and retries the floor. These are the growth candidates of
  `layout.rs`. c5 reads a stale stack slot and is refused (2x2 only); Double
  Trouble (3700..3708) is 26x7 or 13x14.
