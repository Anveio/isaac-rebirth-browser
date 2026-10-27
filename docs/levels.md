# Level framework

A floor is data. `engine/src/levels.rs` holds two tables, and every
stage-dependent consumer reads them instead of matching on stage numbers:

- **Stage nodes** (`STAGES`, keyed by Level stage) are the progression state
  machine: the ShowName streak (and its " XL" form under Curse of the
  Labyrinth, which only odd stages up to 7 roll), the trapdoor edge
  (SetNextStage585620's target), the generated floor's contract and, where
  584240 treats that stage alone differently, its own boss chain (Depths II
  runs only Mom's).
  A stage without a node has no generator, so no run start or trapdoor
  reaches it. A trapdoor edge is admitted only into another node, so an
  edge may name a stage that has no node yet.
- **Chapters** (`CHAPTERS`, keyed by stages.xml id) hold the parameters and
  assets the original keys by that id: the authored STB room set, the stage
  backdrop (backdrops.xml), the fxlayers.xml 1x1 overlay group, the music
  track, the 584240 boss draw chain, and the admissions that grid ambience and
  the bloody boss backdrop need. A Level stage resolves to its chapter through
  5d49b0 (`chapter_id`: odd stages keep their number, even stages drop by one,
  the alternate stage type adds one), which also selects the room set and
  backdrop, so one chapter row serves both floors of a chapter.

A `None` chapter field is a port gap, not a claim about the original: its
consumer keeps refusing, or keeps its existing Basement fallback, until the
field is imported with evidence.

## Consumers

| Consumer | Reads |
| --- | --- |
| `floor::generate_normal_basement` | node (admission, contract, boss chain override), chapter room set and boss chain |
| `room_assets::for_descriptor_with_special` | chapter room set by the descriptor's `asset_stage` |
| `room::backdrop`, overlay setup | chapter backdrop and overlay |
| `room` boss backdrop and blood scatter | chapter `boss_blood_backdrop` |
| `grid_records` ambient admission | chapter `shape0_ambient` |
| `music::stage_track` | chapter music (normal and alternate) |
| `stage_transition` | node name and trapdoor edge |
| `nightmares::select_normal_basement` | node admission |

Curse of the Labyrinth is a node and floor rule, not per-floor code: the
floor scales its room count, places a second boss room (589d5c: 584240 for
the next stage, fitted beside the first boss room) and a second treasure
room (58bae3), and the boss trapdoor sits only in the last boss room. The
trapdoor edge of a node with an XL name skips one stage (585640), and the
boss award reads the stage one past the Level's (5c9740). The first boss
drops its ordinary collectible; only the last boss takes the trapdoor or
effective-stage-6 completion branch (5c99da). This keeps a Depths XL first
boss out of Mom's completion branch.

Floor generation itself (`floor.rs`) is the source Level::Init port. Its
stage-number conditions (room count, difficulty band, special-room odds,
Ultra Pride, the surprise miniboss flag) are the original's own arithmetic on
the stage, shared by every floor, and stay there.

## Importing a floor

1. Decode the chapter's room set (`tools/foundation/decode-rooms.py`) and set
   `rooms`. Owner-private sets live in the ignored `.cache/private-base-assets`.
2. Fill the chapter's backdrop (adding its backdrops.xml record to
   `room::BACKDROPS` if missing), overlay range and boss chain from the
   original. Steps run in draw order; a fresh profile's unavailable pools are
   `Draw` steps that consume their roll.
3. Admit `shape0_ambient` and `boss_blood_backdrop` only once their producers
   are verified for that backdrop.
4. Add the stage nodes and point the previous node's trapdoor at them. A
   stage whose 584240 path leaves the chapter's chain carries its own
   `bosses` steps on its node.
5. Import or register the chapter's monsters and bosses; authored spawns
   without a family are dropped on admission (the `floor` bin's `spawns`
   output lists them per room).
6. Run `check:build` and the stage suites, then record the new floor's
   evidence and parity in `TEST_MATRIX.md`.

Migrating Basement I/II and Caves I/II onto these tables was behavior
preserving: 2,550 `floor` bin requests (150 seeds, stages 1..4, with and
without retained family flags and coins, plus the default Basement I path)
produced byte-identical floors and spawn selections before and after.

## Curse and XL qualification

Combined source `8c98576a` passes fresh `check:build`. Production
`fd21587b41a2` (WASM `68d3743a5425`) passes seven fresh Original/actual DOM
Browser `floor` cases: the fixed retry seeds `QNXPTVT1`, `0YAZCZ7N` and
`BHX1EF8A`, the recorded first-OS control `B2JPXEYW`, Lost `B9119L47`
(mask 4), XL `B911VDHM` (mask 2) and Unknown `B9119PTT` (mask 8). Every
ordered descriptor, all 169 grid cells, seeds, door masks and actual attempt
counts match. The OS control replays an earlier recorded seed; this run draws
no new random seed.

Hard `0JLB1WMP` on the same production passes 482/482 checks, including
public Character-menu Down selection and an XL floor (mask 2). This Browser
run reuses the closed Original capture made for `44de9a9d`; it adds no new
Original process. The mandatory hostile `traversal` replay `T1PK6Y0W` also
passes all 173 compared phases with no required failures.

The ordinary Monstro `E1T36YXW` stage-exit fixed and recorded first-OS
approach cases pass 232/232 paired checkpoints each (464/464 total) and both
stage-specific comparers. That OS case varies the approach, not the run seed.

A Rust Game A/B probe on Depths XL `B910LD98` changes the first boss clear
from an error on `d07f70ec` to an ordinary collectible on `44de9a9d`. It is a
port regression control; Original and Browser qualification of that clear,
the second boss and XL descent remain open. Unknown-heart, curse-text and
combat Door.Close pixels have no live grade here. The remaining curse
consumers in [difficulty gaps](difficulty.md#gaps), complete Hard runs,
hard completion marks and later-stage parity remain open.
