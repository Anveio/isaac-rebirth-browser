export default {
  type: 12, variant: 0,
  about: 'Horf rooms: appear, delayed aimed shots, damage, death and room clear',
  required: ['admission'],
  fields: (native, actor) => [
    ['projectileCooldown', native.npcProjectileCooldown, actor.shooter?.cooldown],
    ['projectileDelay', native.npcProjectileDelay, actor.shooter?.delay],
  ],
};
