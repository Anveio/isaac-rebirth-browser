// Grid spec: enter an ordinary room that authors spider webs (1940) through a
// starting-room door and shoot. Web Init 571c70 draws three shared MT words
// (Swaying frame, flip, rotation) per web at admission, and AddSlowing slows
// the player crossing one, so the player's exact motion is compared.
export default {
  gridType: 1940,
  shape: [13, 7],
  about: 'Spider-web rooms: web admission draws and the slowed player motion',
  // Eight webs and no NPCs behind the start room's door slot 3.
  contrived: ['B73EC3HH:3'],
  count: 1,
  required: ['shape'],
  roomFields: (source, actual) => {
    const player = source.players[0];
    return [
      ['player.x', player.position[0] * 0.65 + 32, actual.player.x],
      ['player.y', player.position[1] * 0.65 - 46, actual.player.y],
    ];
  },
};
