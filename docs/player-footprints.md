# Player footprint cadence

Player50BEF0 calls IsFrame444890 with `(6, 0)` for footprint cadence and
`(12, 0)` for the foot side. That helper reads Game+CE9F4 directly. Ordinary
Game Full increments this counter before dispatching Player; ChangeRoom's
extra Room.Update uses the unchanged counter. The footprint helper now uses
that actual counter in both paths, removing its extra increment.

The selected `PHCVWE1C` upper-door route comes from the previously closed
8,192-draw OS bank, ordinal 3,497. No draw was replaced or repeated. The
retained public input case ends at `entry-1`, two presentations after the first
Super Envy admission. It contains inputs and seed transport, without Original
resources or observations. The full fight declaration and its failed author,
lifecycle and room-clear coverage remain separate and unchanged.

A fresh production build on main `377d74d2` plus this counter correction
(`40295031`, 1,407 files) was compared with unchanged main's `16d85002` build.
Both actual Browser captures used the same literal 1,110-presentation prefix
and the same complete independently validated Original evidence. All 83 sparse
captures and 76 matched Original step ends were graded with unchanged field
keys and expectations. The 5,301 fields improve from 5,289 passing to all
passing: 12 position/velocity fixes for the first root, with no lost passes.
Player, room, NPC identity, HP, state and animation checks remain passing.

Strict full-frame diagnostics remain 7 exact and 76 failing on each artifact.
The Browser's observed random state does not establish an Original MT cursor;
Original MT state was not captured. This result qualifies the short admission
prefix, not full Super Envy combat, child lifecycle, complete shared-MT parity,
or the broader transition-velocity reconstruction in #459.

The registered `footprint-admission` suite reuses the paired gate and exact
entity comparator. Fresh runs mix this fixed upper-door route with one
Source-selected OS-random adjacent miniboss room. Both tapes stop at the
existing `entry-40` settle before shooting. Every initial NPC is graded by
identity, seed and occurrence, including exact f32 position and velocity;
both games must actually reach room type 6 with a living NPC. Generic paired
state checks alone do not cover these NPC fields. Explicit public clock and
lossless Native archival options leave other suites' defaults unchanged.

The first registered fixed replay passes all 3,271 fields on frozen `40295031`;
the same Original replay against parent `16d85002` retains six root position/
velocity failures while its 83 generic paired checkpoints pass. This separate
negative control demonstrates the reusable comparator catches the counter bug;
it does not replace the denser 5,301-field proof above.

The initial mixed bank stopped at `entry-1` and correctly failed four admission
guards on `HH936E8Z`: both games were still in the starting room. The same two
cases were extended through the existing 39 remaining settle presentations,
without changing any earlier action, selecting another seed or shooting. On
`40295031`, fixed `PHCVWE1C` passes 5,884 fields; `HH936E8Z` passes 5,869 of
5,947 and retains 78 required position/velocity failures for its initial Envy
root at `entry-2` through `entry-40`. All four admission guards, Native
provenance, public clocks and generic paired checkpoints pass. The new bank
retains every earlier field and expectation. Strict RGBA diagnostics are
7 exact/115 failing for fixed and 7 exact/113 failing for the recorded OS case.
The earliest observed direction difference remains a motion/shared-consumer
investigation; it does not establish an Original MT cursor or a causal fix.

Replay both retained public input-only cases without drawing another seed:

```sh
npm run test:e2e -- footprint-admission --scenarios references/isaac/cases/footprint-admission/recorded-fixed-and-hh936e8z-entry40-v1.json
```

The default command draws a new Source-eligible OS case; recorded replay keeps
the saved draw. Extra case-level lifecycle markers are outside this short
admission scope. A fresh replay on PR629 build `f0eb9e70` (1,407 files) uses
the identical bank and preserves every historical field, expectation, value
and status. Its shared entity comparator adds 115 fixed and 113 OS required
Browser draw-order-presence rows, all passing: fixed 5,999/5,999 and OS
5,982/6,060 with the same 78 motion failures. Fresh Native provenance and
public clocks pass; RGBA remains 7/115 failing and 7/113 failing respectively.
No overlapping NPC/player order row is exercised by these prefixes. The
historical `40295031` result and original four-guard failure remain retained;
this latest replay does not claim broader fight, clear or shared-MT parity.
