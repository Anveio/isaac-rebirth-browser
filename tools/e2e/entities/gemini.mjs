export default {
  type: 79, variant: 0,
  boss: true,
  about: 'Gemini boss rooms reached by a recorded route: baby and umbilical cord spawn, FindGridPath chase, rage/pant cycle, tethered baby predictive shots',
  // S1E7BJQ2 and SL9K4GJ3 reach a Gemini room through two quiet rooms;
  // :churn shoots long enough for damage and the baby's shots.
  contrived: ['S1E7BJQ2', 'SL9K4GJ3:churn'],
  count: 1,
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.gemini?.counter],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
