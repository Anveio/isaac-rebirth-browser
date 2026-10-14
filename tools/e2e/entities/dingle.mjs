export default {
  type: 261, variant: 0,
  boss: true,
  about: 'Dingle boss rooms reached by a recorded route: Scene, spit fan, whistled Dips, slides with poop, splat trail and damage',
  // VZ1FV4J6:churn stands and turns its aim while Dingle whistles Dips that
  // die: by frame 780 it spawns Dips while the room list's length names a live
  // NPC, the handle collision that froze the game before mid-room births took
  // fresh ids.
  contrived: ['G66GMA3N', 'VZ1FV4J6:churn'],
  count: 1,
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.dingle?.counter],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
