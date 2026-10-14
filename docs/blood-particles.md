# Ordinary NPC blood particles

Blood Particle is entity1000/variant5. Rock Particle4, Blood Explosion2,
Blood Gush42 and the large-boss Effect77 have separate factories and callbacks.
The authored5 catalogue contains BloodGib01..03, Bone01/02, Guts01/02, Liver,
Eye and Particle. [Catalogue pins](../references/isaac/foundation/combat/blood-particles/catalogue.json)
retain the actual base resource hashes.

The first producer is ordinary Stage1 Round Worm244/0. Base447190 subtracts
queued damage and calls virtual kill47fc80 at a lethal record. In the supported
fresh context,4464c0 marks the NPC dead and immediately calls445c60. The actual
gibs child supplies amount6 and flags13. The callback admits SFX30/volume1 before
the bounded2 budget draw, samples blood/gut/eye counts before any birth, then
executes each speed, raw seed and two independent direction consumers. Every
actual5 birth copies inherited color and executes its separate first Update
before the next particle. The final Blood7 has zero velocity and a null spawner
argument and executes its own immediate Update. No fixed draw advance replaces
these calls.

GameSpawn caches its caller's type/variant at Entity+1c/+20; it does not link
Entity.parent+498. The common effect metadata stores those words separately.
An ordinary5 birth and its self-child have a null linked parent. Its cached
spawner words select the controller's ordinary versus child path.

The first5 Update advances Base before initializing height/gravity/vertical
velocity and the selected authored clip. Later ordinary airborne callbacks
sample actual Blood7 drops and re-evaluate the bounded child count after each
birth. Landing at45e579 seeks the last authored pose while preserving the current
playing flag, then moves the actor into the background-paint path.
Full updates own physical motion, animation and controller draws; fast updates
predict the presented position and wall query. Entity render offset remains
distinct from the sprite offset and the stored controller height.

`npm run test:e2e:blood-particles` runs both actual games freshly, one case at a
time. It retains both0/1 phase offsets of the mandatory T1 route and the existing
recorded OS-selected244 route, adds declared cardinal firing and tail intervals,
and requires an actual244 death and5 particles in both games. No observed target
position drives input. Real particle seed/order, source room sequence, body,
initialized controller, color, clip and full/fast fields are blocking. Source
storage before the first actual5 Update is retained as raw bits without claiming
that pooled bytes are initialized. Each closed native/browser manifest is
losslessly gzipped with raw/archive SHA and byte counts; its sidecar gives the
original restoration path. Default CI includes the same suite.

The frozen1e9d02f source passes the full build, strict all-target Clippy and
separate floor build. Its production artifact is
`4e0a87622c0910c0df920fa017a591d6d74e89c15579d1ee4e51f896df1c7e68`
(941 files). All four fresh pairs completed with stable source/artifact hashes
and fail: core648/2386, scoped7090 checks/58 failures. The scoped checks cover
room, phase, ordered presence and execution. No native case killed a Worm or
produced5; one browser case did. Zero native particle bodies matched, so these
counts do not establish particle-body/controller parity. Both recorded OS
offsets remain in the failed cohort. [Compact evidence](../references/isaac/foundation/combat/blood-particles/first-live-1e9d02f.json)
retains every case and report hash.

The complete native/browser manifests total1,730,631,504 raw bytes and are
retained as88,413,976 gzip bytes. Independent streamed readback checks all eight
archives against their original raw hashes. A subsequent movement/shooting bank
must be separately declared and preserve this complete failed execution.

Status/champion/inherited-color modifiers, perks225/234, the exact full
per-record transient-color damage path, other gibs categories and later stages
remain unaccepted. The simplified background model also does not establish the
complete native render-array insertion/resort and persistence contract. The
Monstro42→2→77 chain and its final boss rewards remain required separate work.
