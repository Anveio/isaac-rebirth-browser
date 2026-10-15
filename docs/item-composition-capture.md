# Public resource capture

The composition adapter runs an explicit Challenge 1 menu tape with declared
raw Challenge XML and an optional complete Special room catalogue. It writes
those bytes to a private resource view, linking every other original resource
and the original executable unchanged. The browser must independently receive
the same declared resources and boot epoch; native observations never supply
its seed, items, stats, RNG or actors.

After installing the original runtime, an existing schema-1 bank can be
prepared without launching the game:

```sh
python3 tools/original/composition-run.py --scenarios /path/to/scenarios.json --case CASE --output /path/to/new-output --prepare-only
```

A real reference capture runs under the existing display wrapper:

```sh
LP_NUM_THREADS=2 bash tools/original/with-display.sh python3 tools/original/composition-run.py --scenarios /path/to/scenarios.json --case CASE --output /path/to/new-output --archive-evidence --observe-every-step --state-observer tools/original/composition-observe.py
```

The adapter resolves and verifies the sole original binary pin from
`tools/e2e/lib/original.mjs`. The observer independently validates its directly
owned process before reading blocked-gate memory. It performs no state or code
writes. D9C and Knife4A0 eligibility reads are explicit observer context; the
latter does not infer a child chain from Parent498.

The bank declares `name`, unsigned `bootEpochSeed`, `initialPhaseOffset` (0 or
1), `fixture` and `actions`. The fixture names `authoredResourcePath`, optional
`specialRoomsPath`, `challengeId: 1`, ordered `startingItems` and any explicit
`attributeChanges`. Every action has a unique simple `label`, public `keys`
and 1..400 `presentations`. `requires.sourceExpectedActorContract` describes
the required source actor family; a declaration is not proof that it executes.
The adapter retains every real run-confirm/admission presentation and all
subsequent gameplay gates, including fixed observation-only PNG steps 3 and 4.

`--challenge-run` is separate from manual seed entry and the Return-only
ordinary mode. It excludes both modes and rejects Tab, then checks the actual
generated run code/value after the public Challenge confirmation. The native
manifest and committed journal preserve actual inputs, acknowledgements and
observations. Missing fields, unsupported actors, partial captures and failed
comparisons remain failures; syntax and preparation do not establish parity.

The reusable browser capture/readback and registry driver consume this same
resource/tape contract. Full-frame pixels are a separate diagnostic; no crop,
mask or fabricated actor value can stand in for an unmet functional witness.
