# Base Rebirth systems and evidence

This map tracks the private base Rebirth reference set and the implementation
wave starting from the Rust/WASM foundation at `936fda4`.
The [generated test matrix](../TEST_MATRIX.md) indexes executable
scenarios and current implementation contracts. A declared test is coverage;
only an actual report establishes its outcome. Rust/WASM replay equality checks
the port's determinism, while original logs, graphics and controls establish
independent reference evidence.

## Current live-game testing policy

Checked-in Rust, Node and Python unit suites were removed at the user’s request.
The current required gate is [run.mjs](../tools/e2e/run.mjs), which executes
every registered live original/browser suite, retaining all results even if an
earlier suite fails. The [paired gameplay gate](../tools/foundation/paired-game.mjs) gives
fresh original processes/profiles and an immutable production browser receive
identical public input/presentation tails across contrived and recorded
OS-random scenarios. Source-observable title/fader clocks, seed readback,
player motion, pause/controller state, red HP/immunity and active tear/bomb
admission are functional checks. Reports
retain complete original PNGs, consumed X11 phases/focus, DOM traces and
strict pixel differences. Full-frame equality remains an optional diagnostic;
its failures are preserved and do not become functional passing results.

The [paired tape registry](../tools/original/foundation-paired-scenarios.py)
retains the native cutscene prelude and explicitly aligns browser reset at the
independently observed first Title frame0/f32 fade .975 boundary. Cutscene,
complete original-state parity, general actor integration and full gameplay
remain incomplete. Saved `case.json` files replay both seed and randomized
controls with `--replay-cases`; seed-only replay generates new controls.

The Wave1/Wave2 snapshots below are historical results from the earlier test
policy. Their removed unit/integrity suites are not current executable gates.
Consult the latest paired report for current implementation outcomes.

## Current integrated runtime checkpoint

The latest completed six-suite run is commit `5b4901f`, immutable artifact
`a7a5b4972a72c56cd07b33001295955596c9e8f7910e177f522ea037b25444ef`.
At that checkpoint the core passed 140/149 functional checks, finalized floors
5/5, weapons 480/480, initialized-Title MT 6/6 and coin/spike phases 863/863.
All eight core transports passed, with 61/149 whole frames exact. Traversal
executed all three routes and retained the hostile-room lock/return failure.
The runner recorded every suite before returning failure.

[Six-suite evidence](../references/isaac/foundation/runtime-progress/verification.json)
retains exact failed fields and source/build/report hashes. The unchanged
`1E0WJX06` and `W0L0LYYM` random tapes remain required regression inputs.

A subsequent eight-case core run at commit `55f850e`, artifact
`6fcd80f941f4e68e02673d64a949e898fc82c876a5f451ae5d9a9ba2002358ae`,
passes 146/149 functional checkpoints. Both fresh OS-random runs and all eight
original/browser transports pass. Whole-frame equality remains 61/149. The
three remaining failures preserve exact HP and former-rock-cell movement
differences in `T1PK6Y0W`; this run precedes ordinary rock destruction.
[Core evidence](../references/isaac/foundation/runtime-progress/history/6fcd80f941f4-core.json)
records each checkpoint and the immutable source commit.

After ordinary Rock integration, the known-room replay and two fresh random
cases pass 61/64 checkpoints on artifact
`61186ab300e0fdd92e172655f1a5a78d1404c95e72238eac921984f0db09022a`.
All three transports pass complete command and committed-seed checks. The
former-rock-cell XY/VXY now match exactly; the three remaining checkpoints
fail only on source HP3 versus port4. This damage gap remains required.
[Targeted evidence](../references/isaac/foundation/runtime-progress/history/61186ab300e0-rock-core.json)
preserves the actual fields without attributing an unproved cause.

Subsequent scoped aggregate checks pass 848 audio checks with 36 actual PCM
starts, 294 curse-door phases and both retained random regressions, 36/36
checkpoints. Strengthened weapon and tear-lifecycle gates pass all 12 scenarios,
including 1,440 complete command comparisons and 4,704 paired player XY/VXY
fields. The 18 RNG-dependent tear differences remain explicit diagnostics.
[Post-checkpoint evidence](../references/isaac/foundation/runtime-progress/post-checkpoints.json)
retains each report hash and its actual build scope.

Reviewed runtime now includes regular-tear range/grid death and removal, rare
Rock factory metadata, free first-floor treasure doors and ordinary bomb-driven
Rock destruction. The rock gate's historical eight-case cohort passes five
cases and retains three explosive/tinted failures across 2,957 compared phases.
Its current default adds the preserved troll-bomb case; every selected case
remains blocking. Reviewed runtime also includes Fly13/AttackFly18 contact,
queued tear/bomb damage, retirement and delayed room clearance. The scoped
contact/bomb/tear actor comparisons pass 9,344 fields; full AI/shared-stream
and center-grid scenarios retain their exact required failures.

Secret blast discovery, reciprocal doorway persistence and minimap flags have
538 paired phases of scoped evidence. Regular and double keys now admit and
collect through real room actors: the four-case cohort passes 1,690 phases,
followed by a contrived/recorded-random NPC-composite pair passing 845 phases.
Keeper17 admission remains required and red; key spending is a separate slice.
[Key evidence](../references/isaac/foundation/keys/npc-composite-live-report.json)
and [NPC boundaries](../references/isaac/foundation/combat/npcs/verification.json)
record each immutable build's scope.

Public seed-dialog opening, reopening and commitment now follow the source
Character initializer and ordered decoded setter. A fresh six-case cohort
passes 54 checkpoints, 33,696 complete MT words and 582 public commands.
[Seed-dialog evidence](../references/isaac/foundation/seed-dialog-mt/accepted-report.json)
retains the source/build identities and queued-input case. The preserved W0
random tape still fails one tear-lifetime checkpoint; later shared consumers
remain incomplete. Music and item implementation continue in small PRs.

CI discovers all sixteen registered live suites and runs separate jobs with
`fail-fast: false`, preserving each result and evidence. The first matrix run
completed floors, weapons, boot MT and pickups successfully while retaining
core and traversal failures. Build, typecheck, assets, matrix and strict Clippy
pass after the menu, secret, NPC and natural-key integration at `e63723a`. This is a build checkpoint, not a fresh all-suite passing result. Complete gameplay RNG, other floors/characters,
persistence, music/cutscenes and whole-game parity remain open. GitHub required-check
enforcement is unavailable for this private repository's current plan.

## Initial live paired verification after unit-suite removal

The reviewed paired gate replayed all eight recorded tapes against immutable artifact
`d602ad5ecccf9880e204282353cb372e144acaec58c67e4a07bd170a610d48ff`.
All eight fresh original/browser processes, consumed X11 validations, real DOM
traces and seed readbacks completed successfully; the gate source hashes stayed
unchanged during execution. Every capture now requires typed menu or gameplay
observations; gameplay also requires the browser's playing screen. Functional
assertions pass 111 of 149 checkpoints.
The CI result is **failure**, with the actual gaps retained:

| Live scenario | Functional checkpoints | Result / observed gap |
| --- | ---: | --- |
| Title/menu/reentry |19/19 | Pass |
| Movement/fire |12/12 | Pass |
| Pause/release |12/18 | Source remains paused at exit start; port resumes/coasts early |
| Moving bomb, even phase |14/18 | Source and port blast/coasting position and velocity differ |
| Moving bomb, odd phase |13/18 | Source blasts at phase120; port blasts one phase later, shifting HP/immunity and motion |
| Adjacent room/rocks/combat |14/28 | Room relocation/collision and player health loss from room combat are absent from the port; enemy HP is not compared |
| Random `KMAG B3XH` |16/18 | Source/port wall motion and active-tear count differ |
| Random `E74E V3YT` |11/18 | Source/port tear count, resumed motion and immunity differ |

All 149 paired frames were compared as strict pixel diagnostics: 61 match exactly;
`exactOriginalPixels` remains false. Pixel failures are separate from functional
CI status. The report, actual original observations and each exact replayable
`case.json`/public tape are under `.cache/foundation/paired-ci-reviewed/`.

Run `npm run test:e2e:paired -- --replay-case ABS_CASE_JSON --output NEW_DIRECTORY`
to reproduce a scenario's recorded seed and controls. To exercise a subsystem
branch's build without sharing its mutable files, add
`--production-directory ABS_WORKTREE_DIST`; the harness snapshots and hashes
that build before launching Chromium. Typecheck, strict clippy and matrix
synchronization pass. No unit/schema suites were added.

## Reference material

The original macOS base archive extraction contains 2,459 resource files: 21 XML,
2,116 PNG, 82 OGG, 207 WAV, 17 OGV, 10 bitmap fonts, three room STB files and three
text files. [The manifest](../references/isaac/manifest.json) records extraction
provenance and hashes; [the Linux manifest](../references/isaac/linux/manifest.json)
pins depot 250903 manifest `367036646469636831`. The installed build also has DLC,
but the reference resources here use the eight base archives only.

Supplementary extraction from those same pinned Linux base archives restores
94 missing authored WAV samples and the secret-room music jingle. All 301 SFX
and 83 music paths referenced by the XML now exist and are pinned individually
in the resource manifest. [The audio extractor](../tools/foundation/extract-audio.py)
records archive/decoder hashes and validates complete references with `--check`;
`npm run assets:check` also requires this coverage. This establishes resource
availability, while playback and event parity require live game evidence.

Packed base assets contain more information than that unpacked inventory:

- [The authored animation decoder](../tools/foundation/extract-animations.py)
  recovers 790 actors from `animations.a`, with zero unknown names, including
  frame durations, pivots, nulls, events, layers, color operations and sheet paths.
  ANM2 definitions are packed cache data, so their absence from the unpacked
  directory does not mean the animations are unavailable.
- `rooms.a` contains all 13 stage sets declared by `stages.xml`, although only
  Special Rooms, Basement and Cellar exist in the unpacked directory. The shared
  decoder's archive helpers recover all of them without changing the references.
- The Mach-O binary has a symbol table with 21,380 records. Inspection identified
  gameplay, generation, persistence and host subsystems by demangled names.
  Duplicate records are present. The code section does not yield usable plain
  instructions at these symbol addresses; names and addresses establish
  subsystem presence, not recovered formulas or Linux code locations.
- The restored Linux executable is stripped ELF, build ID
  `5531b6a920dee0bc89a7725acdf0ef60415d1990`. The existing offline host runtime
  supplies reproducible original launches and independent fixtures. Old setup
  notes saying Steam startup is blocked describe the earlier environment.

The reusable [room decoder](../tools/foundation/decode-rooms.py) verifies the
Linux archive against its pinned manifest and writes structural room catalogues.
Run `python3 tools/foundation/decode-rooms.py --all-base` to inspect all stages,
or add `--output .cache/room-catalog` to preserve private JSON. Its
earlier structural validation covered truncation, malformed fields, unknown
selectors and exact original
byte round trips. The tool does not reinterpret those selectors as runtime
entities or mutate reference files.

Temporary shared analysis lives under `.cache/coordination/system-map/`:
`macho-symbols.json`, all 790 decoded animations, all 13 decoded STB sets, and
the static Linux index/queries. `next-subsystems.md` and its validated 75-function,
97-direct-edge graph map transition, spawn/update, content and persistence seams;
`mt-consumers.json` inventories the shared MT wrappers with callsites and consumers.
These are private working notes; they are not port behavior or reference
replacement images.

## Authored system inventory

Counts describe definitions, including placeholder and internal entries; they
are not playable-content or parity percentages.

| Subsystem | Direct resource evidence | What remains behavioral |
| --- | --- | --- |
| Characters | `players.xml`: 13 definitions, including transformed Lazarus/Judas; Isaac has hp 6, bombs 1 and item 105 | Unlock gating, whether item 105 is equipped on a fresh profile, stat initialization, transformations and random Eden state |
| Entities | `entities2.xml`: 495 definitions with type/variant/subtype, base/stage HP, collision and shadow metadata | AI, spawn rules, physics, damage, champions, animations and interactions |
| Collectibles and trinkets | `items.xml`: 213 passive, 55 familiar, 75 active and 60 trinket entries | Numeric stat effects, cache evaluation, synergies, use behavior and acquisition rules |
| Item pools | `itempools.xml`: 15 pools with weight, decrease and removal metadata | Pool selection, unlock filters, RNG routing, depletion and rerolls |
| Pocket items | `pocketitems.xml`: 33 cards, eight runes and 28 pill effects | Effects, identification, per-run pill assignment and eligibility |
| Rooms and stages | `stages.xml`: 13 stage sets; STB records include room weights, difficulty, doors, weighted spawn groups and interior dimensions | Floor graph generation, shape selection, entity spawning, persistence and room transitions |
| Presentation | 21 backdrops, 223 costumes, 56 boss portraits, 25 boss color entries, 34 giantbook entries, nine nightmares, 19 cutscene entries | Layer timing, selection, camera, animation clocks, lighting and transitions |
| Audio | Authored catalogue, source SFX admission/shared-MT selection, 301 verified WAV buffers, 16-voice browser output, menu events and trusted-gesture unlock; [audio contracts](audio-runtime.md) | Music lifecycle, remaining gameplay callers, alias/saturation live cohorts and post-Title random lifecycle |
| Metagame | 178 achievement entries, 20 challenge definitions and 203 baby definitions | Save state, unlock conditions, completion marks, challenge rules and co-op |

`ambush.xml` and `fxlayers.xml` contain multiple top-level fragments. A single-root
XML parser rejects them as junk after the root. Do not truncate the files at the
first root or silently discard their later definitions.

## Room-resource contract

The [tracked STB decoder](../tools/foundation/decode-rooms.py) consumed every byte
and re-encoded the exact original bytes of every declared room record in
all 13 base stage sets: 5,314 room records and 206,690 weighted spawn entries
with 298 distinct type/variant/subtype tuples. It is structural resource evidence, not proof of native
generation or entity semantics:

```text
file: count u32
room: type u32, variant u32, difficulty u8, nameLength u16, name bytes,
      weight f32, width u8, height u8, doorCount u8, spawnCount u16
door: x i16, y i16, enabled u8
spawn group: x i16, y i16, entryCount u8
spawn entry: type u16, variant u16, subtype u16, weight f32
```

| Authored stage set | Room records |
| --- | ---: |
| Special Rooms | 492 |
| Basement | 572 |
| Cellar | 570 |
| Caves | 528 |
| Catacombs | 528 |
| Depths | 506 |
| Necropolis | 504 |
| Womb | 503 |
| Utero | 503 |
| Sheol | 239 |
| Cathedral | 186 |
| Dark Room | 95 |
| Chest | 88 |

The authored Start Room is Special Rooms type 1, variant 2, difficulty 1,
weight 1, interior 13×7. Its four possible doors are at grid coordinates
`(-1,3)`, `(6,7)`, `(13,3)` and `(6,-1)`. It contains two weighted spawn groups,
at `(10,2)` and `(9,4)`, both type 0, variant 20, subtype 0, weight 1. Grid type
meaning and which doors the generated floor connects still require original
observations. STB interior coordinates differ from the port's outer 17×11
backdrop assembly grid and must not be silently substituted for it.

The room files also contain spawn types such as 1000, 1002 and 3000 that have no
exact entry in `entities2.xml`, and type-zero spawn variants 10, 20 and 30.
Collection subtypes may be zero in templates while the XML describes concrete
variants such as Penny `(5,20,1)`. A catalogue must distinguish template/spawn
selectors, grid objects and concrete runtime entities; a failed exact XML join
must not silently turn a room object into a generic enemy or delete it.

## Runtime subsystem boundaries

| Native symbol family | Port location / present contract | Reachable missing contract |
| --- | --- | --- |
| `Manager`, `Game`, menus | ABI 2 preserves independent bootEpochSeed in Rust/WASM and browser/native replay; retained title animation/f32 fade uses normalized systemClock; state-driven profiles/run/introduction/pause | Shared MT lifecycle/routing, complete transitions, settings, challenges, continue, game-over, restart and all other modes |
| `Seeds`, `RNG`, global MT19937 | `engine/src/seed.rs`: general seed codec and original-log-verified Stage 1 startup seed; recovered backdrop index-35 stream | Special seed effects, distinct generation/spawn/decoration/item streams, global MT initialization/routing and exact draw consumption |
| `LevelGenerator`, `Level`, `RoomConfig` | `engine/src/room.rs`: recovered supplied-seed backdrop; `engine/src/layout.rs` and `room_templates.rs`: supplied-input growth/expansion and weighted selection kernels, separate from gameplay | Full stage/profile/challenge routing, special fitting/retries, runtime spawning, connected doors, room persistence and floor progression |
| `Room`, `RoomTransition`, `StageTransition` | Room descriptors, initial adjacent minimap cells, authored room composition | Crossing doors, combat locks, discovery, transitions, stage banners and camera behavior |
| `Entity_Player` | Observed previous-velocity motion law, coasting, projection, pause and facing/head/body animation; starting red HP/counters and regular-bomb damage | Original update cadence/full state, item stats, health containers, consumables, death and transformations |
| `Entity_Tear`, `Entity_Projectile`, `Entity_Laser`, `Entity_Knife` | Four-direction regular tears with inherited velocity and corrected emission origins/order; provisional boundary impacts and expiry | Original ground/height/range separation, initial RNG/eye routing, proper projectile bounds, collision damage, enemy shots and weapon families |
| `Entity_NPC`, `Entity_Familiar` | Authored entity definitions available; no runtime actors | Initial enemy behaviors, combat, bosses, champions, spawn/death rules and familiars |
| `Entity_Bomb`, `Game::BombDamage` | E-only rising-edge placement, inventory debit, fuse/explosion, strict regular contact, native f32 self-knockback and damage countdown with pause freezing | Native radial/full-state holdouts, hurt visuals, craters, pushed bombs/variants and enemy/grid damage |
| `Entity_Pickup`, grid entities | Authored entity/STB definitions available; no runtime pickups/grid obstacles | Hearts/coins/keys/bombs, shop/chest collection, rocks/pits/poops/fires and damage |
| `ItemConfig`, `ItemPool`, player item evaluation | No item-effect or pool runtime; XML references available | Pool routing and depletion, passive/active/trinket/pocket effects, synergy cache and costumes |
| Graphics, `ANM2`, backdrop/shadow/FX | `web/animation.ts`, `web/gl-menu.ts`, `web/gl-stage.ts`, `engine/src/effects.rs`: authored animation and GPU masks | Full original pixels, all room/backdrop systems, native motion projection and effect selection |
| `Music`, sound playback | Live paired menu SFX admissions and actual native OpenAL/browser Web Audio PCM submissions | Music events/queues/fades and broader gameplay caller coverage |
| `PersistentGameData`, `GameState` | In-memory profiles and deterministic JSON replay | Durable browser profile/save/continue, achievements, completion marks, unlocks and co-op |
| Controller/button maps, host input | Actual DOM keyboard commands and text; original consumed X11 keys | Controller/mouse/remapping, focus device parity, fullscreen and unsupported action keys |

## Original tick and state observation gap

The clock adapter controls main-thread clocks and presented frames, records
consumed X11 input, and can repeat original RGBA output. It explicitly emits
`simulationTick: null` and `fullInternalState: false`. Rendering repetition does
not establish a one-to-one relationship with simulation ticks.

Wrapped Mach-O landmarks include `Game::Update`, `Game::Render`,
`Game::InterpolatePositions`, `Game::RevertInterpolatedPositions`,
`Room::Update`, `Entity_Player::Update`, player movement/shooting controls and
tear height/falling-speed/falling-acceleration setters. In particular, multiple
numeric position copies may reflect simulated, interpolated or projected
state. The existing [position observer](../tools/original/position-observer.py)
tracks bounded read-only f32 hypotheses. Its address matches are not typed
player fields or a complete original state serializer.

Validate a candidate numeric contract with independent random seeds, separate
horizontal/vertical/diagonal tapes, pause and release, and stationary/moving/
coasting shots. For tick calibration, vary virtual frame-time steps while
preserving elapsed time and consumed-input order; include zero, half and double
steps around presentation boundaries. Require observed numeric evolution to
distinguish update cadence from interpolation. Hold out direction/length cases
from fitting, and keep graphical projection and quantization separate from
world integration. A full-state adapter must identify all future-affecting
state and RNG streams before claiming continuation parity.

Fresh continuous original holdout/inheritance evidence lives under
`.cache/coordination/physics/{holdout,inheritance}-case/run/`. Its observed axial
world-position/velocity pairs support advancing position with the previous
velocity, followed by `v' = (v + 0.6 × input) × 0.8803408` at the tested
presentation cadence. Numeric world coordinates project to logical ground
coordinates as `(0.65 × x + 32, 0.65 × y − 46)`; the actor sprite anchor is one
logical pixel above ground Y. Coasting does not snap to zero in those traces.
These are scoped observed numeric relationships, not a full typed native-state
contract or proof that presentation count equals simulation tick count.

The same traces show moving tears around 4.64 logical pixels per presentation
versus stationary 3.25, establishing a velocity-inheritance requirement.
Original right-facing tears cross even the old port's logical right bound 410 and
can survive more than 45 presentations. The old shared player-wall clamp,
fixed 45-step expiry and constant projected Y drift therefore remain inadequate
until independent ground/height/range/steering contracts replace them.
The latest physics integration implements the observed motion law, inherited
velocity and emission origins/order; its integration checkpoint passed all 44
native engine tests. Current projected `+0.125` tear drift and 45-step expiry are
tracked incomplete contracts. Projectile planes now live in a separate
`projectileBounds` descriptor, preserving provisional 70/410/48/222 values
while measured player walls derive from authored floor/radius metadata. Native ground
Y, interpolated render offset, height and falling-speed observations exist, but
global RNG consumer routing and range-end drop remain incomplete in production.
The working evidence and exact source tapes are in
`.cache/coordination/physics/README.md`; original pixel/mask gates remain active.

The physics workstream also mapped concrete restored-Linux RTTI/vtable paths
instead of relying on wrapped Mach-O landmarks. `Entity_Tear` typeinfo/vtable
are at `0x671230` / `0x671270`, with its candidate Update entry `0x542530`;
`Entity_Player` typeinfo/vtable are `0x670c60` / `0x670c90`, with Update
`0x531850`. Static/runtime cross-checks identify common position/velocity
offsets `0x448` / `0x464`, and tear-specific height `0x7d4`, falling speed
`0x7d8`, falling acceleration `0x7dc`, friction `0x7e0` (initial 0.85) and scale
`0x7e4` / `0x7e8`. These are private reference-observation landmarks, not
production addresses or proof that the entire objects are understood. They
unblock bounded typed trajectory observations for the next height/range task.

Native player movement source at restored-Linux entry `0x529750` uses exact
retention `0.8803408145904541` and perpendicular suppression
`0.8944271802902222` (their product matches the port's scoped turn damping).
Fresh directional wall probes independently establish world collision limits
X 70…570 and Y 150…410, projecting to logical X 77.5…402.5 and Y 51.5…220.5.
They correspond to the authored floor plane plus the player's 6.5-logical-pixel
circle radius. The source reversal branch adds
`max(−0.04 × forwardVelocity, 0)` to acceleration 0.6 and is now implemented
following an independent immediate-turn probe. Native collision velocity
response and overall update cadence remain open.

Tear initialization also identifies a separate global MT19937 subsystem. Its
restored-Linux Init path (`0x5423d0`) calls float helper `0x42e850`, which calls
Next at `0x5e4fe0` with a 624-word state and the standard observed masks
`0x9d2c5680`, `0xefc60000`, `0x80000000` and `0x7fffffff`. The helper converts
the unsigned output to f32 with a factor near 2⁻³². Reference index/state
landmarks are `0x8e6b64` / `0x8ed680`; native storage uses eight-byte words with
32-bit MT values. Startup wrapper `0x42e830` calls `time(NULL)` and tail-jumps to
seed initializer `0x5e4e90`. Independent full-state snapshots match the clock
adapter's epoch 1700000000 exactly across distinct run seeds. Menu, room ambient
and tear consumers share this stream: it is separate from run/stage/room and
per-entity xorshift streams. The private consumer inventory has 277 function
owners, with 443 uniform, 717 bounded-integer and 599 raw-uint direct callsites;
these static counts are not executed draw counts.

Production has not yet verified the full boot-clock lifecycle and shared consumer routing.
Native first-shot eye/height observations are source discoveries, not implemented
projectile parity. Replacing the stream with startup xorshift or skipping to a
fitted cursor cannot establish the original routing. Complete-state observations
must include its future-affecting state/index and the menu, ambient and gameplay
draw ordering as well as per-room streams. No native addresses enter production.

The shared MT lifecycle now has concrete state-producing boundaries. Level
construction and GameState construction each initialize 135 room descriptors,
each with 448 grid descriptions and two raw seeds per description: 120,960
draws. GameState Clear repeats that initialization and creates one saved
PlayerManager seed, for 120,961 draws. Startup's three capacity initializations
plus the Clear/backdrop seeds match the full native MT checkpoint at 362,882
draws. Save-menu initialization enumerates profiles 1, 2 and 3 and restores the
previous profile, each invoking Clear. Character initialization generates one
run seed, and stats initialization generates 179 authored pose values. Independent
native state, run-seed and pose observations verify all 624 words at
`362882 + 4×120961 + 1 + 179 = 846906`.

These are initialization consumers with actual generated state, separate from
variable gameplay consumers. Level reset initializes its grid descriptions again;
Room Save initializes only absent cells within the actual room grid, consuming
two words per absent cell. Grid construction consumes two words per object.
Admitted sound events consume one sample-selection draw after ID/loading/cooldown
checks, including a one-sample event; volume is scaled afterward. Conditional
room rendering and update effects still require exact consumer admission and
ordering. Matching a checkpoint cannot justify an arbitrary cursor skip.

## Backdrop contract recovered within its scope

Four independent fresh-profile OS-random original runs match the recovered
backdrop when their observed decoration seed is supplied:
`N4Z8GWV3` (run 1766104246, decoration 4271600350) and `P01MXJW0`
(run 1648477791, decoration 1985222680), followed by held-out `B29ZRLP1`
(decoration 2463833516) and `PNWJRQD3` (decoration 1082003586).
The comparison covers all four floor
and 44 wall quads, every submitted XY/UV vertex, crops and mirrors. Destination
coordinates match; the maximum source-coordinate difference is 0.00003072
pixels and is bounded by one f32 UV ULP plus the native trace's nine-digit
serialization. This numeric-coordinate bound does not relax original RGBA
pixel gates.

The generator now uses decoration xorshift index 35, a repeated floor catalogue
with a general shuffle, and conditional wall-variant draws. Earlier Rust room
and native verifier integrity cases passed before their removal. The historical
report is `.cache/coordination/room-generation/verification-report.json`; current
gameplay acceptance uses the [live paired gate](../tools/foundation/paired-game.mjs).
The [persistent backdrop observations](backdrop-observations.md) record exact
licensed/restored binary hashes, identified Linux routine roles and the four
reference cases for the next generation workstream.
Production still substitutes Stage 1 seed for the decoration seed. Conditional
generation draw routing and source graph observations have advanced, but the
general algorithm is not implemented; FX and other stage variants remain open.
Do not turn observed draw counts into a per-seed offset table.

Static notes additionally identify a 169-cell level topology grid with initial
cell `(6,6)` / index 84, a primary stage-index-35 stream and a secondary
index-2 stream seeded from the stage seed. Conditional placement and template
draws are being reconciled through their actual consumers. This level grid differs
from both STB room interiors and backdrop assembly coordinates.

Two independent source runs now join all 21 observed descriptors against authored
STB records, room RNG triples, shape cell rectangles and enabled door targets,
including two actual initial-room→neighbor transitions. `N4Z8GWV3` traverses
cell 84→71 to native room 1.252; `W2K03EEH` traverses 84→83 to room 1.240 and includes
a 26×14 four-cell shape. The read-only
[descriptor observer](../tools/original/foundation-descriptor-observer.py)
and private `topology-validation-report.json` retain the exact source contracts.
Room config fields are asset stage/type/variant, not type/stage/variant: StartRoom
has header `(0,1,2)`, while ordinary Basement type1 masks that distinction.
Starting descriptor `+0x8148` also differs from current/previous `+0x8188/+0x818c`.
Superseded diagnostic schemas remain clearly labeled. These are independently
verified source joins, not a production topology or transition implementation.

The moved 3BMB GPU-mask contract now has controlled native provenance. Its
legacy capture requested a wall-clock 450 ms hold but actually held about
482 ms, whereas the browser tape used D27/release30. Two new fresh-profile
original captures with consumed key phases 1207 down / 1234 up and endpoint
1263 reproduce all light/shadow RGBA bytes and primitive bounds exactly; their
neutral targets also equal the old fixtures. The native controlled originals
replace only that moved-case reference, retain the legacy timing evidence, and
preserve the exact RGBA gate. This corrects an observed tape mismatch without
asserting original simulation ticks or full hidden state.

## Supplied-input room growth and template selection

The [room-layout evidence](room-layout-evidence.md) records accepted standalone
Rust kernels for rectangular growth/dead-end expansion and weighted authored
template selection. The earlier bounded layout CLI accepted independently
supplied generator or selector inputs and is now ignored research. This does not provide
complete run-seed floor generation, connected gameplay rooms or traversal.

Eight retained original floors verify all 32 ordinary template identities across
all four rectangular shapes. The picker preserves authored order, door-mask
filtering, source f32 arithmetic and mutable weight decay. Two fresh independent
originals, `7X7BGA4Y` and `GS9E16DG`, add 12 exact selector identities. Six Rust
cases and six verifier integrity cases pass; the source observer checks complete
descriptor/grid/template/RNG chains against the actual initial-room log.

Seven retained finalized rectangle sets match the growth kernel, including zero,
one and two dead-end expansions. The strict comparison still fails `MPCMQQN6`:
source special assignment changes a generated 1×2 rectangle at cell 49 into a
1×1 shop at cell 62. Fresh `7X7BGA4Y` matches growth, while `GS9E16DG` retains a
second fitting difference, a generated 2×2 at 56 versus native 1×1 at 70.
Running the verifier with `--contract growth` or `both` reports these as nonzero
failures and still attempts every selected case. Primary stage/profile/challenge
draw routing, special fitting, retries, secret placement and runtime spawning
remain separate open contracts; no captured map or seed-specific offset is used.

## New bomb reference observations

The bomb workstream captured fresh stationary, escape and continuous-graphics
native runs under `.cache/foundation/bomb-stationary/run/`, `bomb-escape/run/`
and `bomb-exact/run/`. E consumes the starting bomb once, with the displayed
counter changed by relative presentation 2. In these starting-room tapes, the first explosion is observed at
relative presentation 121, after 120 bomb presentations. The authored Pulse
animation has 59 frames; native playback advances at two presentations per
animation frame in these captures. These units describe observed presentations,
not a proven native simulation tick.

A stationary full-health Isaac loses two half-heart units (6→4); moving upward
for 50 presentations escapes without damage. Horizontal probes around 28
logical pixels from the bomb hurt, while probes around 57 and 86 do not.
Those probes alone bracketed the effect. The subsequently recovered
[regular bomb path](bomb-evidence.md) establishes native blast radius 75 plus
fresh Isaac collision radius 10, converted by the observed 0.65 world scale:
the center-distance boundary is exactly 55.25 logical pixels with a strict
comparison, so equality is safe. The implementation now applies the recovered
f32 outward knockback before attempting damage and stores the normal hit's
120-step damage countdown. Eleven focused Rust bomb cases pass, including exact
contact, radial offsets, impulse during immunity, new-run reset and the
independent complete coasting-blast trajectory. The existing port applies blast
impulses before player motion/drag, reproducing the D27/release30/E fixture's
sampled native x/vx at presentations 121, 123 and 133. That is a scoped
presented-step fit, not the source full-phase actor order. Fresh typed admission
lists include the player; later-born bombs update after it in a full phase.
Their impulse advances the player on the following intermediate phase. The
source player update calls base integration, movement control and then DropBomb.

Empty inventory and native pause/fuse captures pass. Four independent Shift
probes, including window-targeted and physical XTest events for each alias,
leave native inventory and HP unchanged. The unsupported port Shift emission
was therefore removed; E is the independently verified placement action.
Native immunity observations show countdown 120 at explosion presentation 121
through zero at 241, matching the port's decrement within this tested cadence.
Floor craters, pushed bombs, variants, hurt visuals and enemy/grid damage remain
explicit gaps. Independent radial/full-state holdouts remain useful; the source
phase scheduler identified below is not yet implemented by the port or exposed
as verified phase delivery by the original adapter.

A later independent native combat-room tape observes blast placement phase 120,
while the starting-room tapes observe 121. Fly HP 3 retains pending damage 60
through phases 120/121, then drains to HP −57/removed at 122. This exposes unresolved
simulation/render phase alignment and deferred damage scheduling. The existing
121-based mask/countdown contracts remain scoped to their controlled starting-room
tapes; they are not a universal original fuse-phase assertion.

Independent bomb presentation comparisons cover 23 original/production public
GL ages and match authored crops, animation phase and nontransparent layer
order, including the fuse fragment during the first 20 explosion ages. Full
sprite pixels remain open: original axis-aligned corners snap at half logical
pixels, while current raw port corners can differ by 0.25; rotated smoke still
has small float-order differences. Original explosion-light submissions have
radius 225 from the authored gradient scale/pivot and a quadratic falloff over
the first 12 observed presentations. The
primitive reports under `.cache/coordination/presentation/`
are scoped sprite evidence, not complete frame parity.

The frozen GPU-mask manifest now selects nine cases: the three neutral seeds,
controlled movement, and coasting bomb phases 121, 123, 127, 131 and 133.
All five new native shadow/light target pairs repeat exactly in independent
fresh profiles. Their persisted evidence binds actual E consumption, complete
paired GL phases, controller/binary/log/target hashes and original repeats.
The unchanged browser gate compares every straight RGBA byte of both complete
512×512 targets with a zero budget. Seven Node integrity cases and eight native
capture guards reject omitted phases, altered actual input, partial/reused GL
frames and changed target bytes. Agreement on these complete GPU targets still
does not establish full sprite/frame pixels or original hidden-state parity.

The integrated frozen production artifact now matches all nine pairs, all 18
complete RGBA targets, exactly. Native public GL queries identified an attached
24-bit depth buffer, `GREATER`, depth writes, clear depth 0 and monotonically
decreasing submitted layer Z. Retaining a real depth attachment changes Mesa's
interpolation path; enabling depth testing without one does not reproduce the
scaled-light texel boundaries. The browser uses matching 24-bit depth semantics
and per-target layer ordering, resets depth on clear, and explicitly declares
`highp sampler2D` to preserve desktop texture-alpha precision during fades.
An independent native shader/geometry/texture replay isolated these two causes;
the query's original repeats remain byte-identical. No geometry epsilon, target
replacement or tolerance change was introduced. Working controls and reports
are under `.cache/coordination/presentation/sampler-proof/`. The retained
[independent native depth contract](../references/isaac/foundation/stage-masks/3bmb-controlled-depth-contract.json)
binds source depth format/function/writes/clear/range/geometry to raw GL, actual
controller input, logs and target hashes. Loader and capture guards reject
missing or altered source fields and provenance.

## Source entity and grid boundaries for the next implementation wave

Restored-Linux transition consumers form a concrete chain: transition update
`0x5ddd30` → change `0x5dd1b0` → game change-room `0x550970` → level change
`0x58d900` → load `0x586c90` → room init `0x5cecc0`. Room init selects weighted
entries, resolves template selectors and dispatches entities or grid objects.
Game spawn `0x550a40` uses typed factory `0x555700`, virtual initialization,
position/velocity assignment and room admission `0x5c6940`. Base initialization
stores type/variant/subtype and spawn seed, and seeds the per-entity xorshift
stream with index 30. The source weighted group picker and spawn stream are
separate from global MT; empty groups still consume their preceding random draw.

Room update `0x5d12a0` updates grid cells before the entity manager `0x454df0`,
which traverses the active entity array. Its separate extra player-update loop
runs only when `EnableInterpolation` is disabled; with interpolation enabled,
the fast phase supplies the additional player update. Player grid collision
`0x443d00` follows that update; render interpolation also calls it with a distinct flag.
Independent typed lists corroborate this array includes the player. Ordinary
Fly-room admission is NPCs, player, then later-born bomb; the full update retains
that order. Newly appended births are fenced by the last actor present when the
full update starts. The source does not universally update all nonplayers before
players, and the legacy E121 fit does not establish that ordering.
NPC RTTI, typed AI dispatch and deferred base damage/update consumers identify
current/max HP fields. These static boundaries help isolate scheduling and
collision contracts; they are not complete object layouts or accepted enemy AI.

Physics additionally recovered the actual mixed-rate dispatcher. System loop
`0x5950e0` contains the gameplay branch `0x595490`, which calls game input and
branches on the full/fast phase flag. The config loader explicitly identifies
`EnableInterpolation`, default 1, at the manager's `+0x2dfd4`.
Full phases revert interpolated positions and call game update; intermediate
phases call game fast-update `0x5507f0` → room fast-update `0x5cd920`. Players
update in both phases, while nonplayer simulation updates occur on full phases
and intermediate tear work uses its render/interpolation virtual slot. The
observed 60-presentation cadence advances players 60 times and nonplayer
simulation 30 times; there is no universal entity tick. Independent typed tear
traces now verify 215 phase points across five entropy seeds. This is source
discovery; production still has its prior tear drift/expiry and lacks the shared
MT routing. The original adapter still emits `simulationTick: null` and does not
serialize complete source state.

Pending run dispatch at `0x594a10` also aligns an odd System counter to an even
pre-update phase before gameplay continues; it does not reset the System clock
for each run. The port now exposes a normalized System clock with this run-entry
alignment and uses it for retained title animation and f32 menu fade. Full
gameplay mixed-rate integration and native run-entry phase holdouts remain open;
the presence of a clock field does not establish complete phase parity.

Fresh native combat observations independently enter an authored adjacent room
with RoundWorms and rocks. Typed rock state changes from type 2/state 1/class 3 to
state 2/class 0 after a blast, and passage through the former cell is observed.
Independent vertical and horizontal blast traces recover source ray/cell/line
checks. Native grid index conversion uses 40-world-unit cells with a padded 15×9
grid, distinct from the 13×7 authored interior. The private source traces and
contract tooling are in `.cache/coordination/combat/`; there is no grid/enemy
runtime or combat parity gate in the port yet.

Content initialization, collectible configuration, weighted pool depletion,
AddCollectible/use dispatch and persistent unlock/save consumers are mapped to
concrete Linux boundaries in the ignored graph. Resource definitions alone still
do not implement any item effects, room-clear progression or durable saves.

## Introduction and pause presentation verified regions

The presentation workstream's two focused production browser cases pass exact
original comparisons for the stage-banner text (5,760 pixels), pause paper with
stat/cursor widgets (119,880 pixels), and the two-line actual seed (7,168 pixels).
An independent fresh original/browser run with arbitrary seed `0HKFG1LX` also
has zero changed pixels in those regions; its report and original manifest are
under `.cache/coordination/presentation/random/`. This validates the general
authored attachments and bitmap-font layout within those regions. It does not
establish complete stage-entry/loading/fade pixels or unsupported Options/Exit
behavior.

The generic animation math now uses sequential f32 interpolation and transforms.
Earlier component validation compared all 75 original title quads across 25
observed phases and all 42 smoke quads with their alpha values; 38 smoke primitives
are from independently held-out phases. The retained
[original primitive fixture](../references/isaac/foundation/animation-math/primitives.json)
feeds [generic animation playback](../web/animation-math.ts),
and the native scenario registry includes first-title, fade and authored-loop
checkpoints. This establishes the tested generic geometry/alpha contracts;
fresh production title/fade pixels are independently checked, while the legacy
whole-foundation comparison still has its separate title mismatch.

The [native title/fade evidence](animation-evidence.md) also retains ten
byte-identical paired original PNGs for entry, fade, loop and menu reentry.
The read-only menu observer confirms the title animation keeps its actor frame
while another menu is selected; returning does not restart Idle. Three captured
transition checkpoints currently establish retained source state, with complete
transition rendering still pending. Reproducible source tapes and observations
are in the [title reference manifest](../references/isaac/foundation/title-animation/manifest.json).

An immutable ABI 2 artifact with digest
`a98a29f426a8c8f58174b02f31cedf94528b78513c4261486af5e021597e72ad`
matches all seven selected complete original title frames with zero changed
pixels and zero channel error. All ten scoped title/fade state checkpoints also
match. The [browser contract](../tools/tests/title.spec.ts) keeps the three menu
transition captures as source-state checks until their rendering is implemented.
The focused presentation/generic-GL suite passes five cases. This scoped result
does not replace the legacy references, full gameplay comparisons or original
complete-state requirements.

The reusable [presentation tape](../tools/original/foundation-presentation-scenario.py)
is registered as `run-pause-presentation` in the strict executable original
scenario index. Introduction age, pause appearance/idle/disappearance, cursor
selection and stat-frame state live in the engine snapshot, so pause freezes
gameplay while its menu presentation clock advances independently.

## Verification snapshot for the Wave1 checkpoint

Required cloud verification completed against immutable production artifact
`6f87ca918d1e0da3eb191822065a2da5aa4a84038e5dd48839fcae291daff7c7`.
All check groups passed except the final original full-frame gate. That checkpoint executes all eleven selected
checks: native Rust/browser WASM actual-command state equality and five full menu frames
pass; title, gameplay-start, movement, shooting and after-fire remain failures.
The matching menu frames are profile, main, character, seed-entry and
character-seeded. The report's `exactOriginalPixels`, `originalStateParity` and
`originalSimulationTickDelivery` flags remain false.

The frozen runtime passes all 14 compiled behavior browser cases, including all
nine GPU-mask pairs/all 18 complete RGBA targets. The source-discovered Node
suite passes all 43 cases with zero skips, including the strict native depth
evidence guard; native capture and graphics observer integrity suites each pass
eight cases. All 13 room resource sets pass complete structural decoding and
exact byte round trips. The generated matrix agrees with executable source.
These outcomes validate their scoped contracts; the five original pixel failures,
phase delivery and complete native-state gaps still block full parity.

## Historical verification snapshot for Wave2

The shared immutable ABI 2 artifact
`e141553ff314fe5029e2067638e9bdb582c901f7a71721aca731feb60058c1cb`
passes strict clippy, all 75 Rust tests, all 49 Node tests and all 17 browser
tests. Browser coverage includes all nine native mask pairs/all 18 complete
straight-RGBA targets at zero tolerance, seven complete new native title frames,
ten scoped title/fade checkpoints, exact pause/banner regions, independent
boot-epoch replay and held-input regressions. All six layout integrity checks
and the eight-original/32-template supplied-selector comparison also pass.
The growth/special-fitting failures described above remain outside that accepted
selector scope and still fail their strict comparison.

Required cloud verification also passes all six room decoder cases, the native
clock and observer integrity groups, two fresh original clock runs, four fresh
original seed probes and two fresh backdrop probes. The retained combat contract
passes seven integrity checks and all eight cases/100 exact f32 checkpoints.
The final physics evidence hardening was rerun separately: all 16 integrity
checks, 215 exact projectile phases, two complete 624-word MT states and 18
descriptor boundary seed pairs pass. These kernel checks remain conditional on
their declared source inputs; they do not establish runtime routing.

The retained legacy foundation gate still reports 6 passing checks and 5
failures. Its title mismatch decreases from 39,484 to 10,538 changed pixels;
the four gameplay whole-frame failures remain. Its complete menu-frame and
native/browser command-state checks pass. The new controlled title fixtures
and this older comparison are separate evidence contracts; neither result
replaces the other. Current actual results are at
`.cache/foundation/verification/latest/report.json`. Full original state,
simulation-tick delivery, shared MT consumer routing and complete gameplay
parity remain unverified.

## Parallel implementation acceptance

| Independent workstream | Next verifiable result |
| --- | --- |
| Motion and regular tears | Paired read-only numeric/graphics traces for holdout directions and moving/coasting shots; native Rust/WASM continuation matches; original failures remain visible |
| Room generation and decoration | General seed-driven draw routing checked on fresh held-out originals; authored templates/door geometry feed state; no per-seed tile or draw-offset table |
| Bombs and inventory | Extend verified E/fuse/pause/escape/inventory/self-damage to radial holdouts, hurt visuals, pushed/variant bombs and enemy/grid/crater interactions |
| Run introduction and animation | Stage banner/input enablement timing observed across seeds; authored events advance from a run-transition clock and freeze/reset correctly |
| Asset catalogue and combat | Typed authored room/entity metadata, unsupported entities reported explicitly, one observed enemy behavior and combat room with damage/death/door unlocking |
| Renderer and browser verification | Immutable production build actually exercised, exact original GPU masks preserved, all selected pixel/state assertions run, failed regions retain source-indexed investigation tasks |
| Persistence and audio | Real profile reload/continue and browser audio event contracts backed by original observations; browser policy handling is explicit |

Keep the broad subsystem rows in the matrix generator synchronized with these
contracts when implementation or reference evidence changes. Executable browser,
Node and original-clock scenarios continue to be discovered from their source;
do not claim a broad subsystem passes merely because its index row exists.
