# NPC Freeze and Mom's Bra

This is a bounded source implementation, **not complete live parity
acceptance**. The fresh Horf12/AttackFly18 bank passes fixed phase1 and the
recorded OS case, while phase0 and no-Space retain required AttackFly velocity
failures. The four-case bank remains **FAIL: 269,019 checks / 276 failures**.
The existing target-free Bra case retains its earlier, separate scope.

## Source contract

Mom's Bra39's `523B90` room-list walk has no Player-variant gate. Every actor
that passes `444810` receives config39's signed cooldown220 through
AddFreeze`444970`. The predicate requires type10..999, collision class other
than0/1, clear494, living31F and clear flags318 bit0x10. Visibility is not an
eligibility test.

## Implemented scope

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
before Kill4464C0 sets dead31F. For ordinary frozen Horf, Die445C60 tests
positive400, selects SFX137, then rejoins the common Effect5/7 gib/blood path;
there are no separate ice-specific effect IDs. The implementation here still
preserves the HP/dead prefix, records `frozenDeath445c60`, and returns a
source-frontier error before death sound/gibs/blood. It does not discard the
NPC or invent a continued dead update loop. Other newly reached actor shapes
also refuse with `unsupportedActor`. Qualification requires no status frontier.

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

## Live gate

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
fade38 and propagation39. Missing or unjoined owners fail. It also reads the
actual Game+A1E8 information-manager state and caller delay. Native current470 is compared only
with available `Npc.status.currentFriction`; the ordinary AI-scaled
`Npc.friction` field never substitutes for it. Its pre-admission differences
remain a separate raw-friction diagnostic. Successful application still grades
the actual transfer and all admitted reset/motion checks strictly.

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

The comparer requires consecutive actual System counters, the actual game
Full-counter change and the actual information-manager gate. A scheduled Full
with no admitted RoomFull requires state2; a RoomFast requires entry state0.
An information-blocked callback must hold the NPC owners. Missing information
state leaves the classification unavailable and fails its mandatory witness;
phase parity alone does not prove RoomFast. Direct per-NPC callback entry
instrumentation is not added. Grid correction, skipped callbacks and any
missing mandatory witness remain failures, never inferred equality.
Pixels remain separate diagnostics. JSON can erase negative-zero sign bits;
those erased signs remain unobservable, not verified equal.

## Current live evidence

Source `3c11e2cf`, based on main `6474`, passed a fresh `check:build` and the
affected preview checks. The immutable candidate production is
`df93f7889aecf1b2376c846d6ebc150e0c4e2bb4c5f0a7fe9048e52b5234cc6c`, with WASM
`092ef321770e71cd3c0f856f83d69d8d7a4b1c2de07822f6a3ab4c54902b7aee`.
These fresh Native information-manager observations belong to this artifact.
The fixture contains Horf12 and AttackFly18; implemented Gaper10/Fly13
admission has no live coverage here.

| Case | Checks | Failures | Paired presentations | Result |
| --- | ---: | ---: | ---: | --- |
| Fixed phase0 | 71,368 | 188 | 599 | FAIL, AttackFly velocity only |
| Fixed phase1 | 71,422 | 0 | 600 | PASS |
| First recorded OS raw `331af1d0`, boot epoch `3505461811`, phase1 | 71,422 | 0 | 600 | PASS |
| Same-target no-Space | 54,807 | 88 | 599 | FAIL, AttackFly velocity only |
| Whole bank | 269,019 | 276 | 2,398 | FAIL |

All four cases pass transport, actual readback and source/protected-pin
checks. Each game independently witnesses its own callback path. Fixed0
actually applies on Full GP201 and reaches its first admitted Fast at GP314;
Native and Browser each retain 56 information-blocked Fast and 53 blocked
Full callbacks, with zero missing information owners. Fixed1 and the recorded
OS case apply on Fast GP202, then reach Full GP203; each game observes 55
blocked Fast and 53 blocked Full callbacks. Blocked callbacks hold the NPC
owners. These are observed application phases, not parity-inferred callbacks.

The fresh main comparison uses artifact `8c` from runtime source `6474`,
production `fd21587b41a28bcb3a26e1353060a9f36efec82db44a206b41495f6f52f9ac01`
and WASM `68d3743a5425c21fe48082da9b7b0294ed4d9f9029c226cd1149fde31b018224`.
On the same fixed0 input with fresh Native Info, main has
58,802 checks / 12,683 failures versus candidate 71,368 / 188. The exact audit
has 58,706 common keys: 46,113 pass-to-pass, 12,405 fail-to-pass and 188
fail-to-fail, with zero common pass losses and unchanged semantic Native
expectations. Ninety-six conditional SpriteHit rows disappear: 90 failures
and six passing frame0 rows. All 12,662 new rows pass. Row-count changes do
not erase the retained baseline report.

The 188 candidate velocity tuples are identical to the older raw459-failure
report. Current main already fails all 188 keys against unchanged Native
expectations; its 42 pre-Bra actual values are also identical. The candidate
retains 144 failures during Freeze and two after expiry. Every paired
physical-position row in this cohort is exact.
The first source primitive difference occurs in ordinary wander at GP149,
before Bra application at GP201: Native glibc `sincosf` and the port's `libm`
cosine differ by one binary32 bit. All velocity mismatches remain required;
no tolerance or trigonometry change was introduced.

## Retained reports

The older immutable phase0 journals retain raw baseline 55,148 checks /
13,448 failures and candidate 62,866 / 459. Strict replay of those same
journals remains baseline 54,387 / 13,285 and candidate 66,953 / 790. Its 602
Native information-owner qualification failures remain failures in that
report; the older journals never captured that owner. The strict comparer
also withdraws two old Native Fast passes on both sides. The old 267
wrong-moment raw-friction comparisons remain diagnostics; current470 has no
legacy fallback. Fresh actual Info independently resolves those 602 missing
qualification checks and adds 4,415 passing Native Info checks. It does not
waive or replace either older failed report.

## Open qualification

Current main/candidate target-free Bra39/Pad41 controls and Browser focus/seed
checks are **pending**. Two default invocations omitted the `--npc-freeze`
extension needed by the retained whole bank and failed during setup with zero
cases; those failures are retained separately. The corrected four-case runs
above use the same recorded bank. Earlier target-free item evidence retains
its original artifact scope.

The complete Freeze gate remains unaccepted. Frozen death, living
reached-status morph/reentry and saved-body restoration retain the frontiers above.
Negative/repeated duration, boss/champion/linked branches, Fear, complete MT,
PCM and full pixels remain unqualified. Fear is a separate unmerged workstream
at `5ce256c8` and adds no feature claim here.
