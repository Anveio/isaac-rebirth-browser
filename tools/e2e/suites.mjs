// The one registry of live e2e suites. Every suite plays the fresh original
// and the immutable production build through public inputs only; most mix
// contrived cases (fixed seeds/phases that pin known edges) with recorded
// OS-random cases that catch behavior which only works for particular seeds.
//
// Run:     npm run test:e2e -- [suite ...] [--jobs N] [suite flags]
// Replay:  the runner prints the exact command for every failed suite.
//
// Fields:
//   id      suite name used by the runner, CI and docs
//   entry   driver script, run under a virtual display
//   args    default driver arguments (caller flags of the same name replace them)
//   about   one line describing what the suite proves
//   replay  flag that replays the suite's recorded scenarios.json
//           (default --scenarios; paired-game suites replay case.json tapes;
//           false when the driver has no replay input yet)

import { entitySpecs } from './entities.mjs';

const suite = (id, entry, args, about, extra = {}) => ({ id, entry, args, about, ...extra });
const paired = 'tools/foundation/paired-game.mjs';
const e2e = name => `tools/e2e/${name}.mjs`;
const helper = name => `tools/original/${name}`;
// A scenario helper's tapes graded by a state comparator (see state-pair.mjs).
const statePair = e2e('state-pair');
// Multi-room routes from a scenario helper (see traversal.mjs).
const traversal = e2e('traversal');
// One state-pair suite per entity spec in tools/e2e/entities/: OS-random rooms
// admitting the entity, plus the spec's contrived seed/door routes.
const pass = (flag, values) => values.flatMap(value => [flag, String(value)]);
const entitySuite = spec => suite(`entity-${spec.name}`, statePair, [
  '--scenario-helper', helper(spec.boss ? 'boss-scenarios.py' : 'entity-scenarios.py'),
  ...pass('--helper-arg', [...(spec.type === undefined ? [] : ['--type', spec.type, '--variant', spec.variant]),
    ...(spec.shape ? ['--shape', spec.shape.join('x')] : []),
    ...(spec.roomType !== undefined ? ['--room-type', spec.roomType] : []), '--count', spec.count,
    ...spec.contrived.flatMap(route => ['--contrived', route])]),
  '--comparator', 'tools/foundation/entity-state-pair.mjs', ...pass('--comparator-arg', ['--entity', spec.name]),
  '--native-frame-timeout', '120',
], spec.about);

export const SUITES = [
  suite('paired', paired, [], 'Title/menu, movement/fire, pause, bombs and combat rooms; all declared contrived + OS-random runs'),
  suite('regressions', paired,
    ['--replay-cases', 'references/isaac/foundation/runtime-progress/1E0WJX06.json,references/isaac/foundation/runtime-progress/W0L0LYYM.json'],
    'Recorded paired runs that previously regressed'),
  suite('floor', 'tools/foundation/floor-e2e.mjs', ['--count', '2'],
    'Complete Basement floor generation: descriptors, topology, retries and starting doors'),
  // One fresh original per case grades the title-only, menu-navigation and
  // seed-dialog browser modes (--modes boot,menu,seed-dialog).
  suite('boot-mt', e2e('boot-mt'), ['--count', '4'],
    'Boot-time shared MT state at Title, through menu navigation and through seed dialog entry/reopening'),
  // --ambient-grid also enables the level-descriptor, saved-game and grid-record
  // checks; the tapes and cases are identical with or without them.
  suite('ordinary-run', e2e('ordinary-run'), ['--ambient-grid', '--count', '4'],
    'No-dialog run admission, level descriptors, Game.Save stores, grid records and ambient Room costs'),
  suite('weapons', e2e('weapons'), ['--count', '4'], 'Tear cadence, direction and inherited motion'),
  suite('tear-lifecycle', e2e('weapons'), ['--lifecycle', '--count', '4'], 'Tear birth, flight and death phases'),
  suite('traversal', traversal, [], 'Ordinary doorway exit/return and hostile-room locks'),
  suite('doors', traversal, ['--scenario-helper', helper('door-scenarios.py'), '--count', '1'], 'Curse and free treasure-room doors'),
  suite('secrets', traversal, ['--scenario-helper', helper('secret-door-scenarios.py'), '--count', '1'],
    'Bomb-revealed secret doors, entry and return'),
  suite('keepers', traversal, ['--scenario-helper', helper('keeper-scenarios.py'), '--count', '1'],
    'Keeper construction and anchored idle in secret rooms'),
  suite('key-doors', traversal, ['--scenario-helper', helper('key-door-scenarios.py'),
    '--state-observer', helper('key-door-observe.py'), '--count', '1'], 'Key collection, library unlock and revisit'),
  suite('large-rooms', traversal, ['--scenario-helper', helper('large-room-scenarios.py'), '--count', '2'],
    'Tall-room bounds, doors and camera'),
  suite('chest-opening', traversal, ['--scenario-helper', helper('chest-scenarios.py'),
    '--state-observer', helper('chest-observe.py'),
    // Public time(NULL) and SFX preference given to both games.
    '--boot-epoch-seed', '1700000000', '--sfx-volume', '1', '--count', '2'], 'Free normal Chest opening and loot'),
  suite('pickups', e2e('pickups'), ['--count', '2'], 'Natural authored pickups: admission, animation and collection'),
  suite('keys', e2e('keys'), ['--count', '2'], 'Natural secret-room keys: admission and collection'),
  suite('bomb-pickups', e2e('bomb-pickups'), ['--count', '2'], 'Regular/double bomb pickup admission and collection'),
  suite('hearts', e2e('hearts'), ['--count', '2'], 'Natural Heart pickups through the shared collection guard'),
  suite('troll-bombs', e2e('troll-bombs'), ['--count', '2'], 'Troll/Megatroll conversion and lifetime'),
  suite('rock-blast', e2e('rock-blast'), ['--count', '2'], 'Ordinary rock destruction by bombs'),
  suite('items', e2e('items'), ['--count', '2'], 'Item pools and treasure-room pedestals'),
  suite('item-acquisition', e2e('items'), ['--acquisition', '--count', '2'], 'First collectible acquisition'),
  suite('stat-cache', e2e('items'), ['--stat-cache', '--count', '2'],
    'Player stat cache after acquiring Roid Rage and Speed Ball: stats, motion and tear velocity'),
  suite('damage-fire-delay', e2e('items'), ['--damage-fire-delay', '--count', '2'],
    'Sad Onion and Cricket Head damage/fire delay, born tear properties and actual contacts'),
  suite('active-item-use', e2e('items'), ['--active-use', '--count', '2'],
    'Acquiring and using an active item (Yum Heart): charge, health, animation and SFX'),
  suite('familiars', e2e('familiars'), ['--count', '2'], 'Familiar admission, appearance and following'),
  suite('npcs', statePair, ['--scenario-helper', helper('npc-scenarios.py'), '--comparator', 'tools/foundation/npc-state-pair.mjs',
    '--require-from-case', 'npcRequired'], 'Fly/AttackFly/Gaper construction, AI and contact'),
  suite('roundworms', statePair, ['--scenario-helper', helper('roundworm-scenarios.py'), '--comparator',
    'tools/foundation/roundworm-state-pair.mjs', '--native-frame-timeout', '120'], 'Round Worm actors, projectiles and room locks'),
  suite('fireplaces', statePair, ['--scenario-helper', helper('fireplace-scenarios.py'), '--comparator',
    'tools/foundation/fireplace-state-pair.mjs'], 'Fireplace damage, extinguish and Ember effects'),
  suite('monstro', statePair, ['--scenario-helper', helper('monstro-scenarios.py'), '--comparator',
    'tools/foundation/monstro-state-pair.mjs', '--native-frame-timeout', '120'],
    'Level2 Fly route into Monstro: admission, AI, projectiles and scene cleanup'),
  suite('dips', e2e('dips'), ['--count', '2'], 'Dip admission, AI, poop explosions and death callbacks', { replay: false }),
  suite('blood-start', e2e('blood-start'), [], 'Blood effect start-up and background admission', { replay: false }),
  suite('health-damage', e2e('health-damage'), ['--count', '2'],
    'Red/Soul/Black health damage, Hit and Black-heart callbacks', { replay: '--scenario-file' }),
  suite('player-death', e2e('player-death'), ['--count', '2'],
    'Normal Isaac death clip, GameOver freeze and deferred popup restart/exit actions', { replay: '--scenario-file' }),
  suite('run-admission', e2e('run-admission'), ['--count', '2'],
    'Initial run admission: menu confirmation clock anchor and Player control enable timing'),
  suite('audio', e2e('audio'), ['--count', '2'], 'SFX admission and Web Audio output'),
  suite('music', e2e('music'), ['--count', '2'], 'Music channels, fades and queued tracks'),
  ...(await entitySpecs()).map(entitySuite),
];

export const suiteById = new Map(SUITES.map(item => [item.id, item]));
