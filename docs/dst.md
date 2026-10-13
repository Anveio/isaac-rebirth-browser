# Antithesis deterministic simulation testing

`engine/dst/` is the engine's deterministic simulation testing (DST) harness,
built on the [Antithesis Rust SDK](https://antithesis.com/docs/using_antithesis/sdk/rust/).
It is the **only** checked-in unit-level verification in this repository: the
standing rule against Rust/Node/Python unit suites and component harnesses
(see [AGENTS.md](../AGENTS.md)) has exactly this one user-authorized
exception. Nothing else may use it as precedent.

DST establishes engine safety, determinism, snapshot completeness, native/WASM
agreement and documented input contracts. It is not original-game evidence:
parity with base Rebirth remains the job of the live e2e suites, and no DST
result may be cited as a parity result.

```sh
npm run test:dst                      # build engine.wasm + harness, run every unit
DST_CASES=48 DST_STEPS=4000 npm run test:dst     # a longer local soak
npm run test:dst -- --seed 0x1f2e3d4c5b6a7988    # rerun a whole recorded run
engine/dst/target/release/rebirth-dst --units game --case-seed 7233388093878974408 --steps 5000
engine/dst/target/release/rebirth-dst --replay .cache/dst/run-…/tapes/game-…-violation.json
engine/dst/target/release/rebirth-dst --list     # the unit registry
npm run lint:dst                      # rustfmt + strict clippy for the harness
```

## How it works

Every simulated decision draws from one entropy source
([entropy.rs](../engine/dst/src/entropy.rs)). Locally it is a seeded SplitMix64
stream: each run prints its master seed and each case its case seed, so any
case reproduces exactly. With `--entropy antithesis` every draw calls
`antithesis_sdk::random::get_random()`, letting the platform steer and replay
exploration. Units never read clocks, OS randomness or the SDK directly.

Properties ([props.rs](../engine/dst/src/props.rs)) are Antithesis assertions:
`always`, `sometimes`, `reachable` and `unreachable`. Each call forwards to the
SDK macro of the same kind, so a platform run reports every property natively.
Because the SDK never fails a process by itself, each call site also records
into a local ledger, and the run's verdict applies Antithesis semantics to the
SDK's own registration catalog: an `always` must be hit and never false, a
`sometimes` must be true at least once (the workload really reached that
branch), a `reachable` must be hit and an `unreachable` never. Unmet must-hit
properties fail a local run; under Antithesis they are judged across the whole
exploration instead. Property messages start with their unit name and are the
property identities; keep them stable.

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
| `grid` | grid queries and both ground-collision solvers on generated rooms | resolution never fails for bounded motion (including residual speeds whose squares underflow); previews keep velocity; correction moves only out along the contact normal; full nonplayer reflections leave the wall; blast candidate ordering and bounds |
| `game` | the whole engine through public commands | per-step state invariants, per-command transition contracts, exact rejection semantics, tape replay, serialized-midpoint continuation, C ABI and shipped-WASM differentials |
| `abi` | the exported C ABI the browser bridge drives | framing bounds, malformed requests, failed resets, pre-reset commands on a fresh thread |

The `game` driver ([driver.rs](../engine/dst/src/units/game/driver.rs)) only
emits commands the browser bridge can send, plus adversarial variants of them
(unsupported keys, malformed seed text, out-of-range volumes, over-budget
ticks, stale music callbacks, key repeats and releases). Each case follows one
persona (direct play, menu explorer or seeded run), sends browser-style
host-timestamped frames (with jitter, hitches and rollbacks) or controlled
steps, and steers by the published state: shortest grid routes to doors and
reachable pickups, a door-graph route toward the boss room, demolishing rocks
with bombs, aiming at NPCs and pausing. This makes ordinary local budgets reach
room traversal, combat, rock destruction and collection; Antithesis explores
further from the same decisions.

### Oracles beyond invariants

* **Replay determinism.** The case's tape replays on a fresh boot and must
  reproduce every command outcome and state digest.
* **Snapshot completeness.** The game serialized at a random midpoint and
  resumed must reach the identical final game, so no simulation state lives
  outside `Game`'s serialization.
* **Lossless publication.** Published states and whole games round-trip
  through JSON and the engine's own deserialization validators.
* **C ABI differential.** The tape drives the exported `reset`/`request_ptr`/
  `command`/`snapshot_*` functions exactly as `web/engine.ts` does.
* **Shipped-WASM differential.** With `--wasm web/generated/engine.wasm` (the
  default in `npm run test:dst`) each tape also runs through the production
  artifact in the deterministic `wasmi` interpreter; every outcome and sampled
  snapshots must equal native dispatch. Native-only properties therefore speak
  for the artifact browsers run.

## Tapes and the corpus

Game tapes use the engine's replay format (`initialSeed`, `bootEpochSeed`,
`commands`), so a finding replays unchanged through `engine/src/bin/replay.rs`
or the browser's `game.reset(seed, { bootEpochSeed })` plus `game.command(...)`.
A case that violates any property retains its tape under the run's output
directory. Every tape in [engine/dst/corpus](../engine/dst/corpus) replays
through the full `game` property set on every run, after the generated cases:

* `regression-*` tapes reproduce fixed findings and fail without their fix.
* `coverage-*` tapes are DST-generated cases that together reach every `game`
  must-hit property (boss-scene key handling, collection and other rare
  branches), so ordinary budgets never pass or fail on luck. When an engine
  change moves them off those branches, harvest new candidates with
  `--retain-all`, replay each with `--replay … --report …`, and keep a small
  covering set.

When an engine change legitimately alters a documented contract (for example
a newly implemented character runtime), update the unit's contract model in
the same change.

## Findings and engine refactors

Each finding below was fixed in the engine and keeps a guard that fails
without the fix: a corpus regression tape, a kernel property or a static rule.
Game tapes drift as the engine evolves, so a tape that stops reproducing its
finding is retired rather than kept under a regression label.

| Guard | Finding | Engine change |
| --- | --- | --- |
| `regression-locked-character-start` | Confirming a locked or unimplemented character queued a run whose start failed inside the transactional tick, so every later tick failed. In the browser the throwing tick stops the frame loop: the game froze permanently. | Character confirmation validates the selection and rejects the input, like the existing hard-mode rejection ([lib.rs](../engine/src/lib.rs)). |
| `grid: nonplayer ground resolution never fails for bounded motion` | A bomb coasting into a wall with a ~1e-26 residual velocity underflowed its f32 squared speed to zero in an interpolation preview, which the reflection guard reported as overflow, failing the tick. The preview never divides by that speed. The original game tape stopped reaching the contact once main added the run-start hold; the `grid` unit now drives the solver directly. | The zero-speed guard applies only to the full update, which divides by a reflected speed provably above 0.1 ([grid.rs](../engine/src/grid.rs)). |
| [clippy.toml](../engine/clippy.toml) `disallowed-methods`; the WASM differential on every tape | Native `f32::sin/cos` call the host C library, which disagrees with wasm32's libm on about 1.9% of inputs (77,065 of 3,992,186 sampled); a game tape diverged from the shipped WASM at step 665. That tape stopped reproducing after later main changes moved its trajectory, so the static rule now guards the fix. | [fmath.rs](../engine/src/fmath.rs) routes all engine trigonometry through the `libm` crate, bit-identical to wasm32 std on every sampled input, so the shipped artifact is unchanged while native builds (DST, replay, floor tools) now agree with it. |

## Running on Antithesis

[antithesis/](../antithesis) packages the harness for the platform:
`docker build -f antithesis/Dockerfile -t rebirth-dst .` builds the harness
and the production `engine.wasm`; the entrypoint signals `setup_complete` and
Test Composer runs the
[drivers](../antithesis/test/v1/rebirth-engine) with `--entropy antithesis`.
`--build-arg INSTRUMENT=1` adds Antithesis coverage instrumentation and needs
the platform's `libvoidstar.so` at `antithesis/libvoidstar.so`.

The binary embeds data compiled from the private original resources (room
layouts and catalogues). Pushing the image to any external registry,
Antithesis included, requires the repository owner's explicit approval. This
configuration has not yet been run on the Antithesis platform; the SDK's local
mode exercises the same assertions, and the platform's own evaluation remains
to be done.

## Extending

Add a unit as a module under `engine/dst/src/units/`, register it in
`units/mod.rs`, and prefix its property messages with its name. Prefer model
or differential oracles and properties grounded in source evidence or
documented contracts over restating implementation arithmetic. A property that
fails is a finding: fix the engine or record it explicitly, keep its tape in
`corpus/` as `regression-<finding>.json`, and never weaken the property or its
workload to pass. Run
`npm run matrix:update` after adding or renaming properties.
