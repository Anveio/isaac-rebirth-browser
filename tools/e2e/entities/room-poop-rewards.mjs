import { pickupFields } from './champion-drop.mjs';

// Grid spec: enter a room whose raw1500 poop rolled golden (variant 3) or
// rainbow (variant 4) and shoot it. Poop Hurt 56a390 makes a golden poop
// ignore two hits in three; breaking it drops five to seven scattered coins
// or trinket 52 (56acc4), and a rainbow poop's colorized particles and
// break (56ae48: GiantBook, sound, full red hearts) draw from the shared
// stream. The grid rows grade each hit's state; every pickup's identity,
// seed and position and the player's hearts are graded at each capture.
export default {
  gridType: 1500,
  shape: [13, 7],
  about: 'Golden and rainbow poop: tougher golden hits, coin/trinket drops and the rainbow break',
  // A rainbow poop just inside the top neighbour's bottom door (FRHDQ6R9:1);
  // a golden poop on the right neighbour's middle row, in the first shots'
  // line, among five Dips (LHS0PPQG:2).
  contrived: ['FRHDQ6R9:1', 'LHS0PPQG:2'],
  count: 0,
  required: ['shape'],
  roomFields: (source, actual) => [
    ...pickupFields(source, actual),
    ['player.redHp', source.players[0].redHp, actual.player.hp],
  ],
};
