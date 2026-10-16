# Game roadmap

The whole base Rebirth game, split into subsystems, with where the port stands
on each and every outstanding bug filed under the subsystem that owns it.
Afterbirth and Afterbirth+ are out of scope.

This file is the planning view. Evidence lives elsewhere and is not repeated
here: [system-map.md](system-map.md) holds original-game evidence and
acceptance records, [TEST_MATRIX.md](../TEST_MATRIX.md) holds executable
coverage, [dst.md](dst.md) holds DST findings, and GitHub issues hold bug
detail. A row here is a pointer and a status, never a parity claim.

Last full review: 2026-10-16, main `8c9d68b`, 31 open issues.

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
| **M0 Basement run** | Play normal Isaac from the title through Basement I and II to the boss and trapdoor without a freeze | Current focus. Blocked by #413. |
| **M1 Chapter 1** | Basement I/II and Cellar with every Chapter 1 enemy, boss, room type and common item | All Chapter 1 bosses; shops, devil/angel rooms, Cellar unlock |
| **M2 Chapters 2-4** | Caves/Catacombs, Depths/Necropolis, Womb/Utero, Mom and Mom's Heart | Other stage generators, their enemies and bosses |
| **M3 Endgame** | Sheol/Cathedral, Dark Room/Chest, Satan, Isaac, ???, The Lamb, endings | Final floors, final bosses, ending cutscenes |
| **M4 Full catalogue** | All 343 collectibles, 60 trinkets, cards, runes, pills, all characters | Item and character imports |
| **M5 Metagame** | Saves and continue, unlocks, achievements, challenges, co-op | Persistent profile in the browser |

## Subsystem overview

Counts of open bugs are as of the last full review.

| Subsystem | State | Port today | Open bugs |
| --- | --- | --- | ---: |
| [Boot, menus and pause](#boot-menus-and-pause) | Partial | Title, profile, main, character and seed menus; pause | 1 |
| [Input](#input) | Partial | Keyboard only | 0 |
| [Seeds, RNG and shared MT](#seeds-rng-and-shared-mt) | Partial | Seed codec, run/stage/item streams; shared MT incomplete | 7 |
| [Floor generation](#floor-generation) | Partial | Normal Basement I/II generator | 0 |
| [Rooms, doors and transitions](#rooms-doors-and-transitions) | Partial | 1x1 and four large shapes; eight room types | 2 |
| [Stage progression](#stage-progression) | Partial | Boss clear award, trapdoor, Basement I to II | 2 |
| [Player](#player) | Partial | Normal Isaac movement, stats cache, health, death | 2 |
| [Weapons and tears](#weapons-and-tears) | Partial | Tears plus a set of weapon items and synergies | 2 |
| [Bombs](#bombs) | Partial | Placement, fuse, blast, Troll/Mega conversion | 0 |
| [Collision and physics](#collision-and-physics) | Partial | Player, actor, grid and pickup contacts | 4 |
| [Grid objects](#grid-objects) | Partial | Rocks, bomb rocks, spikes, poop, TNT, webs, trapdoor, decorations | 0 |
| [Enemies](#enemies) | Partial | 18 ordinary NPC types, including the Gaper family | 3 |
| [Bosses](#bosses) | Partial | 7 bosses and minibosses | 2 |
| [Pickups and chests](#pickups-and-chests) | Partial | Coins, keys, bombs, hearts, chests | 2 |
| [Items, pools and familiars](#items-pools-and-familiars) | Partial | Pools complete; about two dozen items with behaviour | 1 |
| [Pocket items](#pocket-items) | Partial | Card/Pill Init, held slots/HUD, collection/spill and held Save; effects incomplete | 0 |
| [Special rooms and economy](#special-rooms-and-economy) | Partial | Treasure, secret, curse rooms; shop room without sales | 0 |
| [Characters](#characters) | Partial | Normal Isaac only | 0 |
| [Rendering, animation and camera](#rendering-animation-and-camera) | Partial | ANM2, backdrops, lighting masks, camera, HUD | 4 |
| [Audio](#audio) | Partial | SFX and floor music | 0 |
| [Saves and metagame](#saves-and-metagame) | Missing | In-run save records only | 0 |
| [Co-op](#co-op) | Missing | None | 0 |
| [Test routes and tooling](#test-routes-and-tooling) | n/a | Live e2e, DST, playtest | 2 |

## Boot, menus and pause

Rebirth scope: intro cutscene, title, profile select, main menu, character
carousel, seed entry, challenge menu, options, stats, pause menu, death and
win screens.

Port: Title, profile, main, character and seed menus run with source timing;
run introduction banner; pause menu; Challenge1 XML ingress. Missing: intro
cutscene (the paired tapes skip it), options, stats, bestiary, challenge list,
win screen.

Next: options menu; death screen parity; intro cutscene.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #168 | P2 | Escape during a room transition pauses in the port; the original refuses | e2e paired-random-run |

## Input

Rebirth scope: keyboard, controllers, mouse, remapping, fullscreen.

Port: keyboard actions through the native action map, with phase-aware
polling. Missing: controllers, mouse, remapping, drop/map/restart actions.

Next: drop (Ctrl), map (Tab) and restart (R) actions; gamepad.

No open bugs.

## Seeds, RNG and shared MT

Rebirth scope: seed codec, special seeds, run/stage/room/item RNG streams and
the one global MT19937 every effect, sound and particle draws from.

Port: seed codec and entry, run/stage/descriptor streams, ItemPool and Player
child RNGs, and one shared MT owner for boot, menu, audio, tears and admitted
actors. Missing: special seeds, and many shared-MT consumers, so the port's
MT stream drifts from the original during play. Every missing consumer is a
P2 bug here even when its caller lives in another subsystem.

Next: close the run-start gap (#162), then add consumers in the order the
paired runs hit them.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #162 | P2 | Port is about 288 draws behind at run start and does not advance while walking | e2e |
| #194 | P2 | Room light pass draws per fireplace light every presented frame (missing) | e2e |
| #213 | P2 | "Swaying" grid object Init draws frame, flip and rotation (missing) | e2e |
| #252 | P2 | Rock destruction debris particles draw (missing) | e2e |
| #256 | P2 | Insect-swarm ambience loop starts with a sample draw (missing) | e2e |
| #416 | P2 | Sound sample draw during the Duke and Grub fights (missing) | e2e |
| #417 | P2 | Large-room boundary constructs extra door GridBase objects (missing) | e2e |

## Floor generation

Rebirth scope: floor graph growth for every stage, room shape and template
selection, special room placement, curses, XL floors, alternate stages.

Port: fresh normal Basement generation with authored weights, retries,
fitting and secret rooms, verified against 17 original floors. Basement II
generates. Missing: Cellar and every later stage, curses (Labyrinth, Lost,
Darkness, Unknown, Maze, Blind), XL floors.

Next: Cellar generation and unlock; curses.

No open bugs.

## Rooms, doors and transitions

Rebirth scope: 1x1, tall, wide, large and L-shaped rooms; every room type;
door styles and locks; room transitions; minimap; room persistence.

Port: 1x1 rooms of types normal, shop, treasure, secret, super secret,
curse, library and sacrifice; tall (13x14), wide (26x7) and large (26x14)
normal rooms; boss and miniboss rooms when their boss is implemented;
reciprocal doors, key doors, curse-door damage, minimap and supported room
persistence. Missing: narrow and L-shaped rooms, arcade, challenge, devil,
angel, crawlspace and other special rooms, coin and bomb locks, full actor
reentry.

Next: narrow (1x1 thin) shapes; devil/angel door after a boss.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #147 | P1 | Doors into unsupported room types render open but never transition | playtest |
| #212 | P2 | Seed J8JF7M4M: start room has no top door and hides the bottom door | e2e |
| #450 | P3 | Doors into boss rooms use the normal frame instead of the boss frame (render part under *Rendering*) | playtest 2026-10-16 `playtest/2045007/` |

## Stage progression

Rebirth scope: boss clear award, trapdoor, stage transition animation and
name streak, chapter order, Womb/Utero to Sheol/Cathedral, final floors,
endings.

Port: boss clear award (with poop occupants), trapdoor, stage transition and
the Basement I to Basement II step. Missing: every step after Basement II,
alternate stages, endings.

Next: Basement II to Caves (#413 first).

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #413 | P0 | Basement II boss trapdoor fails every tick; next stage only admits 1 to 2 | playtest |
| #449 | P3 | No "BASEMENT I" name streak at run start; Basement II start room keeps the controls drawing | playtest 2026-10-16 `playtest/2045007/` |

## Player

Rebirth scope: movement, stats and the cache, health containers (red, soul,
black, eternal), damage and invulnerability, death, revival, transformations.

Port: normal Isaac f32 movement, phase-aware controls, cache evaluation for
the supported items, red/soul/black/eternal health, damage and immunity,
death animation. Missing: transformations (Guppy, Beelzebub, ...), revival
items, broader damage sources.

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
| #372 | P1 | Regression from #287: recorded pedestal route stops firing regular tears | e2e damage-fire-delay |
| #191 | P2 | Port spawns a regular tear one presentation early after an item pickup | e2e stat-cache |

## Bombs

Rebirth scope: placement, fuse, blast, damage, craters, bomb items and
variants (Troll, Mega, Bobby, Butt, ...).

Port: placement, fuse, motion, blast damage to player, NPCs and grid, Troll
and Mega Troll conversion. Missing: bomb-modifying items, craters, Bomb Bum
and other variants.

No open bugs. Bomb contact bugs are under *Collision and physics*.

## Collision and physics

Rebirth scope: actor/actor contact and mass separation, actor/grid contact,
walls, knockback, pickups pushed by actors.

Port: player, NPC, bomb and pickup contacts against actors, grid and walls,
including large rooms.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #349 | P2 | Bombs skip the post-contact grid pass, so an NPC can push one into a wall | DST |
| #365 | P2 | Walking a chest into a wall sinks it 10.8 px into the wall cell | DST |
| #218 | P2 | Original leaves about 2 px of player/NPC overlap for one update; the port separates at once | e2e |
| #190 | P2 | Player/grid contact slides sub-unit amounts at rock edges in large rooms | e2e |

## Grid objects

Rebirth scope: rocks (normal, tinted, bomb, marked), pots, poop variants,
spikes, TNT, pits, blocks, keys blocks, trapdoors, crawlspaces, buttons.

Port: rocks, authored bomb rocks, spikes, poop (authored too), TNT, spider
webs, trapdoor, decorations, grid records and costs. Missing: tinted rocks,
pits, blocks, buttons, crawlspace ladders, poop variants.

No open bugs.

## Enemies

Rebirth scope: about 140 ordinary enemy types and their champions across
all chapters.

Port (18 ordinary types): Frowning Gaper 10.0, Gaper 10.1, Gusher 11.0,
Pacer 11.1, Horf 12, Fly 13, Pooter 14, Attack Fly 18, Maggot 21, Charger 23,
Spitty 31, Fire Place 33, Level 2 Fly 214, Dip 217, Wall Hugger 218, Gurgling
237, Round Worm 244, and Shopkeeper 17. Missing: every other enemy,
champions, Chapter 2+ enemies. Bulk enemy imports are owned by
the user's import workflow (see AGENTS.md).

Next: the rest of the Basement/Cellar spawn table.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #241 | P1 | Red (attacking) Fire Place 33.1 is not admitted | e2e |
| #154 | P2 | Round Worm teleport positions diverge (Wall Hugger part now implemented) | e2e bomb-damage |
| #536 | P2 | Gusher wanders a different way from the original after the Gaper morph | playtest 2026-10-16 `playtest/354e7e8/` gusher-contrived-2 |

## Bosses

Rebirth scope: about 50 bosses and minibosses, champion bosses, VS screen,
boss health bar, boss rush.

Port (7): Monstro 20, Larry Jr. 19, Grub 239, Gluttony 49,
The Duke of Flies 67, Gemini 79, Dingle 261; VS scene from boss portraits.
Missing: every other Chapter 1 boss (Gurdy, Gurdy Jr., Widow, Pin, Famine,
Little Horn, Steven, Fistula, Lust and the other sins), all later bosses,
champion variants, Super Gluttony.

Next: remaining Chapter 1 bosses.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #174 | P1 | Super Gluttony 49.1 is not admitted (needs lob projectiles and its second laser) | issue |
| #408 | P2 | #374 changed the Monstro paired random route result | e2e monstro |

## Pickups and chests

Rebirth scope: coins, keys, bombs, hearts (all kinds), batteries, chests
(normal, gold, red, spiked, mimic), sacks, pickup variants and drop tables.

Port: natural coins, keys, bombs, hearts, chest opening, troll bombs,
collection and persistence. Missing: batteries, sacks, gold/red/spiked
chests, room-clear reward table beyond the supported cases.

| Bug | Sev | Symptom | Source |
| --- | --- | --- | --- |
| #177 | P2 | Chest loot child actors and their motion differ from the original | e2e chest-opening |
| #178 | P2 | Some rooms admit 2 of 3 natural bomb pickups (seed Q8CMHWHP) | e2e bomb-pickups |

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
producer is public; selected scalar/RNG/Card endpoints pass on the current
qualified artifact. This does not implement general Card/Pill effects.

Evidence: the current public-WASM Node AL regression passes 9,349 selected
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
enter; shop room and Shopkeeper enter but nothing is for sale. Missing:
prices and purchases, devil/angel chance and rooms, slots and beggars,
challenge rooms.

Next: shop purchases; devil/angel chance after the boss.

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
| #450 | P3 | Boss rooms lack the blood floor scatter (walls fixed by #483); nightmare lacks the floor progress bar | playtest 2026-10-16 `playtest/9ff507f/` |
| #535 | P2 | Killed Gapers leave flat blood decals without the gib and bone pile | playtest 2026-10-16 `playtest/354e7e8/` gusher-contrived-2 |

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

Port: in-run SavedGame records and the no-achievements seeded badge. Missing:
durable browser profile, continue, unlocks, achievements, challenges beyond
Challenge1 ingress.

No open bugs.

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

## Recently fixed

- #329 removed the Dip-death freeze path with Gimpy/Infestation 2 (#295 awaiting confirmation).
- #385 destroys a poop occupying the boss clear award cell; playtest 2026-10-16 killed Larry Jr. on WMAJ8HGV without a freeze (#383 closed).
- #473 selects each boss's authored VS splash and names; footage of four bosses matches (#448 closed).
- #551 admits authored poop, bomb rocks and TNT; playtest 2026-10-16 `playtest/b2f08ac/` shows Dingle's and the Gusher room's poops (#447 closed).
