# Setup validation — 2026-10-12

The repository and setup configuration are prepared. A managed ChatGPT cloud
environment has **not been created or published**. The available ChatGPT web
session is signed out, and native desktop app control is unavailable.

## Passed on the owner's Mac

- Base-only extraction with the Steam-supplied ResourceExtractor.
- SHA-256 verification of 2,459 extracted base resource files and two native
  binaries; core player, item, entity and stage XML parsed successfully.
- Exact Node dependency installation, TypeScript checking and Vite build.
- Playwright Chromium: an original 384 × 320 sprite sheet rendered through
  PixiJS/WebGL, with WebAssembly, Web Audio and IndexedDB API availability.
- Pinned Python requirements installed; the sprite decoded with Pillow/NumPy,
  player XML parsed with lxml and native Mach-O headers parsed with macholib.
- Private GitHub repository created, uploaded and confirmed private.

## Still required in the managed cloud environment

Run `bash cloud/install.sh`, `bash cloud/verify.sh` and `bash cloud/start.sh` in
the actual setup executor. LLVM inspection, Emscripten compilation, Linux
Chromium rendering and startup readiness have not been verified there.

A local Linux validation was attempted with an official checksum-verified
Node.js 24.21.0 binary in a Debian container. Package installation failed because
Docker could not fetch Debian packages. The separate Dockerfile build also
stalled fetching image metadata. These observations concern local Docker;
they do not establish the managed cloud executor's network behavior.

Follow the configuration and setup request in [SETUP.md](SETUP.md). Verify an
`Environment published` confirmation before treating the cloud environment as
ready, then check a fresh task from that published environment.
