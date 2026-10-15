export default {
  type: 239, variant: 0,
  boss: true,
  about: 'Grub chains in the Larry Jr. boss room 1026: pooled chain HP, head charge, chain death spawning Chargers, Spitties and Maggots',
  // Room 1026 (four Grub segments beside a five-segment Larry Jr.). The
  // playtest's blocked seeds YXDKBRE4/YAVTWZ2L reach it only through hostile
  // rooms, so the quiet-route seed 1219DJFZ stands in (NBAAKG30's route drifts in the original); :churn shoots long
  // enough for the Grub chain to die. 1026 is one of nine Larry templates
  // (about one quiet route per ~5000 OS-random seeds, beyond the boss
  // budget), so OS-random fuzz for this room comes from entity-larry.
  contrived: ['1219DJFZ', '1219DJFZ:churn'],
  count: 0,
  fields: (native, actor) => [
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
