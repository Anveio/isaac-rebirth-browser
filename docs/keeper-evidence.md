# Secret Keeper constructor and idle

Authored type17/variant1 selectors now create actual NPC actors in the mixed
room scheduler. The raw STB subtype remains0. The first full NPC update derives
its native subtype directly as `(spawnSeed %9)+1`, selects the authored `GuyN`
clip, stores its anchor and sets idle state3. It consumes no local or shared RNG
draw and bypasses the Fly appearance/AI/blood path.

Pinned Linux v1.05 SHA-256 is
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.
NPC dispatcher4f9670 maps entry66e3c8 through4f98b5 to48af20. The first-AI
block48af50–48af8f reads the public spawn seed directly, computes modulo9 plus1,
sets entity/grid classes1/0 and changes flags to `(flags &~4)|0x11`.
The variant1 branch48b2c0 selects `Guy%u`;48b090–48b09b stores the anchor and
state3. Living AI copies that anchor at48b0a5. Color setters42f480/42f500
establish float32 tint(.2,.2,.2,1) and offsets(.2,.2,.2).

Production collision/health/countability fields come from authored
entities2.xml17/1: HP5, mass100, radius1, damage0, friction1, shutdoorsfalse and
bossfalse. Native class1 keeps the Keeper out of the class4 Tear/Fly-pair/Bomb
body contact paths. This slice covers fresh construction and idle; Keeper
bomb damage/death/drops, other variants and actor mutation persistence remain
separate. No observation enters production state or mutates a raw selector.

The live gate retains opposite-phase JF2DSF92 and the independently recorded
OS-random WS03MPHH from the secret-door cohort. Public bomb/movement inputs
reveal and enter their actual secret rooms, wait60 presentations and return.
The first uses seed3364657625→Guy3; the second uses442555422→Guy7. The latter's
native frame1 freezes during entry, then alternates0/1 on active full updates.
Frame counts and the complete clips come from the authored animation cache.

Every selected phase requires the complete actual Keeper set and exact
seed/subtype, world position/velocity, health/collision/countability,
state/frame-age/countdown/flags/anchor, clip/frame/playing and float32 color.
It also retains exact player/HP/immunity, room/door/display/minimap and complete
manual-seed input provenance. Existing secret/key admission checks join the
actual NPC by seed/subtype/position; unsupported actor metadata fallbacks stay
explicit. Native float32 colors compare by float32 value, preserving exact
bits despite JSON's shortest decimal formatting. The first replay's decimal
comparison failure is retained separately.

After a fresh production build:

```sh
source cloud/env.sh
npm run build
cargo build --manifest-path engine/Cargo.toml --locked --bin floor
bash tools/original/with-display.sh node tools/e2e/keepers.mjs --count 1
```

Complete rendered pixels, contact/damage/drop callbacks and total shared MT
consumer alignment are separate claims.

The final actual-DOM production snapshot
`6f0aa3bb6205255b81c2872f632d44e3e6e548426c2ed740c978021fce7823f3`
passes all658 selected phases,329 per case, with complete384-command public
input traces and385 retained command states. Source and actual actors match
through both entry freezes, all60 idle presentations and return. The
[compact evidence](../references/isaac/foundation/doors/keeper-verification.json)
pins source/asset/input/comparator/build hashes, exact constructor tuples and
animation boundaries. Full artifacts remain in
`.cache/e2e/keepers/paired-composite-final`; the earlier decimal comparison
report remains separately hashed. Historical secret/key raw-selector
constructor failures remain retained; their current admission checks require
this actual initialized NPC instead.
