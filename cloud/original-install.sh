#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
test "$(id -u)" = 0
apt-get update
apt-get install -y --no-install-recommends \
  libglu1-mesa libgl1 libx11-6 libxxf86vm1 libxrandr2 libxi6 libopenal1 \
  xvfb mesa-utils libgl1-mesa-dri x11-utils xdotool imagemagick ripgrep strace
# Python 3.12+ is required by the depot's safe tar extraction filter.
.venv/bin/python -m pip install --disable-pip-version-check uv==0.9.5
UV_NATIVE_TLS=true .venv/bin/uv python install 3.13.9
py="$(UV_NATIVE_TLS=true .venv/bin/uv python find 3.13.9)"
ln -sfn "$py" /usr/local/bin/python3
python3 tools/restore-linux-depot.py
if ! test -x /root/.cargo/bin/rustup; then
  curl --fail --silent --show-error https://sh.rustup.rs -o /tmp/isaac-rustup.sh
  sh /tmp/isaac-rustup.sh -y --profile minimal --default-toolchain 1.99.0
fi
export PATH=/root/.cargo/bin:$PATH
rustup toolchain install 1.99.0 --profile minimal
rustup default 1.99.0
rustup target add wasm32-unknown-unknown
depot="$PWD/references/isaac/linux/depot_250903"
LD_LIBRARY_PATH="$depot/lib64" ldd "$depot/isaac.x64" > .cache/original-ldd.txt
if rg -q 'not found' .cache/original-ldd.txt; then cat .cache/original-ldd.txt >&2; exit 1; fi
bash tools/original-linux.sh display
