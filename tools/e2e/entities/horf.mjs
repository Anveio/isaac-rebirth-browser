export default {
  type: 12, variant: 0,
  about: 'Horf rooms: appear, delayed aimed shots, damage, death and room clear',
  // V1XCDXXL door0 admits a native champion Horf (color 10, HP20/contact2).
  // G66GMA3N door0 is room 7: a champion Horf in a rock pocket at (520,200)
  // that tear knockback pushed through the rocks while Horf lacked ground
  // collision (#229).
  contrived: ['V1XCDXXL:0', 'G66GMA3N:0'],
  count: 1,
  required: ['admission'],
  fields: (native, actor) => [
    ['projectileCooldown', native.npcProjectileCooldown, actor.shooter?.cooldown],
    ['projectileDelay', native.npcProjectileDelay, actor.shooter?.delay],
  ],
};
