#!/usr/bin/env bash
# Source this in install, startup and verification; retain proxy/CA variables.
task_repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -f "$HOME/.cargo/env" ]]; then source "$HOME/.cargo/env"; fi
if [[ -d "$task_repo/.cache/toolchain/root" ]]; then
  task_node="$(command -v node)"
  task_tools="$task_repo/.cache/toolchain/root"
  # Keep selected Node 24 ahead of Debian's dependency-only Node 20.
  export PATH="$(dirname "$task_node"):$task_tools/usr/lib/llvm-19/bin:$task_tools/usr/share/emscripten:$task_tools/usr/bin:$PATH"
  export LD_LIBRARY_PATH="$task_tools/usr/lib/x86_64-linux-gnu:$task_tools/usr/lib/llvm-19/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  export EM_LLVM_ROOT="$task_tools/usr/bin"
  export EM_BINARYEN_ROOT="$task_tools/usr"
  export EM_NODE_JS="$task_node"
  export EM_CACHE="$task_repo/.cache/emscripten-cache"
  export EM_FROZEN_CACHE=0
fi
if [[ -x "$task_repo/.venv/bin/python3" ]]; then
  export PATH="$task_repo/.venv/bin:$PATH"
fi
