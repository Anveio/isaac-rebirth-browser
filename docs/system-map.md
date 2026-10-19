# Base Rebirth systems and evidence

The [roadmap](roadmap.md) is the planning view: every subsystem's state,
milestones and outstanding bugs triaged by subsystem.

This map tracks the private base Rebirth reference set and the implementation
wave starting from the Rust/WASM foundation at `936fda4`.
The [generated test matrix](../TEST_MATRIX.md) indexes executable
scenarios and current implementation contracts. A declared test is coverage;
only an actual report establishes its outcome. Rust/WASM replay equality checks
the port's determinism, while original logs, graphics and controls establish
independent reference evidence.

## Source implementation and live scope

Merged source and a passing captured artifact are separate qualifications. The
records below retain their own source, WASM and input bindings; none is an
automatic rerun on advancing `main` or confirmation of the served build.

PR680 adds Hopper 29.0, Trite 29.1 and Flaming Hopper 54.0 source through
`engine/src/hoppers.rs`, including typed admission, hop/grid behavior and fire
conversion. The existing `entities/hopper` and `entities/trite` drivers declare
admission coverage. Independent live qualification on pinned main `ca408124`
is **UNRUN**; merged source and inputs establish no Original/Browser,
fire-conversion, shared-MT or pixel acceptance. Bulk imports keep their owner.

Caves I/II generation places Double Trouble's 26x7 and 13x14 boss rooms
through 58f480's candidates (source-decoded; no native Double Trouble layout
is graded yet). Separately, the Caves I wall catalogue is built before selection
and labels set 1 as patches 0..2 instead of 3..5. The reviewed catalogue
correction is unbuilt and live-unrun; it changes metadata, not wall quads or
RNG order. A real set 1 arrival and unchanged Basement controls remain
required. Caves II (stage 4) generates, including the shop's surprise-miniboss
flag and first-entry Greed/Super Greed template substitution. PR731 admits
Caves I's trapdoor into Caves II. Its `stage-descent` capture on `ad1a60b5`
passes 247/247 paired checkpoints per fixed/recorded-OS case and 2,744/2,744
and 2,748/2,748 required tail checks through arrival. Each game plays its own
earlier fights; this does not grade those fights, a complete Caves II layout,
Double Trouble or Greed parity. Raw coin/bomb asymmetries and MT trapdoor-seed
diagnostics stay retained. Caves II's exit still refuses stage 5: PR732 imports
the 506-room Depths catalogue, without generator or descent admission. See
[layout boundaries](room-layout-evidence.md) and
[the artifact-bound descent](stage-exit-evidence.md#caves-ii-descent-413).

| Area | Implemented source | Independent live scope and remaining work |
| --- | --- | --- |
| Options audio/display leaf | Partial eight-row main-family navigation, actual SFX/Music settings, preferences loaded before construction, Back save and a Browser fullscreen intent/result bridge. Controls and Map opacity have separate bounded scopes; Gamma has a separate bounded functional slice below; Filter and Popups callbacks remain separate. | The 37-file rebase on head `402af20e` preserves PR629/630 and final footprint tools. Its immutable artifact `99f8986d`/1,407 passes build and the same-Native 428/428-field Options replay with zero errors. All 428 rows and 16 Browser PNG bytes equal historical 3a9; this is not computed Native RGBA equality. The same artifact also passes both unchanged Browser runtime tests (2/2, no skipped or flaky cases). Default paired-kernel replays preserve every prior row and actual value: fixed 5999/5999 passes; recorded HH936E8Z retains 78 failures among 6060 checks, with zero lost passes. The historical main4820-based artifact 3a9 passes runtime 2/2, fixed SFX 368/368 and canonical free-SKU 1778 plus four named transport rows; its outer launcher FAIL remains beside the strict additive readout. Main 16d8 retains typed-zero FAIL. The historical 403/1112 history cap FAIL remains. The registered `options-audio` suite first passes 866/866 fields on a716/99f with one fixed and one unfiltered OS seed/epoch/hold case. Fresh Native/Browser integration on main634-based `4ca034b8` replays that unchanged fixed plus recorded `JWMM YY0Y` bank: 866/866 pass, without new draws or filtering. All 32 Browser PNG bytes equal 99f; fresh Native object identities and journal sequence labels differ, with zero verdict changes. Current history caps (400/1112 and 400/1142) and 16/32 pixel diagnostics remain failed. Current capture-tool producer `404a5617` replays the same fixed/JWMM inputs against fresh Native processes on unchanged `2a706930`: 866/866 fields pass, all 32 Browser PNG files equal main0ad and 50 raw pointer/sequence rows differ with zero verdict changes. The outer raw-index guard fails; independent diagnostics verify unchanged staged entries, Source, artifact and inputs. That receipt and all history/RGBA failures remain preserved. Other callbacks, broader OS/reload, shaders, integrated fullscreen/fatal recovery and full mixed PCM/MT remain open; durable Controls tooling remains separate. |
| Map opacity and map surfaces | Partial Options opacity/gauge, repeat/clamp, Back persistence, versioned preferences, configurable Map input and small/expanded surfaces. | Frozen `48e1477f`/1,408 passes build, the required 32-function WASM interface and runtime 3/3. The unchanged fixed plus first-unfiltered-OS bank passes 3,122/3,122 functional fields, preserving all 2,942 old passes; four invalid fade expectations are explicitly corrected and 176 checks are additive. Required RGBA is 33/48 exact: all 26 gauges, seven fixed Playing regions and zero OS Playing regions. Fifteen required-region failures and 50/60 full-frame diagnostic failures remain. Music passes one recorded audible lifecycle case, 261/261, without current OS Music coverage. Options Audio V3 fixed plus recorded JWMM passes 948/948; both history caps and 16/32 pixel diagnostics remain failed, and the first unlaunched OS reserve failure stays retained. Same-Native Save fixed plus recorded OS preserves all 3,748 ordered field values/expectations/statuses versus frozen137b: 3,650 pass and 98 fail, zero lost passes; 66 own-save checks, 48 MT snapshots and two natural reloads pass. Save RGBA stays 34/62 exact with 28 failed metrics changed. Banner [#600](https://github.com/Anveio/isaac-rebirth-browser/issues/600) and lighting [#268](https://github.com/Anveio/isaac-rebirth-browser/issues/268) remain open. See [the bounded Map evidence](map-opacity.md). |
| Options Gamma | Partial f32 row callback, percentage, final RGB pass with retained alpha, explicit V4 Back persistence and Browser reload. | Two fixed/first-unfiltered-OS public cases pass 1,376/1,376 functional fields. Same-Native replay after the sampler precision correction preserves all functional values and verdicts and passes 26/26 required percentage regions; the ten initial region failures remain retained. Full-frame diagnostics stay 10/40 exact. Initial Browser runtime passes 4/4. Historical V2/V3 IO and shared regression scopes remain separate; Filter/HQ4X, Popups, arbitrary color configuration, Native reload and complete MT remain open. See [bounded evidence](options-gamma.md). |
| Keyboard Controls | Actual 24-action/two-column keyboard mapping, ascending raw GLFW scan, duplicate clearing, completeness/Defaults and Back persistence. Gameplay consumers use the owned map with non-Skipped polling and physical bomb edges; focused DOM ownership and separate origin-local InputConfigs are implemented. | On main632-based artifact `d10fd511`, four retained-Native fixed cases pass 2,904/2,904 fields; one fresh unfiltered OS seed `9XZMZPR0` passes 784/784. Browser runtime passes 2/2. Two fresh default held-E Full/Fast bomb cases pass 370 fields; all ordered rows and 36 Browser PNG bytes equal parent `99f8986d`, with no lost passes. RGBA/history failures, the historical duplicate-case 3/782 failures and explicit unsupported callbacks remain retained. Fresh main173-based artifact `4ca034b8` passes runtime 2/2 and exact fixed W→I plus recorded OS replays: 1,568 fields and all 44 Browser PNG bytes equal d10, with zero gains or losses. Its RGBA 18 pass/26 fail and history caps remain; other d10 results retain their older artifact scope. The registered gate on main0ad artifact `2a706930` passes four fixed routes plus unfiltered OS `M3MJ EY2Z`/epoch `670687023`: 3,688/3,688 fields, 109 checks, 302 trusted physical edges and zero Browser errors. All 2,904 fixed ordered field values and 83 Browser PNG bytes match historical witnesses; two movement aggregate checkpoint names differ. Fresh shared Options fixed/JWMM regression passes 866/866 with 32 identical Browser PNG bytes; 64 raw pointer/sequence rows change with zero verdict changes. Controls strict RGBA 45 pass/60 fail, Options 16/16 and all history-cap failures remain. Current capture-tool producer `404a5617` replays all five recorded cases against fresh Native processes: 3,688/3,688 fields pass, every ordered row and all 105 Browser PNG files equal main0ad. Shared Options adds 866 passing fields and 32 unchanged PNG files. No new random inputs are drawn. The outer raw-index guard fails; independent diagnostics verify unchanged staged entries, Source, artifact and inputs. Its failed receipt and all history/RGBA failures remain preserved. The opt-in main668 Keyboard document-reload slice passes 298 checks across fixed and recorded OS W→I routes: actual Back save, natural constructor load, separate reset-load and trusted remapped gameplay. Default/reload comparisons preserve all 1,568 inherited expectations, actuals and statuses and all 44 original-document PNG bytes; RGBA remains 18 pass/26 fail. Options V4 and InputConfigs storage stay separate. The whole five-case bank is validated before selecting two; no new Native process or random draw is used. Native process reload, Browser OS-process restart, controller/mouse, further remapped callbacks and disabled-interpolation/Skipped live coverage remain open. See [exact scope](controls-keyboard.md). |
| Fresh Character seed-effect mode | Fresh NewRun validates and forwards the actual compact menu code to the existing Seeds+48 lookup before startup streams; Challenge keeps its separate input path. No RNG or inferred neutral mode is added. | Frozen cc38 passes all 1,749 existing free-SKU checks plus 29 direct actual Native/Browser seed-effect owner checks; unchanged-main9841 retains a missing owner. The canonical comparer requires this owner separately from existing gameplay and ChallengeID rows. Restart/legacy, untyped live entry, special-seed effects, room-clear placement and complete MT/pixel/audio parity remain separate. See [owner and replay scope](run-admission.md#fresh-character-seed-effect-owner). |
| Rectangular room art | [PR576](https://github.com/Anveio/isaac-rebirth-browser/pull/576) sizes floor, wall, boss-wall and shading assembly from the four base rectangles: 13x7, 13x14, 26x7 and 26x14. | The qualified c194/a985/W8c7 WF82 replay passes 635/635 Original/public-WASM State comparisons and 571 camera-history phases, reaching standard and tall rooms. [PR582](https://github.com/Anveio/isaac-rebirth-browser/pull/582) records that scope without a new run. Wide/double and boss-specific live art, completed current-build Browser camera PNGs and full-frame equality remain unqualified by those reports. |
| Browser GPU masks | The existing presentation test compares retained independent native shadow/light targets. | The fresh recovery run on production `8beae6bbe77714fb59236a91510cf4fa6892be28326b645db77798f6c8a789ae` (1,407 files) passes all nine selected cases with zero skipped. Original state parity and simulation-tick delivery remain false. This is retained-target Browser equality, not a fresh Native capture or wide/double/overlay RGBA qualification. |
| Wide/double bounds and coverage | The existing room specs compare cache dimensions, floor/wall bounds and covered floor/wall cells with independent native bounds. | On the same recovery artifact, `room-26x7-random-0` / 6RG7LV8Z passes 3,150/3,150 art comparisons (630 per predicate); `room-26x14-random-0` / V01TLK80 passes 2,870/2,870 (574 per predicate). Both cases remain red: wide has 2,199/28,690 field failures and 196/637 paired matches; double has 2,229/32,971 failures and 188/581 paired matches. Decoration, projectile-seed, camera-scroll and player-position failures remain preserved. Actual Original/Browser PNGs are retained; no RGBA, shading or overlay equality follows. |
| Overlays | Main still selects the Basement 1x1 overlay and its 663x429 crop for every rectangle. | Size-specific overlay selection/crop and camera-shake routing remain separate source work; an independently built candidate does not establish integration or live images. No positive base narrow/L contour evidence is established. |
| Pill and cache owners | Canonical Pill28 history, reached Range, Range Down/Up use, independent active words, reached Move Speed and the controlled Knife's canonical active identity are merged through PRs572/580/586/595–597. | [PR586](https://github.com/Anveio/isaac-rebirth-browser/pull/586) reports 5,639 passing checks on its frozen candidate; [PR602](https://github.com/Anveio/isaac-rebirth-browser/pull/602) persists `pill-range-use` and reports a 1,409-check retained-Original replay on c224/ee04/Wfa54. These do not qualify the whole current runtime, other Pill effects, complete ceremony, pixels or PCM. |
| Menus and imported bosses | Menu page/carousel presentation and Envy/Super Envy source are merged. | [PR601](https://github.com/Anveio/isaac-rebirth-browser/pull/601) records 1,781/1,782 menu frames and closes #587–589; [#600](https://github.com/Anveio/isaac-rebirth-browser/issues/600) retains the stage-streak placement gap. [PR605](https://github.com/Anveio/isaac-rebirth-browser/pull/605) retains Envy failures of 475/14,893, 468/14,982 and 2/13,320; Super Envy room coverage remains pending. |
| Green champion creep (PR656) | Green lifecycle owners share the existing pool callback; actual admission/NPC counters, incoming contact immunity, contact phase order and Gaper Fast grid preview are corrected. | Frozen `35edb299` passes 173,135 fresh Original/Browser entity checks, 419 paired checkpoints and all seven lifecycle/Acid milestones per side; Native damage causes join to live 1000.23 identities. Three same-Native regressions retain all 269,637 entity and 9,605 paired field keys with zero lost passes and two Fast gains. Fixed MT 196,875 and recorded OS checks pass. Older routes' missing Acid witnesses and identity-key changes remain retained. New-route RGBA stays 7 pass/412 fail; longer-tail whole MT remains unmeasured. Combined Gamma/starting-trinket frozen `62300f05` replays all four retained Native cases: all 456,532 inherited entity/paired fields and 196,875 fixed MT fields are unchanged, with all 134,916 Gaper-spawner rows passing. The four older missing-Acid failures and all 1,390 pixel verdicts/metrics (28 pass, 1,362 fail) remain unchanged. Prior Bomb/NPC controls retain their own scope of 4,364 fields and 38 gains. The [public input bank](../references/isaac/cases/entities/gaper-recorded-fixed-and-dt3mfa-acid-v1.json) contains no captures; its other three case inputs are unchanged. |
| Closed Shop/Super discovery | One closed 8,192-draw OS bank, independent complete Source floor evaluation and no outcome filtering. | Shop selected 0/2 and remains FAIL; Super retained all 32,768 slot outcomes and selected ordinal 3,497 / `PHCVWE1C` via upper door 6→4. Ordinal 7,028 stays surplus. Its short admission prefix now has the separately scoped comparison below; the full fight author/lifecycle/clear coverage and whole cohort remain FAIL. See [discovery scope](shop-economy.md#closed-source-only-discovery). |
| Player footprint cadence | Footprint cadence and foot side read the actual Game counter, already incremented before ordinary Full and unchanged during ChangeRoom's extra Room.Update. | Actual same-Original Browser A/B on frozen main377 parent16d8 and candidate4029 grades 5,301 fields over 83 captures/76 matched step ends: 12 Super Envy root position/velocity gains, zero losses. Strict RGBA remains 7 exact/76 failing on each build. Only the 1,110-presentation admission prefix is qualified; full fight, Original MT, children, clear and broader #459 velocity remain open. The separate registered admission comparator catches six failures on parent16d8 and passes3,271 fixed rows on4029; its extended mixed bank passes5,884 fixed rows and retains78/5,947 OS Envy P/V failures with all actual admission guards passing. Fresh629 f0eb retains every prior row and adds115/113 passing draw-order-presence rows: fixed5999PASS, OS5982/6060 with the same78FAIL; overlapping NPC/player order is not exercised. See [counter evidence](player-footprints.md). |
| Normal Shop economy | Partial ordinary Shop owns descriptor stock table/cursor/sale/following RNG, real common SKU bodies, nested price/slot, paid grant/debit/copy, next-Full manager retirement and live reentry. | Frozen eb33/1,407 files qualifies an actual paid Bomb: fresh v9 source 117/117 and owned fields 390/390 pass. Corrected canonical 1,182/1,190 retains eight failures; historical raw 1,068/1,190 and 52/59 RGBA failures stay retained. Same-Native v8 retirement passes 373/373 with exactly eight fixes and no lost passes; v8 strict overlap remains FAIL. First-four source-authored normal-run SKU controls on frozen5c33 complete actual Native+MesaBrowser: free no-active1749/1749; paid no-active10479/10652, undercharged10809/10899 and collectible18175/18181. All64 summary witnesses pass; strict aggregate41212/41481 retains250 health+19 BF4 guard failures, raw40804/41146 stays archived. Complete typed wire/actual-owner readbacks lose no prior pass; PNGs retained, RGBA/PCM not computed. The registered free-no-active replay passes1749 functional+4 transport checks on9841; cc38 preserves those rows and passes29 added seed-effect owner checks. DefaultOS, freshNative wrapper execution, other current-runtime fixed cases, three other Battery branches and remaining SKU grants retain their separate scopes. Modifiers, eligible achievements, upgrades, ABD contexts, Continue, broader Full/Fast grid parity and complete audio/pixels remain separate. See [Shop scope and controls](shop-economy.md). |
| Gameplay frontiers | Basement II's trapdoor descends into Caves I and Caves I's into Caves II; Caves II's exit lacks an admitted next-stage generator and refuses descent; held active IDs survive the supported stage-save normalization. | [#413](https://github.com/Anveio/isaac-rebirth-browser/issues/413): the [Caves I descent](stage-exit-evidence.md#caves-i-descent-413) grades the Caves I arrival after two per-game boss clears. The [Caves II descent](stage-exit-evidence.md#caves-ii-descent-413) separately qualifies PR731's bounded arrival tail on ad1a60b5; Depths has a catalogue only and remains the next progression frontier. [#499](https://github.com/Anveio/isaac-rebirth-browser/issues/499) now has [scoped carried-active descent evidence](stage-exit-evidence.md): on frozen f837, two active routes and an ordinary control pass 7,642 tail checks, including 48 actual active-word/settled-descent checks. The OS draw varies approach actions on fixed Y9QL8QZM; it is not random-run-seed coverage. Pixel and per-game fight differences remain retained. [PR609](https://github.com/Anveio/isaac-rebirth-browser/pull/609) qualifies its source-derived run-admission clock cohort; the independently declared [death clock](player-death.md) resolves the separate producer mismatch in [#438](https://github.com/Anveio/isaac-rebirth-browser/issues/438), with the fresh OS cohort still failing a required Original audio observation. [Dead-callback motion](player-death.md) fixes517 Y checks in the scoped [#493](https://github.com/Anveio/isaac-rebirth-browser/issues/493) replay; broader living contact/grid parity remains unresolved. Ordinary-bomb late-grid response [#349](https://github.com/Anveio/isaac-rebirth-browser/issues/349) has [scoped moving-doorway and regression qualification](#ordinary-bomb-late-grid-response-349); separate NPC/bomb timing failures remain open. |
| Player Full grid after contacts | Ordinary Full Player motion/actions defer the retained ground query until after entity contacts, before other final ground queries, subject to the existing class0 bypass. Fast and standalone transition callbacks retain their paths. | Exact immutable 5c33 parent to 9841 candidate A/B reuses unchanged Native and captures all 266 Browser checkpoints/15,510 scalar fields: 36 gains, zero prior-pass losses or changed Native expectations, and 38 unchanged failures. The 40-gain prediction remains FAIL: 34 bomb birth-counter and four Full tear-alive/admission rows stay red. Both Full coverage guards pass; both Fast same-birth-solid-response guards fail. Strict RGBA is 28/266 exact with 238 retained failures. Same-Native Shop v9 preserves all 1,190 rows and 59 Browser PNG bytes; fixed declared-clock Restart/Exit passes 26,977 fields/1,481 checkpoints on each artifact with every actual value and 1,481 Browser PNG bytes identical. Full PNGs remain local; death RGBA is not computed. This Full-grid evidence does not establish complete physics, pixel, audio or whole-main acceptance. See [source and scoped controls](player-death.md#player-full-grid-after-contacts). |
| Death Exit menu presentation | Deferred GameOver Exit initializes a fresh Title actor at frame0 and black alpha1 fade before the existing System fade advance; an existing menu-controller lifetime retains its clock. | [Focused death evidence](player-death.md) on frozen `46a4d706` fixes 240 Title/fade fields among 13,140 canonical fields, with 12,900 identical rows, zero regressions or changed Native expectations, and 654 unchanged Game-age failures. All741 checkpoints, Native provenance and actual death/popup execute. Complete checkpoint states/PNGs remain local; aggregate state-history reaches its byte cap and RGBA was not computed. Exit music/SFX and public-clock qualification remain separate. |
| Death Exit audio | Fresh Exit MenuInit selects Music61 and SetMenu0-to3 skips forward-confirm SFX8. Visual Title/fade and public-clock leaves retain their separate qualifications. | Frozen `f837cf59` (1,407 files) repeats the exact seven-capture public Exit prefix against a fresh Native and actual Browser: all 95 canonical rows are identical to703, while 14/14 bounded Music/SFX8 owner/backend checks pass versus 10/14 on703. Four former audio failures become passing with no changed Native expectations or prior-pass losses. Native 0/Browser 0 actual SFX8 starts and zero observed Browser SFX8 events in the dispatch window require three earlier actual unique-owner SFX8 controls in each runtime; Browser controls reach the output graph with actual buffer hashes. Native 690 clock acknowledgements/eight capture times, Browser 1,631 public ticks/seven captures, source provenance and required death/popup coverage pass. All 15 actual PNGs remain local; RGBA was not computed for that initial prefix capture. Complete audio/PCM/music lifecycle/Shared-MT parity remains open, as do the separate fresh-OS required Original SFX30 observation and historical 733/740 Restart pixel failures. The registered player-death --exit-audio replay on the same artifact then passes 7/7 checkpoints, 95/95 canonical fields and 14/14 audio checks with mandatory Exit/request/control coverage; its seven strict RGBA diagnostics have four exact passes and three retained failures. See [death evidence](player-death.md). |
| Admission/death public clock (#438) | Run-admission retains its own PR609 qualification. The shared paired gate defaults its public clock schedule off; only the registered death driver opts into predeclared count-one timestamps through the no-draw step API. Actual Native epoch/bootstrap/interval/acknowledgements/capture times validate that prior plan and never choose Browser inputs. Omitting timestamps preserves the prior count-only command; count-one batching changes are explicit. | Combined frozen70314c6f passes all 2,962 retained checkpoints / 52,190 canonical fields, including 2,774 former Game-age failures; the 240 Exit-field gains belong to the separately qualified visual leaf. Registered fresh-Native Restart passes 740 checkpoints / 13,837 fields but retains 733/740 full-RGBA diagnostic failures. Both fresh OS clocks/provenance validate, yet the cohort remains **FAIL** on the required Original SFX30 observation at 7KMTELQG/hurt-89 despite all 25,196 emitted scalar rows passing. No waiver, resampling, causal clock diagnosis or complete audio/backend acceptance follows. Retained/fresh functional wrappers do not compute pixels. See [death evidence](player-death.md) and [run admission](run-admission.md). |

## Retained d995/Wd869 qualification snapshot

Public main `d995a582` includes refused-pedestal runtime PR557 and
optional Node tools PR558. Primary/playable remains `4543c59b`, serving the
qualified current-CF8 artifact `ed5b8d1e` / WASM `d869557b` (Wd869), product
`44ab4b10`/1403, with delivery `d526140f`. Its six production steps passed;
Use45 passes 43/43 for the real TEMP45 TTL0 record and continued updates, and
the selected item194 Own/reused-Original endpoint checks have zero failures.
Item194's five unknown labels, actual cache/Add invocations, positive-TTL expiry
and unrelated effects remain explicit. This artifact does not qualify557.

The 21 scalar cases belong to the prior Wecee artifact: 187,499 Own
checks with zero scalar/producer failures, 210 independent Original endpoint
checks with zero failures, and 85 unknowns. They are not a 21-case rerun on
Wd869. The old reports and their artifact bindings remain unchanged.

The retained key-fixed Original compared with Wd869 through the public-WASM
backend passes 422/422 presentations with zero failures (`e07f9f8a`), including
both real key collections. The unchanged default Pocket AL CLI control on
the same artifact passes 9,350 checks with zero failures and 11 unavailable
obligations (`8ec05bef`). This is a transport regression control, not another
mandatory pocket case: the earlier 3/11 scope remains. The Browser key attempt
and menu-audio attempts remain resource-partial with zero comparison checks.
Node results supply no Browser DOM, renderer or backend audio/PCM equality.

The separate Get4+557 source `35e65b91` / WASM `e5b6ace1` has a genuine
six-step production pass and sealed binding `88933f3a`. Its first fixed0
live3 case passes all eight new target-lifecycle laws, from actual real3
acquisition while the target is alive through the same Knife's registration,
contact, target death/retirement and cleared target slot. The corrected Own
diagnostic retains five required unknowns. Pending cache16 is MoveSpeed/D60
and remains recorded; it is not FireDelay and is not cleared or completed.
The original broad strict report retains 7,009 failures. Its retained-data
correction records 7,006 failures across 86,561 checks (`60c292d0`), with six
Get4 diagnostics and all eight target laws passing. Attribution remains open
until the same fixed0 case is compared on the prior Wd869 artifact; an unlike
historical case is not that baseline. Full cohort, callback and
broader synergy acceptance remain open. This result does not qualify191,
satellites, Get1, canonical Pill/Range/Luck or the separate pickup/transition
source candidates.

Every earlier row, strict failure and resource-partial report below retains
its named artifact and scope. A retained section's word “current” refers to
that recorded snapshot.

## Retained 05a6/Wecee qualification snapshot

Current public/primary source is `05a6dc7c`. Playable serves the six-step-qualified
`Hb355` scalar product `Wecee6363` / product `10270f93` through exact source
projection `642debe3` (delivery `aa538ade`). Current Own21 has 187,499 checks,
zero scalar/producer failures and 85 mandatory unknowns; its 21 independent
Original endpoint comparisons pass 210 selected checks. The separate AL Node
pocket case passes 9,349 functional checks. Separated fixed-phase0 and retained
recorded-0 each pass 29,385 checks with zero failures and 11 unavailable
obligations (428c4301/c0022b06). Initial2 closed exit 0 with stable inputs and
all owned children exited (212ea35d): three of eleven cases are complete,
with eight still mandatory.
AL's immutable report retains 12 unavailable obligations and unreached SAME-v1
AD0 refresh. DOM/rendering/audio-backend/PCM evidence is unavailable. Build, source
integration, selected functional results and complete parity remain distinct.

| Area | Source/public | Own/port-port | Original scope | Remaining behavior or evidence |
| --- | --- | --- | --- | --- |
| Public interfaces and artifact scope | Public main 05a6dc7c includes pickup prerequisites and Magic 8 Ball through PRs 538–541, query/Flags/Color through 543–545, and reached-query scalar consumers through 546. Primary/playable now serves the current Wecee6363/product10270f93 under the exact public-to-qualified runtime projection 642debe3 (delivery aa538ade). The H2a candidate plus public 534 and its Hb355 scalar successor each passed all six production steps. Build and source integration do not qualify unrun query/appearance cases, generic homing callers or broader pickup families. | The current Hb355 scalar product passed all six production steps and its actual engine/web and build inputs match public 05a6 under the explicit projection; docs/tool differences carry no extra runtime credit. Earlier SAFE6a/354 (Wa8eed73a/product9fe3006e) retains its own six-step PASS; SAFE Clippy/fresh floor remain UNRUN and not requested. Historical AA6 retains its own six steps, strict Clippy and floor, with floor reuse limited to discovery input. The staged SAFE NPC case completed Original/provenance, then Browser guarded before title readiness (0/315); later reuse admission stopped before Popen. No completed NPC browser comparison is inferred. | The prior private658 comparison against exact b319 has 40,655,111 checks / 9,452 failures, solely added fields/keysets with no changed or removed existing values/types. It excludes external492/494 and is not whole-current-main acceptance. | The first local13-draw difference is exactly one first-visit treasure jingle plus two light passes over three fires. The later first SFX269 request matches the source half-volume and cooldown. Other723,235 strict failures are not waived; Original ambience evidence remains open. |
| Stat inputs, acquisition history and Experimental Treatment | Public 516/517 own independently preserved stat inputs and acquisition history; 525 owns the reviewed 240 producer, 529 corrects Polyphemus to +5, and 538–541 add canonical spawner, seed-effect, ordinary placement history and 194 Card production. Query/Flags/Color 543–545 and scalar 546 replace whole-inventory completion with reached queries and actual field/effect provenance. Unowned histories remain pending; source-context plumbing alone grants no actor/getter/caller capability. | Current qualified Own21 completes 187,499 checks with zero scalar or producer failures and 85 mandatory unknowns; all 21 current-Own/independent-Original endpoint pairs pass 210 selected checks with zero failures (8aee4bda). Existing independent Original captures are reused after canonical validation, with their historic binding pinned separately. Historical AA6 Own21 retains 187,414 checks, its former two 194 producer failures and 85 unknowns; those immutable reports are not rewritten. Earlier 658 real199/240 and all failed owner fields keep their recorded artifact scopes. | Current 21 scalar endpoint pairs establish only their reached queued/grant/count/persistent-word and completed-cache comparisons. The selected 194 pair has 10 checks with zero failures and 1.3600000143051147 on both games (08c98c68). Historical AA6 all21/210 and first-six59 passes remain separate; Polyphemus values are 8.5/8.5/15/8.5 and final 143→165→Have5 is 2.2880001068115234 on both. None proves callback invocation, unrelated effects or whole gameplay. Historical 240 retains 511 rows, six matching words/RNG/health and 2,199 failures. | Actual initialized 510550/191 inserts TEMP count 1 or increments an existing record; an empty port vector does not establish Original query history. Unsafe AA6 completion prototypes retain their historical hold. The corrected query and scalar successors passed their six production checks. On unchanged item 194 inputs, actual Own records 118 completed-cache rows at 1.3600000143051147 with zero scalar or producer failures; RNG 2704386553→1036187744 and Card seed 3587811879 match independent Native observations. The separate validated same-input Original/Own endpoint pair adds 10 selected checks with zero failures and the same cached value. The immutable Own report still records its original five unknown classes; the new pair addresses only its cached-field comparison obligation. Actual Add/cache invocation, unrelated effects/controllers, rendered/audio parity and broader 194 initialization/held-effect coverage remain separate. Actual 191 invocation capture is UNRUN despite the frozen default-off hardware hook. |
| Controlled Knife and weapon precedence | Public 504 includes the controlled-Knife actor, retirement ordering and unavailable-query containment; public 528 adds independent source-context plumbing. SAFE's Own-only first-death check retains four failures across eight obligations because no registered target was observed; ordinary NPC retirement and continued ticks pass, and its Original counterpart is UNRUN. The sole NPC retires before 329/3; unchanged H8ec/6a source guards identify an existing permission/coverage gap, without an observed bad unlink or A/B regression. The actual Knife/Brim/Ludo triple case has a closed same-case Original authority (8927643d): 3,613 checks retain five strict failures (88ab2052), alongside 426 controlled-body rows, two HP drains and one death/retirement. Numeric FIFO/two 7-damage contacts/HP 8→1→−6 pass; unobserved owners in both games and unavailable pedestal/old-118-child history remain mandatory failures. Historical private H12 death endpoint obligations pass 8/8, while its full eight-case strict report retains 3,614 failures. Homing getter/caller roles remain private and inherit no callback completion; AA6's generic scalar completion prototype has a separately identified TEMP-history gap. | The new first homing case on old005de reaches a real3 grant, then fails at tick453 with an unavailable previous mask8 query owner. Source review identifies a retirement boundary also reachable through the new658 roles. The earlier eight-case 2,324 / 2,310 failure cohort remains preserved. | Historical Knife/Brimstone/Ludovico, FIFO/HP and satellite results retain their named artifacts and failures. Neither a source permission split nor an unchanged ordinary405 partition establishes positive controlled homing or full synergy parity. | Preserve the public actor safety boundaries and every retained full8 failure. Generic Flags32/Color64 and other cache read plans need reached source operands and independent histories before new getter/caller completion. Ordinary-Tear homing from private005de has unique module/registered-target ownership absent main; its old qualification remains separate integration debt. |
| Currencies, Golden Key and procedural pickup births | Public505/506 adds Card300/Pill70 Init and canonical chest/procedural consumers;509 owns held-pocket Save and519 adds ordinary collection, full-capacity spill and refused-contact physics. These share canonical held records, copied presentation and pickup owners; no separate pocket inventory is inferred. | Distinct-cell coverage retains20,387 checks/32 unresolved and same-cell193,049 selected passes/28 unknowns. Fresh H8ec separated pocket v2 all4cases checks292,059 with zero selected-law failures and28 mandatory unknowns only; each observes3 held changes,2 spills and3 requested sounds. Request/slot checks do not establish backend playing or audible PCM. | Same-input pocket v1 retains120,968 checks/3,279 failures:28 unknowns,4 unreached AD0 refreshes and later room/selector cascades. At declaredGP341 the retained Native position warps while Own exits its room; Native descriptor identity is unavailable. The same-GP130 initial Pill position gap remains despite matching held chronology/player motion; no passing old field became red in the4-case regression partition. | Magic 8 Ball's canonical Card world producer is public through 541 and reached-query scalar consumers through 546. Fresh Own producer checks match independent Native RNG/Card observations; 118 completed Own rows and a separate validated Original/Own pair establish the selected 1.3600000143051147 endpoint. The immutable Own report's five unknown labels remain preserved; paired cached-field evidence does not establish Add/cache invocations, unrelated effects/controllers, rendering/audio or broader initialization/held effects. Next pickup consumers are 251/252 world/held conversions and Battery90 through the existing charge owner. Equipped use/effects, capacity2, unseen AD0 refresh, complete contact/room physics, prices/reentry and unsupported child Init remain pending. Strict pocket v1 failures, separated v2 passes and historical basic coin/key/bomb/chest cohorts retain their separate scopes; the pocket11 bank supplies no basic pickup coverage. |
| Chest60, Bomb51, Mom's Key and modifier interactions | Chest60, Bomb51, Mom's Key and shared 203/250/selected-heart operators are public. Actual player/room inputs remain independent; unsupported input ownership is explicit. | Private658 corrected readers each perform 91,380 checks:203 retains 52 failures (48 mandatory unknowns, two Morph and two recorded-heart minima);250 retains 50 (48 unknowns and two Morph minima). Each fixed case matches three real hearts; observed subtype/property and requested-SFX slot laws pass. Old 1,751/1,747 observer failures remain retained. | The first actual Mom's Key Native/Own saved endpoint join on005de matches four initial five-word identities, initial/final RNG30 and final typed identities with zero selected mismatches or unmatched identities. Its prior own13 failures remain; no callback, parent/double-factory, Morph, clock or audio parity is inferred. | Remaining callback, rare-child, Morph and recorded-heart coverage failures stay open. Source SFX requests and matched endpoints do not establish played sound or individual factory/RNG callbacks. |
| Popup, pause, Quit and transition appearance | Popup, pause/menu clocks, Pause Quit and optional Move-save inputs are public; 526 adds loading art and 531 world-entry fade. The separate Options audio/display leaf has its source and bounded live scope above; Controls remains an independent increment. Corrected SFX f32 operands/opening-poll ordering, hurt blink and nightmare progress retain separate qualification. Complete Main Continue world/pickup restoration, actual ceremony/iframe/HUD observation, shaders and complete backend/PCM equality remain unresolved. Existing private Brim/Luck/initial-save evidence keeps its exact captured scope. | Eight private658 public-input Quit routes complete two pause entries, resume, outgoing/opaque fade, one supported Save/Stop handoff and incoming/released menu each. The full saved reader checks 28,764 comparisons:112 observer f32 mismatches resolve to zero selected failures, while all 56 mandatory unknowns remain. Original168 failures and complete report inverse are retained. | Earlier17acc pause routes retain 28,326 selected passes / 16 unknowns. Native lifecycle, framebuffer, Stop/mixer, binary persistence, Continue and callback owners remain independently unobserved in the new own-game Quit result. | Nonzero mutated-input Save, actual frontend pixels, ceremony/iframes/costume/control and Stop audio lifecycle require their own observations. Initial neutral Save cannot certify mutation or restoration. |
| Played sound and browser transport | Native backend/PCM ownership, browser archival tools and streamed identities are public. External492 adds actual looping ambience requests and frontend voice updates; these are separate from proving audible equality. | Three of eleven public-WASM Node pocket cases have completed: AL passes 9,349 functional checks with zero failures and 12 unavailable obligations, including unreached SAME-v1 AD0 refresh; separated fixed-phase0 and retained recorded-0 each pass 29,385 checks with zero failures and 11 unavailable (428c4301/c0022b06). Initial2 closed exit 0 with stable inputs and all owned children exited (212ea35d). Eight cases remain mandatory. DOM input, rendering/PNGs, audio backend and PCM remain unavailable; these pocket results supply no basic coin/key/bomb/chest coverage or sound equality. Historical browser resource partials retain their failed scopes. | Twelve earlier Native starts have actual table owners, accepted PLAYING, positive gain/global1 and nonzero complete PCM matching original samples, with zero selected failures. Latest browser AudioContext readiness is observed; source starts, configured gain, PCM, gameplay sound and played-sound equality remain unestablished. | A returned resume call is not an actual source start. Retain earlier NOT_REACHED/muted/ENOSYS/partial and guard reports. Stop/mixer/loop lifecycle, causal sound windows and full browser equality remain open. |
| Boss assets and scene selection | All 95 original boss UI sheets,56 portrait definitions and the independent source selector are public. Backdrops and bulk level/monster/character imports retain their separate ownership. | Original import inventory/PNG validation and named builds establish asset/type scopes. The current exact production build does not add selected boss-scene gameplay or pixel acceptance. | Original art and selector source do not establish VS animation, pivot, scene-clock or draw-order parity. | Declared player/boss/spot/name selection, transition timing and visible Original/browser witnesses remain required. |
| Contact regression tooling and source-only comparers | The authorized136 DST exception and existing live comparers remain separate. ControlledTechnology366 was resolved through two small public tools PRs513/514 and explicit old-branch supersession; required FIFO/HP/Impact laws and original failures remain preserved. | The existing Contact DST workload actually passes eight cases of1,500 steps and21 properties in Native rlib-only packaging. This is execution of the allowed harness, not an Original game, WASM or default shipped-build pass. | Saved comparer equivalence must retain strict cross-game identities and every missing callback/lifecycle obligation. Own observations cannot manufacture an Original tuple or convert endpoint agreement into callback evidence. | Actual saved708 comparer equivalence passes17,312 preservation checks including3,500 unchanged failures; the historical full four-property report remains349,856 checks/30,892 failures. No fresh335 positive contact or callback completion is inferred. No CI/hard gate or additional unit harness is introduced. |

All earlier strict failures, missing callback owners and resource-partial attempts remain historical with their named artifacts. The retained sections below are not fresh qualification of current main. Pill28/Range, actual191, 251/252, Battery90 and transition work retain their separate implementation/evidence scopes.

## Retained artifact records

The following source snapshot is frozen at main `80d65135`, including durable Homing and satellite case/tool PRs428/430 and Hurt fix429. The runtime reports below belong to named immutable artifacts; they do not qualify every later main change.

The private stat-input composite `0dca907d` / WASM `4a6ad3e0` (binding `0df24fd6`) passed normal production in 31.371s with 3,508 source pins stable. Its unchanged four-tape regular405 comparison against qualified `29c8dab6` retains 40,383,861 checks / 17,260 FAIL: 12,966 added fields and 4,294 key-set differences, with zero missing or changed existing values/types and zero errors. Floor and Clippy checks remain pending. Four Experimental Treatment240 own cases retain 3,859 / 88 FAIL, with no numeric/RNG/health endpoint failures. Three legacy12/14/222 cases retain 10,576 / 12 FAIL, with no numeric failures; these results do not establish individual RNG callback invocations.

The first fresh Experimental Treatment240 Original/port pair retains 26,031 checks / 2,199 FAIL over 511 paired rows. All six signed stat words, RNG words, health and selected cached stats agree. Cache completion remains wrong in 198 rows: Native pending0 versus port4294966523; both generic and typed failures are preserved. Counter/controller fields, identities, callback owners and the other Native cases remain unresolved. The new Damage240 stage-local admission fix is a separate source candidate, not a regrade of this report.

The earlier frozen `29c8dab6` qualifier passed production, but its raw405 comparison against P4 retains 38,099,273 / 1,453,039 FAIL, including material MT/grid changes. Its four Homing cases retain 24,554 / 42 FAIL. Ordinary six-case browser verification ended at a memory guard with only a partial first case; the other five remain ungraded. A corrected restart witness consumed all 5,802 literal commands, including 65 expected rejections, and passed documented outcomes/rollback, but did not reach a later floor or restart after descent: two coverage failures remain.

Historical private Homing4433 retains 24,181 / 42 FAIL across four own cases and 85,739 / 43,069 FAIL in its first Original pair. Historical satellite `ab2aa0a5` retains 38,027,481 / 0 regular regression differences, 12,478 / 232 FAIL across four own satellite cases, 2,470 / 108 FAIL across four retained-child histories, and 34,648 / 3,237 FAIL in its first Original pair. The separate typed callback consumer retains 726 / 31 FAIL. Zero exact identity joins, missing callback/cleanup owners and all raw failures remain explicit. Both games show zero chains at the first Add2 endpoint followed by two later chains; this is a consumer-boundary finding, not a port-only zero-chain bug. Durable [Homing cases](../references/isaac/cases/item-composition/spoon-bender3-v1.json) and [satellite cases](../references/isaac/cases/item-composition/controlled-knife-satellites-v1.json) preserve replay inputs and scoped evidence summaries.

Dedicated systems owners now cover pickups, transitions, played sound and pre-boss assets. Pickups includes coins, keys, bombs, hearts, chests and procedural spawns through the existing level descriptors; initial source slices restore coin sounds and golden-key lifecycle/spending/HUD. Transitions owns menu/world/pause fades and pickup text, sound, costume/control/iframe ceremony; the first source slice adds authored collectible title/description and the shared HUD clock. Sound comparison is being extended from PCM/transport to actual mixer/backend ownership and browser starts within declared public-input windows. The assets census decodes all 95 original boss UI sheets and56 portrait definitions, including the missing exact MegaGurdy portrait. These new runtime/loader/collector slices have no build or paired-game acceptance at this snapshot. Claude retains bulk level/monster/character imports. No CI or hard gate is introduced.

The following production4 and earlier records retain their historical artifact scopes and failures.

The production4 artifact (`d6db18cc` / WASM `e0137262`, binding `a43e576e`) passed normal production in 28.380s with all 3,498 source pins stable. Its full regular405 comparison against qualified main9a retains 38,020,341 checks / 2,856 FAIL: 1,428 added `player.entitySource` fields and 1,428 key-shape differences; changed or missing preexisting values and errors are zero. The item-stage core411 and separate copied-Parent414 are now merged. Their runtime/build/resource inputs match this combined artifact; the standalone intermediate core was not separately built or played. Later tool metadata is outside that build, and optional floor/Clippy checks remain pending.

Six public joint Technology 2 + Polyphemus captures and their finite reader retain 118,443 checks / 79 FAIL. No numeric operator field fails; 78 failures retain strict-join/callback unknowns and one reverse-case Remove-state endpoint coverage minimum remains. Four own contact cases retain 25,397 / 48 FAIL. Only recorded0/NPC18 exercises five complete copied Parent payloads, five FIFO appends, six HP drains, one death and one retirement; the three NPC14 cases retain missing-contact coverage failures.

The first fresh joint Original/browser pair retains 80,714 checks / 3,935 FAIL across 1,567 paired playing rows. All six selected cross-cache fields match across those rows; ordered Damage/Flags match across 32 observations and FireDelay across 1,170 on each side, while C70 ownership matches 1,254 observations. Both sides observe 686 completed joint-cache rows, five joint Tear births, two new born-0x200 C40 actors and 286 retained born-0/current-0x200 rows. Each side has seven C40 births, eleven Tears, 327 Full and 327 Fast body rows and seven NULL transitions. Zero exact identity joins leave all 51 C40 and 16 Tear body fields explicitly UNGRADED/FAIL; clocks, identities, callback and lifecycle owners remain unresolved. Stored retired endpoints differ (Native0/port7); no own-side material witness supplies a cross-game identity.

The first fresh contact Original/browser pair retains 94,894 checks / 9,184 FAIL across 1,216 paired rows. All 12 selected copied-source payload checks pass and retained C40 damage matches across 582 body rows. Both sides show six C40 identities, six HP drains, one death and one retirement. Native records six FIFO appends while the port records five; this observed gap remains, together with strict identity/RNG/clock/HP-geometry and callback failures. These selected contracts do not establish full combat parity or the other Original cases.

The explicit Native journal Zstd option412 preserves gzip as the default and leaves the gzip manifest format unchanged. Actual opt-in capture closed with 3,934 records (200,977,212 raw bytes / 394,681 encoded bytes); the first positive-playing projection completed with 1,258 tuples and the full contact comparison above. Node codec probes retain equal decoded bytes and six malformed-input rejections. This establishes the exercised transport scope, not game parity. Spoon Bender3 homing, neutral2/Knife capability and 4A0 satellite followups remain separate source-only work outside this P4 record. Historical P3, saved-history and corrected271 reports remain unchanged; the ten-case Number One replay remains pending.

The following entries retain their frozen 9a/P3 scope and are not rewritten by P4.

The then-latest validated main runtime is source `9a908a6d` / WASM `c720ec0c` (binding `2d6fbb4d`): normal production passed in 29.307s; optional floor and Clippy checks remain pending. Its regular405 comparison against production2 retains 38,005,081 checks / 11,616 FAIL: 5,808 new fields and 5,808 key-shape differences, with zero changed or missing preexisting values. Later tool-only changes, including the corrected Tear identity comparer, are outside this runtime artifact. This source snapshot retains separate Technology 2 and Polyphemus owners; their future joint pipeline is outside this frozen qualification record; no joint/Parent acceptance is inherited.

The separate immutable production3 artifact (`b25a85e7` / WASM `9d30105b`, binding `11f8e079`) has three completed first Original/browser references. Technology 2 alone retains 34,748 checks / 1,964 FAIL: both own histories show one grant, four C40 births, seven primary Tears, 128 Full and 128 Fast stored-body rows, and four C40 NULL transitions. Selected typed material/property checks pass; strict identities, clocks and missing callback owners remain unresolved. Stored phase summaries do not establish callback entry, and zero exact-identity body joins do not establish actor correspondence. Polyphemus retains 84,313 / 5,378 FAIL: both sides show two grants, two pre-mixed plus ten mixed births, two actual NPC217 receivers, two registry appends, remaining-damage changes, copied FIFO appends and HP endpoints, including one same-Tear/two-receiver registry witness. Native Fast classification remains unknown. The starting Knife + Brimstone + Ludovico pair retains 67,254 / 5,206 FAIL: both sides show one controlled Knife across 508 body rows, two copied FIFO appends/Get4 state diagnostics/HP drains, one negative-HP death and one manager retirement. Strict identities, clocks, actor/callback ownership and lifecycle gaps remain failed. The nullable-C18 tool correction is source-only and has not regraded that report.

Separate browser-only Technology 2 contact coverage is 25,378 checks / 45 FAIL across four cases. Only recorded0 (NPC18) shows five flags8-compatible FIFO appends/retained48C diagnostics, six HP drains, one death and one retirement. The other three NPC14 cases have no contact witness and retain 12 failures each. No Original contact comparison ran. The corrected held-case reader covers only two of eight cases (10 FAIL each); the other six have not been regraded, and the older eight-case report is preserved.

A saved real-pedestal 114→118→329 history replay adds 1,535 checks / 36 FAIL: each side observes eight preexisting and eight continued children, 24 Full and 32 Fast stored-body rows. Native-only event input observes eight Remove post-store and eight Uninit-entry events; browser events remain unknown. Missing birth-copy, current-Get4 callback, dispatch, Uninit completion/weak-link, child-contact and raw-Line owners remain explicit, together with unexercised per-child obligations. The prior 106,884 / 8,755 FAIL plus one history failure is unchanged; this additive own-history check does not establish whole-synergy parity.

The corrected Tear identity comparer was exercised on one saved Polyphemus pair (1,295 playing rows), not the pending ten-case Number One suite. It reports 7,263 checks / 1,319 FAIL with zero exact cross-game joins: all 664 Native and 656 browser manager rows remain unmatched, with 7,304 and 7,216 material-field obligations explicitly UNGRADED. The former comparer reports 13,167 / 2,146 FAIL on the same saved pair. The lower failure count is a reporting/association correction, not improved parity; no ordinal, time or geometry fit supplies a join.

The future joint Technology 2 + Polyphemus typed pipeline and actual Parent-owner slice is outside this frozen qualification record; no joint/Parent acceptance is inherited. Independent item results do not qualify joint admission. These entries consume compact completed reports only; they do not rerun games or rewrite historical failures.

The following production2 and earlier snapshots retain their original artifact scopes and failures.

Snapshot `48b9f4ef` includes Epic cache/grant382 and actor/neutral lifecycle384, Monstro queued4 correction386, Soy full-Add clear387, Monstro head renderer388, Impact birth metadata389, ordinary Inner Eye390, copied damage-source payloads391, selected Epic startup392, and blast/camera shake393. Corrected Monstro comparison tools362 are merged; the original comparer and old results remain preserved. External stage-transition374 and boss-award poop fix385 are retained. These small source integrations do not qualify the entire current main tree against a frozen private artifact.

Separate production2 source3ec42d94/WASMabdb286e passed normal production; floor and Clippy remain pending. Its raw405 comparison against f1 records37,987,905 checks/8,728 FAIL:4,364 added-key-shape and4,364 new GameShake/camera-history/C70 fields, with zero changed or missing preexisting values. All raw failures remain visible. Ordinary Inner Eye has eight completed own cases,198 typed births and66 complete three-Tear cohorts,84,879 checks/72 FAIL. The finite reader hit its final32MiB output quota after writing the complete report; a separate successful opaque component verified all report bytes without rereading State spools. No Native ordinary2 or callback/RNG-draw parity is established.

Both four-case Epic contact cohorts (70 and132.815673828125 damage) record6,663 checks/56 FAIL. Every case observes one copied-Rocket FIFO append, ordered HP drain, negative-HP death and manager retirement; missing callback/argument/RNG/lifecycle event owners remain failures. The first high-damage Original/port pair retains40,872 checks/4,064 FAIL over476 paired rows and261 typed body rows. Both sides show one target, three rockets, one triple cohort, three explosions/record appends and one FIFO-to-retirement state chain. Selected properties match across261 body rows and receiver HP matches377 observations; receiver geometry, exact identities/RNG/Parent/records/counters and missing owners remain failed. Zero observed transient crater18 births do not prove absent callbacks.

Four own shake cases retain867 checks/20 FAIL. The first strict pair observes58/56 positive-duration and64/61 nonzero-offset rows on Native/port; duration62/476, history430/476 and cameraStyle/sharedMT476/476 checks still fail. The port lacks an explicit cameraStyle owner, and matching endpoint MT states would not certify individual draws.

The production2 Monstro queued4 replay reuses the same5149 Original tuples at exactly62,795 checks: failures improve5,346 to2,520. Only three fields change, each942 to0 (historical controller diagnostic, C18 knownNULL and unadmitted controller); all field counts and all other failures are unchanged. Both games retain four complete13-Tear cohorts and three mixed cohorts. Native held/phase owners remain null with mandatory failures; every earlier frozen-f1/e2bb report is preserved.

The retained 48b9 audit had five open PRs:253/271/292/329/366. Unique253/271/292/329 remain preserved. Stale361 was closed only after preserving its complete source/commit metadata and landing the qualifying cache/actor successors382/384; corrected362 was updated and merged rather than discarded. No further duplicate is proved.

The frozen f1/b957 product passed normal production; floor/Clippy remain pending.
Its regular405 comparison retains37,979,876 checks/14,416 raw failures from
recorded key removals/additions, with no changed prior values; typed ledgers do
not waive those failures or qualify the whole current source tree.

First controlledDr46,040/3,508FAIL now observes three births/246 body rows/two
property cohorts in both games, including prior-cache10.5 on the Add4 frame and
later19.92235. All four port-own cases show three births/two cohorts/three C18
cleanup observations, with ten unknown/boundary/event failures each. Monstro's
corrected same-tuple62,795/5,346FAIL observes52 births/four13-cohorts/three mixed
cohorts in both games; all four port-own cases show three mixed13-cohorts.
Native held input/phase remain null with required failures; this is a tool-owner
correction, not a new gameplay result. Old64,147/6,043FAIL remains unchanged.
First Epic92,164/5,207FAIL observes three targets/nine rockets/three triple
cohorts/nine explosions/nine record appends in both games, withselected properties matching across786
body rows. Zero observed crater18 births do not exclude transient
callbacks. Empty-room evidence supplies no NPC-contact proof; the later contact
fixture stopped at the shared Challenge-entry frontier before gameplay.

The frozen-f1 Epic results retain their original artifact scope; the later main actor integration384 is recorded above.
First controlledTechnology117,717/13,575FAIL, mixed17,056/1,558FAIL and Soy's
visual/fullAdd frontiers retain their own scopes. See
[item composition coverage](item-composition-coverage.md) for source owners,
remaining cases and historical reports. These results do not establish complete
game, render, contact, identity or RNG parity.

## Current live-game testing policy

Checked-in Rust, Node and Python unit suites were removed at the user’s request.
The sole, owner-authorized exception is the [Antithesis DST harness](dst.md) in
`engine/dst/`: an inner-loop regression tool for engine safety and determinism,
never original-game evidence or a merge gate.
The live suite runner is [run.mjs](../tools/e2e/run.mjs), which executes
every registered live original/browser suite, retaining all results even if an
earlier suite fails. The [paired gameplay runner](../tools/foundation/paired-game.mjs) gives
fresh original processes/profiles and an immutable production browser receive
identical public input/presentation tails across contrived and recorded
OS-random scenarios. Source-observable title/fader clocks, seed readback,
player motion, pause/controller state, red HP/immunity and active tear/bomb
admission are functional checks. Reports
retain complete original PNGs, consumed X11 phases/focus, DOM traces and
strict pixel differences. Full-frame equality remains an optional diagnostic;
its failures are preserved and do not become functional passing results.

The [paired tape registry](../tools/original/foundation-paired-scenarios.py)
retains the native cutscene prelude and explicitly aligns browser reset at the
independently observed first Title frame0/f32 fade .975 boundary. Cutscene,
complete original-state parity, general actor integration and full gameplay
remain incomplete. Saved `case.json` files replay both seed and randomized
controls with `--replay-cases`; seed-only replay generates new controls.

The Wave1/Wave2 snapshots below are historical results from the earlier test
policy. Their removed unit/integrity suites are not current executable gates.
Consult the latest paired report for current implementation outcomes.

## Retained Monstro BloodStart source leaf

The new partial producer routes ordinary Monstro’s real BloodStart random branches through Effect17, SFX72 and Effect42 in their original order. It preserves the actual authored NullId0 guide, separate cached spawner/link fields, signed scales and layer flips, and distinct active/dead lifecycle. The required `test:e2e:blood-start` gate retains contrived978 and recorded OSBLL routes with both input phases; it requires actual Death-interval execution and matched bodies. Private all-target Rust check and strictClippy pass at `d28154f`; production/WASM and fresh game qualification remain pending. Explosion2/77/category5 remains an explicit runtime error before any final reward, and existing native-prefix failures remain blocking. The final-heart/delayed-clear/Grid17 owners are separate; no boss-clear or stage-completion claim follows from this leaf.

## Retained 3df99031 source and actual-game qualification

Current main `3df99031` includes ordinary Player Brimstone299, per-field comparer
continuation311, the FireDelay descriptor304, optional regular Tear variant298
and standalone Ludovico Tear307. The existing ordinary/Has118/controlled Knife
and Number One/Libra owners remain in main. Ludovico's final C421 artifact has
successful owned builds and retained regressions; three routes match selected
motion fields, while the complete four-case comparison still fails. The exact
source landing preserves external renderer326; no new full merged-tree build
or whole-game parity follows from that merge.

PR299's OWN production, strict Clippy and floor checks pass. Four retained
405-command public-WASM routes compare 37,958,272 preexisting fields with zero
differences. The retained Original ordinary case remains **22,111 checks / 1,030
failures**. It contains two births and 61 Laser/Impact rows (30 Full, 31 Fast);
observed geometry, phase, TTL/fade, Sprite and color fields match. Two
following-Full retirements observe Native absence, port unscheduled/uninitialized
and actual C18 NULL. They do not observe Native Remove/Uninit callback events.
Required HP/maxHP owners, identities/seeds/RNG, prelude timers and diagnostics
remain red; typed contacts, DOM/render and the full shared stream remain open.

The same producer's retained118→114→329 comparison remains **88,941 / 9,079 FAIL**;
its unchanged-v2 history remains **19,838 / 774 FAIL**, with 36 gaps. Actual grants
and primary changes match, as do counts, queue, cache, C0C and later
pendingPrimaryCleanup None across 1,010 rows. There are 470 parent, 614 child and
51 early Laser/Impact rows. Later None values do not prove a captured cleanup
transient or callback. These results leave all older Brim10, private v7/v8 and
Knife reports unchanged.

Merged PR304 keeps FireDelay request2, effect/proven-neutral/context and its
literal ordered operator together. Merged PR298 exposes regular Tear
sourceVariant as optional: Some0 requires the closed normal-Isaac, current
primary1, flags0 and positive1/4/6 domain. Other contexts remain unknown; this
regular-kind1 proof does not admit kind8 or establish Native variant parity.

Merged 307's final `d234fd9e` / WASM `c421a00a` passes production, strict Clippy
and floor checks, plus **37,965,608 / 0** retained regular-field comparisons.
Its four retained Original/public-WASM cases remain **134,364 checks / 20,813
FAIL**, with 1,541 controlled bodies (770 Full,771 Fast). The fixed motion case
and both recorded OS cases match position, velocity, B28 movement delta and
saved position450 across **1,157 body rows**. This qualifies the source-confirmed
saved-position rewind correction, without accepting identity/seed/RNG or missing
actor fields.

The contact case still differs on 259 position, 264 velocity, 255 B28 and258
saved-position rows among 384 bodies. Its first P/V mismatch is at presentation 686;
observed Native FIFO begins at 821. Native type 18 has FIFO observations at 821/822
and 825/826, then a valid nonlethal HP application at 823. The port has **no actual
FIFO or drain observations** and fails its required minimum enqueue/drain
witnesses. Older supported-minimum counters are not port contact evidence.
Receiver geometry, strict source identities, registry/atomic velocity/veto,
lethal lifecycle and unknown owners remain gaps or failures. The report-only
classification establishes no new timing, steering or actor-law correction.

Earlier producers remain separate. Original `d1e58278` / WASM `d9c35ac7` retains
**85,050 / 11,620 FAIL** with zero port Tears because its guard required C98=0
while both games observed 3. Intermediate `7e000d3f` / WASM `8120d8e9` retains
**134,350 / 24,248 FAIL**, including 42 strict contact HP and 10 queue differences;
its independent queue/HP chains are not final C421 contact coverage. The prior
**37,966,740 / 2,264 FAIL** regular report and OS0 guard-stop remain preserved.
Two retained reciprocal Libra histories pass **503,611 / 0**, with 631 gates and
93 post-grant rows each; the first still lacks 631 Native Familiar observations.
Falling speed, prelude timers, missing registry/ordinal and complete
actor/contact/lifecycle or DOM parity remain unaccepted.

Draft314 declares four ordinary-Brim contact cases, and Draft316 owns the
controlled-Brim property/priority marker. Merged315 supplies typed14/18 contact
observations; merged317 supplies optional kind8 Room2494/born83C/velocity-delta
fields. Those tools are consumed by final C421, with strict contact and
observation gaps retained. Merged318 keeps the unchanged Damage operator beside
its footprint and is consumed by the independently built708 producer, whose
scoped builds and retained regressions pass as recorded below. Runtime312's
controlled Brimstone remains a separate Draft. Its historical incomplete-prefix
failures remain; four fresh Native transports complete, while finished paired
case0 grade86,999/7,688 FAIL is complete; other three grades are pending. Missing actor fields, FIFO
attribution and lifecycle events stay failures. The [current compact record](../references/isaac/foundation/items/post-brimstone-299-and-ludo-source.json)
keeps exact report and source pins; the historical checkpoints below retain
their original grades.

The weapon/stat foundation in [PR203](https://github.com/Anveio/isaac-rebirth-browser/pull/203)
landed at `631e678`. Requested primary selection runs before scalar callbacks;
the retained primary, attack flags and independent eleven-component tear/laser
colors have explicit cache owners. Regular tears consume the resolved birth
properties. Unmapped stages and controller requests stay observable, and a
missed registered-actor cleanup cannot disappear on a later equal-primary
evaluation. The PR203 foundation checkpoint itself did not execute nonregular actors;
the later current-main actor owners are recorded above and below.

Exact `60ff78e3` production, strict Clippy and floor builds passed. Its public-ABI
headless differential replayed 405 recorded commands and compared 406 snapshots,
6,809,577 fields, with zero differences against the older `ef876` producer.
The [qualification receipt](../references/isaac/foundation/items/weapon-cache-foundation-203.json)
keeps that local WASM check separate from original-game evidence. The scoped EF8
regular-actor browser replay retained 231,387 comparisons and 24 failures per
browser, with zero new or lost comparisons. The item257 transformed-tear
failures and native actor-seed differences remain unresolved; these reports
do not qualify all current-main gameplay.

[PR214](https://github.com/Anveio/isaac-rebirth-browser/pull/214) adds the public
Main/Challenge route for declared neutral Challenge1 XML, including conditional
curse sampling and source pre-grant Save followed by ordered grants.
[PR216](https://github.com/Anveio/isaac-rebirth-browser/pull/216) leaves startup
Doors initialized until the eligible Full room callback opens them. The retained
original case16 replay on `76d7a01b`/`32b5bcfd` passes all 8,368 required frontdoor
checks, including 631 Door rows; its 631 attack-body rows remain unqualified.
The older d156 report retains all 188 early-opening failures. Native Saved data
and dynamic Save/grant call order were not observed by that scoped replay.

The separate ordinary Brimstone producer `866d2c8`/artifact `fbb48969` completed
ten fixed and recorded-OS cases: 307,112 checks, 23,017 differences and ten
functional failures. All native processes exited zero and captures completed
without errors. The observed 669 Laser and 669 Impact body rows include both
phases; 16 transient Full-Uninit slot-null witnesses were recorded. All ten
frozen strict audits failed while reconstructing manifest bytes, before
later clock/X11/options/resource validation. The separate additive audit passes
all ten cases: only the permitted adjacent root-key reorder, an exact complete
manifest SHA and no nested observation/input difference. Separate executions
of every remaining unchanged clock/X11/options/resource validation also pass all
ten with protected sources, archives and fixtures unchanged. These addenda
establish scoped capture validity; they leave the archived strict and gameplay
grades unchanged. [PR217](https://github.com/Anveio/isaac-rebirth-browser/pull/217) fixes
future capture serialization and does not regrade those archived audits.
[Compact evidence](../references/isaac/foundation/items/public-ingress-and-brim10.json)
retains the exact producer/report scopes and unresolved fields.

The shared cache evaluator now includes known scalar completion (PR233),
negative Flight (PR236), Familiar after Flight (PR239), typed Luck (PR254),
the final ALL tail (PR260), and per-stage effect/proven-neutral item footprints
(PR264).
PR269 replaces config-request inference with explicit callback capabilities.
Each cache leaf passed its own production, strict Clippy and floor checks.
The exact same-base PR269 public-WASM regression preserves 37,914,507 complete
field comparisons and 416 positive-owned completion rows with zero differences.
This is local regression evidence, not new original/browser acceptance.

Merged [PR291](https://github.com/Anveio/isaac-rebirth-browser/pull/291) adds
Libra304's coupled request expansion after primary selection and its
post-callback balance before final clamps. Unsupported input callbacks keep the
coupled outputs pending. Its own production, strict Clippy and floor checks pass;
the local regression retains 37,956,648 fields with zero differences. Two fixed
public-pedestal histories, 304→6 and 6→304, pass 503,611 scoped Native/public-WASM
checks across 1,262 paired gates, including 186 post-grant rows. The first case
lacks 631 Native Familiar-manager selections; its pending bit 512 is observed as 0,
while the second case's actual empty manager matches. Actor identity, source
variant/current color, physical/lifetime and browser/render parity remain open.
Duplicates, partial requests, active clamps and item118 triples remain unexecuted.
The Libra leaf landed at `3080e8e2`; current main retains its four owned blobs.
This does not claim
a separately built whole-current-main artifact. Earlier evidence capsules and
failed attempts remain unchanged.

Merged PR262 carries authored whole Special STB resources through real startup
and doorway factories. Its own production, strict Clippy and floor checks pass,
as do six public-WASM startup/invalid-resource checks. These do not qualify
Original/DOM equivalence or full live item history. PR266 registers the public
item-composition driver and all four literal v7 replay cases.

At the retained main4e122 checkpoint, ordinary Brimstone and Knife actor
execution remained in private producers. The later PR270 result is recorded below.
At that checkpoint, Number One6 and Libra304 had prepared source operators and
scoped admission without inherited runtime or gameplay acceptance. The ordinary Knife six-case
bank completed only cases 0..2 on `2ed8a7b`: 571,311 checks, 37,820 differences,
three functional failures and passing capture audits. Cases 3..5 remain pending.
The later `a3209046` producer passed its own compiler checks; its older DST run
still fails four of 109 properties. Main's NaN-equality source correction does
not retroactively regrade that execution.

The a320 v7 history observed item114 at presentation917 and item329 at1067 in
both games, with 280 controlled bodies. Its archived body113,707/9,052 and
helper 18,607/701/3-gap results remain unchanged. The native history retains
6,274 checks and one missing preexisting-child witness; its separate narrow
comparer replay has 6,263 checks and the same gap. Five selected browser expiry
rows do not regrade the full paired report. The older v5 partial comparison
retains 17,123 differences and its separate 970-row Game-flag correction.

The newer private `90ea8ed2` producer ran the identical retained native v7
inputs. Capture and transport pass with protected inputs unchanged. C0C matches
all 970 observations; each of the 430 primary Knife physical/saved positions and
raw80C/810/7F8/814 owners matches, including 280 controlled rows. There remain 35
rotation differences, and the full body comparison fails 6,822 of 113,714 checks.
Its history has 18,614 checks, 700 failures and two missing preexisting-child
gaps, one in each game. This is a new result, not an a320 regrade or PR270
qualification.

Complete shared-MT/actor parity remains red: 472,992 of 631,488 word comparisons,
970 of 1,012 cursor checks and 970 actor rosters differ. At the first frontier,
all 624 words match while cursors 490/463 and five ambient actors differ. Missing
field/owner errors are zero; these are real simulation differences. Ambient
consumers, actor identities/seeds, child motion, paired Original/browser child
continuation and cleanup through item329, live selector-change cleanup and
whole-main gameplay remain unaccepted. The
[compact evidence](../references/isaac/foundation/items/public-ingress-and-brim10.json)
adds the exact 262/269 and 90ea scopes while preserving the earlier ca63 snapshot,
first32, Brim10 and every historical failure.

The separately declared v8 route has one completed Original-only fixed-phase
capture. Strict transport passes with unchanged protected inputs; grants occur
at 917 and 1107, and an actual preexisting child before 329 is now observed. The
native helper still fails 18 of 6,744 checks: two same-child full-curve movement
gaps, eight Remove-flag gaps and eight Remove-before-full-cleanup gaps. Six
children have actual full-curve movement/copied-line witnesses; two have only
post-conversion fast rows before later Full manager absence. Actual Remove
flags remain unobserved for all eight. No DOM/browser v8 capture or whole-history
acceptance follows; all four declared cases and all older grades remain retained.

At main `8ac96e2f` (PR270), the ordinary parent implementation is merged.
PR270 adds the ordinary Mom's Knife subtype0 parent to main, including its
registered slots, launch/return and Full/fast callbacks. At that checkpoint,
Has118, Brimstone and Ludovico execution were separate private/source leaves. Own
production, strict Clippy and floor checks pass; the four-route local WASM
regression retains 37,928,004 field comparisons with zero differences. The
existing authorized DST consumer also compiles; no full DST run is claimed.

The recorded Original114 case retains its frozen 125,788-check/8,922-failure
grade. A separate overlapping additive report has 87,879 checks and 21,918
failures; these totals are not summed or substituted. Each of 44 parent fields
matches all 1,278 rows (639 Full/639 fast). C0C matches 1,466 rows, with 157
observed charge increases, 104 launched rows, two release resets and 12 declared
render witnesses. Four strict-f32 rotation differences, actor identity/seed,
RNG/Init uniform, missing raw8A4/8A0/Line fields, prebirth C74/BB0 counters and
NPC/contact/clear differences remain red. Child/controlled/triple histories and
whole-case parity remain unaccepted. [Parent evidence](../references/isaac/foundation/items/ordinary-knife-parent-main.json)
preserves the two report scopes and all earlier grades.

At main `e0fd5e9f`, PR282 adds Number One6's scoped scalar operators. Own
production, strict Clippy and floor checks pass; the same-input four-route local
WASM regression compares 37,956,648 fields with zero differences. The real
pedestal witness matches all seven numeric fields across 632 paired rows,
including 93 after acquisition. This is committed-prefix evidence: the durable
8-action, 361-presentation, 698-command tape completed, but final capture/archive
creation failed. Those 93 post-grant browser rows retain pending color cache64
against Native0. At that checkpoint, appearance/auxiliary PR288 and consumed-cache readiness
PR287 were source-only; later scoped PR288 evidence is recorded below.

Merged PR283 adds an optional Native event tap. Its separate actual capture
observes 24 lifecycle events for 12 identities, including eight post329 child
identities. This establishes Native Remove/Uninit observability, without port
or paired triple parity. The old Original-only v8 helper remains 6,744/18/18;
its archived missing-event grades are unchanged. These captures predate the
later merged Has118, controlled Knife and ordinary Brimstone runtime leaves.
[Current scoped evidence](../references/isaac/foundation/items/cache-and-native-lifecycle-main.json)
retains the exact proof pins, incomplete capture status and remaining boundaries.

At main `383d96eb` (PR288, retained through formatting-only PR293), Number One6
has scoped appearance, auxiliary and public-starting-grant code in main. Its own
three builds and 37,956,648-field local regression pass. Retained Native versus
independently executed public-WASM properties match17,423/0 at631 unique gates,
including93 after grant with pendingcache0. Each side's grant-gate Tear remains
white while five later own Tears copy changedCA0. This does not join cross-side
actor IDs or qualify port currentColor2D0, sourceVariant, physics, lifecycle or
candidate DOM/rendering. The older scalar-only93 cache64 differences and failed
incomplete archive remain unchanged.

The separate completed unchanged-v8 public-WASM replay retains 19,359/5,394 FAIL
and old-v2 history 18,759/2,307 FAIL. Native has12 children versus0 in the port;
the118-first transition to114 leaves typed primary cleanup unresolved. DOM/browser
capture remains unexecuted, separate from earlier pregame guard stops.

The retained PR297 snapshot describes Draft286 executing the Knife-first
114→118→329 route through public WASM. Its functional comparison fails11,158 of106,884 checks;12 child
births pair at actual gates and copied controls match, while all550 pre329 child
position/velocity/angle rows and532 raw8A4 rows differ. A separate84/0 birth-law
diagnostic uses each side's own actual8A4/8A8. The source consumes a shared-MT
uniform for8A8; the parent seed/uniform already differs before child birth.
These first-birth witnesses do not demonstrate a new child-law defect, nor close
later movement/nearest-parameter or random-stream alignment. Native24 lifecycle
events/12 children,280 post329 controlled-parent differences and the new marker's
missing helper remain separately scoped. That snapshot establishes no Has118/controlled/Brim whole-game parity;
later merged source and scoped results are recorded below.

Subsequently, PR290 merged at `ce5033cf` and PR286 at `98b35534`.
Main now uses one canonical NPC Laser record/store, shared Line layout and
optional unknown fields, and executes ordinary Has118 subtype1 children through
the existing Knife store. PR290 preserves 17 NPC callbacks and compares
15,268,885 existing-State fields with zero differences; its three own builds
pass. The scoped replay observes 43/86 Full rows and zero fast rows, retaining
fast nonvacuity failures. Original comparisons remain 2,820/21,942 and
2,831/23,070 differences with zero numeric Laser joins and no DOM acceptance.

PR286's three own builds and 37,956,648-field local regular regression pass.
The fixed route observes 12 same-gate child births and 614 body rows with matching
copied Line, flags, damage and color; its universal 106,884/11,158 FAIL and the
older v8 19,359/5,394 FAIL remain unchanged. The inherited shared-MT divergence
already precedes child birth; later physical/parameter and controlled behavior
remain unaccepted. Both recorded-OS attempts stopped at resource guards; the
second reaches 610/748 presentations with an unfinished tape and full transport
FAIL. No new DOM, history-helper, complete lifetime or triple parity follows.
The PR297 evidence capsule remains an unchanged historical snapshot.

PR301 subsequently merged at `78dcabee`, implementing controlled Knife on the
same store, allocator and registered C18 as ordinary/Has118 Knife. Its own
production, strict Clippy and floor checks pass; four public-WASM routes retain
37,956,648 preexisting fields with zero differences. The 1,624 new D9C observations
are additive type checks, not baseline numeric parity. The fixed Knife-first
comparison remains 106,884/8,755 FAIL plus one missing old history marker.
In 281 post329 observations (141 Full/140 fast), parent physical/saved position,
velocity, DA0 and C18 retention match; 280 rows carry bit33. All 12 child births and
614 child bodies remain, 64 after329. The current report retains 37 strict parent
rotation differences (34 post329), 136 pre329 DA0 differences and inherited
identity/seed/Init8A8, shared-MT and child-geometry failures. Snapshot observations
leave short Remove/Uninit events unqualified. No DOM/render, full624, contact/lifetime
or whole triple parity follows. Earlier grades and OS prefixes remain unchanged;
ordinary Player Brimstone299 is now merged with the scoped results above;
controlled Tear307 is also merged; its final retained four-case comparison remains
134,364 checks / 20,813 failures, with contact and observation gaps above.

Controlled Brimstone312 is merged atf4668ae9. Its708 producer passes three owned
builds,37,968,484 strict regular fields and503,611 retained Libra properties.
Four complete Original/public pairs retain349,856 checks/30,892 FAIL over4,058
playing rows,2,192 Laser C18 bodies and1,110 Knife bodies. All2,188 stable Laser
history rows match material fields; geometry, identity/RNG, HP ownership and
actual callbacks remain unresolved. Historical incomplete-prefix86,718/7,667
and87,458/7,667 failures remain separate. The failed full comparer established
no grade; the later bounded paged successor closes all four comparisons.

Portable lifecycle345 is merged at04e52bbf. Four saved-sidecar addenda each retain
19 checks/4 failures, two actual Laser boundary events and one separately observed
following-Full cleanup chain. Remove-post differs from BaseUninit-entry;
completion, live reapply, initial Full/C18 order and paired port callbacks remain
missing. These addenda do not rewrite the functional predecessors.

Technology320 is merged at85962135 with the source-owned post-contact reset and
Eye-history repair. New027c's four-reference comparison retains82,351/5,731 FAIL;
all152 damage checks match, resolving exactly76 old damage failures. Other
per-field counts are identical to the historical77cd82,351/5,807 report, which
remains unchanged. The027c three owned builds pass. Raw405 comparisons still fail:
37,972,668 checks with5,928 differences versus77cd and8,368 versus708, entirely
5,512 optionalRoom key/null additions and reviewed Eye fields. A separate actual
public-ABI history ledger has zero unknown-history issues; raw reports stay red.
Merged348 separates cache/kind2/kind3-contact capabilities without changing their
current[1,4,68] domains. Identity/RNG, geometry, HP/contact/render/lifecycle gaps
remain open. No whole-current-main product or parity follows.

Merged332 now runs in four complete Technology and four controlled-Brim Native
captures. Its original69-window raw/gzip equivalence and36,974,592-byte validation
RSS retain their scope. Current recorder highwater36,585,472B is a measurement,
not a historical peak-reduction claim. Same memory floors, output caps and exact
owned-game pidfd closure remain in force.

Gimpy's old bank still misses item225 and Dip death. Its separate v2 own bank
observes225 grant, lethal Dip and bounded100 failed-roll transition consistency
in all four routes. Atomic callback, Original, successful proc and hurt remain
pending. Room8 requires the unmerged329 kill owner, not an isolated DTO addition.

Dr.Fetus343 remains Draft. Test producer07211886/26cb passes production45.211s,
Clippy29.475s and floor17.465s with all3,451 pins stable; strict four-route405
regression against027c passes37,972,668/0 without exclusions or additive fields.
All four public routes execute three shot births/retirements each, birth fuse30,
fullUpdates0, damage10.5 and actual ParentPlayer0. Each has two Bomb-source FIFO
rows and a following-Full HP4 to-6.5 witness; this is own-side nonvacuity only.
The first fresh Original fixed0 capture closes normally and its strict paired
grade retains28,783/2,740 FAIL. The remaining three Native captures are pending;
strict cross-game body join coverage, identity/RNG, counters and actual callbacks stay failures.
A saved-input Dr counter-v2 addendum retains29,598/2,740 FAIL and adds815 matching
parity checks without regrading absolute counters. Independent Native birth,
retirement, Bomb FIFO and following-Full HP witnesses are separate from strict
cross-game identity failures. Soy330's four public routes complete on26cb; the
regular routes observe16 Tear births each, while118+330 has zero Laser births
and360 costume-transform frontier rows per case, retaining coverageFAIL. Soy
Original comparison is pending.
Controlled Technology68+329 and the Dridle DA0 fix remain source-only work.
Merged314/325/316 preserve default declaration bytes; ordinary-contact and335
controlled-ring contact cases remain unexecuted at this snapshot.

Readiness287 passes a warm production retry and strict Clippy, but its floor
build stopped at the working-memory floor. Its earlier production capacity stop
is also retained; public-game contracts remain unexecuted. Pending capture-tool
PR292 preserves an existing nonnull trace's34,042,951 bytes,1,532 commands,
216 retained states and3 backend events, including disclosed state-history
truncation. Its live attempts remain incomplete; no engine parity follows.
[New scoped evidence](../references/isaac/foundation/items/post-number-one-and-knife-first.json)
keeps these results separate from every earlier report.

Main `19906c4e`, observed on October14, includes the reviewed normal-health source callbacks, the source-only Nightmare resource prerequisite and raw snapshot accessor, and retains39 registered live suites. This is source qualification. No complete current-main39-suite production/original/browser acceptance is claimed; the frozen local f793/0f343 checkpoint and later scoped owner artifacts remain separate.

The integrated Death111 producer at `5afa7ddb` and artifact `c4685585` closed all four fresh original/browser pairs: 2,962 phases, 52,190 scoped checks and 4,226 required failures, with zero capture errors. Both contrived death/popup routes executed in both games; both unfiltered OS inputs remained alive and are retained. The full eight compressed native/browser streams were independently read back and hash-verified. This proves those actual executions and preserves the unresolved age, motion and menu fields; it does not establish complete death or whole-main parity. The earlier1cc interruption below remains a distinct historical attempt.

The separate PR104 moving-bank continuation closed all four selected identities on its frozen1e9/4e0 artifact, retaining its earlier failed prefix and interrupted second attempt. It remains a failure with8,242 scoped checks and103 required differences. Native variant5 particles were observed, including cached244 spawners, but no captured native death requirement or matched body/controller comparison passed. Generic presence does not establish the lethal producer or body parity. Source-only later PR104 composition and the new partial BloodStart126 producer inherit none of that build/live qualification.

## Retained main36 and interrupted-death checkpoint

Main `7492adba` registered36 required live suites. This is a source
checkpoint, with separate cached effect spawner words and an actual null linked
parent for the generic NPC Poof callers. The corrected stat-cache cohort retains
158 required failures over212,485 checks; its earlier609-failure capture remains
separate. The last complete consolidated production build is still f793/0f343,
described below. [Current source and interrupted death scope](../references/isaac/foundation/runtime-progress/history/20261014-main36-source-and-death-abort.json)
records these boundaries without claiming current-main live acceptance.

The root normal-death draft captured two complete failing aggregate cases at
source `1cc127e8` and artifact d0a6: restart733/740 failed checks and exit734/741.
Its third actual native/browser segment reached capture but comparison stopped
with ENOSPC; the fourth selected public OS input was unattempted. Both OS draws
and all raw partial evidence are retained. The supervisor is terminal failure;
the immutable interrupted nested paired report still says running. Later
graphics/streaming/menu-owner source fixes are unbuilt, and no completed
four-case death acceptance or main death integration is claimed.

The latest completed six-suite run is commit `5b4901f`, immutable artifact
`a7a5b4972a72c56cd07b33001295955596c9e8f7910e177f522ea037b25444ef`.
At that checkpoint the core passed 140/149 functional checks, finalized floors
5/5, weapons 480/480, initialized-Title MT 6/6 and coin/spike phases 863/863.
All eight core transports passed, with 61/149 whole frames exact. Traversal
executed all three routes and retained the hostile-room lock/return failure.
The runner recorded every suite before returning failure.

[Six-suite evidence](../references/isaac/foundation/runtime-progress/verification.json)
retains exact failed fields and source/build/report hashes. The unchanged
`1E0WJX06` and `W0L0LYYM` random tapes remain required regression inputs.

A subsequent eight-case core run at commit `55f850e`, artifact
`6fcd80f941f4e68e02673d64a949e898fc82c876a5f451ae5d9a9ba2002358ae`,
passes 146/149 functional checkpoints. Both fresh OS-random runs and all eight
original/browser transports pass. Whole-frame equality remains 61/149. The
three remaining failures preserve exact HP and former-rock-cell movement
differences in `T1PK6Y0W`; this run precedes ordinary rock destruction.
[Core evidence](../references/isaac/foundation/runtime-progress/history/6fcd80f941f4-core.json)
records each checkpoint and the immutable source commit.

After ordinary Rock integration, the known-room replay and two fresh random
cases pass 61/64 checkpoints on artifact
`61186ab300e0fdd92e172655f1a5a78d1404c95e72238eac921984f0db09022a`.
All three transports pass complete command and committed-seed checks. The
former-rock-cell XY/VXY now match exactly; the three remaining checkpoints
fail only on source HP3 versus port4. This damage gap remains required.
[Targeted evidence](../references/isaac/foundation/runtime-progress/history/61186ab300e0-rock-core.json)
preserves the actual fields without attributing an unproved cause.

Subsequent scoped aggregate checks pass 848 audio checks with 36 actual PCM
starts, 294 curse-door phases and both retained random regressions, 36/36
checkpoints. Strengthened weapon and tear-lifecycle gates pass all 12 scenarios,
including 1,440 complete command comparisons and 4,704 paired player XY/VXY
fields. The 18 RNG-dependent tear differences remain explicit diagnostics.
[Post-checkpoint evidence](../references/isaac/foundation/runtime-progress/post-checkpoints.json)
retains each report hash and its actual build scope.

Reviewed runtime now includes regular-tear range/grid death and removal, rare
Rock factory metadata, free first-floor treasure doors and ordinary bomb-driven
Rock destruction. The rock gate's historical eight-case cohort passes five
cases and retains three explosive/tinted failures across 2,957 compared phases.
Its current default adds the preserved troll-bomb case; every selected case
remains blocking. Reviewed runtime also includes Fly13/AttackFly18 contact,
queued tear/bomb damage, retirement and delayed room clearance. The scoped
contact/bomb/tear actor comparisons pass 9,344 fields; full AI/shared-stream
and center-grid scenarios retain their exact required failures.

Secret blast discovery, reciprocal doorway persistence and minimap flags have
538 paired phases of scoped evidence. Regular and double keys now admit and
collect through real room actors: the four-case cohort passes 1,690 phases,
followed by a contrived/recorded-random NPC-composite pair passing 845 phases.
Ordinary library locks now spend one naturally collected regular key after
actual room clearance, with persistent return/revisit state. The earlier
key-door cohort compares 1,686 phases and retains two Keeper admission failures.
Keeper17/1 now has source-backed constructor and anchored idle behavior; a
separate fresh pair passes 658 phases, including subtype/seed, flags, collision
classes, authored Guy clips and frozen/full/intermediate callbacks. On the
Keeper composite, a fresh key-door pair passes 750 phases. The other original
completes 936 phases after its parent deadline fails; a separate actual browser
comparison passes all 936 against that completed fresh original. The failed
parent report remains failed, and the earlier cached-original attempt remains
diagnostic. [Composite evidence](../references/isaac/foundation/runtime-progress/history/565955530a41-key-doors.json)
records those distinct attempts and the complete 936-state recovery. Keeper
damage/death/drop callbacks remain separate.
[Key evidence](../references/isaac/foundation/keys/npc-composite-live-report.json)
and [NPC boundaries](../references/isaac/foundation/combat/npcs/verification.json)
record each immutable build's scope.

Horf12's final dead Full update now dispatches its AI before retirement while
preserving the living no-APPEAR path. The [scoped Horf evidence](npc-evidence.md#horfs-final-dead-update)
retains 77,793 passing enemy rows and every existing base failure across two
fixed routes and one recorded OS route. The separate natural-death Shop case
retains its failures; direct final-shot/MT, status and default base-kill
qualification remain open.

A fresh full key cohort on the menu/seed-dialog/NPC/secret composite passes
another 1,690 phases, including two new recorded OS-random seeds. Direct free
treasure-room coins now admit without shop pricing; the retained N90GNCJW door
case passes 175 phases while unsupported item329 remains outside that gate.

Public seed-dialog opening, reopening and commitment now follow the source
Character initializer and ordered decoded setter. A fresh six-case cohort
passes 54 checkpoints, 33,696 complete MT words and 582 public commands.
[Seed-dialog evidence](../references/isaac/foundation/seed-dialog-mt/accepted-report.json)
retains the source/build identities and queued-input case. The preserved W0
random tape still fails one tear-lifetime checkpoint; later shared consumers
remain incomplete. Music now implements Title intro/loop, entry-jingle
completion and floor main/layer playback through actual Web Audio. Its
source-hashed four-case cohort passes 816 controller checks and 252 separately
verified output checks; early decoder/prefetch and playback-onset differences
remain explicit. [Music evidence](../references/isaac/foundation/music/live-report.json)
distinguishes those contracts. Item-pool work continues in small PRs.

Normal run admission now uses the existing Character menu seed through the actual decoded setter. Six original/browser pairs pass24 menu checkpoints,14,976 MT words,216 descriptor pairs,4,296 Stats poses and96 public commands. All six post-run shared-MT comparisons remain red diagnostics. [Ordinary-run evidence](../references/isaac/foundation/ordinary-run/README.md) retains the rejected observer attempt and immutable accepted scope.

Fresh Level admission now constructs the actual 135×448 descriptor seed pairs;
six original/browser pairs match 270 bounded pair samples, 14,976 menu MT words
and 96 commands. Later shared-MT differences remain explicit. RoundWorm244 and
its real type9 projectile now have typed construction, full/intermediate
updates, contact/damage and retirement. Their retained cohort passes 6,880
independent projectile checks while keeping 598 direct required mismatches
across 50,900 fields. The wider hostile AI and downstream death effects remain
open. Natural pedestal selection now follows authored pool weights and
depletion: four pairs match 10,296 selection checks and 542 presentations,
followed by 7,588 independent room-context checks on those same captures.
First free acquisition now owns the deferred Pickup queue/grant, all 347 item
counts, authored inventory grants, first active charge and retained empty-altar
motion. A fresh browser recovery against four complete original recordings
passes 70,696 scoped checks; initial and intermediate failures remain retained.
Its separate required acquisition suite captures both games freshly, retaining
the two recorded random active/passive cases alongside two new unfiltered
random cases. Item-specific cache/stat and familiar callbacks remain separate.
Natural regular/double bomb pickups pass 2,727 phases across four retained tapes
and two fresh unfiltered OS-random cases. Troll/Megatroll conversions remain
required when observed and in the retained regressions.

Troll pickup conversions now create real Bomb4/3 actors before generic pickup
animation, with source fuse, motion, animation and retirement. Mega4/4 targeting
and friction are implemented from the binary but remain unaccepted live. The
required conversion gate retains all seven cases: the original five public
tapes, both unfiltered random regressions and two corrected Mega candidates.
The initial five-case proof remains red across 2,212 phases with 3,128 required
mismatches in shared seeds, pickup blast motion and grid restoration. The
disproved KPF Mega expectation is corrected from actual source captures while
its tapes and field checks remain mandatory.

Ordinary 13×14 room admission and reciprocal extended doorway routing now
match 1,843 selected geometry, player, grid, door and minimap phases across
three source tapes. The geometry leaf retains three required missing Chest50
actors. The following concrete Chest constructor leaf passes those same 1,843
phases in a fresh actual browser, including identity, closed subtype,
Appear/Idle clocks, radius and collision classes. The following CameraStyle1
leaf passes 1,843 fresh paired geometry, motion and camera-bound phases, nine
actual viewport checks and complete native histories on all 1,545 tall-room
phases. Ordinary-room histories were not observed. Interior scroll differences
remain diagnostics; complete backdrop/pixel parity remains open. Persistent
Chest RNG words remain statically mapped; opening, loot and save/load remain
unaccepted. The registered large-room gate always launches both games fresh.
[Large-room evidence](large-room-evidence.md) and
[Chest evidence](chest-evidence.md) retain the distinct capture and recovery scopes.
[Camera evidence](large-room-camera.md) records the separate fresh scope.
The authored Familiar catalogue includes Leech, Poof and Heart resources.
The following typed Leech/Poof leaf implements cache admission, actual child
RNG ownership, explicit birth updates, appearance and empty-room owner deadband.
Its four fresh pairs complete 80,878 checks with 25,131 required failures; all
input and source-stability guards pass. Sequence, placement and shared-seed
gaps remain blocking. Ember motion rows compare ordinal candidates when
identity or counts differ and do not establish a same-actor motion defect.
Retained Familiar IDs do not establish room-return update/grid or navigation
parity. [Familiar evidence](familiars.md) retains those scopes.

Ordinary Fire33 now admits real actors, drains queued Tear/Bomb damage,
extinguishes to persistent HP1 and produces independently scheduled Ember66
effects. Three fresh pairs execute those paths; 110,642 own-actor fields pass
while 8,525 of 191,217 required paired fields remain red. The required Fire
gate preserves shared-seed, cooldown, lifetime and physical differences.
[Fire evidence](fireplace-evidence.md) records the bounded accepted scope.

Fresh-run save admission now owns destination Clear, ItemPool copy, supported
Room.Save mutation, all 135 ordinary Level rows and supported Player fields.
The required SavedGame gate checks actual current-row seeds and complete live
grid occupancy. Its corrected six-case comparison remains red on 115 required
fields, with copy ownership passing separately. A separate fresh GridBase leaf
now owns supported starting-room construction and seven-field Room.Save
records. Its six pairs retain 254 required record/seed/occupancy failures while
copy ownership and provenance pass; strict recorded W0 passes 18/18 on that
leaf's artifact. A following normal-shape Ambient leaf admits variable grid
decorations and the actual signed 448-word Room cost owner. Six fresh pairs
match all 2,688 cost words, six complete occupancy sets and 360 semantic record
fields; 228 seed/record failures remain blocking. Its strict W0 case passes
18/18 on the same leaf artifact. Other backdrop/shape decorations and grid callbacks,
post-acquisition Player state and complete saved-world restoration remain open.
The current-room regular Save writer captures the supported manager-ordered
pickup list before PlayerManager Save flushes queued grants and installs it in
the copied SavedRun. Fresh PR #711 base candidate `46affff7` / artifact `2a6cbdf4`
(1,417 files; WASM `f415dc46`) passes build/interface checks, affected selection
52 and the registered fixed `PBW9YNC8` storage test: 28 checks, 18 labels and
217 commands. All four paused room 7 coins now appear in the actual packet;
write/store/menu readmission share its bytes. The retained Bomb-only baseline
remains 27 pass / 1 missing-list failure. The same artifact also passes both
initial-room capacity contracts (35 and 4,062 checks), including own-store
Continue and retaining the older store after an oversized-write rejection.
Later-room Continue, NPC restoration,
Original actor-save packet parity and later-main gameplay remain unqualified.

The public initial-room Save/Continue increment now writes bounded origin-local
slots and reconstructs the world through Main and natural same-origin reload.
On current404a/immutable1b30, fixed moving plus first unfiltered OS DL17GHC7
pass66/66 own-storage guards and two natural reloads; runtime2/2 passes.
Strict paired state is1592/1714 with122 failures:49 retained families per case
plus24 real fixed-context post-Continue Player/tear differences. RGBA34 pass/28
fail and Original/browser shared-MT and continued-body differences remain. Unsupported saved
actors/items/rooms and complete progression remain open.
The listed-Bomb/unknown-legacy guard on0af reuses both recorded Native cases:
all1714 ordered rows,62 Browser PNGs and62 MT states equal1b30. Public Bomb
and real own legacy-save app-upgrade refusals pass; stored bytes remain intact.
The saved-ambient successor137b restores owned1000/21,33,68 records through
fresh Init and saved RNG30, corrects the Full/Fast position boundary and the
Stop Room/Backdrop seed constructor. On the unchanged fixed/OS Original inputs,
all1714 inherited rows remain:1644 pass/70 fail,52 gains/zero lost passes.
The additional2034 shared-MT/ambient rows pass2006/fail28; all48 measured
cursor/624-word checkpoints match. All28 additive failures precede Quit in
fixed-case ambient velocity words; OS507/507 passes. OwnSave66/66, both natural
reloads and real old0af legacy ambient-owner refusal pass; RGBA34/28 retains
all verdicts. Intermediate adfb OS14 lost passes are recovered and its failed
reports remain preserved. Broader saved actors/items/rooms and progression,
14 canonical menu/gameplay-scope errors,28 seed separators and28 penalty-stage
rows remain open.
PR696's Save foundation is merged at `9f952efd`. It removes the 4,096 walking
paint cutoff while retaining the 4 MiB complete-document bound. Capacity
failure produces no partial packet, sends one failed acknowledgement,
preserves older stored bytes and resumes the pending Tick. Exact main715
baseline `7ad918af` and candidate `92dc82b9` pass fresh build/ABI, affected
preview and runtime 4/4 each. Four same-Native Save controls retain all 3,748
ordered rows per role (3,650 pass / 98 fail), zero changed/lost passes and
unchanged pixel metrics; 33 own-save guards pass per control. The 98 failures
are 28 ambient velocity-word rows, 14 canonical scope rows, 28 seed separators
and 28 penalty-stage rows. Earlier Source C native port Rust/C ABI capacity, shipped
WASM differential and Browser over-bound results retain their separate scopes;
complete direct WASM owner bytes remain unobserved and the under-bound Browser
wrapper failure remains retained. The merged current-main native port library
check passes separately; its wrapper's failed label remains preserved.
Fresh public-main134 H/tooling artifact `7ddc18cc` separately passes build/ABI
and runtime 4/4. Its one fixed same-Native Save capture keeps all 2,384 ordered
rows exactly equal to dbfab H: 2,349 pass / 35 fail, zero lost/missing/new or
changed Native expectations, 33 own-save guards and unchanged pixels. Actual
natural reload/Continue passes, with stored seed9/live seed8 and penalty0; the
observed movement follows the valid room5-to-room6 door transition. The suite
still fails its 28 ambient velocity and seven canonical scope rows. No new134
baseline, OS or ordinary replay is claimed; its actual canonical affected
preview selects 52 suites. Publication on later main3b08711d preserves Mega Fatty and unrelated
quality tools without inheriting a later-main gameplay grade.

H's separate snapshotdbfab qualification uses fresh baseline `9e3c05e5` and
shared H/tooling artifact `70cb3285`: fresh build/ABI and runtime 4/4 each pass. The
candidate's actual affected preview selects 52 suites. The same retained fixed Native case improves 2,321 pass /
63 fail to 2,349 pass / 35 fail: 14 seed-text and 14 penalty-stage gains, zero
lost/missing/new rows or changed Native expectations, unchanged pixels and
33 own-save guards per role. The remaining 35 are 28 ambient velocity words
and seven canonical scope rows; both comparison exits remain failed. Both
ordinary-run replays pass on the same retained Native case, including the
candidate's exact nine-character saved-seed guard. Both fixed natural reloads
pass; their fresh boot RNG/epochs differ, and unchanged placement permits the
observed different valid doors. This is no identical-boot counterfactual or
Original-process reload parity. Historical main715 H retains its separately
qualified fixed/OS 56 gains and 42 failures. Raw archive audit corrects its
earlier ordinary/reload-unqualified notes; the old notes remain preserved.
No main134 integration, Factory/Get or whole-game qualification is inherited.
[Save/Continue scope](save-continue.md) preserves exact current and historical
producer/artifact boundaries. [SavedGame evidence](../references/isaac/foundation/saved-game/README.md)
retains the original reports and the correction that made those gaps blocking.

Ordinary backdrop-1 Block3 construction now owns raw1900/0/0 initialization
and its state-1 path cost. A retained public route compares eight actual
Original objects: all 64 observed fields match. The isolated leaf preserves
all 1,190 existing comparisons, 59 Browser shared-MT snapshots and 59 PNGs;
123 canonical and 52 pixel failures remain. Special descendants, restore and
damage/destruction remain separate. [Block evidence](block-grid-evidence.md)
records the accepted constructor scope and next observed prerequisite.

The [bounded TNT common-callback result](tnt-grid-evidence.md) on frozen
e3fa5336/1,408 files (WASM e505236c) passes 41,659 required checks:21,038 fixed
and 20,621 first recorded OS boot 4011211959/idle 8, with fresh Native and actual
Browser captures,401 clock acknowledgments and 40 selected Native captures.
All 22,535 prior passes remain exact with zero losses;19,113 added rows pass,
and 11 invalid live-four guards now pass actual Penny conservation while the
old raw failures remain unchanged. Separate corrected-clock ordinary-room
Node and fresh-Native/DOM controls each pass 278 checkpoints,99,406 inherited
fields and 279 added clock rows, recovering 162 marker failures with zero lost
parent passes. Broader recorded11-rock/six-bomb/four-key controls preserve
all1,687,079 inherited field keys and all passing values;8,811 added absolute-clock
fields pass. The252 known Rock failures remain, with64 already-failing tinted
positions changed against actual137b and no changed expectations.
Transient18 seed/callback is UNOBSERVED; capped history,
fullRGBA, rare50/41 pool/pickup350, mixed-grid order, Player/NPC damage, reentry and
whole MT remain open. This is no whole-game/current-main qualification.


The historical CI matrix discovered thirty-six live suites and ran separate
jobs with `fail-fast: false`, preserving each result and evidence. Agents now
own relevant local regression checks before merging; there is no automated
merge gate. The first historical matrix run
completed floors, weapons, boot MT and pickups successfully while retaining
core and traversal failures. Historical build, typecheck, assets, matrix and strict Clippy
pass at integrated checkpoint `472b154`, including the reviewed Level,
RoundWorm, item-selection, bomb-pickup, large-room geometry, Chest constructor
bounded callback-history, first-acquisition, Troll conversion, Fire/Ember,
shared health helpers, scoped save ownership, default tall-room camera and
authored Familiar resource leaves. Its production artifact is
`01bac4e3fd8e11ecfd400ce205a29e939acba46dfa8d560a3440f9ccbf4c718e`.
The full 24-suite local run at detached checkpoint `cb36018` finished with
14 passing suites and five completed gameplay failures. Floor setup failed before capture;
disk exhaustion interrupted RoundWorm grading and prevented the final Level,
item and bomb-pickup suites from capturing. Partial RoundWorm comparisons also
retain gameplay failures. Separate corrected runs at the same immutable
`cb36018` checkpoint pass five floor, six Level, four item-selection and six
bomb-pickup cases. A fresh four-case Worm rerun remains red: 50,420 required
fields, 1,821 failures and 651/882 passing paired checkpoints. Combining these
separate completed scopes gives 18 passing and six gameplay-failing suites;
the original 14/10 attempt stays unchanged. Later leaves do not inherit those
results, and no complete current-main acceptance is claimed.
The latest completed consolidated build/static pass is main `f79338a6`, with
33 registered suites and 938 production files. Its artifact is
`0f3432b0038e4a9dc36a491f312be05c8999096b153e7ed5d3b2e361f03e784b`;
it includes the reviewed Monstro corrections, Familiar, Grid, Chest and ambient
leaves. The subsequent [run-admission leaf](run-admission.md) has its own
frozen build and four fresh pairs: 1,368 phases and 12,504 checks retain 640
blocking clock/fade/control/age differences. Later integrations do not inherit
the f793 build or a complete live-suite acceptance.
Complete gameplay RNG, other floors/characters, persistence, later music
callers/cutscenes and whole-game parity remain open. GitHub required-check
enforcement is unavailable for this private repository's current plan. The
hosted run at `7783e58` was refused before any job started because GitHub
reported an account billing/spending-limit issue. That refusal is historical.
[Run 38449613521](https://github.com/Anveio/isaac-rebirth-browser/actions/runs/38449613521)
completed on former PR2 head `83228caa`: discovery and 20 suites pass, eight
suites fail and rock-blast is cancelled. Seven failures retain gameplay gaps;
key-doors fails while serializing W31 evidence after 936 comparison attempts,
before its field-failure list and some provenance/backend guards are recorded.
That row's field outcome is unknown; its other case passes 750 phases.
The cancelled rock-blast job retains partial gameplay failures. A streamed
complete-evidence writer is now integrated, with a separate 1,814-snapshot
actual-browser storage proof; the failed hosted attempt remains failed.
[Current evidence scopes](../references/isaac/foundation/runtime-progress/history/20261013-main-live-scopes.json)
pin the historical reports, source heads and distinct artifacts.

## Initial live paired verification after unit-suite removal

The reviewed paired gate replayed all eight recorded tapes against immutable artifact
`d602ad5ecccf9880e204282353cb372e144acaec58c67e4a07bd170a610d48ff`.
All eight fresh original/browser processes, consumed X11 validations, real DOM
traces and seed readbacks completed successfully; the gate source hashes stayed
unchanged during execution. Every capture now requires typed menu or gameplay
observations; gameplay also requires the browser's playing screen. Functional
assertions pass 111 of 149 checkpoints.
That historical CI result is **failure**, with the actual gaps retained:

| Live scenario | Functional checkpoints | Result / observed gap |
| --- | ---: | --- |
| Title/menu/reentry |19/19 | Pass |
| Movement/fire |12/12 | Pass |
| Pause/release |12/18 | Source remains paused at exit start; port resumes/coasts early |
| Moving bomb, even phase |14/18 | Source and port blast/coasting position and velocity differ |
| Moving bomb, odd phase |13/18 | Source blasts at phase120; port blasts one phase later, shifting HP/immunity and motion |
| Adjacent room/rocks/combat |14/28 | Room relocation/collision and player health loss from room combat are absent from the port; enemy HP is not compared |
| Random `KMAG B3XH` |16/18 | Source/port wall motion and active-tear count differ |
| Random `E74E V3YT` |11/18 | Source/port tear count, resumed motion and immunity differ |

All 149 paired frames were compared as strict pixel diagnostics: 61 match exactly;
`exactOriginalPixels` remains false. Pixel failures are separate from functional
CI status. The report, actual original observations and each exact replayable
`case.json`/public tape are under `.cache/foundation/paired-ci-reviewed/`.

Run `npm run test:e2e:paired -- --replay-case ABS_CASE_JSON --output NEW_DIRECTORY`
to reproduce a scenario's recorded seed and controls. To exercise a subsystem
branch's build without sharing its mutable files, add
`--production-directory ABS_WORKTREE_DIST`; the harness snapshots and hashes
that build before launching Chromium. Typecheck, strict clippy and matrix
synchronization pass. No unit/schema suites were added.

## Reference material

The original macOS base archive extraction contains 2,459 resource files: 21 XML,
2,116 PNG, 82 OGG, 207 WAV, 17 OGV, 10 bitmap fonts, three room STB files and three
text files. [The manifest](../references/isaac/manifest.json) records extraction
provenance and hashes; [the Linux manifest](../references/isaac/linux/manifest.json)
pins depot 250903 manifest `367036646469636831`. The installed build also has DLC,
but the reference resources here use the eight base archives only.

Supplementary extraction from those same pinned Linux base archives restores
94 missing authored WAV samples and the secret-room music jingle. All 301 SFX
and 83 music paths referenced by the XML now exist and are pinned individually
in the resource manifest. [The audio extractor](../tools/foundation/extract-audio.py)
records archive/decoder hashes and validates complete references with `--check`;
`npm run assets:check` also requires this coverage. This establishes resource
availability, while playback and event parity require live game evidence.

Packed base assets contain more information than that unpacked inventory:

- [The authored animation decoder](../tools/foundation/extract-animations.py)
  recovers 790 actors from `animations.a`, with zero unknown names, including
  frame durations, pivots, nulls, events, layers, color operations and sheet paths.
  ANM2 definitions are packed cache data, so their absence from the unpacked
  directory does not mean the animations are unavailable.
- `rooms.a` contains all 13 stage sets declared by `stages.xml`, although only
  Special Rooms, Basement and Cellar exist in the unpacked directory. The shared
  decoder's archive helpers recover all of them without changing the references.
- The Mach-O binary has a symbol table with 21,380 records. Inspection identified
  gameplay, generation, persistence and host subsystems by demangled names.
  Duplicate records are present. The code section does not yield usable plain
  instructions at these symbol addresses; names and addresses establish
  subsystem presence, not recovered formulas or Linux code locations.
- The restored Linux executable is stripped ELF, build ID
  `5531b6a920dee0bc89a7725acdf0ef60415d1990`. The existing offline host runtime
  supplies reproducible original launches and independent fixtures. Old setup
  notes saying Steam startup is blocked describe the earlier environment.

The reusable [room decoder](../tools/foundation/decode-rooms.py) verifies the
Linux archive against its pinned manifest and writes structural room catalogues.
Run `python3 tools/foundation/decode-rooms.py --all-base` to inspect all stages,
or add `--output .cache/room-catalog` to preserve private JSON. Its
earlier structural validation covered truncation, malformed fields, unknown
selectors and exact original
byte round trips. The tool does not reinterpret those selectors as runtime
entities or mutate reference files.

Temporary shared analysis lives under `.cache/coordination/system-map/`:
`macho-symbols.json`, all 790 decoded animations, all 13 decoded STB sets, and
the static Linux index/queries. `next-subsystems.md` and its validated 75-function,
97-direct-edge graph map transition, spawn/update, content and persistence seams;
`mt-consumers.json` inventories the shared MT wrappers with callsites and consumers.
These are private working notes; they are not port behavior or reference
replacement images.

## Authored system inventory

Counts describe definitions, including placeholder and internal entries; they
are not playable-content or parity percentages.

| Subsystem | Direct resource evidence | What remains behavioral |
| --- | --- | --- |
| Characters | `players.xml`: 13 definitions, including transformed Lazarus/Judas; Isaac has hp 6, bombs 1 and item 105 | Unlock gating, whether item 105 is equipped on a fresh profile, stat initialization, transformations and random Eden state |
| Entities | `entities2.xml`: 495 definitions with type/variant/subtype, base/stage HP, collision and shadow metadata | AI, spawn rules, physics, damage, champions, animations and interactions |
| Collectibles and trinkets | `items.xml`: 213 passive, 55 familiar, 75 active and 60 trinket entries | Numeric stat effects, cache evaluation, synergies, use behavior and acquisition rules |
| Item pools | `itempools.xml`: 15 pools with weight, decrease and removal metadata | Pool selection, unlock filters, RNG routing, depletion and rerolls |
| Pocket items | `pocketitems.xml`: 33 cards, eight runes and 28 pill effects | Effects, identification, per-run pill assignment and eligibility |
| Rooms and stages | `stages.xml`: 13 stage sets; STB records include room weights, difficulty, doors, weighted spawn groups and interior dimensions | Floor graph generation, shape selection, entity spawning, persistence and room transitions |
| Presentation | 21 backdrops, 223 costumes, 56 boss portraits, 25 boss color entries, 34 giantbook entries, nine nightmares, 19 cutscene entries | Layer timing, selection, camera, animation clocks, lighting and transitions |
| Audio | Source SFX admission/shared-MT selection, 301 verified WAV buffers, 16-voice output, three music streams, authored intro/loop/fade/queue cues, actual callbacks and trusted unlock; [SFX contracts](audio-runtime.md), [music contracts](music-runtime.md) | Early decoder/prefetch and autoplay/decode onset parity, remaining gameplay/layer callers, initial video/voiceover, alias/saturation and complete shared random routing |
| Metagame | 178 achievement entries, 20 challenge definitions and 203 baby definitions | Save state, unlock conditions, completion marks, challenge rules and co-op |

`ambush.xml` and `fxlayers.xml` contain multiple top-level fragments. A single-root
XML parser rejects them as junk after the root. Do not truncate the files at the
first root or silently discard their later definitions.

## Room-resource contract

The [tracked STB decoder](../tools/foundation/decode-rooms.py) consumed every byte
and re-encoded the exact original bytes of every declared room record in
all 13 base stage sets: 5,314 room records and 206,690 weighted spawn entries
with 298 distinct type/variant/subtype tuples. It is structural resource evidence, not proof of native
generation or entity semantics:

```text
file: count u32
room: type u32, variant u32, difficulty u8, nameLength u16, name bytes,
      weight f32, width u8, height u8, doorCount u8, spawnCount u16
door: x i16, y i16, enabled u8
spawn group: x i16, y i16, entryCount u8
spawn entry: type u16, variant u16, subtype u16, weight f32
```

| Authored stage set | Room records |
| --- | ---: |
| Special Rooms | 492 |
| Basement | 572 |
| Cellar | 570 |
| Caves | 528 |
| Catacombs | 528 |
| Depths | 506 |
| Necropolis | 504 |
| Womb | 503 |
| Utero | 503 |
| Sheol | 239 |
| Cathedral | 186 |
| Dark Room | 95 |
| Chest | 88 |

The authored Start Room is Special Rooms type 1, variant 2, difficulty 1,
weight 1, interior 13×7. Its four possible doors are at grid coordinates
`(-1,3)`, `(6,7)`, `(13,3)` and `(6,-1)`. It contains two weighted spawn groups,
at `(10,2)` and `(9,4)`, both type 0, variant 20, subtype 0, weight 1. Grid type
meaning and which doors the generated floor connects still require original
observations. STB interior coordinates differ from the port's outer 17×11
backdrop assembly grid and must not be silently substituted for it.

The room files also contain spawn types such as 1000, 1002 and 3000 that have no
exact entry in `entities2.xml`, and type-zero spawn variants 10, 20 and 30.
Collection subtypes may be zero in templates while the XML describes concrete
variants such as Penny `(5,20,1)`. A catalogue must distinguish template/spawn
selectors, grid objects and concrete runtime entities; a failed exact XML join
must not silently turn a room object into a generic enemy or delete it.

## Runtime subsystem boundaries

| Native symbol family | Port location / present contract | Reachable missing contract |
| --- | --- | --- |
| `Manager`, `Game`, menus | ABI2, causal boot/Profile/Main/Character/seed-dialog initialization, normalized System phases, source music fade and deferred run admission. The owned clock now drives the initial run fade and distinct Player controls field; its four fresh pairs retain640 required differences. | Complete shared consumers, early pause/initial-fade ordering,593660 input reset, other transitions/settings/challenges/continue/game-over and restart |
| `Seeds`, `RNG`, global MT19937 | General seed codec, source run/stage/descriptor streams, authored weighted floor/room draws, independent ItemPool and Player child RNGs, and one shared MT owner for boot/menu/audio/tear/admitted actors | Special seeds, later floors and all missing actual spawn/effect consumers; complete post-run MT equality remains unaccepted |
| `LevelGenerator`, `Level`, `RoomConfig` | Fresh normal Basement generation uses authored weights, source retries/fitting/secrets and complete descriptors; runtime rooms consume their descriptor selectors. Live floors verify full ordered descriptors and169 cells. | Other stages/profiles/challenges/curses and unsupported shapes/selectors; a passing source kernel does not accept its missing gameplay caller |
| `Room`, `RoomTransition`, `StageTransition` | Reciprocal normal/curse/free treasure/secret and ordinary key doors, visited minimap, supported room persistence, tall-room geometry and scoped camera bounds have runtime owners and required public-route gates | Hostile/special lock rules, complete actor reentry, floor exits, alternate door styles, complete transition/camera/backdrop pixels |
| `Entity_Player` | Native f32 movement, phase-aware motion/fire/bomb controls, inventory/health helpers, damage/immunity and an actual controlsEnabled field distinct from native Entity visibility; source cache4/8/16 and scoped passive callbacks retain their separate158-failure paired proof | Other stat/cache aggregation, broader health containers/caps and damage sources, death/transformations and complete Player callback/animation order |
| `Entity_Tear`, `Entity_Projectile`, `Entity_Laser`, `Entity_Knife` | Regular tears own source ground/height/full/fast prediction, ordered birth RNG/audio admission and next-full removal; RoundWorm has a typed type9 projectile | Complete RNG-dependent numerical parity, other weapons, tear Poof/death effects, broader projectile contacts and downstream callbacks |
| `Entity_NPC` | Authored Fly/AttackFly/RoundWorm/Keeper/Fire and ordinary Monstro have typed source admission/update/contact/damage paths. Scoped paired failures remain required; implemented AI is not a whole-NPC parity claim. | Unsupported subclasses/champions, exact shared seeds/trajectories, final-boss rewards/death effects and exceptional spawn contexts |
| `Entity_Familiar` | Leech cache/child270 admission, separate room sequence owner, appearance and real Poof15. Reviewed NPC Poof source now separates cached spawner words from its null linked parent; this correction has no new production/live acceptance. Four earlier fresh pairs retain25,131 mandatory failures despite matching scoped local/sprite/reveal/retirement fields. [Evidence](familiars.md) | Missing spawn/shared consumers and placement/sequence/Poof seeds, path/target/contact/heal, other subclasses, full cache completion and room-return lifecycle |
| `Entity_Bomb`, `Game::BombDamage` | Rising-edge E placement, full/fast regular motion/fuse/blast, inventory debit, self/NPC/grid damage and typed Troll/Mega Troll conversion. Real1000/18+1 child/record order and grid-before-actor fence are implemented; two fixed ordinary-Bomb routes retain290 canonical/80 Bomb passes and add243 child passes (256/292), zero lost passes | All36 child seed/RNG/counter/flip failures remain; counter offset already exists at gameplay-start. StrictRGBA stays14/36 with22 failures. Crater18 paint/records and Dr/Troll/rock/TNT callers are unpaired here; converted seeds, Room11/unknown history, modifiers, complete shared MT, rendering/audio and live-effect Continue remain open. [Evidence](bomb-evidence.md#bomb-and-bomb-rock-effect-children-source-follow-up) |
| `Entity_Pickup`, grid entities | Natural coin/key/bomb actors, collection and scoped persistence; deferred collectible grant and empty-altar motion; Chest constructor/opening source path; actual grid occupancy, rock/spike/door response and fresh signed448-word Room costs | Regular saved readmission and anchor corrections remain separately reviewed; hearts/other families, paid/modifier contexts, Chest loot/live completion, weighted path costs and other grid callbacks |
| `ItemConfig`, `ItemPool`, player item evaluation | Pinned343 configurations/16 pools, independent source RNG owners, weighted depletion/eligibility, natural selection and first queued acquisition/charge. Four frozen-original/fresh-browser acquisition recoveries pass70,696 scoped checks; the default local paired tool captures both games fresh. [Evidence](item-pools.md) | Item-specific cache/stat/costumes, active use/swaps, pocket/trinket effects, broader familiars, progression/paid/override and exceptional selectors |
| Graphics, `ANM2`, backdrop/shadow/FX | Authored sprite playback, actual source f32 transforms and scoped GPU light/shadow masks; supported actor/room renderers consume runtime state | Complete room/backdrop/sprite pixels, broader effects and native projection/composition edges |
| `Music`, sound playback | Paired SFX and Title/run/normal-floor music, source fades/deferred admission, original queued-path/output graph/PCM and actual callback evidence | Early decoder/prefetch lifecycle, alternate floors, later jingles/layer/pitch callers, intro video/voiceover and broader gameplay sounds |
| `PersistentGameData`, `GameState` | Fresh-run SavedGame Clear, ItemPool copy,135 owned Level rows and supported Room/grid/Player save fields. Save/grid gates retain their required seed/record failures. | Durable browser profile/load/continue, complete post-acquisition Player/actor state, achievements/completion/unlocks and co-op |
| Controller/button maps, host input | Actual DOM commands/text and consumed native X11; complete planned input/backend evidence survives capped state history, with explicit truncation flags | Controller/mouse/remapping, focus/device parity, fullscreen and unsupported action keys |

## Original tick and state observation gap

The clock adapter controls main-thread clocks and presented frames, records
consumed X11 input, and can repeat original RGBA output. It explicitly emits
`simulationTick: null` and `fullInternalState: false`. Rendering repetition does
not establish a one-to-one relationship with simulation ticks.

Wrapped Mach-O landmarks include `Game::Update`, `Game::Render`,
`Game::InterpolatePositions`, `Game::RevertInterpolatedPositions`,
`Room::Update`, `Entity_Player::Update`, player movement/shooting controls and
tear height/falling-speed/falling-acceleration setters. In particular, multiple
numeric position copies may reflect simulated, interpolated or projected
state. The existing [position observer](../tools/original/position-observer.py)
tracks bounded read-only f32 hypotheses. Its address matches are not typed
player fields or a complete original state serializer.

Validate a candidate numeric contract with independent random seeds, separate
horizontal/vertical/diagonal tapes, pause and release, and stationary/moving/
coasting shots. For tick calibration, vary virtual frame-time steps while
preserving elapsed time and consumed-input order; include zero, half and double
steps around presentation boundaries. Require observed numeric evolution to
distinguish update cadence from interpolation. Hold out direction/length cases
from fitting, and keep graphical projection and quantization separate from
world integration. A full-state adapter must identify all future-affecting
state and RNG streams before claiming continuation parity.

Fresh continuous original holdout/inheritance evidence lives under
`.cache/coordination/physics/{holdout,inheritance}-case/run/`. Its observed axial
world-position/velocity pairs support advancing position with the previous
velocity, followed by `v' = (v + 0.6 × input) × 0.8803408` at the tested
presentation cadence. Numeric world coordinates project to logical ground
coordinates as `(0.65 × x + 32, 0.65 × y − 46)`; the actor sprite anchor is one
logical pixel above ground Y. Coasting does not snap to zero in those traces.
These are scoped observed numeric relationships, not a full typed native-state
contract or proof that presentation count equals simulation tick count.

The same traces show moving tears around 4.64 logical pixels per presentation
versus stationary 3.25, establishing a velocity-inheritance requirement.
Original right-facing tears cross even the old port's logical right bound 410 and
can survive more than 45 presentations. The old shared player-wall clamp,
fixed 45-step expiry and constant projected Y drift therefore remain inadequate
until independent ground/height/range/steering contracts replace them.
The latest physics integration implements the observed motion law, inherited
velocity and emission origins/order; its integration checkpoint passed all 44
native engine tests. Current projected `+0.125` tear drift and 45-step expiry are
tracked incomplete contracts. Projectile planes now live in a separate
`projectileBounds` descriptor, preserving provisional 70/410/48/222 values
while measured player walls derive from authored floor/radius metadata. Native ground
Y, interpolated render offset, height and falling-speed observations exist, but
global RNG consumer routing and range-end drop remain incomplete in production.
The working evidence and exact source tapes are in
`.cache/coordination/physics/README.md`; original pixel/mask gates remain active.

The physics workstream also mapped concrete restored-Linux RTTI/vtable paths
instead of relying on wrapped Mach-O landmarks. `Entity_Tear` typeinfo/vtable
are at `0x671230` / `0x671270`, with its candidate Update entry `0x542530`;
`Entity_Player` typeinfo/vtable are `0x670c60` / `0x670c90`, with Update
`0x531850`. Static/runtime cross-checks identify common position/velocity
offsets `0x448` / `0x464`, and tear-specific height `0x7d4`, falling speed
`0x7d8`, falling acceleration `0x7dc`, friction `0x7e0` (initial 0.85) and scale
`0x7e4` / `0x7e8`. These are private reference-observation landmarks, not
production addresses or proof that the entire objects are understood. They
unblock bounded typed trajectory observations for the next height/range task.

Native player movement source at restored-Linux entry `0x529750` uses exact
retention `0.8803408145904541` and perpendicular suppression
`0.8944271802902222` (their product matches the port's scoped turn damping).
Fresh directional wall probes independently establish world collision limits
X 70…570 and Y 150…410, projecting to logical X 77.5…402.5 and Y 51.5…220.5.
They correspond to the authored floor plane plus the player's 6.5-logical-pixel
circle radius. The source reversal branch adds
`max(−0.04 × forwardVelocity, 0)` to acceleration 0.6 and is now implemented
following an independent immediate-turn probe. Native collision velocity
response and overall update cadence remain open.

Tear initialization also identifies a separate global MT19937 subsystem. Its
restored-Linux Init path (`0x5423d0`) calls float helper `0x42e850`, which calls
Next at `0x5e4fe0` with a 624-word state and the standard observed masks
`0x9d2c5680`, `0xefc60000`, `0x80000000` and `0x7fffffff`. The helper converts
the unsigned output to f32 with a factor near 2⁻³². Reference index/state
landmarks are `0x8e6b64` / `0x8ed680`; native storage uses eight-byte words with
32-bit MT values. Startup wrapper `0x42e830` calls `time(NULL)` and tail-jumps to
seed initializer `0x5e4e90`. Independent full-state snapshots match the clock
adapter's epoch 1700000000 exactly across distinct run seeds. Menu, room ambient
and tear consumers share this stream: it is separate from run/stage/room and
per-entity xorshift streams. The private consumer inventory has 277 function
owners, with 443 uniform, 717 bounded-integer and 599 raw-uint direct callsites;
these static counts are not executed draw counts.

Production has not yet verified the full boot-clock lifecycle and shared consumer routing.
Native first-shot eye/height observations are source discoveries, not implemented
projectile parity. Replacing the stream with startup xorshift or skipping to a
fitted cursor cannot establish the original routing. Complete-state observations
must include its future-affecting state/index and the menu, ambient and gameplay
draw ordering as well as per-room streams. No native addresses enter production.

The shared MT lifecycle now has concrete state-producing boundaries. Level
construction and GameState construction each initialize 135 room descriptors,
each with 448 grid descriptions and two raw seeds per description: 120,960
draws. GameState Clear repeats that initialization and creates one saved
PlayerManager seed, for 120,961 draws. Startup's three capacity initializations
plus the Clear/backdrop seeds match the full native MT checkpoint at 362,882
draws. Save-menu initialization enumerates profiles 1, 2 and 3 and restores the
previous profile, each invoking Clear. Character initialization generates one
run seed, and stats initialization generates 179 authored pose values. Independent
native state, run-seed and pose observations verify all 624 words at
`362882 + 4×120961 + 1 + 179 = 846906`.

These are initialization consumers with actual generated state, separate from
variable gameplay consumers. Level reset initializes its grid descriptions again;
Room Save initializes only absent cells within the actual room grid, consuming
two words per absent cell. Grid construction consumes two words per object.
Admitted sound events consume one sample-selection draw after ID/loading/cooldown
checks, including a one-sample event; volume is scaled afterward. Conditional
room rendering and update effects still require exact consumer admission and
ordering. Matching a checkpoint cannot justify an arbitrary cursor skip.

## Backdrop contract recovered within its scope

Four independent fresh-profile OS-random original runs match the recovered
backdrop when their observed decoration seed is supplied:
`N4Z8GWV3` (run 1766104246, decoration 4271600350) and `P01MXJW0`
(run 1648477791, decoration 1985222680), followed by held-out `B29ZRLP1`
(decoration 2463833516) and `PNWJRQD3` (decoration 1082003586).
The comparison covers all four floor
and 44 wall quads, every submitted XY/UV vertex, crops and mirrors. Destination
coordinates match; the maximum source-coordinate difference is 0.00003072
pixels and is bounded by one f32 UV ULP plus the native trace's nine-digit
serialization. This numeric-coordinate bound does not relax original RGBA
pixel gates.

The generator now uses decoration xorshift index 35, a repeated floor catalogue
with a general shuffle, and conditional wall-variant draws. Earlier Rust room
and native verifier integrity cases passed before their removal. The historical
report is `.cache/coordination/room-generation/verification-report.json`; current
gameplay acceptance uses the [live paired gate](../tools/foundation/paired-game.mjs).
The [persistent backdrop observations](backdrop-observations.md) record exact
licensed/restored binary hashes, identified Linux routine roles and the four
reference cases for the next generation workstream.
Production still substitutes Stage 1 seed for the decoration seed. Conditional
generation draw routing and source graph observations have advanced, but the
general algorithm is not implemented; FX and other stage variants remain open.
Do not turn observed draw counts into a per-seed offset table.

Static notes additionally identify a 169-cell level topology grid with initial
cell `(6,6)` / index 84, a primary stage-index-35 stream and a secondary
index-2 stream seeded from the stage seed. Conditional placement and template
draws are being reconciled through their actual consumers. This level grid differs
from both STB room interiors and backdrop assembly coordinates.

Two independent source runs now join all 21 observed descriptors against authored
STB records, room RNG triples, shape cell rectangles and enabled door targets,
including two actual initial-room→neighbor transitions. `N4Z8GWV3` traverses
cell 84→71 to native room 1.252; `W2K03EEH` traverses 84→83 to room 1.240 and includes
a 26×14 four-cell shape. The read-only
[descriptor observer](../tools/original/foundation-descriptor-observer.py)
and private `topology-validation-report.json` retain the exact source contracts.
Room config fields are asset stage/type/variant, not type/stage/variant: StartRoom
has header `(0,1,2)`, while ordinary Basement type1 masks that distinction.
Starting descriptor `+0x8148` also differs from current/previous `+0x8188/+0x818c`.
Superseded diagnostic schemas remain clearly labeled. These are independently
verified source joins, not a production topology or transition implementation.

The moved 3BMB GPU-mask contract now has controlled native provenance. Its
legacy capture requested a wall-clock 450 ms hold but actually held about
482 ms, whereas the browser tape used D27/release30. Two new fresh-profile
original captures with consumed key phases 1207 down / 1234 up and endpoint
1263 reproduce all light/shadow RGBA bytes and primitive bounds exactly; their
neutral targets also equal the old fixtures. The native controlled originals
replace only that moved-case reference, retain the legacy timing evidence, and
preserve the exact RGBA gate. This corrects an observed tape mismatch without
asserting original simulation ticks or full hidden state.

## Supplied-input room growth and template selection

The [room-layout evidence](room-layout-evidence.md) records accepted standalone
Rust kernels for rectangular growth/dead-end expansion and weighted authored
template selection. The earlier bounded layout CLI accepted independently
supplied generator or selector inputs and is now ignored research. This does not provide
complete run-seed floor generation, connected gameplay rooms or traversal.

Eight retained original floors verify all 32 ordinary template identities across
all four rectangular shapes. The picker preserves authored order, door-mask
filtering, source f32 arithmetic and mutable weight decay. Two fresh independent
originals, `7X7BGA4Y` and `GS9E16DG`, add 12 exact selector identities. Six Rust
cases and six verifier integrity cases pass; the source observer checks complete
descriptor/grid/template/RNG chains against the actual initial-room log.

Seven retained finalized rectangle sets match the growth kernel, including zero,
one and two dead-end expansions. The strict comparison still fails `MPCMQQN6`:
source special assignment changes a generated 1×2 rectangle at cell 49 into a
1×1 shop at cell 62. Fresh `7X7BGA4Y` matches growth, while `GS9E16DG` retains a
second fitting difference, a generated 2×2 at 56 versus native 1×1 at 70.
Running the verifier with `--contract growth` or `both` reports these as nonzero
failures and still attempts every selected case. Primary stage/profile/challenge
draw routing, special fitting, retries, secret placement and runtime spawning
remain separate open contracts; no captured map or seed-specific offset is used.

## New bomb reference observations

These early observations retain their original runtime and test scope. The later
[ordinary-bomb late-grid correction](#ordinary-bomb-late-grid-response-349)
records its implemented source and separate frozen-artifact qualification.

The bomb workstream captured fresh stationary, escape and continuous-graphics
native runs under `.cache/foundation/bomb-stationary/run/`, `bomb-escape/run/`
and `bomb-exact/run/`. E consumes the starting bomb once, with the displayed
counter changed by relative presentation 2. In these starting-room tapes, the first explosion is observed at
relative presentation 121, after 120 bomb presentations. The authored Pulse
animation has 59 frames; native playback advances at two presentations per
animation frame in these captures. These units describe observed presentations,
not a proven native simulation tick.

A stationary full-health Isaac loses two half-heart units (6→4); moving upward
for 50 presentations escapes without damage. Horizontal probes around 28
logical pixels from the bomb hurt, while probes around 57 and 86 do not.
Those probes alone bracketed the effect. The subsequently recovered
[regular bomb path](bomb-evidence.md) establishes native blast radius 75 plus
fresh Isaac collision radius 10, converted by the observed 0.65 world scale:
the center-distance boundary is exactly 55.25 logical pixels with a strict
comparison, so equality is safe. The implementation now applies the recovered
f32 outward knockback before attempting damage and stores the normal hit's
120-step damage countdown. Eleven focused Rust bomb cases pass, including exact
contact, radial offsets, impulse during immunity, new-run reset and the
independent complete coasting-blast trajectory. The existing port applies blast
impulses before player motion/drag, reproducing the D27/release30/E fixture's
sampled native x/vx at presentations 121, 123 and 133. That is a scoped
presented-step fit, not the source full-phase actor order. Fresh typed admission
lists include the player; later-born bombs update after it in a full phase.
Their impulse advances the player on the following intermediate phase. The
source player update calls base integration, movement control and then DropBomb.

Empty inventory and native pause/fuse captures pass. Four independent Shift
probes, including window-targeted and physical XTest events for each alias,
leave native inventory and HP unchanged. The unsupported port Shift emission
was therefore removed; E is the independently verified placement action.
Native immunity observations show countdown 120 at explosion presentation 121
through zero at 241, matching the port's decrement within this tested cadence.
Floor craters, pushed bombs, variants, hurt visuals and enemy/grid damage remain
explicit gaps. Independent radial/full-state holdouts remain useful; the source
phase scheduler identified below is not yet implemented by the port or exposed
as verified phase delivery by the original adapter.

A later independent native combat-room tape observes blast placement phase 120,
while the starting-room tapes observe 121. Fly HP 3 retains pending damage 60
through phases 120/121, then drains to HP −57/removed at 122. This exposes unresolved
simulation/render phase alignment and deferred damage scheduling. The existing
121-based mask/countdown contracts remain scoped to their controlled starting-room
tapes; they are not a universal original fuse-phase assertion.

Independent bomb presentation comparisons cover 23 original/production public
GL ages and match authored crops, animation phase and nontransparent layer
order, including the fuse fragment during the first 20 explosion ages. Full
sprite pixels remain open: original axis-aligned corners snap at half logical
pixels, while current raw port corners can differ by 0.25; rotated smoke still
has small float-order differences. Original explosion-light submissions have
radius 225 from the authored gradient scale/pivot and a quadratic falloff over
the first 12 observed presentations. The
primitive reports under `.cache/coordination/presentation/`
are scoped sprite evidence, not complete frame parity.

The frozen GPU-mask manifest now selects nine cases: the three neutral seeds,
controlled movement, and coasting bomb phases 121, 123, 127, 131 and 133.
All five new native shadow/light target pairs repeat exactly in independent
fresh profiles. Their persisted evidence binds actual E consumption, complete
paired GL phases, controller/binary/log/target hashes and original repeats.
The unchanged browser gate compares every straight RGBA byte of both complete
512×512 targets with a zero budget. Seven Node integrity cases and eight native
capture guards reject omitted phases, altered actual input, partial/reused GL
frames and changed target bytes. Agreement on these complete GPU targets still
does not establish full sprite/frame pixels or original hidden-state parity.

The integrated frozen production artifact now matches all nine pairs, all 18
complete RGBA targets, exactly. Native public GL queries identified an attached
24-bit depth buffer, `GREATER`, depth writes, clear depth 0 and monotonically
decreasing submitted layer Z. Retaining a real depth attachment changes Mesa's
interpolation path; enabling depth testing without one does not reproduce the
scaled-light texel boundaries. The browser uses matching 24-bit depth semantics
and per-target layer ordering, resets depth on clear, and explicitly declares
`highp sampler2D` to preserve desktop texture-alpha precision during fades.
An independent native shader/geometry/texture replay isolated these two causes;
the query's original repeats remain byte-identical. No geometry epsilon, target
replacement or tolerance change was introduced. Working controls and reports
are under `.cache/coordination/presentation/sampler-proof/`. The retained
[independent native depth contract](../references/isaac/foundation/stage-masks/3bmb-controlled-depth-contract.json)
binds source depth format/function/writes/clear/range/geometry to raw GL, actual
controller input, logs and target hashes. Loader and capture guards reject
missing or altered source fields and provenance.

## Ordinary bomb late-grid response (#349)

The [ordinary bomb implementation](../engine/src/bombs.rs) now resolves class5
Full ground contact at the existing manager fence after NPC contact writes,
including newly admitted bombs and still-active Exploding bodies. The actor's
Full callback owns Base motion, drag and fuse/animation; the manager owns the
late full query. Full response commits physical/presented position and stored
velocity together. Intermediate response corrects only the predicted position
using copied velocity. The separate class6 shot-bomb solver is unchanged.
Both ordinary phases clamp the center to actual Room bounds before sampling.

Frozen candidate `a48aa48ceb5b9ba334bfd9aefe4c0099dcf0782ba695e5233c44d4762f06ac51`
and isolated guarded parent
`524351dbbd078b0fc62980668f633a36474b12e597c8a3a2a4c4affdf1a92781`
(1,407 files each) replay four exact public-input cases against the same
independently validated Native captures, with fresh actual Browser DOM/WASM
captures. All 1,485 bomb-specific fields keep their keys and Native expectations:
four moving-doorway yfields become passing and no prior pass is lost. The
doorway bomb fields improve from 16/20 to 20/20; its canonical checkpoints already
pass 152/152 on both artifacts. The moving-birth clamp is observed; no separate
zero-speed resting-doorway witness or DST execution is claimed.

Across 4,898 canonical state fields, 233 near-wall fields become passing with no
lost prior passes. C7YP dense wall remains 125/125 bomb fields and 42/42 canonical
checkpoints. Center and near-wall Hugger routes remain red: 502/670 and 430/670
passing bomb fields respectively; candidate canonical checkpoints are 99/151
and 84/158.
The near-wall route retains 240 bomb failures and 88 canonical failures whose
actual values change; some absolute errors increase. Its NPC contact occurs
16 presentations before Native, so this accepts the observed late response
without accepting NPC/bomb timing.
Both games' centers stay within bounds on that route; it does not reproduce
the retained DST wall-escape case.

The same complete pickup bank passes 6/6 cases and 2,784/2,784 presentations on
both builds. The same Troll bank retains 4/9 passing cases, 3,388/3,935 passing
presentations, 547 field failures and two unchanged required-Native-readmission
errors. Both complete Rock banks attempt all 11 cases: 9 pass, 2 fail, with
4,136/4,316 passing presentations, 252 field failures and no case errors.
Their statuses, coverage, Native expectations and all remaining failure values
are identical; the 252 failures retain 209 pickup-position and 43 countdown
differences. These failures and guard gaps remain explicit. Actual PNGs and
input/backend traces are retained locally; this functional A/B does not compute
RGBA or establish pixel equality. Historical bomb results above and in
[bomb evidence](bomb-evidence.md) retain their original scope.
The registered [bomb-pickup](../tools/e2e/bomb-pickups.mjs),
[Troll](../tools/e2e/troll-bombs.mjs) and [Rock](../tools/e2e/rock-blast.mjs)
drivers retain their own whole-case results; the existing
[Troll/tinted-rock regression issue](https://github.com/Anveio/isaac-rebirth-browser/issues/485)
remains open.

A separate later integration check on frozen
`bf3036402648adf643b8bfe56aa48ceb786f4c1fdf7e30b32daeb84f57899281`
(1,407 files) replays only the unchanged doorway case: Native provenance and
actual Browser capture pass, with 152/152 canonical checkpoints and 20/20 exact
bomb fields. Input/backend traces and PNGs are retained; RGBA is not computed.
This confirms that case on the integrated artifact, without qualifying the
whole current runtime or any other workstream.

## Source entity and grid boundaries for the next implementation wave

Restored-Linux transition consumers form a concrete chain: transition update
`0x5ddd30` → change `0x5dd1b0` → game change-room `0x550970` → level change
`0x58d900` → load `0x586c90` → room init `0x5cecc0`. Room init selects weighted
entries, resolves template selectors and dispatches entities or grid objects.
Game spawn `0x550a40` uses typed factory `0x555700`, virtual initialization,
position/velocity assignment and room admission `0x5c6940`. Base initialization
stores type/variant/subtype and spawn seed, and seeds the per-entity xorshift
stream with index 30. The source weighted group picker and spawn stream are
separate from global MT; empty groups still consume their preceding random draw.

Room update `0x5d12a0` updates grid cells before the entity manager `0x454df0`,
which traverses the active entity array. Its separate extra player-update loop
runs only when `EnableInterpolation` is disabled; with interpolation enabled,
the fast phase supplies the additional player update. Player grid collision
`0x443d00` follows that update; render interpolation also calls it with a distinct flag.
Independent typed lists corroborate this array includes the player. Ordinary
Fly-room admission is NPCs, player, then later-born bomb; the full update retains
that order. Newly appended births are fenced by the last actor present when the
full update starts. The source does not universally update all nonplayers before
players, and the legacy E121 fit does not establish that ordering.
NPC RTTI, typed AI dispatch and deferred base damage/update consumers identify
current/max HP fields. These static boundaries help isolate scheduling and
collision contracts; they are not complete object layouts or accepted enemy AI.

Physics additionally recovered the actual mixed-rate dispatcher. System loop
`0x5950e0` contains the gameplay branch `0x595490`, which calls game input and
branches on the full/fast phase flag. The config loader explicitly identifies
`EnableInterpolation`, default 1, at the manager's `+0x2dfd4`.
Full phases revert interpolated positions and call game update; intermediate
phases call game fast-update `0x5507f0` → room fast-update `0x5cd920`. Players
update in both phases, while nonplayer simulation updates occur on full phases
and intermediate tear work uses its render/interpolation virtual slot. The
ordinary RoomFast path dispatches Player1 through its actual virtual Update
slot531850 on both presentations; Base animation advancement still branches on
phase. Generic Entity fast callback4446a0 cannot substitute for this Player
call. There is no universal entity tick. Independent typed tear traces retain
215 source phase points across five entropy seeds; production now routes the
typed ground/height/full/fast lifecycle and ordered shared birth calls.
Remaining RNG-dependent differences stay required in its live gate. The original adapter still emits `simulationTick: null` and does not
serialize complete source state.

Pending run dispatch at `0x594a10` also aligns an odd System counter to an even
pre-update phase before gameplay continues; it does not reset the System clock
for each run. The port now exposes a normalized System clock with this run-entry
alignment and uses it for retained title animation and f32 menu fade. Full
gameplay mixed-rate integration and native run-entry phase holdouts remain open;
the presence of a clock field does not establish complete phase parity.

Fresh native combat observations independently enter an authored adjacent room
with RoundWorms and rocks. Typed rock state changes from type 2/state 1/class 3 to
state 2/class 0 after a blast, and passage through the former cell is observed.
Independent vertical and horizontal blast traces recover source ray/cell/line
checks. Native grid index conversion uses 40-world-unit cells with a padded 15×9
grid, distinct from the 13×7 authored interior. The private source traces and
contract tooling are in `.cache/coordination/combat/`. Those early source traces
now have typed grid/NPC runtime consumers and required live combat gates; their
scoped results do not establish complete shared-seed or enemy parity.

Content initialization, collectible configuration, weighted pool depletion,
AddCollectible/use dispatch and persistent unlock/save consumers are mapped to
concrete Linux boundaries. The [item pool selection leaf](item-pools.md) now
routes real run-owned RNG consumers, fresh achievement eligibility, current
weights/maps and natural free pedestals. Selection retains its separate live
gate. First free acquisition now routes the actual Pickup queue/grant, owned
counts, first active charge and retained empty-altar motion. Required acquisition
The default local paired invocation captures both games freshly; scoped frozen-original/live-browser recovery
passes all four recorded cases while preserving initial failures. Item-specific
cache/stat effects, familiars, use behavior and durable progression remain open. Internal Game age uses the actual `ce9f4`
counter rather than a tally of animation callbacks during room loading.

## Introduction and pause presentation verified regions

The presentation workstream's two focused production browser cases pass exact
original comparisons for the stage-banner text (5,760 pixels), pause paper with
stat/cursor widgets (119,880 pixels), and the two-line actual seed (7,168 pixels).
An independent fresh original/browser run with arbitrary seed `0HKFG1LX` also
has zero changed pixels in those regions; its report and original manifest are
under `.cache/coordination/presentation/random/`. This validates the general
authored attachments and bitmap-font layout within those regions. It does not
establish complete stage-entry/loading/fade pixels or unsupported Options/Exit
behavior.

The generic animation math now uses sequential f32 interpolation and transforms.
Earlier component validation compared all 75 original title quads across 25
observed phases and all 42 smoke quads with their alpha values; 38 smoke primitives
are from independently held-out phases. The retained
[original primitive fixture](../references/isaac/foundation/animation-math/primitives.json)
feeds [generic animation playback](../web/animation-math.ts),
and the native scenario registry includes first-title, fade and authored-loop
checkpoints. This establishes the tested generic geometry/alpha contracts;
fresh production title/fade pixels are independently checked, while the legacy
whole-foundation comparison still has its separate title mismatch.

The [native title/fade evidence](animation-evidence.md) also retains ten
byte-identical paired original PNGs for entry, fade, loop and menu reentry.
The read-only menu observer confirms the title animation keeps its actor frame
while another menu is selected; returning does not restart Idle. Three captured
transition checkpoints currently establish retained source state, with complete
transition rendering still pending. Reproducible source tapes and observations
are in the [title reference manifest](../references/isaac/foundation/title-animation/manifest.json).

An immutable ABI 2 artifact with digest
`a98a29f426a8c8f58174b02f31cedf94528b78513c4261486af5e021597e72ad`
matches all seven selected complete original title frames with zero changed
pixels and zero channel error. All ten scoped title/fade state checkpoints also
match. The former browser contract kept the three menu transition captures as
source-state checks until their rendering was implemented. That historical
component suite was removed; current `test:e2e:paired` captures both real games.
The historical focused presentation/generic-GL suite passed five cases. This scoped result
does not replace the legacy references, full gameplay comparisons or original
complete-state requirements.

The reusable [presentation tape](../tools/original/foundation-presentation-scenario.py)
is registered as `run-pause-presentation` in the strict executable original
scenario index. Introduction age, pause appearance/idle/disappearance, cursor
selection and stat-frame state live in the engine snapshot, so pause freezes
gameplay while its menu presentation clock advances independently.

## Historical verification snapshot for the Wave1 checkpoint

Required cloud verification completed against immutable production artifact
`6f87ca918d1e0da3eb191822065a2da5aa4a84038e5dd48839fcae291daff7c7`.
All check groups passed except the final original full-frame gate. That checkpoint executes all eleven selected
checks: native Rust/browser WASM actual-command state equality and five full menu frames
pass; title, gameplay-start, movement, shooting and after-fire remain failures.
The matching menu frames are profile, main, character, seed-entry and
character-seeded. The report's `exactOriginalPixels`, `originalStateParity` and
`originalSimulationTickDelivery` flags remain false.

The frozen runtime passes all 14 compiled behavior browser cases, including all
nine GPU-mask pairs/all 18 complete RGBA targets. The source-discovered Node
suite passes all 43 cases with zero skips, including the strict native depth
evidence guard; native capture and graphics observer integrity suites each pass
eight cases. All 13 room resource sets pass complete structural decoding and
exact byte round trips. The generated matrix agrees with executable source.
These outcomes validate their scoped contracts; the five original pixel failures,
phase delivery and complete native-state gaps still block full parity.

## Historical verification snapshot for Wave2

The shared immutable ABI 2 artifact
`e141553ff314fe5029e2067638e9bdb582c901f7a71721aca731feb60058c1cb`
passes strict clippy, all 75 Rust tests, all 49 Node tests and all 17 browser
tests. Browser coverage includes all nine native mask pairs/all 18 complete
straight-RGBA targets at zero tolerance, seven complete new native title frames,
ten scoped title/fade checkpoints, exact pause/banner regions, independent
boot-epoch replay and held-input regressions. All six layout integrity checks
and the eight-original/32-template supplied-selector comparison also pass.
The growth/special-fitting failures described above remain outside that accepted
selector scope and still fail their strict comparison.

Required cloud verification also passes all six room decoder cases, the native
clock and observer integrity groups, two fresh original clock runs, four fresh
original seed probes and two fresh backdrop probes. The retained combat contract
passes seven integrity checks and all eight cases/100 exact f32 checkpoints.
The final physics evidence hardening was rerun separately: all 16 integrity
checks, 215 exact projectile phases, two complete 624-word MT states and 18
descriptor boundary seed pairs pass. These kernel checks remain conditional on
their declared source inputs; they do not establish runtime routing.

The retained legacy foundation gate still reports 6 passing checks and 5
failures. Its title mismatch decreases from 39,484 to 10,538 changed pixels;
the four gameplay whole-frame failures remain. Its complete menu-frame and
native/browser command-state checks pass. The new controlled title fixtures
and this older comparison are separate evidence contracts; neither result
replaces the other. Current actual results are at
`.cache/foundation/verification/latest/report.json`. Full original state,
simulation-tick delivery, shared MT consumer routing and complete gameplay
parity remain unverified.

## Parallel implementation acceptance

| Independent workstream | Next verifiable result |
| --- | --- |
| Motion and regular tears | Paired read-only numeric/graphics traces for holdout directions and moving/coasting shots; native Rust/WASM continuation matches; original failures remain visible |
| Room generation and decoration | General seed-driven draw routing checked on fresh held-out originals; authored templates/door geometry feed state; no per-seed tile or draw-offset table |
| Bombs and inventory | Extend verified E/fuse/pause/escape/inventory/self-damage to radial holdouts, hurt visuals, pushed/variant bombs and enemy/grid/crater interactions |
| Run introduction and animation | Stage banner/input enablement timing observed across seeds; authored events advance from a run-transition clock and freeze/reset correctly |
| Asset catalogue and combat | Typed authored room/entity metadata, unsupported entities reported explicitly, one observed enemy behavior and combat room with damage/death/door unlocking |
| Renderer and browser verification | Immutable production build actually exercised, exact original GPU masks preserved, all selected pixel/state assertions run, failed regions retain source-indexed investigation tasks |
| Persistence and audio | Real profile reload/continue and browser audio event contracts backed by original observations; browser policy handling is explicit |

Keep the broad subsystem rows in the matrix generator synchronized with these
contracts when implementation or reference evidence changes. Executable browser,
Node and original-clock scenarios continue to be discovered from their source;
do not claim a broad subsystem passes merely because its index row exists.
