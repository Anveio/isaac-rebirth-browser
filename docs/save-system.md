# Save system design

This is the design for saving a run and continuing it. It replaces the
narrow "admitted Continue domain" approach recorded in
[save-continue.md](save-continue.md), which stays as the evidence log of the
earlier slices. The goal: **Continue resumes every run the port can create**,
exactly as base Rebirth would, from durable browser storage, with the
original's anti-save-scum rules and without any Steam interface.

The design has two halves. Part 1 is what the original binary does, derived
from the Linux x86-64 executable (`isaac.x64`, build ID `5531b6a9…`) with
function names taken from the symbolized macOS build
(`references/isaac/native/isaac-rebirth.macho`; its code is encrypted, but
its symbol table names `GameState::write`, `Game::SaveState` and the rest).
Part 2 is the port: record model, save points, Continue pipeline, storage,
schema migrations and the parity test plan. The coverage tracker at the end
records progress across PRs and container restarts.

Addresses below are Linux `isaac.x64` virtual addresses unless they are
prefixed `mac:`. Field names marked *(offset only)* have a known location,
size and save/restore path but no confirmed meaning yet; they are still saved
and restored verbatim.

## Part 1: the original

### Files and the Steam layer

Each of the three save slots owns two files in the save directory:

| File | Owner | Purpose |
| --- | --- | --- |
| `persistentgamedata%d.dat` | `PersistentGameData` (Manager+0x10) | Profile: achievements, counters, unlocks, collection history, and the checksum of the slot's current run record |
| `gamestate%d.dat` | `GameState` (Manager+0x4A8) | The suspended run |

`Manager::SelectSaveSlot` (594DC0) clears the in-memory GameState (55BD80),
loads the profile, then loads the run record into Manager+0x4A8. When
`Manager::IsSteamCloudAvailable` (5943F0: `SteamRemoteStorage()` is non-null
and Options.ini `SteamCloud` is set) is true, both files go through
`SteamCloudFile` (`GameState::SaveToSteamCloud` 55B890,
`GameState::LoadFromSteamCloud` 55F510, `PersistentGameData::SaveToSteamCloud`,
`::LoadFromSteamCloud`) under bare names (`gamestate%d.dat`). Otherwise they
are plain files under the user directory (`%sgamestate%d.dat`,
`GameState::Save` 55B7B0, `GameState::Load` 55F430). The record format is
identical on both backends; Steam only supplies the byte store. Achievements
are additionally mirrored to Steam (`ImportSteamAchievements`,
`unlock_steam_achievement`).

### When the original writes and deletes the run record

`Manager::SaveGameState` (594810) refuses outside gameplay ("Cannot save game
state while not in game", Manager state ≠ 2) and skips the write while
player 0 is dead (Player+0x31F). Otherwise it calls `Game::SaveState`
(54F230) into Manager+0x4A8, stores the selected player type
(Manager+0x2F2CC → GameState+0x8898), writes the file (Steam or local),
then binds the record into the profile with
`PersistentGameData::SetGameStateChecksum` (5BF010, PGD+0x48C ← GameState+0xEF94)
and saves the profile (5BD3A0).

It is called from exactly two places:

1. **The end of `Level::Init`** (58BE30, call at 58C411): every new floor,
   including the first floor of a new run, after the starting room is
   loaded.
2. **`Game::Exit(save=true)`** (54F3F0, call at 54F5D0): Pause → Exit to the
   menu.

`Level::ChangeRoom` does *not* write the file. It runs `Room::SaveState`
(5CE2D0) on the room being left, which only updates that room's in-memory
Level record. A run killed without Exit therefore continues from the start
of its floor.

`Manager::DeleteGameState` (594910: `GameState::Delete` 55CE40, then
`SetGameStateChecksum(0)`) removes the record:

- `Game::Exit(save=false)` (54F441);
- the GameOver screen's init (558900, call at 558A79);
- the run-end trigger (5529D0), reached from the ending pickup path
  (505A48, 505C1F) that also fires the ending (594FE0);
- the pause-menu exit while player 0 is dead (5BBB40, call at 5BC1E4).

### Game::SaveState order (54F230)

1. `GameState::Clear` (55BD80) on the destination.
2. Game scalars: seed text Game+0xA1E0 → GS+0xEF80 (9 bytes, `XXXX XXXX`),
   PlayerManager RNG word Game+0xA138 → GS+0xEF88, Game RNG word
   Game+0xA0F8 → GS+0xEF8C, challenge Game+0x10EB10 → GS+0, Game+0x10ECD0
   (achievements-disabled / seeded byte) → GS+4, Game+0x10ECE0 → GS+5,
   difficulty Game+0x10ECD8 → GS+8, 33 game-state flags (bitmask
   Game+0x10EAE0) → GS+0xC/0xE, seven counters Game+0x10EAE8..0x10EB00 →
   GS+0x30..0x48 *(offset only)*, Game+0xCE9F8 → GS+0x4C, Game+0xCE9F4 →
   GS+0x50, Game+0x10ECD4 → GS+0x88, Game+0x10ECDC → GS+0xEF90.
3. `ItemPool::StoreGameState` (581ED0) from Game+0x8BD0.
4. `Room::SaveState` (5CE2D0) on the current room (Game+0x8180).
5. `Level::StoreGameState` (58CD90).
6. Item-queue flush on player 0 (52EE40: a queued collectible is granted
   now, "item queue flush").
7. `PlayerManager::StoreGameState` (5BF400), which calls
   `Entity_Player::StoreGameState` (518350) for player 0.
8. `History::StoreGameState` (576BD0) from Game+0x8BA0.
9. `Ambush::StoreGameState` (5B83B0) and the nightmare scene owner (415420).

`Room::SaveState` keeps only the families accepted by 5C57C0: bombs (4),
pickups (5), slots (6), shopkeepers (17), fireplaces (33) and the listed
effects (1000, with the variant set 5C57C0 tests: 21, 33, 39, 63–65, 68,
69, 74 and 76, under its flag conditions). NPCs, tears, projectiles, familiars and the player are not
entity records. It also refreshes the room's 448 grid descriptors, drawing
two fresh shared-MT words for each empty cell.

### GameState record layout (version 18)

`GameState::write` (55A240) emits a 16-byte header `ISAACNG_GSR0018`, then
every field as little-endian scalars, then a checksum trailer. The running
checksum (GS+0xEFB8) starts at `0xFEDCBA76` (42F810); each u32/u16/u8 does
`c = rotl(c, 31) + v` (42F820), each float does `c = rotr(c, 1) + (int)(f*666)`
(42F830), byte arrays fold four bytes at a time (42F850). The header is not
summed. The trailer is `checksum ^ 0x96696996`; the unxored value is kept in
GS+0xEF94 for the profile binding.

| GS range | Contents | Writer |
| --- | --- | --- |
| 0x00–0x08 | challenge, achievements-disabled byte, GS+5 byte, difficulty | Game |
| 0x0C–0x2E | u16 count (33) + 33 game-state flags | Game |
| 0x30–0x50 | seven Game counters *(offset only)*, Game+0xCE9F8, Game+0xCE9F4 | Game |
| 0x58 | map: u32 count + per entry 6 fields (u32, u32, u8, u32, u32, u32) | History (acquisition history) |
| 0x88–0x90 | Game+0x10ECD4; nightmare-scene byte and word | Game, 415420 |
| 0x94–0xAF | stage (Level+0), stage type (Level+4), Level+0x8148, Level+0x8190, Level+8, three Level bytes, Level+0x81A8, Level+5 | Level |
| 0xB0–0xBC | u16 count (11) + 11 Level flags (bitmask Level+0x81B0) | Level |
| 0xC0–0xEC | twelve u32 *(offset only; writer not yet identified)* | — |
| 0xF0 | room count; then each room `0..count-1` and the seven special rooms `0x80..0x86` | Level |
| per room | RoomConfig reference (3 u32, GS+0x7F88+i·12); descriptor (stride 0xF0 from GS+0x100): grid index, safe grid index, 8 neighbour/door words, flags (visited, cleared, …, 6 bytes), four seeds (spawn, decoration, award, …), **448 grid descriptors** (type, variant, state, var-data, seed, byte, u32), **saved entity list** (EntitySaveState, u32 count), persistent-spawn set (u32 count + u32 keys), second EntitySaveState list, two u32 and six u32 trailers | `GameState::write_Room` (559BF0) |
| 0x85DC | 13×13 room grid (169 i32, Level+0x7EA0) | Level |
| 0x8880–0x8894 | current room (Level+0x8188), previous room (+0x818C), entry position vector (+0x819C), Level+0x81A4, entry door slot (+0x8198) | Level |
| 0x8898–0x8920 | selected player type; player type (Player+0xBFC); max red, red, soul hearts, black-heart mask, eternal hearts (summed/merged over co-op players); golden hearts, coins, keys, bombs, golden key byte, Player+0xBF0, Player+0xBF5; 13 stat inputs Player+0xD28..0xD58 (speed, fire delay, damage, …); active charge (D88), active item (D74), D78, battery charge (D8C); two float vectors | Entity_Player |
| 0x8924–0x8933 | two pocket records {u32 id, u8 kind} (Player+0x3044, +0x304C) | Entity_Player |
| 0x8934 | two trinket slots (Player+0x3068) | Entity_Player |
| 0x893C | 347 collectible counts (Player+0xE1C) | Entity_Player |
| 0x8EA8 | two u32 (Player+0xE14) *(offset only)* | Entity_Player |
| 0x9628 | costume list: u32 count + {u32, u32} per costume | Entity_Player |
| 0x8EB0, 0x941C, 0x9514 | per-collectible (347), per-trinket (62) and per-pocket (41) RNG words | Entity_Player |
| 0x95B8 | 28 pill-effect history words (Player+0x1388..0x13F4) | Entity_Player |
| 0x9648 | 16 words, first two from PlayerManager's list | PlayerManager |
| 0x9688–0x9690 | PlayerManager RNG (PM+0x40), Player+0xC70, Player+0xBEC | PlayerManager, Entity_Player |
| 0x9694 | 16 pools × 347 item weights | ItemPool |
| 0xED54 | 16 per-pool words | ItemPool |
| 0xED94–0xEDA4 | ItemPool seed and four RNG/factor words (one float) | ItemPool |
| 0xEDA8, 0xEF03 | removed-collectible (347) and removed-trinket (62) flags | ItemPool |
| 0xEF44–0xEF79 | pill-effect table (count + 10) and 10 identified-pill flags | ItemPool |
| 0xEF80–0xEF90 | seed text, PlayerManager RNG, Game RNG, Game+0x10ECDC | Game |
| trailer | `checksum ^ 0x96696996` | write |

EntitySaveState (559950) is 16 fields: type, variant, subtype, four u32
(charge, price, touched, …), one byte, position x/y (floats), one byte, the
initial seed (+0x2C), a u32, a u64 drop/RNG record and a trailing u32/byte.
`Room::restore_entity_from_state` / 5C5860 respawns each record with its own
seed, so a chest, pickup or slot comes back exactly as it was.

### Continue (load) in the original

`GameState::read` (55DB90) parses the header with `ISAACNG_GSR%u` and accepts
versions **8 to 18**. Fields that were added later are read only when the
file's version is new enough (gates at versions 10, 11, 13, 14, 15, 16 and
17); older files keep the constructor defaults. Every list count is bounded
("Invalid room count", "Invalid history count", "Invalid costume count";
counts are clamped, a failed room read aborts). The trailer must equal the
running checksum **and** the checksum the profile holds (PGD+0x48C, compared
at 55F3E0). Any failure prints "Checksum invalid!", deletes the in-memory
record and leaves the loaded flag (GS+0xEFA0) clear, so the main menu
(5A4427 and siblings, Manager+0xF448) offers no Continue. A failed read does
not fall back to anything.

`Game::StartFromSavedState` (54ECE0) then rebuilds the run in this order:

1. Reset (571F40); seed text, challenge, achievement and difficulty fields,
   the 33 game-state flags and the Game counters.
2. `History::RestoreGameState` (576BE0).
3. Seeds from the seed text (5E1470); Game RNG reseeded from GS+0xEF8C
   (5C2D40, shift 0x23).
4. ItemPool constructed from GS+0xED94 (583040), then
   `ItemPool::RestoreGameState` (581CF0).
5. PlayerManager constructed with the saved player type (GS+0x889C) and
   Seeds+0x44 (5BF4A0); its RNG reseeded from GS+0xEF88;
   `PlayerManager::RestoreGameState` (5BFAA0) creates the player and calls
   `Entity_Player::RestoreGameState` (hearts, consumables, items, RNGs).
6. `Level::RestoreGameState` (58C8F0): every descriptor, seed, grid and
   saved entity list comes back; the current room is entered through
   `Room::Init`, whose revisit branch restores grid and entity records and,
   unless the room is cleared, respawns the layout's other entities (5CE6B0).
7. `Entity_Player::RestoreGameState_PostLevelInit` (515F20): the whole cache
   is re-evaluated (stats, flight, familiars) and the player is placed 40
   units inside the entry door (or at the saved entry vector).
8. PlayerManager post-init (5BF220), Game RNG follow-up (54D750),
   `Ambush::RestoreGameState` (5B8380), nightmare scene (416730).
9. One `Level::Update` (584200) without advancing the game counters, then the
   fade-in (54EAA0).

Loading does not delete the record: a crash after Continue continues again
from the same record until the next floor or Exit replaces it.

### Anti-save-scum in the original

1. **The record is bound to the profile.** The trailer checksum is stored in
   `persistentgamedata` at every write and checked at every read. Restoring
   an older copy of `gamestate%d.dat` fails ("Checksum invalid!") because the
   profile holds the newer checksum.
2. **Losing or ending a run deletes the record immediately**: at GameOver
   init, at the run-end trigger and when exiting while dead. Exiting a dead
   run never saves.
3. **No re-rolls.** Item pools (weights, removed flags, pool RNGs), every
   per-item RNG, the Game and PlayerManager RNG words, every room descriptor
   seed and every saved entity seed are stored. Pedestals, chests, shop
   stock, room drops and boss rewards come out the same however often the
   run is quit and continued, and whether or not a room was entered before
   the quit.
4. **Fights restart instead of resuming.** NPCs are never saved; an
   uncleared room is re-entered from its layout with full-health enemies,
   so quitting mid-fight gives no information and no advantage beyond what
   leaving the room would.
5. **Save points are coarse.** Only floor start and Exit write the file. A
   killed process resumes at the start of the floor.

The shared Mersenne Twister is *not* saved. After Continue, champion rolls
and other shared-MT consumers draw from the process's current MT state, as
they do after any room change.

## Part 2: the port

### Invariants

1. **One record shape, original semantics.** A durable run record holds
   what `Game::SaveState` stores, organized by the same owners (Game,
   ItemPool, current Room, Level, Player, PlayerManager, History, Ambush,
   nightmare scene). Continue follows `StartFromSavedState`'s order. What
   the original derives on load, the port derives on load.
2. **Every port record equals an original Exit record.** Each write, at any
   save point, holds exactly what `Game::SaveState` would store if the
   player chose Exit on that frame. Port-only save points (below) never
   introduce a state the original could not produce.
3. **Saving does not perturb the live run.** `Game::SaveState` has side
   effects (empty-cell shared-MT draws, item-queue flush). At the original's
   two save points the port performs them live, as the original does. At
   port-only save points the port runs the same save on a clone and keeps
   only the record.
4. **Continue never refuses a real run.** Any record the port wrote must
   load. A load failure on a valid record is a bug in the owning system,
   fixed there; refusals are reserved for corrupt, foreign or tampered data.
5. **No Steam.** Byte storage is IndexedDB; cloud sync, Steam achievements
   and the Options.ini `SteamCloud` switch have no equivalent.

### Record model

The store holds two kinds of record per slot, mirroring the two files:

- **Profile record** (`persistentgamedata`): the existing `Profile` owner
  (achievements, counters, unlocked characters, selected character,
  collected history) plus persistent runtime state.
- **Run record** (`gamestate`): the `RunRecord` owner.

Both are wrapped in the same **envelope**:

```text
SaveEnvelope {
  magic: "ISAAC-RB-SAVE",
  schema: u32,            // record schema version, see Migrations
  kind: "profile" | "run",
  slot: 1..=3,
  generation: u64,        // strictly increasing per slot and kind
  engineBuild: string,    // diagnostic only, never used for acceptance
  checksum: u32,          // CRC-32 (IEEE) of the canonical payload bytes
  payload: <owner document>
}
```

The engine builds, serializes, checksums and validates envelopes; the host
never interprets payloads. Payloads stay JSON (bounded at 4 MiB per record)
so a stored record can be inspected and diffed in tests.

The run payload is organized by original owner. The table says, for each
part of the original record, where the port keeps it and whether Continue
reads it from the record or derives it.

| Original owner and fields | Port owner (today's name) | On load |
| --- | --- | --- |
| Game: seed text, challenge, difficulty, achievements byte, seeded flag | `SavedRun.run_seed/seeded_run`, `RunContext.challenge_id/difficulty/run_seed_value` | stored |
| Game: 33 game-state flags, Game counters, Game+0xCE9F4/F8, +0x10ECD4/DC | `level_progress` and the owners that read them; unported fields get a `game.flags`/`game.counters` raw block | stored |
| Game RNG (GS+0xEF8C), PlayerManager RNG (GS+0xEF88, GS+0x9688) | `RunContext.player_manager_rng`, `SavedGameSeeds`, `player_manager_secondary_rng` | stored, reseeded in source order |
| Seeds (from seed text) | `seed::RunSeeds` | derived from the seed text, then seed effects re-applied |
| ItemPool: weights, per-pool words, seed, RNGs, removed flags, pill table and identified pills | `items::ItemPools` (`copy_for_save`/`restore_after_init`) | stored |
| Level: stage, stage type, curses, flags, room grid, room count | `floor::Floor`, `stage`, `stage_seed`, `level_progress` | stored |
| Level descriptors: RoomConfig reference, seeds, neighbours, visited/cleared flags, persistent-spawn sets | `floor::Floor.descriptors`, `visited_rooms`, `room_display_flags`, `revealed_secret_connections` | stored |
| Per-room grid descriptors (448) | `grid_records` (current room), `persisted_rooms[i]` grids | stored; grid objects re-created by their factories |
| Per-room EntitySaveState lists | `Room.saved_regular_pickups`, slots, fireplaces, ambient records, **bombs** | stored; actors respawned from their own seeds |
| Current room, previous room, entry door, entry vector | `RunContext.level_entry`, `SavedRun.current_room` | stored |
| Current room layout, backdrop, FX rays, decorations | `Room` | derived from RoomConfig + descriptor seeds through Room.Init |
| Layout NPCs of an uncleared room, champion rolls | `room.npcs` | derived (revisit respawn, shared MT) |
| Player: type, hearts, mask, eternal, golden hearts, consumables, golden key, items, active + charges, trinkets, pockets, costume list, per-item RNGs, pill history, transformations | `Player`, `Inventory`, `player_save::SavedPlayerItems` | stored |
| Player stats, flight, tear flags, weapon | `player_stats`, `weapon_cache`, `weapon_admission` | derived by full cache evaluation (515F20) |
| Familiars | `familiars` | derived by the cache's familiar pass |
| Player position | `player.x/y` | derived: 40 units inside the entry door, or the saved entry vector |
| History (acquisition order) | `acquisition_history` | stored |
| Ambush, nightmare scene | `ambush` / `nightmares` owners | stored when the owner exists; absent owner → constructor default, as version-gated fields are |
| Tears, projectiles, effects, lasers, knives, explosions, animation clocks, camera, HUD, music position | live owners | not saved, as in the original; constructed fresh |
| Shared MT | `boot_lifecycle.shared` | not saved; continues from the current process state |

Two port-specific notes:

- The port keeps visited rooms as full `Room` values (`persisted_rooms`),
  where the original keeps descriptor records. On save the port writes the
  persisted subset: grid records and the 5C57C0 entity families. Live-only
  fields (NPC bodies, animation state) are stripped by `Room::SaveState`
  semantics before serialization, so a stored room is the original's room
  record in port types.
- Owners whose original fields are not yet identified are saved as raw
  blocks named by their offset (for example `game.counters[7]`). They round
  trip and are compared field by field against the original, so an owner can
  be named later without a schema break.

### Save points

| Trigger | Original | Port |
| --- | --- | --- |
| End of `Level::Init` (new run, every new floor) | write | write, live (side effects as the original) |
| Pause → Exit | write | write, live |
| Room transition complete (target room initialized) | in-memory `Room::SaveState` of the left room only | in-memory as the original, **plus a durable write from a clone** |
| Page hidden (`visibilitychange` → hidden, `pagehide`) | — (no equivalent; a killed process keeps the floor-start record) | durable write from a clone, if a run is live and not paused in a transition |
| GameOver init, run end, Exit while dead, Exit without save | delete | delete |
| New run started over an existing record | overwritten at `Level::Init` | same |

The two port-only triggers exist because a browser tab is normally closed,
not exited, and `pagehide` work is not guaranteed to finish. Each produces
the record an Exit on that frame would, so they never give a player a
state the original could not reach; they only change what a crash or a
closed tab costs (the room you were in instead of the whole floor). This is
the one deliberate deviation from the original's durability behaviour, and
it is visible only after a crash or a closed tab, never through Pause → Exit.

The clone write runs after the frame's simulation, at most once per room
transition, and is skipped while a write is still in flight (the next
trigger supersedes it). It never consumes the live shared MT and never
flushes the live item queue.

### Continue pipeline

The port's `continue_run` follows `StartFromSavedState`:

1. Validate the envelope (magic, kind, slot, schema, checksum, binding).
2. Migrate the payload to the current schema (pure, versioned functions).
3. Reset Game; restore Game scalars, flags and counters; History.
4. Seeds from the stored seed text; seed effects re-applied from the same
   text; Game RNG reseeded.
5. ItemPool constructed from its stored seed, then restored.
6. PlayerManager constructed for the stored player type; Player restored
   (hearts through the same helper sequence as 5116B0, consumables, items,
   pockets, trinkets, costumes, RNGs, transformations).
7. Level restored; current room entered through the revisit branch
   (`restore_continued_room`): grid factories, saved entity records
   (pickups, slots, fireplaces, shopkeepers, ambient effects, bombs),
   layout NPC respawn unless cleared.
8. Full cache evaluation (stats, flight, familiars); placement at the entry
   door or entry vector.
9. Ambush and nightmare owners; one Room update without counter advance;
   fade-in; system clock alignment.

Character, challenge, difficulty and seed-effect specifics enter only
through the owners that already handle them on a fresh run (character
definition, challenge parameters, `RunSeeds`), never through a Continue-only
branch. The two load callers that exist today (`InitialRoom`,
`VisitedRoom`) collapse into this one pipeline; the initial room is a
visited room whose entry door is unset.

### Storage

**Backend.** IndexedDB database `isaac-rebirth`, schema version 1, with two
object stores:

- `records`, keyed `[slot, kind, generation]`, value
  `{bytes: Uint8Array}` (the envelope, as exported by the engine);
- `heads`, keyed `slot`, value:

```text
SlotHead {
  slot,
  profile: {generation, checksum},
  run: {generation, checksum} | null,          // the bound run record
  runPrevious: {generation, checksum} | null,  // last good run, for recovery
}
```

The head is the analogue of the profile's bound checksum: a run record is
loadable only if the head names it.

**Atomic commit.** A save is one `readwrite` transaction over both stores:
put the new profile/run records, put the new head (new run becomes `run`,
old `run` becomes `runPrevious`), delete records the new head no longer
names. IndexedDB commits the transaction atomically or not at all, so a
crash leaves either the old head with its records or the new head with its
records. The host reports success only after `oncomplete`, then the engine's
pending write is acknowledged; a failed or aborted transaction is
acknowledged as failed and the older head stays in force.

**Integrity and fallback.** On load the host reads the head and the
records it names and passes them to the engine. The engine checks each
envelope's magic, slot, kind, schema range and checksum, and that the
checksum equals the head's. If `run` fails, the engine tries `runPrevious`
and, if that passes, Continue offers it and the menu reports the recovery.
If both fail, Continue is unavailable for the slot; the records are kept
(quarantined, not deleted) and the menu reports the corrupt save. Profile
records follow the same rule with no previous generation: a profile that
fails validation is never replaced by a fresh one silently; the existing
"preserve rejected own-profile bytes" behaviour stays.

**Single writer.** The game takes an exclusive Web Lock
(`navigator.locks.request('isaac-rebirth-save', …)`) for the tab's lifetime.
A second tab that cannot get it runs with saving disabled and Continue
greyed out, so two tabs never interleave generations.

**Durability.** At first successful save the host calls
`navigator.storage.persist()` so the origin's data is exempt from
best-effort eviction where the browser grants it. The result is logged, not
required.

**localStorage.** Kept for two reasons only:

1. **Migration source.** The existing document under
   `isaac-rebirth.savegames.v1` (schema 1 or 2) is read once when IndexedDB
   has no heads, migrated, committed to IndexedDB, read back and verified,
   and only then removed.
2. **Fallback backend** when `indexedDB.open` fails (storage disabled,
   some embedded or privacy modes). It stores the same envelopes and heads
   under `isaac-rebirth.v3.record.<slot>.<kind>.<generation>` and
   `isaac-rebirth.v3.head.<slot>`. Records are written first and the head
   last; each `setItem` is atomic, so the head swap is the commit point, and
   unreferenced records are removed after it.

### Migrations

Each payload schema has a frozen DTO and a pure migration to the next:

| Schema | Shape | Migration |
| --- | --- | --- |
| 1 | own V1 `Profiles` document (no collected history) | → 2: existing `LegacyProfiles` conversion |
| 2 | own V2 `Profiles` document with embedded `RunRecord` | → 3: split into per-slot profile and run envelopes; compute checksums; generation 1 |
| 3 | envelopes, owner-organized run payload | current |

Rules, as in the original's version window:

- Load accepts every schema from 1 to the current one; a newer schema is
  refused without touching its bytes ("saved by a newer build").
- A migration only adds fields with their constructor defaults or
  reorganizes existing ones. It never fills an owner from current game state
  and never turns an unknown into a zero; an owner that a legacy record
  genuinely lacks becomes the same constructor default the original uses
  for a version-gated field, and the field's provenance is kept.
- Migrated records are re-written as the current schema only by the next
  real save; loading alone never rewrites storage.
- Every change to a payload DTO bumps the schema and adds a migration in the
  same PR.

### Anti-save-scum in the port

| Original rule | Port rule |
| --- | --- |
| Run record bound to the profile checksum | Run record loadable only if the slot head names its generation and checksum; heads change only in the same transaction as the records they name |
| Losing or ending a run deletes the record | GameOver init, run end, Exit while dead and Exit without save clear `run` **and** `runPrevious` and delete their records in one transaction, so recovery can never resurrect a lost run |
| Pools, item RNGs, Game/PlayerManager RNGs, descriptor seeds and entity seeds saved | Same fields saved; Continue reseeds every stream from the record; nothing is redrawn from a fresh seed |
| NPCs not saved; fights restart | Same: layout NPCs respawn at full health unless the room is cleared |
| Coarse save points | Port-only saves are Exit-equivalent, so they add no new reachable state; a closed tab costs the current room instead of the floor |
| Load does not delete | Same; the head stays until the next save or delete |

Recovery to `runPrevious` happens only when the bound record fails its
integrity check, never by player choice, and the previous record is itself a
legitimate Exit-equivalent of the same run.

### Steam replacement

| Original | Port |
| --- | --- |
| `SteamCloudFile`, `IsSteamCloudAvailable`, Options.ini `SteamCloud` | none; the IndexedDB backend is the only store |
| `GameState::SaveToSteamCloud` / `LoadFromSteamCloud` | `SaveGameIO` IndexedDB transaction / head read |
| `PersistentGameData::SaveToSteamCloud` / `LoadFromSteamCloud` | profile envelope in the same transaction |
| `PersistentGameData::SetGameStateChecksum` | the slot head's `run` reference |
| `ImportSteamAchievements`, `unlock_steam_achievement` | none; achievements live only in the profile record |
| Local `%sgamestate%d.dat` / `%spersistentgamedata%d.dat` | localStorage fallback backend |

### Failure handling

- A write failure (quota, aborted transaction) keeps the previous head; the
  engine resumes the tick and reports the failure; the next save point
  retries with a new generation.
- A quota failure first deletes unreferenced records, then retries once.
- Validation failures name the failing envelope field (magic, schema,
  checksum, binding, owner validation) in the host log and the menu.
- Development builds expose the slot heads and envelope headers in the
  existing save snapshot so e2e tools can assert on them.

## Parity test plan

Continue is graded in four layers. Each case is a public input tape run on
the original and on the port with the same seed, recorded before either game
runs, as the existing suites do.

### A. Record parity (field by field)

The original's `gamestate%d.dat` is captured after Exit by the existing
observer (`tools/original/save-continue-observe.py`), and decoded by a new
reader that follows `GameState::write` (header, checksum, every field in the
table above). The port exports a *native projection* of its run record: the
same fields in the same units (stage, descriptors, seeds, grid descriptors,
entity save states, player stores, item pools, RNG words). The comparator
walks both field by field. Expected differences are not tolerated or
filtered: a field the port does not yet own is reported as a failing row,
and owners whose original meaning is unknown compare raw.

### B. Post-Continue state (frame by frame)

After Main → Continue in both games, the existing paired observers
(`paired-room-observe.py`, `tools/foundation/continue-run-state-pair.mjs`)
compare per frame: room, previous room, player position/velocity/controls,
hearts, consumables, owned items, active charge, pockets, trinkets, visited
set, clear flags, pickups (multiset with seeds), slots, fireplaces, bombs,
familiars, layout NPCs (type, variant, subtype, position, HP at the first
capture; identities afterwards, once shared-MT AI diverges). Shared-MT
consumers are graded against the original's captured MT state, as today.

### C. Anti-save-scum behaviour

- Quit → Continue → Quit → Continue gives identical records (A) and states
  (B) at each Continue, except for the frame counters that advanced.
- A pedestal, chest, shop item and room drop is the same in "quit before
  entering" and "quit after entering" routes (extends 3YSTBQPG to chests,
  shops and later floors).
- Death and win delete the save; Continue is unavailable afterwards in both
  games.
- An uncleared room restarts with full-health layout NPCs.

### D. Storage robustness (port only)

Browser checks with no original counterpart: interrupted commit (tab
crash between record and head), corrupted current record (falls back to
`runPrevious` and says so), stale record (an older envelope re-inserted
under the current key is refused: binding), newer schema (refused,
untouched), legacy localStorage document (migrated once, verified, removed),
IndexedDB unavailable (localStorage backend), second tab (saving disabled),
reload after the room-transition and `pagehide` saves.

### Case matrix

| Case | Layers | Suite |
| --- | --- | --- |
| Initial room, moving before Exit (fixed + OS) | A B | save-continue |
| Visited room, uncleared fight (KRAYFRWA) | A B C | continue-run |
| Treasure room before/after entry (3YSTBQPG) | A B C | continue-run |
| OS neighbour room, random idle | A B | continue-run |
| Floor 2 and floor 3 start (Level::Init save) | A B | new `continue-floors` cases |
| Mid-floor Exit on floor 2+ after clearing rooms | A B C | continue-floors |
| Mid-fight Exit with damaged enemies and a boss room | B C | continue-run |
| Live bomb on the floor at Exit | A B | continue-run |
| Collectibles, active item with partial charge, trinket, pill and card held | A B | continue-items |
| Familiars (e.g. Brother Bobby, Sister Maggy) | B | continue-items |
| Shop with stock and a bought item; opened chest | A B C | continue-run |
| Hard difficulty, seeded run, challenge run | A B | continue-run |
| Other characters (as they are ported) | A B | continue-run |
| Death, win, Exit while dead | C | continue-run |
| Storage robustness | D | save-continue (Browser) |

Every PR runs the touched suites on its branch and on `origin/main` with the
same recorded cases and compares per case; no case that passes on main may
fail on the branch.

## Implementation plan

Incremental PRs, each self-contained and merged after the comparison above:

1. **This design.**
2. **Storage.** Envelope, CRC-32, schema 3 and the 2→3 migration in the
   engine; IndexedDB backend with heads, atomic commit, fallback, Web Lock,
   persistence request; localStorage fallback and one-time migration;
   suites updated to read IndexedDB heads.
3. **One Continue pipeline.** Fold `InitialRoom` into the visited-room
   path; remove the Isaac/challenge0/seed-effect0 and legacy-owner
   refusals that a real record can trip; restore live bombs from the saved
   records; make every refusal on a valid record a bug.
4. **Later floors.** Floor-start and mid-floor Continue on every ported
   stage; `continue-floors` cases.
5. **Items and familiars.** Active charge, pockets, trinkets, costumes,
   transformations, familiars after Continue; `continue-items` cases.
6. **Record parity reader.** The `gamestate.dat` decoder and native
   projection (layer A) for all suites.
7. **Port-only save points.** Room-transition and `pagehide` clone writes.
8. **Remaining owners.** Game counters/flags, Ambush, nightmare scene,
   shopkeepers, challenge and difficulty specifics, as layer A exposes them.

## Coverage tracker

Status: **done** (merged, suites compared), **wip** (branch open),
**todo**. Update this table in every save-system PR.

| # | Area | Status | PR | Notes |
| --- | --- | --- | --- | --- |
| 1 | Design (this document) | done | #990 | |
| 2 | Envelope, checksum, schema 3, 2→3 migration | todo | | |
| 3 | IndexedDB heads, atomic commit, fallback, lock, persist | todo | | |
| 4 | localStorage fallback backend and legacy migration | todo | | |
| 5 | Single Continue pipeline (initial room folded in) | todo | | today: `InitialRoom` + `VisitedRoom` callers |
| 6 | Refusals removed: challenge, seed effects, legacy owners | done | #1006 | `admit_continue` refuses only dead runs, non-Isaac, bad manager RNG, bad slot/records, unvisited room; unknown costume history and stale scheduled Effect ids no longer refuse |
| 7 | Live bombs restored from Room.Save records | done | #1006 | `RunContext.bomb_records` (variant, subtype, position, spawn seed); respawned with Appear/Pulse like 5C5860; fuse restarts (actor RNG +0x7c0 not yet stored) |
| 8 | Floor 2+ Continue (floor start and mid-floor) | wip | #1006, this PR | floor start: `stage-continue` suite (Monstro and Larry Jr. exits into Basement II, Exit and Continue in its start room, graded against the original: stage, room, entry position, controls; 2908 checks each, pass). Mid-floor and Caves still engine fuzz only (#1006) |
| 9 | Items: active charge, pockets, trinkets, costumes | todo | | |
| 10 | Familiars after Continue | wip | | engine fuzz after familiars core (#1002): 465 rounds, 110 with familiars, same variant/subtype set after Continue; positions not compared |
| 11 | Hard difficulty, seeded runs | wip | | Hard covered by the engine fuzz; seeded runs untested |
| 12 | Challenge runs | todo | | |
| 13 | Other characters | todo | | blocked on character runtime (fresh runs are Isaac-only) |
| 14 | Delete on death, win, Exit while dead (both generations) | todo | | today: profile slot cleared |
| 15 | Record parity reader (layer A) | todo | | |
| 16 | Room-transition and pagehide clone saves | todo | | |
| 17 | Game counters/flags, Ambush, nightmare owners | todo | | |
| 18 | Storage robustness checks (layer D) | todo | | |

## References

| Function | Linux | mac symbol |
| --- | --- | --- |
| Game::SaveState | 54F230 | `Game::SaveState(GameState&)` |
| Game::StartFromSavedState | 54ECE0 | `Game::StartFromSavedState(GameState const&)` |
| Game::Exit | 54F3F0 | — |
| Manager::SaveGameState / DeleteGameState | 594810 / 594910 | `Manager::SaveGameState()` / `DeleteGameState()` |
| Manager::SelectSaveSlot (load both files) | 594DC0 | — |
| Manager::IsSteamCloudAvailable | 5943F0 | `Manager::IsSteamCloudAvailable()` |
| GameState::Clear / Delete | 55BD80 / 55CE40 | `GameState::Clear()` / `Delete()` |
| GameState::write / read | 55A240 / 55DB90 | `GameState::write()` / `read()` |
| GameState::write_Room / write_EntitySaveState | 559BF0 / 559950 | same names |
| GameState::Save / Load | 55B7B0 / 55F430 | same names |
| GameState::SaveToSteamCloud / LoadFromSteamCloud | 55B890 / 55F510 | same names |
| Checksum Reset / Add(u32) / Add(float) / Add(bytes) | 42F810 / 42F820 / 42F830 / 42F850 | `Checksum::*` |
| PersistentGameData::SetGameStateChecksum | 5BF010 | same name |
| ItemPool Store / Restore | 581ED0 / 581CF0 | `ItemPool::StoreGameState` / `RestoreGameState` |
| Room::SaveState, persisted families, record respawn | 5CE2D0, 5C57C0, 5C5860 | `Room::SaveState()` |
| Level Store / Restore / Init | 58CD90 / 58C8F0 / 58BE30 | `Level::StoreGameState` / `RestoreGameState` |
| PlayerManager Store / Restore | 5BF400 / 5BFAA0 | same names |
| Entity_Player Store / PostLevelInit restore | 518350 / 515F20 | same names |
| History Store / Restore | 576BD0 / 576BE0 | same names |
| Ambush Store / Restore | 5B83B0 / 5B8380 | same names |
