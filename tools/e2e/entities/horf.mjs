export default {
  type: 12, variant: 0,
  about: 'Horf rooms: appear, delayed aimed shots, damage, death and room clear',
  // V1XCDXXL door0 admits a native champion Horf (color 10, HP20/contact2).
  contrived: ['V1XCDXXL:0'],
  count: 1,
  required: ['admission'],
  fields: (native, actor) => [
    ['projectileCooldown', native.npcProjectileCooldown, actor.shooter?.cooldown],
    ['projectileDelay', native.npcProjectileDelay, actor.shooter?.delay],
  ],
};
