export default {
  type: 67, variant: 0,
  boss: true,
  about: 'The Duke of Flies boss rooms reached by a recorded route: diagonal bounce, orbiting Attack Flies, tiny flies, big fly, blow, boss death with tiny-fly burst',
  // FTJ9DSE0 and A7HV6H8Z reach a Duke room through quiet rooms and their
  // routes hold in the original; :churn shoots long enough for fly kills,
  // the blow and tiny-fly deaths.
  contrived: ['FTJ9DSE0', 'A7HV6H8Z:churn'],
  count: 1,
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.deathFrame ?? 0],
  ],
};
