# Original base Rebirth in the reusable cloud environment

Use the existing isolated checkout at `/workspace/isaac-rebirth-browser`; do not
create a Git worktree unless requested. Keep the environment **Only me** and all
original assets private. Do not publish a public site. The existing network
policy, injected proxies, and CA verification are retained.

## Reproduction and startup

From the executor checkout:

```sh
bash cloud/managed-install.sh
bash cloud/managed-start.sh
```

The installer retains the checksum-pinned Node 24.21.0 container, installs the
existing Python asset/LLVM/Clang/Emscripten/Playwright stack, Python 3.13.9 for
safe depot extraction, and Rust/Cargo 1.99.0 with `wasm32-unknown-unknown`. It
runs `python3 tools/restore-linux-depot.py`, verifying transfer checksums and all
14 original files for app 250900, depot 250903, manifest 367036646469636831.
It also checks `ldd` with the bundled `lib64` directory on the library path.

`cloud/managed.sh` starts the retained container, updates its session-specific
proxy hostname mapping, and passes current upper/lowercase proxy variables on
every command. It preserves the mounted combined CA bundle and TLS checks.
Use it for tools inside the container:

```sh
bash cloud/managed.sh python3 tools/restore-linux-depot.py
bash cloud/managed.sh bash -c 'source .cache/cloud-env.sh; bash cloud/verify.sh'
bash cloud/managed.sh node tools/display-probe.mjs
```

CI browser checks need port 5173 free. Stop only the Vite process for this
checkout before verification, then rerun `cloud/managed-start.sh` afterwards.
Do not assume running processes survive Save and publish or a new task.

## Original launch, capture, and control

One command starts/checks the display, launches the original, captures a frame,
sends Return, and captures another frame **only if a visible game window exists**:

```sh
bash cloud/managed.sh bash tools/original-linux.sh exercise
```

Individual controls:

```sh
bash cloud/managed.sh bash tools/original-linux.sh start
bash cloud/managed.sh bash tools/original-linux.sh capture .cache/original-linux/screen.png
bash cloud/managed.sh bash tools/original-linux.sh key Right
```

The virtual display is `:99`, 1280x720x24, with TCP disabled and Mesa software
rendering. OpenAL uses a null audio device in this headless environment. Game
logs and captures go under `.cache/original-linux`. Original depot files remain
checksum-identical; no executable or Steam API library is patched.

## Verification on 2026-10-12

- Clean checkout safely fast-forwarded to
  `90f98d86198b1b7019d4f64b4399ac783bc9d2fb`; no prior changes were present.
  Separate task files were not accessible or modified.
- Complete `cloud/managed-install.sh`: exit 0. Repeated additive installer: exit 0.
- Depot: all parts verified and all 14 files verified, on executor and container.
- Actual `isaac.x64` dependencies: no unresolved libraries with bundled `lib64`.
  Missing system GLU was installed along with explicit GL/X11/OpenAL packages.
- `npm run check`: exit 0; TypeScript and Vite build passed, 1 browser test passed.
- `bash cloud/verify.sh`: exit 0; Python asset/binary libraries, asset hashes,
  Mach-O inspection, C-to-WASM compilation/execution and 1 browser test passed.
- Rust/Cargo 1.99.0: wasm32 target installed; a native Rust compiler produced
  WASM whose exported addition function returned 42 in Node.
- Xvfb/Mesa: llvmpipe, Mesa 22.3.6, OpenGL core and compatibility 4.5.
- Display probe: actual X screenshots saved as `.cache/display-probe/before.png`
  and `after.png`; `xdotool Right` delivered `ArrowRight` to the probe and visibly
  changed its text/background. These are **probe screenshots, not game frames**.
- Original launch and `exercise`: **exit 53 before any game window**:

```text
[S_API FAIL] SteamAPI_Init() failed; SteamAPI_IsSteamRunning() failed.
[S_API FAIL] SteamAPI_Init() failed; unable to locate a running instance of Steam, or a local steamclient.so.
/bin/sh: 1: /root/.steam/root/steam.sh: not found
```

The remaining requirement is an authentic Linux Steam client installation and
a usable owner-authorized session for app 250900, including the genuine
`steamclient.so` and running Steam IPC/session services. This container has no
Steam client/session. Merely setting Steam app IDs did not resolve the failure;
temporary diagnostic files were removed. Do not substitute patched libraries,
fake Steam services, or describe the display probe as original-game success.

**No original screenshot or original keyboard response was established. No
internal game-state parity is claimed.** Development tooling and display are
prepared for the owner's Save and publish review; original launch remains
blocked. Snapshot restoration in a fresh task has not been tested.
