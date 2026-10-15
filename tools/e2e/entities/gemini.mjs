export default {
  type: 79, variant: 0,
  boss: true,
  about: 'Gemini boss rooms reached by a recorded route: baby and umbilical cord spawn, FindGridPath chase, rage/pant cycle, tethered baby predictive shots',
  // DN379LBR reaches a Gemini room through quiet rooms and its route holds
  // in the original (SL9K4GJ3 and 6AQ8KAWZ drift there; S1E7BJQ2's rock ring
  // defeats the router). :churn shoots long enough for the baby's shots.
  contrived: ['DN379LBR', 'DN379LBR:churn'],
  count: 1,
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.gemini?.counter],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
