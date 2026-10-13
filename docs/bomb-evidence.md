# Regular bomb evidence and implemented scope

The port implements fresh normal-mode Isaac's planted bomb: E placement,
one inventory debit, a paused fuse, authored Pulse/Explosion playback, one
full-heart blast hit, strict radial contact, native knockback, and a damage
countdown. Enemy/grid damage, moving/pushed bombs, variants and crater decals
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
  presentation120. Native placement/update/render phase affects this boundary;
  the current runtime preserves the calibrated starting-room phase. Authored
  animation frames last two presentations. A general native phase adapter
  remains unimplemented; these counts are presentation observations.
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
damage attempt and also applies while damage is blocked by immunity. The bomb
step runs **before** player movement: the same presentation integrates the
impulse into position and then applies ordinary motion drag. No extra
bomb-specific decay is added.

An independent controlled original seed `3BMB MFTE` supplies the complete
D27/release30/E tape in
`.cache/coordination/stage-masks/explosion1/targets/mask-report.json`. At
placement, the player is at native x438.4042053222656 with vx0.09337755292654037.
Coasting settles to x439.18450927734375 before the blast. The port reproduces
all three following native position/velocity observations through ordinary
commands; the source fixture guards against applying blast impulse after
position integration:

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

Focused Rust cases cover fuse/damage timing, held E and empty inventory, pause,
native near/far distances, strict equality and radial contacts, knockback during
immunity and the complete coasting-blast trajectory, HP saturation and atomic
ID exhaustion, new-run reset, and identical
single-step/batched replay. Browser coverage exercises public input and HUD.
Original images and memory probes supply evidence; self-equality does not prove
full original state or pixel parity.

The independent Fly-room probe at
`.cache/coordination/combat/fly-bomb/manifest.json` records player HP6→4 at
presentation120 after E. Two nearby Fly13 actors retain HP3 while pendingDamage
is60 at presentations120/121; at122 their base update drains the damage queue
to HP−57 and marks them removed, and by124 they leave the active list. Farther
flies remain healthy. This confirms a native update/render phase distinction
and queued enemy damage; it does not extend the current port's enemy support.
The existing starting-room121 blast references remain unchanged. General phase
alignment across placement, room transitions and pause requires further source
proof before runtime changes.

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
