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
| `hud_icon` | HUD 5742bd: hudpickups.anm2 frame under the counters | none | frame 4 | `web/renderer.ts` HUD, via `State::difficulty_presentation` |
| `pause_blood` | Pause render 5ba8d2: pausescreen.anm2 layer 4 (Blood) | not drawn | drawn | `web/presentation.ts` pause |

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
| 47f5c6, 47f725 | NPC Init 47ec60 | champion chance | gap |
| 58bf7d | Level::Init 58be30 | curse odds, room count, room difficulty | gap |
| 4f27d8 | champion death drop 4f27a0 | drop odds | gap |
| 5399ed | Slot 539540 | beggar payout threshold | gap |
| 53c9fa, 53da69, 53ff04 | Slot 53c290 | slot prize odds and counts | gap |
| 5c87c3 | boss death 5c8430 | boss heart drop | gap |
| 5c9e1c | SpawnClearAward 5c9240, ordinary rooms | heart award odds | gap |
| 5c9ca9, 5ca5a2, 5cb09c, 5cb222, 5cb724, 5cb7c2, 5cb84d | SpawnClearAward 5c9240, Boss Rush and final bosses | hard completion marks | gap |

## Gaps

Each is a difficulty read whose consumer the port does not implement (or
implements without the rule yet). The values are read from the disassembly.

- **Champion chance** (NPC Init 47f5c6/47f725): the chance (0.05, or 0.2 with
  Champion Belt) gains f32 0.15 (66ad1c) at hard, before Purple Heart's
  doubling.
- **Level::Init** (58bf7d sets a stack flag read at 58bf90..58bfd8, 58c08e,
  58c114 and 58c124):
  - curse odds: RandomInt(n) == 0 curses the floor, n = 80 / 30 / 10 / 5 at
    normal and 10 / 5 / 3 / 2 at hard (base, achievement 4, achievement 33,
    counter 10);
  - room count: hard adds 2 + RandomInt(2) of the curse stream (58c089);
  - room difficulty: odd stages draw normal rooms from difficulty 1..5 at
    normal and 5..10 at hard (even stages 5..10 at both).
- **Champion death drop** (4f27d8): a champion's color drop (jump table
  66e320) needs RandomInt(3) == 0 at hard; normal always drops. The port has
  no champion drops.
- **Beggar payout** (5399ed): the payout threshold R2+R4+R4 is raised to at
  least 5 at hard. The port admits no slot payouts.
- **Slot prizes** (Slot update 53c290): 53c9fa voids a prize when
  RandomInt(2) == 0 at hard; 53da69 takes chance f32 0.85 at hard and 0.65 at
  normal (times 0.46 with Lucky Foot); 53ff04 draws a payout count from
  RandomInt(2) at hard and RandomInt(3) at normal (at least 1, +1 with PHD).
  The port admits no slot payouts.
- **Boss heart** (boss death 5c8430, 5c87c3): Boss and Boss Rush rooms drop a
  heart only at normal. The port's boss callback is pending.
- **Room-clear heart award** (5c9240, 5c9e1c then 5c9f93): at hard an
  ordinary room's heart award survives only when RandomInt(100) <= 34. The
  port has no ordinary room-clear awards.
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
