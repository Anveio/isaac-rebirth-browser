# Fresh ambient grid decoration and Room costs

Fresh normal starting rooms now admit the source variable decoration loop (kind 1, variant 0)
and own the actual signed 448-word Room+e98 cost table. Room.Init zeroes that
owner before grid admission. Wall.Init5717e0 and Door.Init5628e0 store 1000 at
their actual cells; Door.Init also maps state0/2 to1.

The shape0 loop constructs its local RNG35 from
`(descriptor.decorationSeed & 0x7fffffff) + descriptor.listIndex`, with wrapping
u32 addition. It makes `2 * (bounded4 + bounded5)` attempts. An occupied pointer
advances once. A null candidate draws the shared caller seed before the signed
cost greater than 900 guard; accepted objects then execute the two actual GridBase raw draws
and primary-only InitSeed. Every null attempt advances twice. The real local
owner and constructor admissions determine the draw sequence.

`Room.path_costs` is an optional `room_costs::RoomCosts` owner with 448 signed
words and get/store/entries/entries_mut access. Supported fresh Room.Init admits
it. Other unimplemented room producers retain None. Authored selectors and
collision metadata remain separate fields.

## Fresh original/browser proof

Production source and gate head: `ed8f2b1aa013526909b925eede0544c41f93524f`,
based on main `5c8a9fb89578167998b859ec485289ef2c9b5168`.
`verification.json` records the immutable artifact, source/native identity pins,
actual cases, every required failure and complete source/port sampled records.

Two contrived and four recorded OS-random boot/reset cases completed against
fresh original processes and one unchanged production artifact:

- All 2,688 actual signed cost-word comparisons pass.
- Whole live-grid occupancy matches 6/6. Native and production null counts are
  88/83/88/85/86/87, with each actual set compared.
- All 360 sampled kind/variant/state/sourceWord14/initialized comparisons pass.
  All 12 authored primary seeds match; the other 60 primary and 72 secondary
  sampled seeds differ.
- Copy ownership and full public input provenance pass 6/6. The proof pins
  36 owned executable identity reads and 96 intended/observed browser commands.
- The required suite remains 0/6, with 228 seed/record failures retained.
  Complete postrun MT differs 6/6 and remains an explicit diagnostic.
- Fresh strict W0 replay passes 18/18 functional checkpoints on the same
  artifact. Pixels match 7/18 frames, with 11 diagnostic differences retained.

The canonical gate is `npm run test:e2e -- ambient-grid`. It compares all 448 live
costs, bounded current-row seed pairs and seven-field records, and complete
physical-grid occupancy. Differences make the case and suite fail; every
selected case executes. Native data supplies comparison outputs only.

`CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 npm run check:build` passed assets,
matrix, TypeScript, production WASM/Vite and strict all-target Clippy. The actual
WASM is 1,363,405 bytes. One ordinary public-admission snapshot is 192,255 bytes;
the cost array contributes 1,029 bytes. Isolated Node WASM tick+snapshot over 64
samples measured 2.25 ms median/2.68 ms p95; these are cost measurements, not
original performance or parity evidence.

## Remaining source scope

This leaf covers the fresh normal starting room and shape0 producer. Other
backdrops/shapes, mixed transition admission, later Door/derived callbacks,
weighted-path reservations, third-full cost decay and durable Save/Load remain
separate owners. PlayerItems Save/Load is also unimplemented. The complete
shared stream still requires the actual missing run/room/effect consumers;
its numerical gap is disclosed independently of the accepted cost/occupancy
mechanisms. The preserved PR67/e7267ba proof remains unchanged.
