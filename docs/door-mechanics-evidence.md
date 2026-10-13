# Authored door mechanics

Fresh ordinary Isaac can enter and leave authored 13x7 curse rooms through
their generated reciprocal doors. The doorway charges one half-heart on each
eligible contact, before player motion, and retains the curse-door asset on
both sides. Room admission, visited identities and returned selected contents
use the ordinary descriptor route. Red chest selectors remain typed without
claiming chest callbacks or simulation.

The shared `combat::try_damage_player` admits damage only for a living fresh
player with no active immunity. Callers supply their own health units and
source-observed immunity, contact/impulse policy and flags. The helper does not
infer a universal damage-to-cooldown rule.
The port currently advances only hurt immunity at the recovered transition
Player Update dispatch points. Other player callbacks still need a unified
transition path; this gate does not claim those clocks or effects.

## Source boundaries

Linux v1.05 plaintext SHA-256
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`:

- Door Update `565000` runs in the full grid pass before player actors. It
  excludes room types7/8; room type10 on either side enables curse contact.
  `565127` compares squared world distance strictly below225 from the actual
  grid door position. The fresh grounded Isaac path calls Player TakeDamage
  with amount1, flag`0x10000`, empty entity reference and argument30 at
  `565654..5656af`. Live readback establishes immunity60, observed as59 after
  that phase's player update. Flight/items and special curse effects remain
  separate from this fresh-Isaac contract.
- Door Init begins with collision class4. Opened state2 receives class5 in the
  first active full grid update (`565581`), after inbound freezing. Door
  callbacks use the same admitted grid that player/bomb contact consumes.
- Transition state1 dispatches Player Update through virtual`+18` at`5ddf87`,
  then the loaded Room Update at`5de048` dispatches it again. Hurt immunity
  advances twice at admission. Inbound phases freeze it; state3 completion
  dispatches one player update at`5dde9e` while grid/nonplayer updates remain
  frozen. These are source callback boundaries, not presentation offsets.

## Required live gate

```sh
source cloud/env.sh
npm run build
npm run test:e2e:doors
```

The gate selects a contrived opposite-phase entry/exit and records a new
OS-random no-hostile curse-room configuration before any original execution.
It uses only public X11/DOM controls, isolated fresh original profiles and an
immutable production-WASM snapshot. Every source capture must pass binary,
log, input/clock and read-only observation provenance. Every browser interval
executes even after a failed comparison; failures are required, with no skips.

It compares every native player position/velocity, room/template/descriptor
seed identity, complete independently authored nonempty group accounting,
supported admission metadata, visited/returned-room state, visible door object
set and state/collision class. Red HP and immunity must agree at every phase;
the original must charge HP6→5 on entry, reach zero immunity during the public
wait, and charge HP5→4 on exit. Source and browser values are retained separately.
Undiscovered native variant6 entrances are retained in the native observation
but excluded from this visible-door contract; the report lists that exclusion.

Initial OS-random63B9JKF8 down and DGJHZLJJ up routes matched all296 source
phases. The opposite-phase contrived63B9JKF8 and recorded OS-randomDV1PYCJA
matched all240 phases in a second fresh paired run. The build on integrated
checkpoint5b4901f also matches all240 phases. Reports live under ignored
`.cache/e2e/doors/curse-integrated-final/report.json`.

Secret discovery, key locks, hostile NPC room-clear locks, other door styles,
large-room camera/backdrops, floor exits, mutable actor/grid callbacks and
complete transition pixels remain tracked by their owning gates. The
mandatory T1PK6Y0W hostile return regression in `test:e2e:traversal` stays required.
