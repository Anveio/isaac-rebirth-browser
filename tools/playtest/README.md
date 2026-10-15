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

## Publishing

Publish `index.html` as the page and the batch files from `manifest.json` via
the Artifact tool's `files` map to the same artifact URL, labelled with the
short sha. Keep the previous `manifest.json` to plan the next publish. The
publish ledger is the private "Isaac playtest log" artifact.
