# Living Restart

This source slice implements the configurable action16 hold for living normal
Isaac at normal difficulty in Challenge0. The actual Keyboard binding feeds the same public action
owner as the other controls. Every eligible Full/Fast Game input callback counts;
release resets the count, hold60 leaves the old run, and hold61 applies counter21
before selecting the requested seed. Pause and incoming fades add no guard.
Menus, Nightmare System4 and notification dispatch do not run this Game callback.
Other characters, hard mode, Challenges and dead R retain the existing explicit
capability failure. The independently supported GameOver Confirm route is separate.
The notification manager keeps the actual fresh Popups default true. Configuring
or persisting that setting remains unsupported; Options and its V4 preferences
retain their existing contract.

The old Game finishes the requesting interval. The next eligible System interval
flushes a dirty profile, clears the old saved run and constructs a fresh run. These
are separate actual storage writes and acknowledgements; none advances the public
clock again. A clean inhibited typed-seed run needs Clear/initial Save only. An
unseeded run selects one runtime-owned shared random word at the request, before
the old Game tail. Typed seeds preserve their owned request string. New admissions
replace the previous context, and Stop resets the hold before returning to Main.
The public and Browser Context code is eight compact seed glyphs. The Native
System request retains the source formatter's nine-byte string, with one ASCII
space after the fourth glyph; each is checked against its declared representation.

Profile version2 retains collected-history ownership. Genuine version1 documents
retain their achievement/counter arrays and acquire unknown collected-history
entries; absent ownership does not become fresh false. Notification metadata is
generated from the owner's local Base resources under ignored
`web/generated/unlock-notifications/`; generated actors/catalogues and original
assets remain private. Direct Cargo callers prepare the normal private Caves and
notification inputs first. The ordinary npm build hooks prepare notifications.
Current Save tools explicitly require version2 packets and347 history entries.
Historical version1 captures remain bound to their archived producer. The migration
capture below uses a real version1 writer, natural same-origin version2 document
load, public Quit/write and reload; its observed legacy arrays were all false/zero.

```sh
python3 tools/foundation/decode-rooms.py --prepare-private-caves
python3 tools/foundation/prepare-unlock-notifications.py
npm run check:build
npm run test:browser -- tools/tests/runtime.spec.ts --grep 'living Restart'
npm run test:e2e -- controls --living-restart --no-build --production-directory FROZEN
```

The Controls mode declares all inputs before either game: three fixed typed-seed
controls cover opposite phases and pause, plus one first unfiltered eight-byte OS
seed/boot draw. It requires menu exclusion, hold60/release reset, hold61 pending,
the next dispatch, continued hold and release in the fresh run. Native observations
are read-only outputs from the canonical directly owned child. Physical inputs,
clock/provenance, actual Keyboard, queue, hold and Persistent owners remain
mandatory. `--scenarios BANK` replays without entropy; `--case` selects a declared
subset after validating and retaining the whole bank.

## Qualified captures

Merged [PR768](https://github.com/Anveio/isaac-rebirth-browser/pull/768) has the
same tree as the reviewed 36-path candidate on `9b34608a`. Its frozen production
artifact is `05fe0fc566faeba841ebfe59e539dc7e32e4e530306136219343ddd33bf27631`
(1,600 files). A fresh build and all six Browser runtime checks pass. The runtime
checks include actual DOM seeded/unseeded R and real own-storage acknowledgements;
that Browser result does not qualify the unseeded Native request draw.

Fresh Native processes and actual Browser replay the unchanged fixed phase-0
case and recorded OS seed `0YY1FRN4`: 288/288 required checkpoints and
19,988/19,988 scalar rows pass. Both cases validate Native provenance, 188 actual
clock acknowledgements and 145 capture times, plus Browser input/error guards.
All 19,988 preceding v8 expected values, actual values and statuses remain exact,
with no missing rows or lost passes. Every pixel metric and verdict also remains
unchanged: full-frame diagnostics retain 20 passes and 268 failures. The whole
four-case input bank is validated and retained before selecting these two cases;
this is a fresh two-case qualification, without a new random draw.

The first v9 runtime attempt aborted before completing tests after concurrent
source preparation materialized redundant reference copies and crossed the TMP
reserve: the monitor recorded 2,034,208,768 bytes and a later observation reached
1,821,241,344, below the 2,181,038,080-byte floor. The first R2 attempt refused
both cases before either game when its new compiler cache reduced actual
HostSHM to 4,208,967,680 bytes, below 4 GiB. Those failures remain retained.
Only byte-verified redundant copies and closed new compiler caches were released.
The final physical-snapshot runtime and fresh pair kept all reserves above their
floors. Complete raw, Source, built code and failures are retained privately;
lossless cold storage has a full filesystem restore proof.

## Earlier evidence and open scope

The predecessor's full four-case R bank passes 576/576 checkpoints. Its initial
125 seed-format reader failures remain preserved beside the corrected reader;
that correction changes neither actual Native bytes nor runtime behavior. The
historical SourceD2 Browser positives retain their separate artifact binding.

Earlier strict paired Save still has 35 fixed-case and 7 OS failures. Prior
actual main07aca/schema1 controls preserve all 3,748 rows with zero lost passes;
66 own-save guards, two natural reloads and 28 regular-writer checks pass on that
predecessor. A real schema1 writer → natural same-origin schema2 load → public
Quit/write → second reload preserves all three profiles and 347 unknown history
entries each. The observed legacy achievement/counter arrays were all false/zero;
nonzero migration remains unqualified. These results do not establish fresh
Save/migration acceptance on the v9 artifact.

The old v7 TMP dip to 1,991,516,160 bytes below its 2,181,038,080-byte floor remains
recorded. Both earlier natural NewRun first-grant failures also remain: the
600-presentation Gemini bound and the later Dingle death. General/dead/hard/
Challenge R, other characters, the unseeded Native request draw, full shared MT,
broader Persistent/notification behavior, natural R reload and full pixel/audio
equality remain open. Existing functional comparisons stay mandatory and every
pixel mismatch remains a diagnostic failure. Advancing main does not transfer
this artifact's qualification.
