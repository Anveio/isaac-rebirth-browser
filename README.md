# Isaac Rebirth browser port

Base Rebirth reconstructed in Rust/WASM, verified live against the original
game. Incomplete. Rules for contributors: [AGENTS.md](AGENTS.md).

```sh
bash cloud/install.sh && bash cloud/start.sh  # http://localhost:5173
npm run check                                 # build + every live suite
npm run test:e2e:paired -- --replay-cases PATH
npm run matrix:update                         # sync TEST_MATRIX.md
```

| Subsystem | Root |
| --- | --- |
| Simulation engine (crate, systems) | [engine/src/lib.rs](engine/src/lib.rs) |
| Native diagnostic binaries (floor, replay) | [engine/src/bin](engine/src/bin) |
| Browser runtime, renderer, audio, WASM bridge | [web/main.ts](web/main.ts), [web/engine.ts](web/engine.ts) |
| Live original/browser e2e suites | [tools/e2e/run.mjs](tools/e2e/run.mjs) |
| Paired gameplay gate, comparators, extractors | [tools/foundation](tools/foundation) |
| Original-game runners and observers | [tools/original](tools/original) |
| Browser (Playwright) checks | [tools/tests](tools/tests) |
| CI | [.github/workflows/live-games.yml](.github/workflows/live-games.yml) |
| Toolchain and dev server | [cloud](cloud) |
| Progress site publishing | [tools/sites](tools/sites) |
| Private reference assets and captures | [references/isaac](references/isaac) |
| System map and per-system evidence | [docs/system-map.md](docs/system-map.md), [docs](docs) |
| Scenario coverage and open parity | [TEST_MATRIX.md](TEST_MATRIX.md), [tools/foundation/systems.mjs](tools/foundation/systems.mjs) |
