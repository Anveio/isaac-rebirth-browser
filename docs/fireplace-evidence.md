Ordinary Fireplace33 now admits from authored entity selectors and raw1400.
Room.spawnConfig5c5240 uses an independent local RNG7 on the group word to
choose ordinary/red Fire; ordinary entity spawnSeed is the first local RNG35
word. The runtime preserves the raw room selector and admits an actual NPC.
Red, blue and purple variants remain unsupported selectors.

Fire AI49a530 owns its damage queue. Each positive ordinary Tear event loses
one or two HP; an event over10 loses two or three. Each event consumes a real
shared bounded3 draw, then HP clamps to1. HP below1.5 or aggregate flag4
extinguishes the actor: HP1, state3, both collision classes0 and Room cell900.
It plays NoFire, admits SFX261, then force-plays authored Dissapear. It remains
an active empty fireplace rather than entering generic death/retirement.
NPC-only contact16/flags2 is guarded by source444800(other.type10..999).
Primary Player contact1 remains separate; this special branch excludes
Player1, Tear2 and Bomb4.

First generic Update initializes Fire AI, which clears appearance flag4, then
invokes active Fire AI again during that same callback. This consumes bounded30
for cooldown and the first bounded7 emission chance in source order. Later
full callbacks copy anchor−position into velocity, process damage, decrement
cooldown and emit; common BaseUpdate applies friction and motion. Intermediate
callbacks predict half of friction-scaled velocity without AI, RNG, HP drain
or sprite advancement. Transition/pause admission uses the shared scheduler.

Each actual Ember1000/66 birth takes four source producer uniforms and a raw
shared spawn word. It joins the manager at birth and waits for the old-tail
fence before its first full Update. Its Init has no extra shared draw. Full
Update moves using entering velocity, then the first callback draws bounded30
for TTL/divisor. Every callback draws two uniforms for damped drift, chooses
the manual authored Idle frame, and handles TTL. A dead active ember receives
one final full callback, including both uniforms, before removal. Fast
prediction freezes the lifetime/AI. This actor uses the same shared MT owner;
no captured word or cursor offset is supplied to the browser.

Ordinary extinction executes actor RNG30's actual loot chance and pickup seed
producer. Fresh unmodified Isaac coin/heart subtype0 uses separate local35
Init and joins the actual pickup scheduler. No W0 loot acceptance is claimed:
the three retained Fire seeds take the no-drop branch. Heart collection and
item/trinket modifier routes remain the inventory owner's separate scope.

The Fire renderer loads the actual ordinary ANM2 with its HP-dependent flame
layer scale. NoFire contains the fireplace base; Dissapear retains the base
while the flame/poof vanish through authored tracks. Entity+460 is an integer
render-sort addend; −1000 is retained and never applied to world Y or actor
visibility. General actor depth ordering and Fire additive blend routing are
not accepted by this leaf. The missing authored grid/effect_005_fire.png page
was decoded directly from the pinned graphics archive and added to the asset
inventory with its real hash, without a filename alias.

Read-only source contracts and hashes are retained in ignored coordination
notes under fire-research, fire-render-research and fire-loot-research. The
live runner is `npm run test:e2e -- fireplaces --production-directory PATH`.
It runs the mandatory recorded OS-random W0L0LYYM controls, followed by public
idle observations, plus contrived natural-door Tear and Bomb routes. Both
games must admit nonempty actual33 sets and execute their declared queue,
drain, extinguish, empty persistence and Ember lifetime markers. Every
declared gameplay capture requires a passing source read and production
playing state. Ordered queue amounts/flags/actual retained attackers, direct
HP/classes/clip/cooldown/local RNG fields, source cell950/900 and null grid
objects remain required; missing observations cannot skip them.

Three fresh original/production pairs now execute ordinary Fire admission,
Tear queue/drain/extinguish, flag4 Bomb queue/extinguish and persistent empty
actors in both games. The immutable artifact is
`3dbd3de19a114636752a8202cf54dca3a77f0acc1eed42fe5df63a973de45c23`.
General paired checks pass 825/852. Direct checks retain 8,525 failures among
191,217 fields; all 110,642 checks of each game's own actor laws pass. These
include dense full/fast motion, frozen lifetime, birth fences and next-full
Ember retirement. Paired cooldown, HP, physical, queue, sprite/extinction
timing and Ember birth/lifetime differences remain blocking. This is not
whole paired gameplay or shared-RNG acceptance. Compact actual queue/empty
witnesses and original/browser/body hashes are in `live-report.json`.

The first comparator attempt failed while recording provenance because it
read a nonexistent production field. Its rejected report is preserved; a
small fail-closed metadata correction replayed the completed immutable pair.
The later two fresh cases use that corrected comparator. No source input,
runtime byte or native state changed to improve the result.

The two-worker production WASM/Vite build, typecheck, strict Clippy,
2557-file asset inventory, matrix and formatter checks pass. No cached
original is accepted as a fresh Fire pair.
Exact shared MT/particle seeds, live loot, saved72 Fire reconstruction on
room return, variant1/2/3 AI and rendering are unresolved. W0 treasure,
T1 hostile return/HP, H2 and center-route failures remain mandatory in their
existing gates; this leaf never replaces or weakens those source references.
