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
  Depths STBs are derived privately from the owner's `rooms.a`, never committed.

  Stage progression needs one trapdoor/next-stage controller. That is
  separate behavior work and builds on `references/isaac/foundation/basement-ii`.
- **Bosses.** The `boss()` draw replay stays source-exact. It returns a
  variant base, and the boss room, entity, portrait and music come from data.
- **Characters.** `players.xml` produces `CharacterDef`: hearts, coins, bombs,
  keys, items, costume, skin sheet and name. Stat literals in
  `player_stats.rs` become per-character data. `character_catalog()` and menu
  admission accept every character with a def.

### 2.6 Generic content e2e harness (built)

This runs on the suite registry (`tools/e2e/suites.mjs`) and the shared
`state-pair` driver.

- **Specs.** `tools/e2e/entities/<name>.mjs` is a short spec. It gives:
  - the entities2.xml type/variant;
  - an `about` line;
  - optional contrived `CODE:SLOT` routes;
  - the number of OS-random rooms per run;
  - the lifecycle markers both games must reach;
  - optional extra `[field, native, port]` rows.

  The `entities` suite runs every spec, and `entity-<name>` runs one spec alone.
  No other registration exists.
- **Scenarios.** `tools/original/entities-scenarios.py` builds every spec's
  cases. Room specs share one OS-random seed stream, and a room admitting
  several specs is one case graded for each; boss specs keep
  `boss-scenarios.py`. For each spec, `entity-scenarios.py --type T --variant V`
  draws OS-random run seeds. It keeps those where a room next to the start
  admits the entity. "Admits" means the port's own floor generator and authored
  spawn selection pick that entity in the room; the `floor` binary reports this
  with `{"spawns": true}`.

  Both games then play the same public route:
  1. the traversal door walk for that slot;
  2. dense entry captures;
  3. a four-direction shoot sweep;
  4. idle.

  It took about 10–40 seeds per room for Horf and Gaper. A recorded
  `scenarios.json` replays exactly with `--scenarios`.
- **Comparator.** `tools/foundation/entity-state-pair.mjs` and the shared
  `entity-compare.mjs` check these at every gameplay capture:
  - every authored NPC identity and spawn seed in the room;
  - the spec entity's shared NPC fields (identity, HP, collision,
    state/age/saved state, dead/active/visible, clip/frame/playing, the queued
    damage FIFO, and position/velocity/target);
  - the spec's extra rows;
  - room clear and countdown;
  - manager counts;
  - doors;
  - enemy projectile seeds.

  Each game must also reach the spec's required markers on its own. The
  markers are admission, damage, death, retirement and clear. All rows are
  exact.
- **Boss routes.** A spec with `boss: true` draws its cases from
  `tools/original/boss-scenarios.py` instead. Discovery keeps OS-random floors
  whose boss room holds the spec's boss and is reachable from the start
  through ordinary rooms with no hostile spawns (about 1 floor in 80; Dingle
  takes about 1 in 500 seeds).
  1. The production build records the route closed-loop and frame-stepped
     (`tools/e2e/boss-route-record.mjs`). It routes with the playtest bot's
     router, now shared as `tools/e2e/lib/route.mjs`, around rocks, poops,
     spikes and fireplaces, door to door.
  2. A fresh original replays the same keys. The seed is kept only if the
     original stays on the port's path at every `route-N` checkpoint and stands
     in the boss room at `boss-room-arrival`. Rejected seeds are listed with
     their reason and counted in the output's `discovery` block; contrived
     seeds that fail stop the run.
  3. The fight segment after `boss-room-arrival` is a Scene wait, a shoot
     sweep along every axis and idle. Later progression tapes (boss kill,
     trapdoor) start from the same arrival checkpoint.

  The comparator adds a `bossRoom` marker that both games must reach.
- **Source index.** `tools/original/ai-dispatch.py` reads the original's
  `Entity_NPC::update_ai` jump table (0x4f9670 → 0x66e390). It prints each
  type's Linux AI routine and the types that share it: 160 types, 107 routines.
  For example, Pooter shares `ai_fly` and Fatty shares `ai_gaper`.

## 3. Recipe: add an enemy or boss

1. Find its source:

   ```sh
   python3 tools/original/ai-dispatch.py --type N
   ```

   This gives the AI routine and its sharers. Read that routine in the pinned
   Linux binary. Families that share a routine share one port.
2. Write `engine/src/<family>.rs`. It needs:
   - a `FAMILY` entry (kinds, spawn, advance, grid query, flags);
   - the AI routine itself.

   The constructor starts from `Npc::authored(id, content::entity(t, v),
   selector)`. Stats, collider, gibs and actor come from entities2.xml. A plain
   `Entity_NPC` uses the source-faithful `plain_npc::advance` base, which
   provides appear, sprite clock, BaseUpdate, damage drain, death and
   retirement. Its family then supplies only its AI routine, built from shared
   primitives such as `try_shoot_player` and `play`/`finished`. Keep the
   source draw order and cite the source addresses.
3. Add one line to `entities::FAMILIES`.
4. Add the actor's ANM2 to `tools/foundation/port-npc-animations.txt` and
   regenerate `web/npc-data.json` with the command in that file's header
   (`extract-animations.py --select-file ... --out web/npc-data.json`; pass
   `--archive`/`--filenames`/`--fonts` from the main checkout when working in a
   worktree). Clip lengths and loop flags then come from the generated clip
   catalogue. The renderer draws any NPC that carries `actor`,
   and every monster/boss sheet is already globbed.
5. Add `tools/e2e/entities/<name>.mjs`, then run:

   ```sh
   npm run test:e2e -- entity-<name>
   ```

   Iterate on step 2. Replay a failing run exactly with `--scenarios`.
6. Run `npm run matrix:update` and open a small PR.

Boss rooms open through the generic boss gate once a registered family
constructs a boss that has a `bossportraits.xml` presentation (minibosses need
none). A boss spec adds `boss: true` and contrived entries are bare seed codes.
Clip events (`IsEventTriggered`/`WasEventTriggered`) come from the anm2
triggers in the clip catalogue: use `plain_npc::event_triggered` and
`event_was_triggered`.

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
- [x] M1 entity catalogue (#145)
- [x] M2 family registry and birth sink (#152)
- [x] M3 registry-derived contact admission (the Dip fix itself landed in #146)
- [ ] M4 clip and appear primitives: `plain_npc` (source generic appear, sprite clock, BaseUpdate, death) and `try_shoot_player` (FireProjectiles modes 0 and 1) are in; Horf, Fly, AttackFly, Pooter/Super Pooter and Gluttony use them; `walk_frame`/`set_frame` and the NPCAI pathfinder are shared too; the remaining legacy families still use their own copies
- [ ] M5 generic renderer and asset wiring: NPCs carrying `actor` draw from the catalogue, the generated clip catalogue covers every NPC actor, and every monster/boss sheet is globbed (only referenced ones decode); legacy families keep their renderer branches
- [x] M6 shared entity comparator (legacy comparators keep their own field sets for now)
- [x] M7 scenario finder, entity specs and the update_ai source index
- [ ] M8 scaffold
- [ ] M9 first import batch
  - Horf 12: **about 30 minutes wall-clock** from opening `ai_horf` to every
    Horf-owned field matching the original.
    - About 7 minutes went on reading the source (ai_horf is 207
      instructions, plus try_shoot_player and the generic appear) and writing
      the code.
    - Two live `entity-horf` runs took about 11 minutes each, with one fix in
      between (registry-derived contact admission).
    - Remaining failures:
      - the shared tear-trajectory gap;
      - the start-room door state;
      - death gibs, which have no admission yet;
      - champion rolls, which this branch adds.
  - Pooter 14.0 and Super Pooter 14.1: **about 80 minutes wall-clock** from
    opening the `ai_fly` Pooter branch to every Pooter-owned row matching
    wherever the Pooter's position matches.
    - About 15 minutes of source reading: the 4f0bd0 clip state, the 4f029a
      shot path, the shared movement block (Fly and Pooter clear their target
      every update) and FireProjectiles mode 1.
    - About 10 minutes of code, on top of `plain_npc`, `try_shoot_player` and
      the Fly family.
    - The rest went on tooling found along the way:
      - `npc-data.json` is now regenerated from its allowlist rather than
        edited by hand;
      - worktree setup;
      - a scenario-helper fix. A room with an unsupported co-spawn shifts the
        actor sequence ids, and with them every frame gate, so the helper now
        only picks rooms whose NPCs are all admitted (`floor` reports
        `unsupportedNpcs`).
    - Three live `entity-pooter` runs of about 11 minutes each.
    - Remaining failures:
      - the shared tear-knockback impulse;
      - wander driven by the shared-MT gap (#162), which also moves `flipX`,
        since the Pooter faces the player by position;
      - projectile seeds (#162).
    - Super Pooter is implemented from the source (FireProjectiles mode 1
      49f1c5: two shots at the aim rotated by about ±16°) but has no live
      evidence yet. The only 13x7 Basement room that authors it is 1/534
      (difficulty 10). A 25-minute OS-random search found no seed that puts it
      next to the start room. Live checks wait for large rooms or later stages.
  - Gluttony 49.0 (miniboss, #147): **about 35 minutes wall-clock** from the
    coordinator's go-ahead to a full paired run with every mechanic
    source-faithful. Most of that went on new shared foundations rather than
    on Gluttony's own AI (about 250 lines):
    - `pathfinder.rs`: NPCAI_Pathfinder MoveRandomly 5b4d40, including a
      native out-of-bounds stack read in the first side probe.
    - `lasers.rs`: Entity_Laser (ShootAngle, init/update, room-bounds endpoint,
      slab hit test, timeout shrink, the Brimstone Impact child). A subagent
      reverse-engineered it in parallel into `laser-notes.md`.
    - FireProjectiles mode 8.
    - A generic boss/miniboss room gate: type 6 rooms open once a registered
      family constructs the boss; there is no VS screen.
    - Shared stage HP (`content::initial_hp`).
    - Observer fields for the pathfinder and lasers, plus a generic laser
      comparison in `entity-compare.mjs`.
    - Verified against the original: Gluttony's first pathfinder choice is
      exact (target, direction, stuck counts and velocities). The laser
      lifecycle (timeout, endpoint and length, the shrink sequence and the
      impact scale) matches the original's beam frame for frame.
    - Open:
      - attack timing follows shared-MT draws, so it diverges until #162;
      - the "ISAAC VS GLUTTONY" streak and the boss HP bar (presentation);
      - Super Gluttony 49.1 needs lob projectiles 48d5c0;
      - laser render quads are approximated.
  - Dingle 261.0 (boss, first boss-route user): **about 60 minutes
    wall-clock** from opening `ai_dingle` to the first paired boss-route run.
    The boss-route harness itself (router library, recorder, original
    validation) took about 2 hours on top.
    - About 15 minutes of source reading, done by a research agent in
      parallel with the harness work.
    - About 30 minutes of code. New shared pieces: clip events from anm2
      triggers, `Birth::Npc` for mid-room spawns, the shared
      `find_free_position`, `Occupancy` modes and the runtime grid spawn
      (`GridEntity::spawn`, poop Init).
    - One live `entity-dingle` run of about 20 minutes with two contrived
      routes.
    - Verified: route, boss-room arrival, appear schedule, idle counter and
      the state-3 timing match the original exactly. Both cases then diverge
      at the first shared-MT consumer: a co-spawned Dip's direction, or the
      Idle attack roll (#162).
    - Open:
      - Spit, whistle, slide and the poop drop are unverified against the
        original until #162;
      - runtime poops draw no spritesheet roll yet (grid admission PR);
      - the boss death sequence goes through the generic NPC death.
  - Larry Jr. 19.0 (boss, multi-actor): **about 50 minutes wall-clock** from
    the RE notes to the first paired boss-route run (the playtest found it
    blocking 3 of 5 Basement runs).
    - New shared pieces: `boss_death.rs` (Entity_NPC state16, generalised
      from Monstro, used by every plain-NPC boss), generic `parent`/`child`
      handles on `Npc` with removal unlinking (448040), and
      `NpcWorld::peers`, a live view of the rest of the room list for
      families whose actors read and write each other.
    - Verified: route, arrival, segment linking (head election), appear,
      HP and every Larry-owned field match the original until the head's
      first MoveRandomly choice, which follows the room-entry shared-MT gap
      (#162).
    - Open: the head's grid Hurt on poop/TNT 40 px ahead (with grid Init
      after #162), death Explosion effects and gibs, Grub 239 and the
      19.1/subtype branches (The Hollow).
  - Grub 239.0 with Charger 23, Maggot 21 and Spitty 31 (#306), then
    Gurglings 237.1 and Gurgling 237.0 with Creep (Red) 1000.22: one family
    each per shared native AI (`larry.rs` for ai_larry, `maggots.rs` for
    ai_maggot_leech), so a new type sharing an AI is a branch, not a module.
    - Research ran in parallel agents (one per AI or effect) while the
      previous PR's suites ran; each family took about 30-45 minutes of code.
    - Boss-room seeds come from `boss-scenarios.py` planning; the playtest's
      own seeds often reach the boss only through hostile rooms, so specs
      use quiet-route stand-ins and say so.
    - Shared pieces: `Npc.grid_collided`, CheckLine mode 0, projectile
      height, flung NPC births without APPEAR, `extract-alpha-masks.py` for
      texel-gated gameplay (creep), and a creep render pass.
    - Verified: Grub linking, pooled HP and the head HP sync; Gurglings
      admission, HP and walk constants. Both diverge at their first
      MoveRandomly choice after the boss room's decorations/rocks draw
      (#162 grid admission).
- [x] Room shapes (coordinator priority 2): one generic admission gate
  (`traversal::admitted_shape`) opens ordinary type-1 rooms in all four
  authored shapes (13x7, 13x14, 26x7, 26x14). Geometry, eight door slots and
  CameraStyle1 scrolling were already shape-generic, from the tall-room leaf.
  - `entity-scenarios.py --shape WxH`, typeless specs, and the `shape` marker
    and `roomFields` rows in `entity-compare.mjs` make every shape a spec file:
    `tools/e2e/entities/room-26x7.mjs` and `room-26x14.mjs`.
  - Verified: geometry, bounds and player motion match the original exactly
    until a gameplay divergence.
  - Open:
    - the large-room backdrop is still the 1x1 assembly;
    - interior camera scroll differs by up to about 3e-5 (the known float
      gap);
    - player-versus-rock corner sliding differs (filed);
    - a runtime type-96 child appears in one 26x14 room;
    - spawnConfig 5c5240 alternate-variant rolls (separate PR).
- [ ] M10 stages
- [ ] M11 characters
