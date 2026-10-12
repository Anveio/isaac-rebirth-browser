# Isaac Rebirth browser port workspace

The objective is a browser-based port of The Binding of Isaac: Rebirth using the
owner's installed Steam copy as reference. This initial repository prepares the
development environment; it does not yet implement gameplay.

- Begin with base Rebirth. The local Steam install also includes Afterbirth and
  Afterbirth+, whose overlays must be treated separately.
- `references/isaac/rebirth/resources` contains the base archives extracted with
  Steam's supplied ResourceExtractor. `references/isaac/manifest.json` records
  source archive hashes and the extracted file inventory.
- Keep this repository private. Original game resources are reference material
  for this workspace. Do not publish a public site or release containing them
  without the owner's direction.
- `references/isaac/native` contains x86_64 macOS binaries for static inspection.
  A Mach-O executable is not C/C++ source and cannot be compiled to WebAssembly
  with Emscripten. Assess reconstruction/reimplementation from observed behavior
  and resources; identify any features that remain unverified.
- Use Node 24, TypeScript, Vite, PixiJS, Playwright, Python, LLVM and Emscripten.
  Run `npm run check` for type checking, fixture build and browser capabilities.
  Run `bash cloud/verify.sh` for the Linux toolchain and asset checks.
- Respect the cloud executor's network policy, proxy and injected CA trust.
  Never print credentials or disable TLS verification.
- The cloud service does not include native browser/computer use. Use the
  installed headless Playwright Chromium for browser tests. Observe actual
  Steam gameplay on the owner's Mac when needed.
- The `tools/smoke` page is an environment verification fixture, not the port.

