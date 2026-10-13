# Combat and grid source evidence

The native world kernels in `engine/src/grid.rs` and `engine/src/combat.rs`
match 100 retained original checkpoints across eight conditional cases. The
component-only harness is now ignored research. Its earlier checks establish
the supplied grid, trajectory, contact
and ordinary damage queue contracts. They do not establish enemy AI, the full
collision manager, rock drops, death callbacks or browser combat integration.

The production contact/grid follow-up now admits ordinary authored rocks and
uses the persisted room grid for both player and regular-bomb ground contact.
`grid::for_room` consumes the STB asset, selected local-stream spawn descriptors,
room decoration seed and admitted doorway geometry. It preserves unsupported
selectors. Ordinary1000/0/0 rocks receive native kind2/state1/class3 with their
actual local spawn seed; tinted/special rock callbacks remain outside this
slice. Boundary and door entries describe collision geometry with seed0 and
do not claim those actors' random admission. Opened doors use class5: players
admit0/5, while ordinary nonplayer ground actors admit only0.

Rock SetType(-1) at `0x56c240` seeds RNG35 with
`(decorationSeed + gridIndex) | 1` and chooses one of the three authored normal
frames. Init at `0x56c390` uses the rock's own spawn seed for a conditional
cluster draw, queries right/down/diagonal neighbors in order, and assigns
horizontal1000/1001, vertical1002/1003 or square1004..1007 frames. Production
floor generation, STB lookup, local selector draws and this factory reproduce
all45 kind/variant/state/collision/spawn-seed fields from the independently
observed nine rocks in Basement181. The source-bound summary is
`references/isaac/foundation/combat/bomb-runtime/rock-admission-verification.json`;
this factory comparison is separate from live traversal acceptance.

The browser displays those admitted rocks through the original
`gfx/grid/grid_rock.anm2` normal/big clips and Basement sheet. Regenerate its
catalogue with `extract-animations.py --archive PATH/animations.a --select-file
tools/foundation/port-grid-animations.txt --json-output web/grid-data.json`.
Rock destruction/drop/linked-frame updates, other hazard callbacks, general enemy AI
and full frame pixels remain incomplete.

## Ordinary active spikes

Authored selector1930/variant0/subtype0 now admits native Spike kind8/state0,
collision class0 and its actual local spawn seed. Graphics configuration
`0x56f830` seeds RNG35 from that spawn seed and selects `Next()%4+1` once.
The original `Spikes01` through `Spikes04` clips each contain one static
15-frame layer exposure; the browser uses those authored clips and sheet.

Spike update `0x56fb80` calls grid damage helper `0x561100` during the full grid
pass before actor updates. Ordinary active contact compares the player's center
against the grid center with strict squared radius21, without adding the player
collision radius. Successful fresh-Isaac damage removes two red HP units,
starts120 presentations of immunity, and adds the rounded normalized radial
vector multiplied by4 to existing velocity. Player motion then integrates that
velocity and decrements immunity in the same presentation. Intermediate
presentations do not run spike contact.

The recorded OS-random22PFCSFS route reaches Basement279, whose spike cell37
is at world `(320,200)`. The original pre-hit presentation1336 has player
`y=184.07395935058594`, `vy=3.457782745361328`; full presentation1337 applies
the upward impulse before player motion, yielding `y=183.53173828125`,
`vy=0.06996197998523712`, HP4 and immunity119. This retained failure originally
had browser HP6. Its dense replay and a one-presentation offset are retained
separately; original observations never enter production state.

The offset replay also exposes source control precision after reverse
knockback. `0x529926..0x529990` repeats the forward dot after transverse
damping, computes `max(forward*-0.2,0)`, then rounds `+3`, `*0.4` and each input
component's `*0.5` separately before velocity addition and drag. The previous
algebraic collapse to `0.6 + max(-forward*0.04,0)` lost one ULP in four retained
velocity comparisons; those failures remain preserved.

The final immutable production artifact passes all143 functional checkpoints
across both spike approaches and the contrived even/odd moving-bomb and pause
cases. The grid comparator accepts all5049 selected spike metadata, derived
world-center, room-dimension and phase checks. Existing nine mask cases retain
all18 complete targets exactly, with zero budgets. Replay cases and the
source-bound verification summary are in
`references/isaac/foundation/combat/spikes/`. Run the actual two-game replay
with `npm run test:e2e -- --random-count 0 --replay-cases
references/isaac/foundation/combat/spikes/22PFCSFS-offset-0.json,references/isaac/foundation/combat/spikes/22PFCSFS-offset-1.json`.
`tools/foundation/grid-state-pair.mjs --case CASE_DIRECTORY --kinds 8 --output
REPORT.json` then checks each declared gameplay observation rather than
silently omitting missing or failed source rows.

This slice covers regular active spikes and fresh Isaac's ordinary damage.
Retracting spikes, sacrifice-room rewards, flying/item exceptions, hurt
presentation/audio and nonplayer spike damage need their separate mechanisms.

## Source and reproduction

The reference is base Rebirth Linux v1.05. The unmodified private ELF has
SHA-256 `c1b2ca9375d60d4354f9086871913901428af41b82a5892a0d235c5d8ae2f768`.
The offline execution derivative has SHA-256
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.
The addresses below refer to that non-PIE Linux image. Static recovery used
ELF load ranges, `.eh_frame` function boundaries, bounded disassembly and
constant/caller checks; labels identify recovered contracts, not guessed
exports. Binary files and temporary disassembly remain private ignored data.

Every retained run uses an isolated fresh profile, an actual public seed
readback and public X11 keys with auto-repeat disabled. The observer checks
the executable file hash before stopping its directly owned original child
at the blocked public GL presentation. It reads only bounded writable data
through a read-only process-memory descriptor, with a 1 MiB cumulative budget,
and resumes the child in `finally`. It never writes native memory or reads
executable process mappings. The checked-in archive preserves selected raw
checkpoints, native counters, public input events and original manifest hashes.

Run both games live after sourcing `cloud/env.sh`:

```sh
npm run test:e2e:paired -- --scenarios paired-room-combat --random-count 0
```

`tools/original/foundation-combat-scenario.py --case rock-held --output PATH`
prints the original runner command, including the hash-checking observer.
The other retained cases are `rock-typed`, `rock-dense`, `fly-aligned`,
`fly-bomb-phase0`, `fly-bomb-phase1`, `attackfly-entry` and `gaper-entry`.
The routes are paired with their observed seeds; another seed can select a
different neighbor. The reusable matrix builder accepts another seed for
discovery without asserting the same room contents.

The live paired gate retains source read-only observations, actual consumed
input edges, browser command traces and pixel diagnostics. Its selected
functional comparisons remain failures until the corresponding runtime hooks
are implemented. The retained component observations below remain source
evidence, not current browser combat coverage.

## Accepted native holds

The seven fresh source runs retained for this checkpoint repeated the earlier
discovery independently. Numeric expectations were not regenerated from the
port. Their raw selected observations, immutable manifest/proof hashes and
input tapes are in `references/isaac/foundation/combat/`.

| Kernel case | Original seed and source scenario | Accepted observation |
| --- | --- | --- |
| `rock-typed-blast` | `T1PK 6Y0W`, right neighbor Basement 181 | Bomb at world `(80,329.9935302734375)` destroys rock cells 91,106,107, including cells whose centers lie outside radius 75. |
| `rock-dense-blast` | Same seed, independent horizontal approach | Bomb at `(289.6022644042969,280)` destroys cells 51,67,83. |
| `rock-held-movement` | Same right neighbor, held S | All 32 position, velocity, contact flags and normal checkpoints match bit for bit, including alternating contact/release presentations. |
| `rock-dense-movement` | Independent D48 then release30 | All 30 coast checkpoints match, including world `x=289.9996643066406`, reflected `vx=-0.07932156324386597` and tiny lateral velocity. |
| `attackfly-mass-contact` | `3BMB MFTE`, lower neighbor Basement 17 | Player mass 5/radius 10 and Attack Fly mass 3/radius 17 yield player `y=213.7989044189453`, `vy=3.3062233924865723`, NPC `y=240.7989044189453`, `vy=2.5432488918304443`. Actual player HP falls 6→5 with countdown 60. |
| `fly-aligned-queue` | Same seed, upper neighbor Basement 277 | Tear 3.5 is pending at system counters 1429/1430 while HP stays 3; next full update 1431 drains HP to −0.5. |
| `fly-bomb-phase0-queue` | Same upper room, intermediate birth | First explosion E+120/counter1501 queues 60; HP remains 3 through1502, then drains to −57 at1503. |
| `fly-bomb-phase1-queue` | Same room, input delayed one presentation | First explosion E+121/counter1503 queues60; HP drains at1505. |

The accepted grid fixtures use actual existing native grid objects and their
collision classes. Empty-cell selector costs were normalized to zero for this
conditional contract, so those fixtures do not prove navigation costs. Fresh
observer reports also retain the complete native selector array for future
room ingestion. Gaper appearance, chase and obstacle routing remain discovery:
`GTJ6 17ZW` reaches Basement variant5 with two Gapers; their routes differ
around obstacles, so a direct chase approximation would not establish parity.

## Recovered source contracts

| Linux boundary | Source behavior used by the kernels |
| --- | --- |
| `0x5c6fa0`, `0x5c78b0` | Grid index floors `(x−40)/40+0.5`, `(y−120)/40+0.5`; centers are `(40+40*column,120+40*row)`. Out-of-grid index is −1. |
| `0x5c7080` → `0x5c5af0` | Grid object collision class `+0x3c` wins. Empty selector3000 maps1, selector≥1000 maps3, otherwise0; outside maps4. |
| `0x554490`, grid branch `0x5548e2..0x554c63` | Bomb selection uses `ceil(radius*8/40)` rays, iterative f32 angles, 20-world-unit increments capped at the radius, sorted unique grid indices and a line check before virtual Destroy. Grid kind3 stops a ray. Center distance alone is insufficient. |
| `0x5c74a0` | Mode2 line check advances10 world units: kind3 blocks; ordinary rocks are allowed; doors kind16 bypass the ordinary collision-class test. |
| `0x446840`, `0x443d00` | Player collider caches40 samples with even angular indices first, then odd. Ground-player admission allows classes0/5. The aggregate blocked normal is normalized before source reflection and12 penetration refinements. Prediction corrects position without changing velocity. |
| `0x5298f4`, `0x529926..0x5299b2` | Control rounds transverse damping, reversal dot/acceleration operations and final `0.8803408145904541` drag separately. Algebraic collapse loses the strict post-knockback holdout by one ULP. Players use this law in full and intermediate phases. |
| `0x445710`, `0x4453a0` | Physical overlap is strict `sqrt(distanceSquared) < radiusSum` after broadphase. Both collision callbacks precede displacement. Source mass splitting halves only the player's velocity correction. A grid-blocked player transfers its displacement share to the NPC. |
| `0x445af8..0x445b95`, `0x612860` → `0x612820` | Statically recovered fast-separating branch normalizes with rounded reciprocal multiplication, then uses `delta + (delta*dot)*2`. The later impulse operator `0x612b30` uses direct component division. This branch has no independent dynamic holdout yet. |
| `0x49b8f0`, player branch `0x49c042..0x49c0aa` | NPC callback supplies native collision damage `+0x48c` to PlayerTakeDamage. Attack Fly18 has1, ordinary Fly13 has0; type alone is not an inferred damage table. |
| `0x446cf0` | Ordinary actor damage appends an event and updates pending sum `+0x378`/flags `+0x37c`; it does not immediately subtract HP. |
| `0x447190`, subtraction `0x44789d..0x4478ac` | Base actor update drains pending events and subtracts HP `+0x484`, retaining negative overkill. NPC update invokes it at `0x4f9eb8`. The kernel exposes the result; AI death/removal callbacks remain caller-owned. |
| `0x5d12a0`, `0x454df0` | Full update processes grid, then the active actor array in admission order, including players. The separate player pass at `0x454f40` runs only when interpolation is disabled (`0x454ef3..0x454efa`). The manager snapshots its original last actor at `0x454e6d` and stops at that pointer, so appended births do not update in that same full phase. |
| `0x5cd920`, `0x595490` | Intermediate phase renders/predicts nonplayers while players update. System counter `System+0x2efa0` advances globally; entities do not reset phase on birth or room entry. |

The diagnostic bounds (448 cells, 128 samples, radius4096 and36000
presentations) protect reusable tooling. Radius4096 is an admission limit,
not a recovered gameplay explosion tier.

## Implementation and remaining scope

`Grid` accepts a supplied descriptor and exposes indexing, collision lookup
and blast candidates. `Collider` caches source samples and resolves native
world player contact. It does not implement the room bound clamp, all grid
classes, every special-grid callback or rock destruction side effects. The observed
ordinary Rock destroy boundary `0x56c670` moves live state1/class3 to
state2/class0 and writes selector900, but drops, particles, linked variants
and RNG draws need their full contract before production adoption.

A fresh coasting-bomb holdout checks the admission interpretation directly.
Before E, the starting room's active array contains player sequence0. Later
it contains player0, bomb3 and explosion5 in that order. With E before system
counter1323's intermediate phase, the bomb explodes at E+120/counter1443 after
the player's full update: native player position remains439.18450927734375
and velocity becomes15. The next intermediate presentation E+121/counter1444
integrates position to454.18450927734375 and applies drag to13.20511245727539.
The current legacy E+121 blast-before-motion path reproduces the latter
trajectory but does not establish the actual full-phase actor order. The
general handoff must preserve admission order and both source checkpoints.

`resolve_player_npc_contact` assumes the manager has already admitted the
pair, including removal/class rules and callbacks. `DamageQueue` supports
observed ordinary tear flags0 and bomb flags4, drains FIFO on `Phase::Update`
and retains overkill. Special damage flags, cooldown events, co-op scaling,
full pair ordering, actor removal, enemy AI and spawning remain separate.

Bomb Init `0x456e00` writes timer59 at `0x456e35`. Update `0x457060` checks
`timer <= 0` at `0x457379` before decrementing at `0x457391..0x457394`, so
60 admitted full updates produce a blast. A birth during the player's full
phase action is excluded by the entity birth fence; this explains both
E+120 and E+121 source cases. Production phase integration must preserve the
nine existing starting-room bomb mask checks before replacing their legacy
presentation cadence. See `docs/bomb-evidence.md` for the accepted blast and
rendering scope.
