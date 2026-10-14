export default {
  type: 14, variant: 0,
  about: 'Pooter rooms: fly wander, aimed shots facing the player, damage, death and room clear',
  contrived: ['J3D7ZC49:2'],
  count: 2,
  required: ['admission'],
  fields: (native, actor) => [
    ['projectileCooldown', native.npcProjectileCooldown, actor.shooter?.cooldown],
    ['projectileDelay', native.npcProjectileDelay, actor.shooter?.delay],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
