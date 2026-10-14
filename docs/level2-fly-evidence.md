# lvl2 Fly and the public Monstro prefix

This leaf admits authored NPC214/0/0 with HP10, radius13, mass6, contact1,
friction1 and eight wall-collision samples from `entities2.xml`. It uses the
actual `213.000_Level2Fly` asset, whose only animation is the two-frame looping
Idle. Entity215 owns `214.000_Level2Spider`; that asset is unrelated.

Pinned AI4ef880 initializes state3 and continues active AI in that callback.
The generic appearance path saves this real initialized state, produces a
Poof2 with cached NPC spawner metadata at incoming counter1, and restores state3 after the missing
Appear fallback. Target/wander guards use the single natural Game counter;
there is no native42 offset. Steering adds the counterclockwise tangent before
halving Y, retains source component division, and reads the previous full grid
contact. Class3 wall sampling runs after entity contacts, blocks classes4/5,
uses six penetration iterations and preserves feedback during fast prediction.

Generic ordered Tear/Bomb damage now admits214. A zero-gib13/18/214 death calls
Sound30 at volume0.8 before its real shared blood seed. Blood7 Init consumes no
shared draw; the caller's immediate Update selects the authored size/frame,
independent flips and conditional alpha. Flags8 skip the ordinary Update array
and enter the independent render-sorted array. That pass paints/removes the
newborn stain during the same full update, in unsigned render-key order, before
entity contacts. A retained stopped pose renders into the current room floor.
Native paint/return persistence, colors/modifiers and complete boss death FX
remain unaccepted boundaries. Monstro appearance now calls the same actual
Poof factory with subtype3; it receives no extra immediate Update.

The required `test:e2e:monstro` suite prepares contrived admission and a public
firing/clear/boss route for BLLV920M. This is the first simple candidate from the
unchanged OS-random banks written before grading: Start6 → up7/Room18 with a
passive Fly13 → up9/Room185 with two214 → right0/Monstro1012. The runner requires
actual actor sets, Poof birth/retirement, ordered pending records with retained
attacker joins, HP drain/death/retirement, delayed clear and its final boundary,
then real boss admission/Scene/cleanup/AI/projectiles. Every differing body,
seed, health, state or execution remains a required failure. Inputs never carry
expected actor state into either game.

The first browser smoke caught missing production PNG whitelist entries and
the initial public tape's omitted Fly13 kill. Passive means contact damage0;
its default config still shuts doors. The revised tape shoots through that
first room before continuing the same recorded seed/graph. The rejected smoke,
original public tape and hash remain preserved; neither received native or
paired acceptance.

One-job Rust/all-target Clippy, TypeScript, authored assets 2557, formatting and
matrix checks pass. The frozen release artifact has 940 files and SHA256
`30183f3349db92c079794be8918ed0cae74bfdfd5c3fdc7c12bd2aa5eadd8ba0`; its
WASM is `fc4cc8b6be50466734d0b8045f6822289c9c869af7010188e2d029f23d17773a`.

Both fresh native/browser captures and native provenance pass. Contrived
admission has 184/184 generic checkpoints, but 576/11692 required actor fields
fail; the first differences are actual 214 position/velocity on arrival, and
Poof seeds also differ. Both sides execute admission, appearance, Poof birth
fence/retirement and active AI. The retained OS-random full route has 149/714
generic checkpoints and 5145/22067 required actor failures. Native queue/damage
drain executes, but native remains in uncleared descriptor 9 while the browser
clears it and reaches Monstro. Missing native death/retirement, delayed-clear
and boss executions stay blocking. No causal origin is inferred for those
physical differences.

The initial wrapper ended after capturing both games without its second/final
report. Exact functional and direct comparators were replayed offline against
the unchanged captures; their failures, body hashes and source identity are
pinned in `references/isaac/foundation/combat/level2-flies/verification.json`.
The missing wrapper exit and unpersisted startup fingerprint remain explicit
orchestration limits. Original PNGs and difference PNGs remain retained; no
recovered full pixel summary or complete wrapper success is claimed. Complete boss blood/gibs, final heart/reward,
Trapdoor and stage progression require subsequent causal callbacks. Existing
T1 damage, W0, H2/center clearance and shared-stream failures remain required.

A subsequent source review found that the frozen Blood factory omitted actual
Room sequence allocation while excluding flags8 from the ordinary scheduler.
Room5c6940 allocates Entity+8 before Manager4524f0 routes that object. The
separate correction reserves the identity without ordinary full/fast membership
and retires it using actual liveness. One-job Rust/strict Clippy and formatting
pass; no corrected-artifact live acceptance is inferred. Original PR84 heads,
artifact and all failed captures remain immutable. Exact source/log hashes are
in `blood-sequence-correction.json` beside the original verification.

The appearance Poof caller also now runs at its actual pre-Base boundary.
NPC.Update4fbe85 draws the seed and4fbeb7 admits the effect from the current
NPC+448 position before447190 moves the NPC or drains queued damage. The old
post-advance adaptor used the later position and could allocate the effect
after a damage-driven blood birth. A typed factory context now borrows the
effect ID, effect array and room sequence owner inside the counter1 callback.
It retains the cached spawner type/variant, zero initial velocity, fresh zero offset/height+1
and newborn fence, with no immediate Poof Update. The caller consumes one
shared seed; Init consumes none. This narrow correction has source/static
validation only. Artifact30183f, both failed captures and every required
comparison remain retained; corrected live parity is pending.

The source-corrected Poof keeps its linked parent pointer null. Game.Spawn
`550acd..550ada` copies the caller's type and variant into cached fields
`+1c/+20`; this appearance caller never invokes `447fc0` to link `+498`.
The prior adapter incorrectly used the cached caller as a linked parent.
All 172 retained native214 Poof observations have that pointer null. The gate
now requires the cached fields and linked pointer separately. The separate
`poof-spawner-correction.json` pins this evidence without rewriting the old
source contract, captures or failed reports. The correction passes its own-target
all-target Rust check and strict Clippy; current production/build/live acceptance
is still pending.
