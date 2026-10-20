# Antithesis deterministic simulation testing

`engine/dst/` is the engine's deterministic simulation testing (DST) harness,
built on the [Antithesis Rust SDK](https://antithesis.com/docs/using_antithesis/sdk/rust/).
It is the **only** checked-in unit-level verification in this repository: the
standing rule against Rust/Node/Python unit suites and component harnesses
(see [AGENTS.md](../AGENTS.md)) has exactly this one owner-authorized
exception. Nothing else may use it as precedent.

DST establishes engine safety, determinism, native/WASM agreement and
documented input contracts. It
is not original-game evidence: parity with base Rebirth remains the job of the
live e2e suites, and no DST result may be cited as a parity result. It is an
inner-loop regression tool, not a merge gate: neither `check:build` nor any
pre-merge check runs it. Run it while changing engine core code.

```sh
npm run test:dst                                  # build engine.wasm + harness, run every unit
DST_CASES=48 DST_STEPS=4000 npm run test:dst      # a longer local soak
npm run test:dst -- --seed 0x1f2e3d4c5b6a7988     # rerun a whole recorded run
engine/dst/target/release/rebirth-dst --units game --case-seed 7233388093878974408 --steps 5000
engine/dst/target/release/rebirth-dst --replay .cache/dst/run-…/tapes/game-…-violation.json
engine/dst/target/release/rebirth-dst --list      # the unit registry
npm run lint:dst                                  # rustfmt + strict clippy for the harness
```

The harness keeps its own `Cargo.lock`, so the shipped engine's lockfile never
names the SDK. `tools/dst/sync-lock.py` runs before every harness build
(`test:dst`, `lint:dst`). It pins each package of the engine's dependency
closure to the version `engine/Cargo.lock` ships, so DST simulates the engine
that ships and an engine dependency change never breaks `--locked` harness
builds. When it changes the lock it says so: commit the lock with your change.

## How it works

Every simulated decision draws from one entropy source
([entropy.rs](../engine/dst/src/entropy.rs)). Locally it is a seeded SplitMix64
stream: each run prints its master seed and each case its case seed, so any
case reproduces exactly. With `--entropy antithesis` every draw calls
`antithesis_sdk::random::get_random()`, letting the platform steer and replay
exploration. Units never read clocks, OS randomness or the SDK directly.

The `game` unit's sampled checks (a batch re-run as single ticks, a snapshot
round trip, a stale save acknowledgement, the continuation's midpoint) leave
the simulation alone, so they draw from the state they check instead: a
stream seeded by the digest of the state before the command and the command's
index. A retained tape therefore replays exactly the checks its case ran, with
no field `replay.rs` would refuse.

Properties ([props.rs](../engine/dst/src/props.rs)) are Antithesis assertions:
`always`, `sometimes`, `reachable` and `unreachable`. Each call forwards to the SDK macro of
the same kind, so a platform run reports every property natively. Because the
SDK never fails a process by itself, each call site also records into a local
ledger, and the run's verdict applies Antithesis semantics to the SDK's own
registration catalog: an `always` must be hit and never false, a `sometimes`
must be true at least once (the workload really reached that branch), a
`reachable` must be hit and an `unreachable` never. Unmet must-hit properties
fail a local run; under
Antithesis they are judged across the whole exploration instead. Property
messages start with their unit name and are the property identities; keep them
stable.

Seeded cases are independent, so they run on `--jobs` worker threads (all
cores by default). A run's report does not depend on the thread count: each
thread counts its own violations, the census tracks each thread's case
separately, and a property reports the failure from the earliest unit, case
and step. `--jobs 1` and `--jobs 4` write byte-identical reports for one
seed. Under Antithesis the harness drives a single thread, so the platform
orders every decision.

An `always` that aggregates unrelated failures signs each one: the Tick
property by the error the engine returned, the walls property by the escaped
body's kind and family (NPC type, pickup variant). The property fails exactly
as an unsigned one does, and the report also lists every signature with its
count and earliest failure (`signatures` in `report.json`, indented under the
property on stderr; the first failure of each new signature prints too). So a
known open finding cannot hide a new failure of the same property: compare a
run's signatures with the Findings table rather than its first failure.

A panic anywhere is an `unreachable` violation with its message attached. The
harness profile unwinds and enables overflow checks and debug assertions, so
the engine runs under debug arithmetic semantics while simulated.

## Units

[The registry](../engine/dst/src/units/mod.rs) lists every unit; the
[test matrix](../TEST_MATRIX.md#antithesis-deterministic-simulation-units)
indexes each one's declared properties from source.

| Unit | What it simulates | Oracles |
| --- | --- | --- |
| `seed_codec` | public seed codes and the run-seed xorshift handoff | inverse round trips, canonical acceptance of arbitrary glyph strings, mutation aliasing, nonzero derived seeds |
| `mt` | MT19937, descriptor seed tables, boot lifecycle | the C++ standard's mt19937 reference outputs; mid-stream JSON continuation; an eager 120,960-word table model of the lazy descriptor table; seed draw accounting |
| `phase` | update/interpolate/skip phases | a counter model; two player and one nonplayer update per two-phase window regardless of interpolation; exhaustion without mutation |
| `actors` | actor admission order and room identities | an independent list model under random admit/retain/reset |
| `layout` | rectangular floor growth (`engine/src/layout.rs`) over every accepted seed, room count and shape mask | the grid marks exactly each room's rectangle; growth starts from the centre 1x1 room; every room has a shape its mask makes available and grows from an earlier room one step farther from the start; requested rooms are added unless the queue runs dry; the reported draw count replays the final RNG35 state through `items::Rng`; dead ends are leaves; expansion stops at five dead ends or five attempts; rejections; JSON round trips |
| `room_spawns` | authored spawn-group selection (`room_spawns::select`, Room Init 0x5cf190) over every authored Basement and special room and synthetic rooms with zero, fractional, invalid and overflowing weights, empty and misplaced groups and padded-capacity grids | admission matches the source's room and per-group checks exactly; every group with entries yields one selector, in authored order, holding one of its entries; the pick is 0x5d49d0's strict running-sum comparison against the group draw, a weighted entry wins unless the first entry is the fallback, and a lone weighted entry always wins; the tinted-rock candidate is an interior cell; the RNG11 room stream advances once per group (empty ones included) and entity spawn seeds take one RNG35 step, both replayed through `items::Rng`; cells and positions; purity; JSON round trips. Ties where the scaled draw exactly equals a running sum are too rare to separate `>` from `>=` |
| `room_templates` | the authored room-template picker (`TemplatePicker`, 5d5878) over the Basement and special catalogues and synthetic ones, through long request sequences whose picks decay the chosen weight | a model of the current weights and an independent compatibility filter: a picker is built exactly from valid catalogues; the pick satisfies its request (type, shape, variant and difficulty ranges, required doors); the exact door-mask pool is used only past the 9.99-scaled gate and only for exact masks; the pick is the strict running-sum choice from the gated pool; only the picked weight changes, to a tenth; the reported pools and RNG7 draws (replayed through `items::Rng`) match; a request is refused exactly when malformed, weightless, or when the threshold rounds up to its pool's whole weight (subnormal decayed weights included); two pickers fed the same requests answer alike |
| `floor` | Basement generation and room admission | determinism; footprints; reciprocal edge-adjacent neighbors; reachability; exactly one boss/super secret/shop/treasure (the source retry rule); retry continuity; every descriptor admits as a `Room`, and so does a uniformly chosen compatible authored room swapped into each descriptor (same catalogue, type and shape, doors available, stage difficulty), which admits about half again as many distinct authored rooms per run |
| `motion` | player motion (`motion::advance_world`, source 529801..529990) over arbitrary positions, velocities, inputs and speed stats, and 600-step held-input runs followed by release | an f64 model that splits velocity along and across the unit input direction (the source projects onto the input scaled by the speed stat): a step runs on digital input, finite vectors and an admitted speed stat, and never returns a non-finite vector (outside that domain it may be refused); position advances by the previous velocity; release keeps the forward retention bit for bit; held input damps drift across it by both retentions and accelerates along it, braking harder against it; speed contracts toward the terminal speed, every held step moves the velocity geometrically closer to the terminal velocity, and a settled player moves at its speed stat's terminal speed along axes and diagonals alike; a release glides the geometric sum of its velocity; every step commutes exactly with the square's eight symmetries; purity. The model is semantic: f32 rounding order (the source's separately rounded transverse steps) stays the live suites' to pin |
| `tears` | regular tear kinematics (`projectiles.rs`): the velocity a shot stores from its aim and the player's motion (51bce6..51bd71), its eye origin, damage-derived scale (50d090, 541ff0), collision radius, animation tier (5418e0) and render offset, and whole flights from FireTear's birth through random update, intermediate and skipped phases and collisions | f64 models: each kernel runs on its admitted inputs and returns finite values (outside them it may refuse); along a unit aim a shot keeps the shot speed times ten plus 1.2 of the player's forward motion and never motion against the aim, across it 1.2 of the sideways motion; an eye sits across the shot, 0.3 to 0.5 of the velocity from the player, each eye on its own side; scale is 0.6 + 0.04·damage + 0.15·√damage; the tier counts the thresholds (+0.05 in binary32) a scale exceeds up to the thirteenth clip, so the table's last entry (2.75) is never consulted; scale, tier and render offset never fall as their inputs rise; the radius is the authored radius times the scale. In flight a lifetime only moves forward, the tear keeps the velocity and falling acceleration it was born with, a skipped phase or removed tear changes nothing, an intermediate phase presents half a step ahead and moves nothing, an update moves one step and falls until the tear lands (above -5 height) or collides, a dying tear is removed at the next update, a step fails only on position overflow and leaves the tear unchanged, and a tear whose falling speed settles positive is removed within 20,000 updates |
| `bombs` | regular planted bombs (`bombs.rs`): whole lives from DropBomb's birth through random update, intermediate and skipped phases to removal, with clocks corrupted now and then; the placement cooldown; Isaac's blast envelope and knockback | a bomb is born from finite vectors at the player with a tenth of its velocity and a full fuse, and its birth is presented in any admitted phase; it detonates once on its 60th full update and is removed ten updates later; its lifetime only moves forward; updates move it by its velocity and slow it by 0.95, halving it below unit speed, so it only ever slows; intermediate phases present it half a step ahead and move nothing; skipped phases and removed bombs change nothing; a bomb whose clocks keep the source contract runs, and a refused bomb is left unchanged; placement is admitted only once the cooldown is spent, only full updates spend it, and it re-arms after exactly 30; a blast reaches a player strictly inside its radius plus Isaac's 10 (f64, away from the f32 edge), keeps a centred player's velocity, and otherwise halves the velocity and pushes straight away from the bomb by the remaining depth, at most 15 |
| `pickups` | regular pickup subtypes (`pickups::regular_subtype`, Pickup.Init's RNG35 rolls 500150, 4ffcae, 4ffd4e) and the pickups each roll spawns as | a nonzero subtype is kept; a zero one is rolled for hearts, coins, keys and bombs from a nonzero seed (other variants are left open), into the family's documented set, and every member (soul, eternal and black hearts, nickels, dimes, golden keys, double, troll and megatroll bombs) is rolled; every roll spawns as a pickup with its Appear or Idle clip that keeps its subtype, the transient troll bombs (40/3, 40/5) drawing the regular bomb's visual; purity |
| `grid` | grid queries and both ground-collision solvers on generated rooms | resolution never fails for bounded motion (including residual speeds whose squares underflow); previews keep velocity; an underflowing inward preview skips refinement (source 443117 → 44346c); the nonplayer solver starts from the centre clamped to the room (442c2d) and, without contact, changes nothing else; correction moves only out along the contact normal; full nonplayer reflections leave the wall; blast candidate ordering and bounds |
| `contact` | the mass contact responses (`combat.rs`, source 445710) for player–NPC, player–pickup, player–projectile, tear–NPC and NPC or bomb–NPC pairs, over arbitrary radii, masses, speeds and separations | computed in f64 from the inputs: the response runs exactly for strictly overlapping circles; rejected and non-overlapping pairs stay bit-for-bit unchanged; positions and velocities change only along the centre line (allowing for the f32 cancellation the source's fast-separating scaling shares); separation keeps the pair's centre of mass in place; the impulse conserves momentum, the player taking half its share, and carries the relative speed split by mass; an ordinary pair ends exactly a radius sum apart; a blocked player keeps its motion and the collider is offered exactly the displacement applied; an NPC pair resolves the same whichever body comes first. The fast-separating branch's overlap scaling (1 + 2·dot) is statically recovered and checked only through these conservation laws |
| `pathfinder` | FindGridPath's flood (`pathfinder::flood_costs`, 5b498f) over room-shaped and arbitrary cost grids | an independent node-weighted Dijkstra: every label is the cell's shortest distance whatever order heap ties take; only open cells of the grid change; the owner is visited exactly when it lies within half the room's cells; without it the flood visits exactly the cells nearer than that cutoff, with it every cell nearer than the expansion that visits the owner and none farther, and that expansion runs to its end; purity; grids outside admission are rejected |
| `item_pools` | the run-owned ItemPool manager (`docs/item-pools.md`) | selection never offers a collectible twice (Breakfast aside), comes from its pool, the treasure fallback, Breakfast or the Bible, and advances none of the manager's streams; depletion only lowers weights and fills the removed/selected maps; a zero-counter counterfactual isolates Rosary's Bible override and the candidate it marks; drains reach Breakfast within the catalog; locked collectibles survive every redraw; error rooms route with one shared draw and special rooms to their namesake pools; JSON continuation; rejections without mutation |
| `game` | the whole engine through public commands, booted with or without the settings and input configuration the browser stored | per-step state invariants, per-command transition contracts (menus including Options, Controls and Keyboard bindings, the own-save protocol and Continue, run-start hold, pause, boss scenes, stage descents, death and GameOver), boot with stored settings, a harness's declared boot clock (#660, `set_boot_elapsed`: exactly the declared time on a fresh reset clock, refused unchanged with a sub-millisecond part of 1 ms or more, once the clock moved or once a Tick ran, and carried as a tape's `bootElapsed` through replay, the C ABI and the WASM; `coverage-boot-elapsed` boots the live suites' clock, so every run meets them), rejection of malformed inputs, tape replay, serialized-midpoint continuation, the C ABI, the shipped WASM |
| `rooms` | every room `traversal::admits` on a floor a seeded run's trapdoors reach, through each trapdoor's own entry check and a door transition, and played by the game driver (after stage/rooms' room-entry scan); each boss room's fight then ends, there and on the floors of ten more runs a case descends for their bosses alone | no Tick fails and nothing panics, signed by the engine's error; a stage transition a trapdoor starts arrives within 1,500 full updates; a boss room is cleared once its enemies keep taking lethal hits, whatever its shape. A refused trapdoor or door is skipped and a player who walks back out has played the room, so no port gap is pinned |
| `abi` | the exported C ABI the browser bridge drives | save documents bounded at 4 MiB, save IO declared only with protocol 1 at a fresh Title and documents loaded only there, corrupt documents refused keeping the game; framing bounds, malformed requests, failed resets, pre-reset commands on a fresh thread, the published key surface; a boot from the authored challenge XML and special-room STB through `reset_with_boot_resources` publishes and plays exactly like the plain boot, and truncated, unknown-field, oversized or corrupted resources fail with an error that keeps the previous game; the settings resets boot with their stored settings, publish the same state from the authored resources, and refuse presence flags other than zero or one and stored volumes or Map opacity outside the unit interval, keeping the previous game; the input-configuration reset boots with the stored configuration and the same state from the authored resources, and refuses more than 32 retained devices, duplicate or unprintable names, Keyboard bindings outside GLFW's keys and unknown request fields, keeping the previous game; the Gamma settings reset boots with every stored setting and the same state from the authored resources, boots what the input-configuration reset does at Gamma 1, and refuses a missing, null or out-of-range Gamma and unknown fields, keeping the previous game; `set_boot_elapsed` succeeds right after reset and fails with an error, keeping the game, before any reset, with a sub-millisecond part of 1 ms or more, once the clock moved or once a Tick ran |

The `game` driver ([driver.rs](../engine/dst/src/units/game/driver.rs)) only
emits commands the browser bridge can send, plus adversarial variants of them
(near misses of public keys, malformed seed text, out-of-range volumes,
over-budget ticks, stale music callbacks, key repeats and releases). A Tick of
exactly the 36,000-step command budget is sent only while paused or in the
menus: in play it would simulate ten minutes in one command, repeated by every
replay and the interpreted WASM differential. Each case
follows one persona (direct play, menu explorer or seeded run), sends
browser-style host-timestamped frames (with jitter, hitches and rollbacks) or
controlled steps, and steers by the published state. The menu explorer
also opens Main's Challenge and Options menus. Options, Controls and
Keyboard act on keys held at a tick (the released System poll), so
there the driver holds its presses into ticks, sometimes long enough to
reach Options' hold repeat. In the Keyboard menu it binds keys, including
its own play keys, so bindings go missing and back is refused until it
resets them. It names its play keys by their default binding and presses
each through the key now bound to its action, so a case that leaves with
custom bindings, or boots with them, plays on with them (rebound cases
reach about as many census keys as default ones). The menus' contracts:
* an Options tick steps only the setting on the selected row (SFX, music,
  Map opacity or Gamma), Gamma stays within .5..1.5, and leaving Options
  saves the settings it leaves with;
* bindings and the stored input configuration change only in the Keyboard
  menu, which binds the awaited key to the selected action alone (clearing
  it from every other configurable action) or resets the defaults;
* the Keyboard menu leaves only with every configurable action bound, and
  then saves every retained device with the bindings it leaves with.

Half the generated cases run with the browser's own-save host
(`engine/src/save_host.rs`, modelling `web/save-games.ts` and the save IO
loop in `web/main.ts`). After every command it answers the writes and menu
readmissions the command left pending, before any later input sees the
state. It stores the document a write exports unless that revision's storage
fails, and answers a readmission with the stored document. Without the host
a readmission finds no document, so Continue stays unavailable. A third of
the host's cases boot as a returning player's browser: a previous session,
a fresh game the same driver plays for 150 to 1,500 commands with a sound
host, leaves the document it stored, and the case boots with it. Another
third boot after a previous session that went deeper than generated play
reaches: it starts a seeded run with a sound host and descends as the
`rooms` unit does (an open trapdoor under the player, whose own entry check
decides each descent) until a trapdoor refuses. Each arrival's level
admission saves the run, and the case boots with the document stored on one
of the floors reached, chosen uniformly (`saved run on stage <n>` in the
census). The tape holds only that document and public commands, so its
import, export and readmission run through the WASM differential too.
Continue refuses these runs today: it admits only a run saved in Basement
I's starting room (`save_profiles`: `Continue has no owned load caller for
this saved run yet`), and a refused Continue leaves the state unchanged.
Once it admits deeper runs, the same cases resume Basement II and the Caves
under every `game` property and the WASM differential, with no change here.
Generated runs never reach past Basement II by play: a staged chase from
the active-item descent's Basement II arrival (3 of 8 hearts) met that
floor's boss in none of 256 branches, 63 of 64 dying first. While the
profile holds a saved run, the driver often moves from New Run down to
Continue, and resumes the run about half the times it selects Continue. The
contracts see each command together with the host's answers:
* the host's answers always succeed, and a Tick a write or readmission
  suspended advances its full count once answered;
* an acknowledgement for another revision, or for IO nothing requested, is
  refused without change (probed whenever IO is pending);
* the document a game would store imports into a fresh Title and exports
  again byte for byte;
* a document a previous session stored loads at the next session's Title,
  whatever its seeds and settings, and exports again byte for byte (a
  replayed tape's document is checked only when it loads, since it may
  predate a save format the engine now refuses);
* Continue (54ECE0) resumes the saved record's character, difficulty,
  challenge, seeds, stage and floor, whether this session saved the run or
  a previous one.

In a room it plays before it leaves:
* It fights what is alive, lining up on one axis to fire along it. It keeps a
  wider range against a boss and never stands still against one, but strafes
  only to align, escape a corner or dodge: tears inherit 1.2× the player's
  velocity across their aim (`projectiles::stored_velocity`), so a strafe
  bends every shot off the line. It first steps away from the nearest threat
  (a hostile NPC, where it will be in ten updates, or a boss's published
  target such as Monstro's landing cell), still firing.
* Movement and fire keys stay held from one macro to the next, as a player's
  would; a macro presses and releases only the keys that change, so commands
  buy game time instead of re-pressing held keys.
* It then collects every pickup the engine marks `can_collect`: pedestal
  items, hearts, chests and consumables.
* It also demolishes blocking rocks with bombs, pauses, uses active and pocket
  items and answers the GameOver popup.

A seeded case first scouts its seed. On copies of the booted game it opens
the seed dialog through the same public commands, starts runs for 96
candidate codes, and asks the engine's own pedestal rule
(`Pickup::from_collectible_selector`) what each floor's treasure room offers.
It types a code whose collectible requests the weapon cache (authored
`cacheFlags` 0x100: Brimstone, Technology, Mom's Knife, the fetuses and the
rest of `weapon_plan::resolve_primary`) and heads for the treasure room, so
the weapon plans and their attacks run under DST in ordinary budgets.

Between rooms it follows door-graph routes toward the boss room or the nearest
unvisited room. Routes cross only the rooms the engine's own
`traversal::admits` lets the player enter, so newly admitted rooms join the
workload without a driver change. A key opens a `KeyClosed` door on contact,
and the driver bombs `Hidden` doors to reach secret rooms.

Boss fights decide how long a tape must be to reach a clear. With key holds,
aligned fire and threat evasion, cases branched from a Monstro room entry
clear it in 7 of 32 runs, and from a Gluttony entry in 8 of 32. On the
shared-MT engine, `tools/dst/chase.py` branched 64 or 128 cases from each
boss room entry:
- Monstro died in 11 of 64.
- Larry Jr. lost half its segments in 46 of 64, and the deaths spawn kind 21.
- Gemini's main body died in 2 of 128, which retires its cord.
- Dingle still wins every fight it starts (128 of 128).
- The Gurglings (237.1), met in a boss room by `coverage-gurglings-entry`,
  survived all 256 branches on ff24cc6, though 6 to 10 of each 64 brought the
  boss below half health.

On c8931a8, with Gapers admitted, Monstro died in 1 of 64 branches. On
4543c59, with poop, TNT and webs admitted, Duke of Flies' room was cleared in
1 of 64 and Gemini's main body died in 1 of 128. On a1972be, after the
door-entry critter change, Duke's room was cleared in 1 of 64, Monstro died
in 1 of 192 and Larry Jr. lost both segment kinds in 47 of 64.

Every report carries a coverage census: the room types, NPC kinds, pickups and
game features (owned collectibles, secret reveals, boss clears) each case
reached, including each owned weapon collectible (`weapon:<id>`), each
floor stage played (`stage:<n>`), each stage's boss and miniboss rooms
entered and cleared (`room:5 on stage 2`, `cleared:5 on stage 2`), a boss
room entered, a boss room cleared, each larger boss room shape cleared
(`cleared:5(26x7)`, beside `room:5(26x7)` for entering it)
and a descent made while an active item is held (`boss room holding an
active item`, `boss room cleared holding an active item`, `descent holding
an active item`), a run started after a descent (`new
run after a descent`, and `new run from the menus after a descent` when it
went through the menus), a run started from Random (`Random run`) and its
GameOver popup's Exit to the menus (`menus after a Random run's GameOver`),
a fire cooldown left above a newly lowered fire delay (`fire cooldown above
the max fire delay`), an Options setting held past its hold-repeat threshold
(`Options hold repeat`), and each property met (`property:<message>`).
`rooms` cases count too (`roomsReached`), so a kind only a later floor's
rooms place, such as Mega Fatty (264) in a Caves I boss room, shows as
simulated once the unit plays it; `reached` and `firstSteps` hold only
`game` cases, the ones with tapes for `cover.py` and `chase.py` to cut. A
census entry is a measurement, never a requirement. The summary
line names the implemented NPC kinds no case simulated
(from `rebirth_cleanroom::implemented_npc_kinds`), the room types that
generated floors contained but no case entered, and the boss rooms, by
shape, that a case entered and none cleared (`unclearedBossRooms`). Unsimulated kinds are split
by whether any authored Basement, Caves or special room places them
(`unplacedNpcKinds`): a kind no room places, such as one from a later stage,
is reachable only when another entity spawns it, so it is not a driver gap.
A `removed:<kind>` key records an NPC of that kind leaving a room the player
stayed in, killed or retired by its family; removed bodies are pruned within
the update that removes them, so the census watches for the absence rather
than the death frame. The summary also names the simulated kinds whose
removal never ran (`unremovedNpcKinds`), the death and retirement paths no
case exercised. A Fireplace (33) goes out rather than leaving (its hp never
falls below one, and only an extinguished fire on a solid cell retires), so
`extinguished:33` counts as its death. Two kinds stay on the list for now:
- Shopkeepers (17) have no ported death yet.
- Wall Huggers (218) are invincible in the source (`wall_hugger.rs`). The census is a measurement,
not a property. Must-hit properties and coverage tapes make the important
branches binding.

Beyond its invariants, every `game` tape is checked by:

* **Replay determinism.** The tape replays on a fresh boot and must reproduce
  every command outcome and state digest.
* **Snapshot completeness.** The game serialized at a random midpoint and
  resumed must reach the identical final game, so no simulation state lives
  outside `Game`'s serialization.
* **Lossless publication.** Published states and whole games round-trip
  through JSON and the engine's own deserialization validators.
* **Batched ticks.** A `Tick` of up to 32 steps is re-run as that many single
  ticks when sampled, and always when it changes screens or waits on save IO,
  and must reach the identical game. Only command-level
  publication may differ: the batch publishes every step's outputs in order
  (what `State::begin_command` clears: sound and music events, SFX volume
  callbacks, settings and input-configuration saves, and unavailable-action
  reports), including those before a step that returned to the menus, and a
  screen change resets the menu
  animation counter at the command boundary. The same engine method defines
  which outputs a key ignored during a boss scene or the run-start fade may
  clear, so a new output cannot drift out of either check.
* **C ABI differential.** The tape drives the exported reset the browser picks
  for its stored settings (`reset`, `reset_with_audio_preferences`,
  `reset_with_fullscreen_preferences`, `reset_with_map_preferences_v2`, or,
  after `request_ptr` receives the stored settings and input configuration,
  `reset_with_settings_preferences` when a Gamma is stored and
  `reset_with_input_configs` otherwise), then
  `request_ptr`/`command`/`snapshot_*`, exactly as `web/engine.ts` does,
  answering save IO through `enable_saved_game_io`, `save_data_ptr`,
  `complete_saved_game_write` and `complete_save_menu_readmission`. The
  shipped-WASM differential answers it through the same exports.
* **Shipped-WASM differential.** With `--wasm web/generated/engine.wasm` (the
  default in `npm run test:dst`) each tape also runs through the production
  artifact in the deterministic `wasmi` interpreter; every outcome, the
  published key surface and sampled snapshots must equal native dispatch.
  A differing snapshot is reported with the JSON pointer of its first
  differing value and both sides' values. Native-only properties therefore
  speak for the artifact browsers run. The differential is only as fresh as
  the WASM it loads: `npm run test:dst` rebuilds it, but a hand-built one
  must come from its own target directory. `engine/build.rs` names its
  inputs to `cargo:rerun-if-changed` by absolute path, so two worktrees
  building into one `CARGO_TARGET_DIR` can reuse each other's generated
  catalogs. A WASM built at c7a2547 from such a directory kept c9f478f's clip
  catalog and failed with `gfx/207.000_fatty.anm2 has no authored clip
  WalkHori` where native passed. A file-age check would not catch it, since
  the stale catalog sits inside a freshly linked module. Give each worktree
  its own target directory.

### Rooms

Ordinary play reaches only the rooms one route crosses, and a later stage
only after its boss, so the `rooms` unit enters every room a floor admits,
after stage/rooms' native room-entry scan. A case starts a seeded run
through the public menus (the seed scout's walk) and plays past the start
hold. It then descends: on each floor it stands the player on an open
trapdoor (state 1, its open clip finished), so the trapdoor's own entry
check (`advance_trapdoor`) decides the descent, and the stage transition
runs NextStage, the nightmare scene and the next level's admission as in
play. The chain ends at the first trapdoor that refuses, so every stage the
port admits joins without a change to the unit. The case plays every
floor it reached, from one it picks uniformly, so every reached floor is
played whatever the case count, and a case seed replays its first floor
exactly as when a case played only that one. On each floor it enters each
room `traversal::admits` through the transition a door starts (`roomTransition`
out of the neighbor behind the target's first door, through that
neighbor's door to it, as walking in does), and the game driver plays it
for 300 to 900 full updates. The starting room stays the one the engine
persists, and admission reads only the target and its door, so the
starting room need not neighbor the target.

Then each boss room's fight ends. Once the driver's pause menu closes and
its transition ends, every enemy alive takes a lethal hit through its own
damage queue each full update, the room-wide way Black Heart's effect lands
(523eaa), so each boss dies by its own rules, splits and spawns included,
and the room's clear and its award run. The room must clear within 1,500
full updates unless the player died, left or quit the run first. A tall or
wide boss room (13x14, 26x7) is a few floors in a hundred, so each case also
descends ten more runs and fights only their floors' bosses. No shape is
named, so none the port admits is pinned; the further runs make an ordinary
run reach each shape the floors place. In 200 cases on 197d870a with #774, 13x14 rooms
were cleared in 55 cases and 26x7 rooms in 46 with six further runs; with
ten, each of eight 12-case runs cleared both shapes, in 2 to 7 of its
cases. Without #774 every such 12-case run fails `rooms: no Tick fails`
10 to 13 times (`boss clear award admits only 1x1 boss rooms`), the halt
#750 shipped. The fights also kill every boss the descent reaches: of the
kinds `rooms` cases simulated, only 17, 49, 51, 50 and 218 now never die
there.

A trapdoor that does not take the player within 120 full updates refused
the descent, which is not a violation; a transition it starts must arrive
within 1,500, and a Tick that fails on the way is a violation like any
other. A room `traversal::admits` refuses is skipped, as its door would
refuse it, and a player who walks back out of the room has played it.
Neither refusal is required, so no port gap is pinned. After every command
the unit also evaluates the `game` unit's state invariants
(`invariants::state`) inside `props::always_only`: their `always`
properties record as they do in a game case, while a `sometimes` or
`reachable` there neither records nor emits, so the game unit's must-hits
still measure only its own workload. Every Caves family the descent
reaches (Chub, Gurdy, Mega Fatty, Boil, Wall Creep and the rest) thereby
runs under the 24 state invariants the game unit checks after every step:
walls, finite motion, health and consumable bounds, identities, the actor
schedule, room bookkeeping and the rest. 200 rooms cases evaluated them
44.6 million times, about 223,000 per case. A report lists every property a
unit it ran evaluated, so a game invariant a rooms-only run fails is never
hidden by the unit selection. A case reproduces from its seed:
`rebirth-dst --units rooms --case-seed SEED`.

### Tapes and the corpus

Game tapes use the engine's replay format (`initialSeed`, `bootEpochSeed`,
the optional `bootAudioPreferences`, `bootFullscreenPreferences`,
`bootInputConfigs`, `bootMapPreferences` and `bootGammaPreferences` a
generated case stores half, a third, a third, a third and a third of the
time, the `saveHost` half of them run with (`stored`, the document
the browser boots with, and `failedWrites`, the revisions whose storage
fails), and `commands`). A stored input configuration retains other
devices' arbitrary bindings and usually the Keyboard's: the defaults,
second keys no default uses, or every configurable action rebound to a
distinct key. So
a finding replays through the browser's `game.reset(...)` with the same
seeds and settings plus `game.command(...)`, or through
`engine/src/bin/replay.rs` once its rejected commands are removed (rejections
leave state unchanged). A case that violates any property retains its tape
under the run's output directory. Every tape in
[engine/dst/corpus](../engine/dst/corpus) replays through the full `game`
property set on every run, after the generated cases:

* `regression-*` tapes reproduce fixed findings and fail without their fix.
* `coverage-*` tapes are generated cases that together reach every `game`
  must-hit property (boss-scene key handling, GameOver, collection, boss and
  miniboss clears and other rare branches), so ordinary budgets never pass or
  fail on luck. The clear tapes were branched with `--resume` from boss and
  miniboss room entries. Other coverage tapes run implemented NPC families
  that ordinary budgets seldom meet, so their AI runs through every property
  and the shipped WASM on every run. Others enter the room types every floor
  generates but players seldom reach (the key-locked shop, the bombed-open
  super secret room) or retire NPC kinds whose deaths ordinary runs never
  see. `coverage-*-carried` tapes keep what a rebuilt tape's predecessor met
  on the way and its recipe does not name. Game tapes drift as the engine
  evolves;
  a regression tape that stops reproducing its finding is retired rather than
  kept under its label.

Every report's census records, per case, the command at which the case first
reached each key (`coverage.firstSteps`). `tools/dst/cover.py` turns that into
corpus upkeep. Harvest with `--retain-all`, measure the corpus alone
(`--units game --cases 0 --corpus engine/dst/corpus --report …`), and the
tool cuts the shortest harvested prefixes, running 300 commands past each
first contact, that cover the keys the corpus misses:

```sh
engine/dst/target/release/rebirth-dst --units game --cases 64 --steps 4000 \
  --retain-all --tapes H/tapes --report H/report.json
engine/dst/target/release/rebirth-dst --units game --cases 0 \
  --corpus engine/dst/corpus --report B.json
python3 tools/dst/cover.py --harvest H/report.json --tapes H/tapes \
  --baseline B.json --prefix npc: --out engine/dst/corpus --verify
```

`--verify` replays each written tape, drops any that no longer reaches its
keys and prints the properties a written tape fails, which make it a finding
rather than coverage. When an engine change moves a must-hit coverage tape
off its branch, the same harvest refreshes it. The census keys every property a case meets
as `property:<message>`: an `always` it evaluated, or a `sometimes` or
`reachable` that held. So `--prefix property:` cuts tapes for the must-hits
the corpus stopped meeting.

A harvest only adds tapes, and each was cut for keys the corpus lacked when
it was written; later tapes often reach them too. So after a harvest, census
the corpus alone again and prune it:

```sh
python3 tools/dst/cover.py --prune --baseline B2.json --out engine/dst/corpus
```

`--prune` deletes, longest first, every tape `cover.py` wrote
(`coverage-<case seed>` and `coverage-*-carried`) whose keys the rest of the
corpus reaches, so the corpus keeps every key in fewer commands. Regression,
recipe and other named tapes stay. It refuses a census that does not measure
the corpus's current tapes. The 87ddf06 harvest added six tapes that reached
ten keys the corpus lacked: Skinny (226) and its death, the deaths of
Crazy Long Legs, Squirt, Charger and Spitty (207, 220, 23, 31), a locked chest
(60), a Lil' Battery (90), and Technology and Cursed Eye (68,
316). With them, every key of 29 older harvested tapes and both carried
tapes had another tape reaching it, so `--prune` retired those 31: the
corpus went from 61 tapes and 103,088 commands on 87ddf06 to 38 tapes and
84,483 commands, losing no key. Chocolate Milk
(69) was not carried then: its one reaching case collected it at command
7,897 of 8,000, and that tape's replay with the WASM differential takes
943 s by itself (97 s natively), against 1,231 s for the whole 63-tape
corpus on the four cores here. A 07aca74f soak case collected it at command
886, so `coverage-4e56a759026d97a0` (1,187 commands, 84 s with the WASM
differential) carries it.

`--resume TAPE` makes every generated `game` case replay that tape (with
every check) and then drive on from its end, so exploration branches from an
interesting state, as Antithesis does, instead of rediscovering it. Cut a
harvested tape at, say, a boss room entry, resume many cases from it, and the
census shows how often each branch clears the room; a clearing branch is a
short coverage tape.

`tools/dst/chase.py` chains those branches for scenarios that need several
rare events in a row. Each `--stage` names census keys. Every stage branches
64 cases from the prefix so far and keeps the cases that reached its keys,
earliest first, each cut just past them. The next stage branches from the
earliest cut, and a round that reaches nothing moves the next round to the
next-earliest: a boss entry no branch survives (Monstro's first entry on
dc38bee, 0 kills in 256) is one dead end among the stage's hits, not the
end of the chase. The final tape is written only if its replay reaches
every stage.

Every scenario tape built this way keeps its recipe in
[engine/dst/chases.json](../engine/dst/chases.json): the tape and its stages.
The first stage is the scenario's precondition, such as `room:5` or `npc:20`
before a boss kill. A chase starts from the recipe's own tape, cut just after
its replay first reaches that stage, so the cut follows the tape through
every rebuild instead of naming a command. When drift moves the tape off its
precondition (main's Gaper admission moved the descent tape out of Duke's
room), the chase starts from a fresh boot instead. These tapes are the
descent, active-item descent, restart, Random-exit, Number One cooldown and
Envy split-death regressions, the Monstro, Larry Jr., Grub and Gemini
kills, the miniboss clear, the Dingle, Gurglings and super secret room
entries and the Options menu (`"from": null` chases from a fresh boot). A
recipe whose tape does not exist yet chases from a fresh boot, so a new
scenario needs only its recipe. The miniboss clear had been a harvested tape
with no recipe, which a menu change sent into Options before any run; its
recipe met Gluttony's room in 2 of 64 fresh boots on da2d591 and cleared it
in 11 of 64 branches from there. Few generated cases reach
Options at the fifth Main row, so `coverage-options-menu` holds a setting
past the hold repeat, binds a key in the Keyboard menu, is refused leaving
it with an action unbound, leaves it with its bindings saved and leaves
Options through its save on every run. `coverage-stored-settings` boots
with stored audio, fullscreen and input settings and plays into a run, so
that boot runs on every run too. Larry Jr. and the Grub (239) each have
their own recipe: a shared one needed the run to meet both, which only an
older tape's floor did, and a rebuild from a fresh boot met no Grub. Chargers
live in a room chain generated floors seldom place, so no fresh-boot branch
met one in 192. The Grub's death spawns two to five Chargers, Spitties or
Maggots (4f2e56), so the Grub recipe's kill stage also needs a Charger
(`npc:23`): on 7cfedf9 the Grub's room was entered in 1 of 64 fresh boots
and killed in 2 of 64 branches from there, its death spawning a Charger, a
Spitty and a Maggot. The separate Charger entry, which branched from the
Grub tape's entry and met no Charger in 64 branches on 7cfedf9, is
retired. A findings
row gives its regression tape's length, command positions and failure
counts as recorded; a rebuild keeps the scenario its recipe names, not
those figures, which describe the tape that showed the fault.
Boss fights and descents are the first coverage a shared-MT change moves, so
one command rebuilds all of them:

```sh
python3 tools/dst/chase.py --recipes engine/dst/chases.json   # --only NAME for one
```

A rebuilt tape reaches its recipe's keys, not everything the tape it replaces
met on the way. The 40469d3 rebuild dropped Squirt (220) and a chest (pickup
50), which the old Grub tape met before its Grub, and Crazy Long Legs (207),
which the old Larry Jr. tape met. No other tape ran them, and only the next
corpus census showed it. So a recipe run measures the corpus alone before and after
(`corpus-before.json` and `corpus-after.json` in its work directory) and
hands every key it lost to `cover.py`. A corpus census and that corpus's tapes
serve as a harvest: `cover.py` cuts the replaced tapes' shortest prefixes that
reach the lost keys and writes them as `coverage-<old name>-carried.json`.
On 87ddf06 the pre-rebuild corpus carried the three keys back as
`coverage-grub-kill-carried` (3,013 commands: Squirt and the chest) and
`coverage-larry-segments-carried` (1,212: Crazy Long Legs), until the same
health check's harvest reached them too and `--prune` retired both. A
carried tape has no recipe; when drift moves it off its keys, the census
lists them and a harvest replaces it.

`cover.py` skips keys that every reaching case met before its
first command, such as the seed scout's must-hits: they come from case
generation, and no tape carries them.

Open findings, which fail on current main and wait on their owner, keep their
tape in [engine/dst/findings](../engine/dst/findings) instead. `test:dst` does
not replay that directory, so one unfixed finding does not turn every run red,
but `--replay engine/dst/findings/NAME.json` reproduces it exactly. The fix
moves the tape into the corpus as a `regression-*` tape.

## Findings

A property that fails is a finding. DST shows that the engine misbehaves; the
fix must follow source evidence rather than merely silence the property.

| Finding | Resolution | Guard |
| --- | --- | --- |
| Confirming a locked or unimplemented character queued a run whose start failed inside the transactional tick, so every later tick failed and the browser's frame loop stopped. | Fixed on main (#120): character confirmation validates the selection and rejects the input with state unchanged. | `regression-locked-character-start` |
| Dip217 room actors (#127) are NPC collision class 4, but the NPC-pair and tear contact responses rejected type 217, so any room whose Dips touch (authored Basement rooms place them 40 px apart) failed every tick and stopped the browser's frame loop. | Fixed on main (#146): Dips take the normal-class `49b8f0` callback and `445710` mass response recorded in the Dip source contract. | `contact: every admitted pair at ordinary scales resolves` takes its NPC kinds from the engine's registry and fails 234 times with the #146 refusal planted back. `regression-dip-contact-freeze` was retired once the shared MT moved its Dips apart for the second time |
| A Random run stores its resolved character in the menu's selected character. Since the GameOver popup can exit to the menus (#111), the Character menu then highlighted Random while confirming started the previous run's character, and the menu invariant failed. Latent while only Isaac is implemented. | The Exit path reloads the menus' character from the profile's selection, as Profile → Main already does. | `regression-random-run-exit-menu-character` (252 commands) dies on a Random run and takes the GameOver Exit at command 221; without the fix it fails the menu invariant 31 times. By 7f2f3b4 the shared MT had moved it to the pause menu's Quit, a different path to the menus, so its `chases.json` recipe now chases the census key `menus after a Random run's GameOver` from the Random run's start |
| A bomb coasting into a wall with a ~1e-26 residual velocity underflowed its f32 squared speed to zero in an interpolation preview, which the reflection guard reported as overflow, failing the tick. | Fixed on main (#118), following the native early return: a zero-speed preview skips refinement. An earlier guard-only fix kept the tick alive but continued into refinement, unlike the original. | `grid: nonplayer ground resolution never fails for bounded motion` fails without any fix; `grid: an underflowing inward preview skips refinement` goes unmet under the guard-only fix. |
| Native std float functions call the host C library, while wasm32 std uses Rust's `libm`. Against `libm`, glibc 2.39 disagreed on 162,416 of 10,405,090 sampled `f32` `sin`/`cos` inputs and on 446,514 of 10,000,000 sampled `f64` `atan2` pairs. Generated tapes diverged from the shipped WASM by one ulp of NPC motion; the corpus tape's first difference is an Attack Fly's wander velocity (`npcs.rs`) at step 749, so native replay and tools could disagree with the artifact. | [fmath.rs](../engine/src/fmath.rs) routes the engine's `sin`, `cos` and `atan2` through the `libm` crate. wasm32 std matched it on every sampled input under Rust 1.97 and 1.99, so the shipped artifact computes the same results while native builds now agree with it. [engine/clippy.toml](../engine/clippy.toml) disallows every std float function whose precision Rust leaves unspecified, so `npm run lint` flags new host calls; a newly needed function gets a `libm` wrapper in `fmath.rs`. This makes no original-game parity claim. | The shipped-WASM differential on every tape and the lint rule. The original `regression-native-wasm-fly-trig`, which the shared MT moved off its branch, stays in the corpus as an ordinary workload (09cc3d1) and passes with host trig; with host trig restored in `fmath.rs`, `regression-new-run-resets-stage` fails the differential |
| Engine validation, the browser and DST each kept their own copy of the accepted key list, so new keys drifted. | Fixed on main (#122): `rebirth_cleanroom::input::PUBLIC_KEYS` is the single owner, published through the `public_keys` exports. | `abi: the public_keys export publishes exactly the engine's key surface`. |
| Main reproduces the original's 0/0 boss bar fill during a boss's first two frames (83112e8): the fill is NaN, published as `null`. `BossBar` kept a derived equality, and NaN never equals itself, so whole-state equality stopped being reflexive while the bar held NaN. Every "unchanged" or "same game" comparison then failed: lossless round trips, ignored boss-scene keys, replay and midpoint continuation. Ordinary runs on main reached that window after the grid changes of #224 and #225. | `BossBar` equality treats any two NaN fills as the same bar, without comparing NaN bits, which native and wasm32 may produce differently. Lossy round trips now report the first differing JSON pointer, or where the values' `Debug` forms differ when the JSON agrees. | `game: state equality is reflexive`; `regression-new-run-resets-stage` publishes the NaN fill entering its boss room at command 688. The original `regression-boss-bar-nan-equality`, which the shared MT moved off its branch, stays in the corpus as an ordinary workload (09cc3d1) |
| Killing a Dip while the player owns Gimpy (225) or Infestation 2 (234) failed the tick with `Dip death requires the owned225/234 source callbacks`, and every later tick failed the same way, so the browser's frame loop stopped. The item pipeline grants these items, but the Dip death callbacks they need were not reconstructed. DST reached it once the driver collected items and fought rooms. | Fixed on main for Gimpy (#329): the Dip death rolls Gimpy's half heart from the player's persistent RNG (47FCE9). Infestation 2's Dip death refused (`Dip death requires the owned234 BlueSpider source callback`) until #695 fixed it forward: the BlueSpider spawn is skipped and recorded as a Factory{3,73,0} frontier, and #295 stays open for the real familiar. The `rooms` unit reached it (`rebirth-dst --units rooms --case-seed 11434317217747007857`, seed B1HEPXPG, a Basement I treasure room, once in 3,000 cases on da2d591), and that case passes on 226cf04. | Retired once the shared MT moved `regression-dip-death-with-gimpy` off its branch: no tape now kills a Dip while owning Gimpy, so DST no longer reaches #329's callback |
| A chest that dropped a Troll or Megatroll Bomb (pickup 40/3 or 40/5) froze the game. Every tick after it failed with `moving pickup requires authored collision definition`. The loot is born moving, so the newborn grid pass looked up its collision definition by the transient subtype, which has none. Three spawn sites already mapped these subtypes to the bomb pickup's temporary visual definition, but the grid and animation lookups did not. | `pickups::definition_subtype` is the single owner of that mapping. Every spawn, animation and grid lookup for a live pickup resolves through it, as the transient subtypes' Init already documents. | `regression-chest-troll-bomb-loot` (380 commands) was retired once the shared MT stopped its chest dropping a troll bomb. Nothing guards the newborn grid lookup now. The census key `moving troll bomb` lets the next harvest that reaches one cut a replacement with `cover.py` |
| The walls invariant (#331) claimed a grid-colliding body's centre never leaves the room interior. A bomb dropped while the player stood in an open top doorway was born on the door cell, outside the interior, and rested there until the player left: the nonplayer ground solver (443117) reflects only motion into solid cells and never moves a body at rest, so nothing pushes it back. The property, not the engine, was wrong; one of 64 generated cases in a harvest hit it. | The invariant admits a centre on a doorway cell (grid kind 16) and still rejects any centre on a boundary wall or beyond the grid. | `regression-bomb-in-doorway` (230 commands) fails the old property at command 209 |
| In a Caves room two Wall Creeps (240) crawled the top wall. One, attacking (state 8, where its AI does not snap it back onto its wall line), touched the other (centres 26 px apart, radii 13), and the contact carried its centre to y 139.5, half a pixel past the interior's top edge, so the walls invariant failed. A Wall Creep on a top or bottom wall has grid class 1, which clamps only x and tests no cells (442e40 / 442ef0, `entities::clamp_walls`; class 2 clamps only y), so the source never holds its y inside the room. The property, not the engine, was wrong: it assumed every grid class clamps both axes, which no body the game unit simulated before contradicted. Found once the `rooms` unit evaluated the game state invariants, on bd7a8e8b. | The invariant checks the axis each class clamps: x for class 1, y for class 2, both for the rest, with the doorway cell still admitted. | `rebirth-dst --units rooms --case-seed 781911311167971346` failed the old property (signature `npc 240`) and passes |
| Gemini leaned on a bomb resting by the right wall and, over 15 ticks, pushed its centre 1 px into the boundary wall cell (the circle about 17 px into the wall) while the bomb's own velocity stayed below 0.4. A Wall Hugger (218) walking along a bomb's row later pushed one's centre 13 px into the left wall cell early in a run, so the fault was not Gemini's. The NPC–bomb contact response wrote the bomb's position after the bomb's own ground query, and the post-contact grid pass covered pickups, familiars and NPCs but not bombs. The source's final manager pass (454b00..454b3a) gives every non-player actor `442b90(false)` after contacts. | Implemented by #614, which closed #349: ordinary bombs now resolve class5 full ground contact at the manager fence after NPC contact writes ([system map](system-map.md#ordinary-bomb-late-grid-response-349)). Its qualification is scoped to the moving-doorway case and the existing regression banks. NPC/bomb timing parity stays open there: the near-wall and centre Hugger routes are still red, and the NPC contact lands 16 presentations before Native's. | Historical: `findings/gemini-pushes-bomb-into-wall` failed `game: grid-colliding bodies stay inside the room's walls` at command 1141 and `findings/wall-hugger-pushes-bomb-into-wall` at command 341. The shared MT moved both off the push before #614 landed, so neither replays the case the late pass corrects. Their non-reproduction is drift, not evidence of bomb parity. They were retired; 3f44a92 still holds them |
| `game: fire cooldown and head timers stay in range` also claimed the fire cooldown never exceeds the max fire delay. A generated run fired on the frame Number One's pickup animation ended: the shot set the cooldown from the old delay (10), then the queued collectible flushed and lowered the delay to 5. Player.Update fires at 52f..530 and flushes the queue at 531f01, so the source does the same, and a lowered delay can sit under a running cooldown. The property, not the engine, was wrong. | The state check keeps the lower bounds; `game: the fire cooldown never rises above a max fire delay the command held` bounds each command's cooldown by its earlier value and the delays before and after. A planted shot that overshoots its delay by 5 fails it. | `regression-number-one-cooldown` (1,010 commands) fails the old property at command 998 |
| `game: a run starts in the starting room of its seed's floor` required the command that starts a run to leave the player in the starting room with no other room visited. A generated player died holding ArrowRight and W, confirmed the GameOver restart, and the next 192-tick command started the new run in its starting room and, once the start hold ended, walked it through a door: held keys survive the restart. The property, not the engine, was wrong. | The contract now requires the visited rooms to be a walk from the seed's starting room through visited neighbours, the current room among them; `game: a new run's player walks on from the starting room within its first command` must be met. | `regression-restart-walks-on` (329 commands since #660 moved it) dies at command 315 and confirms the restart while holding a direction. Its first new-run command, at 318, walks from the starting room into the next. The `chases.json` recipe rebuilds it: the driver's GameOver arm holds a direction through a long confirm tick, so 7 of 64 branches reach it on 3cb9a52 |
| Main admitted the Basement I → II descent: a cleared boss room's trapdoor starts a StageTransition, SetNextStage advances the stage and requests the nightmare scene, and Level::Init generates the next floor from the stage's seed (`stage_transition.rs`, docs/stage-progression.md). One of 96 generated 8,000-step cases descended. Four `game` contracts written for a single floor per run then failed: the floor never changed, the stage seed was always stage 1's, the run clock always advanced once per tick, and controls were disabled only by the run-start fade and room transitions. The properties, not the engine, were wrong. | Each now states the descent contract. Within a run the floor only deepens, and full updates still never rewind. A descent may skip a stage, as Curse of the Labyrinth's XL floors do, and takes that stage's startup seed (stage *s* gets the (*s*+1)-th draw), as a run start takes stage 1's. While its stage and seed hold, the floor keeps its generated layout: grid, room templates and links. A room's own runtime state, such as the persistent-spawn map main added to descriptors (descriptor+88, 1cee4a6), is not layout. Forget Me Now and the five-pip Dice Room may later regenerate a stage under a new seed. The floor follows the current stage and seed while playing, the previous stage's floor holding only during an active stage transition; the menus keep the last run's floor. The run clock is exact except while the nightmare scene is pending or playing, and holds still while it plays. An active stage transition also disables controls. Controls lost outside the named holds must come back within 600 full updates, rather than being enabled exactly when no named hold lasts. That exact form had pinned the port's list of holds, which is how the descent broke it. Planted engine mutations fail them: the next stage's seed not adopted, the previous stage's seed, a run clock that advances during the nightmare, controls never disabled, and controls never restored (101 liveness failures on the regression tape). | `regression-new-run-resets-stage` (below) descends and meets `game: a run descends to a deeper stage's floor` |
| Main ported the Room light pass's fire flicker (54e1341): a lit Fireplace takes two shared draws (5d31b5, 5d31cb) on every presented frame. `game: a paused run freezes gameplay and the shared stream` failed on every paused tick with a lit fire in the room. The disassembly settles it (the shared-MT session, #162). Manager render 593cc0 calls Game::Render 54fcd0 every frame in game, which reaches Room render 5d1a70 unconditionally. Nothing up to the flicker draws tests pause; the pause overlay draws after the room. The property, not the engine, was wrong. | A paused run still freezes gameplay (player, tears, bombs, room, inventory, current room) and the run clock. The shared stream is left to its presentation consumers. A Quit chosen from the pause menu is exempt while pending or fading: its fade (main's #495) lets an active stage transition keep priority (54F9DA, 54FB88), so a player who quits during the Basement II arrival still lands and regains controls. | Every paused tick on the corpus and generated cases |
| Main gave each floor descriptor the source's persistent-spawn map (descriptor+88, 1cee4a6), which room admission fills in. `game: within a run the floor changes only by descending` compared whole `Floor` values, so every room entry that recorded a spawn cell failed it: 113 times in 48 cases on b319e73. The run-start contract compared the new floor with a fresh generation the same way. The properties, not the engine, were wrong: they pinned the floor's representation, not what Level::Init fixes. Their seed check also pinned every stage to its startup draw at all times, which Forget Me Now (it forgets the stage seed) would break once ported. | Both contracts now compare the generated layout: stage, seed, grid, starting room, and each room's grid cell, template and links. The layout must hold while the floor's stage and seed do. A deeper floor must take its stage's startup seed, and a run must start on stage 1 with stage 1's seed. The menus no longer pin a stage seed. | The 68 violation tapes of that run pass. Rewiring a room's links on its admission fails the layout check 14 times in 12 generated cases |
| Branching 64 cases from Basement II's start, a player died there, exited to the menus and started a new run: the run's floor was Basement I but `state.stage` stayed 2. Only the GameOver Restart reset it; `start_run`, which the menu path also takes, did not. The new run then read stage 2 for its music, doors and chests, and its Basement I boss trapdoor asked `next_stage(2)`, which fails, so every later tick failed and the browser's frame loop stopped. DST caught the stale stage through the new stage-seed contract. | Fixed in `start_run`, the single owner of a new run's first stage: it resets the stage and clears any stage transition for every new run, and the Restart's own reset is gone. The pause contract now also states that Escape is ignored while the nightmare scene is pending or playing (Game54f760), as the key handler does. `spawn_boss_award` spawns a trapdoor on stages 1–5 and 7, but `next_stage` admits only 1 → 2, so entering Basement II's own trapdoor failed every tick the same way (#413). Main fixed that in 2efbef1: a trapdoor whose next stage has no admitted generator stays open but never takes the player. | `regression-new-run-resets-stage` (2,970 commands when rebuilt on a1972be) fails the stage-seed contract 301 times and the run-start contract once without the fix. The `chases.json` recipe rebuilds it whenever the shared MT moves it off its branch. Its run enters Duke's room at command 688, clears it at 2538, presses Escape during the nightmare scene (the pause gate), dies on Basement II, exits to the menus and starts a new run at command 2669 |
| The shared-MT stack (#398) ported the ambient Tiny Bugs, Tiny Flies and Wall Bugs. A Tiny Bug that wandered onto a pit failed the tick with `Tiny Bug pit recovery is outside supported grid objects`, and every later tick in the room failed the same way, freezing the browser's frame loop. The check assumed no admitted room places a pit, but Basement rooms do; two corpus tapes reached it within their first rooms. The same stack removed the renderer-only tear `impacts`, so the DST crate stopped compiling on main. | The pit branch now does what its source comment states: the bug returns to its previous position (Effect+440). The tear must-hit now looks for the sized TearPoof effect (1000.12, 13, 0x4f or 0x50) that replaced the impact. | `regression-tiny-bug-pit` (600 commands) fails 8 ticks without the fix |
| Bodies pushed into walls (#365). A player walked a closed chest (5.50) into a bottom wall and settled its centre 10.8 px inside the wall cell; a locked chest (5.60) went into a corner, and one pushed up a door column passed the open door cell and left the grid; a Lil' Battery (5.90) and a Horf (12) went into side walls in front of a walking player; a Gaper behind a Gusher (11), a Larry Jr. segment beside a Charger (23) and a Mulligan (16) behind another pushed those bodies into walls; and Monstro (20), landing on a Gaper (10.1), threw it 27 px into the top wall row. Each time the contact response (4545d0) moved a ground body past the room's interior edge, and the late ground pass (442b90) left its centre there: it reflects only inward motion and corrects at most max(speed, 2) px, so a body the contact kept pushing ratcheted in. The source query clamps the centre to the room bounds before its samples for class 5 as for class 3 (442c2d; 442c3b..69 at any speed and in prediction), but the port's class-5 collider, unlike its class-3 one, left that clamp to its callers. Bombs (#614), familiars and the Gaper preview clamped; ground-walking NPCs, pickups, Dips, Monstro and the Roundworm did not. Once the `rooms` unit evaluated the state invariants (#748) it met the same gap often: of 2,000 cases on 59e1f9c, 281 failed, 218 with a locked chest (signature `pickup 60`), 46 with Mega Fatty (264) landing a jump aimed past a wall (`npc 264`, `--units rooms --case-seed 9306351262927921413`), 10 with a coin (`pickup 20`, `--case-seed 6758684668483473617`), 6 with a chest and one with a Gaper. | The class-5 collider (`resolve_nonplayer_ground` and the Dip's bouncy variant) clamps the centre before its samples, as `resolve_nonplayer_walls` does for class 3, so no caller can skip it; the callers' own clamps were folded into it. DST shows the engine now matches its source contract. It is not original evidence: whether the original's clamped positions match needs the live comparison #365 asks for, a contrived `chest-opening` case pushing a chest into a wall and through an open door, with the NPC suites the clamp touches. | `regression-player-pushes-chest-into-wall` (380 commands), `regression-player-pushes-horf-into-wall` (601) and `regression-monstro-landing-pushes-gaper-into-wall` (2,271) fail `game: grid-colliding bodies stay inside the room's walls` on 59e1f9c, at commands 379, 600 and 2270, and pass. The 2,000 `rooms` cases (`--seed 0x3d92ec53a1f3dfab`) all pass. The battery, Gusher, Charger and Mulligan tapes had drifted off their contacts by 59e1f9c and were retired; it still holds them |
| A run whose Caves I boss draw is Double Trouble (layout 3700) could not arrive on Caves I. `advance_trapdoor` takes the player because `next_stage(2)` is admitted since #654, and the next level's admission (`admit_next_level`) propagates the generator's `Double Trouble's 2x1 boss room requires unimplemented boss placement` (floor.rs). The Tick fails, every retry reaches the same admission, and the browser halts on the first failure: about 9% of the `rooms` unit's Caves I arrivals through the trapdoor (9 of about 100 in 300 cases); stage/rooms measured 288 of 4,000 seeds. | Fixed on main (#688): the trapdoor takes the player only when the next floor generates from the exact request the level's admission uses, so a Double Trouble seed stays on Basement II until content's 2x1 fit admits it. The `rooms` unit descends through the trapdoor's own entry check, so a refusal is no violation. | `rebirth-dst --units rooms --case-seed 15207181494778813838` (seed 6CA0SPPZ, from Basement II) failed `rooms: no Tick fails while a run enters and plays its floor's rooms` at the trapdoor's stage arrival before the fix and passes on 8a5e69f |
| Playing a Caves I room failed the Tick with `pathfinder probe left the room grid; Room+e94 is not modelled`, in a curse room (10.8) and an ordinary one (1.199). Stage/rooms' scan found the same halt in 4 of about 8,700 Caves I room entries, with Larry Jr. in large rooms 1.321 and 1.349. | Fixed on main (#689): stage/rooms' Room+e94 fallback. | `rebirth-dst --units rooms --case-seed 1692406417774197375` (seed CGVFTQDN, Caves I room 8) failed `rooms: no Tick fails while a run enters and plays its floor's rooms` before the fix and passes on 8a5e69f |
| Opening a chest in a Caves I room (1.345) failed the Tick with `Chest350 Init/Get is outside the admitted grant-only slice`: the chest's loot rolled a trinket (5.350), whose Init/Get #668 refuses (`chests::admit_child`). Every later Tick against the chest fails the same way and the browser halts. It is the authored-trinket and TNT halts' refusal (#674, #677) on the chest path. Found by the `rooms` unit on 540197a. | Fixed on main (#692): the chest skips the 350 child and records it, as #674's authored-trinket path does. | `rebirth-dst --units rooms --case-seed 11399924783966766290` (seed WWEA87BT, Caves I room 15) failed `rooms: no Tick fails while a run enters and plays its floor's rooms` before the fix and passes on da2d591 |
| Playing a Basement I room (1.104) failed the Tick with `Hopper state 1 is not admitted`. The decoded 4cfcf0 dispatch handles only states 3 and 4 for Hoppers (29) and Leapers (54) and sends every other state to the shared tail, but the port refused any other state. Found once in 600 `rooms` cases on 6a1a0d41. | Fixed on main (#690): a Hopper or Leaper in any other state runs only the shared tail, as 4cfcf0 does. | `rebirth-dst --units rooms --case-seed 233620001356869716` (seed 9Y4P1T0R, Basement I room 8) failed `rooms: no Tick fails while a run enters and plays its floor's rooms` before the fix and passes on dc38bee |
| Isaac, holding keys, touched a Key Master (slot 6.7). Since #662 slots are admitted only as idle bodies, and a Player contact that would reach a slot's resource gate (`slots::player_contacts`, 539690) was an explicit error: `Key Master contact with keys is unadmitted`. Every later Tick against the Key Master failed the same way (60 of 60), and the browser halted on the first failure. Slot machines and beggars did the same with coins, Blood Donation and Devil Beggar on any touch, and NPC, pickup or bomb contact and explosions near a slot (`slots::unadmitted_contacts`, `slots::bomb_hit`). Found by the deep soak on 139d771. | Fixed forward: an unadmitted slot interaction no longer fails the Tick. Player contact keeps the idle body's mass response and skips the payout, resource gate or Blood Donation hurt; a flying or nonground Player takes that response without the grid-shift veto; NPC, pickup and bomb bodies pass through slots; explosions leave slots idle. Each skipped path is a documented port gap in `slots.rs` and the TEST_MATRIX slot row; payouts stay with the slots workstream. | [`corpus/regression-key-master-contact-halts`](../engine/dst/corpus/regression-key-master-contact-halts.json) (5,922 commands, with a save host) failed `game: valid Tick commands always succeed` at command 5921 before the fix |
| Isaac, holding Technology 2 (collectible 152), fired its laser in a room with a slot machine (6.1). The laser's target query (`laser_runtime.rs`) refused any Slot actor, `Laser query Slot6 target membership is unadmitted`, so the Tick failed, as did all 60 later ones, and the browser halted. Found by the deep soak on 139d771. | Fixed forward: a slot is a non-target of the laser query, so the beam passes it. Its Slot6 membership is not decoded; TakeDamage53bd80 ignores every flag but explosion4, so a hit would deal nothing either way. The membership stays a documented port gap. | [`corpus/regression-laser-targets-slot-halts`](../engine/dst/corpus/regression-laser-targets-slot-halts.json) (4,789 commands) failed `game: valid Tick commands always succeed` at command 4788 before the fix |
| A TNT barrel's blast rolled its 1-in-100 rare reward from the TNT's own RNG (`tnt.rs`), which removes a trinket from the pool and spawns it. The TNT owner refused that Remove/Init (`TNT rare trinket Remove/Init is outside the admitted slice`), so the Tick failed. The roll comes from the barrel's fixed seed, so every retry reached it again: 49 of the next 60 ticks failed, all after the blast frame, and the browser halted on the first. Found by the deep soak on 139d771. | Fixed forward as a recorded port gap: the blast skips the rare trinket's Remove/scatter/Init, records it (the barrel's reward frontier and the `RareTrinketPool` frontier) and continues; the blown barrel keeps its ordinary update. Porting the reward through the trinket pool stays with the trinket workstream (#668). | [`corpus/regression-tnt-rare-trinket`](../engine/dst/corpus/regression-tnt-rare-trinket.json) (150 commands, with a save host) failed `game: valid Tick commands always succeed` at command 149 before the fix |
| On a Basement I run, Isaac walked from room 6 into room 7 (catalog layout 219), whose layout authors a trinket (pickup 5.350). Since #668 the room spawn refused an authored 350 whose Init/Get the port does not represent (`authored350 Init/Get is outside the admitted grant-only slice`), so the Tick failed, every later Tick retried the same entry and failed, and the browser halted on the first failure. The corpus tapes `coverage-89ea6c8c21f7eea2` (pruned on 87ddf06) and `coverage-gemini-kill` shared this prefix and failed on it. Found by `test:dst` on 6dd1186. | Fixed forward: the authored 350 again takes the unsupported-factory path beside it. Its cell is recorded as unknown (a later lookup of that cell still stops), and item queries keep #668's record of the unrepresented spawn (`note_unrepresented_trinket_spawn`), so the room stays traversable. Porting the authored 350 Init/Get stays with the trinket workstream (#668). | [`corpus/regression-authored-trinket-room-entry`](../engine/dst/corpus/regression-authored-trinket-room-entry.json) (139 commands) failed `game: valid Tick commands always succeed` at command 138 before the fix |
| A player spent about 5,700 commands in one Basement room. Its background paint (blood and other stains, which `familiars.rs` appends and nothing caps) grew to 6,350 entries. The player then quit from the pause menu. With save IO, the quit's save serializes the run, and `RunRecord::validate` (#646) refused any saved room holding more than 4,096 stains, so the Tick failed with `saved room is outside supported bounds`. Every later Tick failed the same way, so the browser's frame loop stopped. Found by the deep soak on 431cb74. | Fixed forward: a save whose rooms exceed the record's actor or paint counts is refused instead of failing the Tick. The profile keeps its previous record, no write is requested and the run continues. The original paints blood onto the backdrop, so it has no stain list to cap. The record bounds, and whether stains persist at all (#603), stay with the saves workstream. | [`corpus/regression-save-room-many-stains`](../engine/dst/corpus/regression-save-room-many-stains.json) (with a save host) failed `game: valid Tick commands always succeed` at command 5740 before the fix |
| `Game` skipped `save_io_enabled` in serde (#646), so a game that declared save IO restored from its own JSON with the capability off. Its next menu readmission then rebuilt the menus from no document instead of waiting for the host's, losing every retained profile and saved run, and save IO can be declared only at a fresh Title. The continuation and complete-game round-trip contracts failed on every save-host case, 21 of 48 on 78cacac. | Fixed in this PR: the flag serializes, defaulting to off for older JSON. `save_write_request` and `command_origin_screen` stay transient, since they hold only between a command and the host's answer. | [`coverage-save-continue`](../engine/dst/corpus/coverage-save-continue.json) (638 commands, by its recipe) stores a write, fails another, loads the stored document on a readmission and continues the saved run at command 337. With the field skipped again it fails the continuation contract once and the round trip 14 times |
| #703 made a save store Saved+EF80's seed text, the formatter's nine bytes with a space at byte 4 (`seed::formatted`), while Continue keeps the run's compact code and accepts either form when it loads (`lib.rs`, the saved-run admission). `game: Continue resumes the saved run's character, seeds and floor` compared the two texts byte for byte, so every run saved after #703 failed it on Continue: `coverage-save-continue` failed it on af6fc7e. The property, not the engine, was wrong. | Continue now compares the saved and live seeds by the seed they spell (`seed::decode`), alongside the seed value the record's context holds; the published code stays compact. | `coverage-save-continue` and the saves cases meet the contract again |
| A player holding Deck of Cards (active item 85) cleared Larry Jr.'s room and took the trapdoor. The descent's level admission saves the run (`save_supported_run`, lib.rs), and the save normalized the player (`player_save::normalize_player`, 518120), which refused any active item other than none or Yum Heart (45). Every later tick failed with `Player save normalization requires an unmapped temporary/active owner`, so the browser's frame loop stopped: reaching Basement II with almost any active item froze the game. 8 of 64 cases branched from the Larry Jr. entry hit it on b319e73. | Fixed on main (#503, 01a249f): 518120 has no active-item gate, so the held active, its charges and reserve pass through the save; only an active use in flight (D80/D84, which no port writer sets) is still refused. #613 adds the stage-exit case Y9QL8QZM, which holds an active item through the trapdoor (#499); the [system map](system-map.md) records its live qualification status. | `regression-descent-with-active-item` (2,919 commands) takes Deck of Cards at command 769, enters Larry Jr.'s room at 1677, clears it (14 of 64 branches) and descends holding it at 2618. With the old refusal planted back it fails `game: valid Tick commands always succeed` 97 times. Its `chases.json` recipe asks for each step with the item still held (`boss room holding an active item`, `boss room cleared holding an active item`, `descent holding an active item`), since a run that restarts loses it: the 40469d3 rebuild (3,277 commands) descends holding The Poop (36) at command 2976 and fails the same property 170 times with the refusal planted back. #660's frame counter moved it off its boss clear. The 3cb9a52 rebuild (3,323 commands) takes an active item at command 509, enters the boss room at 1985, clears it (1 of 64 branches, from the third of four prefixes) and descends holding it at 3022. With the refusal planted back it fails `game: valid Tick commands always succeed` 96 times, from that descent on |
| Main ported Envy and Super Envy (#605): a head that dies splits into two or three smaller heads whose runtime variants are `v + 10·level`, spawned from config row `v % 10`. Main's gib port (09a6c50) had Die look up the dying NPC's config by `(kind, variant)`, and no row 51.10 or 51.11 exists, so the first split head to die panicked with `entity is not authored in entities2.xml`. In the browser that traps the WASM and freezes the game. 28 of 64 cases branched from a Basement I run start hit it. | `Npc::config` is the single owner of the authored row an NPC was born from: the original keeps the config its spawn resolved, and Die 445c60, the champion roll and the grid colliders read it. Envy declares its runtime-variant mapping (`envy::config_variant`); every other family's runtime variant is its authored row. The death sound, gibs, champion roll and both colliders now read `npc.config()`. | `regression-envy-split-death` (599 commands) panics at command 568 without the fix; its `chases.json` recipe rebuilds it from the Envy entry to the first Envy death |
| A bomb that destroyed a tinted rock froze the game when the rock's 56d440 reward drew a Megatroll Bomb (40/5): every tick after it failed with `unresolved pickup selector 40/5`. The chest-loot fix had made `pickups::definition_subtype` the owner of the transient Troll and Megatroll subtypes' temporary visual definition, but the newer tinted-rock reward (50167a5) handed the raw subtype to `Pickup::spawn`, which has no 40/5 definition. | `Pickup::spawn` resolves its definition and spritesheet through `definition_subtype` itself and keeps the real subtype, so no birth site can bypass the mapping; existing callers that pre-map behave identically (every corpus tape replays to the same state hash after every command). The reward's 40/5 then converts into a Megatroll Bomb actor on its first full update, as chest loot does. | The `pickups` unit spawns every rolled 40/3 and 40/5 and checks they draw the regular bomb's visual, which fails without the fix. `regression-tinted-rock-megatroll` no longer draws the reward since the shared MT moved it off its branch, and stays in the corpus as an ordinary workload (09cc3d1) |
| The `game` driver kept its own copy of which rooms the player may enter. Main admitted large and miniboss rooms (#171, #181) but the copy did not, so DST never entered Gluttony's room. Lasers, the shared pathfinder and the laser invariants went unexercised; generated runs reached 8 of the 12 implemented NPC kinds. | `rebirth_cleanroom::traversal::admits` is the single owner of doorway admission (the engine's departure check uses it). The driver routes by it, and explorers head for the nearest unvisited admitted room. 48 generated runs then reached Gluttony and Pooters and met every `game` property without the corpus. | `game: the player is in a miniboss room` and `game: a laser is live`, which `coverage-miniboss-clear` meets on every run |
| A batched Tick that returned to the menus dropped what its earlier steps had published. The menu reconstruction Pause Quit ends in (`reconstruct_menu_from_profiles`, directly or when the host answers the menu readmission) reset the Options and Controls menus to their defaults, keeping only their transport sequences, so the command's settings and input-configuration save requests and unavailable-action reports went with them. Single Ticks publish each step's before that step, so the browser reported an M (Mute, action 18) pressed during the quit fade only when it ran one step per command, and a settings save made earlier in the same batch would have been lost the same way. The shared-MT workstream met it on a8056a80: `game: a batched Tick simulates exactly what single Ticks do` failed at step 1994, single Ticks publishing `[18]` and the batch `[]`. Its retained tape did not fail under `--replay`, because the batch check was sampled from the case's entropy. | Both menus' reconstruction keeps the command's outbox along with the sequences (`OptionsMenu::reinitialize`, `ControlsMenu::reinitialize`). The harness now draws sampled checks from the state they check, so a retained tape replays them, and always re-runs a batch that changes screens or waits on save IO. | [`regression-quit-batch-keeps-mute-report`](../engine/dst/corpus/regression-quit-batch-keeps-mute-report.json) (67 commands, with a save host) pauses a run, confirms Quit, presses M and runs one 32-step Tick across the fade into the menus. Without the fix it fails the batched-Tick property at command 65 (single `[18]`, batch `[]`), as it does without a save host; with the fix both pass |

## Running on Antithesis

[antithesis/](../antithesis) packages the harness for the platform:
`docker build -f antithesis/Dockerfile -t rebirth-dst .` builds the harness
and the production `engine.wasm` (with `tools/build-wasm.sh`, the browser
artifact's own build, so the image checks the WASM that ships) from a context that
[Dockerfile.dockerignore](../antithesis/Dockerfile.dockerignore) limits to the
engine build's inputs. Those inputs are what Cargo records in the two builds'
dep-info: sources, `include_str!` files and every path `build.rs` reads.
`npm run test:dst` runs `tools/dst/sync-context.py` after building them, which
re-includes any input the list drops and names the change to commit, so an
engine change that gives `build.rs` a new input cannot silently break the
image. The entrypoint signals `setup_complete` and
Test Composer runs the [drivers](../antithesis/test/v1/rebirth-engine) with
`--entropy antithesis`: every registered unit but `game` (the driver lists them
from `rebirth-dst --list`, so new units join without a driver change), and one
whole-engine game per invocation checked against the shipped WASM. Under Antithesis, must-hit
properties are judged across the whole exploration, so a single driver
invocation never fails for leaving some unmet. `--build-arg INSTRUMENT=1` adds
Antithesis coverage instrumentation and needs the platform's `libvoidstar.so`
at `antithesis/libvoidstar.so`.

The platform is optional. Local runs (`npm run test:dst`) are the suite: they
use the same SDK assertions and Antithesis's must-hit semantics, with seeded,
replayable entropy, and every finding so far came from them. A platform run
would add guided input search and far larger parallel exploration. Its fault
injection and scheduling control add little to a single-threaded simulation
without I/O.

The binary embeds data compiled from the private original resources (room
layouts and catalogues). Pushing the image to any external registry,
Antithesis included, requires the repository owner's explicit approval. The
entrypoint and both drivers run cleanly in the SDK's local mode
(`ANTITHESIS_SDK_LOCAL_OUTPUT`).

## Extending

Add a unit as a module under `engine/dst/src/units/`, register it in
`units/mod.rs`, and prefix its property messages with its name. Prefer model
or differential oracles and properties grounded in source evidence or
documented contracts over restating implementation arithmetic.

Pin only safety and what the original establishes. Safety means:
- no panic or frozen tick;
- a refusal changes nothing;
- determinism and lossless round trips;
- native/WASM agreement.

What the original establishes comes from its source or a documented
contract. Never pin a port gap, such as:
- an unimplemented feature's refusal;
- a list of what is ported today (stages, NPC families, pickup variants,
  menu entries, the holds that disable controls);
- a ported system's current limits.

Such a property fails whoever ports the feature, and they then have to argue
with the suite. So:
- **Admission is one-directional:** the admitted domain must run correctly.
  Outside it the engine may refuse, leaving everything unchanged, or serve
  the input. It is never answered with a non-finite value.
- **Must-hits name what the workload explores** ("steps outside the admitted
  domain are tried"), never a refusal the engine happens to make.
- **Liveness replaces enumeration where a gap is likely.** For example, lost
  controls must come back within 600 full updates; the suite doesn't require
  controls to be off exactly while today's holds last. When an engine
change legitimately alters a documented contract, update the unit's model in
the same change. Fix or record every violation and never weaken a property or
its workload to pass. Run `npm run matrix:update` after adding or renaming
properties.

Before a PR that adds properties, make sure none can go unmet by chance:
- Run the corpus-only census (`rebirth-dst --units game --cases 0 --corpus
  engine/dst/corpus`). A `game` property met only by generated cases is a
  flake, because a 12-case `test:dst` run can miss it. Give it a corpus tape
  that meets it every run, as `coverage-boot-elapsed` does for #761's boot
  clock.
- In the other units, which have no corpus, a must-hit that a 12-case run
  meets only a few times is a flake too. Make the workload reach it on
  purpose, as the blast workload's draw on the envelope does for
  `bombs: a player stands on a blast's edge`.
