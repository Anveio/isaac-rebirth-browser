# Game roadmap

The whole base Rebirth game, split into subsystems, with where the port stands
on each and every outstanding bug filed under the subsystem that owns it.
Afterbirth and Afterbirth+ are out of scope.

This file is the planning view. Evidence lives elsewhere and is not repeated
here: [system-map.md](system-map.md) holds original-game evidence and
acceptance records, [TEST_MATRIX.md](../TEST_MATRIX.md) holds executable
coverage, [dst.md](dst.md) holds DST findings, and GitHub issues hold bug
detail. A row here is a pointer and a status, never a parity claim.

The counts below describe the reconciled open issue rows, not a whole-game
passing result or a fixed source snapshot.

## How to update this file

Any session may edit this file through a short PR to `main`. Keep edits small
and re-read the file just before changing it, since other sessions update it
too.

**Filing a bug.**

1. Every bug is a GitHub issue first. Search open issues and reuse a matching
   one rather than filing a duplicate. Title it with what a player sees, not
   the native address.
2. Add one row to the **Open bugs** table of the subsystem that owns the
   fix. Pick the subsystem whose code must change, not the one where the
   symptom shows: a freeze on Dip death caused by a missing item callback
   belongs to *Items*, not *Enemies*. When it genuinely spans two, file it
   under the one that must change first and name the other in the row.
3. Row format: `| #N | severity | one-line symptom | source |`.
   - Severity: **P0** crash, freeze, softlock or a run that cannot continue;
     **P1** wrong behaviour a player notices in ordinary play; **P2** a
     divergence only a paired original/port comparison shows; **P3**
     cosmetic, tooling or a test route that needs regenerating.
   - Source: `playtest <date> <footage link>`, `e2e <suite>`, `DST`, or
     `issue` for reports from elsewhere.
4. If you cannot tell which subsystem owns it, still file it under your best
   guess and append `(triage?)`. The roadmap owner settles it.

**Closing a bug.** When the fix merges, delete the row and add one line to
*Recently fixed* with the issue and PR. Close the issue once a playtest or the
owning suite confirms the fix. Lines in *Recently fixed* older than about two
weeks are pruned.

**Changing a subsystem's state.** Update its row in the overview and its
section together. States are **Missing** (no runtime), **Partial** (some of
the base game runs) and **Complete** (all base Rebirth content of that
subsystem runs; parity may still have open P2 bugs).

The playtester finishes every playtest by reviewing its side-by-side
original/port footage, then filing what it saw here by these rules.

## Milestones

| Milestone | What a player can do | Gate |
| --- | --- | --- |
| **M0 Basement run** | Play normal Isaac from the title through Basement I and II to the boss and trapdoor without a freeze | Verify the complete route. PR654 adds Caves I descent source; selected-wall catalogue and current-build qualification remain separate. |
| **M1 Chapter 1** | Basement I/II and Cellar with every Chapter 1 enemy, boss, room type and common item | All Chapter 1 bosses; shops, devil/angel rooms, Cellar unlock |
| **M2 Chapters 2-4** | Caves/Catacombs, Depths/Necropolis, Womb/Utero, Mom and Mom's Heart | Other stage generators, their enemies and bosses |
| **M3 Endgame** | Sheol/Cathedral, Dark Room/Chest, Satan, Isaac, ???, The Lamb, endings | Final floors, final bosses, ending cutscenes |
| **M4 Full catalogue** | All 343 collectibles, 60 trinkets, cards, runes, pills, all characters | Item and character imports |
| **M5 Metagame** | Saves and continue, unlocks, achievements, challenges, co-op | Persistent profile in the browser |

## Subsystem overview

Counts include unresolved evidence and confirmation work on still-open issues.

| Subsystem | State | Port today | Open bugs |
| --- | --- | --- | ---: |
| [Boot, menus and pause](#boot-menus-and-pause) | Partial | Title, profile, main, character, seed and Challenge pages with page pans; pause | 3 |
| [Input](#input) | Partial | Keyboard only | 0 |
| [Seeds, RNG and shared MT](#seeds-rng-and-shared-mt) | Partial | Seed codec, run/stage/item streams and shared MT owner; consumer alignment incomplete | 6 |
| [Floor generation](#floor-generation) | Partial | Normal Basement I/II and Caves I source | 0 |
| [Rooms, doors and transitions](#rooms-doors-and-transitions) | Partial | Four base rectangle sizes; eight ordinary/special room types and admitted bosses | 3 |
| [Stage progression](#stage-progression) | Partial | Boss clear award, trapdoor, Basement I/II to Caves I source | 3 |
| [Player](#player) | Partial | Normal Isaac movement, stats cache, health, death | 3 |
| [Weapons and tears](#weapons-and-tears) | Partial | Tears plus a set of weapon items and synergies | 1 |
| [Bombs](#bombs) | Partial | Placement, fuse, blast, Troll/Mega conversion | 0 |
| [Collision and physics](#collision-and-physics) | Partial | Player, actor, grid and pickup contacts | 3 |
| [Grid objects](#grid-objects) | Partial | Rocks and tinted rewards, bomb rocks, spikes, poop, TNT Full countdown/nonrare blast, webs, trapdoor, decorations | 2 |
| [Enemies](#enemies) | Partial | Ordinary NPC families, now including Hopper/Trite and Flaming Hopper source | 3 |
| [Bosses](#bosses) | Partial | 8 boss/miniboss families, including Envy/Super Envy | 2 |
| [Pickups and chests](#pickups-and-chests) | Partial | Coins, keys, bombs, hearts and supported chest families | 3 |
| [Items, pools and familiars](#items-pools-and-familiars) | Partial | Pools complete; about two dozen items with behaviour | 1 |
| [Pocket items](#pocket-items) | Partial | Card/Pill Init, held slots/HUD, collection/spill, held Save and Range Down/Up use | 0 |
| [Special rooms and economy](#special-rooms-and-economy) | Partial | Normal Shop stock/prices and scoped paid Bomb witness; other economy frontiers remain | 0 |
| [Characters](#characters) | Partial | Normal Isaac only | 0 |
| [Rendering, animation and camera](#rendering-animation-and-camera) | Partial | ANM2, backdrops, lighting masks, camera, HUD | 4 |
| [Audio](#audio) | Partial | SFX and floor music | 0 |
| [Saves and metagame](#saves-and-metagame) | Partial | Origin-local initial-room Save/Continue and own reload; Native restoration gaps remain | 1 |
| [Co-op](#co-op) | Missing | None | 0 |
| [Test routes and tooling](#test-routes-and-tooling) | n/a | Live e2e, DST, playtest | 3 |

## Boot, menus and pause

Rebirth scope: intro cutscene, title, profile select, main menu, character
carousel, seed entry, challenge menu, options, stats, pause menu, death and
win screens.

Port: Title, profile, main, character and seed menus run with source timing;
the menu board pans between pages, with File select clips, the character
carousel, Back/Select widgets and hard-mode fades
(`engine/src/menu_presentation.rs`, frame-exact on the menus tour); run
introduction banner; pause menu; Challenge page and declared Challenge1 XML
ingress. The separate Options leaf owns eight-row navigation, actual
SFX/Music settings, pre-constructor preferences, Back save and a fullscreen
intent/result bridge. The 402af-based artifact 99f passes build and all 428
bounded same-Native Options fields and both unchanged Browser runtime tests.
Default paired-kernel replays preserve all prior values: fixed 5999/5999 passes;
recorded HH936E8Z keeps its 78 failures among 6060 checks, with no lost passes.
Historical 3a9 runtime/SFX/free-SKU checks keep their separate artifact scope.
The registered `options-audio` suite owns actual request/query/gain/reset, Back
persistence and final public run-seed observations. Its first a716/99f fixed plus
unfiltered OS result passes 866/866 fields. Fresh main634-based 4ca Native/Browser
integration replays the same fixed plus recorded JWMM YY0Y bank with no new draws
or filtering: 866/866 pass and all 32 Browser PNG bytes equal 99f. Fresh Native
identities and journal labels differ; verdicts do not. Both 400-state history caps
and 16/32 pixel diagnostics remain failed. The [public replay bank](../references/isaac/cases/options-audio/recorded-fixed-jwmm-epoch-holds-v1.json)
keeps the inputs. Keyboard Controls now owns
the 24-action/two-column editor and its persistence; its [artifact-bound fixed,
unfiltered OS and Full/Fast bomb results](controls-keyboard.md) remain separate
from other systems. Fresh main173 artifact 4ca passes runtime 2/2 and exact fixed
W→I plus recorded OS replay (1,568 fields unchanged and 44 identical Browser PNGs).
The [partial Map opacity slice](map-opacity.md) now passes all 3,122 functional
checks on fixed plus first-unfiltered-OS inputs and all 26 menu-gauge regions;
15 of 48 required render regions remain failed. Runtime 3/3 and one recorded
Music case (261/261) pass on the same frozen artifact. Options Audio V3 fixed
plus recorded JWMM passes 948/948; history caps and pixel diagnostics remain
failed. Same-Native Save replays preserve all 3,748 field values and statuses
(3,650 pass, 98 fail), with zero lost passes, 66 own-save checks, 48 MT snapshots
and two real reloads passing. Filter and Popups callbacks,
shaders, further OS interactions and reload stay separate. Missing: intro cutscene (the paired tapes
skip it), stats and win screen. General challenge rules remain separate.

Keyboard document reload now passes 298 own-save/load and gameplay checks on
the retained fixed and recorded OS W→I routes. All 1,568 paired fields and 44
original-document Browser PNG bytes stay unchanged; RGBA remains 18 pass/26 fail.
This opt-in scope does not qualify Native process reload or the whole Controls bank.

Next: complete Native Controls reload and remaining callbacks; close Map rendering gaps; death screen parity; intro cutscene.
See [source and live scopes](system-map.md#source-implementation-and-live-scope).

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #168 | P2 | Escape during a room transition pauses in the port; the original refuses | e2e paired-random-run |
| #647 | P2 | Options page draws no Filter or Pop-ups values (Map Opacity and Gamma fixed by #661/#666) | playtest 2026-10-18 `playtest/6606044/options-tour-side-by-side.mp4` |
| #663 | P3 | Board pan to and from Options leaves out the Stats page and a sticky note | playtest 2026-10-18 `playtest/431cb74/options-tour-side-by-side.mp4` |

## Input

Rebirth scope: keyboard, controllers, mouse, remapping, fullscreen.

Port: configurable 24-action/two-column keyboard mapping, raw key selection,
duplicate clearing, completeness/Defaults and Back persistence. The Browser
captures focused physical keys and releases owned keys on focus loss. Actual
gameplay consumers use phase-aware action polling and physical bomb edges.
The main632-based artifact passes four fixed cases (2,904 fields), one fresh
unfiltered OS case (784 fields), runtime 2/2 and both default bomb phases with
370 rows unchanged against its parent. Fresh main173 artifact 4ca independently
passes runtime 2/2 and the fixed W→I plus recorded OS replays: 1,568 fields
unchanged from d10. Other d10 results retain their artifact scope.
The registered Controls gate on main0ad artifact `2a706930` passes all four fixed
routes plus unfiltered OS `M3MJ EY2Z`/epoch `670687023`: 3,688/3,688 fields,
302 trusted physical edges and zero Browser errors. All 83 fixed Browser PNG
bytes and 2,904 ordered field values match historical witnesses. Fresh shared
Options fixed/JWMM regression passes 866/866; raw Native identities/sequence
labels differ, with zero verdict changes and 32 identical Browser PNG bytes.
Controls RGBA 45/60 pass/fail, Options 16/16 and all history-cap failures remain.
Current capture-tool integration on producer `404a5617` replays the same seven
recorded cases with fresh Native processes: 4,554/4,554 fields pass, all 137
Browser PNG files match main0ad, and Options verdicts remain unchanged despite
50 raw pointer/sequence differences. The outer raw-index guard fails; its
independent diagnostic verifies unchanged staged entries, Source and inputs.
Both that failure and the RGBA/history failures remain preserved.
Missing: controllers, mouse, Native process reload, Browser OS-process
restart and remaining remapped action callbacks, including drop and restart. The configurable Map action has the
separate bounded scope above. Strict pixels and history caps remain red;
see [Controls scope](controls-keyboard.md).

Next: Native own-saved Keyboard process reload; drop (Ctrl) and restart (R); gamepad.

No open bugs.

## Seeds, RNG and shared MT

Rebirth scope: seed codec, special seeds, run/stage/room/item RNG streams and
the one global MT19937 every effect, sound and particle draws from.

Port: seed codec and entry, run/stage/descriptor streams, ItemPool and Player
child RNGs, and one shared MT owner for boot, menu, audio, tears and admitted
actors. Fresh Character admission also forwards its owned menu code through
the seed-effect lookup; the [scoped live owner comparison](run-admission.md#fresh-character-seed-effect-owner)
does not establish special-seed effects or later placement behavior.
Missing: special seeds, and many shared-MT consumers, so the port's
MT stream drifts from the original during play. Every missing consumer is a
P2 bug here even when its caller lives in another subsystem.

Next: investigate the first divergent source consumer on each retained route
(#162), preserving already matched draw prefixes.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #162 | P2 | Shared-MT routes still diverge after the merged admission/room consumer stack; the old 288-draw diagnosis is historical | e2e --mt-draws |
| #213 | P2 | Swaying/Web Init draws are implemented; stored/drawn appearance and live closure remain unverified | issue; f5849102 |
| #252 | P2 | Rock destruction debris particles draw (missing) | e2e |
| #417 | P2 | Large-room boundary constructs extra door GridBase objects (missing) | e2e |
| #459 | P2 | ChangeRoom tail footprint uses the wrong player velocity; candidate awaits live rebaseline | e2e --mt-draws |
| #592 | P2 | Pocket collection requests match but an OS case chooses a different sound sample | e2e pockets |

## Floor generation

Rebirth scope: floor graph growth for every stage, room shape and template
selection, special room placement, curses, XL floors, alternate stages.

Port: fresh normal Basement generation with authored weights, retries,
fitting and secret rooms, verified against 17 original floors. Basement II
and Caves I/II (stages 3 and 4) now have generation source, including
Double Trouble's rectangular boss rooms (no native Double Trouble or Caves II
layout graded yet). The separate Caves set 1 wall catalogue labels patches 0..2
instead of 3..5; its reviewed metadata correction
is unbuilt and live-unrun. This does not change wall quads or RNG order.
Missing: Cellar, Depths and later stages, Caves II's flagged-shop Greed swap,
curses (Labyrinth, Lost,
Darkness, Unknown, Maze, Blind), XL floors.

Next: Cellar generation and unlock; curses.

No open bugs.

## Rooms, doors and transitions

Rebirth scope: 1x1, tall, wide and large rectangles; every base room type;
door styles and locks; room transitions; minimap; room persistence.

Port: 1x1 rooms of types normal, shop, treasure, secret, super secret,
curse, library and sacrifice; tall (13x14), wide (26x7) and large (26x14)
normal rooms; boss and miniboss rooms when their boss is implemented;
reciprocal doors, key doors, curse-door damage, minimap and supported room
persistence. Missing: arcade, challenge, devil, angel, crawlspace and other
special rooms, coin and bomb locks, full actor reentry. The evidenced base
loader has four rectangle sizes; expansion contours are not missing-base goals.

Next: complete supported-room reentry; devil/angel door after a boss.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #147 | P1 | Doors into unsupported room types render open but never transition | playtest |
| #212 | P2 | Seed J8JF7M4M: start room has no top door and hides the bottom door | e2e |
| #450 | P3 | Doors into boss rooms use the normal frame instead of the boss frame (render part under *Rendering*) | playtest 2026-10-16 `playtest/2045007/` |

## Stage progression

Rebirth scope: boss clear award, trapdoor, stage transition animation and
name streak, chapter order, Womb/Utero to Sheol/Cathedral, final floors,
endings.

Port: boss clear award (with poop occupants), trapdoor, stage transition,
Basement I to Basement II and PR654 Caves I descent source. The [carried-active follow-up](stage-exit-evidence.md)
passes 48 mandatory identity/charge/reserve/ownership/descent checks across
two active routes on frozen f837, with an ordinary control. Double Trouble
seeds now generate Caves I; current arrival/catalogue qualification is separate. Missing: Caves II and
later steps, alternate stages, endings.

Next: coordinate later-stage admission with the level workstream (#413).

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #413 | P1 | Caves I descent source landed (#654), Double Trouble boss rooms place, Caves II generates behind a refused trapdoor; the selected-wall catalogue also needs correction | issue; stage-exit-evidence.md |
| #600 | P3 | Run-start "BASEMENT I" streak sits about 3 px lower than the original | playtest 2026-10-17 `playtest/7b74719/` menus-tour |
| #681 | P2 | A new floor's start room appears instantly; the original fades it in from a pixel mosaic over about 80 frames | playtest 2026-10-18 `playtest/540197a/caves-descent-side-by-side.mp4` |

## Player

Rebirth scope: movement, stats and the cache, health containers (red, soul,
black, eternal), damage and invulnerability, death, revival, transformations.

Port: normal Isaac f32 movement, phase-aware controls, cache evaluation for
the supported items, red/soul/black/eternal health, damage and immunity,
death animation. Missing: base transformations such as Guppy, revival
items, broader damage sources.

The Player Full-grid change moves
ordinary Full Player ground response after entity contacts. Its isolated 9841
candidate versus actual 5c33 gains 36 scalar passes with zero prior-pass losses;
the 40-gain prediction, 38 scalar failures, both Fast coverage guards and 238/266
RGBA failures remain red. Same-Native Shop and fixed declared-clock death
controls preserve their fields and actual Browser PNGs. [scoped source/evidence](player-death.md#player-full-grid-after-contacts)
does not establish complete physics or current-main acceptance.

Next: transformations once their items exist.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #245 | P2 | Seed PQE9YPQP diverges in player position, velocity and health from presentation 269 | e2e damage-fire-delay |
| #195 | P2 | Round Worm seed T1PK6Y0W: port player stops at counter 156 while the original keeps moving (triage?) | e2e roundworm |
| #451 | P3 | Death diary has an empty "killed by" picture and no Esc/Space hints (triage?) | playtest 2026-10-16 `playtest/2045007/` |

## Weapons and tears

Rebirth scope: tears and their flags, lasers (Brimstone, Technology), Mom's
Knife, Dr./Epic Fetus, Ludovico and every synergy between them.

Port: regular tears with source motion; Inner Eye, Spoon Bender, Number One,
Polyphemus, Soy Milk, Technology, Technology 2, Brimstone (ordinary and
controlled), Mom's Knife, Dr. Fetus, Epic Fetus, Ludovico, Monstro's Lung and
several of their pairwise synergies. Missing: most tear-flag items, most
synergies, tear death effects beyond the supported ones.

Next: remaining Chapter 1 pool weapons.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #191 | P2 | Port spawns a regular tear one presentation early after an item pickup | e2e stat-cache |

## Bombs

Rebirth scope: placement, fuse, blast, damage, craters, bomb items and
variants (Troll, Mega, Bobby, Butt, ...).

Port: placement, fuse, motion, blast damage to player, NPCs and grid, Troll
and Mega Troll conversion. Ordinary class5 Full ground response now follows
entity contacts and both phases clamp to actual Room bounds; [scoped #349
evidence](system-map.md#ordinary-bomb-late-grid-response-349) retains unresolved
center/near-wall NPC timing. Missing: bomb-modifying items, craters, Bomb Bum
and other variants.

No open bugs. Bomb contact bugs are under *Collision and physics*.

## Collision and physics

Rebirth scope: actor/actor contact and mass separation, actor/grid contact,
walls, knockback, pickups pushed by actors.

Port: player, NPC, bomb and pickup contacts against actors, grid and walls,
including large rooms.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #365 | P2 | Player-pushed chests sink into walls or open doorways; original behavior needs a live comparison before changing the constraint | DST; original comparison pending |
| #218 | P2 | Original leaves about 2 px of player/NPC overlap for one update; the port separates at once | e2e |
| #190 | P2 | Player/grid contact slides sub-unit amounts at rock edges in large rooms | e2e |

## Grid objects

Rebirth scope: rocks (normal, tinted, bomb, marked), pots, poop variants,
spikes, TNT, pits, blocks, keys blocks, trapdoors, crawlspaces, buttons.

Port: ordinary/tinted rocks and supported tinted rewards, authored bomb rocks,
spikes, poop (authored too), TNT signed fuse/countdown and nonrare Full blast,
spider webs, trapdoor, ordinary backdrop-1 Block initialization, decorations,
grid records and costs. The [Block constructor comparison](block-grid-evidence.md)
preserves its existing gameplay and pixel failures. Missing: broader pit/block
behavior, buttons, crawlspace ladders and poop variants.
The [bounded TNT comparison](tnt-grid-evidence.md) on frozen `e3fa5336`/1,408
files (WASM `e505236c`) passes 41,659 required checks on the unchanged recorded
fixed-plus-first-OS bank:21,038 fixed and 20,621 boot 4011211959/idle 8. All 22,535
prior passes remain identical, with zero lost rows or changed expectations;
19,113 added rows pass and 11 invalid live-four guards now use actual Penny
conservation. Earlier raw failures remain unchanged. Separate corrected-clock
ordinary-room Node and fresh-Native/DOM controls each pass 278 checkpoints,
99,406 inherited fields and 279 added clock rows, recovering 162 old marker
failures with no lost parent passes. Broader recorded11-rock/six-bomb/four-key
controls preserve all1,687,079 inherited field keys and all passing values;
8,811 added absolute-clock fields pass. The252 known Rock failures remain,
including64 already-failing tinted-position values changed against actual137b.
Transient18 seed/callback remains
UNOBSERVED; history caps, fullRGBA, rare50/41 pool/pickup350, mixed-grid order,
Player/NPC damage, reentry and whole shared MT remain open. No whole-game or
current-main qualification is inferred.


| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #485 | P2 | Retained tinted-rock/troll-bomb regressions remain red; the issue also retains separate Dip phase failures | e2e rock-blast, dips |
| #507 | P2 | Wide/double rooms lack the original's ambient decoration admission and associated draws | e2e entities |

## Enemies

Rebirth scope: about 140 ordinary enemy types and their champions across
all chapters.

Port: Frowning Gaper 10.0, Gaper 10.1, Gusher 11.0,
Pacer 11.1, Horf 12, Fly 13, Pooter 14, Attack Fly 18, Maggot 21, Charger 23,
Spitty 31, Fire Place 33, Level 2 Fly 214, Dip 217, Wall Hugger 218, Gurgling
237, Round Worm 244, and Shopkeeper 17. PR680 also implements Hopper 29.0,
Trite 29.1 and Flaming Hopper 54.0 through the shared Hopper family. The
existing `entities/hopper` and `entities/trite` drivers declare admission
coverage. Independent live qualification of these imports on pinned main
`ca408124` is **UNRUN**; this records source availability, not Original/Browser,
fire-conversion, shared-MT or pixel acceptance. Remaining enemy families and
champion behavior are incomplete. Bulk enemy imports retain their existing
owner (see AGENTS.md).

Horf's final dead Full update now runs its AI before retirement. Scoped
[regression evidence](npc-evidence.md#horfs-final-dead-update) preserves all
77,793 passing enemy rows; default base-kill effects, complete status handling
and direct final-shot/shared-MT qualification remain open.

[PR656](https://github.com/Anveio/isaac-rebirth-browser/pull/656) adds green
champion creep. Its current-main666-based frozen `35edb299` passes a fresh
Original/Browser fixed route: 173,135 entity fields, 419 paired checkpoints and
all seven lifecycle/Acid milestones on each side. Actual Native damage sources
identify live Green actors. Three retained-Native regressions preserve all
269,637 entity and 9,605 paired field keys with zero lost passes; both Fast
presentation mismatches improve. Fixed MT remains 196,875/196,875 exact and
the recorded OS case passes. Older routes retain their missing Acid witnesses;
full RGBA on the new route remains 7 pass/412 fail. Earlier Bomb/NPC controls
retain their separate scope (4,364 fields, 38 gains). The input-only replay
bank and default fixed fixture use the qualified public route. Longer-tail MT,
other champion hooks and whole-game acceptance remain open.

After integrating Gamma and the starting-trinket runtime, frozen `62300f05`
replays the same four Native cases with all 456,532 inherited entity/paired
fields and 196,875 fixed MT fields unchanged. All 134,916 Gaper-spawner rows
pass. The four older missing-Acid failures and all 1,390 pixel verdicts remain
unchanged (28 pass, 1,362 fail); this adds no fresh Native or longer-tail MT claim.

Next: the rest of the Basement/Cellar spawn table.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #241 | P1 | Red (attacking) Fire Place 33.1 is not admitted | e2e |
| #154 | P2 | Round Worm teleport positions diverge (Wall Hugger part now implemented) | e2e bomb-damage |
| #536 | P2 | Gusher wanders a different way from the original after the Gaper morph | playtest 2026-10-16 `playtest/354e7e8/` gusher-contrived-2 |
| #693 | P2 | Hoppers and Trites jump at different times and land in different places than the original | playtest 2026-10-18 `playtest/dc38bee/` hopper/trite side-by-sides |
| #705 | P2 | Mulligan and Mulligoon fights drift from the original within seconds (Mulligoon evade path, an extra half heart lost) | playtest 2026-10-18 `playtest/8b70d62/` mulligan/mulligoon side-by-sides |

## Bosses

Rebirth scope: about 50 bosses and minibosses, champion bosses, VS screen,
boss health bar, boss rush.

Port (8 families): Monstro 20, Larry Jr. 19, Grub 239, Gluttony 49,
The Duke of Flies 67, Gemini 79, Dingle 261 and Envy/Super Envy 51;
VS scene from boss portraits. Envy's new live cases remain red, and Super
Envy's short upper-door admission prefix passes 5,301 fields after the
[footprint counter fix](player-footprints.md), with 12 gains and no losses;
full fight, children and clear remain unqualified (#605). The bounded registered admission suite catches the historical parent bug; its unchanged recorded `HH936E8Z` OS case still fails78 initial Envy position/velocity rows on counter4029, with actual room/NPC admission passing. Fresh PR629 artifact f0eb retains those same78 failures and adds115/113 passing draw-order-presence rows without changing earlier fields; no overlapping order coverage is claimed. Missing: other Chapter 1 bosses
(Widow, Pin, Famine, Steven, Fistula, Lust and the other sins), later bosses
including Gurdy/Gurdy Jr., remaining champion variants and Super Gluttony.

A separately closed Source-only cohort retained all 8,192 OS draws before
floor evaluation. Independent Shop planning evaluated every draw and selected
0 of 2 required routes (6,378 unsupported template contexts, 1,807 missing
route rooms and seven unsupported enclosure geometries), so the cohort remains
FAIL. Independent Super Envy planning retained all 32,768 door-slot outcomes
and selected first eligible ordinal 3,497, `PHCVWE1C` (run seed 1714079471,
stage seed 2559733126), via the upper door from start room 6 to room 4.
Eligible ordinal 7,028 remains surplus and cannot replace a later failed
selected route. The short selected-route admission comparison is qualified
separately above; the full public-input fight and parity checks remain failed
or pending. The Source discovery cohort itself is not gameplay qualification.
No draw was retried, refilled or selected from an original-game outcome.

Next: remaining Chapter 1 bosses.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #174 | P1 | Super Gluttony 49.1 is not admitted (needs lob projectiles and its second laser) | issue |
| #408 | P2 | #374 changed the Monstro paired random route result | e2e monstro |

## Pickups and chests

Rebirth scope: coins, keys, bombs, hearts (all kinds), batteries, chests
(normal, Bomb, locked, Big and red), pickup variants and drop tables.

Port: natural coins, keys, bombs, hearts, supported normal/locked/Bomb Chest
opening, troll bombs, collection and persistence. Missing: batteries,
Big/red chest behavior and the room-clear reward table beyond the
supported cases. Source admission is not complete chest parity.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #177 | P2 | Historical Chest child/motion differences; #480 narrows the retained fixed cases to scatter direction | e2e chest-opening |
| #178 | P2 | Some rooms admit 2 of 3 natural bomb pickups (seed Q8CMHWHP) | e2e bomb-pickups |
| #480 | P2 | Three retained Chest cases match child identities/speed but scatter in different directions | e2e chest-opening |

## Items, pools and familiars

Rebirth scope: 213 passives, 75 actives, 55 familiars, 60 trinkets, 15 pools,
pedestals, rerolls, charge.

Port: complete pool configuration and selection; pedestal admission and
acquisition; active charge and use for Yum Heart; stat items such as Sad
Onion, Cricket's Head, Speed Ball, Roid Rage, Anti-Gravity, Belt, Libra;
Leech familiar; Gimpy and Infestation 2 Dip callbacks; the weapon items under
*Weapons and tears*. Missing: most items, trinkets, most familiars and
actives. Bulk item imports are owned by the user's import workflow.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #295 | P0 | Killing a Dip with Gimpy or Infestation 2 froze the game. Error path removed by #329; confirm and close | DST |

## Pocket items

Rebirth scope: 33 cards, 8 runes, 28 pill effects, identification, per-run
pill colours, drop and swap.

Port: partial. Public runtime owns Card/Pill Init, canonical chest/procedural
consumers, two held records and HUD, collection/refused-contact response,
capacity spill and held-pocket Save. The canonical Magic 8 Ball Card world
producer and Range Down/Up Pill use are implemented. The latter updates the
canonical counter, requests/evaluates Range, consumes/promotes held slots and
plays Sad/Happy; other effects and the complete ceremony remain incomplete.

Evidence: #586's frozen candidate passes 5,639 Original/public-WASM checks;
#602 persists `pill-range-use` and passes a 1,409-check retained-Original replay
on c224/ee04/Wfa54. Neither result is a whole-current-runtime, Browser pixel
or PCM qualification. The historical public-WASM Node AL regression passes 9,349 selected
functional checks against its independent Original capture (AL retains 12 unavailable obligations
and unreached SAME-v1 AD0 refresh). Separated fixed-phase0 and retained recorded-0
each pass 29,385 checks with zero failures and 11 unavailable obligations
(428c4301/c0022b06). Initial2 closed exit 0, so three of eleven cases are
complete and eight remain mandatory; DOM/rendering, audible backend
and PCM equality are unavailable in this Node result. Pocket coverage does
not qualify basic coin/key/bomb/chest scenarios.

Next: complete the remaining durable cases, 251/252 world/held conversions,
Battery90, actual equipped effects/identification and complete save/Continue
restoration, with real callback and audio evidence kept separate.

No open bugs.

## Special rooms and economy

Rebirth scope: shops and prices, devil and angel deals and chances,
sacrifice rooms, arcades (slots, fortune teller, blood bank, beggars),
libraries, challenge rooms, Isaac's bedroom, Boss Rush.

Port: treasure, secret, super secret, curse, library and sacrifice rooms
enter. [Normal Shop economy](shop-economy.md) has descriptor-owned stock,
real common SKU bodies, prices, paid grant/debit/copy and live reentry. The
frozen lifecycle artifact qualifies one actual funded Bomb purchase: fresh
v9 passes 117/117 source guards and 390/390 owned fields. The corrected
canonical view retains eight failures; 52/59 RGBA diagnostics fail. Earlier
v8 missing-overlap evidence stays red. First-four source-authored common-SKU
fixtures on frozen5c33 establish free no-active refusal, affordable priced
no-active refusal, an undercharged Battery grant and a paid collectible
queue/deferred grant. All64 summary witness checks pass; the strict aggregate
41,212/41,481 retains250 health and19 BF4 guard failures. Raw reports and
representation/actual-owner readbacks remain separate; RGBA/PCM is ungraded.
The registered `common-sku` replay now preserves all 1,749 free-no-active rows
and, on frozencc38, passes 29 additional actual seed-effect owner checks.
The remaining fixed cases and default OS approach are not newly qualified.
These are fixture witnesses, not whole retail-floor or current-main parity.

Next: promote source-only local fixture recipes and owned-schema coverage;
qualify free-full/free-undercharged/paid-full Battery and other paid SKU
callbacks; implement
canonical Store Credit/Steam Sale and eligible achievements. Upgraded
shops, unadmitted ABD contexts, charge/effect48, Continue, broader Full/Fast grid parity,
complete audio/pixels, devil/angel chance and rooms, slots, beggars and
challenge rooms remain separate.

No open bugs.

## Characters

Rebirth scope: Isaac, Magdalene, Cain, Judas, ???, Eve, Samson, Azazel,
Lazarus, Eden, The Lost, plus transformed Lazarus and Judas.

Port: normal Isaac. The carousel shows the roster but other characters are
rejected at start. Character imports are owned by the user's import workflow.

No open bugs.

## Rendering, animation and camera

Rebirth scope: ANM2 sprites, backdrops, lighting and shadows, effects,
camera, HUD, minimap, giantbook, nightmares, cutscenes.

Port: ANM2 playback, backdrops and decoration, GPU light and shadow masks,
camera for large rooms, HUD for health and inventory, minimap, screen shake.
Missing: cutscenes, giantbook, nightmares, many effects; whole-frame pixel
parity is a diagnostic only.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #173 | P2 | Laser beam render geometry is approximated | issue |
| #268 | P3 | Tracking: where the full-frame pixel error comes from | e2e |
| #670 | P3 | Enemy spawn smoke puffs draw light tan and opaque; the original's are dark grey and see-through | playtest 2026-10-18 `playtest/6606044/gaper-contrived-1-side-by-side.mp4` |
| #450 | P3 | Boss/nightmare appearance still needs live closure; boss blood and wall source have landed (#458/#483) | playtest 2026-10-16 `playtest/9ff507f/` |
| #535 | P2 | Historical Gaper gib/bone gap; #573 adds gib source, but the reported scene still needs confirmation | playtest 2026-10-16 `playtest/354e7e8/` gusher-contrived-2 |

## Audio

Rebirth scope: 169 sound effects, 45 music definitions, jingles, layers,
voiceovers.

Port: SFX with source admission, Title, run and Basement music, fades and
queues. Missing: other floor music, boss and jingle layers, voiceovers.
Shared-MT sound draws are tracked under *Seeds, RNG and shared MT*.

No open bugs.

## Saves and metagame

Rebirth scope: save and continue, profiles, achievements (178), unlocks,
completion marks, challenges (20), Eden tokens, win streaks.

Port: in-run SavedGame records, origin-local initial-room Save/Continue,
natural reload and owned Tiny Bug/Fly/Wall Bug saved records through fresh Init.
The successor137b also restores the physical/fast boundary and Stop Room's
Backdrop seed constructor. The unchanged fixed/OS Original inputs retain all1714
inherited rows:1644 pass/70 fail,52 gains and zero lost passes versus0af. All14
OS regressions on the intermediate adfb candidate are recovered; its failures
remain preserved. Additional MT/ambient rows pass2006/2034; all48 measured
shared cursors and624-word states match. The28 added failures are fixed-case
pre-Quit ambient velocity words; OS507/507 passes. OwnSave66/66, both reloads
and real legacy missing-ambient-owner refusal pass; RGBA remains34 pass/28 fail.
The70 inherited failures remain14 canonical menu/gameplay-scope errors,
28 seed separator rows and28 last-penalty-stage rows. Other saved rooms/items/
actors, complete progression/unlocks, broader RNG/PCM and challenges beyond
Challenge1 ingress remain open. Historical1b30/0af122 failures and listed-Bomb/
legacy-refusal evidence retain their exact scope. PR696 is merged at
`9f952efd`: complete saves retain the 4 MiB limit without a 4,096 walking-paint
cutoff; capacity refusal acknowledges once, keeps older bytes and resumes the
Tick. Exact main715 baseline/candidate build and runtime 4/4 each pass. Four
same-Native Save controls preserve 3,748 rows per role (3,650 pass / 98 fail),
zero lost passes and pixel metrics; the 98 failures include 28 ambient velocity
words, 14 canonical scope, 28 seed separator and 28 penalty-stage rows.
Historical Source C capacity and failed wrappers retain their separate scopes.
Fresh public-main134 H/tooling artifact `7ddc18cc` separately passes build/ABI
and runtime 4/4. Its one fixed same-Native Save capture keeps all 2,384 ordered
rows exactly equal to dbfab H: 2,349 pass / 35 fail, zero lost/missing/new or
changed Native expectations, 33 own-save guards and unchanged pixels. Actual
natural reload/Continue passes, with stored seed9/live seed8 and penalty0; the
observed movement follows the valid room5-to-room6 door transition. The suite
still fails its 28 ambient velocity and seven canonical scope rows. No new134
baseline, OS or ordinary replay is claimed; its actual canonical affected
preview selects 52 suites. Publication on later main3b08711d preserves Mega Fatty and unrelated
quality tools without inheriting a later-main gameplay grade.

H's separate snapshotdbfab qualification uses fresh baseline `9e3c05e5` and
shared H/tooling artifact `70cb3285`: fresh build/ABI and runtime 4/4 each pass. The
candidate's actual affected preview selects 52 suites. The same retained fixed Native case improves 2,321 pass /
63 fail to 2,349 pass / 35 fail: 14 seed-text and 14 penalty-stage gains, zero
lost/missing/new rows or changed Native expectations, unchanged pixels and
33 own-save guards per role. The remaining 35 are 28 ambient velocity words
and seven canonical scope rows; both comparison exits remain failed. Both
ordinary-run replays pass on the same retained Native case, including the
candidate's exact nine-character saved-seed guard. Both fixed natural reloads
pass; their fresh boot RNG/epochs differ, and unchanged placement permits the
observed different valid doors. This is no identical-boot counterfactual or
Original-process reload parity. Historical main715 H retains its separately
qualified fixed/OS 56 gains and 42 failures. Raw archive audit corrects its
earlier ordinary/reload-unqualified notes; the old notes remain preserved.
No main134 integration, Factory/Get or whole-game qualification is inherited.
See [Save scope and replay](save-continue.md).

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #499 | P2 | PR503 removes the active-ID stage-save refusal; the bounded Lemon Mishap descent and recorded OS approach now pass 48 actual carried-word/descent checks on frozen f837. Random-held-active-item fuzz remains owed; later stages, other active effects and pixels remain separate | [scoped evidence](stage-exit-evidence.md); e2e stage-exit |

## Co-op

Rebirth scope: co-op babies (203 definitions), join and leave, shared hearts.

Port: none.

No open bugs.

## Test routes and tooling

Bugs in a test route or tool rather than in the game.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #196 | P3 | blood-start contrived route never reaches the boss in the original; regenerate it | e2e blood-start |
| #220 | P3 | health-damage black-heart route never acquires the Black Heart in the original; regenerate it | e2e health-damage |
| #603 | P3 | Complete backend snapshots repeatedly serialize large room/stain state and approach evidence budgets | e2e stage-exit |

## Recently fixed

- #650 draws the controls only in the first floor's start room; playtest 2026-10-17 `playtest/78cacac/` shows Basement II's start room plain after Monstro, matching the original (#449 closed).
- #629 draws Isaac and NPCs in the original's depth order, with blocking `render.npcsOverPlayer` checks in the entities suite; playtest 2026-10-17 `playtest/de63a72/` shows the Duke covering Isaac.
- #438 resolves the source-derived admission/death clock mismatch; [scoped clock evidence](player-death.md) retains the fresh OS cohort's required Original audio-observation failure, pixel reds and incomplete audio acceptance.
- [PR615](https://github.com/Anveio/isaac-rebirth-browser/pull/615) restores the fresh Title/frame0 and black fade on GameOver Exit; clock and audio qualification remain separate.

- [PR614](https://github.com/Anveio/isaac-rebirth-browser/pull/614) resolves #349 ordinary-bomb ground response after NPC contacts and Room clamping; [scoped qualification](system-map.md#ordinary-bomb-late-grid-response-349) preserves unresolved Hugger timing and existing regression failures.

- #612 restores dead Player Base carried motion and ground response (#493): the scoped four-case replay fixes517 Y differences, with all unrelated clock/Exit failures preserved.

- #527 completes Flags32 for items 48/222; the three recorded regular-fire regression cases return to zero failures (#372 closed).
- Shared-MT light-pass, ambience-loop and pickup DropSound source consumers have landed (#194, #256, #416 closed); other first-divergence routes remain open.
- #598 pans the menu board between pages and animates the carousel; playtest 2026-10-17 `playtest/7b74719/` menus tour (with Challenges) is pixel-exact on 1781 of 1782 menu frames (#587, #588, #589 closed).
- #329 removed the Dip-death freeze path with Gimpy/Infestation 2 (#295 awaiting confirmation).
- #385 destroys a poop occupying the boss clear award cell; playtest 2026-10-16 killed Larry Jr. on WMAJ8HGV without a freeze (#383 closed).
- #473 selects each boss's authored VS splash and names; footage of four bosses matches (#448 closed).
- #551 admits authored poop, bomb rocks and TNT; playtest 2026-10-16 `playtest/b2f08ac/` shows Dingle's and the Gusher room's poops (#447 closed).

The bounded [Gamma slice](options-gamma.md) implements the f32 callback, percentage, final RGB pass and V4 Back/reload. Fixed plus first-unfiltered OS inputs pass 1,376 functional checks and 26 required percentage regions; full-frame diagnostics remain 10/40 exact. Historical preference versions and shared regression evidence keep their own contracts.
