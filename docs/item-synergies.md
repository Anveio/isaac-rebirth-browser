# Item synergies: design

This is the design for porting every Rebirth item and synergy. It comes before
any framework code. A synergy is not a separate feature of the original. It is
what falls out when several items pass through the same composition rules. So
the port reproduces those rules in the original's structure and order, and
synergies follow from them. Hand-written pair cases would never finish (there
are 343 collectibles and 60 trinkets) and would drift from the game.

Progress is tracked in [items.md](items.md), per item, per mechanism and per
tested item set.

## How the original composes items

All addresses below are in the pinned Linux `isaac.x64` (see
`tools/e2e/lib/original.mjs`). `tools/original/item-xrefs.py` disassembles
that local binary and writes
[item-xrefs.json](../references/isaac/foundation/items/item-xrefs.json): for
every function, the literal item ids it queries, and for `EvaluateItems`, the
ids read by each cache block. The file holds derived addresses and ids only.

**Item state.** A Player keeps one signed count per collectible at
`Player+0xE1C + 4*id` (346 slots). `HasCollectible` (`5090C0`) is
`count > 0`, plus a trinket-21 (Mysterious Paper) branch for 327/328.
`HasTrinket` (`508FE0`) scans the held trinket slots `+0xE14/+0xE18`.
Temporary effects (books, pills, cards) keep their own records, queried by
`5E3ED0` (count) and `5E3D70` (presence). Most stat blocks read the count array
directly, so they never need a query call.

**Acquisition.** The chain is `52E1A0 -> AddCollectible 52CC90 -> 511F10 -> 581750`.
AddCollectible increments the count and ORs the item's `cache` flags from
`items.xml` into `Player+0xD6C`. It applies the config's one-time grants
(`hearts`, `maxhearts`, `soulhearts`, `blackhearts`, `coins`, `bombs`,
`keys`), registers the costume, and runs the per-id one-time branch
(`52CF87` dispatch, then the `52D0D0` transformation tail). After that,
`EvaluateItems` runs on the pending flags and `581750` removes the item from
the pools.

**EvaluateItems `511F10`.** This is the heart of composition. It is one
function that reads the pending flag word D6C and runs a block for each
requested flag, in this fixed order:

| Flag | Block entry | Writes | Items the block reads (binary) |
| --- | --- | --- | --- |
| 0x100 weapon | `513AD0` | primary weapon C08 | 8 (precedence 304, 168, 114, 52, 118, 68, 329, 229) |
| 0x1b with Libra | `5122C0` | expands the request | 304 |
| 0x2 fire delay | `5122D3` | MaxFireDelay C78 | 29 + trinkets 10, 39 |
| 0x4 shot speed | `511F68` | C7C | 15 + trinket 27 |
| 0x1 damage | `5134B0` | damage | 44 + trinket 35 |
| 0x8 range | `5127C8` | range / tear height C8C/C88 | 23 + trinket 26 |
| 0x10 speed | `512C48` | move speed | 29 + trinket 37 |
| 0x20 tear flags | `5130CA` | C98 tear flags | 25 + trinkets 9..12, 26 |
| 0x40 tear color | `513355` | CA0/CCC colors | 19 |
| 0x80 flying | `512166` | flight | 9 |
| 0x200 familiars | `513E80` | familiar roster via CheckFamiliar `508D10` | 58 + trinkets 54, 57 (68 familiar calls) |
| 0x400 luck | `513C80` | luck | 3 + trinket 42 |
| always | `5121EF` | final Libra average and the clamps (fire delay at least 1, shot speed at least 0.6, ...), then D6C = 0 | 304 |

Each block is straight-line arithmetic over the counts, always in the same
order: additive terms, then multipliers, then character and challenge
branches, then clamps. Every block reads every relevant item on every call.
No block ever asks whether the whole inventory is known.

The tool cross-checks each block against the `cache` attribute in `items.xml`.
Of 112 items with explicit flags, 24 declare a flag whose block never reads
their id as a literal. Examples are Spoon Bender speed, Cricket's Head shot
speed and Technology 2 weapon. Those effects live elsewhere (a request with no
term, a temporary record, or a read through a register or table). Another 9
items are read by a block their config does not request; Inner Eye, for
example, is read by the damage block. These lists are in
`cacheCrossCheck` and are leads for the stat port, not bugs.

**Consumers.** Ranked by how many item ids each function queries, the special
cases sit in a few large functions:

| Function | Ids | Role (from port evidence) |
| --- | --- | --- |
| `519FC0` | 99 | Player fire: weapon dispatch, multi-shot, per-item shot branches |
| `50D090` | 36 + 2 trinkets | Tear and attack property getter by kind (flags, variant, scale) |
| `52AE90` | 37 + 9 trinkets | RemoveCollectible and its inverse effects |
| `52F370` | 15 | Secondary C40 caller (lasers, thin beams) |
| `473180` | 12 | Tear and laser update branches (Brimstone 118, 213, 89, ...) |
| `5177F0` | 5 | FireTear factory (154, 69, 309, 182, 330) |
| `515F40` | 9 + 2 trinkets | Temporary effect ticks (Cookbook bombs, ...) |
| `47EC60`, `4F9670`, `49B890` | 37, 26, 16 | NPC Init, NPC AI and enemy filters reading items |
| `45A3E0` | 118 | Not yet identified (reads more ids than any function but 511F10) |

The full list (94 functions) is in `item-xrefs.json`. Weapon synergies (for
example Brimstone with Knife, Fetus, Technology or Ludovico) come from three
things combined:

1. **Weapon selection.** The `0x100` block picks one primary weapon C08 by a
   fixed precedence.
2. **The fire routine.** `519FC0`, `52F370` and `5177F0` branch on that
   primary and on individual item counts.
3. **Attack properties.** The `50D090` getter combines tear flags, color and
   damage into whichever body is fired (tear, laser, knife, bomb).

Port each of these once, the way the original structures it, and every
weapon pair composes.

## What is wrong today (#895)

The port's `item_pipeline` gates every stage on an *admitted readset*: a
hand-written list of the item sets that stage has been proven for (for example
`FireDelayStage::EFFECT_ITEMS` and `NEUTRAL_ITEMS`, and `admits_*_context`).
When the Player owns any item outside the list, the stage refuses and its
requested cache bits stay pending. `weapon_plan::resolve_live_weapon` then returns
`Unadmitted("regularTearCache")`, so the regular-tear controller never fires.

The Odd Mushroom (Thin, 120) shows the whole chain. Its Add requests
`damage firedelay speed`. The binary's fire delay, damage and speed blocks all
read 120, but the port's lists do not include it. So the bits stay pending and
Isaac stops shooting. 101 collectibles request at least one of the
damage, fire delay, shot speed, range, tear flag or tear color flags (or
`all`). Every one of them outside the lists does the same. This is the `cache-admission` mechanism in the tracker. It is
structural, so special-casing 120 would not fix it.

## Decisions

1. **Port the original's structure, not pairs.** `EvaluateItems` becomes one
   port function with one block per flag, in the binary's order, reading the
   whole count array. Synergies are never written as pair rules. A pair that
   differs from the original is a bug in a block, a consumer or the order.

2. **Stats always resolve; missing behaviour is local.** Readset admission is
   removed. Every requested block computes from all counts. An item's
   behaviour that is not ported yet (a tear flag's on-hit effect, a
   familiar's AI) records a frontier where it is consumed. It never leaves the
   cache pending or stops the weapon. Picking up any item leaves Isaac
   playable. This closes the #895 class.

3. **Items are rows; order comes from the binary.** Each item is one entry in
   a `passive_items` table, beside `active_items::USES` and the D4 agent's
   `collectible_grants`, which this extends rather than duplicates. An entry
   lists the item's effects:
   - stat terms (block, operator `add`/`mul`/`set`/`clamp`, constant, count
     source permanent/temp/either);
   - tear flag bits and tear color;
   - flight;
   - familiar rows (variant and count source);
   - a weapon precedence slot;
   - one-time Add grants;
   - costume.

   Each term carries the **original instruction address** that reads it. At
   build time a block runs its terms sorted by that address, so f32
   operations happen in the source's order. Per-item rows give the owner's
   "an item is a row", and the address order keeps float parity and points
   every term at its evidence. Character, challenge and trinket branches inside
   a block are also rows, keyed the same way.

4. **Consumers stay code, registered by mechanism.** Behaviour in `519FC0`,
   `50D090`, `5177F0`, `473180` and peers is ported as functions following
   those originals: one fire routine and one property getter, not one per
   item. An item row names the consumer mechanisms it takes part in, so the
   tracker can tell whether every consumer of an item is ported.

5. **The bulk import is data, from build-time resources.** Once the framework
   lands, one import brings in every item, trinket and familiar. `engine/build.rs`
   already reads `items.xml` and `itempools.xml`. It will also read
   `costumes2.xml`, the item, trinket and costume sprites and the familiar
   anm2s from the local original resources at build time, as other content
   does. None of it is committed. After the import, every item has config,
   pools, pedestal sprite, costume and Add grants. Behaviour is then a matter
   of filling block terms and consumers, tracked per item and per mechanism.

6. **The binary readset is the checklist.** `item-xrefs.json` lists, per block
   and per consumer, the ids the original reads. A mechanism is complete only
   when every id the binary reads there is a port term or consumer branch, and
   its live cases pass. The tracker shows *ported terms / binary terms* for
   each block.

## Verification

Live original-vs-port cases remain the only acceptance. Every comparison is
per field: a failing field stays a failure, and pixels and video are
diagnostics.

**Granting arbitrary item sets in both games.** The
`pickup-interaction-scenarios.py` sandbox already rewrites base Challenge 1's
`startingitems` and the Start Room spawns from the local resources. Both games
read the same generated bytes. That is a public resource input, not injected
state, and it grants any set of collectibles, a trinket and a card. The
synergy pipeline reuses it:

- **Case generator.** `tools/original/item-set-scenarios.py --items 1,118,329
  --enemy 10.0 [--trinket N]` writes one sandbox case: grant the set, fire
  each direction for a fixed tape, spawn the chosen enemy, then let the tears
  and bodies resolve.
- **Bulk generation from item data.** Pairs and triples are chosen from
  `item-xrefs.json`, never by hand. For each block, cross every pair of items
  that block reads. For weapons, cross every pair and triple of the 8
  precedence items with the tear-modifier items read by `519FC0` and
  `50D090`. Weight by pool weight, and journal OS-random sets beside the
  contrived ones, as every suite does.
- **What is recorded.** The observer reads the Player's evaluated fields after
  the grant: counts, damage, MaxFireDelay, shot speed, range and height,
  speed, luck, C98 tear flags, CA0/CCC colors, C08 primary, flight and the
  familiar roster. It also records every born tear, laser, knife or bomb with
  its source fields, and enemy HP over time. The port records the same fields
  from its state. The grader compares each field exactly: f32 bit-exact, and
  flags and ids equal.
- **Video.** `npm run playtest:video` renders the two recordings side by side
  for review. It is never a pass/fail input.
- **Results feed the tracker.** Each graded set is written to the *Tested item
  sets* table as match or mismatch, with its case name.

**Instrumenting the original (research, not a gate).** For fast iteration on a
block, a gdb probe on the original can break at `5122B0` (the
`EvaluateItems` return) and dump D6C plus every written field for each call.
This gives the exact inputs and outputs of each evaluation during a live
case. It lives in `.cache/research`. A probe that writes counts into the
original's memory, then calls `511F10`, can sweep thousands of item vectors
in seconds. It is only a research oracle for finding the term order; the
acceptance gate stays the public-input sandbox cases above.

## Debug sandbox floor (port only, compiled out of release)

Base Rebirth has no item room (Death Certificate is Repentance-only), so the
port adds its own for playtests:

- **Floor and enemy.** A debug-only floor lists every imported collectible and
  trinket on pedestals, paged across rooms by pool. Isaac can pick up any
  number of items. A selector spawns a chosen enemy or boss to fight.
- **Recording.** A *record case* control exports the current item set, trinket
  and enemy to the case generator above. The same set then runs in both
  games through the sandbox Challenge and records side by side. The
  playtester can generate one case per pass with
  `npm run sandbox:case -- --items ... --enemy ...`.
- **Compiled out.** The floor is behind a Rust cargo feature `debug-sandbox`
  and a Vite define. The release build and `npm run playtest:bundle` build
  without them. `check:build` scans the shipped bundle for the sandbox marker
  strings, so a leaked debug floor fails the build.

## Order of work

1. This design, merged alone.
2. The framework:
   - `EvaluateItems` blocks as address-ordered terms over the whole count
     array (done for FireDelay and Damage; ShotSpeed, Range and Speed were
     already whole);
   - readset admission removed (done for those five scalar blocks, with live
     Odd Mushroom cases; #895 closed); Flight, Familiar, Luck and the tail
     still to follow;
   - `passive_items` rows on top of `collectible_grants`;
   - per-block *ported/binary* columns in the tracker.
3. The synergy pipeline:
   - the item-set case generator;
   - the per-field grader;
   - the debug sandbox floor;
   - all pairs among already-ported items run and recorded.
4. The one-time bulk import of every item, trinket and familiar (data and
   assets).
5. Behaviour, block by block and consumer by consumer, highest pool weight
   first. Each step is proven by generated live cases and updated in
   [items.md](items.md).
