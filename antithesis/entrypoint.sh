#!/usr/bin/env sh
# The simulation needs no services: signal readiness, then idle while Test
# Composer runs the drivers in /opt/antithesis/test/v1/rebirth-engine.
set -eu
rebirth-dst --setup-complete
exec sleep infinity
