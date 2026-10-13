# Bounded live replay evidence

`window.game.reset(seed, { trace: true })` enables actual production command
recording. Ordinary play keeps no history. The trace retains public commands
and generation-scoped actual music backend completion/loop callbacks
independently of full state snapshots. Backend metadata includes its position
after a public command and real audio-device time. Replay inserts those actual
callbacks at the recorded positions; it never synthesizes completions from ticks.

Public command and backend metadata share a 4 MiB encoded-record budget and a
36,000 record limit.
Full state snapshots have a separate
32 MiB budget. Each budget counts JSON record bodies; snapshot headers, array
punctuation and browser object overhead are outside those byte counters.

When state history fills, the retained snapshots remain a prefix. Later actual
input metadata continues recording while its own budget allows. The trace
exposes `inputTraceTruncated`, `stateHistoryTruncated`, separate byte counters,
limits and reasons. The existing overall `truncated` flag stays true whenever
either history is incomplete. Overflow is never reported as complete history.

Input-only paired gates require `enabled === true`,
`inputTraceTruncated === false` and exact equality with their intended public
command tape. Their state comparisons come from independently captured actual
game snapshots at every requested phase, rather than from the capped replay
history. They preserve the full trace and disclose any state-history cap.
Original/browser functional differences still fail those state comparisons.

Gates that replay or compare the entire stored state history must continue
requiring `truncated === false`. Incomplete state history cannot establish a
complete native/WASM replay contract, even when its input tape is complete.
Metadata byte or record overflow stops retention explicitly and also marks
state history incomplete; no commands or snapshots are synthesized.
