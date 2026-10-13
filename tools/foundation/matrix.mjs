import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { CHECKPOINTS } from './verify.mjs';
import { TASKS } from './systems.mjs';
import { discoverNodeCases, discoverTestCases, discoverOriginalClockScenarios, validateOriginalClockScenarios, ORIGINAL_CLOCK_SOURCE } from './scenario-index.mjs';
const root = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const target = resolve(root, 'TEST_MATRIX.md');
const labels = {
  title: 'Boot to title', profile: 'Choose a fresh profile', main: 'New Run menu',
  character: 'Isaac, normal mode', 'seed-entry': 'Enter observed T1PK 6Y0W seed',
  'character-seeded': 'Return to character screen with committed seed',
  'gameplay-start': 'Enter playable tutorial room', movement: 'Hold D, release, settle',
  shooting: 'Hold Right, release, capture live tear', 'after-fire': 'Release fire and expire tears',
};
const missing = [
  ['original-tick-state', 'Original simulation-tick scheduling and complete hidden-state observation'],
  ['core-gameplay', 'Combat rooms, enemies, health changes, bombs, pickups and item effects'],
  ['all-inputs', 'Equipped active items/cards/pills, drops, map/restart, remapping, mouse and controllers'],
  ['hidden-progression', 'RNG streams, room/floor generation, item pools, devil/angel chances and progression'],
  ['metagame', 'Other characters/modes, profiles, unlocks, achievements and save/continue'],
  ['numerics-performance', 'Observed native floating-point edges, long runs and comparable simulation/render performance'],
];
const sources = ['engine/build.rs', 'engine/src/content.rs', 'engine/src/lib.rs', 'engine/src/seed.rs', 'engine/src/room.rs', 'engine/src/effects.rs', 'engine/src/bin/replay.rs', 'web/engine.ts', 'web/main.ts', 'web/renderer.ts', 'web/menu.ts', 'web/animation.ts', 'web/graphics-contracts.ts', 'tools/tests/foundation.spec.ts', 'tools/foundation/verify.mjs', 'tools/foundation/compare.mjs', 'tools/foundation/compare.test.mjs', 'tools/foundation/verify.test.mjs', 'tools/foundation/seed-invariants.mjs', 'tools/foundation/seed-invariants.test.mjs', 'tools/foundation/seeds.mjs', 'tools/foundation/original-seed-contract.mjs', 'tools/foundation/original-seed-contract.test.mjs', 'tools/original/foundation-random-seeds.py', 'web/compositor.ts', 'tools/foundation/compositor.test.mjs', 'web/png.ts', 'web/gl-menu.ts', 'tools/foundation/png.test.mjs', 'tools/foundation/stage-masks.mjs', 'tools/foundation/stage-masks.test.mjs', 'references/isaac/foundation/stage-masks/manifest.json', 'tools/original/foundation-capture.py', 'tools/original/foundation-clock-run.py', 'tools/original/foundation-clock-gate.c', 'tools/original/foundation-clock-verify.py', 'tools/original/foundation-clock-gate-checks.py', 'tools/original/foundation-clock-verify-checks.py', 'tools/original/foundation-gl-observe-checks.py', ORIGINAL_CLOCK_SOURCE];
export function renderMatrix({ originalClockScenarios } = {}) {
  assert.equal(new Set(CHECKPOINTS).size, CHECKPOINTS.length);
  for (const path of sources) assert.ok(existsSync(resolve(root, path)), `Missing matrix source ${path}`);
  assert.equal(new Set(TASKS.map(task => task.id)).size, TASKS.length, 'System task IDs must be unique');
  for (const task of TASKS) for (const path of task.sources) assert.ok(existsSync(resolve(root, path)), `Missing system task source ${path}`);
  for (const id of CHECKPOINTS) assert.ok(labels[id], `Document new executable checkpoint ${id}`);
  const browserCases = discoverTestCases({ root, pattern: 'tools/tests/**/*.spec.ts' });
  const nodeCases = discoverNodeCases({ root });
  const clockCases = originalClockScenarios === undefined
    ? discoverOriginalClockScenarios({ root, systemIds: TASKS.map(task => task.id) })
    : validateOriginalClockScenarios(originalClockScenarios, { systemIds: TASKS.map(task => task.id) });
  return `# Foundation test matrix

Generated from the executable checkpoints in [verify.mjs](tools/foundation/verify.mjs). Run \`npm run matrix:update\` after adding a checkpoint; \`npm run matrix:check\` rejects drift. The previous port and its scenarios have been removed.

\`npm run check\` builds the Rust/WASM foundation, runs native unit, real-keyboard browser and fresh random-seed original tests, then compares its screenshots against two independent original captures. Every selected check executes and failures remain failures. There are no skip or expected-failure paths.

| Executable checkpoint | Required behavior | Source |
| --- | --- | --- |
${CHECKPOINTS.map(id => `| \`${id}\` | ${labels[id]}; original full-frame and critical-region pixel comparison | [verify.mjs](tools/foundation/verify.mjs) |`).join('\n')}
| \`command-state\` | Every actual DOM key/text/tick command matches native Rust and browser WASM state | [replay.rs](engine/src/bin/replay.rs), [browser tests](tools/tests/foundation.spec.ts) |
| \`pause-release\` | Pause freezes the run; released keys cannot stick | [browser tests](tools/tests/foundation.spec.ts) |
| \`random-seed-invariants\` | New OS-random seeds replay identical movement, every shooting direction, pause and release state in native Rust and WASM | [invariants](tools/foundation/seed-invariants.mjs), [seed bank/replay](tools/foundation/seeds.mjs) |
| \`random-seed-browser\` | Actual keys enter ordinary and manual-seed runs; eligibility, movement, shots and generated room images respond to the supplied seeds | [browser tests](tools/tests/foundation.spec.ts) |
| \`production-entropy\` | Browser boot uses crypto entropy; invalid explicit seed inputs reject without changing state | [browser tests](tools/tests/foundation.spec.ts) |
| \`original-random-seed-contract\` | Fresh random native runs accept the code/value and report the Stage 1 seed predicted by Rust/WASM; every selected bank case executes and has real log/PNG evidence | [adapter](tools/foundation/original-seed-contract.mjs), [adapter tests](tools/foundation/original-seed-contract.test.mjs), [native probes](tools/original/foundation-random-seeds.py) |
| \`original-clock-repeat\` | New random seeds repeat every selected native RGBA checkpoint, public log state and consumed X11 input at identical virtual presentation phases | [clock runner](tools/original/foundation-clock-run.py), [repeat gate](tools/original/foundation-clock-verify.py), [ABI tests](tools/original/foundation-clock-gate-checks.py), [negative tests](tools/original/foundation-clock-verify-checks.py) |
| \`original-graphics-frame-contract\` | Pair captures with complete public-graphics frames; reject nested/incomplete/reused frames, post-presentation cache draws and unavailable geometry | [observer regression tests](tools/original/foundation-gl-observe-checks.py) |
| \`original-stage-compositing\` | Actual DOM inputs produce full512×512 straightRGBA light/shadow targets equal to independent original targets, across three seeds and a movement case | [browser cases](tools/tests/foundation.spec.ts), [mask comparator](tools/foundation/stage-masks.mjs), [native fixtures](references/isaac/foundation/stage-masks/manifest.json) |
| \`authored-mask-raster\` | Authored XML/PNG shadow dimensions feed general effect descriptors; every light/shadow has valid signed integer raster corners, compared in native Rust/WASM state and original GPU masks | [content extraction](engine/build.rs), [content catalogue](engine/src/content.rs), [effects](engine/src/effects.rs), [semantic guards](tools/foundation/seed-invariants.mjs) |
| \`comparison-guards\` | Wrong/blank graphics, missing references and missing state cannot pass | [compare tests](tools/foundation/compare.test.mjs), [reference tests](tools/foundation/verify.test.mjs) |

Actual browser cases below are discovered from the executable browser scenario sources; adding or removing one requires regenerating this matrix.

| Executable browser case | Source |
| --- | --- |
${browserCases.map(({ name, source }) => `| ${name} | [${source.split('/').at(-1)}](${source}) |`).join('\n')}

Node cases follow the same source glob as \`test:verification\`. The index rejects disabled or undocumented dynamic registrations.

| Executable verification case | Source |
| --- | --- |
${nodeCases.map(({ name, source }) => `| ${name} | [${source.split('/').at(-1)}](${source}) |`).join('\n')}

Original presentation scenarios below are discovered by executing [foundation-clock-scenarios.py --list](tools/original/foundation-clock-scenarios.py). Their captures, input events and interval counts come from the actual validated tapes. The index rejects missing required scenarios, malformed tapes and unknown system tasks. These are original-only reference scenarios; their presence does not assert that the browser already matches them. Replay a seed with \`python3 tools/original/foundation-clock-scenarios.py --scenario ID --seed CODE\`, then run the emitted capture command.

| Executable original scenario | Required behavior / systems | Tape checkpoints / intervals | Source |
| --- | --- | --- | --- |
${clockCases.map(scenario => `| \`${scenario.id}\` | ${scenario.description}; \`${scenario.systems.join(', ')}\` | \`${scenario.captureLabels.join(', ')}\`; ${scenario.presentedFrames} presentation intervals, ${scenario.inputEvents} input events | [foundation-clock-scenarios.py](${scenario.source}) |`).join('\n')}

The original uses seed **T1PK 6Y0W**, fresh profiles, and a 960×540 window. Captures and observed log/input metadata live in [baseline](references/isaac/foundation/baseline/manifest.json) and [repeat](references/isaac/foundation/repeat/manifest.json). They are comparison evidence; the renderer composes authored sprites and never draws a captured game frame. [Capture](tools/original/foundation-capture.py) and [save-reference.py](tools/foundation/save-reference.py) refresh these fixtures from the runnable original.

Pixel gates compare decoded RGBA over the whole frame, HUD, actor and projectile regions. Any tolerance is measured from the independent native captures; it is not a freely chosen similarity threshold. Reports distinguish exact pixels from native capture variability. Legacy screenshot fixtures use wall-clock keys/SIGSTOP. The newer clock adapter controls presented frames and main-thread time, disables X11 autorepeat and records actual consumed key events; neither adapter proves native simulation ticks or hidden-state parity. Native process CPU/RSS and browser draw/encoding timings remain separate.

## Divergence tasks by game system

Generated from [systems.mjs](tools/foundation/systems.mjs). Each failed image region links to these investigation tasks, state fields and next tests. A region can have several candidate causes; attribution is not proof. Fix the responsible system and rerun its reference scenario before closing a task. Do not substitute a renderer offset, per-seed tile map or screenshot-derived correction image for a missing system.

| Task | Owning system / state | Required next test |
| --- | --- | --- |
${TASKS.map(task => `| \`${task.id}\` | ${task.system}; \`${task.fields.join(', ')}\` | ${task.nextTest} |`).join('\n')}

Production behavior is seed-independent: seeds enter a general codec and generators. Seed-specific expected outputs belong only in reference fixtures. Invariants use new OS-random seeds, record those values for replay, and verify the same tape in native Rust and WASM. Randomness does not excuse missing original comparisons or permit seed-specific implementation branches.

## Required extensions

These remain missing original-observed scenarios, not implemented behavior or passing tests. Add a setup, input tape and assertions to the executable verifier before claiming coverage.

| Missing area | Required scenarios |
| --- | --- |
${missing.map(([id, text]) => `| \`${id}\` | ${text} |`).join('\n')}

Extend the [input catalogue](tools/verification/input-catalog.json), [Rust commands/state](engine/src/lib.rs), [asset renderer](web/renderer.ts), [authored animation loader](web/animation.ts) and [browser scenario](tools/foundation/verify.mjs) together. Register discoveries before implementing them, keep failing comparisons, and code to green. Do not replace a reference with clone output, weaken a pixel gate, omit a selected checkpoint, or call native Rust/WASM self-equality original-game parity.

Latest actual results, state, pixels, differences and phase timings: \`.cache/foundation/verification/latest/report.json\` and checkpoint directories. Declared coverage here is not a cached pass result.
`;
}
export function checkMatrix({ path = target, rendered = renderMatrix() } = {}) {
  assert.equal(readFileSync(path, 'utf8'), rendered, 'TEST_MATRIX.md is stale; run npm run matrix:update');
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv.includes('--write')) {
    writeFileSync(target, renderMatrix());
    console.log('Updated TEST_MATRIX.md from executable foundation checkpoints and scenarios');
  } else {
    checkMatrix();
    console.log('Foundation checkpoint matrix and source agree');
  }
}
