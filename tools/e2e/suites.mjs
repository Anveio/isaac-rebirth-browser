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
//   alias   id of the suite this one runs a subset of; excluded from full runs
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
// Every entity spec in tools/e2e/entities/ runs in the one `entities` suite:
// entities-scenarios.py keeps each spec's own room selection and contrived
// routes but shares OS-random discovery, and a room admitting several specs is
// launched once and graded for each. `entity-<name>` is an alias that runs one
// spec alone (the per-entity inner loop); aliases stay out of full runs.
// An alias also names its spec to the comparator, which grades recordings made
// before cases carried entity.specs.
const entities = (only = []) => [
  '--scenario-helper', helper('entities-scenarios.py'), ...only.flatMap(name => ['--helper-arg', '--only', '--helper-arg', name]),
  '--comparator', 'tools/foundation/entity-state-pair.mjs', ...only.flatMap(name => ['--comparator-arg', '--entity', '--comparator-arg', name]),
  '--native-frame-timeout', '120',
];

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
  suite('traversal', traversal, [], 'Ordinary doorway exit/return, hostile-room locks and spider-web crossings'),
  suite('doors', traversal, ['--scenario-helper', helper('door-scenarios.py'), '--count', '1'], 'Curse and free treasure-room doors'),
  suite('secrets', traversal, ['--scenario-helper', helper('secret-door-scenarios.py'), '--count', '1'],
    'Bomb-revealed secret doors, entry and return'),
  suite('keepers', traversal, ['--scenario-helper', helper('keeper-scenarios.py'), '--count', '1'],
    'Keeper construction and anchored idle in secret rooms'),
  // Each game fights its own keys into a fixed window; the trapdoor exit tail
  // is shared (see stage-exit-scenarios.py).
  suite('stage-exit', statePair, ['--scenario-helper', helper('stage-exit-scenarios.py'),
    '--helper-arg', '--contrived', '--helper-arg', 'E1T36YXW-monstro',
    '--helper-arg', '--contrived', '--helper-arg', 'SXREBPLT-larry', '--helper-arg', '--count', '--helper-arg', '1',
    '--comparator', 'tools/foundation/stage-exit-state-pair.mjs', '--native-frame-timeout', '120', '--gl-trace'],
    'Boss-room clear award trapdoor and pedestal, stage transition and Basement II arrival'),
  // The starting bomb damages a cracked doorway into a bedroom (18/19) and
  // scatters wood particles; the door stays shut (bedroom-door-scenarios.py).
  suite('bedroom-doors', statePair, ['--scenario-helper', helper('bedroom-door-scenarios.py'),
    '--helper-arg', '--count', '--helper-arg', '1', '--comparator', 'tools/foundation/bedroom-door-state-pair.mjs'],
    'Cracked bedroom doorway: one bomb damages it and scatters wood particles'),
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
  suite('item-composition', e2e('item-composition'), ['--count', '2'],
    'Public Challenge resources, ordered pedestal grants, cache and weapon bodies/history across fixed phases and recorded OS cases'),
  suite('pockets', e2e('item-composition'), ['--pockets', 'all', '--count', '1'],
    'Card/Pill procedural Init, held collection and spills, copied presentation and sound requests; fixed and recorded/fresh OS public inputs'),
  suite('card-pill-init', e2e('item-composition'), ['--pockets', 'init', '--count', '1'],
    'Card/Pill Init and random IDs through procedural source groups', { alias: 'pockets' }),
  suite('pocket-collection', e2e('item-composition'), ['--pockets', 'collection', '--count', '1'],
    'Card/Pill held records, two capacity spills, presentation and sound requests', { alias: 'pockets' }),
  suite('pill-range-use', e2e('item-composition'), ['--pockets', 'collection', '--pill-use', '--count', '1'],
    'Range Down/Up Pill acquisition, Q consumption, canonical counters, cached Range and response clips; fixed and unfiltered OS epochs'),
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
  suite('entities', statePair, entities(),
    'Every authored entity, room-shape and boss spec in tools/e2e/entities/: admission, AI, damage, death and room state'),
  ...(await entitySpecs()).map(spec => suite(`entity-${spec.name}`, statePair, entities([spec.name]), spec.about, { alias: 'entities' })),
];

// Suites a full run, --list and the matrix cover (aliases run a subset of one).
export const RUNNABLE = SUITES.filter(item => !item.alias);

export const suiteById = new Map(SUITES.map(item => [item.id, item]));

// Logical areas, selected as `npm run test:e2e -- @<group>`. A suite may sit
// in several; every runnable suite sits in at least one.
export const GROUPS = {
  boot: ['paired', 'regressions', 'boot-mt', 'ordinary-run', 'run-admission', 'floor'],
  rooms: ['traversal', 'doors', 'secrets', 'keepers', 'key-doors', 'large-rooms', 'bedroom-doors', 'chest-opening', 'floor'],
  stage: ['stage-exit', 'monstro'],
  pickups: ['pickups', 'keys', 'bomb-pickups', 'hearts', 'troll-bombs', 'chest-opening'],
  items: ['items', 'item-composition', 'pockets', 'pill-range-use', 'item-acquisition', 'stat-cache', 'damage-fire-delay',
    'active-item-use', 'familiars'],
  combat: ['weapons', 'tear-lifecycle', 'rock-blast', 'health-damage', 'player-death', 'blood-start', 'fireplaces'],
  monsters: ['npcs', 'roundworms', 'monstro', 'dips', 'entities'],
  audio: ['audio', 'music'],
};
for (const [group, ids] of Object.entries(GROUPS)) for (const id of ids)
  if (!suiteById.get(id) || suiteById.get(id).alias) throw new Error(`Group ${group} names unknown or alias suite ${id}`);
const ungrouped = RUNNABLE.filter(item => !Object.values(GROUPS).some(ids => ids.includes(item.id)));
if (ungrouped.length) throw new Error(`Suites in no group (add them to GROUPS): ${ungrouped.map(item => item.id).join(', ')}`);
