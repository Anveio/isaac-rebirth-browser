export default {
  type: 218, variant: 0,
  about: 'Wall Hugger rooms: invincible wall crawl around walls and obstacles, harmless No-Spikes window, then contact damage',
  // CQHR8NEH start room 6's down door leads to room 8 with two subtype-0
  // Wall Huggers (#154: the original's Hugger touches the player there).
  contrived: ['CQHR8NEH:3'],
  count: 1,
  required: ['admission'],
  fields: (native, actor) => [
    ['flipX', native.spriteFlipX, actor.flipX ?? false],
  ],
};
