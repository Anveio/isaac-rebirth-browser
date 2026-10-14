# Ordinary Dip217 room actor

Base Rebirth entity217/0 is **Dip**, with authored `216.000_Dip.anm2`,
Idle10 and Move25 looping clips, and no Appear clip or animation events.
Its ordinary preallocated NPC class is `66ef30`; dispatch table entry
`66ea08` selects `4f9725 → 4949c0`. The exact original binary and function
hashes are recorded in [the source contract](dip-source-contract.json).

Room selectors admit the real actor through the existing scheduler and manager
count owners. Constructor HP uses actual Stage and `446a90`'s capped stage
formula: base3 plus stage1 on the first floor gives4. The source state0/1/3/4
callbacks own staged initialization, small Poof15/1, visibility, Idle bursts,
Move damping and full/fast ground collision. No captured position, RNG state,
counter or HP is supplied to the port.

The family tail creates a scaled, white Blood7, invokes its initial callback,
then transfers Dip color. This ordering preserves the white alpha draw before
the transfer. The same existing Blood7 constructor and initial callback serve
the other NPC callers; the borrowed factory now exposes both Poof and Blood
admission. Pending damage may admit SFX60 before common BaseUpdate. Ordered
damage retains the first negative HP crossing, skips later records after the
actual death flag, and keeps death distinct from next-full manager retirement.

e2e suite `dips` is a required fresh original/production browser gate. Its default
cohort contains two contrived phase offsets for public Cellar seed69/Room213 and
two OS seed tapes drawn once without filtering. It requires actual Dip presence,
both phases, Idle/Move, negative death and retirement in both contrived games.
All native actor identities, queued damage, AI/clip/geometry/trajectory and
actual Poop58/43 births remain required comparisons. Selected failures, aborted
captures, inputs, complete observations and decoded-RGBA differences are retained.

The follow-up implements the actual authored poop-gib58, final Blood7/SFX30
death consumer and next-full Poop43 admission. Category sampling finishes before
births; zero particles is valid. Each58 completes its explicit initial callback
before the next birth. Final7 copies Dip color before its callback, while43 has
no immediate callback. Typed58/43 full/fast/grid/flags8 paint and real authored
sprites are shared effect consumers, with source pins in
[the effect contract](poop-effects-source-contract.json).

Neither actor nor effect leaf has production or live-game acceptance yet.
The unchanged four public tapes now require actual58/43 body fields and43
execution in both contrived games. Positive Entity400 status and owned225/234
kill callbacks fail explicitly until their actual owners exist. Native Blood background
paint/return, inherited status/color/fear/charm modifiers, saved-room rebuilding,
later-stage HP witnesses and complete causal shared-stream alignment also remain
unaccepted. Static validation cannot establish those game-level outcomes.
