# Isaac Rebirth browser workspace

Development environment for a browser-based port of The Binding of Isaac:
Rebirth, using the owner's Steam installation as reference. Gameplay is not yet
implemented. The environment smoke page exercises PixiJS/WebGL, an original
sprite, WebAssembly and browser API availability.

## Included

- Node 24, TypeScript, Vite, PixiJS, Vitest and Playwright Chromium.
- Python asset tools: Pillow, NumPy, lxml, Capstone, macholib and pyelftools.
- Linux analysis/build tools: LLVM/Clang/LLDB, Emscripten/Binaryen, CMake/Ninja,
  binutils and FFmpeg.
- Base Rebirth resources extracted from Steam's base archives, plus the macOS
  Rebirth executable for static inspection. Afterbirth/Afterbirth+ overlays are
  excluded from this base reference set.

## Prepare and run

```sh
bash cloud/install.sh  # Linux with Node 24
bash cloud/start.sh    # Vite listens on port 5173
bash cloud/verify.sh   # assets, binary inspection, C -> WASM, browser check
```

For a reproducible local Linux check:

```sh
docker build -f cloud/Dockerfile -t isaac-rebirth-cloud .
docker run --rm -p 5173:5173 isaac-rebirth-cloud
```

For ChatGPT's reusable cloud environment, follow [cloud/SETUP.md](cloud/SETUP.md).
The installed game executables are x86_64 Mach-O binaries; Emscripten is available
for new/recovered native code, not direct conversion of the existing executable.

Keep this workspace private. Game assets remain reference material in the
development environment. No public deployment is configured.

