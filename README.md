# Isaac Rebirth browser port

Base Rebirth reconstructed in Rust/WASM, verified live against the original
game. Incomplete. There is no CI: each agent runs `npm run check:build` and the
live suites its change touches before merging. Rules: [AGENTS.md](AGENTS.md).

```sh
bash cloud/install.sh && bash cloud/start.sh  # http://localhost:5173
npm run check                                 # build + every live suite
npm run test:e2e -- --list                    # registered live suites
npm run test:e2e:paired -- --replay-cases PATH
npm run matrix:update                         # sync TEST_MATRIX.md
python3 tools/foundation/decode-rooms.py --prepare-private-caves  # before raw cargo
```

| Subsystem | Root |
| --- | --- |
| Simulation engine (crate, systems) | [engine/src/lib.rs](engine/src/lib.rs) |
| Native diagnostic binaries (floor, replay) | [engine/src/bin](engine/src/bin) |
| Authored content pipeline | [docs/content-pipeline.md](docs/content-pipeline.md) |
| Level framework (stage nodes, chapters) | [docs/levels.md](docs/levels.md) |
| Antithesis DST harness (only unit exception) | [engine/dst](engine/dst), [antithesis](antithesis), [tools/dst](tools/dst), [docs/dst.md](docs/dst.md) |
| Browser runtime, renderer, audio, WASM bridge | [web/main.ts](web/main.ts), [web/engine.ts](web/engine.ts) |
| Live original/browser e2e suites | [tools/e2e/suites.mjs](tools/e2e/suites.mjs), [tools/e2e/README.md](tools/e2e/README.md) |
| Paired gameplay gate, comparators, extractors | [tools/foundation](tools/foundation) |
| Original-game runners and observers | [tools/original](tools/original) |
| Browser (Playwright) checks | [tools/tests](tools/tests) |
| Playtest bundle | [tools/playtest](tools/playtest) |
| Lint and format upkeep | [docs/lint-upkeep.md](docs/lint-upkeep.md) |
| Toolchain and dev server | [cloud](cloud) |
| Progress site publishing | [tools/sites](tools/sites) |
| Reference cases and original evidence | [references/isaac](references/isaac) |
| Roadmap: subsystem status and triaged bugs | [docs/roadmap.md](docs/roadmap.md) |
| System map and per-system evidence | [docs/system-map.md](docs/system-map.md), [docs](docs) |
| Scenario coverage and open parity | [TEST_MATRIX.md](TEST_MATRIX.md), [tools/foundation/systems.mjs](tools/foundation/systems.mjs) |
