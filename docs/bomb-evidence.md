# Regular bomb evidence and implemented scope

The port implements fresh normal-mode Isaac's planted bomb: E placement,
one inventory debit, a paused fuse, authored Pulse/Explosion playback, one
full-heart blast hit, strict radial contact, native knockback, and a damage
countdown. Moving births inherit source velocity and use the global full/intermediate
phase. Enemy/grid damage, pushed bombs, variants and crater decals
remain separate work. This does not establish native simulation-tick parity.

## Public original scenarios

`tools/original/foundation-bomb-scenario.py` produces arbitrary-seed stationary,
escape and pause tapes and records the seed and actual presentation phases.
The registry in `foundation-clock-scenarios.py` exposes all three to the matrix.
Tests ran against fresh v1.05 profiles at seed `T1PK 6Y0W`.

E is reference-verified. Separate `Shift_L` and `Shift_R` probes consumed the
requested public key events but did not place a bomb: inventory stayed1 and
HP stayed6. This occurred with both window-targeted X11 delivery and independent
physical XTest delivery on isolated focused displays. Candidate Shift emission
was removed from the port. Alternate bindings/remapping remain unverified;
the historical catalogue's Shift alternatives are not proof of this default
Linux build's behavior. Native artifacts are under
`.cache/foundation/bomb-shift_{l,r}/{run,physical}/`.

* E consumes 1 bomb; the counter displays 0 by the second presentation. Empty
  inventory does not create a replacement bomb.
* In these starting-room tapes, the bomb appears for 120 presentations and the
  first explosion is presentation121 relative to E. A fresh traversal into
  Basement277 at seed `3BMB MFTE` instead produces its first blast at
  presentation120. Bomb Init stores timer59; the source checks zero before
  decrement, so detonation takes60 full updates after admission. A birth in an
  intermediate phase therefore detonates at120; a full-phase birth at121.
  The production bomb runtime now uses this global phase and birth fence.
* A stationary blast changes HP from 6 half-hearts to 4. W held for 50
  presentations escapes without damage. Pause preserves the fuse while the
  pause menu is displayed.
* D held for 19 presentations then released gives a settled preblast native
  center at `(403.81317138671875, 400)` and takes damage. D20 gives
  `(408.22662353515625, 400)` and stays healthy. The bomb begins near native
  `(320, 400)`. Only preblast positions support the contact measurement: the
  near-hit original then moves farther away through blast knockback.
* A regular bomb shadow uses authored shadowSize 13% and the original shadow
  PNG crop `(0,0,120,49)`. At the starting bomb the public submitted corners are
  `(232,210)`, `(248,210)`, `(232,216)`, `(248,216)`. The shadow disappears at
  detonation. The main explosion sprite anchor is logical `(240,213)`.

Working artifacts are in `.cache/foundation/bomb-{stationary,escape,exact,pause}/`
and `bomb-distance-{10,12,14,16,17,18,19,20,30}/`. The exact case includes a
continuous public GL trace. Position-observer candidates were independently
checked against the existing axial-motion evidence and rendered displacement.

## Static native path and exact contact predicate

The private original Linux executable is stripped. Function boundaries were
read from its ELF `.eh_frame` entries; candidate call paths were identified by
float-constant cross references and the authored `Pulse`/`Explode` strings,
then checked against public behavior. Binary addresses below are reference
evidence only; they do not enter production behavior or persisted game state.

Source SHA-256: `c1b2ca9375d60d4354f9086871913901428af41b82a5892a0d235c5d8ae2f768`.
Offline-derived executable SHA-256:
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.
The existing offline preparation records its validated wrapper restoration in
`provenance.json`; the supplied source executable is unchanged.

| Evidence | Native location | Meaning |
| --- | --- | --- |
| Bomb initialization | `0x456e00`, store at `0x456e11` | Regular explosion damage at object `+0x7d8` starts as f32 60 (`0x42700000`). Variant 6 takes a different branch and sets 110. An independent original writable-data probe confirms entity4/variant0, radius16, damage60 and native position `(320.1005554199219,400)` before the D19 blast. |
| Bomb update | `0x457060`, call at `0x45776a` | Reads damage `+0x7d8`, position `+0x448`, flags `+0x7e0` and calls the explosion helper before setting `Explode`. |
| Explosion helper | `0x555300`–`0x555380` | Damage ≤85 selects native blast radius 75; larger tiers select 90 or 105. The regular default damage60 takes radius75. |
| Blast processing | `0x554490` | Calls the circle collector, applies an outward impulse, then calls virtual damage. Player damage is 2 when original explosion damage is at least50. |
| Circle collector | `0x4536f0`, predicate at `0x4537a6`–`0x4537ef` | Adds entity collision radius `+0x474` to blast radius, squares it, and includes only entities with strictly smaller squared center distance. Equality is safe. |
| Player geometry | Authored entity 1/0; independent writable-data observation | Fresh Isaac radius10. Motion observations give logical distance = native distance ×0.65. |
| Player damage | Vtable `0x670c90`, slot `+0x38` → `0x52b080` | The fresh player damage path writes a countdown at object `+0xbf8`; normal damage sets rounded damage ×60. A bomb's full-heart hit sets120. Independent temporal observation confirms countdown120 on presentation121,119 on122,1 on240 and0 on241. |

The implemented fresh-Isaac contact envelope is therefore exactly
`(75 + 10) × 0.65 = 55.25` logical pixels, with a strict comparison. The bomb's
authored collisionRadius16 governs the bomb object, not the blast envelope.
Near/far native observations support the recovered predicate independently.

For a fresh player of mass5, the blast path at `0x554673`–`0x5547a5` applies:

```text
impulse = normalized(player - bomb) × min(75 + 10 - nativeDistance, 15)
velocity = previousVelocity × 0.5 + impulse
```

A zero-distance blast skips knockback. The port calculates the impulse in f32
native units and projects back to logical coordinates. Knockback precedes the
damage attempt and also applies while damage is blocked by immunity. The
production scheduler fences full-phase births and preserves admission order.
The starting player updates before a bomb it later places, so its blast changes
velocity immediately and the following player phase integrates that velocity.
Intermediate bomb presentation predicts position plus half velocity without
mutating actual position. No extra player decay is added.

An independent controlled original seed `3BMB MFTE` supplies the complete
D27/release30/E tape in
`.cache/coordination/stage-masks/explosion1/targets/mask-report.json`. At
placement, the player is at native x438.4042053222656 with vx0.09337755292654037.
Coasting settles to x439.18450927734375 before the blast. The port reproduces
all three following native position/velocity observations through ordinary
commands; the source fixture guards their complete displacement and drag:

| Presentations after E | Native player x | Native player vx | Original lamp x bounds |
| --- | --- | --- | --- |
| 121 | 454.18450927734375 | 13.20511245727539 | 177–477 |
| 123 | 479.0146179199219 | 10.23396110534668 | 194–494 |
| 133 | 540.6290283203125 | 2.8612258434295654 | 234–534 |

The renderer now applies the source-observed half-logical-pixel render anchor,
which reproduces these submitted lamp bounds. Correct numeric movement and
lamp bounds do not establish exact light-target pixels: the nine selected GPU
comparisons also exposed depth-buffer and alpha-pipeline differences. Their
source mask gate owns the current pixel results and any remaining differences.

## Verification and limits

Verification uses fresh original processes paired with the immutable production
browser/WASM artifact. Public X11 and DOM events execute the same recorded
presentation tapes. Even/odd moving births exercise held E, first blast,
HP/countdown and the following player phases; pause/release and recorded
OS-random tapes check controller freezing and resumed motion. Full RGBA
differences remain separately reported failures. The existing nine GPU mask
cases retain their zero-byte budgets. No bomb unit suite is checked in.

DropBomb `0x509540` runs after PlayerUpdate's movement/control. Action8 uses
the keyboard's rising-edge predicate at `0x62bb90`; spawn copies post-motion
position and velocity multiplied by f32 .1. Placement cooldown starts30 and
decrements only on full player updates; a decrementing update cannot admit
another placement. Bomb motion integrates old velocity, rounds the .95 drag,
then applies .5 drag when squared speed is below1. Its fuse, pulse, corpse and
explosion clocks advance only in full phases; pause still advances the global
clock while freezing gameplay.

The odd moving-birth holdout also reaches the right wall after knockback.
The production player now uses the existing40-sample source grid solver in
both player phases against the collision-only 15x9 boundary ring. At the last
holdout the native center is `569.9998779296875`, vx `0.18612265586853027`,
and vy `-7.182904227964551e-8`; the old logical clamp retained excessive vx
and discarded the lateral rounding. `motion::advance_in_grid` admits a real
room grid for the subsequent floor integration. This boundary profile has no
spawn RNG or admitted grid actors and does not establish door traversal.

Recorded cases and artifact/source hashes are retained in
`references/isaac/foundation/combat/bomb-runtime/verification.json`. All54
contrived bomb/pause checkpoints pass; all18 complete original GPU mask targets
pass. The full randomized gate remains failing: `EQCA3XZX` has three live tear
count differences from the provisional tear lifetime, and `C7YPWS61` has a
bottom-wall bomb contact difference. The native bomb moves from y409.993896 to
y407.803467 through its separate nonplayer ground-collision path; that response
is not implemented by this handoff, so the subsequent blast trajectory differs.
Both complete input cases remain replayable with the paired gate's
`--replay-cases` option. Raw original/browser observations and pixel differences
remain under `.cache/foundation/bomb-handoff-contact-live/`; no failed result
was replaced or weakened.

The subsequent ordinary nonplayer ground-contact path now resolves the
`C7YPWS61` bottom-wall holdout. Static `0x442b90`/`0x443628` evidence and a
new dense public-input replay distinguish it from player reflection. A full
contact first adds `normal * (1.8 * inward - .1)` to velocity, uses that
reflected speed for12 penetration refinements, then scales the whole vector
by `1 + (inward / reflectedSpeed) * .5`. Intermediate prediction corrects the
presented position while preserving stored position/velocity. The source's
first full wall hit supplies vy−.1, and the subsequent .95/.5 drag settles the
bomb at y407.803467. No captured positions enter gameplay.

`bomb-state-pair.mjs` reads fresh original and production browser manifests,
verifies their provenance, and compares the single admitted bomb's presented
world position, stored velocity, and timer. The dense replay passes42/42 paired
checkpoints and125/125 exact f32/integer field comparisons; all nine original
GPU mask cases still pass. Its complete tape and hash-bound verification
summary are beside the earlier failing evidence as
`C7YPWS61-dense-wall.json` and `nonplayer-contact-verification.json`. The
ordinary class5 kernel does not implement type-specific bounce exceptions,
other grid classes, or grid destruction.

The independent Fly-room probe at
`.cache/coordination/combat/fly-bomb/manifest.json` records player HP6→4 at
presentation120 after E. Two nearby Fly13 actors retain HP3 while pendingDamage
is60 at presentations120/121; at122 their base update drains the damage queue
to HP−57 and marks them removed, and by124 they leave the active list. Farther
flies remain healthy. This confirms a native update/render phase distinction
and queued enemy damage; it does not extend the current port's enemy support.
The existing starting-room121 blast references remain unchanged. Room-entry
admission and enemy/grid damage still require production integration.

The native initializer sets remaining fuse updates to59 at `0x456e35`.
Bomb Update checks for zero at `0x457379` before decrementing at
`0x457391`/`0x457394`, giving60 full nonplayer updates before detonation.
Independent `fly-bomb-phase{0,1}` captures add a single presentation before E:
the first case detonates after120 presentations, the delayed case after121.
Bounded writable-data observations record System iterations (`0x8ed560`
singleton, field `+0x2efa0`) and full Game updates (`0x8ecd30` singleton,
field `+0xce9f4`) alongside each fuse value. These source fields establish the
captured native phase; they are reference tooling and do not enter the runtime.

The typed bomb probe is limited to fields identified from the static path; it
does not establish all bomb internals. The object is in an anonymous writable
mapping, so earlier heap-limited probes missed it. The successful bounded scan
and original manifest are under `.cache/foundation/bomb-typed5/`; no memory was
written. The regular damage/radius is anchored by both that independent probe
and the initializer-to-update-to-blast path above. The independent HP/immunity
temporal probe under `.cache/foundation/bomb-health/` confirms HP6→4 and one
countdown decrement per presentation. A stationary blast leaves position
`(320,400)` and velocity zero throughout that tape; the D19 bomb's tiny lateral
offset occurred while planting/moving and is not applied as a general spawn
correction. Alternate characters/items and multiple damage sources are outside
this starting-room contract.

## Blast admission and the issue #154 divergences

Static reading of the pinned executable completes the blast-to-player path:

| Native location | Meaning |
| --- | --- |
| `0x45776a → 0x555300 → 0x555380` | Bomb Update passes damage `+0x7d8`, position `+0x448`, its tear flags `+0x7e0` and itself as source. The helper calls `0x554490` with line check on (`dl=1`) and incoming DamageFlags4 (`r9d`). |
| `0x554490` entity loop | Among others, skips the source, dead (`+0x31f`), zero-mass and tear (type 2) entities from the strict circle collector. |
| `0x554b68 → 0x5c74a0` | With the line check on, `Room::CheckLine(blast, target, mode2, 0, ignoreWalls=1)` must pass. Failure skips both knockback and damage. Mode2 samples every 10 units from the blast. It blocks on a type-3 block, continues through doors and (with ignoreWalls) walls, and otherwise blocks on a cell whose collision class (`0x5c7080`) exceeds 3. A zero-length line passes. |
| `0x554673`–`0x5547a5` | Knockback as recorded above. It precedes the damage call and does not depend on it. |
| `0x554d6b`–`0x554e25` | Player/familiar damage is 2 when explosion damage is ≥50, else 1. It calls virtual `+0x38` with flags4 and the bomb source. |
| `0x52b23b` | Player damage returns false while countdown `+0xbf8` is positive. No flag bypasses it. |
| `0x52bed6` (`0x52b2c4` Holy Mantle) | Accepted damage writes countdown `round(damage) × 60`: 120 for a bomb, 60 for a half-heart hit. |
| `0x56c670`, `0x56bf00` | Rock Destroy. A type-5 bomb rock destroyed by a blast (`immediate=0`) only enters state3. Rock Update counts `+0x348` down and then destroys it again. That explodes through `0x554490` with damage40, radius60 and flags4, so it hurts the player by 1 and its countdown blocks it like any hit. |

The port's regular-bomb amount (2), countdown (120), knockback-before-damage
and immunity rejection already match this path. Neither #154 divergence is a
bomb defect. Dense original replays of the recorded tapes, with captures every
3–6 presentations through `foundation-paired-observer.py`, show:

* `paired-room-combat` T1PK6Y0W. A Round Worm (244) at native (240,160) fires
  an enemy projectile at about presentation 1570. It hits the player at about
  1614: red6→5, countdown60, last source type9 flags0. The countdown expires at
  about 1673. The blast at 1676 then deals 2 (red3, countdown119, source
  bomb/flags4). The destroyed rocks are all type2; no type-5 callback runs. In
  the port, the two worms teleport to (320,400)/(520,400) instead of
  (480,160)/(240,160), so its single shot misses.
* `paired-random-run` CQHR 8NEH. A Wall Hugger (218) sliding along the top wall
  touches the player at about presentation 1568: red6→5, countdown60, source
  218. The blast at about 1589 pushes the player (vx +12.59) but is rejected
  by immunity, so HP stays 5. Continued Hugger contact explains the final
  leftward motion. The port spawns no NPCs in this room, so the bomb's 2-HP hit
  lands.

Both are now implemented (issue #206) and exercised by `rock-blast`:

* **CheckLine.** Every circle target (player, NPCs, pickups) passes
  `Grid::blast_line_clear(blast, target, ignoreWalls=true)` before knockback or
  damage. Authored Block selectors 1900 count as native type3, which also stops
  the grid-candidate rays and the candidate line. In the contrived `block-line`
  route (seed `286TWPAC`, Room283 above the start room) the bomb sits in the
  bottom notch beside Block(5,4) and Isaac waits in the notch above it, 77.2
  native units away. The original keeps red6, countdown0 and velocity unchanged
  at the presentation301 blast and destroys no Rock. `random-block-line` repeats
  the route on a fresh OS-random seed with the same room above the start.
* **Bomb rocks.** A blasted type5 rock enters state3 with countdown3 (Init
  stores `state==1 ? 3 : -1` at `0x56efc0`). Rock Update `0x56bf00` runs
  before marker aging every full update, post-decrements, and on the fourth
  update calls Destroy, which sets state2, collision0 and marker900 and calls
  `0x554490` with damage40 (`0x665460`), radius60 (`0x668968`), line check on,
  flags4 and no source. In `regression-rock-phase-0/1` (seed `26T688VM`) the
  original enters state3 at full update140 and explodes at144; the port now
  matches every presentation of both cases. The secondary blast's 1000/1 and
  particle spawns, and their two shared-stream draws (`0x42e8c0`), remain
  unadmitted like ordinary rock debris.
