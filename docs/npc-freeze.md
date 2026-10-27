# NPC Freeze and Mom's Bra

This is a bounded source implementation, **not live parity acceptance**. No
build, Original process or Browser capture has qualified this change yet.
The existing target-free Bra case retains its earlier, separate scope.

Mom's Bra39's `523B90` room-list walk has no Player-variant gate. Every actor
that passes `444810` receives config39's signed cooldown220 through
AddFreeze`444970`. The predicate requires type10..999, collision class other
than0/1, clear494, living31F and clear flags318 bit0x10. Visibility is not an
eligibility test.

The first positive admission covers fresh, unchampioned, unlinked living
Gaper10.0/10.1, Horf12.0, Fly13.0 and AttackFly18.0 after Appear. The entire
room list is preflighted before changing any target: an eligible unknown,
custom-base, boss, champion, linked or appearing target retains the existing
`UseLeaf` frontier. A known unlinked immune target (flags318 bit1 or signed
420>0) is a no-op and still permits the ordinary Use tail/discharge. The
earlier GiantBook/TEMP prefix keeps its existing ownership.

`Npc.status` owns signed freeze400, shared application lockout420 and freeze
flag0x20. Fresh construction clears them. Positive AddFreeze adds to400 then
clamps to120; a negative duration extends to at least its magnitude, without
the120 clamp. Negative-duration and repeated-application public routes remain
unqualified. Freeze inserts a separate timed color at priority255, RGB0.22,
alpha1 and RGB offsets40/255, through the existing color stack. Reapplication
retains older entries and uses the same insertion law; rendering already
consumes the actual current sprite color.

At full entry, the existing freeze flag suppresses ordinary AI and sprite
advancement. Motion still multiplies velocity by current friction times0.01,
then integrates position. BaseUpdate restores current470 from base46C.
Color countdown precedes status countdown, which precedes FIFO damage and
hit-color insertion. A timer1 callback still suppresses AI/sprite and reduces
motion; it expires its color, clears freeze and resumes at the next full
callback. Fast advances no AI, sprite, color or timer. It predicts
`P + ((V * currentFriction) / 2)` with the separate binary32 operations.

`baseUpdateAdmitted` records an actual successful Bra application and remains
true after expiry, so the actor keeps this source BaseUpdate and Fast path
when its timer becomes0. It does not mean the actor remains frozen. Actors
never successfully frozen retain their existing update path. That ordinary
path currently stores pre-reset friction and predicts `P + V/2`; correcting
all ordinary actors is separate work requiring its own regression evidence.
`status.currentFriction` separately records the source470 reset after each
mapped common Full, without changing ordinary motion. It starts unavailable;
Bra admission requires this proof and a living actor past Appear. Successful
application transfers the proven source owner into the admitted path before
the first Fast/full callback. The old AI-scaled field is never promoted as
Native current470. Morph and room restoration invalidate the prior proof.

Frozen death remains unresolved. Static source confirms damage stores HP
before Kill4464C0 sets dead31F, then Die445C60 tests positive400 before its
distinct ice effects/SFX137. The implementation preserves that HP/dead prefix,
records `frozenDeath445c60`, and returns a source-frontier error before any
ordinary death sound/gibs/blood. It does not discard the NPC or invent a
continued dead update loop. Other newly reached actor shapes also refuse with
`unsupportedActor`. Qualification requires no status frontier.

Boss AddFreeze writes shared420=240; root/child propagation follows the
source's selected linked types. Neither branch is admitted here. Clear-status
4452E0 is not assumed at generic death or room transition. Morph, room
exit/reentry, Save/Continue status carry/reset, frozen damage/death effects,
boss/champion/linked behavior, other statuses and complete MT/RGBA remain open.
The admitted Continue domain still refuses saved NPC bodies. A reached-freeze
actor's morph explicitly refuses, including after expiry. Ordinary room exit
keeps its existing path; an attempted return that would restore a living
reached-freeze actor refuses before either room or saved body changes. Dead
or removed actors do not trigger that readmission guard. This does not infer
whether Native saves, clears or carries status. Exit/revisit qualification
remains UNRUN.

Mom's Pad is separate: Fear4450D0 uses418/flag0x800 and resets two pathfinder
vectors. Its variant0 leaf reads **config39's220**, although item41's own
cooldown is180. This change adds no Fear behavior.

The existing `active-items` gate has an opt-in extension:

```sh
npm run test:e2e -- active-items --npc-freeze --no-build --production FROZEN
```

It retains the old default bank/rows and adds fixed phases0/1, the first
unfiltered recorded OS boot/phase draw for the same Horf12/AttackFly18 fixture,
and a no-Space control. Replay uses the recorded bank without entropy. Only
declared `activeItems.npcFreeze` cases select the additive read-only observer;
the driver requires `--active-items`. No Native output enters Browser inputs.

The new wrapper composes the existing active/pocket and combat readers. It
independently reads real preview/physical positions, base/current friction,
400/420/318, parent/child handles and the timed-color vector. Static SetColor
446F00 confirms entry stride0x3C: priority0, KColor4, duration30, remaining34,
fade38 and propagation39. Missing or unjoined owners fail.

Each game must independently witness held39/charge3, physical use/discharge,
live stable target identities/HP, a moving target and Horf shooter progression
before use, positive application, full-only countdown, AI/ANM suppression,
reduced full integration, exact Fast prediction, timer1-to0, resumed full AI
and post-expiry Fast. The initial application separately grades either the
actual Fast120 physical/velocity hold and preview, or Full119 reduced
integration/AI/ANM hold, then requires the first Full and Fast callbacks.
Only each game's observed application phase is recorded. The no-Space control
must keep statuses clear and make no temporary39/use/discharge. The long
post-use tape retains expiry margin.
The comparer uses the actual game full-counter change; direct per-NPC callback
entry instrumentation is not added. Grid correction, skipped callbacks and
any missing mandatory witness remain failures, never inferred equality.
Pixels remain separate diagnostics.
