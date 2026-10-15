// Room-type spec: enter a Sacrifice Room (type 13) through a starting-room
// door and shoot, comparing its grid (rocks, alt rocks, spider webs, the
// centre spikes), room lock state and the player's exact motion.
export default {
  roomType: 13,
  about: 'Sacrifice rooms: admission, alt rocks, spider webs and the centre spikes',
  // Start-room right doors: rocks + spikes (PHHELWL1), alt rocks + webs
  // (HCA0SXCH), fireplaces (RWD2LEA0).
  contrived: ['PHHELWL1:2', 'HCA0SXCH:2', 'RWD2LEA0:2'],
  count: 1,
  required: ['roomType'],
  roomFields: (source, actual) => {
    const player = source.players[0];
    return [
      ['player.x', player.position[0] * 0.65 + 32, actual.player.x],
      ['player.y', player.position[1] * 0.65 - 46, actual.player.y],
      ['player.redHp', player.redHp, actual.player.hp],
    ];
  },
};
