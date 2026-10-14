The shared Player main-animation owner covers authored `Trapdoor`, `Appear`,
`Death`, and `UseItem` clips from `gfx/001.000_player.anm2`. Build-time extraction reads
their frame counts and nonlooping flags from the same authored catalogue used
by the renderer. No captured animation clock or fitted delay enters runtime.

`PlayerAnimation::force_play` resets the selected main clip and sets the
animation guard. It follows Player50a5c0 and Sprite40fd80; controls, visibility,
velocity, and completion callbacks remain caller-owned. Full Player updates
advance the main sprite before the current-clip completion query clears the
guard, following Base447190, Sprite40f920, and Player531850. Intermediate
updates retain the main frame. `is_finished(None)` queries the owned current
clip. Walking and Pickup presentation are unowned here and cannot satisfy
that query.

The main guard is not a movement veto. Source529750 uses controls7d4 and
input7d8; source52f370 separately suppresses shooting input while bf4 is set,
continuing its full timer work. The concrete normal45 active caller owns
UseItem and its copied item presentation. Its completed main clip returns to
ordinary Player presentation in the full callback.

The copied item sprite has a separate late full-update owner at531e6f,
after active/pocket/shot callbacks. Main Pickup playback still advances in
the early Base portion. Resetting the copy on a full-phase use therefore
advances it in the same callback; fast updates retain the copy frame.

`Player.visible` models Entity+31d independently of Player+7d4 controls.
The renderer consumes this visibility and the selected authored main clip.
New-run initialization resets the shared animation owner. Pickup acquisition
retains its existing presentation owner and queue.

This is a mapped integration prerequisite for the concrete ordinary stage
callbacks and normal Death caller. It does not claim fresh live acceptance of
those systems. Required public original/browser gates must join the actual
main clip, frame, playing flag, guard, visibility, and caller transitions;
stage readiness must use successful owned asset loading. It introduces no
public command to force an animation or stage change, and no unit suite.
