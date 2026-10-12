# ChatGPT reusable cloud environment

Official workflow: https://learn.chatgpt.com/docs/environments/cloud-environments

## Configuration

- Name: `Isaac Rebirth Browser Port`
- Repository: `Anveio/isaac-rebirth-browser`, branch `main`
- Privacy / Who can use: `Only me`
- Runtime: Node.js 24 and Linux; Python 3 via apt.
- Install script: `bash cloud/install.sh`
- Start skill: the instructions in `cloud/START_SKILL.md`
- Internet access: Package managers preset.
- Additional allowed domains: `nodejs.org`, `cdn.playwright.dev`,
  `playwright.download.prss.microsoft.com`, `cdn.jsdelivr.net`.
- No Steam credentials, VPN or cloud identity are required. Original references
  are in the private repository selected for checkout.

## Setup request

Paste this into the environment setup conversation after selecting the repo:

> Set up this repository for a browser-based port of The Binding of Isaac:
> Rebirth. Use Node 24. Run `bash cloud/install.sh`, which installs the pinned
> Node and Python dependencies, Playwright Chromium, LLVM/Clang/LLDB,
> Emscripten/Binaryen, CMake/Ninja and FFmpeg. Preserve the executor's proxy and
> CA trust. Check `references/isaac/manifest.json` and keep base Rebirth separate
> from DLC overlays. Record `bash cloud/install.sh` as the install script and
> `cloud/START_SKILL.md` as the start instructions. Use package-manager internet
> access with the additional download hosts listed here, and keep privacy Only
> me. Run `bash cloud/verify.sh` and `bash cloud/start.sh`; report the actual
> results and any failures before publishing. This request prepares the
> environment; it does not ask you to implement gameplay yet.

## Finish and verify

Open Settings > Codex Cloud > Environments > Create environment. Select the
private repository and Get started. Supply the setup request above, review the
report and configuration, and Publish when the checks pass.

Only an `Environment published` confirmation establishes that the reusable
environment is ready. A local Docker check is supporting evidence, not proof of
managed-cloud networking, repository access or readiness. After publication,
start a task from the environment and verify `bash cloud/verify.sh` again.
