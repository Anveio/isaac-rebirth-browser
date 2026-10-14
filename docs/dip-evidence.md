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

`test:e2e:dips` is a required fresh original/production browser gate. Its default
cohort contains two contrived phase offsets for public Cellar seed69/Room213 and
two OS seed tapes drawn once without filtering. It requires actual Dip presence,
both phases, Idle/Move, negative death and retirement in both contrived games.
All native actor identities, queued damage, AI/clip/geometry/trajectory and
actual Poop58/43 births remain required comparisons. Selected failures, aborted
captures, inputs, complete observations and decoded-RGBA differences are retained.

This first actor leaf has no production or live-game acceptance yet. The actual
five-poop-gib58, final Blood7/SFX30 death consumer and next-full Poop43 admission
remain unfinished required callbacks. They are not replaced by Fly's zero-gib
splat or artificial shared draws. Their source-prepared Effect58/43 prerequisite
is tracked separately and must be completed promptly. Native Blood background
paint/return, inherited status/color/fear/charm modifiers, saved-room rebuilding,
later-stage HP witnesses and complete causal shared-stream alignment also remain
unaccepted. Static validation cannot establish those game-level outcomes.
