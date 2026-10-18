# Options Gamma

The Options Gamma row changes the actual display value by f32 steps of 0.01,
clamped to 0.5–1.5, using the existing held-key cadence. Its percentage uses
f32 arithmetic and the authored number layers. Back saves Gamma in the explicit
V4 preferences record; a normal document reload loads it before construction.
The final display pass transforms RGB while preserving sampled alpha. Gamma 1
uses the existing final program and draw path unchanged.

The fixed route and first unfiltered OS seed/boot case (`1K6Q HDDF`) both pass
688 functional checks. The [recorded whole bank](../references/isaac/cases/options-gamma/recorded-fixed-and-1k6qhddf-v1.json) therefore passes 1,376/1,376,
including actual input, settings, percentage, GPU parameters, Back and reload
owners. The first build passes the four Browser runtime cases. A Source-derived
sampler precision correction keeps every functional expectation, actual value
and verdict unchanged on the same Native evidence and public inputs; all 26
required percentage regions then compare exactly. The ten earlier required
pixel failures remain preserved. Full-frame diagnostics remain partial: 10/40
exact, 30/40 failed. These results belong to the main726-based Gamma build.

Preferences V4 deliberately changes the storage key and saved record. Historical
V2/V3 reports retain their original contracts. Shared input, audio, simulation
and visual regression results are assessed separately from those new IO
expectations. Same-Native main controls preserve all 2,996 shared Map and 890 shared Audio expectations, actual values and verdicts. Map required RGBA remains 33/48 and Audio diagnostics remain 16/32, with zero lost passes. The explicitly changed V4 IO checks and new Gamma-default checks pass separately. One recorded audible Music route passes 261/261; 259 expectations and actuals are unchanged, while two device schedule checks retain their own observed clock values. This does not qualify current OS Music.

This is a bounded default color-configuration slice. Filter/HQ4X, the Popups
notification consumer, arbitrary Exposure/Brightness/Contrast settings, Native
reload and complete shared-MT parity remain open. It does not claim full-game
RGBA equality or completion of every Options callback.
