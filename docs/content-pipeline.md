# Content pipeline: adding enemies, bosses, stages and characters fast

Owner directive: the high-volume parts of the game (enemies, bosses, rooms,
stages, characters, items) need a fast, clean path to add, test and iterate.
This document maps how content is added today, names the repeated work, sets
the target architecture, and keeps the migration checklist. It follows
AGENTS.md. Base Rebirth is the only target. Verification stays at the game
level, through live original/browser pairs. There are no unit suites.

## 1. How content is added today

### Cost of the last additions

| Commit | Content | Files | Lines | Bespoke code outside the family module |
|---|---|---|---|---|
| `483ca61` | Dip 217 | 22 | +1123 / −104 | build.rs, npcs.rs ×4, lib.rs ×3, `dip: None` in 4 sibling constructors, renderer/engine/animation TS, registration in 5 places |
| `1cab195` | Monstro 20 | 29 | +891 / −39 | the same, plus combat.rs whitelists, enemy_projectiles, room.rs, traversal.rs boss scene, hand-copied boss-scene PNGs |
| `c9ce037` | lvl2 Fly 214 | 20 | +1006 / −28 | the same, plus grid.rs and familiars.rs effect plumbing |
| `fc7e629` | Fire 33 | 16 | +805 / −14 | the same, plus lib.rs Ember and loot birth wiring |

A new ordinary enemy touches about 12 engine/web sites in 9 files. It also
needs about 5 test-registration files and three hand-written test files: a
scenario script, a driver and a comparator. Only about half of the added lines
are the entity's own behavior.

### Where every entity is wired by hand

The engine has one `npcs::Npc` struct (`engine/src/npcs.rs`). It holds one
`Option<FamilyState>` per family: `keeper`, `fireplace`, `monstro`,
`level2_fly` and `dip`. Fly, AttackFly and RoundWorm have no family state.
There is no shared trait. The per-type dispatch sites are:

| Site | What is hand-written per type |
|---|---|
| `Npc::from_selector` (`npcs.rs`) | first-match chain of family probes, then inline 13/18/244 stat tuples |
| every family constructor | `other: None` for each sibling family (`deny_unknown_fields`) |
| `admit_room_pickups` (`lib.rs`) | `birth_game_update` stamping, one arm per family |
| `Game::advance_npc` (`lib.rs`) | six arms. Each builds its own `World` struct and handles its own output: blood seed, Ember and loot, projectile births, Worm shot |
| `Npc::advance` / `fly_ai` (`npcs.rs`) | `kind == 17`, `13` vs `18` branches |
| `Npc::entity_flags` (`npcs.rs`) | Option chain, one arm per family |
| `post_contact_grid` (`npcs.rs`) | `match kind`, one arm per family. Each family keeps its own `static COLLIDER` |
| `nonplayer_contacts` (`npcs.rs`) | `kind == 33` fire contact special case |
| `combat.rs` contact whitelists | literal `13 \| 18 \| 20 \| 33 \| 214 \| 244` |
| room clear / boss scene | `npc.monstro` flag check (`lib.rs`), `kind == 20` (`lib.rs`, `traversal.rs`) |
| `engine/build.rs` | NPC kind list, grid-points list, per-entity clip-frame and event-frame constants, Dip-only `stageHP` and gibs |
| `web/engine.ts` | TS mirror of `Npc`, one nullable object per family |
| `web/renderer.ts` | visibility filter by family, if-chain by family, nested ternary of literal ANM2 paths |
| `web/animation.ts` | hand-curated spritesheet PNG list |
| `tools/foundation/port-npc-animations.txt` | hand-curated ANM2 allowlist that feeds `web/npc-data.json` |

**Latent bug the registry would have caught.** Dip 217 (entity class 4) is
missing from both `combat.rs` contact whitelists. Those lists still read
`13 | 18 | 20 | 33 | 214 | 244`. A tear/Dip or NPC/Dip contact pair reaches
the whitelist `Err`. This needs its own PR with Dip suite evidence, not a silent
fix.

### Content data: what is data-driven and what is not

**Data-driven today:**
- STB room templates for Special Rooms and Basement (`room_assets.rs`,
  `include_bytes!`).
- The collectible catalogue and item pools (`item-catalog.py` → `items.rs`).
- Pickup definitions (`extract-pickup-catalog.py`).
- The sound and music catalogues.
- The familiar/effect `Definition` catalogue.
- Numeric NPC stats from `entities2.xml`, but only for the kinds listed in
  `build.rs`.

**Hand-coded today:**
- The list of supported entities and their AI.
- Floor generation (Basement 1/2 only, `floor.rs`). `boss()` returns only
  1010, 1020, 2010, 2050, 5020 and 5140.
- The stage backdrop and overlay (`room.rs` hard-codes `01_basement`).
- The character: Isaac stats are literals in `player_stats.rs`. Every other
  character fails with "not implemented".
- Every renderer actor path.
- The VS-screen portrait: Monstro and Isaac PNGs copied into
  `web/assets/boss-scene`.

**Parsed nowhere:** `players.xml` (parse check only), `bossportraits.xml`,
`backdrops.xml`, `bosscolors.xml`, `costumes2.xml`. `stages.xml` is used only by
the analysis decoder.

### Tests

Every entity suite already runs through `runPairedGate`
(`tools/foundation/paired-game.mjs`). The native observer is already generic:
`foundation-combat-observe.py` emits every room entity, and NPCs are detected by
vtable. The duplication is in the files around it:

- **Drivers** (`tools/e2e/{npcs,monstro,roundworms,fireplaces}.mjs`) are about
  90% identical. After normalising names, npcs and fireplaces differ by 43
  lines.
- **Comparators** (`tools/foundation/*-state-pair.mjs`, 135–340 lines) repeat
  the same pieces:
  - the manifest/case preamble;
  - the gameplay capture loop;
  - the spawn-seed actor join;
  - the same 25-field NPC list. It is repeated five times, with the same
    native→port renames (`npcState`/`state`, `removed`/`dead`, phase-0
    `presentedPosition`, and so on);
  - room/manager/door/player checks and Poof checks;
  - an execution-marker state machine;
  - the report tail.

  About 60–70% of each comparator is shared plumbing.
- **Scenario scripts** (`tools/original/*-scenarios.py`) copy
  `key/step/observed/dense` helpers (11 copies). Each hard-codes a seed that
  someone found by hand to contain the entity.
  - `pickup-scenarios.py` already has the right primitive. It draws OS seeds,
    runs the port's floor generator, decodes the neighbouring STB room, and
    filters its spawns by entity type.

### Import backlog: Basement and Cellar

Entity types spawned by authored rooms (rooms containing the type, from
`01.basement.stb` and `02.cellar.stb`). ✅ marks a type that is already
implemented.

- **Basement:** Attack Fly ✅ 70, Horf 53, Frowning Gaper 52, Gaper 48,
  Pooter 41, Fatty 41, Dip ✅ 40, Round Worm ✅ 35, Hopper 34, Fly ✅ 24,
  Gusher 24, Ring Fly 24, Skinny 21, Clotty 20, Wall Hugger 20, lvl2 Fly ✅ 18,
  Pacer 17, Mulligan 17, Squirt 13, lvl2 Spider 12, plus 22 rarer types.
- **Cellar:** Spider 155, Trite 57, Frowning Gaper 41, Sack 37, Dip ✅ 35,
  Big Spider 33, Mulliboom 32, Ticking Spider 30, Gaper 29, Fatty 27,
  Round Worm ✅ 25, Nest 22, …
- **Stage-1 boss rooms** reachable from `boss()`: Monstro ✅ (1010), Larry Jr.
  (1020), Duke of Flies (2010), Gemini (2050), and the 5020/5140 families
  (Dingle, Gurgling).

There are 42 Basement types and 46 Cellar types, about 60 distinct; 7 are done.
The pipeline has to make each of the rest a small, mostly data-driven change.

## 2. Target architecture

Principle: the original's data files are the registry. A new entity is defined
by three things: its `entities2.xml` row, which the build already has; an
optional behavior script where the original runs bespoke AI; and one registry
line. Lifecycle, serialization, rendering, assets and tests are generic.

### 2.1 Generated entity catalogue (build time)

`engine/build.rs` parses every `entities2.xml` entity into one generated table.
The struct is `content::EntityConfig`:

- **Identity and asset:** id/variant/subtype, name, `anm2path`
- **Stats:** baseHP, stageHP, boss, champion
- **Collision:** collisionDamage, collisionMass, collisionRadius, friction,
  shadowSize, gridCollision, collisionPoints
- **Other:** gibs (amount/flags), tags, portrait

For every ANM2 those rows reference, it also emits clip tables: name,
FrameNum, Loop and event frames. These are read from the extracted animation
cache.

Lookups become `content::entity(type, variant)` and `content::clip(actor, "Move")`.
These replace the per-kind `NPC_<k>_*` and `<FAMILY>_<CLIP>_FRAMES` constants
and the special cases in `build.rs`. The familiar and pickup catalogues are the
precedent for this.

On the web side, the same pass writes `web/generated/entity-catalog.json`,
mapping (type, variant) to its actor and portrait. It also regenerates
`npc-data.json` from the registry instead of `port-npc-animations.txt`.
`animation.ts` globs `gfx/monsters/**` and `gfx/bosses/**` as lazy URLs instead
of a curated list.

### 2.2 One NPC family registry (engine)

```rust
// engine/src/entities/mod.rs
pub(crate) struct Family {
    pub keys: &'static [(u32, u32)],            // (type, variant) admitted
    pub spawn: fn(&mut Npc, &SpawnContext) -> Result<(), String>,
    pub advance: fn(&mut Npc, Phase, &mut Mt19937, &mut NpcWorld) -> Result<(), String>,
    pub post_contact_grid: fn(&mut Npc, &Grid) -> Result<(), String>, // default: catalogue class
}
pub(crate) static FAMILIES: &[&Family] = &[&dips::FAMILY, &monstro::FAMILY, /* one line each */];
```

- **One world and one output sink.** `NpcWorld` is the union of today's six
  `World` structs: player position and velocity, grid, path costs, audio,
  system counter, game update count, player items, and effect admission.
  Families append to an ordered `births: Vec<Birth>`, where `Birth` is one of
  Blood(seed), Projectile, Ember, Pickup or Effect. `apply_births` admits them
  in producer order right after the callback returns. That is the same point
  and order as today, so shared-MT draws and actor ids are unchanged.
  Occupancy for RoundWorm-style dig targets is computed only when a family
  asks for it.
- **One family state.** The five `Option` fields become one `family` enum. The
  serialized snapshot keeps the current `npc.dip` / `npc.monstro` / … keys, so
  browser observers and `renderer.ts` keep reading the same shape.
- **Derived from the registry:** the spawn probe chain, `birth_game_update`
  stamping, `entity_flags`, the `combat.rs` whitelists (any registered type
  with entity class ≠ 0), `post_contact_grid`, the boss flag, and the room-clear
  boss flag.
- **RNG order is preserved.** There is still one callback per actor, in
  scheduler order. Families are never batched, and each family keeps its exact
  internal draw order, including conditional audio draws.

### 2.3 Shared behavior primitives

These are extracted from the existing families. Each takes parameters for its
known variations, so every call keeps its current draw order:

- `appear::advance`: the state-1 appear lifecycle used by 214, 217, 20 and 244.
  Parameters are the Poof subtype, whether Appear is gated, and the friction
  source.
- `sprite::Clip`: a catalogue-driven clip clock (loop/hold/events). It replaces
  six bespoke `update_sprite` functions and `play()` copies.
- `motion::{direction, normalized, divided}`: one copy each. The float-parity
  comment stays.
- `base::update_tail`: friction, integration, FIFO damage drain and death test,
  with an explicit order flag for the Dip and Monstro variants.
- `base::intermediate`: half-velocity or friction-weighted prediction.
- `death::standard`: SFX, gib sampling from the catalogue gib data, and blood,
  in source order.
- Movement AIs as they appear: `wander`, `chase`, `hop`, `fly_orbit`, and
  `shoot(pattern)` through `enemy_projectiles::birth_with_parameters`.

A behavior script is the entity's `spawn` plus its `advance` written with those
primitives. Simple enemies such as Gaper, Horf or Pooter should be 50–150 lines.

### 2.4 Generic rendering

The engine exports generic presentation fields on every NPC: `flipX`, `tint`,
`offsets`, `layerScales` and `clockSource`. Today these are per-family
overrides. `renderer.ts` draws every NPC with one path:

```text
catalog[kind.variant].actor → draw(npc.animation, clock, presentation)
```

Bosses get their VS portrait from `bossportraits.xml`. The PNG is pulled
straight from `gfx/ui/boss`, falling back to `graphics.a` extraction, instead of
being copied by hand.

### 2.5 Stages, bosses and characters as data

- **Stages.** `stages.xml` and `backdrops.xml` produce `StageDef`:
  - STB asset
  - backdrop, overlay and rock sheet
  - music
  - boss variant table

  `floor.rs` takes a `StageDef` instead of `stage == 1` predicates.
  `room_assets.rs` loads the STB named by the stage. The missing Caves and
  Depths STBs are extracted from `rooms.a` and committed next to the existing
  ones.

  Stage progression needs one trapdoor/next-stage controller. That is
  separate behavior work and builds on `references/isaac/foundation/basement-ii`.
- **Bosses.** The `boss()` draw replay stays source-exact. It returns a
  variant base, and the boss room, entity, portrait and music come from data.
- **Characters.** `players.xml` produces `CharacterDef`: hearts, coins, bombs,
  keys, items, costume, skin sheet and name. Stat literals in
  `player_stats.rs` become per-character data. `character_catalog()` and menu
  admission accept every character with a def.

### 2.6 Generic content e2e harness

This builds on the suite registry in #137 (`tools/e2e/suites.mjs`).

- `tools/foundation/entity-compare.mjs` provides:
  - `loadPair`
  - `forEachGameplayCapture`
  - `joinBySpawnSeed`
  - one declarative `COMMON_NPC_FIELDS` native→port map
  - room, door, player, Poof and projectile check sets
  - `ExecutionMarkers` with standard predicates: admission, appear, damage,
    drain, death, retirement, clear
  - one report writer
- `tools/e2e/entities/<name>.mjs` is a short spec: the type/variant filter, the
  browser family key, extra field pairs, extra marker predicates and the
  required markers. The registry discovers every spec as a suite id
  `entity:<name>`. No npm script, `LIVE_SCRIPTS` edit or matrix row is needed.
- `tools/original/content_scenarios.py` provides:
  - shared `key/step/observed/splice` helpers;
  - `find_seeds(type, variant, room_kind)`. It runs rejection sampling over
    fresh OS seeds, using the port floor generator (`engine/src/bin/floor.rs`)
    and the STB decode. Every accepted seed is written into `case.json`, so
    replay is exact.
  - a route template: walk from the start room to the target room (neighbour
    or boss room), dense entry captures, a shoot sweep, then the clear tail.

  Each spec yields contrived cases (a fixed seed bank) and OS-random cases (a
  fresh draw that contains the entity) on every run.
- **Native offsets** move into one shared table module. The five observers that
  re-declare the entity layout import it, and type-specific extras become table
  rows.

## 3. Recipe: add a new boss (target state)

1. `npm run content:new -- 19.0 larry_jr` scaffolds three things:
   - `engine/src/entities/larry_jr.rs` (`FAMILY` with spawn/advance stubs);
   - the registry line;
   - `tools/e2e/entities/larry-jr.mjs`.

   Stats, collider, clips, sprites, portrait and boss-room variants already
   come from `entities2.xml`, the ANM2 cache, `bossportraits.xml` and the STB.
2. Write the behavior in `larry_jr.rs` using the shared primitives. Keep the
   source draw order and cite the original function addresses in comments, as
   today.
3. If the boss is new to the stage pool, set its variant base in the
   `StageDef` boss table. The `boss()` replay already draws it.
4. Run `npm run test:e2e -- entity:larry-jr`. The harness finds OS seeds whose
   boss room is Larry Jr., builds the public route to it, and compares the
   common NPC, room, door and player fields. It also compares the boss's
   segments, which share spawn seeds in both games.
5. Add only the boss-specific extra fields and markers to the spec. Iterate on
   steps 2 and 4. `--replay-cases` replays any failing case exactly.
6. Run `npm run matrix:update`, write the evidence doc, and open a small PR.

An ordinary enemy is the same without step 3. Its scenarios come from normal
rooms that contain the type.

## 4. Migration order

Every PR is small and goes straight to main. It must not change any existing
live comparison. Before a PR is offered for merge, the affected suites are run
locally on main and on the branch, and their failure sets compared. Each PR
also runs `npm run check:build`.

| # | Change | Removes | Verified by |
|---|---|---|---|
| M1 | Generated `EntityConfig` catalogue from `entities2.xml`. `NPC_<k>_*` consts become catalogue lookups | build.rs kind lists and special cases | npcs, roundworms, keepers, monstro, fireplaces, dips |
| M2 | `Family` registry, `NpcWorld`, ordered `Birth` sink, single family enum with the same snapshot keys | the `advance_npc` chain, probe chain, `None` boilerplate, `entity_flags`, `post_contact_grid` match, birth stamping | the same suites, plus blood-start and player-death |
| M3 | Registry-derived contact whitelist, including the Dip fix as its own commit with Dip evidence | `combat.rs` literals | dips, npcs, fireplaces |
| M4 | `sprite::Clip` and `appear::advance` primitives, migrated family by family | six clip clocks, four appear copies | the suite of each migrated family |
| M5 | Generic renderer, generated entity catalogue JSON, lazy sprite glob, `npc-data.json` generated from the registry | renderer chain, curated PNG and ANM2 lists | typecheck, build, pixel diagnostics unchanged |
| M6 | `entity-compare.mjs`. npcs, monstro, roundworms, fireplaces and dips comparators ported with identical field sets | about 600 lines of duplicated comparator code | each ported suite's check count and failures unchanged |
| M7 | `content_scenarios.py` `find_seeds` plus route template, and `tools/e2e/entities/` discovery | hand-found seeds for new content | a new spec for an existing entity matches its legacy suite |
| M8 | `content:new` scaffold | hand wiring | first import |
| M9 | First import batch: Gaper/Frowning Gaper, Horf, Pooter, Fatty, Hopper, Gusher/Pacer, then the stage-1 bosses | — | entity suites; time per entity recorded below |
| M10 | `StageDef` from stages.xml/backdrops.xml, generalized `floor.rs`, Cellar and Basement II assets | Basement literals | floor, level-descriptors, traversal |
| M11 | `CharacterDef` from players.xml | Isaac literals | run-admission, ordinary-run, menus |

The e2e work in M6 and M7 depends on #137 (the suite registry) landing. The
engine work in M1–M5 does not.

## 5. Checklist

- [x] Survey and design (this document)
- [ ] M1 entity catalogue
- [ ] M2 family registry and birth sink
- [ ] M3 registry contact whitelist and Dip contact fix
- [ ] M4 clip and appear primitives
- [ ] M5 generic renderer and asset wiring
- [ ] M6 shared comparator library
- [ ] M7 scenario finder and entity specs
- [ ] M8 scaffold
- [ ] M9 first import batch (time per entity: —)
- [ ] M10 stages
- [ ] M11 characters
