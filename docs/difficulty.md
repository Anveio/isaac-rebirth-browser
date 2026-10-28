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
| `boss_heart` | Boss death 5c8430: when the last boss of a boss (5) or Boss Rush (0x11) room dies, 5c87c3 spawns its heart (and a second without achievement 33) | drops | none | `boss_reward::trigger` |
| `champion_chance_bonus` | NPC Init 47f5c6/47f725: added to the champion chance (0.05, 0.2 with Champion Belt) | 0.0 | f32 0.15 (66ad1c) | `champions::roll` |
| `champion_drop_needs_zero_roll` | Champion death drop 4f27a0 (4f2d10 at removal): the NPC's own RNG draws RandomInt(3) at both difficulties (4f27c8); 4f27d8 then gates the color's drop | always drops | drops only on 0 | `champions::death_drops` |
| `clear_heart_max_roll` | Ordinary room-clear award 5c9240: a heart award draws RandomInt(100) on the award RNG (5c9e1c, 5c9f93) and spawns only at or below the limit | no draw | at most 34 | `clear_award::spawn` |
| `curse_odds` | Level::Init 58bf90..58bfd8: RandomInt(n) == 0 curses the floor; n for a fresh profile / achievement 4 / achievement 33 / counter 10 | 80 / 30 / 10 / 5 | 10 / 5 / 3 / 2 | `challenges::sample_curse`, run by `floor::generate_normal_basement` on every floor |
| `extra_rooms` | Level::Init 58c08e: rooms added to the floor's count, plus the curse stream's RandomInt(2) (58c089) | none | 2 | `floor::generate_normal_basement` |
| `odd_stage_room_difficulty` | Level::Init 58c114/58c124: normal-room difficulty on odd stages | 1..5 | 5..10 | `floor::difficulty_band` (even stages are 5..10 at both) |
| `hud_icon` | HUD 5742bd: hudpickups.anm2 frame under the counters | none | frame 4 | `web/renderer.ts` HUD, via `State::difficulty_presentation` |
| `pause_blood` | Pause render 5ba8d2: pausescreen.anm2 layer 4 (Blood) | not drawn | drawn | `web/presentation.ts` pause |

Every floor rolls its curse (58bfe8) on a copy of the primary stream, so the
roll consumes nothing the generator reads; hard only raises the odds. The
floor keeps the curse in `Floor::curses` (Level+8). Curse of the Labyrinth
multiplies the room count by 1.8 (at most 45, 58c061) and Curse of the Lost
adds four rooms (58c4c0), both before hard's extra rooms. These are source
generation rules; the [live qualification](levels.md#curse-and-xl-qualification)
covers selected Normal and Hard first floors. The curse's other consumers
are listed under Gaps.

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
| 5c87c3 | boss death 5c8430 | boss heart drop | row `boss_heart` |
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
- **Boss death callback** (5c8430, `boss_reward`): the row `boss_heart`
  covers the two hearts of a boss or Boss Rush room. The port runs the
  callback when the room's last boss dies: the sins' collectible or pickups
  (tables 67a7a0 and 67a768) and the ordinary boss's hearts, with Daemon's
  Tail and achievement 33. It refuses with an error for Krampus (81.1),
  Ultra Pride (46.2, 5c89e8), a boss champion (subtype != 0) and an angel
  room (type 15, Uriel and Gabriel), and it skips the drop when a trinket
  owner is unknown. It does not write the kill records 5bed50 and 5bec80
  (achievements 86 and 22) or the stats calls 5b2c20 and 5b3190. Larry's AI
  492df0 draws Larry's own RNG at 49a8d6 (a 1000.15 effect, then a soul heart
  when bounded(4) is 0) on a path the port does not run, so a Larry fight
  that reaches it moves the drop seeds.
- **Ordinary room-clear award** (5c9240, `clear_award`): the port spawns
  the award from the award RNG, Luck, Lucky Foot, Lucky Toe, the trinkets
  22/34/36/41/44/45, Guppy's Tail and Contract from Below. It spawns nothing
  when one of those owners is unknown, in room type 0x11 (which takes the boss
  branch 5c9740), or for a trinket (5.350) or red chest (5.360) award, which
  have no factory yet.
- **Curse effects** (Level+8, rolled at both difficulties with the
  `curse_odds` row): the port draws the curse names (5852f0) under the floor
  name on the streak's paper layer in black TeamMeatFont10 (ShowItemText
  5733ee), hides the hearts under Curse of the Unknown (HUD 5742f0), and runs
  Curse of the Labyrinth's generation (the XL name, a second boss 589d5c and
  treasure room 58bae3, one stage more through the trapdoor 585640) and Curse
  of the Lost's four rooms. Curse of the Maze runs in Level::ChangeRoom
  (`maze.rs`): after Room.Save, RandomInt(10) on the shared MT takes the
  teleport (58dc90: shared-MT shuffle of the rooms within two doors, minimap
  shake and SFX 22) on a zero, then RandomInt(5) takes the room swap (58dbf0:
  glibc `rand()` shuffles, `libc_rand.rs`, exchanging two visited rooms of one
  type, shape and door mask while the doors stay in place). It does not
  implement Curse of Darkness's lighting, Curse of the Lost's map hiding,
  Curse of the Blind's hidden items or Curse of the Cursed's doors, nor
  586594's curse handling past the roll. The Maze draws and its first room
  change are graded by EK3SH68W below; no live case yet takes the teleport
  or the swap.
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

The curse roll is graded as part of the floor: `foundation-floor-verify.py`
(the `floor` suite's capture) compares the original's Level+8 with the
port's `Floor::curses`, and the `floor` suite also checks the live browser's.
`foundation-floor-verify.py --capture --hard` starts each original at hard. Hard
WPTB6SQK (Curse of the Unknown, 8), EK3SH68W (Curse of the Maze, 32),
0JLB1WMP (Curse of the Labyrinth, 2) and two OS-random floors without a curse
match the original's descriptors, grid, attempts and curse mask.

The `champion-drop` and `champion-drop-hard` entity specs fight champion
Horfs to death (an entity spec with `hard: true` starts both games at hard)
and grade every live pickup's identity and f32 position at each capture.
With a capture on every fight update, the normal key champion (EHL7N0AV) and
the hard pill champion (7VFTKQCK, own roll 0) drop at the same update with
the same variant, subtype, seed and position in both games. The hard red
champion in 4J3JGA0V (own roll 2) dies in both games without a heart.

The same captures grade the ordinary room-clear award. The normal Horf rooms
EHL7N0AV (a chest) and 31T4YVZ3 (a bomb) spawn the award at the room centre
(320, 280) with the original's variant, subtype and seed. The
`clear-award-hard` spec fights single-Dip rooms at hard: HNXKNQ2T (roll 14)
and G6MD4K0A (roll 2) spawn the heart at the same seed and position in both
games, and WPTB6SQK (roll 58) clears with no award in either game. The
Dip room at slot 2 of 0JLB1WMP's hard Labyrinth floor matches in every
graded field as well, and so does the Dip room at slot 0 of EK3SH68W's hard
Maze floor, whose first room change takes the two Maze draws. The curse
line's paper scroll is compared at gameplay-start: WPTB6SQK's full-frame
diagnostic drops from 16,212 changed pixels to 12, 0JLB1WMP's from 16,344
to 72.

The `stage-exit` suite's boss fixtures carry the boss callback's drops as a
diagnostic (each game fights with its own keys, so which drops are left
differs). In E1T36YXW (Monstro, normal) both hearts the original drops
(5.10.3 seed 511906572 and 5.10.1 seed 1755029133) match the port's in
variant, subtype and seed. The Larry Jr. fixtures still differ (see the gap
above). No capture grades the hard row yet: at hard the
callback draws nothing for an ordinary boss, so the row only skips the
spawn. `boss-scenarios.py --hard` (an entity spec with `boss: true` and
`hard: true`) routes into hard boss rooms, and the original validates the
routes, but no open-loop fight tried kills the boss in either game: Monstro
in TNPNSRPX (whose route crosses red fireplaces, 1410, which the port does
not admit, so the shared RNG drifts before the fight) and Gurglings in
V9X18LNY (sweep and churn) and 41FNCL0Y (churn). Grading the row needs a
fight the original wins on its own keys, as the `stage-exit` fixtures'
`originalFight` does at normal.
