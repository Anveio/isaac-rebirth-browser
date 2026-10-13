// The one registry of live e2e suites. Every suite plays the fresh original
// and the immutable production build through public inputs only; most mix
// contrived cases (fixed seeds/phases that pin known edges) with recorded
// OS-random cases that catch behavior which only works for particular seeds.
//
// Run:     npm run test:e2e -- [suite ...] [--jobs N] [suite flags]
// Replay:  npm run test:e2e -- <suite> --scenarios <run>/<suite>/scenarios.json
//
// Fields:
//   entry  driver script, run under a virtual display
//   args   fixed driver arguments (callers may append or override flags)
//   about  one line describing what the suite proves

const traversal = 'tools/e2e/traversal.mjs';
const helper = name => `tools/original/${name}`;

export const SUITES = [
  { id: 'paired', entry: 'tools/foundation/paired-game.mjs', args: [],
    about: 'Title/menu, movement/fire, pause, bombs and combat rooms; six contrived + two OS-random runs' },
  { id: 'regressions', entry: 'tools/foundation/paired-game.mjs',
    args: ['--replay-cases', 'references/isaac/foundation/runtime-progress/1E0WJX06.json,references/isaac/foundation/runtime-progress/W0L0LYYM.json'],
    about: 'Recorded paired runs that previously regressed' },
  { id: 'floor', entry: 'tools/foundation/floor-e2e.mjs', args: ['--count', '2'],
    about: 'Complete Basement floor generation: descriptors, topology, retries and starting doors' },
  { id: 'boot-mt', entry: 'tools/e2e/boot-mt.mjs', args: ['--count', '4'],
    about: 'Boot-time shared MT state through title and menus' },
  { id: 'menu-mt', entry: 'tools/e2e/boot-mt.mjs', args: ['--menu-navigation', '--count', '4'],
    about: 'Menu navigation draws from the shared MT stream' },
  { id: 'seed-dialog-mt', entry: 'tools/e2e/boot-mt.mjs', args: ['--seed-dialog', '--count', '4'],
    about: 'Seed dialog entry and its MT consumption' },
  // --ambient-grid enables the level-descriptor, saved-game and grid-record
  // checks too; the run, tapes and cases are identical with or without them.
  { id: 'ordinary-run', entry: 'tools/e2e/ordinary-run.mjs', args: ['--ambient-grid', '--count', '4'],
    about: 'No-dialog run admission, level descriptors, Game.Save stores, grid records and ambient Room costs' },
  { id: 'weapons', entry: 'tools/e2e/weapons.mjs', args: ['--count', '4'],
    about: 'Tear cadence, direction and inherited motion' },
  { id: 'tear-lifecycle', entry: 'tools/e2e/weapons.mjs', args: ['--lifecycle', '--count', '4'],
    about: 'Tear birth, flight and death phases' },
  { id: 'traversal', entry: traversal, args: [],
    about: 'Ordinary doorway exit/return and hostile-room locks' },
  { id: 'doors', entry: traversal, args: ['--scenario-helper', helper('door-scenarios.py'), '--count', '1'],
    about: 'Curse and free treasure-room doors' },
  { id: 'secrets', entry: traversal, args: ['--scenario-helper', helper('secret-door-scenarios.py'), '--count', '1'],
    about: 'Bomb-revealed secret doors, entry and return' },
  { id: 'keepers', entry: traversal, args: ['--scenario-helper', helper('keeper-scenarios.py'), '--count', '1'],
    about: 'Keeper construction and anchored idle in secret rooms' },
  { id: 'key-doors', entry: traversal,
    args: ['--scenario-helper', helper('key-door-scenarios.py'), '--state-observer', helper('key-door-observe.py'), '--count', '1'],
    about: 'Key collection, library unlock and revisit' },
  { id: 'large-rooms', entry: traversal, args: ['--scenario-helper', helper('large-room-scenarios.py'), '--count', '2'],
    about: 'Tall-room bounds, doors and camera' },
  { id: 'chest-opening', entry: traversal,
    args: ['--scenario-helper', helper('chest-scenarios.py'), '--state-observer', helper('chest-observe.py'),
      // Public time(NULL) and SFX preference given to both games.
      '--boot-epoch-seed', '1700000000', '--sfx-volume', '1', '--count', '2'],
    about: 'Free normal Chest opening and loot' },
  { id: 'pickups', entry: 'tools/e2e/pickups.mjs', args: ['--count', '2'],
    about: 'Natural authored pickups: admission, animation and collection' },
  { id: 'keys', entry: 'tools/e2e/keys.mjs', args: ['--count', '2'],
    about: 'Natural secret-room keys: admission and collection' },
  { id: 'bomb-pickups', entry: 'tools/e2e/bomb-pickups.mjs', args: ['--count', '2'],
    about: 'Regular/double bomb pickup admission and collection' },
  { id: 'troll-bombs', entry: 'tools/e2e/troll-bombs.mjs', args: ['--count', '2'],
    about: 'Troll/Megatroll conversion and lifetime' },
  { id: 'rock-blast', entry: 'tools/e2e/rock-blast.mjs', args: ['--count', '2'],
    about: 'Ordinary rock destruction by bombs' },
  { id: 'items', entry: 'tools/e2e/items.mjs', args: ['--count', '2'],
    about: 'Item pools and treasure-room pedestals' },
  { id: 'item-acquisition', entry: 'tools/e2e/items.mjs', args: ['--acquisition', '--count', '2'],
    about: 'First collectible acquisition' },
  { id: 'familiars', entry: 'tools/e2e/familiars.mjs', args: ['--count', '2'],
    about: 'Familiar admission, appearance and following' },
  { id: 'npcs', entry: 'tools/e2e/state-pair.mjs',
    args: ['--scenario-helper', helper('npc-scenarios.py'), '--comparator', 'tools/foundation/npc-state-pair.mjs', '--require-from-case', 'npcRequired'],
    about: 'Fly/AttackFly/Gaper construction, AI and contact' },
  { id: 'roundworms', entry: 'tools/e2e/state-pair.mjs',
    args: ['--scenario-helper', helper('roundworm-scenarios.py'), '--comparator', 'tools/foundation/roundworm-state-pair.mjs', '--native-frame-timeout', '120'],
    about: 'Round Worm actors, projectiles and room locks' },
  { id: 'fireplaces', entry: 'tools/e2e/state-pair.mjs',
    args: ['--scenario-helper', helper('fireplace-scenarios.py'), '--comparator', 'tools/foundation/fireplace-state-pair.mjs'],
    about: 'Fireplace damage, extinguish and Ember effects' },
  { id: 'audio', entry: 'tools/e2e/audio.mjs', args: ['--count', '2'],
    about: 'SFX admission and Web Audio output' },
  { id: 'music', entry: 'tools/e2e/music.mjs', args: ['--count', '2'],
    about: 'Music channels, fades and queued tracks' },
  { id: 'browser', entry: 'node_modules/@playwright/test/cli.js', args: ['test'],
    about: 'Browser-only lifecycle checks in tools/tests (focus, input, presentation)' },
];

export const suiteById = new Map(SUITES.map(suite => [suite.id, suite]));
