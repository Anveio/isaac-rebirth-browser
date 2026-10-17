# Normal Isaac death

The normal Player damage callback now carries the admitted caller's damage
flags into Kill and Death. Both required public routes execute in actual
original/browser pairs. The retained four-case selected-field gate now passes
with the corrected public clock on the combined candidate below. The two new
unfiltered OS cases remain **FAIL** on one required Original audio observation.
Visual acceptance is still open: the registered Restart replay retains 733 of
740 full-RGBA diagnostic failures. The bounded Exit music/SFX contracts below
now pass; complete audio acceptance remains open. Revival items,
other characters, cooperative players, black hearts, and complete diary content
remain unimplemented.

| Contract | Original evidence | Port state | Live acceptance |
| --- | --- | --- | --- |
| Health-depleted damage and NO_KILL bit1 | Player52b080/52c675/52cbc9 | Explicit admitted damage branch; no global zero-HP death trigger | Pending |
| Normal zero-gib Kill | Entity4464c0/445c60/44622e | SFX30 volume f32.8, delay2; own Shared raw seed, real Blood7 and immediate Update | Pending |
| Death animation and motion | Player531850/531b24/531bb6; RoomFast5cd920 | Base carried motion precedes the dead branch and velocity reset on both Player callbacks; callbacks entered dead retain the source ground query; ANM advances only on full Base callbacks | Required-route position/velocity and Death frame/playing/guard agree; the complete gate remains red |
| Finish and GameOver | Player52e770; Manager5bf8f0; Popup558900 | Hide body, immediate Music93 and queue8, popup state2 | Actual hidden/dead/popup2 and stopped Diary0 agree in both required routes |
| Popup clock fence | Game550760 | Gameplay freezes; released System input and music continue | Pending |
| Public actions14/15 | Keyboard62b520/620440/62bb90; Popup557650 | Per-key default Space/Return/Escape edges; deferred System requests | Pending |
| Restart/Exit dispatch | System594710/594a10/5945f0/594cb0 | Normal seeded restart retains queued seed; unseeded restart uses own Shared; Exit selects Main menu | Pending, including save/reset interactions |

RoomFast special-cases type1 Player: it sets entity flags318 bit0x4000, invokes
virtual18 (531850), runs443d00(false), and clears the bit. The generic
virtual20 slot is not the actual fast Player dispatch. Base447190 skips main
ANM advancement under4000, while the Player dead branch still clears velocity
and returns before ordinary cooldown, movement, active-item and shot callbacks.
The independent animation guard bf4 does not blanket-disable movement.

Two original-only public spike traces from recorded seed22PFCSFS observed
Death0 at presentation1601/Game231, Death0 at the next fast presentation,
Death1 at the next full presentation, and popup2 by presentation1777/Game288.
The gap before that popup observation does not establish its exact admission
phase. Hurt countdown120 stays120 during Death and popup, while controls7d4
remain enabled. These observations do not establish browser parity or audio
backend parity.

The authored Diary is stopped by SetFrame0; it has no invented timer. Exit and
Restart widgets retain authored frame0. The renderer currently draws the
ordinary page and the spikes cause frame66; remaining portrait, stage, enemy,
item and boss diary selectors are tracked as incomplete. Pixel parity is
unaccepted. Paired verification must keep functional failures blocking and
retain the public input, audio/backend history and immutable build identities.

The historical complete fresh four-case cohort uses reviewed source `5afa7dd` and immutable
production `c4685585`. Both public Restart and Exit reach their actual destination
in both games; both retained OS cases remain alive. All 2,962 presentations and
52,190 selected checks execute without capture errors. The gate retains 4,226
required failures: 2,774 Game-age, 1,097 Y, 115 VY and 240 Exit-title/fade fields.
Eight complete manifest streams were independently decompressed and hash/byte
verified (830,681,575 raw bytes). The pinned proof is
[first-complete-live.json](../references/isaac/foundation/player-death/first-complete-live.json).
The historical aborted cohort and unused builds remain preserved separately.

Base447190 applies temporary friction and carried velocity before the dead
Player callback clears it. A lethal ordinary Spike hit still returns true from
Player damage and receives the four-unit radial impulse; suppressing that
impulse would omit source behavior. The dead callback now integrates native
f32 position before clearing velocity, without ordinary controls or immunity
advancement. RoomFast runs the retained ground query after that callback.
In the historical guarded493 leaf, a full Player callback entered dead skipped
the existing living-motion query, so the full manager restored its query after
contacts. The Player Full-grid change below relocates the ordinary living query too. Common Kill
retains Isaac's class5 and collision samples, so zero velocity still permits
bounded penetration correction. The existing player collider supplies that
query.

The current [kept public-input bank](../references/isaac/cases/player-death/recovery-dead-player-base-motion.json)
has SHA256 `c2fec061861dba7f2ad432c1e7cbc2253edca2706dfc3c0a39740f52a39f2ebb`.
All 2,962 checkpoints and 52,190 canonical state checks execute. Original
backend/input provenance and fresh DOM Browser captures pass for all four
cases; both required routes actually die and reach popup2, while both
unfiltered OS rows remain alive. The same validated Original bank is reused
for each immutable Browser artifact.

Against main88b3 production
`8beae6bbe77714fb59236a91510cf4fa6892be28326b645db77798f6c8a789ae`,
guarded source2589dd2e production
`d1a9b8d7703ecc755791570a5e1a06f2549e95e16e40f7e251bda578088ad185`
removes 517 Y failures (Restart258, Exit259), with no regressed fields, changed
passing values or changed remaining failures. All 52,190 selected fields agree
with the earlier unguarded production
`0331a7ea6c716cdbf5bdc994dd4def92346374a0fad18e84cf1b483fccf5fcff`.

At that unchanged-clock guarded493 checkpoint, the gate remained **FAIL**
with 3,014 strict failures: 2,774 Game-age, 80
Exit-title-frame, 80 Exit-fade-color, 40 Exit-fade-active and 40 Exit-fade-alpha.
The guarded A/B replay uses the unchanged canonical capture, Original
validator, bank/death coverage guards and field comparer. Full actual Browser
PNGs, manifests and input/backend traces are retained; RGBA differences are
explicitly not computed for this replay. The complete RGBA diagnostics from
production0331 remain preserved and red, without regrading or a pixel-parity
claim.

After the format-only main merge, source3028eb11 production
`524351dbbd078b0fc62980668f633a36474b12e597c8a3a2a4c4affdf1a92781`
also passes fresh DOM Browser capture and Original validation for the same
26-checkpoint lethal prefix. All 329 selected fields agree with production0331;
the seven baseline Y failures stay fixed and the 19 existing Game-age failures
stay red. This final-artifact check supplements the complete cohort above.

Replay the kept bank through `player-death --scenario-file
references/isaac/cases/player-death/recovery-dead-player-base-motion.json`;
an explicit `--case death-spikes-restart` selects its focused required route.

Deferred GameOver Exit now restores the fresh menu presentation owned by
System594cb0. Run entry destroyed the prior menu controller; MenuInit59ab7c
therefore initializes the offscreen Title actor at frame0, while 59ad17 and
StartFadeIn59b0f0 initialize alpha1 and the request's opaque-black color.
The same System dispatch subtracts f32.025 before presentation. Returning to
Title within an existing menu-controller lifetime still retains its clock.

A focused recovery replay of the same `death-spikes-exit` public case uses
production
`46a4d7068777e616264b2f1c814d5f3d072a84ac02ae04e3e8d751aa96ff5d43`.
This artifact contains committed guarded493, the separately bound ordinary-bomb349
candidate, and the nine-line Exit presentation change. Actual DOM Browser
capture and independent Native provenance pass; all 741 checkpoints execute,
and the required route actually dies and reaches popup2. Against the d1a9b8d7
Exit replay, exactly 240 of 13,140 canonical fields improve: 80 Title frames,
80 fade colors, 40 fade-active fields and 40 fade-alpha fields. The other
12,900 field rows are identical, with zero
regressions or additional changes. The case remains **FAIL** on the same 654
Game-age checks. The canonical default step transport and field comparer are
unchanged; all 741 actual Browser PNGs, full manifests and input/backend traces
remain local. RGBA differences were not computed for this focused replay.
The aggregate state-history trace reaches its recorded byte cap; all 741
independent checkpoint states remain complete in the manifest.

At that visual checkpoint, Exit audio remained a separate source gap: fresh
MenuInit requests Music61 and SetMenu0-to3 emits no forward SFX8, while the
port requested Music1 and SFX8. Its SFX30 checks did not qualify those backend
contracts. The historical four-case results above retain their original reds.

The Exit path now requests Music61 and omits that forward-confirm SFX8 request.
A separate seven-capture public prefix first reproduces the audio gap on
production703: all 95 canonical fields pass, but only 10 of 14 bounded audio
checks pass. Native has zero uniquely attributed configured SFX8 starts in
the Exit dispatch window versus one actual Browser request/API start; Native
Music61 differs from Browser Music1 at dispatch1, dispatch2 and dispatch4.
Both runtimes retain three earlier actual SFX8 starts as positive controls.

The identical tape and declared public clock then pass against a fresh Native
and immutable production
`f837cf593084788c7ab9ebcc135458c0d55fbc52afbd4e3c378d6ab9357b10f8`
(1,407 files; `check:build` passed), containing only the separately bound Exit
audio leaf beyond its recorded runtime base. All 95 canonical rows remain
identical to production703, including Native expectations. All 14 audio checks
pass: the four baseline failures become passing, while the ten prior-passing
rows are unchanged. Music61 agrees at all three dispatch captures; both games
have zero actual SFX8 starts in the window, and the Browser emits no SFX8
request there. All three earlier unique-owner SFX8 controls remain mandatory
in each runtime; Browser controls reach the actual output graph with retained
PCM hashes. SFX volume is explicitly one in both games, with the Native slot
loaded and past cooldown. This is actual owner/backend evidence, not a
negative inference from muted slot bytes.

Natural death/popup coverage, fresh Native provenance and 690 clock
acknowledgements/eight capture times pass; the Browser records all 1,631 public
ticks and seven complete capture states. Input and backend traces are
untruncated. Aggregate state history reaches its recorded byte cap after 409
states; all seven independent checkpoint states and 15 actual Native/Browser
PNGs remain retained. RGBA differences were not computed for that initial
prefix capture. These bounded
controller/request/start checks do not establish audible or PCM equality,
complete music lifecycle/device equivalence, complete Shared-MT parity or a
count of rejected Native request attempts. The separate fresh-OS SFX30
observation failure and historical pixel failures remain unchanged.

A subsequent registered `player-death --exit-audio` replay on the same frozen
f837 artifact passes all seven checkpoints, 95 canonical fields and 14 audio
checks against a fresh Native. Those 95 and 14 field rows are identical to the
prior f837 capture. Mandatory actual Exit-to-Main execution, zero observed
forward SFX8 events/starts and both sets of three earlier controls pass.
Native provenance and 690 clock acknowledgements/eight capture times validate;
the Browser retains all 1,631 ticks and seven capture states. Input, backend
and actual API traces are untruncated; aggregate state history retains the
same documented byte cap. All seven strict RGBA diagnostics are now computed:
Title-ready and dispatch1/2/4 are exact, while gameplay-start, death-settled and
request-1 remain red (four exact passes, three failures). The initial prefix's
uncomputed pixels and the separate historical pixel failures are not regraded.

The [kept seven-capture input bank](../references/isaac/cases/player-death/recovery-death-exit-audio-prefix.json)
contains only the public case, without local observation paths. Replay it with
`player-death --exit-audio --scenario-file references/isaac/cases/player-death/recovery-death-exit-audio-prefix.json`.
The explicit mode preserves canonical fields, public-clock validation and
mandatory death and actual Exit-to-Main coverage, and repeats all 14 bounded
actual owner/backend checks with both sets of three earlier SFX8 controls and
zero observed Browser forward SFX8 events. It rejects a full tape or other
selected routes before launching either game; no inputs are silently thinned. Pixel diagnostics are computed and retained separately. Omitting
`--exit-audio` replays only canonical death fields and pixel diagnostics; it
does not qualify the supplementary audio contracts from snapshots alone.

## Player Full grid after contacts

The Player Full-grid change defers the ordinary Full Player ground query until
entity contacts have completed. The retained Player is queried first at the
manager's final fence (454b1d–454b2f), before the other final ground queries.
Living movement and actions therefore see provisional vectors, including
new Bomb/Tear births. A callback entered dead, or a Player killed later during
contacts, receives that same single query; class0 keeps its existing bypass.
The Fast query and standalone transition callbacks retain their existing paths.
This source change is separate from the historical guarded493 qualification.
This qualification covers the Full-grid change and its listed controls.

The exact two-file patch `4ec7d9784403db5663218f97598b37ec45eb03f66df8a0f6d97143c9a3ad5b66`
is frozen in candidate
`9841c7de5255936b0052488c8280eff0adf8b2cf0a413e37f70422f403a5bfca`.
Its actual Browser parent is integrated production
`5c33bb75136bab9b5052ccae337d40aad3d958a07e48bc117183375b115f061f`
(1,407 files each), rather than the older bf303 or 703 artifacts. Four unchanged
public tapes reuse their independently validated Native captures; no new
Native, entropy or clock schedule was introduced.

All 266 checkpoints and 15,510 scalar keys/Native expectations are retained.
Passing fields improve from 15,436 to 15,472: 17 Bomb positions, 17 Bomb
velocities and two Tear positions become passing. There are **36 gains**, zero
lost prior passes and zero changed remaining failure values. The original
**40-gain prediction remains FAIL**. All 38 scalar failures remain required:
34 Bomb birthGameUpdate rows (17 per phase), plus tears.alive and
tear.admission at Full grid-producer-held-2 and grid-producer-held-3 (four rows,
Native 0/Browser 1). Both Full coverage guards pass; both Fast cases retain their
required same-birth Native solid-response failure. The complete bank remains
**FAIL**. Canonical checkpoint passes are 230/266; strict RGBA remains 28 exact
passes and **238 failures**. Of the Browser PNGs, 232/266 are byte-identical to
the actual parent. Source, artifact and cached Native identities stay stable.

The same-Native Shop v9 control against that actual 5c33 parent preserves all
1,190 historical flat-comparer rows, their Native expectations and actuals:
1,068 pass and 122 remain red, with zero gains/losses or changed remaining
failures. Its separately owned 117 source and 390 typed fields pass; all 59
Browser PNG bytes are identical (the retained RGBA diagnostics remain seven
exact and 52 red). This does not regrade the corrected nested Shop comparison
or qualify complete Shop parity.

Two fixed declared-clock death controls additionally capture both actual 5c33
parent and candidate9841 against the same unchanged current-clock Native
references. Restart passes 740 checkpoints/13,837 fields and Exit passes
741 checkpoints/13,140 fields on both builds: **26,977/26,977 fields** and
1,481 checkpoints per side, with zero changed keys, Native expectations,
actual values or prior-pass losses. Both required death/popup guards, Native
provenance/acknowledgement/capture validation and actual Browser public tick
plans pass. All 1,481 Browser PNG bytes match the actual parent; 2,962 full
PNGs remain retained and SHA-verified. RGBA differences are **not computed**
for this control. Production703 remains separate historical clock evidence.

The four input-only cases were kept with the unchanged canonical case:keep
command and preserve their exact source timing declarations:

- [Bomb Full](../references/isaac/cases/paired/player-full-grid-bomb-phase-0.json)
- [Bomb Fast](../references/isaac/cases/paired/player-full-grid-bomb-phase-1.json)
- [Tear Full](../references/isaac/cases/paired/player-full-grid-tear-phase-0.json)
- [Tear Fast](../references/isaac/cases/paired/player-full-grid-tear-phase-1.json)

For example, replay public inputs and ordinary paired checks with
`npm run test:e2e -- paired --replay-cases references/isaac/cases/paired/player-full-grid-bomb-phase-0.json`.
The additional grid-timing comparer and mandatory same-birth coverage laws
remain bound to the retained scoped capture harness; keeping an input case
does not promote those extra guards into the ordinary paired driver. No new
framework or comparison law is introduced here. Complete physics, near-solid
late-contact death, Shared-MT, audio/PCM and pixel acceptance remain open.
NO_KILL, held-arrow, Soul health, revival and full diary art are not accepted
by these controls. No counter or position is copied from Native into WASM.

The player-death producer's earlier scoped clock qualification follows below.


The paired death producer now declares its complete public wall-clock plan
before either capture. It requires the full Native tape to equal the declared
source-only prelude plus paired suffix, and derives Native entry time from
that prelude and the launcher's independently declared 30 bootstrap
presentations and 16,667,000-nanosecond interval. Actual Native epoch,
bootstrap, interval, acknowledgements and capture times must validate that
prior declaration. They never choose Browser timestamps. Native evidence
loading and clock validation stay deferred until Browser capture has closed.

The shared paired gate keeps this schedule off by default; only this driver
opts in. Clocked steps send the predeclared count-one timestamps through
`game.step(1, timestampMillis)` without drawing. The complete actual command
trace must equal the declaration. Omitting the optional timestamp retains the
previous count-only bulk command. Splitting a group into count-one commands
changes command/event batching; the qualification does not claim a
transport change limited to timestamps. Field comparisons, required source
observations, mandatory death coverage and existing pixel laws are unchanged.

Production
`70314c6fae40e3001df91b48b6af56f7ea6fcf2e1aa4f3aef6e53cb11ed34a9c`
contains guarded493 main `a0fed798`, the separately owned ordinary-bomb349 and
visual Exit candidates, and the optional public step timestamp. Its source
binding and complete patch were recorded before the immutable copy. The
capture producers are bound separately to their recorded source hashes;
this is a combined-runtime qualification, not an isolated clock-runtime fix.
`check:build`, including typecheck, passed.

On that production artifact, the actual registered driver reruns the kept
`death-spikes-restart` case with a fresh independent Original: all 740
checkpoints and 13,837 canonical fields pass, with actual death/popup coverage,
Native provenance and 765 clock acknowledgements/741 capture times validated.
All actual Browser inputs/backend trace checks pass. Every full-RGBA diagnostic
is computed and retained: seven exact passes and **733 exact failures**.
These diagnostics do not establish visual acceptance.

A separate export-only wrapper calls the unchanged canonical capture and
clock-validation bodies, bank admission, field comparer and required-death
coverage for all four kept cases. All 2,962 checkpoints and 52,190 fields pass;
both required routes actually die, while both retained OS rows remain alive.
Against the earlier d1a9b8d7 cohort there are zero changed Native expected
values and zero prior-pass regressions: 2,774 Game-age failures and the
separately owned 240 Exit-title/fade failures become passing. Complete actual
Browser PNGs, manifests and traces remain local. This faster cohort explicitly
does not compute pixel differences; the registered pixel reds remain red.

One new canonical generator invocation records both unfiltered OS draws,
`WKGV166F` and `7KMTELQG`, with complete invocation/log and bank SHA256
`4875ada16b40c56155c7de02badb5ede4a3160a4dc213589fc6020d24a56feca`
before fresh Native/Browser captures. Both input/provenance and complete clock
plans validate. The first case passes 740/740 checkpoints and 12,517 fields.
The second passes 740/741 checkpoints and 12,679 emitted fields, but remains
**FAIL** at `hurt-89`: the required Original SFX30 observation has status
`fail` and no volume/slot readback. All 25,196 emitted scalar comparisons pass;
that does not qualify the rejected checkpoint. Both draws, the failed source
observation and the cohort's failing status are retained without resampling,
substitution or a status waiver.

The complete generated four-case [public bank](../references/isaac/cases/player-death/recovery-death-public-clock-fresh-os.json)
retains the same SHA256 above. Replay only the two fresh OS rows through
`player-death --scenario-file references/isaac/cases/player-death/recovery-death-public-clock-fresh-os.json --case 'death-unfiltered-*'`.

The audio observer's original exception was discarded by the historical death
observer projection; it is unavailable in that capture. Future observations
retain that existing error field, without changing status or comparison laws. Its failure occurs before the audio report publishes
its global-volume/table readback; the retained evidence cannot conclusively
identify a read/stop failure versus an early source-value assertion. No
source audio-behavior mismatch, transient-read diagnosis or runtime-clock
regression is established by this source-only required observation failure. SFX30
snapshot checks do not establish complete backend/music parity. The two
existing Browser runtime tests also pass on this frozen artifact using the
default count-only step API; those UI tests make no Original-parity claim.
