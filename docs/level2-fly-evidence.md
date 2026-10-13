# lvl2 Fly and the public Monstro prefix

This leaf admits authored NPC214/0/0 with HP10, radius13, mass6, contact1,
friction1 and eight wall-collision samples from `entities2.xml`. It uses the
actual `213.000_Level2Fly` asset, whose only animation is the two-frame looping
Idle. Entity215 owns `214.000_Level2Spider`; that asset is unrelated.

Pinned AI4ef880 initializes state3 and continues active AI in that callback.
The generic appearance path saves this real initialized state, produces a
parented Poof2 at incoming counter1, and restores state3 after the missing
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

Compilation, the frozen production artifact and fresh original/browser proof
are pending. Syntax, TypeScript, authored assets, formatting and matrix checks
are preparation evidence only. Complete boss blood/gibs, final heart/reward,
Trapdoor and stage progression require subsequent causal callbacks. Existing
T1 damage, W0, H2/center clearance and shared-stream failures remain required.
