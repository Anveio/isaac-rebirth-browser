export default {
  type: 239, variant: 0,
  boss: true,
  about: 'Grub chains in the Larry Jr. boss room 1026: pooled chain HP, head charge, chain death spawning Chargers, Spitties and Maggots',
  // The playtest's blocked Basement runs: both seeds reach 1026 (four Grub
  // segments beside a five-segment Larry Jr.); :churn shoots long enough for
  // the Grub chain to die.
  contrived: ['YXDKBRE4', 'YAVTWZ2L:churn'],
  count: 1,
  fields: (native, actor) => [
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
