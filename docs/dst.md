# Antithesis deterministic simulation testing

`engine/dst/` is the engine's deterministic simulation testing (DST) harness,
built on the [Antithesis Rust SDK](https://antithesis.com/docs/using_antithesis/sdk/rust/).
It is the **only** checked-in unit-level verification in this repository: the
standing rule against Rust/Node/Python unit suites and component harnesses
(see [AGENTS.md](../AGENTS.md)) has exactly this one owner-authorized
exception. Nothing else may use it as precedent.

DST establishes engine safety, determinism and documented input contracts. It
is not original-game evidence: parity with base Rebirth remains the job of the
live e2e suites, and no DST result may be cited as a parity result. It is an
inner-loop regression tool, not a merge gate: neither `check:build` nor any
pre-merge check runs it. Run it while changing engine core code.

```sh
npm run test:dst                                  # build the harness, run every unit
DST_CASES=48 DST_STEPS=4000 npm run test:dst      # a longer local soak
npm run test:dst -- --seed 0x1f2e3d4c5b6a7988     # rerun a whole recorded run
engine/dst/target/release/rebirth-dst --units game --case-seed 7233388093878974408 --steps 5000
engine/dst/target/release/rebirth-dst --replay .cache/dst/run-…/tapes/game-…-violation.json
engine/dst/target/release/rebirth-dst --list      # the unit registry
npm run lint:dst                                  # rustfmt + strict clippy for the harness
```

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
| `floor` | Basement generation and room admission | determinism; footprints; reciprocal edge-adjacent neighbors; reachability; exactly one boss/super secret/shop/treasure (the source retry rule); retry continuity; every descriptor admits as a `Room` |
| `grid` | grid queries and both ground-collision solvers on generated rooms | resolution never fails for bounded motion (including residual speeds whose squares underflow); previews keep velocity; an underflowing inward preview skips refinement (source 443117 → 44346c); correction moves only out along the contact normal; full nonplayer reflections leave the wall; blast candidate ordering and bounds |
| `game` | the whole engine through public commands | per-step state invariants, per-command transition contracts (menus, run-start hold, pause, boss scenes, death and GameOver), exact rejection semantics, tape replay, serialized-midpoint continuation, the C ABI |
| `abi` | the exported C ABI the browser bridge drives | framing bounds, malformed requests, failed resets, pre-reset commands on a fresh thread, the published key surface |

The `game` driver ([driver.rs](../engine/dst/src/units/game/driver.rs)) only
emits commands the browser bridge can send, plus adversarial variants of them
(near misses of public keys, malformed seed text, out-of-range volumes,
over-budget ticks, stale music callbacks, key repeats and releases). Each case
follows one persona (direct play, menu explorer or seeded run), sends
browser-style host-timestamped frames (with jitter, hitches and rollbacks) or
controlled steps, and steers by the published state: shortest grid routes to
doors and reachable pickups, a door-graph route toward the boss room,
demolishing rocks with bombs, aiming at NPCs, pausing, and answering the
GameOver popup.

Beyond its invariants, every `game` tape is checked by:

* **Replay determinism.** The tape replays on a fresh boot and must reproduce
  every command outcome and state digest.
* **Snapshot completeness.** The game serialized at a random midpoint and
  resumed must reach the identical final game, so no simulation state lives
  outside `Game`'s serialization.
* **Lossless publication.** Published states and whole games round-trip
  through JSON and the engine's own deserialization validators.
* **C ABI differential.** The tape drives the exported `reset`/`request_ptr`/
  `command`/`snapshot_*` functions exactly as `web/engine.ts` does.

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
  must-hit property (boss-scene key handling, GameOver, collection and other
  rare branches), so ordinary budgets never pass or fail on luck. When an
  engine change moves them off those branches, harvest candidates with
  `--retain-all`, replay each with `--replay … --report …`, and keep a small
  covering set. Game tapes drift as the engine evolves; a regression tape that
  stops reproducing its finding is retired rather than kept under its label.

## Findings

A property that fails is a finding. DST shows that the engine misbehaves; the
fix must follow source evidence rather than merely silence the property.

| Finding | Resolution | Guard |
| --- | --- | --- |
| Confirming a locked or unimplemented character queued a run whose start failed inside the transactional tick, so every later tick failed and the browser's frame loop stopped. | Fixed on main (#120): character confirmation validates the selection and rejects the input with state unchanged. | `regression-locked-character-start` |
| Dip217 room actors (#127) are NPC collision class 4, but the NPC-pair and tear contact responses rejected type 217, so any room whose Dips touch (authored Basement rooms place them 40 px apart) failed every tick and stopped the browser's frame loop. | Fixed on main (#146): Dips take the normal-class `49b8f0` callback and `445710` mass response recorded in the Dip source contract. | `regression-dip-contact-freeze` |
| A Random run stores its resolved character in the menu's selected character. Since the GameOver popup can exit to the menus (#111), the Character menu then highlighted Random while confirming started the previous run's character, and the menu invariant failed. Latent while only Isaac is implemented. | The Exit path reloads the menus' character from the profile's selection, as Profile → Main already does. | `regression-random-run-exit-menu-character` |
| A bomb coasting into a wall with a ~1e-26 residual velocity underflowed its f32 squared speed to zero in an interpolation preview, which the reflection guard reported as overflow, failing the tick. | Fixed on main (#118), following the native early return: a zero-speed preview skips refinement. An earlier guard-only fix kept the tick alive but continued into refinement, unlike the original. | `grid: nonplayer ground resolution never fails for bounded motion` fails without any fix; `grid: an underflowing inward preview skips refinement` goes unmet under the guard-only fix. |
| Native `f32::sin/cos` call the host C library, which disagrees with wasm32's libm on about 1.9% of sampled inputs (77,065 of 3,992,186). On current main a generated tape diverges from the shipped WASM at step 499 by one ulp of a Fly's velocity (`npcs.rs` trigonometry). | Open. Native replay and the shipped WASM can disagree wherever engine trigonometry runs; the WASM differential lands with a fix routing engine trig through one implementation. | The shipped-WASM differential (next slice). |
| Engine validation, the browser and DST each kept their own copy of the accepted key list, so new keys drifted. | Fixed on main (#122): `rebirth_cleanroom::input::PUBLIC_KEYS` is the single owner, published through the `public_keys` exports. | `abi: the public_keys export publishes exactly the engine's key surface`. |

## Roadmap

The harness lands in small slices. Still to come:

* **Native/WASM agreement**: every game tape also replays through the
  production `engine.wasm` in a deterministic interpreter and must agree with
  native dispatch, together with a fix for the open trigonometry finding.
* **Antithesis packaging**: an image whose entrypoint signals
  `setup_complete` and Test Composer drivers running `--entropy antithesis`.
  The binary embeds data compiled from the private original resources, so
  pushing an image to any external registry, Antithesis included, requires
  the repository owner's explicit approval.

## Extending

Add a unit as a module under `engine/dst/src/units/`, register it in
`units/mod.rs`, and prefix its property messages with its name. Prefer model
or differential oracles and properties grounded in source evidence or
documented contracts over restating implementation arithmetic. When an engine
change legitimately alters a documented contract, update the unit's model in
the same change. Fix or record every violation and never weaken a property or
its workload to pass. Run `npm run matrix:update` after adding or renaming
properties.
