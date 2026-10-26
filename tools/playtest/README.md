# Playtest builds

Tools for publishing a playable build of `main` as a private Claude Artifact
and playing it before anyone else does. They are not parity checks and are not
registered as live e2e suites.

```sh
source cloud/env.sh
npm run playtest:bundle                # .cache/playtest/bundle (+ manifest.json)
npm run playtest:self                  # 180s self-playtest; exits 1 on failure
npm run playtest:self -- --seconds 60 --initial-seed N --boot-epoch-seed M   # replay a reported run
npm run playtest:self -- --run-seed 1X37ZP4V    # type a run seed on the character screen
npm run playtest:self -- --boss --run-seed VZ1FV4J6   # head for the boss room (Dingle)
```

## Bundle (`bundle.mjs`)

Builds the production game with a relative base and lays it out for Artifact
limits: page ≤16MB, files ≤15MB, ≤511 files, ≤64MB per publish call and
≤256MB per version.

- SFX WAVs, PNGs and the WASM engine (~30MB, ~920 files) are concatenated into
  three ~12MB packs, named `.wasm` because Artifacts serve only web media
  types. The inline `loader.js` fetches them, then serves the original bytes
  through `fetch()` and `<img>.src`, so the runtime's pinned byte checks see
  the authored assets.
- The 83 music tracks stay the authored Vorbis files (~183MB). Re-encoding
  was rejected: decoders disagree on Vorbis end trimming and the runtime pins
  each track's bytes and decoded length. The bundle is ~216MB in 88 files, so
  the first publish takes four calls.
- File names are stable. `manifest.json` lists every file's hash and plans
  publish batches; `--previous OLD/manifest.json` plans only files whose hash
  changed (normally the page, `game.js` and the packs: one call).

The loader also carries two counted browser-compatibility shims, reported by
the self-playtest: a fallback when `HTMLImageElement.decode()` rejects under
many concurrent decodes, and trimming of decoded Vorbis to the final Ogg
granule for decoders that ignore end trimming. Production `dist/` has neither,
so a browser that needs them cannot boot production.

## Self-playtest (`playtest.mjs`)

Serves the bundle the way the Artifact host does, boots it in Chromium
(SwiftShader WebGL), clicks to focus, presses Enter through the menus into an
OS-random run (or a typed `--run-seed`) and plays with real keyboard input. It routes
around rocks over the room grid, fights enemies that hold the
doors shut from a clear line of fire, collects pickups and items, explores the least recently visited
rooms, returns to the start room, bombs, pauses and restarts after death. With
`--boss` it heads for the floor's boss room instead, so a publish exercises
the newest boss: pair it with a run seed whose boss room holds that boss.

It fails on uncaught errors, console errors, failed requests, a frozen
simulation (no tick for 3s), black frames, held movement that never moves the
player, a player who never leaves the start room or never fires a tear, and a
pause menu that does not open, freeze or close. Doors that look open but never
let the player through are reported as warnings with the run seed.

Output: `.cache/playtest/runs/<time>/report.json` with the verdict, stats,
frame times, timeline (initial/boot seeds, run seeds, rooms and NPC kinds) and
screenshots. Frame times come from software WebGL and are not representative
of real hardware; simulation ticks/s is the liveness figure.

## Side-by-side footage (`side-by-side.mjs`)

Records the fresh original (under a virtual display) and the production build
replaying one paired case's public tape, then stacks the frame pairs into one
MP4: original on the left, port on the right, at real-time speed. It only adds
capture commands to the tape (every `--every` presentations, default 2), so
neither game receives input the case did not declare. Footage is a viewing aid
for playtests, not parity evidence.

Any paired case is a state the video can show. The run seed fixes the level
generation, and the tape's keys walk the route, so scenario helpers reach
specific boss fights, item pedestals or stage exits:

```sh
npm run playtest:video -- --case references/isaac/foundation/runtime-progress/1E0WJX06.json
npm run playtest:video -- --helper tools/original/entities-scenarios.py \
  --helper-arg --only --helper-arg dingle --from gameplay-start --extra 600 --out DIR
npm run playtest:video -- --helper tools/original/stage-exit-scenarios.py \
  --helper-arg --contrived --helper-arg E1T36YXW-monstro --helper-arg --count --helper-arg 0
npm run playtest:video -- --reel DIR/a/side-by-side.mp4,DIR/b/side-by-side.mp4 --title reel --out DIR
```

- `--case FILE[#id]` takes a `case.json` or a scenarios file (any e2e run's
  `scenarios.json`, `references/isaac/cases/`); `--pick ID` selects cases.
- `--helper SCRIPT --helper-arg A ...` runs a scenario helper first (the
  entity helper needs `cargo build --manifest-path engine/Cargo.toml --bin floor`).
- `--from LABEL` starts the footage at a tape capture label (for example
  `gameplay-start`); `--extra N` keeps both games running N presentations
  past the tape with no input, which shows how a fight plays out.
- Output: `DIR/<case>/side-by-side.mp4` and `result.json`; frames are deleted
  unless `--keep-frames`. A side that stops early holds its last frame.

## Publishing

Publish `index.html` as the page and the batch files from `manifest.json` via
the Artifact tool's `files` map to the same artifact URL, labelled with the
short sha, and attach the side-by-side videos for the build to the playtest
thread. Keep the previous `manifest.json` to plan the next publish. The
publish ledger is the private "Isaac playtest log" artifact.

## Sandbox (`sandbox.mjs`)

`npm run playtest:sandbox` serves the production build with a generated
Challenge 1 "Sandbox": starting items, a starting card or rune and extra Start
Room spawns, optionally in a 2x2 Start Room (default preset `sandbox-2x2`:
Blank Card, The Battery, Jera, corner pickups and a Lil Battery). Open the
printed URL and choose Challenges, Sandbox. `--preset NAME` picks another case
from `tools/original/pickup-interaction-scenarios.py`; `--out FILE` only writes
the `{challengesXml, specialRooms}` JSON that the page's `/?sandbox=URL` hook
loads. The JSON derives from original resources, so keep it local or in
private artifacts. `side-by-side.mjs --case RUN/pickup-interactions/scenarios.json#CASE`
records any of these cases next to the original. See
[pickup-interactions.md](../../docs/pickup-interactions.md).
