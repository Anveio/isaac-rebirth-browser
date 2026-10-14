# Original sound admission and browser playback

The browser plays pinned base Rebirth WAV files from authoritative engine sound
events. Original sound IDs, sample counts and paths come from `sounds.xml`;
`web/audio-data.json` also preserves all authored music records and asset hashes.
`assets:check` verifies complete resource coverage and the generated catalogue.

The restored Linux executable is pinned at SHA-256
`724b9c59ad4dc7964852a51728c0f76d2031a3dd3d0d6ba2dcade1faa7ea7d13`.
Source `SystemManager::PlaySound` (`0x594530`) dispatches its ordinary mode to
the SFX table at SystemManager + `0x12c20`. Admission (`0x5e1980`) rejects unsigned
IDs above 323 and unconfigured records, loads samples before testing cooldown,
and uses the signed system counter shifted right one bit. A request is blocked
only while a nonnegative `nextAllowed` is greater than that half-counter.
Admission stores the half-counter, wrapping half-counter + requested delay, and
requested volume, then takes exactly one shared bounded draw for the selected
sample, including one-sample sounds and muted requests. All base sample weights
are one. The selected wrapper receives requested loop/pitch and
`min(globalSfxVolume * requestedVolume, 1)` gain.

`State.audio.admit` owns that sample draw through `State.bootLifecycle.shared`.
Browser autoplay, resource output and voice allocation cannot suppress or
duplicate it. The current command's admitted events are transient; sequence IDs
prevent duplicate browser playback. Continuation validation retains all 169
configured slots, 303 sample settings, cooldowns, pending requests and sequence
state. Virgin native `lastTick` bytes are uninitialized history and are never
treated as 169 startup sound events.

The browser verifies every WAV file hash and parses its actual PCM16 frames
into a Web Audio buffer. The 301 unique samples are mono, 44.1 kHz PCM16. Native
configured sound backends share authored path identities, including IDs 33/34
and 170/218. The native pool (`0x610780`, `0x6109e0`) has 16 ordinary voices and
allocates the first free voice. Configured SFX Play (`0x62a2b0`) stops its own
playing backend before starting it again. The browser retains this path
ownership and restart rule; pool exhaustion does not change engine admission.
Voice completion follows the audio device clock rather than simulated ticks.

A trusted key or canvas pointer gesture resumes the browser AudioContext.
Before unlock, admitted sounds use the same bounded, suspended voice graph.
Reset stops old voices and resets event consumption. Pause menu entry does not
suspend audio: fresh audible original traces contain no `alSourcePause`, and
streaming music continues while the original pause controller is active.
Current SFX volume is configurable through the engine `audioVolume` command;
this setting controls subsequent admissions. The browser default is one and
does not claim to reproduce a particular saved profile's volume.

Run the scoped live gate with:

```sh
npm run build
bash tools/original/with-display.sh node tools/e2e/audio.mjs --count 2
```

The default cohort contains audible repeated-menu and muted-admission cases,
plus two recorded OS-random run seeds, boot epochs and balanced character tap
pairs. Every case enters its public seed in both games and checks native/browser
seed readback. A parent-owned, stopped original observer validates configured
paths/counts/loading and source sound slots without writing memory. A forwarding
OpenAL observer records actual native calls and PCM. An independent forwarding
browser observer records actual AudioBufferSource start/stop, submitted PCM
frames, gain/pitch/loop and forwarded node connections. Every actual sample
start must reach that AudioContext's destination through the observed graph.
The gate verifies the complete production WASM input
trace, original process identity, library/source/build hashes and bounded traces.
Reports, tapes, screenshots, submitted PCM and source snapshots remain under
`.cache/e2e/audio/**`; staged `game/` resource symlinks are excluded from evidence archives.

The reviewed four-case cohort passes 848 checks with 36 original/browser sample
starts (3 confirmations per case, 4 taps per contrived case and 8 per random
case). These menu sounds each have one sample. Variant SFX153's source admission
callback belongs between the discarded tear initializer uniform and final
falling uniform; projectile work wires that caller separately. Alias switching,
pool saturation, remaining gameplay sound callers and music intro/loop/layer
controllers are not claimed by this menu gate.

Complete shared MT state matches at Title and Profile. Source Profile-to-Main
initialization changes the MT cursor by 531 modulo 624, including sound8.
The total draw count is unresolved: the saved-state Clear loop alone takes
120961 draws, so a cursor delta cannot establish executed draw count. Causal
menu routing remains incomplete in the broader lifecycle. Every later full
624-word comparison remains an explicit failing diagnostic; stable character
admissions must each advance both streams by one draw. No native RNG state is
injected into browser input, and no compensating draws hide those differences.
