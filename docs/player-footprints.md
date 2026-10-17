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
