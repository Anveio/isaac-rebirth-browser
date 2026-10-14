import { vector } from '../../foundation/entity-compare.mjs';
// Room-shape spec: no target entity. Enter an ordinary 26x7 room through a
// starting-room door, sweep both axes and shoot, comparing room geometry, the
// player's exact motion and the CameraStyle1 scroll.
export default {
  shape: [26, 7],
  about: 'Ordinary 26x7 rooms: admission, geometry, player motion and CameraStyle1 scrolling',
  count: 2,
  required: ['shape'],
  roomFields: (source, actual) => {
    const player = source.players[0];
    return [
      ['room.boundsTopLeft', source.room.boundsTopLeft, [60, 140]],
      ['room.boundsBottomRight', source.room.boundsBottomRight, actual.room.grid && [60 + (actual.room.grid.width - 2) * 40, 140 + (actual.room.grid.height - 2) * 40]],
      ['player.x', player.position[0] * 0.65 + 32, actual.player.x],
      ['player.y', player.position[1] * 0.65 - 46, actual.player.y],
      ['camera.scroll', vector(source.room.renderScroll), actual.camera && vector(actual.camera.scroll)],
    ];
  },
};
