export default {
  type: 237, variant: 1,
  boss: true,
  about: 'Gurglings boss rooms reached by a recorded route: walk, axis charge with creep, Stop releasing Attack Flies, blood splats, boss death',
  // The playtest's blocked seed GNT8W183 reaches 5142 only through hostile
  // rooms; B2PBST1V walks a quiet route in. :churn shoots long enough for
  // charges, creep contact and kills.
  contrived: ['B2PBST1V', 'B2PBST1V:churn'],
  count: 1,
  fields: (native, actor) => [
    ['stateFrame', native.npcStateFrame, actor.maggot?.counter],
  ],
};
