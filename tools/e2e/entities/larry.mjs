export default {
  type: 19, variant: 0,
  boss: true,
  about: 'Larry Jr. boss rooms reached by a recorded route: segment linking, head random walk, body trail follow, tail poop, splat trail and segment deaths',
  // WMAJ8HGV reaches its Larry Jr. room through one quiet room.
  contrived: ['WMAJ8HGV'],
  count: 1,
  fields: (native, actor) => [
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
