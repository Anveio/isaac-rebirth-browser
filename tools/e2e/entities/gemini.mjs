export default {
  type: 79, variant: 0,
  boss: true,
  about: 'Gemini boss rooms reached by a recorded route: baby and umbilical cord spawn, FindGridPath chase, rage/pant cycle, tethered baby predictive shots',
  // SL9K4GJ3 and DN379LBR reach a Gemini room through quiet rooms (S1E7BJQ2's
  // room 10 rock ring defeats the router);
  // :churn shoots long enough for damage and the baby's shots.
  contrived: ['SL9K4GJ3', 'DN379LBR:churn'],
  count: 1,
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.gemini?.counter],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
