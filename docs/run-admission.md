# Initial run admission clocks

The actual menu confirmation calls `59b110 -> 593470` before fresh Game
initialization. After selecting the load image, `593470` writes its own
`gettimeofday` milliseconds to System+2f2b8. The runtime stores that anchor
at the actual pending-menu dispatch, rather than at a zero-tick DOM Enter.

Fresh `54eb20` initializes Level/Save and the primary manager before
`54eaa0` sets Game+10eb14 to one, Game+10eb1c to authored f32 `.04`, and
Player+7d4 to false. This byte is `controlsEnabled`, distinct from Entity+31d
visibility. Input uses the actual shared `Player.controls_enabled` owner;
the legacy stage-title overlay does not disable movement, fire or bombs.

Only full Game `54f910` takes the initial fade guard. Signed elapsed
milliseconds must exceed 3,000 before an f32 subtraction. A host clock
rollback holds the fade. The subtraction that reaches zero restores controls
and permits Game+ce9f4 to increment and full actors to advance in that same
dispatch. Fast `5507f0 -> 550760 -> Room5cd920` continues while full updates
are held. Game age increments before normal room/grid callbacks.

The elapsed clock is separate from phase counter and active Game age. Manual
`step(count)` declares a presentation interval of 16,667,000 nanoseconds;
the fraction survives integer-millisecond conversion. An optional second
`timestampMillis` argument sends the same public timestamp through the
rendering-disabled step path; omitting it retains the original count-only
command. `trace.clockInput` retains this initial clock contract even when
state history reaches its cap.
Normal RAF ticks record an optional `timestampMillis: Date.now()` input.
It must be a nonnegative JavaScript-safe integer and requires count one.
Replay applies these actual timestamps, including backward jumps. No observed
native timestamp, age, counter or fitted freeze count enters the runtime.

The required live producer [run-admission.mjs](../tools/e2e/run-admission.mjs)
defaults to two contrived plus two unfiltered recorded OS-random cases. Each
executes 342 declared one-presentation steps across menu confirmation,
initialization, held input and idle. The original observer checks its owned
blocked process's actual executable hash and reads bounded writable fields.
The browser receives the independent public tape through actual DOM inputs.
Actual System state, f32 fade/rate, controls byte, absolute Game age, phase
parity, player world position/velocity and relative clocks remain required;
every mismatch is retained. Complete selected snapshots, commands and backend
callback states are streamed losslessly with independently recorded hashes.

The producer declares each Browser presentation's public `timestampMillis`
before launching the original. Its schedule uses only the selected boot epoch,
the original launcher's declared 30 bootstrap presentations, and the declared
Native and Browser tape intervals through `before-run-confirmation`. It
derives the Browser's first Title time from those public lifecycle boundaries;
it does not copy an observed clock or request anchor. The complete Native tape
and timestamped Browser command plan are retained in `public-clock-plan.json`.
The actual original bootstrap, presentation counts and elapsed nanoseconds must
then validate that declaration, while the Browser trace must preserve every
timestamped command exactly. Existing gameplay comparisons remain unchanged.

The first frozen four-case cohort executes all 1,368 dense phases and 12,504
checks on artifact `fa86a65280ede95da479a4ce98839ae30969fdc9b9bfdb8f42911c25bb0c55c6`.
All four gates remain failed with 640 required differences: 308 relative-clock,
100 fade, four controls-byte and 228 active-age comparisons. The two idle
cases end one full update behind the original. The opposite-phase held-input
cases match their owned fade/controls/age and physical fields, while their
relative-clock differences still block acceptance. Every selected outcome
remains required. All four original/browser input and callback histories are
complete; independent full archive readback reproduces every required error.

The [live record](../references/isaac/foundation/run-admission/first-live.json)
pins the unchanged report, exact producer, raw/archive hashes and the separate
native-validation sidecar. The sidecar additionally verifies every actual
issued/consumed X11 event, clock acknowledgement and source seed readback.
Those checks now also run in the default live gate. No recapture, native field,
report rewrite, tolerance or counter/time offset was used to strengthen them.

The [source contract](../references/isaac/foundation/run-admission/source-contract.json)
pins exact static functions and preserves the first source-only 339-state
trace, including its historical controls-byte label and declared-hash
limitation. The existing stage-name streak now starts at the one-time positive-to-zero
full-update boundary (`54fc2b -> 585500`), after that dispatch's earlier HUD
update and before active Game age and actor callbacks. It no longer expires
during the opaque startup wait. Its shared HUD clock is unchanged. The
independent `593660` input-reset callback remains explicitly pending; the
existing completion-pending flag is retained for that missing effect. This
source change does not regrade the historical live comparisons above. Saved-run Load, the omitted native
introductory cutscene and complete freeze/death/transition behavior are
separate; unmatched startup ages or clocks cannot be repaired with a baseline.
Early pause remains unaccepted: the current zero-tick Escape handler can open
pause before the initial full guard, whereas the source fade guard precedes
pause admission. The pending `593660` reset also prevents claiming general
held movement/aim semantics at the completion boundary from controls alone.

## Public clock origin mismatch (#438)

An unchanged `88b3ec75` build reproduces all 640 historical differences on
the checked-in four-case bank: 243/77/243/77 failures, with all 342 phases per
case retained. This is the longstanding baseline, not a new game regression.
Its local report and original/Browser histories remain under
`.cache/e2e/run-admission-438-main88b3ec75/run-admission/`. Replay the declared
bank with `npm run test:e2e -- run-admission --scenarios
references/isaac/foundation/run-admission/scenarios.json`; the producer now
uses the complete public clock schedule described above.

The former Browser tape started its clock at first Title, whereas the Native
tape included 30 bootstrap presentations, 180 introductory presentations and
84 through its Title checkpoint. Browser advanced 63 presentations to that
checkpoint. The omitted public interval is therefore 231 presentations, or
3,850,077,000 nanoseconds. The 77,000-nanosecond fraction matters because the
original independently truncates absolute wall time to milliseconds at the
menu request and at each full fade guard.

For the fixed even-phase tape, confirmation is Browser presentation 664 and
Native presentation 895 from their former clock origins, with fractional
milliseconds of 888,000 and 965,000 nanoseconds. After 105 presentations,
Native crosses the next integer millisecond and Browser does not, explaining
the first clock mismatch at run step 106. After 180 presentations, the
independent integer readings produce elapsed 3,001 versus 3,000 milliseconds,
explaining the first fade mismatch at step 181 and age/controls at step 229.
Fresh original observations confirm these predictions from the public tapes.

The correction supplies the same independently declared public wall time to
both games through the existing timestamp command. It changes no runtime
clock, fade threshold, f32 subtraction, age counter or comparison tolerance.
The first Title time can vary with public intro-skip input, so neither 231
presentations nor 77,000 nanoseconds becomes a runtime constant. Historical
reports retain their original failures; other live producers must establish
their own complete public clock schedule before claiming age parity.

The corrected producer passes all four replayed cases on the same immutable
`88b3ec75` artifact
`8beae6bbe77714fb59236a91510cf4fa6892be28326b645db77798f6c8a789ae`:
1,368 dense phases, the unchanged 12,504 required comparisons and zero
differences. Both actual input histories validate in every case. The local
report is
`.cache/e2e/run-admission-438-public-clock-replay-main88b3ec75/run-admission/report.json`.
This qualifies the equal-clock admission producer and that artifact. It does
not regrade the earlier captures, fix a game runtime, qualify another suite's
clock plan, or establish introductory cutscene or full-frame visual parity.

The paired [normal-death gate](player-death.md) now independently declares and
validates its own complete public clock plan, using the optional timestamped
step path. Its retained four-case selected-field gate passes on a separately
bound combined runtime; its fresh OS cohort retains one required Original
audio-observation failure. Pixel and complete audio acceptance remain open.
Other paired drivers retain the default count-only transport until their own
clock contracts are qualified.

## Unsupported character requests

The public command API rejects a locked or unsupported character when normal
Enter is confirmed, before setting `run_start_request.input_pending`. Invalid
commands already leave state and held keys unchanged. Delaying this rejection
until `start_run` would retain the deferred request in each rejected tick; an
exception there stops the browser RAF callback before it schedules the next
frame. A keyboard callback already catches a rejected confirmation input.

Random admission requires a nonempty unlocked list whose possible choices
all have supported runtimes. Pending menu initializers can change the seed
between confirmation and deferred startup, so validating just the choice from
the current seed would not guarantee a startable request. The deferred-start
validation remains in place. Fresh profiles support Isaac.

This is API error recovery, with static validation only. It does not establish
the original locked-character menu behavior, other character gameplay, Random
selection parity, or a passing fresh two-game recovery scenario.
