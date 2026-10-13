#!/usr/bin/env sh
# One whole-engine game per invocation; the tape also replays through the
# shipped engine.wasm so platform findings speak for the browser artifact.
exec rebirth-dst --entropy antithesis --units game --cases 1 --steps 3000 --wasm /opt/rebirth/engine.wasm
