The authored base Rebirth music catalogue contains 45 configured IDs and 83
unique OGG resources. `tools/foundation/audio-catalog.py` retains each original
file's SHA-256 and byte length and validates its Vorbis identification header,
channel count, sample rate, page sequence, final granule frame count and bounded
trailing archive padding. It generates `web/audio-data.json`; `engine/build.rs`
also generates the source ID/path/intro/layer/loop table for runtime consumers.
This metadata describes original resources, without claiming playback parity.

The Vite `virtual:original-music` map emits exact original file bytes and returns
bundled URLs. Development uses SHA-addressed resource routes. Authored names
containing spaces or a literal `#` remain catalogue keys; the `#` never becomes
an import URL fragment. WAV loading identifies its separate PCM metadata through
`sampleBytes`, since both WAV and OGG metadata contain frame counts.
