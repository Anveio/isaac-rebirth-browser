# Native animation and title presentation

The asset renderer now samples ANM2 data in the original f32 units and computes
corners through separate layer and actor transformations. This is derived from
the pinned base Rebirth ELF
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.
It does not consume reference vertices or screenshots during rendering.

`Interpolate<Frame>` at `0x42331b` computes a rounded reciprocal delay, then
`0x4235a6` accumulates it once per animation frame. Vector interpolation uses
separate f32 subtraction, multiplication and addition. `ColorMod::Lerp`
(`0x42f6b0`) blends weighted normalized endpoints. `InterpolateAngle`
(`0x42eeb0`) takes the shortest signed degree path. `GetDestQuad` (`0x40eeb0`),
quad scale (`0x5ee320`) and rotation (`0x5ee040`) retain intermediate rounding
around the layer and actor anchors. Degree conversion (`0x6124a0`) multiplies
by f32 radians per degree. Aligned quads round on the final display-pixel grid;
rotated geometry remains unsnapped.

The retained original-only primitive fixture contains 75 title quads
(600 coordinates) at 25 source-observed phases,
and 42 explosion-smoke quads (336 coordinates and 42 alpha values). All match
exactly. The formula was derived on four smoke quads; 38 other smoke quads and
all title quads are additional checks. Rounded JavaScript trigonometry agrees
with source `sincosf` for these authored angles; arbitrary-angle cross-library
equivalence has not been established. The component harness is now ignored
research; `npm run test:e2e` exercises the live original/browser title lifecycle
and records complete frame differences separately from functional contracts.

Title initialization (`0x5ae590`) loads `TitleMenu.anm2` and plays Idle at frame0.
Title update (`0x5ae6c0`) advances ANM2 only when the current global System
counter is even. MenuManager dispatches the selected menu's update. Title reset
(`0x5ae7f0`) is a no-op, so leaving and returning must retain its animation
position. The authored animation loops at 40 frames.

MenuManager initialization (`0x59a4d0`, instruction `0x59ad17`) sets fade alpha
to 1. The pending menu transition (`0x594cb0`) initializes the menu, selects it,
and invokes StartFadeIn (`0x59b0f0`). The same System iteration then calls menu
update, subtracting f32 `.025` at `0x59b2c3`. Thus the first rendered title has
alpha `.9750000238418579`. This is a white full-screen fade. Menu rendering
(`0x59c740`) uses its stored RGB and replaces color alpha with the current fade
alpha; it clears the fade after alpha becomes nonpositive.

`tools/original/foundation-title-scenario.py` supplies real public inputs for
first title, first update, fading, final fade, settled and loop checkpoints.
`foundation-menu-observer.py` reads only the owned, gated original child's
source-located title/fade fields and never writes game memory. The fresh native
pair in `references/isaac/foundation/title-animation` repeats all ten screenshots
byte-identically. First title is actor0/System230; the next presentation is
actor1/System231; later checkpoints are actor12,20,32 and1 after the first loop.
These are reference phases, not established simulation ticks or complete state.
Leaving after that loop freezes actor1 in the profile menu; returning preserves
actor1, then another 83 presentations advance it to actor3. Seven complete
title frames and three retained-clock checkpoints are checked by the browser
test. The three intervening menu-transition images are retained as evidence;
their camera movement is outside the current full-frame gate.

The legacy title references and their existing failing comparison are preserved.
Fresh full-frame browser gates and all prior exact menu/pause/mask gates must
pass before claiming integrated title parity.
