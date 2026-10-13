# Normal Basement floor routing

The production runtime now generates its starting-room decoration seed and
visible doorway locations from the public run seed. `floor::generate_basement`
accepts only the derived stage seed. Both catalogues come directly from the
checked-in original Special Rooms and Basement STBs. Native descriptor maps,
selected templates and RNG cursors are comparison outputs, never generation
inputs. `room_assets` preserves every authored spawn group and selector.

The supported generation contract is fresh-profile normal Isaac on Basement I
without Curse of the Labyrinth. Other curse effects, unlocked profiles, alternate
stages and later floors remain separate work. The structural comparison also
covers an independently selected Curse of the Blind run; that curse does not
change the generation branches used here. It does not establish item-visibility
behavior. Door traversal and room-content admission are the next implementation
step; this change does not substitute empty neighboring rooms for authored spawns.

## Source boundaries

All executable addresses below refer to the independently unwrapped Linux
v1.05 binary SHA-256
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.
The wrapped depot executable SHA-256 is
`c1b2ca9375d60d4354f9086871913901428af41b82a5892a0d235c5d8ae2f768`.
The existing Linux manifest pins the archive bytes. Reproduce extraction with
`tools/original/compat/unwrap.py` and `tools/foundation/decode-rooms.py`.

| Boundary | Recovered contract |
| --- | --- |
| `58be30`, `587ec0` | Independent primary RNG35 and secondary RNG2; reset consumes primary Next, then backdrop Random(1), count Random(2), generator constructor Next. Fixed global MT grid-reset consumption is a different stream. |
| `591a50`, `5910b0`, `5907c0`, `5918c0` | Rectangle growth and bounded dead-end expansion use the generator's own RNG35. Queue exhaustion below requested count logs without rejecting the attempt. |
| `58eb80`, `58f1b0` | Distance selection-sort of ends, with actual tie swaps; remaining rooms concatenate non-ends in creation order and the current end list. |
| `5d5600`, `5d5878`, `5d5490` | Local RNG7/f32 weighted choice and 0.1 weight decay. Authored weights reset once before the outer retry loop, not per attempt. |
| `584240`, `5897e0` | Fresh-profile boss family and optional room branch draws, including admission draws after failed fitting. Game flags carry across retries, with actual source set/clear operations for boss bits. |
| `58f480`, `58fad0`, `58fcf0`, `58f160` | Small special rooms crop at the incoming connector. Boss requires distance greater than one. Rejected optional placement appends its index to the end list and retains the cropped shape. |
| `589300`, `590380` | Secret candidates exclude absent authored door slots and every neighbor of boss/supersecret rooms. Each eligible empty cell draws Random(5) before neighbor scoring; final tie selection consumes another generator draw. |
| `58c260`–`58c315` | Retry checks minimum five ends and mandatory boss/supersecret/shop/treasure fitting. Generator state, primary state, template weights and game flags continue. |
| `584bc0` | Descriptor seed derives decoration/spawn/following words through three local RNG12 draws. |
| `58c337` | Post-placement level primary Next. Serialized `Floor.primaryState` stops at this boundary, before further RoomInit/FX consumption. |
| `5c78b0`, `5610e0`, `5629c0`, `565b00`, `563b70` | Authored padded door indices, common render-ground projection, normal inset 15; secret entrances select invisible Hidden, first-floor treasure is free, shop/library use KeyClosed. |

`Floor` serializes complete descriptors, connections, generation streams and
current catalogue weights; `State` retains the current and visited descriptor
indices. RoomInit spawn selection, typed placeholder resolution, FX stream
routing and post-generation primary consumption are not inferred from these
fields.

## Live acceptance

The promoted Rust model matched 17 complete original floors: ordered descriptors,
all 169 grid cells, template identities and dimensions, eight-slot connectivity,
all three descriptor RNG12 words and actual attempt counts. This includes the
previous MPCM and GS special-cropping failures and three independent actual
retry runs. The retry codes QNXPTVT1, 0YAZCZ7N and BHX1EF8A each report two native
map attempts; they are test inputs, not runtime exceptions. Further OS-random
originals are captured after the implementation is frozen.

Run the production browser paired gate:

```sh
source cloud/env.sh
cargo build --manifest-path engine/Cargo.toml --locked --bin floor
npm run build
bash tools/original/with-display.sh node tools/foundation/floor-e2e.mjs --count 2
```

This executes those three contrived retry cases plus two OS-random cases in
isolated fresh native profiles, then enters each seed through actual browser DOM
keys against an immutable production WASM build. It compares complete generated
floor descriptors, topology and retries with the originals. The source helper
retains and validates the public X11 tape, consumed input acknowledgments, actual
clock phase, native binary identity and source log. Every selected case is
reported; unsupported or mismatching originals fail the gate. Bulky captures and
reports remain in `.cache`.

For retained observations, or to recheck cached independent source runs:

```sh
python3 tools/original/foundation-floor-verify.py --report .cache/floor-native.json
# Additional manifests can be supplied as positional arguments.
bash tools/original/with-display.sh node tools/foundation/floor-e2e.mjs \
  --source-report .cache/floor-native.json
```

Supplied-input `layout` and `room_templates` diagnostics remain available with
their original narrower scopes; finalized geometry is verified by the new floor
entry rather than relabeling the old growth kernel.
