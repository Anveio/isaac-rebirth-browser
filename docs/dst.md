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
| `floor` | Basement generation and room admission | determinism; footprints; reciprocal edge-adjacent neighbors; reachability; exactly one boss/super secret/shop/treasure (the source retry rule); retry continuity; every descriptor admits as a `Room`, and so does a uniformly chosen compatible authored room swapped into each descriptor (same catalogue, type and shape, doors available, stage difficulty), which admits about half again as many distinct authored rooms per run |
| `grid` | grid queries and both ground-collision solvers on generated rooms | resolution never fails for bounded motion (including residual speeds whose squares underflow); previews keep velocity; an underflowing inward preview skips refinement (source 443117 → 44346c); correction moves only out along the contact normal; full nonplayer reflections leave the wall; blast candidate ordering and bounds |
| `item_pools` | the run-owned ItemPool manager (`docs/item-pools.md`) | selection never offers a collectible twice (Breakfast aside), comes from its pool, the treasure fallback, Breakfast or the Bible, and advances none of the manager's streams; depletion only lowers weights and fills the removed/selected maps; a zero-counter counterfactual isolates Rosary's Bible override and the candidate it marks; drains reach Breakfast within the catalog; locked collectibles survive every redraw; error rooms route with one shared draw and special rooms to their namesake pools; JSON continuation; rejections without mutation |
| `game` | the whole engine through public commands | per-step state invariants, per-command transition contracts (menus, run-start hold, pause, boss scenes, death and GameOver), exact rejection semantics, tape replay, serialized-midpoint continuation, the C ABI, the shipped WASM |
| `abi` | the exported C ABI the browser bridge drives | framing bounds, malformed requests, failed resets, pre-reset commands on a fresh thread, the published key surface |

The `game` driver ([driver.rs](../engine/dst/src/units/game/driver.rs)) only
emits commands the browser bridge can send, plus adversarial variants of them
(near misses of public keys, malformed seed text, out-of-range volumes,
over-budget ticks, stale music callbacks, key repeats and releases). Each case
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
clear it in 7 of 32 runs, and from a Gluttony entry in 8 of 32. Dingle still
wins every fight it starts.

Every report carries a coverage census: the room types, NPC kinds, pickups and
game features (owned collectibles, secret reveals, boss clears) each case
reached, including each owned weapon collectible (`weapon:<id>`). The summary
line names the implemented NPC kinds no case simulated
(from `rebirth_cleanroom::implemented_npc_kinds`) and the room types that
generated floors contained but no case entered. Unsimulated kinds are split
by whether any authored Basement or special room places them
(`unplacedNpcKinds`): a kind no room places, such as one from a later stage,
is reachable only when another entity spawns it, so it is not a driver gap. The census is a measurement,
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
  and the shipped WASM on every run. Game tapes drift as the engine evolves;
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
the same harvest refreshes it.

`--resume TAPE` makes every generated `game` case replay that tape (with
every check) and then drive on from its end, so exploration branches from an
interesting state, as Antithesis does, instead of rediscovering it. Cut a
harvested tape at, say, a boss room entry, resume many cases from it, and the
census shows how often each branch clears the room; a clearing branch is a
short coverage tape.

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
| Dip217 room actors (#127) are NPC collision class 4, but the NPC-pair and tear contact responses rejected type 217, so any room whose Dips touch (authored Basement rooms place them 40 px apart) failed every tick and stopped the browser's frame loop. | Fixed on main (#146): Dips take the normal-class `49b8f0` callback and `445710` mass response recorded in the Dip source contract. | `regression-dip-contact-freeze` |
| A Random run stores its resolved character in the menu's selected character. Since the GameOver popup can exit to the menus (#111), the Character menu then highlighted Random while confirming started the previous run's character, and the menu invariant failed. Latent while only Isaac is implemented. | The Exit path reloads the menus' character from the profile's selection, as Profile → Main already does. | `regression-random-run-exit-menu-character` |
| A bomb coasting into a wall with a ~1e-26 residual velocity underflowed its f32 squared speed to zero in an interpolation preview, which the reflection guard reported as overflow, failing the tick. | Fixed on main (#118), following the native early return: a zero-speed preview skips refinement. An earlier guard-only fix kept the tick alive but continued into refinement, unlike the original. | `grid: nonplayer ground resolution never fails for bounded motion` fails without any fix; `grid: an underflowing inward preview skips refinement` goes unmet under the guard-only fix. |
| Native std float functions call the host C library, while wasm32 std uses Rust's `libm`. Against `libm`, glibc 2.39 disagreed on 162,416 of 10,405,090 sampled `f32` `sin`/`cos` inputs and on 446,514 of 10,000,000 sampled `f64` `atan2` pairs. Generated tapes diverged from the shipped WASM by one ulp of NPC motion; the corpus tape's first difference is an Attack Fly's wander velocity (`npcs.rs`) at step 749, so native replay and tools could disagree with the artifact. | [fmath.rs](../engine/src/fmath.rs) routes the engine's `sin`, `cos` and `atan2` through the `libm` crate. wasm32 std matched it on every sampled input under Rust 1.97 and 1.99, so the shipped artifact computes the same results while native builds now agree with it. [engine/clippy.toml](../engine/clippy.toml) disallows every std float function whose precision Rust leaves unspecified, so `npm run lint` flags new host calls; a newly needed function gets a `libm` wrapper in `fmath.rs`. This makes no original-game parity claim. | `regression-native-wasm-fly-trig`, the shipped-WASM differential on every tape, and the lint rule |
| Engine validation, the browser and DST each kept their own copy of the accepted key list, so new keys drifted. | Fixed on main (#122): `rebirth_cleanroom::input::PUBLIC_KEYS` is the single owner, published through the `public_keys` exports. | `abi: the public_keys export publishes exactly the engine's key surface`. |
| Main reproduces the original's 0/0 boss bar fill during a boss's first two frames (83112e8): the fill is NaN, published as `null`. `BossBar` kept a derived equality, and NaN never equals itself, so whole-state equality stopped being reflexive while the bar held NaN. Every "unchanged" or "same game" comparison then failed: lossless round trips, ignored boss-scene keys, replay and midpoint continuation. Ordinary runs on main reached that window after the grid changes of #224 and #225. | `BossBar` equality treats any two NaN fills as the same bar, without comparing NaN bits, which native and wasm32 may produce differently. Lossy round trips now report the first differing JSON pointer, or where the values' `Debug` forms differ when the JSON agrees. | `game: state equality is reflexive` and `regression-boss-bar-nan-equality` |
| **Open.** Killing a Dip while the player owns Gimpy (225) or Infestation 2 (234) fails the tick with `Dip death requires the owned225/234 source callbacks`, and every later tick fails the same way, so the browser's frame loop stops. The item pipeline grants these items, but the Dip death callbacks they need are not reconstructed. DST reached it once the driver collected items and fought rooms. | Tracked in #295 for the Dip death owner (#128): reconstruct the 225/234 death callbacks from source, or keep those items out of reach until then. DST does not fake the callbacks. | [`findings/dip-death-with-gimpy`](../engine/dst/findings/dip-death-with-gimpy.json) fails `game: valid Tick commands always succeed` at command 5819 |
| A chest that dropped a Troll or Megatroll Bomb (pickup 40/3 or 40/5) froze the game. Every tick after it failed with `moving pickup requires authored collision definition`. The loot is born moving, so the newborn grid pass looked up its collision definition by the transient subtype, which has none. Three spawn sites already mapped these subtypes to the bomb pickup's temporary visual definition, but the grid and animation lookups did not. | `pickups::definition_subtype` is the single owner of that mapping. Every spawn, animation and grid lookup for a live pickup resolves through it, as the transient subtypes' Init already documents. | `regression-chest-troll-bomb-loot` (380 commands) |
| The `game` driver kept its own copy of which rooms the player may enter. Main admitted large and miniboss rooms (#171, #181) but the copy did not, so DST never entered Gluttony's room. Lasers, the shared pathfinder and the laser invariants went unexercised; generated runs reached 8 of the 12 implemented NPC kinds. | `rebirth_cleanroom::traversal::admits` is the single owner of doorway admission (the engine's departure check uses it). The driver routes by it, and explorers head for the nearest unvisited admitted room. 48 generated runs then reached Gluttony and Pooters and met every `game` property without the corpus. | `game: the player is in a miniboss room`, `game: a laser is live` and `coverage-5923f07d83419608` |

## Running on Antithesis

[antithesis/](../antithesis) packages the harness for the platform:
`docker build -f antithesis/Dockerfile -t rebirth-dst .` builds the harness
and the production `engine.wasm` from a context that
[Dockerfile.dockerignore](../antithesis/Dockerfile.dockerignore) limits to the
engine build's inputs; the entrypoint signals `setup_complete` and
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
documented contracts over restating implementation arithmetic. When an engine
change legitimately alters a documented contract, update the unit's model in
the same change. Fix or record every violation and never weaken a property or
its workload to pass. Run `npm run matrix:update` after adding or renaming
properties.
