# Normal Isaac death

The normal Player damage callback now carries the admitted caller's damage
flags into Kill and Death. Both public death routes now execute in fresh
original/browser pairs; the complete four-case gate remains red. Revival items,
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

The complete fresh four-case cohort uses reviewed source `5afa7dd` and immutable
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
A full Player callback entered dead has skipped the existing living-motion
query, so the full manager restores its query after contacts. Common Kill
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

The gate remains **FAIL** with 3,014 strict failures: 2,774 Game-age, 80
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

Exit audio remains a separate source gap: fresh MenuInit requests Music61 and
SetMenu0-to3 emits no forward SFX8, while the current Exit path requests Music1
and SFX8. The focused replay's SFX30 checks do not qualify those backend
contracts. The historical four-case results above retain their original reds.

The port's existing living-player grid query still runs before full-phase
entity contacts. A Player killed later during contacts has already received
that query and does not receive a second bounded correction in this leaf.
Moving all living full-phase queries to the source final pass remains a
separate physics change; near-solid contact-death ordering is unaccepted.
The player-death producer's clock correction remains pending production
qualification. No counter or position is copied or fitted from original
observations. NO_KILL, held-arrow, Soul health, revival and full diary art are
not accepted by the historical cohort above.
