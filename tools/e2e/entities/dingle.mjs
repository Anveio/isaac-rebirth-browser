export default {
  type: 261, variant: 0,
  boss: true,
  about: 'Dingle boss rooms reached by a recorded route: Scene, spit fan, whistled Dips, slides with poop, splat trail and damage',
  contrived: ['G66GMA3N'],
  count: 1,
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.dingle?.counter],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
