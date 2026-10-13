# Basement secret-door revelation and discovery

A real player bomb now reaches each admitted hidden door through the same
ascending grid-candidate snapshot as ordinary Rock destruction. The callback
opens the authored secret connection, stores its reciprocal reveal and updates
descriptor visibility. Admission and return restore the revealed entrance;
other secret exits remain hidden until their own public revelation.

The implementation uses authored STB door positions and generated descriptor
neighbors. Native actor/grid snapshots never enter production. Hidden doors
are actual collision objects: kind16, variant6, state1, class4. Revelation
selects variant7/state2 and retains class4 for that blast phase; the next full
grid update selects class5. A saved blown-open secret entrance retains class5
at admission while ordinary opened doors begin with class4. The hole-in-wall
asset applies on both sides; authored Hidden/Opened clips are extracted from
the pinned resources. Full door animation/FX/pixels remain separate.

## Source policy and persistence

Addresses refer to Linux v1.05 plaintext SHA-256
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.

- Door.TryBlowOpen564870 calls Level.UncoverHiddenDoor5875f0 for a hidden
  variant0/6. The reciprocal saved grid record becomes kind16/variant7 when
  missing or hidden. The eligible current door becomes variant7/state2 and
  records its blown-open byte+611.
- Door.CanBlowOpen5644b0 uses the generated descriptor byte+40 and clear+3d.
  Byte+40 marks surprise-miniboss substitution; ordinary CanShutDoors actors
  never produce it. Constructors587850/55cf30 clear it. Secret/shop writers
  58a10e/58a9c7 require stage>4/>3; the reserved Devil-room selector58669b has
  its own Krampus guards. Fresh Basement's generated ordinary descriptors
  retain false, independently of NPC countability. Later-stage substitution
  and failed-open saved variants remain separate.
- Level.UpdateVisibility586940 ORs5 into the current descriptor flags. A secret
  endpoint requires the door's actual blown-open flag for fresh Isaac; the
  revealed neighbor receives flags3. Entering/returning retains those flags
  and the visited metadata. Items, other curse/UI policies and complete
  rendered minimap pixels remain separate.

## Public live acceptance

After a fresh build and the production floor diagnostic, run the original and
actual-DOM browser gate:

```sh
source cloud/env.sh
npm run build
cargo build --manifest-path engine/Cargo.toml --locked --bin floor
bash tools/original/with-display.sh node tools/e2e/secrets.mjs --count 1
```

The gate captures a contrived opposite-phase JF2DSF92 secret route plus a newly
recorded OS-random no-hostile 13×7 secret configuration. Isaac plants his natural
starting bomb, retreats through public movement, enters the revealed room and
returns. Each original uses a fresh isolated profile and passes complete
binary/log/frame/input/clock provenance. Every browser interval executes even
after comparison failures against an immutable production artifact.

Assertions compare complete hidden/open door objects and endpoint assets,
actual blown-open flags, generated surprise policy, every descriptor display
flag, complete minimap cell routing, room/template/spawn identity, exact player
motion/HP/immunity, independent nonempty authored groups and returned admission.
Keeper17 selectors are metadata only; regular pickup admissions join the native
multiset in both directions. Actual key collection belongs to inventory.

The retained JF2DSF92 source route has two natural regular keys at (280,280) and
(360,280), spawn seeds 3966468430/2341774344. Its source and browser match all 269
revelation/entry/return phases. A contrived curse return also matches 120/120.
The only required mismatch is missing regular-key admission; it remains red
until that owned inventory callback lands. The fresh opposite-phase JF2DSF92
and OS-random WS03MPHH original pair executes 538 phases with the same two
required key-admission failures. Source/browser values and earlier corrected
map-refresh failures remain under ignored `.cache/e2e/secrets/`.

The secret branch depends only on ordinary Rock blast dispatch. It does not
include the broad unpublished NPC prototype. The existing W0L0LYYM fire
admission and T1PK6Y0W hostile return failures remain required in their gates.
