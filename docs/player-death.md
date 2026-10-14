# Normal Isaac death

The normal Player damage callback now carries the admitted caller's damage
flags into Kill and Death. Both public death routes now execute in fresh
original/browser pairs; the complete four-case gate remains red. Revival items, other characters, cooperative players,
black hearts, and complete diary content remain unimplemented.

| Contract | Original evidence | Port state | Live acceptance |
| --- | --- | --- | --- |
| Health-depleted damage and NO_KILL bit1 | Player52b080/52c675/52cbc9 | Explicit admitted damage branch; no global zero-HP death trigger | Pending |
| Normal zero-gib Kill | Entity4464c0/445c60/44622e | SFX30 volume f32.8, delay2; own Shared raw seed, real Blood7 and immediate Update | Pending |
| Death animation and motion | Player531850/531b24/531bb6; RoomFast5cd920 | Death forced once, velocity cleared on both Player callbacks; ANM advances only on full Base callbacks | Selected Death frame/playing/guard agree in both required routes; Base motion remains incomplete |
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

Base447190 applies carried velocity before the dead Player callback clears it;
the current dead early return omits that integration. System Exit-menu fade and
retained title initialization also remain incomplete. These source boundaries
are tracked for small follow-ups; no counter or position is copied or fitted
from original observations. NO_KILL, held-arrow, Soul health, revival and full
diary art are not accepted by this cohort.
