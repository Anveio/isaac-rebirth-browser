# Start the private Isaac Rebirth development environment

Use the existing checkout at /workspace/isaac-rebirth-browser. Each task is isolated; do not create a Git worktree unless explicitly requested. Read AGENTS.md, docs/linux-depot-transfer.md and docs/original-linux.md.

Keep privacy Only me and preserve the existing network policy, injected proxies and combined CA trust. Do not disable TLS verification or publish a public site. Read cloud environment status and /etc/codex/network-policy.json before network access.

1. Run from /workspace/isaac-rebirth-browser: bash cloud/managed-start.sh
   This starts the retained isaac-rebirth-dev container, repairs the current session proxy hostname mapping, passes current uppercase/lowercase proxy variables, checks Xvfb :99 and software OpenGL, starts the development server, and checks its Isaac HTML response on port 5173. Live processes do not survive publication reliably.
2. If the container or tools are missing, run the saved install_script (also cloud/managed-install.sh), then repeat startup. This retains Node 24.21.0, LLVM/Clang/Emscripten/Playwright, Python asset tooling, Python 3.13.9 depot restore support, Rust/Cargo 1.99.0 and wasm32-unknown-unknown. It restores and checksum-verifies every transfer part and all 14 original depot files.
3. Use bash cloud/managed.sh COMMAND... for all container commands; it preserves current proxy/CA settings and activates Cargo PATH. Asset tools use .venv/bin/python. Source .cache/cloud-env.sh for LLVM 14 Mach-O inspection and LLDB compatibility.
4. Original launch/capture/control in one command:
   bash cloud/managed.sh bash tools/original-linux.sh exercise
   Individual commands: start, capture [path], key KEY... . Captures require a visible original-game window.
5. IMPORTANT remaining launch blocker: independently verified exit 53 before any original-game window. SteamAPI_Init()/SteamAPI_IsSteamRunning() fail; neither a running Steam instance nor local steamclient.so is available, and /root/.steam/root/steam.sh is absent. Authentic Linux Steam client installation and a usable owner-authorized app 250900 session, including genuine steamclient.so and Steam IPC/services, are required. No original screenshot/input response or internal game-state parity has been verified. Do not patch/fake Steam services or treat display-probe success as game success.
6. Display-only validation: bash cloud/managed.sh node tools/display-probe.mjs. Before/after actual X screenshots go to .cache/display-probe; Right must produce ArrowRight and a visible probe change. This is explicitly not the original game.
7. Full browser/toolchain verification needs port 5173 free. Stop only this checkout's Vite process, then run:
   bash cloud/managed.sh bash -c 'source .cache/cloud-env.sh; CI=1 npm run check; bash cloud/verify.sh'
   Restart with bash cloud/managed-start.sh afterward.

Current instance validation on 2026-10-12: complete installer exit 0; depot 14/14 verified; ldd with bundled lib64 has no missing libraries; npm run check and cloud/verify.sh exit 0 (one browser test each); C and Rust WASM probes execute successfully; Xvfb/llvmpipe OpenGL 4.5 and actual display screenshots/input probe pass. Original exercise fails accurately with exit 53. No separate task changes were modified. Original launch remains blocked; fresh-task snapshot restoration remains unverified. The owner handles Save and publish after review.
