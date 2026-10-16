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
| `grid` | grid queries and both ground-collision solvers on generated rooms | resolution never fails for bounded motion (including residual speeds whose squares underflow); previews keep velocity; an underflowing inward preview skips refinement (source 443117 → 44346c); correction moves only out along the contact normal; full nonplayer reflections leave the wall; blast candidate ordering and bounds |
| `contact` | the mass contact responses (`combat.rs`, source 445710) for player–NPC, player–pickup, player–projectile, tear–NPC and NPC or bomb–NPC pairs, over arbitrary radii, masses, speeds and separations | computed in f64 from the inputs: the response runs exactly for strictly overlapping circles; rejected and non-overlapping pairs stay bit-for-bit unchanged; positions and velocities change only along the centre line (allowing for the f32 cancellation the source's fast-separating scaling shares); separation keeps the pair's centre of mass in place; the impulse conserves momentum, the player taking half its share, and carries the relative speed split by mass; an ordinary pair ends exactly a radius sum apart; a blocked player keeps its motion and the collider is offered exactly the displacement applied; an NPC pair resolves the same whichever body comes first. The fast-separating branch's overlap scaling (1 + 2·dot) is statically recovered and checked only through these conservation laws |
| `pathfinder` | FindGridPath's flood (`pathfinder::flood_costs`, 5b498f) over room-shaped and arbitrary cost grids | an independent node-weighted Dijkstra: every label is the cell's shortest distance whatever order heap ties take; only open cells of the grid change; the owner is visited exactly when it lies within half the room's cells; without it the flood visits exactly the cells nearer than that cutoff, with it every cell nearer than the expansion that visits the owner and none farther, and that expansion runs to its end; purity; grids outside admission are rejected |
| `item_pools` | the run-owned ItemPool manager (`docs/item-pools.md`) | selection never offers a collectible twice (Breakfast aside), comes from its pool, the treasure fallback, Breakfast or the Bible, and advances none of the manager's streams; depletion only lowers weights and fills the removed/selected maps; a zero-counter counterfactual isolates Rosary's Bible override and the candidate it marks; drains reach Breakfast within the catalog; locked collectibles survive every redraw; error rooms route with one shared draw and special rooms to their namesake pools; JSON continuation; rejections without mutation |
| `game` | the whole engine through public commands | per-step state invariants, per-command transition contracts (menus, run-start hold, pause, boss scenes, stage descents, death and GameOver), rejection of malformed inputs, tape replay, serialized-midpoint continuation, the C ABI, the shipped WASM |
| `abi` | the exported C ABI the browser bridge drives | framing bounds, malformed requests, failed resets, pre-reset commands on a fresh thread, the published key surface; a boot from the authored challenge XML and special-room STB through `reset_with_boot_resources` publishes and plays exactly like the plain boot, and truncated, unknown-field, oversized or corrupted resources fail with an error that keeps the previous game |

The `game` driver ([driver.rs](../engine/dst/src/units/game/driver.rs)) only
emits commands the browser bridge can send, plus adversarial variants of them
(near misses of public keys, malformed seed text, out-of-range volumes,
over-budget ticks, stale music callbacks, key repeats and releases). A Tick of
exactly the 36,000-step command budget is sent only while paused or in the
menus: in play it would simulate ten minutes in one command, repeated by every
replay and the interpreted WASM differential. Each case
follows one persona (direct play, menu explorer or seeded run), sends
browser-style host-timestamped frames (with jitter, hitches and rollbacks) or
controlled steps, and steers by the published state. In a room it plays
before it leaves:
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

On c8931a8, with Gapers admitted, Monstro died in 1 of 64 branches and Duke
of Flies' room was cleared in 1 of 128.

Every report carries a coverage census: the room types, NPC kinds, pickups and
game features (owned collectibles, secret reveals, boss clears) each case
reached, including each owned weapon collectible (`weapon:<id>`), each
floor stage played (`stage:<n>`), a run started after a descent (`new run
after a descent`, and `new run from the menus after a descent` when it went
through the menus), a run started from Random (`Random run`) and its
GameOver popup's Exit to the menus (`menus after a Random run's GameOver`),
and each property met (`property:<message>`). The summary
line names the implemented NPC kinds no case simulated
(from `rebirth_cleanroom::implemented_npc_kinds`) and the room types that
generated floors contained but no case entered. Unsimulated kinds are split
by whether any authored Basement or special room places them
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
* **Batched ticks.** A sampled `Tick` of up to 32 steps is re-run as that many
  single ticks and must reach the identical game. Only command-level
  publication may differ: the batch carries every step's sound and music
  events, and a screen change resets the menu animation counter at the
  command boundary.
* **C ABI differential.** The tape drives the exported `reset`/`request_ptr`/
  `command`/`snapshot_*` functions exactly as `web/engine.ts` does.
* **Shipped-WASM differential.** With `--wasm web/generated/engine.wasm` (the
  default in `npm run test:dst`) each tape also runs through the production
  artifact in the deterministic `wasmi` interpreter; every outcome, the
  published key surface and sampled snapshots must equal native dispatch.
  A differing snapshot is reported with the JSON pointer of its first
  differing value and both sides' values. Native-only properties therefore
  speak for the artifact browsers run.

### Tapes and the corpus

Game tapes use the engine's replay format (`initialSeed`, `bootEpochSeed`,
`commands`), so a finding replays through the browser's
`game.reset(seed, { bootEpochSeed })` plus `game.command(...)`, or through
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
  super secret room), retire NPC kinds whose deaths ordinary runs never see,
  and simulate kind 21, which no authored room places. Game tapes drift as
  the engine evolves;
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

`--verify` replays each written tape and drops any that no longer reaches its
keys. When an engine change moves a must-hit coverage tape off its branch,
the same harvest refreshes it. The census keys every property a case meets
as `property:<message>`: an `always` it evaluated, or a `sometimes` or
`reachable` that held. So `--prefix property:` cuts tapes for the must-hits
the corpus stopped meeting.

`--resume TAPE` makes every generated `game` case replay that tape (with
every check) and then drive on from its end, so exploration branches from an
interesting state, as Antithesis does, instead of rediscovering it. Cut a
harvested tape at, say, a boss room entry, resume many cases from it, and the
census shows how often each branch clears the room; a clearing branch is a
short coverage tape.

`tools/dst/chase.py` chains those branches for scenarios that need several
rare events in a row. Each `--stage` names census keys. Every stage branches
64 cases from the prefix so far and keeps the case that reached its keys
earliest, cut just past them. The next stage branches from that cut, and the
final tape is written only if its replay reaches every stage.

Every scenario tape built this way keeps its recipe in
[engine/dst/chases.json](../engine/dst/chases.json): the tape, the command
its prefix ends at and its stages. The stages start with the scenario's
preconditions, such as `room:5` or `npc:20` before a boss kill. A prefix that
still reaches them skips those stages; one that drift moved off them (main's
Gaper admission moved the descent tape out of Duke's room before command
302) branches forward until it reaches them again. These tapes are the
descent, restart and Random-exit regressions, the Monstro, Larry Jr. and
Gemini kills, and the Dingle entry (`"from": null` chases from a fresh boot).
Boss fights and descents are the first coverage a shared-MT change moves, so
one command rebuilds all of them:

```sh
python3 tools/dst/chase.py --recipes engine/dst/chases.json   # --only NAME for one
```

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
| Killing a Dip while the player owns Gimpy (225) or Infestation 2 (234) failed the tick with `Dip death requires the owned225/234 source callbacks`, and every later tick failed the same way, so the browser's frame loop stopped. The item pipeline grants these items, but the Dip death callbacks they need were not reconstructed. DST reached it once the driver collected items and fought rooms. | Fixed on main for Gimpy (#329): the Dip death rolls Gimpy's half heart from the player's persistent RNG (47FCE9). Infestation 2 still refuses a Dip death (`Dip death requires the owned234 BlueSpider source callback`), tracked in #295; no tape reaches it yet. | Retired once the shared MT moved `regression-dip-death-with-gimpy` off its branch: no tape now kills a Dip while owning Gimpy, so DST no longer reaches #329's callback |
| A chest that dropped a Troll or Megatroll Bomb (pickup 40/3 or 40/5) froze the game. Every tick after it failed with `moving pickup requires authored collision definition`. The loot is born moving, so the newborn grid pass looked up its collision definition by the transient subtype, which has none. Three spawn sites already mapped these subtypes to the bomb pickup's temporary visual definition, but the grid and animation lookups did not. | `pickups::definition_subtype` is the single owner of that mapping. Every spawn, animation and grid lookup for a live pickup resolves through it, as the transient subtypes' Init already documents. | `regression-chest-troll-bomb-loot` (380 commands) was retired once the shared MT stopped its chest dropping a troll bomb. Nothing guards the newborn grid lookup now. The census key `moving troll bomb` lets the next harvest that reaches one cut a replacement with `cover.py` |
| The walls invariant (#331) claimed a grid-colliding body's centre never leaves the room interior. A bomb dropped while the player stood in an open top doorway was born on the door cell, outside the interior, and rested there until the player left: the nonplayer ground solver (443117) reflects only motion into solid cells and never moves a body at rest, so nothing pushes it back. The property, not the engine, was wrong; one of 64 generated cases in a harvest hit it. | The invariant admits a centre on a doorway cell (grid kind 16) and still rejects any centre on a boundary wall or beyond the grid. | `regression-bomb-in-doorway` (230 commands) fails the old property at command 209 |
| **Open.** Gemini leaned on a bomb resting by the right wall and, over 15 ticks, pushed its centre 1 px into the boundary wall cell (the circle about 17 px into the wall) while the bomb's own velocity stayed below 0.4. A Wall Hugger (218) walking along a bomb's row later pushed one's centre 13 px into the left wall cell early in a run, so the fault is not Gemini's. The NPC–bomb contact response writes the bomb's position after the bomb's own ground query, and the post-contact grid pass covers pickups, familiars and NPCs but not bombs. The source's final manager pass (454b00..454b3a) gives every non-player actor `442b90(false)` after contacts. | Tracked in #349 for the bomb owner: give bombs the late grid pass from the bomb's Update disassembly, checked by the live bomb suites. | [`findings/gemini-pushes-bomb-into-wall`](../engine/dst/findings/gemini-pushes-bomb-into-wall.json) fails `game: grid-colliding bodies stay inside the room's walls` at command 1141, [`findings/wall-hugger-pushes-bomb-into-wall`](../engine/dst/findings/wall-hugger-pushes-bomb-into-wall.json) at command 341 |
| `game: fire cooldown and head timers stay in range` also claimed the fire cooldown never exceeds the max fire delay. A generated run fired on the frame Number One's pickup animation ended: the shot set the cooldown from the old delay (10), then the queued collectible flushed and lowered the delay to 5. Player.Update fires at 52f..530 and flushes the queue at 531f01, so the source does the same, and a lowered delay can sit under a running cooldown. The property, not the engine, was wrong. | The state check keeps the lower bounds; `game: the fire cooldown never rises above a max fire delay the command held` bounds each command's cooldown by its earlier value and the delays before and after. A planted shot that overshoots its delay by 5 fails it. | `regression-number-one-cooldown` (1,010 commands) fails the old property at command 998 |
| `game: a run starts in the starting room of its seed's floor` required the command that starts a run to leave the player in the starting room with no other room visited. A generated player died holding ArrowRight and W, confirmed the GameOver restart, and the next 192-tick command started the new run in its starting room and, once the start hold ended, walked it through a door: held keys survive the restart. The property, not the engine, was wrong. | The contract now requires the visited rooms to be a walk from the seed's starting room through visited neighbours, the current room among them; `game: a new run's player walks on from the starting room within its first command` must be met. | `regression-restart-walks-on` (303 commands) dies at command 224 and confirms the restart while holding a direction. Its first new-run command, at 292, walks from the starting room into the next. The `chases.json` recipe rebuilds it: the driver's GameOver arm holds a direction through a long confirm tick, so 3 of 64 branches reach it |
| Main admitted the Basement I → II descent: a cleared boss room's trapdoor starts a StageTransition, SetNextStage advances the stage and requests the nightmare scene, and Level::Init generates the next floor from the stage's seed (`stage_transition.rs`, docs/stage-progression.md). One of 96 generated 8,000-step cases descended. Four `game` contracts written for a single floor per run then failed: the floor never changed, the stage seed was always stage 1's, the run clock always advanced once per tick, and controls were disabled only by the run-start fade and room transitions. The properties, not the engine, were wrong. | Each now states the descent contract. Within a run the floor only deepens, and full updates still never rewind. A descent may skip a stage, as Curse of the Labyrinth's XL floors do, and takes that stage's startup seed (stage *s* gets the (*s*+1)-th draw), as a run start takes stage 1's. While its stage and seed hold, the floor keeps its generated layout: grid, room templates and links. A room's own runtime state, such as the persistent-spawn map main added to descriptors (descriptor+88, 1cee4a6), is not layout. Forget Me Now and the five-pip Dice Room may later regenerate a stage under a new seed. The floor follows the current stage and seed while playing, the previous stage's floor holding only during an active stage transition; the menus keep the last run's floor. The run clock is exact except while the nightmare scene is pending or playing, and holds still while it plays. An active stage transition also disables controls. Controls lost outside the named holds must come back within 600 full updates, rather than being enabled exactly when no named hold lasts. That exact form had pinned the port's list of holds, which is how the descent broke it. Planted engine mutations fail them: the next stage's seed not adopted, the previous stage's seed, a run clock that advances during the nightmare, controls never disabled, and controls never restored (101 liveness failures on the regression tape). | `regression-new-run-resets-stage` (below) descends and meets `game: a run descends to a deeper stage's floor` |
| Main ported the Room light pass's fire flicker (54e1341): a lit Fireplace takes two shared draws (5d31b5, 5d31cb) on every presented frame. `game: a paused run freezes gameplay and the shared stream` failed on every paused tick with a lit fire in the room. The disassembly settles it (the shared-MT session, #162). Manager render 593cc0 calls Game::Render 54fcd0 every frame in game, which reaches Room render 5d1a70 unconditionally. Nothing up to the flicker draws tests pause; the pause overlay draws after the room. The property, not the engine, was wrong. | A paused run still freezes gameplay (player, tears, bombs, room, inventory, current room) and the run clock. The shared stream is left to its presentation consumers. A Quit chosen from the pause menu is exempt while pending or fading: its fade (main's #495) lets an active stage transition keep priority (54F9DA, 54FB88), so a player who quits during the Basement II arrival still lands and regains controls. | Every paused tick on the corpus and generated cases |
| Main gave each floor descriptor the source's persistent-spawn map (descriptor+88, 1cee4a6), which room admission fills in. `game: within a run the floor changes only by descending` compared whole `Floor` values, so every room entry that recorded a spawn cell failed it: 113 times in 48 cases on b319e73. The run-start contract compared the new floor with a fresh generation the same way. The properties, not the engine, were wrong: they pinned the floor's representation, not what Level::Init fixes. Their seed check also pinned every stage to its startup draw at all times, which Forget Me Now (it forgets the stage seed) would break once ported. | Both contracts now compare the generated layout: stage, seed, grid, starting room, and each room's grid cell, template and links. The layout must hold while the floor's stage and seed do. A deeper floor must take its stage's startup seed, and a run must start on stage 1 with stage 1's seed. The menus no longer pin a stage seed. | The 68 violation tapes of that run pass. Rewiring a room's links on its admission fails the layout check 14 times in 12 generated cases |
| Branching 64 cases from Basement II's start, a player died there, exited to the menus and started a new run: the run's floor was Basement I but `state.stage` stayed 2. Only the GameOver Restart reset it; `start_run`, which the menu path also takes, did not. The new run then read stage 2 for its music, doors and chests, and its Basement I boss trapdoor asked `next_stage(2)`, which fails, so every later tick failed and the browser's frame loop stopped. DST caught the stale stage through the new stage-seed contract. | Fixed in `start_run`, the single owner of a new run's first stage: it resets the stage and clears any stage transition for every new run, and the Restart's own reset is gone. The pause contract now also states that Escape is ignored while the nightmare scene is pending or playing (Game54f760), as the key handler does. `spawn_boss_award` spawns a trapdoor on stages 1–5 and 7, but `next_stage` admits only 1 → 2, so entering Basement II's own trapdoor failed every tick the same way (#413). Main fixed that in 2efbef1: a trapdoor whose next stage has no admitted generator stays open but never takes the player. | `regression-new-run-resets-stage` (2,721 commands, rebuilt on c8931a8) fails the stage-seed contract 301 times and the run-start contract once without the fix. The `chases.json` recipe rebuilds it whenever the shared MT moves it off its branch. Its run enters Duke's room at command 688, clears it, presses Escape during the nightmare scene (the pause gate), quits from the pause menu on Basement II, then starts a new run from the menus at command 2420 |
| The shared-MT stack (#398) ported the ambient Tiny Bugs, Tiny Flies and Wall Bugs. A Tiny Bug that wandered onto a pit failed the tick with `Tiny Bug pit recovery is outside supported grid objects`, and every later tick in the room failed the same way, freezing the browser's frame loop. The check assumed no admitted room places a pit, but Basement rooms do; two corpus tapes reached it within their first rooms. The same stack removed the renderer-only tear `impacts`, so the DST crate stopped compiling on main. | The pit branch now does what its source comment states: the bug returns to its previous position (Effect+440). The tear must-hit now looks for the sized TearPoof effect (1000.12, 13, 0x4f or 0x50) that replaced the impact. | `regression-tiny-bug-pit` (600 commands) fails 8 ticks without the fix |
| **Open.** A player walked down into a closed chest resting against the bottom wall. Each frame the player–pickup contact moved the chest 1.7–3.4 px down (the shift block guards only the player), and the chest's late ground pass, which does run after contacts, reflected its velocity but corrected at most max(speed, 2) px. The chest settled with its centre 10.8 px inside the wall cell. The order (4545d0 then 442b90) and the mass split (445710) follow the source contracts, so whether the original also sinks it needs a live comparison. | Tracked in #365: a contrived `chest-opening` case pushing a chest into a wall decides between a missing engine constraint and a DST property that must admit player-pushed bodies. | [`findings/player-pushes-chest-into-wall`](../engine/dst/findings/player-pushes-chest-into-wall.json) fails `game: grid-colliding bodies stay inside the room's walls` at command 379 |
| **Open.** A player holding Deck of Cards (active item 85) cleared Larry Jr.'s room and took the trapdoor. The descent's level admission saves the run (`save_supported_run`, lib.rs), and the save normalizes the player (`player_save::normalize_player`, 518120). The normalization refuses any active item other than none or Yum Heart (45), or any nonzero active state, with `Player save normalization requires an unmapped temporary/active owner`. Every later tick fails that way, so the browser's frame loop stops: reaching Basement II with almost any active item freezes the game. 8 of 64 cases branched from the Larry Jr. entry hit it on b319e73, and 9 of 64 on 720944a. The refusal dates from 8b468a5, the descent from 88ec650. | Needs the owner of the Player save boundary: 518120's active-item and BF5 handling, which the code records as an explicit source gap. | `findings/descent-with-active-item` (2,800 commands, re-recorded on 720944a) fails from command 2737 |
| A bomb that destroyed a tinted rock froze the game when the rock's 56d440 reward drew a Megatroll Bomb (40/5): every tick after it failed with `unresolved pickup selector 40/5`. The chest-loot fix had made `pickups::definition_subtype` the owner of the transient Troll and Megatroll subtypes' temporary visual definition, but the newer tinted-rock reward (50167a5) handed the raw subtype to `Pickup::spawn`, which has no 40/5 definition. | `Pickup::spawn` resolves its definition and spritesheet through `definition_subtype` itself and keeps the real subtype, so no birth site can bypass the mapping; existing callers that pre-map behave identically (every corpus tape replays to the same state hash after every command). The reward's 40/5 then converts into a Megatroll Bomb actor on its first full update, as chest loot does. | The `pickups` unit spawns every rolled 40/3 and 40/5 and checks they draw the regular bomb's visual, which fails without the fix. `regression-tinted-rock-megatroll` no longer draws the reward since the shared MT moved it off its branch, and stays in the corpus as an ordinary workload (09cc3d1) |
| The `game` driver kept its own copy of which rooms the player may enter. Main admitted large and miniboss rooms (#171, #181) but the copy did not, so DST never entered Gluttony's room. Lasers, the shared pathfinder and the laser invariants went unexercised; generated runs reached 8 of the 12 implemented NPC kinds. | `rebirth_cleanroom::traversal::admits` is the single owner of doorway admission (the engine's departure check uses it). The driver routes by it, and explorers head for the nearest unvisited admitted room. 48 generated runs then reached Gluttony and Pooters and met every `game` property without the corpus. | `game: the player is in a miniboss room`, `game: a laser is live` and `coverage-5923f07d83419608` |

## Running on Antithesis

[antithesis/](../antithesis) packages the harness for the platform:
`docker build -f antithesis/Dockerfile -t rebirth-dst .` builds the harness
and the production `engine.wasm` from a context that
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
