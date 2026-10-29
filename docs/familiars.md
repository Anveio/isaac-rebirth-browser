# Familiars: design and tracker

This is the design for porting every base-Rebirth familiar (entity type 3). It
comes before any implementation code. The owner's rule is that familiars are
**one durable system ported from the original's familiar entity**, not
per-item code. Each familiar is then a table row plus the variant AI the
original's Update dispatches to. Cube of Meat and Ball of Bandages level-ups,
Dry Baby's Necronomicon proc and Sacrificial Dagger's orbital contact damage
should all follow from shared primitives.

Progress is tracked in the [tracker](#tracker) below and per item in
[items.md](items.md) (through `tools/foundation/item-status.json`). The item
framework that feeds the familiar cache flag is described in
[item synergies](item-synergies.md).

All addresses are in the pinned Linux `isaac.x64` (see
`tools/e2e/lib/original.mjs`). Entity field offsets are relative to the entity
object; `Familiar+F58` means byte offset `0xF58`.

## How the original builds familiars

### Count reconciliation: 513E80 and CheckFamiliar 508D10

The `EvaluateItems` (`511F10`) block for cache flag `0x200` starts at
`513E80`. It is straight-line code with one `CheckFamiliar(player, variant,
count, rng)` call (`508D10`) per familiar variant, in a fixed order (see the
[roster](#roster) table). For an ordinary row, count is the owned count
`Player+0xE1C+4*id` plus the temporary count `5E3ED0(id)`. The RNG argument
is the item's own RNG at `Player+0x1424+16*id`. Some rows compute the count
differently:

| Rows | Count expression | Address |
| --- | --- | --- |
| Isaac's Heart 62 | `min(owned[276], 1)`, with no temporary count | `51461E` |
| Halo of Flies 33 | `min(owned[10], 10)` | `5146C6` |
| Dead Cat 40 | `(owned[81] + 8) / 9` | `5146EB` |
| Dead Bird 14, Best Bud 60 | presence of a temporary record (`5E3D70` 117, 274) | `514718`, `51473C` |
| Cube of Meat 44..47 | `n = owned[73] + temp[73]`. One of L1/L2/L3 gets count 1, chosen by `n % 4` (1, 2, 3; 0 gives none), and L4 47 gets `n / 4`. Achievement `0x90` when `n / 4 > 0`. | `514760..51481E`, `514B30..514B86`, `5154D8` |
| Ball of Bandages 69..72 | Same law over `owned[207] + temp[207]`, achievement `0x13`. | `51481E..5148E3`, `5154F8` |
| Isaac's Body 48 | temporary count of 107 (The Pinking Shears) | `5148FB` |
| 78 | temporary count of 325 (Scissors). Base Rebirth has no `3.78` entity row. | `51491E` |
| Key Pieces 26/27, Full Key 28 | With 238 and 239 both owned: 26 = 0, 27 = 0, 28 = 1, achievement `0x3A`. Otherwise 28 = 0 and 26/27 count normally. | `514941`, `515468` |
| Isaac's Head 12, ???'s Soul 13 | presence of trinket 54 or 57 (`HasTrinket 508FE0`) | `5149C4`, `5149E8` |
| Eve's Bird Foot 15 | presence of temporary trinket record 60 (`5E3DC0`) | `514A0C` |

Every level-up and "count" law is in the arguments, so `508D10` stays generic:

1. Collect every live type-3 entity of the variant (`455680(3, variant)`).
2. The first `count` owned by this Player are kept.
   Halo of Flies 33 ignores the owner test.
3. Each one past the count is removed through `vtable+0x48`, which is Remove
   `466AD0`. Remove unregisters the orbit layer (`466780(layer, false)`) and
   restores a few Player hooks (43, 73, 68).
4. For each missing one, `5C2D70` draws a seed from the item RNG. Then
   `42E8D0` draws a shared direction, scaled to 20 units and added to the
   Player position. `5C7E50` clamps the result into the room. `Spawn 550A40(3,
   variant, 0, pos, zero, player, seed)` creates the familiar, and its Update
   (`vtable+0x18`) runs **once immediately**.

So a second Cube of Meat removes the L1 entity and spawns an L2 entity. Box of
Friends-style counts (any count source above 1) are the same loop.

### The Familiar class

The vtable lives at `0x66CB70`. Its familiar-specific slots are:

| Slot | Function | Role |
| --- | --- | --- |
| +0x10 | `46EE90` | Init |
| +0x18 | `4704A0` | Update |
| +0x20 | `46F850` | Render |
| +0x28 | `466540` | Render pass (Guillotine 68, 78 special) |
| +0x38 | `466450` | TakeDamage. Isaac's Heart 62 forwards it to the Player. Punching Bag 67 takes knockback. |
| +0x48 | `466AD0` | Remove |
| +0x80 | `46FA50` | Collision callback (familiar vs other) |

**Init `46EE90`** zeroes the AI words F58 (fire cooldown), F5C (pose hold),
F6C (orbit phase) and F70 (orbit radius vector). It also sets:

- F78 orbit speed 0, F7C orbit layer -1;
- F60..F68 = -1, F88 = F8C = 0;
- the appearance counter FC8 = 5;
- the Player link `+0x7D8`.

Then:

- **Grid collision class `+0x340`.** It is 5 for 46, 47, 48, 56, 71, 72 and
  73, 6 for 75 and 0 for every other variant (`46EF36..46F1A0`). Peeper 17
  also skips the `+0x318 |= 4` flag.
- **Follower train.** If `4666B0(variant)` is true (1-7, 9-12, 20-23, 25-28,
  40, 41, 51, 52, 54, 55, 57, 59, 61, 74), `46ED90` links the new familiar
  behind the current tail. It sets `+0x498` (the one ahead) on the new
  familiar and `+0x4A0` (the one behind) on the tail.
- **Orbit layer.** Variants 30..70 jump through `0x66C470` and set F7C:
  - layer 0: 32, 33, 35, 44, 45, 50, 69, 70;
  - layer 1: 43, 65, 68;
  - layer 2: 31;
  - layer 3: 30, 60.

  Then `466780(layer, true)` respaces that layer, and F70 takes the layer
  radius from the global table `0x8ECB00` (built at `40B040`): layer 0
  (25, 20), layer 1 (35, 30), layer 2 (60, 45), layer 3 (110, 90).
  Sacrificial Dagger 35 multiplies it by 1.5 (`46F308`).
- **Per-variant tails.**
  - Blue fly 43 and Blue spider 73: collision damage = 2 × the Player's
    `50D090` tear damage.
  - Guppy's Hairball 42: state `+0x18` = 1.
  - BBF 58: collision damage 10.
  - Familiar flag `0x200000`.
  - Isaac's Heart 62 registers on the Player (`519130`).
  - Samson's Chains 75: `+0x344` = 4.

**Orbit respacing `466780(layer, includeSelf)`** collects every live familiar
of the same Player on that layer, appending itself when asked. The i-th of n
gets:

- phase `F6C = i·2π/n`;
- speed `F78 = k · speed[layer]`, where `speed` is `0x66CAC0` with Guardian
  Angel 112 held (0.06, 0.045, -0.03, 0.02) and `0x66CAD0` otherwise
  (0.03, 0.03, -0.02, 0.013);
- `k` = 1/3 with item 279 held, else 1;
- Best Bud 60 multiplies its speed by 3.

**Update `4704A0`** runs in this order:

1. `447190`, the shared Entity update: sprite, velocity, friction.
2. The appearance countdown FC8: it goes 5 to 0, and the Poof is created at
   3. This is the existing Leech port.
3. A first-frame pass over layer-0 orbitals (`470550`).
4. **The variant switch** at `47075A`, through jump table `0x66C818`
   (variants 0..79; see [mechanism groups](#mechanism-groups)).
5. **The common tail** at `47064C`:
   - `444890(interval)` clears the contact hit-set F98 every N frames. N is 6
     for 35; 14 for 14, 15, 16, 17, 42, 43, 46, 47, 56, 63, 75 and 79; and 2
     for all others.
   - BFFS! (247) scales the sprite ×1.25, except for 43, 66, 73 and 79.
   - A pending-remove flag `+0x31F` calls Remove.

### Shared behaviour primitives

| Primitive | Function | What it does |
| --- | --- | --- |
| Follow | `466B50(fast)` | Walks `+0x498` forward to the first familiar to follow. It skips 3 and 59 while their F58 runs, and 25 in state 1. With none, it follows the Player. Past 20 units the position moves `(d - 20)/d · 0.1` of the gap (×2 when `fast`). Friction (`+0x470`) is then multiplied by 0.8. |
| Orbit | `466DB0(this, center)` | The angle is `fmod(-F78 · (gameFrame + parity·0.5) · π · 0.5 + F6C, 2π)`, done in double with x87 `fprem`, plus the layer offset `0x66CAA0` (1.5, 0.5, 2.0, 1.2). It returns `center + (cos, sin) · F70`. The orbital case writes this to the position and zeroes the velocity (`470CC0`). Dagger 35 also writes its sprite rotation as `angle·180/π`. |
| Shooter | `4702E0` | Counts F58 down. Reads the Player fire direction `Player+DA4`; Abel 8 mirrors it, and Mongo Baby 74 mimics a variant from `+0x18`. Off: F5C counts down to the idle pose `467BA0(3)`. On with F58 ≤ 0: fire, then F5C = 18 and F58 = 30 (15 for 74). Shooting poses are `4683B0` (FloatShoot*) while F5C > 8, else `467BA0` (Float*). Left flips X (`+0x2FC`). |
| Fire | `467170(dir)` | The ordinary tear: `Spawn(2, …)` at the familiar, velocity `dir·10 + playerVel·0.6·2`, renormalized to 10 below length 100. Per-variant damage, flags and scale are inside. Robo-Baby 6 (and 74 mimicking 4) fires the laser `467010`. Rotten Baby 54 (and 74 mimicking 9) takes the blue-fly path `46E6B0`/`4688D0`. |
| Chase | `469A00` | Targets the nearest enemy, else the Player deadband, and moves toward it (Leech 56, Cube L3/L4 46/47, Bandage L3/L4 71/72). |
| Bag spawner | `470CEE` | Follows, counts rooms cleared and plays Spawn to drop a pickup (Bomb Bag 20, Sack of Pennies 21, Little C.H.A.D. 22, The Relic 23, Mystery Sack 57). |
| Bum | `46B060` | Collects pickups and pays out (Bum Friend 24, Dark Bum 64). |

### Collision callback 46FA50

The manager's contact pass (`454940..454B3C`) puts familiars in partition
`+0`, tears and knives in `+8`, NPCs and bombs in `+0x10`, and projectiles in
`+0x18`. It pairs them in this order:

1. familiars × projectiles;
2. familiars × NPCs;
3. familiars × Players;
4. then tears × NPCs and the rest.

So every familiar contact lands before any tear hit in the same frame. The
callback `46FA50(this, other, low)` does the following:

- **Contact damage.** The multiplier is 2 with BFFS! (247), or 248 for 43/73,
  and 1 otherwise. If `other` is a vulnerable enemy (`444810`), and either
  collision damage `+0x48C` > 0 or the variant is 33, and `other` is not yet
  in the hit-set F98: it inserts `other`. Then it calls
  `other->TakeDamage(mult·+0x48C, 0, EntityRef(this), 30)` (`vtable+0x38`).
  Per-variant tails follow:
  - Guppy's Hairball 42 grows;
  - Lil' Haunt 63 has a 1-in-10 fear chance;
  - 58/59 (BBF/Bob's Brain) explode once over 20 room frames for
    `60 + 25·(mult-1)` (+50 for some characters);
  - blue fly 43 and spider 73 spawn an effect and remove themselves.

  The hit-set plus the interval clear in the Update tail is why Sacrificial
  Dagger (15 damage) hits every 6 frames.
- **Projectile blocking.** If `other` is a projectile (type 9) and the variant
  is a blocker (44, 35, 45, 33, 65, 32, 42, 17, 51, 75, 50 with state 0, 68,
  67), it sets the projectile's remove flag `+0x31F`. **Dry Baby 51** first
  rolls `Random(4) == 0` (`42E8A0`, shared stream) with F58 ≤ 0. On success
  it plays `Hit` and sets F58 = 150 (`470198`).
- **The Dry Baby proc.** Case `471401` in the Update checks whether `Hit` has
  finished. If so, it plays `Idle`, calls `UseActiveItem(35, false, true,
  false)` (`522250`) on the Player and records `5805C0`. It then follows,
  and F58 counts down. The Necronomicon itself is already ported
  (`active_items.rs`, Leaf::Necronomicon).

### Mechanism groups

The groups are taken straight from the Update jump table `0x66C818`. Where the
original shares a case, the port shares a primitive.

| Group | Case | Variants | Update body |
| --- | --- | --- | --- |
| `none` | `47064C` | 0, 18, 19, 29, 34, 36..39, 49 | none (unused variants) |
| `follower` | `4708A9` | 26, 27, 40, 41 | Follow |
| `shooter` | `4708E6` | 1, 4, 5, 6, 7, 9, 10, 11, 12, 54 | Follow, Shooter |
| `shooter-mirror` | `470910` | 8 Abel | mirror of the Player about the room, Shooter |
| `shooter-mimic` | `4708BB` | 74 Mongo Baby | Follow, Shooter mimicking another baby |
| `shooter-free` | `470E5B` | 13, 76 | own motion, Shooter |
| `orbital` | `471759` | 30, 31, 32, 33, 35, 44, 60, 69 | Orbit at the Player |
| `orbital-shooter` | `471178` | 45, 70 | Orbit, Shooter |
| `chaser` | `471168` | 46, 47, 56 | Chase |
| `chaser-shooter` | `470ADF` | 71, 72 | Chase, Shooter |
| `spawner` | `470CEE` | 20, 21, 22, 23, 57 | Follow, bag spawn |
| `bum` | `470CDE` | 24, 64 | bum |
| `unique` | one each | 2 `46E760`, 3/59 `46A050`, 14/15/63 `46BE30`, 16 `46C2F0`, 17 `470DD0`, 25 `470B97`, 28 `46DBC0`, 42 `4714FF`, 43 `468AD0`, 48 `469700`, 50 `46C8B0`, 51 `471401`, 52 `4712F7`, 53 `46A780`, 55 `4711E7`, 58 `47103A`, 61 `46EA60`, 62 `470F9D`, 65 `470F4F`, 66 `46CA90`, 67 `46D280`, 68 `470AF7`, 73 `468CB0`, 75 `46F4A0`, 77 `46ADD0`, 78 `470A30`, 79 `46D790` | each its own Update, built from the primitives |

Cube of Meat and Ball of Bandages move through `orbital` (L1), then
`orbital-shooter` (L2), then `chaser` / `chaser-shooter` (L3/L4) purely by
which variant the count law spawns.

## Port design

### Modules

- **`familiars.rs`: the entity.** `Familiar` keeps the original's fields,
  each named for its offset:
  - F58 `cooldown`, F5C `pose_hold`;
  - F6C `orbit_phase`, F70 `orbit_radius`, F78 `orbit_speed`, F7C
    `orbit_layer`;
  - FC8 `appear`, F98 `hits`;
  - +0x18 `state`, +0x498/+0x4A0 `follow_parent`/`follow_child`;
  - collision damage +0x48C, grid class +0x340.

  It also holds Init `46EE90`, the Update shell `4704A0` (base update,
  appearance, switch, common tail) and Remove `466AD0`. The existing Leech
  appearance and Poof code stays and becomes the shared appearance step.
- **`familiar_roster.rs`: the 513E80 table.** `ROSTER: &[RosterRow]` follows
  the binary's call order. A row is `{ variant, count: CountSource, rng_item,
  address }`. `CountSource` is a small closed enum with one entry per count
  law above: `Owned(id)`, `OwnedMin(id, n)`, `DeadCat`, `TempPresent(id)`,
  `Levels { item, level, achievement }`, `KeyPieces`, `Trinket(id)`,
  `TempTrinket(id)`, `TempCount(id)`. The block walks every row and calls one
  `check_familiar` (`508D10`). This replaces today's `SUPPORTED` list.
  Spawning a variant whose AI is not ported still admits the entity, so it is
  visible and follows its group defaults. It records a frontier on the
  familiar (`unported_ai`), never on the cache: every familiar request clears
  `0x200`, so picking up any familiar keeps Isaac playable (item-synergies
  decision 2).
- **`familiar_variants.rs`: per-variant rows.** `VARIANTS: &[VariantRow]`,
  keyed by variant. A row holds:
  - `update: UpdateCase`, an enum with one entry per jump-table case in the
    table above;
  - `follower` (`4666B0`);
  - `orbit_layer` (`0x66C470`);
  - `grid_class`;
  - `hit_interval` (`444890` argument);
  - `blocks_projectiles`;
  - `bffs_scale`;
  - `fire: FireKind`, holding the `467170` damage, flags and scale per
    variant, or the laser/fly alternatives;
  - the evidence address.

  Entity numbers (radius, mass, collision damage, shadow, friction) come from
  `entities2.xml` at build time, as today.
- **`familiar_ai.rs`: the primitives.** Follow `466B50`, orbit position
  `466DB0`, respace `466780`, shooter `4702E0`, fire `467170`, chase
  `469A00`, bag spawn, bum. Variant cases in the `unique` group are functions
  here, each named for its address. They call only primitives and state.
- **`familiar_contact.rs`: the collision callback `46FA50`.** It runs in the
  manager contact pass at the original's position: familiars × projectiles,
  then familiars × NPCs, then familiars × Players, before the existing tear ×
  NPC loop in `lib.rs`. Candidates come from `contact_candidates` (partition
  `+0`) and current circle overlap. Damage goes through the same NPC
  TakeDamage path tears use, with the source ref and countdown 30.

### Rendering and data

`engine/build.rs` builds a type-3 `Definition` for every variant in
`entities2.xml` from the generated familiar actor catalogue
(`web/generated/item-assets/familiar-data.json`, produced by `npm run
assets:items`). This replaces the hand-written `(3, 56)` row. The renderer
draws each familiar by the actor path its definition names, and
`web/animation.ts` bundles `gfx/familiar/*.png`. No per-variant renderer
branch is allowed.

### Shared randomness

Every draw stays on the original's streams:

- spawn seed from the item RNG `5C2D70`;
- placement direction `42E8D0`;
- Tear Init and fire draws on the shared MT;
- Dry Baby's `Random(4)`.

A familiar case that draws in the original must draw in the same order in the
port. A missing draw is a parity bug, because later NPC rolls shift (as in
set 8+46+101, where the shared MT was 16 draws behind). The per-case trace
records the shared MT index every frame, so a divergence shows up at the
first frame it happens.

### Float rules

Vector math uses the `motion::WorldVector` f32 operations in the original's
order. Orbit angles follow `466DB0` exactly: f32 product, widened to f64,
`·π·0.5 + phase`, x87 `fprem` remainder, back to f32. Sine and cosine go
through `fmath.rs`, never std.

## Parity test plan

Acceptance is original-vs-port live cases only. There are no unit tests.

1. **Per-familiar item-set case.** For each familiar item `id`, run `npm run
   sandbox:case -- --items <id> --enemy 10.0`, which uses the existing
   `item-set` interaction. Both games get the same Challenge grant. The
   graded rows add a `familiars.trace` field per label:
   - for each type-3 entity in manager order: variant, position and velocity
     (f32), sprite clip and frame, F58/F5C where observable, and state;
   - the shared MT index.

   Positions are compared bit-exact (f32), not with the pickup tolerance.
   Clip and frame must be equal.
2. **Frame-by-frame familiar case.** The `familiar-frames` interaction
   samples every full update over a fixed tape:
   - 30 idle frames, the appearance and Poof;
   - walk in a square, which exercises follow and orbit;
   - shoot in each direction for 40 frames each;
   - stand inside the Gaper's path, which exercises contact and blocking.

   It compares the trace above at each frame and reports the first divergent
   frame per field. Native rows come from the existing pickup-interaction
   observer, extended to read the Familiar fields named above.
3. **Paired cases per familiar.** Every tracker row names its case: the item
   alone plus a Gaper (10.0). Batches that touch counts add multiples:
   - `--items 73,73`, `73,73,73` and `73,73,73,73` for Cube levels;
   - `207 ×1..4` for Bandages;
   - `8,8` for two Bobbies in the follower train;
   - `172,73` for two layer-0 orbitals respacing.

   Dry Baby needs projectiles, so its case uses a projectile-firing enemy
   (Pooter 14.0, Clotty 15.0) instead of a Gaper.
4. **Regression gate per batch.** Before each self-merge:
   - run `npm run format` and `npm run check:build`;
   - run the touched suites (`familiars`, `pickup-interactions`, plus any
     suite whose driver changed) on the branch and on `origin/main` with the
     same seeds, and compare per case. No case that passes on main may fail
     on the branch.
5. **Footage.** `tools/playtest/side-by-side.mjs` renders the item-set case for
   review. It is never a pass/fail input.

A failing field stays failing. A tracker row is **complete** only when its
paired cases pass field by field.

## Order of work

1. This design, merged alone.
2. Core, in one batch with Brother Bobby (branch `claude/familiars-core`,
   with the `claude/brother-bobby` work folded in; merged with batch 1):
   - roster table and `check_familiar` for every row;
   - Init, follower train, Update shell and common tail;
   - the variant table;
   - Follow, Shooter and Fire primitives;
   - build-time definitions and renderer for all variants;
   - the trace rows in the item-set grader.
3. Batch 1:
   - Sister Maggy 7 (shooter);
   - Sacrificial Dagger 35 and Cube of Meat 44..47 (orbital, orbit respacing,
     the contact callback, the level law);
   - Dry Baby 51 (projectile blocking and the Necronomicon proc).
4. Next batches by group:
   - the remaining shooters (Demon Baby, Little Gish, Little Steven,
     Robo-Baby, Ghost, Harlequin, Rainbow, Rotten, Abel, Mongo);
   - orbitals (Forever Alone, Distant Admiration, Guardian Angel, Halo of
     Flies, Best Bud) and Ball of Bandages;
   - chasers (Leech contact);
   - followers (Key pieces, 1up, Dead Cat);
   - spawners and bums;
   - then the `unique` cases in pool-weight order.

## Tracker

Status: **missing** means not ported, or the entity is admitted with group
defaults only. **partial** means its case is ported but a paired case fails
or is unrun. **complete** means every paired case passes field by field.

### Core system

| Part | Original | Status | Live case | Notes |
| --- | --- | --- | --- | --- |
| Roster table, count laws | `513E80` | partial | pickup-interactions `set-*` | Every call in order (`familiar_roster.rs`), the key-piece and level laws included. Temporary trinket records (Eve's Bird Foot) are not modeled; Cube and Bandage achievements are not read. |
| CheckFamiliar reconcile | `508D10` | partial | pickup-interactions `set-73-73-*` | Removal past the count, seed, shared direction, clamp and the explicit first Update for every variant; checked for single familiars and the Cube level swap. |
| Init | `46EE90` | partial | pickup-interactions `set-*` | Follower link, orbit layer, respacing and radius (Dagger x1.5), grid class. |
| Follower train | `46ED90`, `4666B0` | partial | `set-8-46-101`, `set-67` | Links and unlinks; trains of two or more are not yet compared. |
| Update shell, appearance | `4704A0` | partial | `set-*` | Base update, appearance counter and Poof, the switch by `UpdateCase`; unported cases count `unresolvedAiUpdates`. |
| FastUpdate | `46F850` | partial | `trace-bobby-random-0` | Base half step, Follow(fast), the half-frame orbit point. Robo-Baby 2.0 and Samson's Chains bodies are not ported. |
| Common tail (hit-set clear, BFFS scale) | `47064C` | partial | `set-172-*` | Hit-set interval and BFFS scale; Remove on +31F is not ported. |
| Follow | `466B50` | complete | `trace-bobby-random-0` | Full and fast forms, frame by frame. |
| Orbit, respace | `466DB0`, `466780` | partial | `set-172-73-*`, `set-73-*` | Two layers checked (Dagger and Cube L1 share layer 0). Big Fan and Guardian Angel speeds are unchecked. |
| Shooter, fire | `4702E0`, `467170` | partial | `set-8-46-101`, `set-67`, `set-73-73` | The ordinary tear rows (Bobby, Maggy, Cube L2). Robo-Baby's laser, Rotten Baby's flies, Rainbow's effect roll and Bandages' charm roll are not ported. |
| Chase | `469A00`, `4688D0` | partial | `trace-73x3-fixed` | Target search, FindGridPath walk and Cube L3/L4 blood splats (Cube L3 frame by frame). The query partition is rebuilt at the call, not at the frame start. |
| Collision callback | `46FA50` | partial | `set-172-*`, `necro-265-*` | Enemy contact damage with the hit-set, projectile blocking, Dry Baby's roll. Familiar-specific cases (Punching Bag, Holy Water, Guillotine...) are not ported. |
| Remove | `466AD0` | partial | `set-73-73-*` | Respace and unlink on a count drop. |
| Definitions and rendering for all variants | build.rs, renderer | complete |  | Every catalogue row and its actor. |
| Trace rows in the item-set grader | pickup-interactions | complete |  | `familiars.position` (presented position, player tolerance) and `familiars.state` (clip, F58, F5C, orbit layer) per label. |

### Roster

Order is the `513E80` call order. *Group* is the Update case group.

| Item | Name | Variant | Group | Update | Count | Status | Live case |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 8 | Brother Bobby | 1 | shooter | `4708E6` | owned+temp | complete | `set-8-46-101-e10.0-*`, `trace-bobby-random-0` |
| 113 | Demon Baby | 2 | unique | `46E760` | owned+temp | missing |  |
| 88 | Little Chubby | 3 | unique | `46A050` | owned+temp | missing |  |
| 99 | Little Gish | 4 | shooter | `4708E6` | owned+temp | missing |  |
| 100 | Little Steven | 5 | shooter | `4708E6` | owned+temp | missing |  |
| 95 | Robo-Baby | 6 | shooter | `4708E6` | owned+temp | missing |  |
| 67 | Sister Maggy | 7 | shooter | `4708E6` | owned+temp | complete | `set-67-e10.0-*` |
| 188 | Abel | 8 | shooter-mirror | `470910` | owned+temp | missing |  |
| 163 | Ghost Baby | 9 | shooter | `4708E6` | owned+temp | missing |  |
| 167 | Harlequin Baby | 10 | shooter | `4708E6` | owned+temp | missing |  |
| 174 | Rainbow Baby | 11 | shooter | `4708E6` | owned+temp | missing |  |
| 131 | Bomb Bag | 20 | spawner | `470CEE` | owned+temp | missing |  |
| 94 | Sack of Pennies | 21 | spawner | `470CEE` | owned+temp | missing |  |
| 96 | Little C.H.A.D. | 22 | spawner | `470CEE` | owned+temp | missing |  |
| 98 | The Relic | 23 | spawner | `470CEE` | owned+temp | missing |  |
| 144 | Bum Friend | 24 | bum | `46B060` | owned+temp | missing |  |
| 128 | Forever alone | 30 | orbital | `471759` | owned+temp | missing |  |
| 57 | Distant Admiration | 31 | orbital | `471759` | owned+temp | missing |  |
| 112 | Guardian Angel | 32 | orbital | `471759` | owned+temp | missing |  |
| 155 | The Peeper | 17 | unique | `470DD0` | owned+temp | missing |  |
| 170 | Daddy Longlegs | 16 | unique | `46C2F0` | owned+temp | missing |  |
| 178 | Holy Water | 25 | unique | `470B97` | owned+temp | missing |  |
| 11 | 1up! | 41 | follower | `4708A9` | owned+temp | missing |  |
| 187 | Guppy's Hairball | 42 | unique | `4714FF` | owned+temp | missing |  |
| 172 | Sacrificial Dagger | 35 | orbital | `471759` | owned+temp | complete | `set-172-e10.0-*`, `set-172-73-e10.0-*` |
| 264 | Smart Fly | 50 | unique | `46C8B0` | owned+temp | missing |  |
| 267 | Robo-Baby 2.0 | 53 | unique | `46A780` | owned+temp | missing |  |
| 206 | Guillotine | 68 | unique | `470AF7` | owned+temp | missing |  |
| 265 | Dry Baby | 51 | unique | `471401` | owned+temp | complete | `set-265-e14.0-*`, `necro-265-e14.0-fixed` |
| 266 | Juicy Sack | 52 | unique | `4712F7` | owned+temp | missing |  |
| 268 | Rotten Baby | 54 | shooter | `4708E6` | owned+temp | missing |  |
| 321 | Samson's Chains | 75 | unique | `46F4A0` | owned+temp | missing |  |
| 271 | Mystery Sack | 57 | spawner | `470CEE` | owned+temp | missing |  |
| 278 | Dark Bum | 64 | bum | `46B060` | owned+temp | missing |  |
| 270 | Leech | 56 | chaser | `469A00` | owned+temp | partial | familiars (the full chase is shared with Cube L3; no Leech case against an enemy yet) |
| 319 | Cain's Other Eye | 76 | shooter-free | `470E5B` | owned+temp | missing |  |
| 320 | ???'s Only Friend | 77 | unique | `46ADD0` | owned+temp | missing |  |
| 279 | Big Fan | 65 | unique | `470F4F` | owned+temp | missing |  |
| 269 | Headless Baby | 55 | unique | `4711E7` | owned+temp | missing |  |
| 272 | BBF | 58 | unique | `47103A` | owned+temp | missing |  |
| 273 | Bob's Brain | 59 | unique | `46A050` | owned+temp | missing |  |
| 280 | Sissy Longlegs | 66 | unique | `46CA90` | owned+temp | missing |  |
| 281 | Punching Bag | 67 | unique | `46D280` | owned+temp | missing |  |
| 322 | Mongo Baby | 74 | shooter-mimic | `4708BB` | owned+temp | missing |  |
| 276 | Isaac's Heart | 62 | unique | `470F9D` | min(owned, 1) | missing |  |
| 318 | Gemini | 79 | unique | `46D790` | owned+temp | missing |  |
| 277 | Lil' Haunt | 63 | unique | `46BE30` | owned+temp | missing |  |
| 275 | Lil' Brimstone | 61 | unique | `46EA60` | owned+temp | missing |  |
| 10 | Halo of Flies | 33 | orbital | `471759` | min(owned, 10) | missing |  |
| 81 | Dead Cat | 40 | follower | `4708A9` | (owned+8)/9 | missing |  |
| 117 (passive) | Dead Bird | 14 | unique | `46BE30` | temp present | missing |  |
| 274 | Best Bud | 60 | orbital | `471759` | temp present | missing |  |
| 73 | Cube of Meat L1..L4 | 44..47 | orbital, orbital-shooter, chaser | `471759`, `471178`, `471168` | level law | partial | `set-73-e10.0-*`, `set-73-73-e10.0-*`, `set-73-73-73-e10.0-*`, `set-73-73-73-73-e10.0-*`, `trace-73x3-fixed`, `trace-73x3-kill`. One case fails after the Gaper dies: the room-clear trinket (pickup 350.44) is missing in `set-73-73-73-73-e10.0-random-0`, where the shared MT diverges as it does in sets without familiars (1+4+12). The Cube itself matches (L3 frame by frame) |
| 207 | Ball of Bandages L1..L4 | 69..72 | orbital, orbital-shooter, chaser-shooter | `471759`, `471178`, `470ADF` | level law | missing |  |
| 107 (active) | Isaac's Body | 48 | unique | `469700` | temp count | missing |  |
| 325 (active) | (no Rebirth row) | 78 | unique | `470A30` | temp count | missing |  |
| 238, 239 | Key Piece 1, 2, Full Key | 26, 27, 28 | follower, unique | `4708A9`, `46DBC0` | key law | missing |  |
| trinket 54 | Isaac's Head | 12 | shooter | `4708E6` | trinket | missing |  |
| trinket 57 | ???'s Soul | 13 | shooter-free | `470E5B` | trinket | missing |  |
| trinket 60 | Eve's Bird Foot | 15 | unique | `46BE30` | temp trinket | missing |  |
| (other spawners) | Blue fly | 43 | unique | `468AD0` | spawned | missing |  |
| (other spawners) | Blue spider | 73 | unique | `468CB0` | spawned | missing |  |

## Leech evidence (existing slice)

The text below is the original Leech slice's evidence record, condensed; the
full report is pinned in `references/isaac/foundation/familiars/`.

The Familiar resource catalogue contains the original base-Rebirth Leech3/56,
appearance Poof1000/15, and healing Heart1000/49 animations. The existing pinned
resource inventory owns all five PNG sheets; the browser now bundles them and
loads the extracted actor metadata through the same animation renderer.

Regenerate the metadata with `python3 tools/foundation/extract-animations.py
--select-file tools/foundation/familiar-animations.txt --out
web/familiar-data.json`. The recorded catalogue in
`references/isaac/foundation/familiars/authored-catalogue.json` pins the archive,
extractor, metadata, sheets, clip lengths and loop flags. Leech defaults to
Move Hori (12 frames, looping), Poof (43 frames) is nonlooping, and Heart (8 frames) is nonlooping.

The first runtime leaf routes AddCollectible52d173 into the supported Familiar
cache callback. The Player has separate permanent and temporary count owners;
the temporary getter searches actual records without sampling RNG. Each
Leech deficit consumes child270 once, samples one shared direction, clamps
world placement with the source 20-unit inset, admits the actor and explicitly
updates it once. The manager's captured old tail prevents another full update
in that same pass. Source room sequence IDs are allocated separately from
browser handles and survive ordinary room-clear retention for Player/Familiar
pointers; restored ordinary records receive new IDs.

The generic 5-frame appearance counter owns visibility. Birth takes 5 to 4; the
next full callback takes 4 to 3, samples a separate shared raw seed, creates the
real Poof15 and explicitly advances its authored frame0 to1. Fast callbacks
predict motion without advancing sprite/countdown/AI. Full nonplayer grid
responses run after entity contacts; fast prediction owns its preview response.
The Player/Familiar callback vetoes their mass separation for fresh Leech56.
Poof virtual Remove clears active and marks removed while retaining visibility;
its TTL branch is a separate dead-only law.

Run `npm run test:e2e -- familiars`. The default bank captures both real games
freshly through public commands: W0 opposite phase offsets and two unfiltered
OS-random treasure rooms, selected by room identity before item outcomes. The
gate requires all actual Familiar/effect presence, typed fields, source room
context and sequence order, child RNGs, full/fast world positions and Sprite
state. `references/isaac/foundation/familiars/initial-live.json` pins the
initial failing report (0/4, 25,131 of 80,878 checks failing: the initial
LightBeam33 and OS-grant Effect68, Ember seeds, actor order and sequence, Leech
placement and Poof seed). No failing capture has been replaced or called
passing.
