export default {
  type: 19, variant: 0,
  boss: true,
  about: 'Larry Jr. boss rooms reached by a recorded route: segment linking, head random walk, body trail follow, tail poop, splat trail and segment deaths',
  // WMAJ8HGV reaches its Larry Jr. room (1025, one 9-segment chain) through
  // one quiet room; :churn stands and turns its aim for 960 frames so
  // wandering segments die, split the chain and run the boss death.
  contrived: ['WMAJ8HGV', 'WMAJ8HGV:churn'],
  count: 1,
  fields: (native, actor) => [
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
