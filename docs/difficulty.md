# Difficulty framework

The Game difficulty is run state, and every rule the original keys by it is
one row of a table. `engine/src/difficulty.rs` holds both:

- **`Difficulty`** is Game+10ECD8: `Normal` (0) or `Hard` (1). The character
  menu's Up/Down toggles `State::menu_difficulty`; a run started from that
  menu copies it into `State::difficulty` (59f63c passes it as the System
  request's r8, which Game::Start 54eb20 stores at 54eb4e). Challenges pass 0
  (59d218). The run keeps it through the save (54f294 writes it, the saved
  loader restores it at 54ed4d), Continue, the death screen's Restart and the
  held-R living Restart (594a10 restarts with the System request's stashed
  r8). Saves and the state snapshot carry it as `"normal"`/`"hard"`; the
  living-Restart context and Challenge run parameters keep it as the source
  integer `sourceR8`.
- **`Rules`** is the table. Each row is one line of the `rules!` macro,
  `field: Type = [normal, hard]`, with a doc comment naming the original's
  address(es). A consumer reads `difficulty::rules(d).field`; it never
  branches on the difficulty itself.

Every gameplay read compares Game+10ECD8 with 1. A read whose consumer the
port implements is a row; a read whose consumer the port does not implement
yet has no row and is listed under [Gaps](#gaps) with its address, the way a
`None` chapter field is a port gap in [levels](levels.md). A gap is not a
claim that the difficulty changes nothing there.

## Rows

| Row | Original | Normal | Hard | Consumer |
| --- | --- | --- | --- | --- |
| `champion_chance_bonus` | NPC Init 47f5c6/47f725: added to the champion chance (0.05, 0.2 with Champion Belt) | 0.0 | f32 0.15 (66ad1c) | `champions::roll` |
| `champion_drop_needs_zero_roll` | Champion death drop 4f27a0 (4f2d10 at removal): the NPC's own RNG draws RandomInt(3) at both difficulties (4f27c8); 4f27d8 then gates the color's drop | always drops | drops only on 0 | `champions::death_drops` |
| `clear_heart_max_roll` | Ordinary room-clear award 5c9240: a heart award draws RandomInt(100) on the award RNG (5c9e1c, 5c9f93) and spawns only at or below the limit | no draw | at most 34 | `clear_award::spawn` |
| `curse_odds` | Level::Init 58bf90..58bfd8: RandomInt(n) == 0 curses the floor; n for a fresh profile / achievement 4 / achievement 33 / counter 10 | 80 / 30 / 10 / 5 | 10 / 5 / 3 / 2 | `challenges::sample_curse` (Challenge floors, hard's extra rooms) |
| `extra_rooms` | Level::Init 58c08e: rooms added to the floor's count, plus the curse stream's RandomInt(2) (58c089) | none | 2 | `floor::generate_normal_basement` |
| `odd_stage_room_difficulty` | Level::Init 58c114/58c124: normal-room difficulty on odd stages | 1..5 | 5..10 | `floor::difficulty_band` (even stages are 5..10 at both) |
| `hud_icon` | HUD 5742bd: hudpickups.anm2 frame under the counters | none | frame 4 | `web/renderer.ts` HUD, via `State::difficulty_presentation` |
| `pause_blood` | Pause render 5ba8d2: pausescreen.anm2 layer 4 (Blood) | not drawn | drawn | `web/presentation.ts` pause |

A curse the hard stream selects has no consumer, as on every ordinary floor
at either difficulty: hard only raises its odds. Curse of the Labyrinth
(58c061 multiplies the count by 1.8 before the extra rooms) stays a gap.

The two presentation rows reach the renderer through
`State::difficulty_presentation`, rebuilt from the run's difficulty on every
step. With both the hard and the seeded icon, 5742bd draws frame 4 at
(20, 84) and frame 5 at (36, 84); either alone sits at (24, 84).

## Adding a row

1. Find the read: every one is `cmpl $0x1,0x10ecd8(reg)` (or a `mov` of it)
   off the Game pointer at 0x8ecd30. The [enumeration](#every-read) below
   lists all 24 instructions in the pinned binary.
2. Add one line to `rules!`: `field: Type = [normal, hard]`, with a doc
   comment naming the address and what it decides. Values come from the
   disassembly (constants, branch arms), never from the wiki or a guess.
3. Read it at the consumer as `difficulty::rules(state.difficulty).field`.
   A normal row value must reproduce the consumer's existing behavior
   exactly, so normal runs stay byte-identical.
4. Move the read from [Gaps](#gaps) to [Rows](#rows), run `check:build` and
   the consumer's suites, and record the evidence. Removing or changing a row
   is the same one-line edit.

## Every read

`objdump -d` of the pinned `isaac.x64` finds 24 instructions that touch
Game+10ECD8 (the Game pointer is the global at 0x8ecd30).

| Address | Function | What it decides | Port |
| --- | --- | --- | --- |
| 54eb4e | Game::Start 54eb20 | stores r8 | run start |
| 54ed4d | saved loader 54ece0 | restores the saved value | Continue |
| 54f01e | Game::Start variant 54efe0 | stores 0 | not reached (no caller the port admits) |
| 54f294 | save writer 54f230 | writes the value | save |
| 594bb8 | System dispatch 594a10 | stashes a loaded run's value for Restart | living Restart context |
| 5742bd | HUD 573a60 | difficulty icon | row `hud_icon` |
| 5ba8d2 | pause render 5ba830 | Blood layer | row `pause_blood` |
| 47f5c6, 47f725 | NPC Init 47ec60 | champion chance | row `champion_chance_bonus` |
| 58bf7d | Level::Init 58be30 | curse odds, room count, room difficulty | rows `curse_odds`, `extra_rooms`, `odd_stage_room_difficulty` |
| 4f27d8 | champion death drop 4f27a0 | drop odds | row `champion_drop_needs_zero_roll` |
| 5399ed | Slot 539540 | beggar payout threshold | gap |
| 53c9fa, 53da69, 53ff04 | Slot 53c290 | slot prize odds and counts | gap |
| 5c87c3 | boss death 5c8430 | boss heart drop | gap |
| 5c9e1c | SpawnClearAward 5c9240, ordinary rooms | heart award odds | row `clear_heart_max_roll` |
| 5c9ca9, 5ca5a2, 5cb09c, 5cb222, 5cb724, 5cb7c2, 5cb84d | SpawnClearAward 5c9240, Boss Rush and final bosses | hard completion marks | gap |

## Gaps

Each is a difficulty read whose consumer the port does not implement (or
implements without the rule yet). The values are read from the disassembly.

- **Champion death drop colors** (4f27a0's jump table 66e320): the row
  covers every color, but the port constructs the drops of colors 0 (red
  heart 5.10.1), 1 (battery 5.90.0), 2 (pill 5.70.0), 3 (1..3 coins, the
  count from the shared MT before the gate), 6 (eternal heart 5.10.4), 7
  (key 5.30.0), 10 (none) and 12 (double heart 5.10.5) only. Colors 4 (three
  Blue Flies through the player, 508740), 5 (an explosion 554490 with its
  effects before the gate, then a bomb 5.40.0), 8 (locked chest 5.60.0), 9
  (red chest 5.360.0), 11 (trinket 5.350.0) and 13 (4f2852) still drop
  nothing, as do champions of families whose death bypasses the plain base
  kill (4fb885 and 4fcb3b also reach 4f2d10).
- **Beggar payout** (5399ed): the payout threshold R2+R4+R4 is raised to at
  least 5 at hard. The port admits no slot payouts.
- **Slot prizes** (Slot update 53c290): 53c9fa voids a prize when
  RandomInt(2) == 0 at hard; 53da69 takes chance f32 0.85 at hard and 0.65 at
  normal (times 0.46 with Lucky Foot); 53ff04 draws a payout count from
  RandomInt(2) at hard and RandomInt(3) at normal (at least 1, +1 with PHD).
  The port admits no slot payouts.
- **Boss heart** (boss death 5c8430, 5c87c3): Boss and Boss Rush rooms drop a
  heart only at normal. The port's boss callback is pending.
- **Ordinary room-clear award** (5c9240, `clear_award`): the port spawns
  the award from the award RNG, Luck, Lucky Foot, Lucky Toe, the trinkets
  22/34/36/41/44/45, Guppy's Tail and Contract from Below. It spawns nothing
  when one of those owners is unknown, in room type 0x11 (which takes the boss
  branch 5c9740), or for a trinket (5.350) or red chest (5.360) award, which
  have no factory yet.
- **Completion marks** (5c9240's Boss Rush and stage 6/8+ branches: 5c9ca9,
  5ca5a2, 5cb09c, 5cb222, 5cb724, 5cb7c2, 5cb84d): a per-character
  Persistent counter at 1 is raised to 2 at hard; 5ca5a2 also unlocks a
  per-character achievement (167..177) at hard. The port has no completion
  marks: Mom's stage-6 clear award refuses with an error at hard.

## Verification

`paired-hard-run` (tools/original/foundation-paired-scenarios.py, part of the
`paired` suite) presses Down on the seeded Character page before Enter, so
both games start the same seed at hard, then moves, fires and pauses. Record
it side by side with:

```
python3 tools/original/foundation-paired-scenarios.py --scenario paired-hard-run --seed T1PK6Y0W --output DIR/case
node tools/playtest/side-by-side.mjs --case DIR/case/case.json --out DIR
```

The `champion-drop` and `champion-drop-hard` entity specs fight champion
Horfs to death (an entity spec with `hard: true` starts both games at hard)
and grade every live pickup's identity and f32 position at each capture.
With a capture on every fight update, the normal key champion (EHL7N0AV) and
the hard pill champion (7VFTKQCK, own roll 0) drop at the same update with
the same variant, subtype, seed and position in both games. The hard red
champion in 4J3JGA0V (own roll 2) dies in both games without a heart.
