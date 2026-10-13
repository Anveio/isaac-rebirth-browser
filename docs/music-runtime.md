# Original music controller and browser output

The runtime reads all 45 authored music IDs and 83 exact OGG resources from
`music.xml` and the pinned original archives. Each file's SHA-256, byte length,
Vorbis rate/channel count and final granule frame count are validated before
playback. Original trailing archive padding remains part of the hash. The
bundler emits exact file bytes through a virtual asset map so the literal `#`
in the secret-room jingle filename does not become an import URL fragment.

The source controller has two alternating main channels and one independent
layer channel, outside the 16 SFX voices. Crossfade changes the old main's
target to zero and disables its current decoder's loop before toggling main
channels. It preserves the queued track's future loop flag. Intro and loop
files retain one logical generation. Play prepares the main first but starts
an authored layer before the main. Main fades use f32 `current + delta * .08`;
layer fades use `.02`, with the source `.001` snap boundary and equal-zero
inactive-channel early return. Volume zero still loads and plays streams.

Browser entry is first Title (61); the earlier native intro video and music100
voiceover tail are outside that entry. Its source-only old channel is retained
as a disclosed scope difference in live evidence. Character confirmation starts
jingle 95 in its first menu dispatch. The source fade adds 0.025 with f32
arithmetic in subsequent dispatches; forty additions remain below one and the
forty-first crosses the threshold. That sets a deferred System request. The
following dispatch aligns the counter and constructs the Level, whose load
immediately admits room music. The earlier Title fade and this confirmation
fade are separate source fields. Room construction therefore waits for that
request boundary rather than a zero-tick DOM key callback.

The queue is consumed when the actual backend stops, then prepares the authored
main and layer introductions. Starting-room layer output remains zero.
Alternate floor generation, later room jingles, count-driven layer admission and special-mode pitch
callers still need source caller integration.

Web Audio shares the SFX context and its trusted keyboard/pointer unlock.
Each music channel owns its gain/output graph and actual source nodes.
Introductions schedule their queued loop at the real device boundary; no
simulation tick or fabricated duration retires a stream. Every terminal node
reports actual completion, including a formerly looping node after its loop
is disabled. Reset, replacement and pitch rescheduling cannot retire a newer
generation. Gameplay pause continues music updates and device playback.

Native streaming refills four 65,536-byte OpenAL buffers. Its decoder queue can
pop before the introduction is audibly finished; browser `onended` reports the
actual device boundary. Their independent real clocks also advance differently
while the source renders hundreds of phases and the browser steps them. Early
decoder/prefetch parity is unimplemented. Raw queue and decoder-loop differences
remain exact diagnostics; completed-playback equality after recorded late holds
does not establish that earlier lifecycle parity. Kernel monotonic stamps retain
native wall time since observed device Play, including observer thread stops;
they are not a native audible playback cursor. Independent browser AudioContext
times retain its actual device clock. Exact queue and loop state is required
after the recorded late audio holds. Actual scheduled
browser loop inputs are separately joined to native queued resource paths and
their shared output graph. Crossfade's real loop setter must reach that same
old loop node and logical generation.

Actual completion and intro-loop callbacks are explicit generation-scoped
engine commands. Diagnostic replay retains their public-command position and
real audio time in the separately bounded input metadata budget. Full state
replay requires complete state history and compares callback states as well
as public-command states. See `replay-traces.md` for truncation disclosure.

`tools/e2e/music.mjs` starts a fresh original and a production WASM browser for
two contrived audible/muted cases plus two recorded OS-random cases. Public
menu, seed, run and pause tapes include explicit 15/6/29-second real audio
holds while presentations stay blocked. These let both actual devices reach
Title loops, jingle completion and floor loops without synthesizing backend
callbacks. The gate preserves native PID/executable/catalogue/library pins,
process manifests, source and build hashes, original PCM uploads, browser API
buffers/graphs, actual input/callback traces and source-only entry differences.
Complete submitted music PCM is compared with an independent decode of the
pinned original resource, and native uploads also verify initial PCM prefixes.
These Vorbis decoder comparisons allow at most one int16 quantization unit;
SFX PCM remains exact. The first causal development pair passed 142 of 145
checks; three early single-presentation Escape pulses did not admit native
pause. That failure is preserved at `.cache/e2e/music-dev/run3/report.json`.
The acceptance tape retains a normal 300-presentation run settle and
12-presentation key holds across both source phases. It also captures the
actual fade threshold and following Level admission separately. Live
acceptance results will be recorded separately.

The frozen four-case acceptance passes all 816 required checks on production
`c974ad2a`. A separately hashed read-only verifier adds 252 passing actual
queued-node and checkpoint-output checks to those same raw captures. Two early
native decoder/prefetch queue mismatches remain exact diagnostics. The same
production build passes the fresh four-case SFX regression, 860 checks.
[Recorded scenarios and pins](../references/isaac/foundation/music/live-report.json)
retain the original capture source commit and the independent verifier identity;
the runnable gate uses these stronger output assertions for future captures.
