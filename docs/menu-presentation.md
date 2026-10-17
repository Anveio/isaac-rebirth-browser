# Menu presentation evidence

`engine/src/menu_presentation.rs` reconstructs the MenuManager presentation of
the Linux Rebirth binary: the board every menu page sits on, the view that
pans across it, and the per-page animations. Addresses are in the unwrapped
`isaac.x64`.

## Board and view

- Page targets are the +88CF0 table (negated origins): Title (0,0), Save
  (0,-270), Game (0,-810), Character (-480,-810), seed entry (-960,-810) via
  59CCD0, Challenge (480, 59CBC0) where 59CBC0 scrolls 18 per row after the
  fourth challenge and rests at -1025 past the fifteenth.
- Update 59B180 runs before the page update. It snaps the view to the target
  unless MenuManager+0 (moving) is set; otherwise it moves
  `max(1, 0.1 * len)` along the f32 unit vector and stops once `len <= 1`.
- SetMenu 59CAC0 sets moving between two nonzero pages. Save to Game resets
  the carousel (5A0680); Game back to Save only reloads the selected file's
  drawing sheet (5A8B50), so File select resumes its clips.
- Render 59C4xx draws EmptyScreen with the view as a wrapping source offset,
  then Title, Challenge, Game, Character (with seed entry), Save, the
  Back/Select widgets (59B5E0, 6 px per frame, Back at (10,215), Select at
  (400,195)), MenuOverlay and MenuShadow.
- The page update dispatches on MenuManager+38 before that iteration's input,
  so the Title update that reads Start still advances the title clip.

## Pages

- File select 5A9490 updates sprites on even System counters. Moving the
  selection plays the old file's `FileNDeselect`; a finished clip that is not
  the selected file's `FileNSelect` plays it and the file's drawing
  `Animated` loop. Drawings sit on the clip's Drawing1..3 nulls.
- Carousel 59DD20 turns the shared angle by at most `step / 75` toward
  `slot * step` (slot `(n - selection) % n`), then places each portrait from
  `(cos, sin)(-i * 2pi / n) * 70` rotated by `90 - angle` degrees (f32
  0x3C8EFA35), offset by (240, 70) and pulled toward the screen centre.
  Render 59F8A0 sorts by height and uses `min(1, (y - 135) / 50 * 0.5 + 1)`
  as both scale and alpha. Random's label is at (240 - width/2, 165).
- Hard mode: character render 59FA6B fades the BloodStain sprite by 0.05 per
  render; MenuManager render 59C5B9 fades a second MenuOverlay pass by 0.025
  up to 0.5. The difficulty widget frame follows the even-tick sprite pass
  before that iteration's input.

## Replay

`docs/menu-tour-case.json` walks Title, File select, main menu, Character
(carousel turns, difficulty, seed entry) and back to Title. Record it with:

```
npm run playtest:video -- --case docs/menu-tour-case.json --every 1 --keep-frames --out DIR
```

On 2026-10-17 all 1405 port frames matched a fresh original capture pixel for
pixel. The Challenge page pan is implemented from the same tables but is not
on this tour.
