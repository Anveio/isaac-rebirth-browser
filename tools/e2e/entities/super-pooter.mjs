export default {
  type: 14, variant: 1,
  about: 'Super Pooter rooms, mostly Pooters promoted by the spawnConfig 5c5240 alternate-variant roll: identity, paired shots, damage and death',
  // MAJH9R4E: start room 5's down door leads to difficulty-1 room 11, where one
  // of two authored Pooters rolls variant 1.
  contrived: ['MAJH9R4E:3'],
  count: 1,
  required: ['admission'],
  fields: (native, actor) => [
    ['projectileCooldown', native.npcProjectileCooldown, actor.shooter?.cooldown],
    ['projectileDelay', native.npcProjectileDelay, actor.shooter?.delay],
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
