# Ordinary Basement doorways and retained room admission

Ordinary style0 doors now use the generated floor's reciprocal descriptors to
enter 13x7 authored normal rooms and return. First admission selects complete
STB groups through descriptor.spawnSeed; returning restores the exited Room
clone, including selected group/actor seeds, supported grid objects and mutable
pickups. The visited minimap is derived from reached descriptor identities.
Public keys and run/stage seeds are the only browser inputs. Source geometry,
actor snapshots, template identities and cursor offsets never enter WASM.

The live comparison covers ordinary unpriced doors between 13x7 normal rooms
without hostile authored actor choices. A mandatory room181 regression checks forward admission, actual Rock objects
and its attempted return. Its two authored NPCs require combat completion before
return, so the live gate remains red until those lock semantics are implemented.
All requested browser phases still execute and failing comparisons remain in cache. Locked/special doors, NPC behavior and
combat locks, other room shapes/camera/backdrops, floor exits, complete transition
pixels and arbitrary actor save/restore semantics remain separate. Unsupported
STB selectors stay typed in Room.spawns. This does not assert that raw NPCs,
chests or every grid selector have a complete runtime implementation.

## Recovered source boundaries

Addresses refer to Linux v1.05 plaintext SHA-256
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.

- Room Init `5cf190` seeds local RNG11 with descriptor.spawnSeed, consumes the
  tinted-rock width/height draws, then one weighted f32 draw per authored group.
  Entity admission `5ce6b0` derives its local RNG35 seed from that group state;
  grid admission preserves the group seed. Empty groups still consume a draw.
- Door check `5c71c0` runs after full-phase entity updates. The open-door distance
  is strictly below 25 from its grid position displaced outward by 18 units.
  Player collision admits opened-door class5 cells while bombs still collide.
- `54f910` chooses the transition branch before Room Update. A newly requested
  normal style0 transition loads the next room on the following full update
  through `5ddf20`/`5de26f`; it has no seven-update outward wait. State3's inbound
  pre-counter uses limit7 at `5dde10`. Room loading clears player velocity and
  places the player one 40-unit cell inside the reciprocal authored doorway.
- The first source room load admits one full nonplayer update before inbound
  freezing: fresh pickup Appear frame1 persists through the transition. General
  pickup contact and rendering are the inventory component's separate gate.
- Ordinary Rock variants derive from RNG35((decorationSeed + gridIndex) | 1).
  `56c390` uses the local spawn seed to select linked horizontal/vertical/quad
  clusters. Room181 reproduces all nine native kind/variant/state/class/seed
  records. Boundary/door grid entries currently describe collision geometry;
  their actor RNG, callbacks and all unsupported grid kinds remain unclaimed.

## Executable acceptance

After a fresh build, run:

```sh
source cloud/env.sh
npm run build
npm run test:e2e:traversal
```

The gate records an ordinary no-hostile return, a mandatory hostile rock-room
return-lock regression, and an OS-random ordinary no-hostile configuration selected from
production floor/STB data. It executes
all selected originals in isolated fresh profiles, validates the complete actual
X11/clock acknowledgments, retains each source framebuffer and read-only phase,
and replays every requested browser key/interval against immutable production
WASM. It requires exact room identities/descriptor seeds/public SpawnRNG joins,
f32 player positions and velocities, and exact nonempty authored group accounting
from a separate STB decode joined to the native template identity. Raw selector
type/variant/subtype, positions and grid indices must belong to that authored
group. On first admission, resolved regular pickups and stationary NPC244
selector metadata join the native type/variant/subtype/seed/position multiset in
both directions; this metadata check does not claim NPC simulation. Chest
subtype callbacks, ambient effects and unsupported actors remain excluded.
Ordinary Rock index sets must agree in both directions before checking every
supported kind/variant/state/class/seed record. The gate also checks visited
identities, retained exited rooms and first-admission restoration.
Failures remain failures; no selected source is skipped.

The initial frozen traversal matched independent CSTGNPJ8 right and P793Q770 up
runs at all348 captured source phases, including exact admission and return.
These reports remain in cache. Native shader/FX/complete pixels and complete
internal state are outside this functional gate; prior strict pixel diagnostics
keep their existing budgets and results.
